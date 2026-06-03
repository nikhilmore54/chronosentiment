#!/usr/bin/env bash
# Phase 0D — Host B (Linux x86_64 VM) Setup and Certification Script
# Run this inside the Ubuntu Server 24.04 AMD64 VM after first boot.
#
# Usage:
#   chmod +x vm_setup_and_certify.sh
#   ./vm_setup_and_certify.sh
#
# The script will:
#   1. Install build dependencies
#   2. Install Rust 1.91.1 via rustup
#   3. Clone the repo at the certified commit
#   4. Prompt you to copy chronology files (or use scp)
#   5. Verify chronology hashes
#   6. Capture Host B environment
#   7. Build financial_replay
#   8. Run SPY and GDAXI replays
#   9. Compare hashes against Host A baseline
#  10. Print PASS or FAIL

set -euo pipefail

# ─── Constants ────────────────────────────────────────────────────────────────

CERTIFIED_COMMIT="1eebebef8cbbb53aacdd22237945195b41260de0"
RUST_VERSION="1.91.1"

REPO_URL="${REPO_URL:-}"  # set via env or prompted below
REPO_DIR="${HOME}/ChronoSentiment_MEGA_FINAL"

SPY_CHRONOLOGY="chronology/historical/spy_capture/spy_1779283800000.jsonl"
GDAXI_CHRONOLOGY="chronology/historical/dax_capture/gdaxi_1779346800000.jsonl"

EXPECTED_SPY_CHRONO="d8566522996a1192b0580fadd2e5b6dca3d9a3eed21ab14340af9543d98dc415"
EXPECTED_GDAXI_CHRONO="de8d68019ba2be5106bb10bde10c81187b748f7eb8fa7b470a9d28fc4441a6c8"

EXPECTED_SPY_REPLAY="871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7"
EXPECTED_GDAXI_REPLAY="e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0"

RESULT_FILE="${HOME}/phase0d_host_b_result.txt"

# ─── Helpers ──────────────────────────────────────────────────────────────────

log()  { echo "[0D] $*"; }
pass() { echo "[0D] ✓ PASS: $*"; }
fail() { echo "[0D] ✗ FAIL: $*"; exit 1; }

check_hash() {
  local file="$1"
  local expected="$2"
  local label="$3"
  local actual
  actual=$(sha256sum "$file" | awk '{print $1}')
  if [ "$actual" = "$expected" ]; then
    pass "$label hash matches: $actual"
  else
    fail "$label hash MISMATCH\n  expected: $expected\n  actual:   $actual"
  fi
}

# ─── Step 1: Build Dependencies ───────────────────────────────────────────────

log "Step 1: Installing build dependencies..."
sudo apt-get update -qq
sudo apt-get install -y build-essential git curl pkg-config python3 python3-pip clang
log "Build dependencies installed."

# ─── Step 2: Rust 1.91.1 ─────────────────────────────────────────────────────

log "Step 2: Installing Rust ${RUST_VERSION} via rustup..."
if ! command -v rustup &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "${RUST_VERSION}"
  source "${HOME}/.cargo/env"
else
  log "rustup already present, setting default to ${RUST_VERSION}..."
  source "${HOME}/.cargo/env"
  rustup default "${RUST_VERSION}"
fi

rustc --version
cargo --version
log "Rust toolchain ready."

# ─── Step 3: Clone Repo ───────────────────────────────────────────────────────

log "Step 3: Cloning repository at certified commit..."

if [ -z "${REPO_URL}" ]; then
  echo ""
  echo "Enter the repository URL (git remote URL or local path via scp):"
  read -r REPO_URL
fi

if [ -d "${REPO_DIR}" ]; then
  log "Repo directory already exists, skipping clone."
else
  git clone "${REPO_URL}" "${REPO_DIR}"
fi

cd "${REPO_DIR}"
git checkout "${CERTIFIED_COMMIT}"
ACTUAL_COMMIT=$(git rev-parse HEAD)

if [ "${ACTUAL_COMMIT}" = "${CERTIFIED_COMMIT}" ]; then
  pass "Commit matches: ${ACTUAL_COMMIT}"
else
  fail "Commit mismatch: expected ${CERTIFIED_COMMIT}, got ${ACTUAL_COMMIT}"
fi

# ─── Step 4: Chronology Files ─────────────────────────────────────────────────

log "Step 4: Checking chronology files..."

if [ ! -f "${SPY_CHRONOLOGY}" ] || [ ! -f "${GDAXI_CHRONOLOGY}" ]; then
  echo ""
  echo "Chronology files not found. Copy them now from Host A using scp:"
  echo ""
  echo "  scp <host-a-user>@<host-a-ip>:\"<repo>/chronology/historical/spy_capture/spy_1779283800000.jsonl\" \\"
  echo "      ${REPO_DIR}/chronology/historical/spy_capture/"
  echo ""
  echo "  scp <host-a-user>@<host-a-ip>:\"<repo>/chronology/historical/dax_capture/gdaxi_1779346800000.jsonl\" \\"
  echo "      ${REPO_DIR}/chronology/historical/dax_capture/"
  echo ""
  echo "Press ENTER when done..."
  read -r
fi

# ─── Step 5: Verify Chronology Hashes ────────────────────────────────────────

log "Step 5: Verifying chronology hashes..."
check_hash "${SPY_CHRONOLOGY}"   "${EXPECTED_SPY_CHRONO}"   "SPY chronology"
check_hash "${GDAXI_CHRONOLOGY}" "${EXPECTED_GDAXI_CHRONO}" "GDAXI chronology"

# ─── Step 6: Capture Host B Environment ──────────────────────────────────────

log "Step 6: Capturing Host B environment..."
{
  echo "=== Host B Environment ==="
  echo "Date: $(date -u)"
  echo "git commit: $(git rev-parse HEAD)"
  echo "rustc: $(rustc --version)"
  echo "cargo: $(cargo --version)"
  echo "rustup active: $(rustup show active-toolchain 2>/dev/null || echo 'n/a')"
  echo "uname -a: $(uname -a)"
  echo "uname -m: $(uname -m)"
  echo "Cargo.lock sha256: $(sha256sum Cargo.lock | awk '{print $1}')"
} | tee "${RESULT_FILE}"

# ─── Step 7: Build ────────────────────────────────────────────────────────────

log "Step 7: Building financial_replay (release)..."
cargo build --release --manifest-path financial/strategies/Cargo.toml

BINARY=$(find . -name financial_replay -type f | head -1)
if [ -z "${BINARY}" ]; then
  fail "financial_replay binary not found after build"
fi
BINARY_HASH=$(sha256sum "${BINARY}" | awk '{print $1}')
log "Binary: ${BINARY}"
log "Binary sha256: ${BINARY_HASH}"
echo "binary: ${BINARY}" >> "${RESULT_FILE}"
echo "binary sha256: ${BINARY_HASH}" >> "${RESULT_FILE}"

# ─── Step 8: SPY Replay ───────────────────────────────────────────────────────

log "Step 8: Running SPY replay..."
cargo run --release \
  --manifest-path financial/strategies/Cargo.toml \
  --bin financial_replay -- \
  --substrate SPY \
  --substrate-file "${SPY_CHRONOLOGY}" \
  --topology baseline \
  --cognition rolling_50

SPY_ACTUAL=$(cat artifacts/SPY/baseline/rolling_50/replay_hash.txt)
echo "SPY replay hash (actual): ${SPY_ACTUAL}" >> "${RESULT_FILE}"

if [ "${SPY_ACTUAL}" = "${EXPECTED_SPY_REPLAY}" ]; then
  pass "SPY replay hash matches Host A: ${SPY_ACTUAL}"
  echo "SPY: PASS" >> "${RESULT_FILE}"
else
  echo "[0D] ✗ SPY replay hash MISMATCH"
  echo "  expected: ${EXPECTED_SPY_REPLAY}"
  echo "  actual:   ${SPY_ACTUAL}"
  echo "SPY: FAIL (expected=${EXPECTED_SPY_REPLAY}, actual=${SPY_ACTUAL})" >> "${RESULT_FILE}"
  SPY_PASS=false
fi
SPY_PASS="${SPY_PASS:-true}"

# ─── Step 9: GDAXI Replay ─────────────────────────────────────────────────────

log "Step 9: Running GDAXI replay..."
cargo run --release \
  --manifest-path financial/strategies/Cargo.toml \
  --bin financial_replay -- \
  --substrate GDAXI \
  --substrate-file "${GDAXI_CHRONOLOGY}" \
  --topology baseline \
  --cognition rolling_50

GDAXI_ACTUAL=$(cat artifacts/GDAXI/baseline/rolling_50/replay_hash.txt)
echo "GDAXI replay hash (actual): ${GDAXI_ACTUAL}" >> "${RESULT_FILE}"

if [ "${GDAXI_ACTUAL}" = "${EXPECTED_GDAXI_REPLAY}" ]; then
  pass "GDAXI replay hash matches Host A: ${GDAXI_ACTUAL}"
  echo "GDAXI: PASS" >> "${RESULT_FILE}"
else
  echo "[0D] ✗ GDAXI replay hash MISMATCH"
  echo "  expected: ${EXPECTED_GDAXI_REPLAY}"
  echo "  actual:   ${GDAXI_ACTUAL}"
  echo "GDAXI: FAIL (expected=${EXPECTED_GDAXI_REPLAY}, actual=${GDAXI_ACTUAL})" >> "${RESULT_FILE}"
  GDAXI_PASS=false
fi
GDAXI_PASS="${GDAXI_PASS:-true}"

# ─── Step 10: Final Verdict ───────────────────────────────────────────────────

echo "" | tee -a "${RESULT_FILE}"
if [ "${SPY_PASS}" = "true" ] && [ "${GDAXI_PASS}" = "true" ]; then
  echo "=== 0D-A RESULT: PASS ===" | tee -a "${RESULT_FILE}"
  echo "Both SPY and GDAXI replay hashes match Host A baseline." | tee -a "${RESULT_FILE}"
  echo "macOS ARM64 → Linux x86_64 replay reproducibility: CERTIFIED" | tee -a "${RESULT_FILE}"
else
  echo "=== 0D-A RESULT: FAIL ===" | tee -a "${RESULT_FILE}"
  echo "One or more replay hashes did not match Host A baseline." | tee -a "${RESULT_FILE}"
fi

echo ""
echo "Full result written to: ${RESULT_FILE}"
echo "Copy this file back to Host A as: phase0d/host_b_0da_result.md"