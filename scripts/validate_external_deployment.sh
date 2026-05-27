#!/usr/bin/env bash
# scripts/validate_external_deployment.sh
# ChronoSentiment — External Operational Validation
#
# Purpose: Validate that a packaged release artifact operates deterministically
#          in a clean, minimal Linux environment without relying on developer
#          toolchains (cargo, rustc), with absolute scarcity (zero network, 512MB RAM).
#
# Usage: bash scripts/validate_external_deployment.sh <tarball_path> <docker_image>
# Example: bash scripts/validate_external_deployment.sh chronosentiment_0.1.0.tar.gz ubuntu:22.04

set -euo pipefail

TARBALL="${1:-}"
IMAGE="${2:-ubuntu:22.04}"

if [ -z "${TARBALL}" ] || [ ! -f "${TARBALL}" ]; then
    echo "[FAIL] Validation aborted: tarball not found or missing argument."
    echo "       Usage: $0 <tarball_path> <docker_image>"
    exit 1
fi

TARBALL_ABS="$(cd "$(dirname "${TARBALL}")" && pwd)/$(basename "${TARBALL}")"
echo "========================================================"
echo " ChronoSentiment — Air-Gapped Operational Validation"
echo " Image   : ${IMAGE}"
echo " Tarball : ${TARBALL_ABS}"
echo "========================================================"

# Phase 1: Networked Setup (Ephemeral preparation)
TRANSIENT_IMAGE="chrono-eval-${RANDOM}"
echo "[INFO] Phase 1: Preparing transient operational environment..."
echo "[INFO] Building ${TRANSIENT_IMAGE} from ${IMAGE} with minimal python3 runtime."

TMP_DOCKER_DIR="$(mktemp -d)"
cat <<EOF > "${TMP_DOCKER_DIR}/Dockerfile"
FROM ${IMAGE}
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -qq >/dev/null 2>&1 && apt-get install -y -qq python3 >/dev/null 2>&1
EOF

if ! docker build -q -t "${TRANSIENT_IMAGE}" "${TMP_DOCKER_DIR}" >/dev/null 2>&1; then
    echo "[FAIL] Failed to build transient preparation image."
    rm -rf "${TMP_DOCKER_DIR}"
    exit 1
fi
rm -rf "${TMP_DOCKER_DIR}"

# Phase 2: Air-Gapped Execution
echo "[INFO] Phase 2: Executing Air-Gapped Strict Validation..."
echo "       Constraints: --network none, --memory 512m, --cpus 1.0"

# Note: GitHub Actions runners might not always support memory-swap limitations perfectly if swap accounting is off.
# We apply memory and swap equally to enforce zero swap growth, falling back gracefully if daemon rejects it.
DOCKER_RUN_CMD=(docker run --rm -v "${TARBALL_ABS}:/mnt/release.tar.gz:ro")
DOCKER_RUN_CMD+=(--network none)
DOCKER_RUN_CMD+=(--memory 512m)
DOCKER_RUN_CMD+=(--cpus="1.0")

# Only add memory-swap if the host supports it, otherwise fallback to memory only.
if docker info 2>/dev/null | grep -q "WARNING: No swap limit support"; then
    echo "[WARN] Host does not support swap limit. Enforcing --memory 512m only."
else
    DOCKER_RUN_CMD+=(--memory-swap 512m)
fi

set +e
"${DOCKER_RUN_CMD[@]}" "${TRANSIENT_IMAGE}" bash -c '
set -euo pipefail

echo "[INFO] Container started: $(cat /etc/os-release | grep PRETTY_NAME | cut -d\" -f2 || echo unknown)"

# Assert cargo and rustc are NOT present
if command -v cargo >/dev/null 2>&1 || command -v rustc >/dev/null 2>&1; then
    echo "[FAIL] Rust toolchain detected in environment. This violates Release Mode isolation."
    exit 1
fi

# Extract artifact
WORKSPACE="$(mktemp -d)"
echo "[INFO] Extracting release artifact to ephemeral workspace: ${WORKSPACE}"
tar -xzf /mnt/release.tar.gz -C "${WORKSPACE}"
cd "${WORKSPACE}/chronosentiment_"*

echo "── Running Bootstrap ──"
./chrono bootstrap

echo "── Running Release Verify ──"
./chrono release-verify --skip-double-build

echo "── Running Replay Smoke Suite ──"
./chrono smoke

echo ""
echo "[PASS] No external network dependency detected"
echo "[PASS] Replay corpus sufficient for offline certification"
echo "[PASS] Operational appliance remained coherent under constrained execution"
'
EXIT_CODE=$?
set -e

# Phase 3: Teardown
echo "[INFO] Phase 3: Teardown of transient infrastructure..."
docker rmi -f "${TRANSIENT_IMAGE}" >/dev/null 2>&1 || true

if [ "${EXIT_CODE}" -ne 0 ]; then
    echo "[FAIL] Replay verification failed or exceeded resource constraints (e.g., >512MB RAM, OOM killed)."
    echo "       Remediation: inspect runtime allocation growth in replay validation path."
    exit 1
fi

echo "[PASS] Air-gapped validation completed perfectly."
exit 0
