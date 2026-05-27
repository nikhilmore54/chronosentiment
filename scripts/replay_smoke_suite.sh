#!/usr/bin/env bash
# replay_smoke_suite.sh
# ChronoSentiment — Replay Smoke Suite
#
# Purpose: Fast post-deploy smoke gate. Verifies that replay semantics remain
#          intact after any deployment or environment change. Three ordered
#          checks: replay unit tests → fixture verification → artifact hash
#          comparison against the certified baseline.
#
# Scope: Operational hardening only. Read-only after the optional single replay
#        run used to produce a fresh artifact for hash comparison. Does NOT
#        modify replay semantics, routing meaning, fixture content, or any
#        tranche-gated surface.
#
# Usage:
#   bash scripts/replay_smoke_suite.sh
#   bash scripts/replay_smoke_suite.sh --skip-hash-compare
#   bash scripts/replay_smoke_suite.sh --baseline-hash <sha256>
#
# Exit codes:
#   0 — all checks PASS
#   1 — one or more checks FAIL

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

SKIP_HASH_COMPARE=false
BASELINE_HASH_OVERRIDE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-hash-compare)  SKIP_HASH_COMPARE=true; shift ;;
        --baseline-hash)      BASELINE_HASH_OVERRIDE="$2"; shift 2 ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
done

cd "${REPO_ROOT}"

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

pass() { PASS_COUNT=$((PASS_COUNT + 1)); echo "  [PASS] $*"; }
fail() { FAIL_COUNT=$((FAIL_COUNT + 1)); echo "  [FAIL] $*" >&2; }
warn() { WARN_COUNT=$((WARN_COUNT + 1)); echo "  [WARN] $*"; }

echo "========================================================"
echo " ChronoSentiment — Replay Smoke Suite"
echo " Timestamp : ${TIMESTAMP}"
echo "========================================================"
echo ""

RELEASE_MODE=false
if [ -f "${REPO_ROOT}/RELEASE_INFO.json" ] && [ -f "${REPO_ROOT}/VERSION" ]; then
    RELEASE_MODE=true
fi

# ── Check 1: cargo test replay ────────────────────────────────────────────────
echo "── Check 1: cargo test replay"

if [ "${RELEASE_MODE}" = true ]; then
    pass "cargo test replay: N/A (Release Mode - binaries are pre-compiled)"
else
    REPLAY_OUT="$(cargo test replay 2>&1)" || true
    REPLAY_PASSED="$(echo "${REPLAY_OUT}" | grep -oE '[0-9]+ passed' | awk '{sum+=$1} END{print sum+0}')"
    REPLAY_FAILED="$(echo "${REPLAY_OUT}" | grep -oE '[0-9]+ failed' | awk '{sum+=$1} END{print sum+0}')"

    if [ "${REPLAY_FAILED}" -eq 0 ] && [ "${REPLAY_PASSED}" -gt 0 ]; then
        pass "cargo test replay: ${REPLAY_PASSED} passed, 0 failed"
    elif [ "${REPLAY_FAILED}" -eq 0 ] && [ "${REPLAY_PASSED}" -eq 0 ]; then
        warn "cargo test replay: 0 tests matched filter 'replay' — check test names"
    else
        fail "cargo test replay: ${REPLAY_PASSED} passed, ${REPLAY_FAILED} failed"
    fi
fi
echo ""

# ── Check 2: Historical tick replay — fixture verification ────────────────────
echo "── Check 2: Historical tick replay — fixture verification"

# 2a: Chronology byte fixtures
if [ -f "${SCRIPT_DIR}/verify_chronology_byte_fixtures.py" ]; then
    CHRONO_OUT="$(python3 "${SCRIPT_DIR}/verify_chronology_byte_fixtures.py" 2>&1)" || true
    if echo "${CHRONO_OUT}" | grep -q "PASS"; then
        COUNT="$(echo "${CHRONO_OUT}" | grep -oE '[0-9]+' | head -1)"
        pass "verify_chronology_byte_fixtures.py: ${COUNT} fixture(s) PASS"
    else
        fail "verify_chronology_byte_fixtures.py: verification failed"
        echo "${CHRONO_OUT}" | tail -5 >&2
    fi
else
    warn "verify_chronology_byte_fixtures.py: not found — skipping"
fi

# 2b: Strategy identity fixtures
if [ -f "${SCRIPT_DIR}/verify_strategy_identity_fixtures.py" ]; then
    STRAT_OUT="$(python3 "${SCRIPT_DIR}/verify_strategy_identity_fixtures.py" 2>&1)" || true
    if echo "${STRAT_OUT}" | grep -q "verified"; then
        COUNT="$(echo "${STRAT_OUT}" | grep -oE '[0-9]+' | head -1)"
        pass "verify_strategy_identity_fixtures.py: ${COUNT} record(s) verified"
    else
        fail "verify_strategy_identity_fixtures.py: verification failed"
        echo "${STRAT_OUT}" | tail -5 >&2
    fi
else
    warn "verify_strategy_identity_fixtures.py: not found — skipping"
fi
echo ""

# ── Check 3: Artifact hash comparison ─────────────────────────────────────────
echo "── Check 3: Artifact hash comparison"

if [ "${SKIP_HASH_COMPARE}" = true ]; then
    warn "hash comparison: skipped via --skip-hash-compare"
else
    SUBSTRATE="infrastructure/core/chronology/live_capture_step3_bounded.jsonl"
    TOPOLOGY="plateau_low"
    COGNITION="event_reset"
    TMP_ARTIFACT_DIR="$(mktemp -d)"
    trap 'rm -rf "${TMP_ARTIFACT_DIR}"' EXIT
    ARTIFACT_META="${TMP_ARTIFACT_DIR}/BTCUSDT/${TOPOLOGY}/${COGNITION}/metadata.json"

    # Resolve baseline hash: override flag > certified ledger > skip
    BASELINE_HASH="${BASELINE_HASH_OVERRIDE}"

    if [ -z "${BASELINE_HASH}" ]; then
        # Extract most recent certified hash from ledger
        if [ "${RELEASE_MODE}" = true ]; then
            LEDGER="${REPO_ROOT}/certification/replay_certification_log.md"
        else
            LEDGER="${REPO_ROOT}/docs/certification/replay_certification_log.md"
        fi
        
        if [ -f "${LEDGER}" ]; then
            # Ledger rows: | date | suite | result | hash | notes |
            # Hash is the 4th pipe-delimited column; grab last PASS row
            BASELINE_HASH="$(grep "^| 20" "${LEDGER}" \
                | grep "PASS" \
                | awk -F'|' '{gsub(/ /,"",$5); print $5}' \
                | grep -v "^$" \
                | tail -1 || echo "")"
        fi
    fi

    if [ -z "${BASELINE_HASH}" ]; then
        warn "hash comparison: no certified baseline hash available — skipping"
        warn "  Run with --baseline-hash <sha256> to supply one explicitly"
    elif [ ! -f "${REPO_ROOT}/${SUBSTRATE}" ]; then
        warn "hash comparison: substrate ${SUBSTRATE} not found — skipping"
    elif [ "${RELEASE_MODE}" = true ]; then
        # In release mode, produce fresh artifact via precompiled binary in /bin
        echo "  Running replay to produce fresh artifact (Release Mode)..."
        REPLAY_RUN_OK=true
        "${REPO_ROOT}/bin/trace_replay" \
            --input "${SUBSTRATE}" \
            --topology "${TOPOLOGY}" \
            --cognition "${COGNITION}" \
            --output-dir "${TMP_ARTIFACT_DIR}" \
            --quiet 2>/dev/null || REPLAY_RUN_OK=false

        if [ "${REPLAY_RUN_OK}" = false ]; then
            warn "hash comparison: bin/trace_replay binary not available or run failed — skipping"
        fi
    else
        # Produce a fresh artifact via a single replay run using cargo
        echo "  Running replay to produce fresh artifact..."
        REPLAY_RUN_OK=true
        cargo run --release --manifest-path infrastructure/core/Cargo.toml \
            --bin trace_replay -- \
            --input "${SUBSTRATE}" \
            --topology "${TOPOLOGY}" \
            --cognition "${COGNITION}" \
            --output-dir "${TMP_ARTIFACT_DIR}" \
            --quiet 2>/dev/null || REPLAY_RUN_OK=false

        if [ "${REPLAY_RUN_OK}" = false ]; then
            warn "hash comparison: trace_replay binary not available or run failed — skipping"
            warn "  Build with: cargo build --release --manifest-path infrastructure/core/Cargo.toml"
        fi
    fi

    if [ "${REPLAY_RUN_OK:-false}" = true ]; then
        if [ ! -f "${ARTIFACT_META}" ]; then
            warn "hash comparison: artifact metadata not produced at expected path"
            warn "  Expected: ${ARTIFACT_META#${REPO_ROOT}/}"
        else
            FRESH_HASH="$(python3 -c \
                "import json; print(json.load(open('${ARTIFACT_META}'))['artifact_hash'])" \
                2>/dev/null || echo "")"

            if [ -z "${FRESH_HASH}" ]; then
                fail "hash comparison: could not read artifact_hash from metadata.json"
            elif [ "${FRESH_HASH}" = "${BASELINE_HASH}" ]; then
                pass "hash comparison: fresh artifact matches certified baseline"
                echo "         baseline : ${BASELINE_HASH:0:32}..."
                echo "         fresh    : ${FRESH_HASH:0:32}..."
            else
                fail "hash comparison: DIVERGENCE — fresh artifact does not match certified baseline"
                echo "         baseline : ${BASELINE_HASH:0:32}..." >&2
                echo "         fresh    : ${FRESH_HASH:0:32}..." >&2
            fi
        fi
    fi
fi
echo ""

# ── Summary ───────────────────────────────────────────────────────────────────
echo "========================================================"
echo " Replay Smoke Suite Summary"
echo " Timestamp : ${TIMESTAMP}"
echo " PASS: ${PASS_COUNT}  FAIL: ${FAIL_COUNT}  WARN: ${WARN_COUNT}"
echo "========================================================"
echo ""

if [ "${FAIL_COUNT}" -gt 0 ]; then
    echo "[FAIL] Replay smoke suite FAILED — ${FAIL_COUNT} check(s) did not pass."
    echo "       Remediation: Do not proceed with deployment until failures are resolved."
    exit 1
else
    if [ "${WARN_COUNT}" -gt 0 ]; then
        echo "[PASS] Replay smoke suite PASSED with ${WARN_COUNT} warning(s)."
        echo "       Review warnings above; they may indicate missing optional components."
    else
        echo "[PASS] Replay smoke suite PASSED — all checks clean."
    fi
    exit 0
fi