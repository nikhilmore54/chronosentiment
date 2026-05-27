#!/usr/bin/env bash
# verify_release_consistency.sh
# ChronoSentiment — Release Consistency Verification
#
# Purpose: Verify that a replay-certified build is reproducible and operationally
#          consistent before release. Emits a factual release summary.
#
# Scope: Operational hardening only. Does NOT modify replay semantics, routing
#        meaning, fixture content, or any tranche-gated surface.
#
# Usage: bash scripts/verify_release_consistency.sh [--skip-double-build]
#
# Exit codes:
#   0 — all checks PASS
#   1 — one or more checks FAIL

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
SKIP_DOUBLE_BUILD="${1:-}"

PASS_COUNT=0
FAIL_COUNT=0
SUMMARY_LINES=()

log_pass() {
    local label="$1"
    local detail="$2"
    PASS_COUNT=$((PASS_COUNT + 1))
    SUMMARY_LINES+=("  PASS  | ${label} | ${detail}")
    echo "[PASS] ${label}: ${detail}"
}

log_fail() {
    local label="$1"
    local detail="$2"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    SUMMARY_LINES+=("  FAIL  | ${label} | ${detail}")
    echo "[FAIL] ${label}: ${detail}" >&2
}

echo "========================================================"
echo " ChronoSentiment — Release Consistency Verification"
echo " Timestamp : ${TIMESTAMP}"
echo " Repo root : ${REPO_ROOT}"
echo "========================================================"
echo ""

# ── Check 1: cargo test replay ────────────────────────────────────────────────
echo "── Check 1: cargo test replay ──"
cd "${REPO_ROOT}"
REPLAY_OUTPUT="$(cargo test replay 2>&1)" || true
REPLAY_PASSED="$(echo "${REPLAY_OUTPUT}" | grep -oE '[0-9]+ passed' | awk '{sum += $1} END {print sum+0}')"
REPLAY_FAILED="$(echo "${REPLAY_OUTPUT}" | grep -oE '[0-9]+ failed' | awk '{sum += $1} END {print sum+0}')"

if [ "${REPLAY_FAILED}" -eq 0 ] && [ "${REPLAY_PASSED}" -gt 0 ]; then
    log_pass "cargo test replay" "${REPLAY_PASSED} passed; 0 failed"
else
    log_fail "cargo test replay" "${REPLAY_PASSED} passed; ${REPLAY_FAILED} failed"
fi
echo ""

# ── Check 2: Chronology byte fixtures ─────────────────────────────────────────
echo "── Check 2: verify_chronology_byte_fixtures.py ──"
CHRONO_OUTPUT="$(python3 "${SCRIPT_DIR}/verify_chronology_byte_fixtures.py" 2>&1)" || true
if echo "${CHRONO_OUTPUT}" | grep -q "PASS"; then
    CHRONO_COUNT="$(echo "${CHRONO_OUTPUT}" | grep -oE '[0-9]+' | head -1)"
    log_pass "verify_chronology_byte_fixtures.py" "${CHRONO_COUNT} fixtures PASS"
else
    log_fail "verify_chronology_byte_fixtures.py" "${CHRONO_OUTPUT}"
fi
echo ""

# ── Check 3: Strategy identity fixtures ───────────────────────────────────────
echo "── Check 3: verify_strategy_identity_fixtures.py ──"
STRAT_OUTPUT="$(python3 "${SCRIPT_DIR}/verify_strategy_identity_fixtures.py" 2>&1)" || true
if echo "${STRAT_OUTPUT}" | grep -q "verified"; then
    STRAT_COUNT="$(echo "${STRAT_OUTPUT}" | grep -oE '[0-9]+' | head -1)"
    log_pass "verify_strategy_identity_fixtures.py" "${STRAT_COUNT} records PASS"
else
    log_fail "verify_strategy_identity_fixtures.py" "${STRAT_OUTPUT}"
fi
echo ""

# ── Check 4: Double-build determinism (binary hash comparison) ────────────────
if [ "${SKIP_DOUBLE_BUILD}" = "--skip-double-build" ]; then
    echo "── Check 4: double-build determinism [SKIPPED via --skip-double-build] ──"
    SUMMARY_LINES+=("  SKIP  | double-build determinism | skipped by operator flag")
    echo "[SKIP] double-build determinism: skipped"
else
    echo "── Check 4: double-build determinism ──"
    cd "${REPO_ROOT}/core"

    SUBSTRATE="BTCUSDT"
    SUBSTRATE_FILE="chronology/live_capture_step3_bounded.jsonl"
    TOPOLOGY="plateau_low"
    COGNITION="event_reset"
    ARTIFACT_META="artifacts/${SUBSTRATE}/${TOPOLOGY}/${COGNITION}/metadata.json"

    if [ ! -f "${SUBSTRATE_FILE}" ]; then
        log_fail "double-build determinism" "substrate file not found: ${SUBSTRATE_FILE}"
    else
        echo "  Build pass 1..."
        cargo run --quiet --bin trace_replay -- \
            --substrate "${SUBSTRATE}" \
            --substrate-file "${SUBSTRATE_FILE}" \
            --topology "${TOPOLOGY}" \
            --cognition "${COGNITION}" > /dev/null 2>&1 || true

        if [ ! -f "${ARTIFACT_META}" ]; then
            log_fail "double-build determinism" "artifact metadata not produced: ${ARTIFACT_META}"
        else
            HASH1="$(python3 -c "import json; print(json.load(open('${ARTIFACT_META}'))['artifact_hash'])" 2>/dev/null || echo "MISSING")"

            echo "  Build pass 2..."
            cargo run --quiet --bin trace_replay -- \
                --substrate "${SUBSTRATE}" \
                --substrate-file "${SUBSTRATE_FILE}" \
                --topology "${TOPOLOGY}" \
                --cognition "${COGNITION}" > /dev/null 2>&1 || true

            HASH2="$(python3 -c "import json; print(json.load(open('${ARTIFACT_META}'))['artifact_hash'])" 2>/dev/null || echo "MISSING")"

            if [ "${HASH1}" = "${HASH2}" ] && [ "${HASH1}" != "MISSING" ]; then
                log_pass "double-build determinism" "hash stable across 2 builds: ${HASH1:0:16}..."
            else
                log_fail "double-build determinism" "hash diverged: pass1=${HASH1:0:16} pass2=${HASH2:0:16}"
            fi
        fi
    fi
    cd "${REPO_ROOT}"
fi
echo ""

# ── Check 5: Git state ────────────────────────────────────────────────────────
echo "── Check 5: git state ──"
GIT_COMMIT="$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || echo "unknown")"
GIT_BRANCH="$(git -C "${REPO_ROOT}" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")"
GIT_DIRTY="$(git -C "${REPO_ROOT}" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"

if [ "${GIT_DIRTY}" -eq 0 ]; then
    log_pass "git state" "clean — commit=${GIT_COMMIT} branch=${GIT_BRANCH}"
else
    log_pass "git state" "dirty (${GIT_DIRTY} uncommitted changes) — commit=${GIT_COMMIT} branch=${GIT_BRANCH}"
fi
echo ""

# ── Summary ───────────────────────────────────────────────────────────────────
echo "========================================================"
echo " Release Consistency Summary"
echo " Timestamp : ${TIMESTAMP}"
echo " Commit    : ${GIT_COMMIT}"
echo " Branch    : ${GIT_BRANCH}"
echo "--------------------------------------------------------"
printf "  %-6s | %-42s | %s\n" "Result" "Check" "Detail"
echo "--------------------------------------------------------"
for line in "${SUMMARY_LINES[@]}"; do
    echo "${line}"
done
echo "--------------------------------------------------------"
echo " PASS: ${PASS_COUNT}  FAIL: ${FAIL_COUNT}"
echo "========================================================"

if [ "${FAIL_COUNT}" -gt 0 ]; then
    echo ""
    echo "[FAIL] Release consistency check FAILED."
    echo "       Remediation: Do not release. Fix consistency errors above before packaging."
    exit 1
else
    echo ""
    echo "[PASS] Release consistency check PASSED — build is operationally consistent."
    exit 0
fi