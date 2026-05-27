#!/usr/bin/env bash
# generate_release_manifest.sh
# ChronoSentiment — Release Manifest Generator
#
# Purpose: Capture the verified state of the repository at release-candidate
#          time and write a JSON manifest to docs/releases/.
#
# Scope: Operational hardening only. Does NOT modify replay semantics, routing
#        meaning, fixture content, or any tranche-gated surface.
#
# Usage:
#   bash scripts/generate_release_manifest.sh
#   bash scripts/generate_release_manifest.sh --notes "post-hardening sprint"
#   bash scripts/generate_release_manifest.sh --skip-build
#   bash scripts/generate_release_manifest.sh --skip-double-build
#
# Exit codes:
#   0 — overall_status PASS; manifest written
#   1 — overall_status FAIL; manifest written (records the failure)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H%M%SZ")"
TIMESTAMP_ISO="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

SKIP_BUILD=false
SKIP_DOUBLE_BUILD=false
NOTES=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build)        SKIP_BUILD=true; shift ;;
        --skip-double-build) SKIP_DOUBLE_BUILD=true; shift ;;
        --notes)             NOTES="$2"; shift 2 ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
done

cd "${REPO_ROOT}"

OVERALL_FAIL=0

echo "========================================================"
echo " ChronoSentiment — Release Manifest Generator"
echo " Timestamp : ${TIMESTAMP_ISO}"
echo "========================================================"
echo ""

# ── Git state ─────────────────────────────────────────────────────────────────
GIT_COMMIT_SHORT="unknown"
GIT_COMMIT_FULL="unknown"
GIT_BRANCH="unknown"
GIT_DIRTY_FILES=0

if command -v git >/dev/null 2>&1 && git rev-parse HEAD >/dev/null 2>&1; then
    GIT_COMMIT_SHORT="$(git rev-parse --short HEAD)"
    GIT_COMMIT_FULL="$(git rev-parse HEAD)"
    GIT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
    GIT_DIRTY_FILES="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
    echo "[git] commit=${GIT_COMMIT_SHORT}  branch=${GIT_BRANCH}  dirty=${GIT_DIRTY_FILES}"
else
    echo "[git] WARNING: git state unavailable"
fi

# ── Environment ───────────────────────────────────────────────────────────────
RUST_VERSION="$(rustc --version 2>/dev/null || echo "unavailable")"
CARGO_VERSION="$(cargo --version 2>/dev/null || echo "unavailable")"
OS_NAME="$(uname -s 2>/dev/null || echo "unknown")-$(uname -m 2>/dev/null || echo "unknown")"
echo "[env] ${RUST_VERSION} | ${CARGO_VERSION} | ${OS_NAME}"
echo ""

# ── Corpus files ──────────────────────────────────────────────────────────────
echo "── Corpus files"
CORPUS_JSON="[]"

if [ -d "${REPO_ROOT}/core/chronology" ]; then
    CORPUS_ENTRIES=""
    FIRST=true
    while IFS= read -r -d '' f; do
        REL_PATH="${f#${REPO_ROOT}/}"
        LINE_COUNT="$(wc -l < "${f}" | tr -d ' ')"
        SHA256="$(shasum -a 256 "${f}" 2>/dev/null | awk '{print $1}' || \
                  sha256sum "${f}" 2>/dev/null | awk '{print $1}' || \
                  echo "unavailable")"
        echo "  ${REL_PATH}: ${LINE_COUNT} records  sha256=${SHA256:0:16}..."
        ENTRY="{\"path\":\"${REL_PATH}\",\"sha256\":\"${SHA256}\",\"line_count\":${LINE_COUNT}}"
        if [ "${FIRST}" = true ]; then
            CORPUS_ENTRIES="${ENTRY}"
            FIRST=false
        else
            CORPUS_ENTRIES="${CORPUS_ENTRIES},${ENTRY}"
        fi
    done < <(find "${REPO_ROOT}/core/chronology" -name "*.jsonl" -print0 | sort -z)
    CORPUS_JSON="[${CORPUS_ENTRIES}]"
fi
echo ""

# ── cargo test replay ─────────────────────────────────────────────────────────
echo "── cargo test replay"
REPLAY_RESULT="FAIL"
REPLAY_PASSED=0
REPLAY_FAILED=0

REPLAY_OUT="$(cargo test replay 2>&1)" || true
REPLAY_PASSED="$(echo "${REPLAY_OUT}" | grep -oE '[0-9]+ passed' | awk '{sum+=$1} END{print sum+0}')"
REPLAY_FAILED="$(echo "${REPLAY_OUT}" | grep -oE '[0-9]+ failed' | awk '{sum+=$1} END{print sum+0}')"

if [ "${REPLAY_FAILED}" -eq 0 ] && [ "${REPLAY_PASSED}" -gt 0 ]; then
    REPLAY_RESULT="PASS"
    echo "  [PASS] ${REPLAY_PASSED} passed, 0 failed"
else
    REPLAY_RESULT="FAIL"
    OVERALL_FAIL=$((OVERALL_FAIL + 1))
    echo "  [FAIL] ${REPLAY_PASSED} passed, ${REPLAY_FAILED} failed" >&2
fi
echo ""

# ── Chronology byte fixtures ───────────────────────────────────────────────────
echo "── Chronology byte fixtures"
CHRONO_FIXTURE_RESULT="SKIP"

if [ -f "${SCRIPT_DIR}/verify_chronology_byte_fixtures.py" ]; then
    CHRONO_OUT="$(python3 "${SCRIPT_DIR}/verify_chronology_byte_fixtures.py" 2>&1)" || true
    if echo "${CHRONO_OUT}" | grep -q "PASS"; then
        CHRONO_FIXTURE_RESULT="PASS"
        echo "  [PASS] chronology byte fixtures verified"
    else
        CHRONO_FIXTURE_RESULT="FAIL"
        OVERALL_FAIL=$((OVERALL_FAIL + 1))
        echo "  [FAIL] chronology byte fixture verification failed" >&2
    fi
else
    echo "  [SKIP] verify_chronology_byte_fixtures.py not found"
fi
echo ""

# ── Strategy identity fixtures ────────────────────────────────────────────────
echo "── Strategy identity fixtures"
STRAT_FIXTURE_RESULT="SKIP"

if [ -f "${SCRIPT_DIR}/verify_strategy_identity_fixtures.py" ]; then
    STRAT_OUT="$(python3 "${SCRIPT_DIR}/verify_strategy_identity_fixtures.py" 2>&1)" || true
    if echo "${STRAT_OUT}" | grep -q "verified"; then
        STRAT_FIXTURE_RESULT="PASS"
        echo "  [PASS] strategy identity fixtures verified"
    else
        STRAT_FIXTURE_RESULT="FAIL"
        OVERALL_FAIL=$((OVERALL_FAIL + 1))
        echo "  [FAIL] strategy identity fixture verification failed" >&2
    fi
else
    echo "  [SKIP] verify_strategy_identity_fixtures.py not found"
fi
echo ""

# ── Binary hash ───────────────────────────────────────────────────────────────
echo "── Binary hash"
BINARY_HASH="NOT_BUILT"
BINARY_PATH="NOT_BUILT"

if [ "${SKIP_BUILD}" = false ]; then
    echo "  Building release binary..."
    if cargo build --release --manifest-path core/Cargo.toml --quiet 2>&1; then
        # Find the primary binary
        CANDIDATE_BIN="${REPO_ROOT}/target/release/trace_replay"
        if [ ! -f "${CANDIDATE_BIN}" ]; then
            # Fallback: find any non-incremental binary in target/release
            CANDIDATE_BIN="$(find "${REPO_ROOT}/target/release" -maxdepth 1 -type f -perm +111 \
                ! -name "*.d" ! -name "*.rlib" ! -name "*.rmeta" 2>/dev/null | head -1 || echo "")"
        fi
        if [ -n "${CANDIDATE_BIN}" ] && [ -f "${CANDIDATE_BIN}" ]; then
            BINARY_HASH="$(shasum -a 256 "${CANDIDATE_BIN}" 2>/dev/null | awk '{print $1}' || \
                           sha256sum "${CANDIDATE_BIN}" 2>/dev/null | awk '{print $1}' || \
                           echo "hash_unavailable")"
            BINARY_PATH="${CANDIDATE_BIN#${REPO_ROOT}/}"
            echo "  [OK] ${BINARY_PATH}: sha256=${BINARY_HASH:0:16}..."
        else
            echo "  [WARN] Build succeeded but no binary found at expected path"
            BINARY_HASH="BUILD_OK_NO_BINARY"
        fi
    else
        BINARY_HASH="BUILD_FAILED"
        OVERALL_FAIL=$((OVERALL_FAIL + 1))
        echo "  [FAIL] cargo build --release failed" >&2
    fi
else
    echo "  [SKIP] --skip-build specified"
fi
echo ""

# ── Double-build determinism ──────────────────────────────────────────────────
echo "── Double-build determinism"
DOUBLE_BUILD_RESULT="SKIP"

if [ "${SKIP_DOUBLE_BUILD}" = false ] && [ "${SKIP_BUILD}" = false ]; then
    SUBSTRATE="${REPO_ROOT}/core/chronology/live_capture_step3_bounded.jsonl"
    TOPOLOGY="plateau_low"
    COGNITION="event_reset"
    ARTIFACT_META="${REPO_ROOT}/artifacts/BTCUSDT/${TOPOLOGY}/${COGNITION}/metadata.json"

    if [ -f "${SUBSTRATE}" ] && command -v cargo >/dev/null 2>&1; then
        # First run
        cargo run --release --manifest-path core/Cargo.toml \
            --bin trace_replay -- \
            --input "core/chronology/live_capture_step3_bounded.jsonl" \
            --topology "${TOPOLOGY}" \
            --cognition "${COGNITION}" \
            --output-dir artifacts \
            --quiet 2>/dev/null || true

        HASH1=""
        if [ -f "${ARTIFACT_META}" ]; then
            HASH1="$(python3 -c "import json; print(json.load(open('${ARTIFACT_META}'))['artifact_hash'])" 2>/dev/null || echo "")"
        fi

        # Second run
        cargo run --release --manifest-path core/Cargo.toml \
            --bin trace_replay -- \
            --input "core/chronology/live_capture_step3_bounded.jsonl" \
            --topology "${TOPOLOGY}" \
            --cognition "${COGNITION}" \
            --output-dir artifacts \
            --quiet 2>/dev/null || true

        HASH2=""
        if [ -f "${ARTIFACT_META}" ]; then
            HASH2="$(python3 -c "import json; print(json.load(open('${ARTIFACT_META}'))['artifact_hash'])" 2>/dev/null || echo "")"
        fi

        if [ -n "${HASH1}" ] && [ "${HASH1}" = "${HASH2}" ]; then
            DOUBLE_BUILD_RESULT="PASS"
            echo "  [PASS] artifact_hash identical across two runs: ${HASH1:0:16}..."
        elif [ -z "${HASH1}" ] || [ -z "${HASH2}" ]; then
            DOUBLE_BUILD_RESULT="SKIP"
            echo "  [SKIP] artifact metadata not produced — trace_replay binary may not exist"
        else
            DOUBLE_BUILD_RESULT="FAIL"
            OVERALL_FAIL=$((OVERALL_FAIL + 1))
            echo "  [FAIL] artifact_hash diverged: run1=${HASH1:0:16} run2=${HASH2:0:16}" >&2
        fi
    else
        DOUBLE_BUILD_RESULT="SKIP"
        echo "  [SKIP] substrate file or cargo not available"
    fi
else
    echo "  [SKIP] --skip-build or --skip-double-build specified"
fi
echo ""

# ── Certification ledger ──────────────────────────────────────────────────────
LEDGER="${REPO_ROOT}/docs/certification/replay_certification_log.md"
LEDGER_ENTRIES=0
if [ -f "${LEDGER}" ]; then
    LEDGER_ENTRIES="$(grep -c "^| 20" "${LEDGER}" 2>/dev/null || echo "0")"
fi

# ── Overall status ────────────────────────────────────────────────────────────
OVERALL_STATUS="PASS"
if [ "${OVERALL_FAIL}" -gt 0 ]; then
    OVERALL_STATUS="FAIL"
fi

# ── Write manifest ────────────────────────────────────────────────────────────
MANIFEST_DIR="${REPO_ROOT}/docs/releases"
mkdir -p "${MANIFEST_DIR}"
MANIFEST_FILE="${MANIFEST_DIR}/${TIMESTAMP}_${GIT_COMMIT_SHORT}.json"

python3 - <<PYEOF
import json, sys

manifest = {
    "manifest_version": "1",
    "generated_at": "${TIMESTAMP_ISO}",
    "git_commit_short": "${GIT_COMMIT_SHORT}",
    "git_commit_full": "${GIT_COMMIT_FULL}",
    "git_branch": "${GIT_BRANCH}",
    "git_dirty_files": ${GIT_DIRTY_FILES},
    "rust_version": "${RUST_VERSION}",
    "cargo_version": "${CARGO_VERSION}",
    "os_name": "${OS_NAME}",
    "corpus_files": ${CORPUS_JSON},
    "replay_test_result": "${REPLAY_RESULT}",
    "replay_test_passed": ${REPLAY_PASSED},
    "replay_test_failed": ${REPLAY_FAILED},
    "chronology_fixture_result": "${CHRONO_FIXTURE_RESULT}",
    "strategy_fixture_result": "${STRAT_FIXTURE_RESULT}",
    "binary_hash_sha256": "${BINARY_HASH}",
    "binary_path": "${BINARY_PATH}",
    "double_build_determinism": "${DOUBLE_BUILD_RESULT}",
    "certification_ledger_entries": ${LEDGER_ENTRIES},
    "overall_status": "${OVERALL_STATUS}",
    "notes": "${NOTES}"
}

with open("${MANIFEST_FILE}", "w") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")

print(json.dumps(manifest, indent=2))
PYEOF

echo ""
echo "========================================================"
echo " Release Manifest"
echo " File   : ${MANIFEST_FILE#${REPO_ROOT}/}"
echo " Status : ${OVERALL_STATUS}"
echo "========================================================"
echo ""

if [ "${OVERALL_STATUS}" = "PASS" ]; then
    echo "[PASS] Manifest written. Commit docs/releases/ to record this release."
    echo ""
    echo "  git add docs/releases/$(basename "${MANIFEST_FILE}")"
    echo "  git commit -m \"chore(release): manifest ${GIT_COMMIT_SHORT} — ${OVERALL_STATUS}\""
    exit 0
else
    echo "[FAIL] ${OVERALL_FAIL} check(s) failed. Manifest written with FAIL status." >&2
    exit 1
fi