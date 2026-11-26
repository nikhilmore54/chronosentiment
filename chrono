#!/usr/bin/env bash
# chrono
# ChronoSentiment — Unified Operator Facade
#
# Purpose: Deterministic command routing for operational tasks.
#          Stateless passthrough. No orchestration or adaptive logic.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Deterministic Environment Banner
if [ -f "${SCRIPT_DIR}/rust-toolchain.toml" ]; then
    RUST_VER="$(grep channel "${SCRIPT_DIR}/rust-toolchain.toml" | cut -d'"' -f2 || echo "unknown")"
else
    RUST_VER="unknown"
fi
if [ -f "${SCRIPT_DIR}/.python-version" ]; then
    PY_VER="$(cat "${SCRIPT_DIR}/.python-version" 2>/dev/null || echo "unknown")"
else
    PY_VER="unknown"
fi

if [ -f "${SCRIPT_DIR}/RELEASE_INFO.json" ]; then
    RUNTIME_MODE="RELEASE"
else
    RUNTIME_MODE="SOURCE"
fi

echo "[INFO] Rust toolchain: ${RUST_VER}" >&2
echo "[INFO] Python version: ${PY_VER}" >&2
echo "[INFO] Runtime mode: ${RUNTIME_MODE}" >&2
echo "[INFO] Platform: $(uname -sm)" >&2
echo "" >&2

usage() {
    echo "========================================================"
    echo " ChronoSentiment — Operator Facade"
    echo "========================================================"
    echo "Usage: ./chrono <command> [args...]"
    echo ""
    echo "Commands:"
    echo "  bootstrap         Environment validation (scripts/operator_bootstrap.sh)"
    echo "  smoke             Replay smoke suite (scripts/replay_smoke_suite.sh)"
    echo "  package           Generate release artifact (scripts/package_release.sh)"
    echo "  manifest          Generate release manifest (scripts/generate_release_manifest.sh)"
    echo "  release-verify    Verify release consistency (scripts/verify_release_consistency.sh)"
    echo "  help              Show this output"
    echo "========================================================"
}

if [ $# -eq 0 ]; then
    usage
    exit 1
fi

COMMAND="$1"
shift

case "${COMMAND}" in
    bootstrap)
        exec bash "${SCRIPT_DIR}/scripts/operator_bootstrap.sh" "$@"
        ;;
    smoke)
        exec bash "${SCRIPT_DIR}/scripts/replay_smoke_suite.sh" "$@"
        ;;
    package)
        exec bash "${SCRIPT_DIR}/scripts/package_release.sh" "$@"
        ;;
    manifest)
        exec bash "${SCRIPT_DIR}/scripts/generate_release_manifest.sh" "$@"
        ;;
    release-verify)
        exec bash "${SCRIPT_DIR}/scripts/verify_release_consistency.sh" "$@"
        ;;
    help|--help|-h)
        usage
        exit 0
        ;;
    *)
        echo "[FAIL] Unknown command: ${COMMAND}" >&2
        echo "       Run './chrono help' for valid commands." >&2
        exit 1
        ;;
esac
