#!/bin/bash
set -euo pipefail

TOOLCHAIN_BIN="/home/ultraspeed/.rustup/toolchains/1.91.1-aarch64-unknown-linux-gnu/bin"
CARGO="/home/ultraspeed/.cargo/bin/cargo"

export PATH="${TOOLCHAIN_BIN}:/home/ultraspeed/.cargo/bin:${PATH}"
export RUSTUP_TOOLCHAIN="1.91.1-aarch64-unknown-linux-gnu"

echo "=== Toolchain check ==="
echo "cargo: $(${CARGO} --version)"
echo "rustc: $(${TOOLCHAIN_BIN}/rustc --version)"
echo "RUSTUP_TOOLCHAIN: ${RUSTUP_TOOLCHAIN}"

cd /home/ultraspeed/ChronoSentiment_MEGA_FINAL

echo ""
echo "=== Building financial_replay ==="
${CARGO} build --release --manifest-path financial/strategies/Cargo.toml
BUILD_EXIT=$?

echo ""
echo "BUILD EXIT CODE: ${BUILD_EXIT}"

if [ ${BUILD_EXIT} -eq 0 ]; then
    BINARY=$(find . -name financial_replay -type f | head -1)
    echo "Binary: ${BINARY}"
    echo "Binary sha256: $(sha256sum ${BINARY} | awk '{print $1}')"
fi