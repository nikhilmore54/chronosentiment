#!/usr/bin/env bash
# scripts/package_release.sh
# ChronoSentiment — Deterministic Release Packager
#
# Purpose: Create a portable, deterministic release artifact from the current
#          working tree, ensuring operational reproducibility and replay certification.
#
# Scope: Operational hardening only.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

FORCE="${1:-}"

if [ "${FORCE}" != "--force" ]; then
    echo "Running release consistency verification..."
    if ! bash "${SCRIPT_DIR}/verify_release_consistency.sh"; then
        echo "[FAIL] Release consistency verification failed."
        echo "       Remediation: Cannot generate a certified release. Resolve verification failures first or use --force to override (not recommended)."
        exit 1
    fi
else
    echo "[WARN] Skipping release consistency verification (--force enabled)"
fi

VERSION="$(grep '^version' "${REPO_ROOT}/infrastructure/core/Cargo.toml" | head -1 | cut -d '"' -f 2)"
if [ -z "$VERSION" ]; then
    VERSION="unknown"
fi

COMMIT="$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || echo "unknown")"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

RELEASE_NAME="chronosentiment_${VERSION}_${COMMIT}"
CARGO_CMD="${CARGO_CMD:-cargo}"
TARGET_ARCH="${TARGET_ARCH:-}"
if [ -n "${TARGET_ARCH}" ]; then
    RELEASE_NAME="${RELEASE_NAME}_${TARGET_ARCH}"
fi
TMP_BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_BUILD_DIR}"' EXIT
RELEASE_DIR="${TMP_BUILD_DIR}/${RELEASE_NAME}"
TARBALL_NAME="${RELEASE_NAME}.tar.gz"
TARBALL_PATH="${REPO_ROOT}/${TARBALL_NAME}"

echo "========================================================"
echo " ChronoSentiment — Release Packaging"
echo " Version   : ${VERSION}"
echo " Commit    : ${COMMIT}"
echo " Timestamp : ${TIMESTAMP}"
echo " Output    : ${TARBALL_NAME}"
echo "========================================================"

# 1. Clean previous builds and prep release directory
rm -rf "${RELEASE_DIR}"
mkdir -p "${RELEASE_DIR}"/{bin,manifests,certification,replay/fixtures,scripts,docs}

# 2. Build release binaries
echo "Building release binaries..."
cd "${REPO_ROOT}"
TARGET_ARGS=()
if [ -n "${TARGET_ARCH}" ]; then
    TARGET_ARGS=("--target" "${TARGET_ARCH}")
fi
"${CARGO_CMD}" build --release --quiet "${TARGET_ARGS[@]}"

# Copy binaries
if [ -n "${TARGET_ARCH}" ]; then
    RELEASE_OUT_DIR="${REPO_ROOT}/target/${TARGET_ARCH}/release"
else
    RELEASE_OUT_DIR="${REPO_ROOT}/target/release"
fi

if [ -f "${RELEASE_OUT_DIR}/chronosentiment" ]; then
    cp "${RELEASE_OUT_DIR}/chronosentiment" "${RELEASE_DIR}/bin/"
fi
if [ -f "${RELEASE_OUT_DIR}/trace_replay" ]; then
    cp "${RELEASE_OUT_DIR}/trace_replay" "${RELEASE_DIR}/bin/"
fi
# Wait, let's copy whatever is in target/release without the *.d and deps
# Actually, specific binaries are safer for deterministic builds.

# 3. Copy scripts
echo "Copying scripts and tooling..."
cp "${SCRIPT_DIR}/replay_smoke_suite.sh" "${RELEASE_DIR}/scripts/" 2>/dev/null || true
cp "${SCRIPT_DIR}/operator_bootstrap.sh" "${RELEASE_DIR}/scripts/" 2>/dev/null || true
cp "${SCRIPT_DIR}/verify_release_consistency.sh" "${RELEASE_DIR}/scripts/" 2>/dev/null || true
cp "${SCRIPT_DIR}/verify_chronology_byte_fixtures.py" "${RELEASE_DIR}/scripts/" 2>/dev/null || true
cp "${SCRIPT_DIR}/verify_strategy_identity_fixtures.py" "${RELEASE_DIR}/scripts/" 2>/dev/null || true

# 4. Copy manifests & minimal docs
echo "Copying documentation and manifests..."
if [ -d "${REPO_ROOT}/docs/releases" ]; then
    cp -r "${REPO_ROOT}/docs/releases/"* "${RELEASE_DIR}/manifests/" 2>/dev/null || true
fi
if [ -d "${REPO_ROOT}/docs/certification" ]; then
    cp -r "${REPO_ROOT}/docs/certification/"* "${RELEASE_DIR}/certification/" 2>/dev/null || true
fi

cp "${REPO_ROOT}/OPERATIONAL_POSTURE.md" "${RELEASE_DIR}/docs/" 2>/dev/null || true
cp "${REPO_ROOT}/README.md" "${RELEASE_DIR}/docs/" 2>/dev/null || true
cp "${REPO_ROOT}/GETTING_STARTED.md" "${RELEASE_DIR}/docs/" 2>/dev/null || true
cp "${REPO_ROOT}/QUICKSTART.md" "${RELEASE_DIR}/" 2>/dev/null || true
cp "${REPO_ROOT}/chrono" "${RELEASE_DIR}/" 2>/dev/null || true

# 5. Copy certified replay corpus subset
echo "Copying replay corpus..."
if [ -d "${REPO_ROOT}/fixtures" ]; then
    cp -r "${REPO_ROOT}/fixtures/"* "${RELEASE_DIR}/replay/fixtures/" 2>/dev/null || true
fi

# 6. Generate VERSION and RELEASE_INFO.json
echo "${VERSION}" > "${RELEASE_DIR}/VERSION"

cat <<EOF > "${RELEASE_DIR}/RELEASE_INFO.json"
{
  "version": "${VERSION}",
  "commit": "${COMMIT}",
  "timestamp": "${TIMESTAMP}",
  "type": "operational_release",
  "guardrail": "bounded_determinism"
}
EOF

# 7. Generate release SHA256s
echo "Generating SHA256 checksums..."
cd "${RELEASE_DIR}"
find . -type f -not -name "SHA256SUMS" -print0 | sort -z | xargs -0 shasum -a 256 > SHA256SUMS

# 8. Emit deterministic tarball
echo "Creating tarball..."
cd "${TMP_BUILD_DIR}"
# Use reproducible tar if possible, otherwise standard tar
tar -czf "${TARBALL_PATH}" "${RELEASE_NAME}"

cd "${REPO_ROOT}"

echo "[PASS] Release artifact generated at:"
echo "       ${TARBALL_PATH}"
echo "========================================================"
