#!/usr/bin/env bash
# operator_bootstrap.sh
# ChronoSentiment — Single-Command Operator Bootstrap
#
# Purpose: Validate that the local environment is operationally ready to run
#          ChronoSentiment. Checks tools, fixtures, replay corpus, and emits
#          a clear READY / NOT READY diagnostic summary.
#
# Scope: Operational hardening only. Read-only. Does NOT build, start services,
#        modify fixtures, or touch any tranche-gated surface.
#
# Usage:
#   bash scripts/operator_bootstrap.sh
#
# Exit codes:
#   0 — environment is READY
#   1 — one or more REQUIRED checks failed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0
LINES=()

ok()   { PASS_COUNT=$((PASS_COUNT + 1)); LINES+=("  PASS  ${1}"); echo "  [PASS] ${1}"; }
fail() { FAIL_COUNT=$((FAIL_COUNT + 1)); LINES+=("  FAIL  ${1}"); echo "  [FAIL] ${1}" >&2; }
warn() { WARN_COUNT=$((WARN_COUNT + 1)); LINES+=("  WARN  ${1}"); echo "  [WARN] ${1}"; }

echo "========================================================"
echo " ChronoSentiment — Operator Bootstrap Diagnostics"
echo " Timestamp : ${TIMESTAMP}"
echo " Repo root : ${REPO_ROOT}"
echo "========================================================"
echo ""

cd "${REPO_ROOT}"

RELEASE_MODE=false
if [ -f "${REPO_ROOT}/RELEASE_INFO.json" ] && [ -f "${REPO_ROOT}/VERSION" ]; then
    RELEASE_MODE=true
    echo "── Environment : Release Bundle Mode"
else
    echo "── Environment : Source Repository Mode"
fi
echo ""

# ── Section 1: Required tools ─────────────────────────────────────────────────
echo "── Section 1: Required tools"

check_tool() {
    local name="$1"
    local cmd="${2:-$1}"
    if command -v "${cmd}" >/dev/null 2>&1; then
        local ver
        ver="$(${cmd} --version 2>/dev/null | head -1 || echo "(version unknown)")"
        ok "${name}: ${ver}"
    else
        fail "${name}: not found in PATH"
    fi
}

check_tool "cargo"
check_tool "rustc"
check_tool "python3"
check_tool "git"
check_tool "jq"
echo ""

# ── Section 2: Rust toolchain ─────────────────────────────────────────────────
echo "── Section 2: Rust toolchain"

if [ "${RELEASE_MODE}" = true ]; then
    ok "Cargo workspace edition: N/A (Release Mode)"
    ok "core/Cargo.toml: N/A (Release Mode)"
else
    if command -v cargo >/dev/null 2>&1; then
        RUST_EDITION="$(cargo metadata --no-deps --format-version 1 2>/dev/null \
            | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; print(pkgs[0].get('edition','?')) if pkgs else print('?')" \
            2>/dev/null || echo "?")"
        ok "Cargo workspace edition: ${RUST_EDITION}"

        # Check core Cargo.toml exists
        if [ -f "${REPO_ROOT}/core/Cargo.toml" ]; then
            ok "core/Cargo.toml present"
        else
            fail "core/Cargo.toml missing"
        fi
    else
        fail "cargo not available — Rust toolchain check skipped"
    fi
fi
echo ""

# ── Section 3: Fixture availability ───────────────────────────────────────────
echo "── Section 3: Fixture availability"

if [ "${RELEASE_MODE}" = true ]; then
    FIXTURE_SCRIPTS=(
        "scripts/verify_chronology_byte_fixtures.py"
        "scripts/verify_strategy_identity_fixtures.py"
    )
    FIXTURE_DIRS=(
        "replay/fixtures"
    )
else
    FIXTURE_SCRIPTS=(
        "scripts/verify_chronology_byte_fixtures.py"
        "scripts/verify_strategy_identity_fixtures.py"
        "scripts/certify_replay_chain.py"
        "scripts/compare_replay_equivalence.py"
        "scripts/capture_live_chronology.py"
    )
    FIXTURE_DIRS=(
        "fixtures"
        "core/chronology"
    )
fi

for f in "${FIXTURE_SCRIPTS[@]}"; do
    if [ -f "${REPO_ROOT}/${f}" ]; then
        ok "${f}"
    else
        fail "${f}: missing"
    fi
done

# Check fixture data directories
for d in "${FIXTURE_DIRS[@]}"; do
    if [ -d "${REPO_ROOT}/${d}" ]; then
        COUNT="$(find "${REPO_ROOT}/${d}" -type f 2>/dev/null | wc -l | tr -d ' ')"
        ok "${d}/: ${COUNT} file(s)"
    else
        warn "${d}/: directory not found"
    fi
done
echo ""

# ── Section 4: Replay corpus presence ─────────────────────────────────────────
echo "── Section 4: Replay corpus presence"

if [ "${RELEASE_MODE}" = true ]; then
    ok "Replay corpus presence: N/A (Release Mode)"
else
    CORPUS_FILES=(
        "core/chronology/live_capture_step3_bounded.jsonl"
    )

    for f in "${CORPUS_FILES[@]}"; do
        if [ -f "${REPO_ROOT}/${f}" ]; then
            LINES_COUNT="$(wc -l < "${REPO_ROOT}/${f}" | tr -d ' ')"
            ok "${f}: ${LINES_COUNT} records"
        else
            warn "${f}: not found (bounded live ingress corpus absent)"
        fi
    done

    # Check for any JSONL corpus files in core/chronology
    if [ -d "${REPO_ROOT}/core/chronology" ]; then
        JSONL_COUNT="$(find "${REPO_ROOT}/core/chronology" -name "*.jsonl" 2>/dev/null | wc -l | tr -d ' ')"
        if [ "${JSONL_COUNT}" -gt 0 ]; then
            ok "core/chronology/: ${JSONL_COUNT} JSONL corpus file(s) present"
        else
            warn "core/chronology/: no JSONL corpus files found"
        fi
    fi
fi
echo ""

# ── Section 5: Certification ledger ───────────────────────────────────────────
echo "── Section 5: Certification ledger"

if [ "${RELEASE_MODE}" = true ]; then
    LEDGER="${REPO_ROOT}/certification/replay_certification_log.md"
else
    LEDGER="${REPO_ROOT}/docs/certification/replay_certification_log.md"
fi

if [ -f "${LEDGER}" ]; then
    ENTRY_COUNT="$(grep -c "^| 20" "${LEDGER}" 2>/dev/null || echo "0")"
    ok "${LEDGER#${REPO_ROOT}/}: ${ENTRY_COUNT} certification entries"
else
    fail "${LEDGER#${REPO_ROOT}/}: missing"
fi
echo ""

# ── Section 6: Git state ───────────────────────────────────────────────────────
echo "── Section 6: Git state"

if [ "${RELEASE_MODE}" = true ]; then
    ok "git state: N/A (Release Mode)"
else
    if command -v git >/dev/null 2>&1 && git -C "${REPO_ROOT}" rev-parse HEAD >/dev/null 2>&1; then
        GIT_COMMIT="$(git -C "${REPO_ROOT}" rev-parse --short HEAD)"
        GIT_BRANCH="$(git -C "${REPO_ROOT}" rev-parse --abbrev-ref HEAD)"
        GIT_DIRTY="$(git -C "${REPO_ROOT}" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
        ok "commit: ${GIT_COMMIT}  branch: ${GIT_BRANCH}  dirty: ${GIT_DIRTY} file(s)"
    else
        warn "git state unavailable"
    fi
fi
echo ""

# ── Section 7: Release tooling ────────────────────────────────────────────────
echo "── Section 7: Release tooling"

if [ "${RELEASE_MODE}" = true ]; then
    RELEASE_SCRIPTS=(
        "scripts/verify_release_consistency.sh"
    )
else
    RELEASE_SCRIPTS=(
        "scripts/verify_release_consistency.sh"
        "scripts/ci_determinism_test.sh"
    )
fi

for f in "${RELEASE_SCRIPTS[@]}"; do
    if [ -f "${REPO_ROOT}/${f}" ]; then
        ok "${f}"
    else
        warn "${f}: not found"
    fi
done
echo ""

# ── Summary ───────────────────────────────────────────────────────────────────
echo "========================================================"
echo " Operator Bootstrap Summary"
echo " Timestamp : ${TIMESTAMP}"
echo "--------------------------------------------------------"
for line in "${LINES[@]}"; do
    echo "${line}"
done
echo "--------------------------------------------------------"
echo " PASS: ${PASS_COUNT}  FAIL: ${FAIL_COUNT}  WARN: ${WARN_COUNT}"
echo "========================================================"
echo ""

if [ "${FAIL_COUNT}" -gt 0 ]; then
    echo "[FAIL] ${FAIL_COUNT} required check(s) failed."
    echo "       Remediation: Resolve missing dependencies listed above before operating."
    exit 1
else
    if [ "${WARN_COUNT}" -gt 0 ]; then
        echo "[READY] Environment is operational. ${WARN_COUNT} warning(s) noted above."
    else
        echo "[READY] Environment is fully operational. No issues detected."
    fi
    echo ""
    echo "  Quick start:"
    if [ "${RELEASE_MODE}" = true ]; then
        echo "    ./chrono release-verify --skip-double-build"
        echo "    ./chrono smoke"
    else
        echo "    cargo build --release --manifest-path core/Cargo.toml"
        echo "    ./chrono release-verify --skip-double-build"
        echo "    cargo test replay"
    fi
    exit 0
fi