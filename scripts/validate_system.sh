#!/usr/bin/env bash
# ChronoSentiment — Canonical System Validation
# ==============================================
# Runs the minimum ordered checks to confirm the system is healthy.
# All checks operate on the frozen-cohort replay path (no mock streamer).
#
# Usage:
#   bash scripts/validate_system.sh
#   bash scripts/validate_system.sh --batch-id 3 --run-label replay_equiv
#
# Exit codes:
#   0 — all checks passed
#   1 — one or more checks failed

set -euo pipefail

BATCH_ID="${BATCH_ID:-3}"
RUN_LABEL="${RUN_LABEL:-replay_equiv}"
ARCHIVE_DIR="state_archive/batches/batch_$(printf '%03d' "$BATCH_ID")/runs/$RUN_LABEL"

# Parse optional args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --batch-id) BATCH_ID="$2"; shift 2 ;;
    --run-label) RUN_LABEL="$2"; shift 2 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

ARCHIVE_DIR="state_archive/batches/batch_$(printf '%03d' "$BATCH_ID")/runs/$RUN_LABEL"

echo "========================================"
echo " ChronoSentiment System Validation"
echo " batch=$BATCH_ID  run=$RUN_LABEL"
echo " archive=$ARCHIVE_DIR"
echo "========================================"
echo ""

PASS=0
FAIL=0

run_check() {
  local label="$1"
  shift
  echo "── $label"
  if "$@"; then
    echo "   ✓ PASS"
    PASS=$((PASS + 1))
  else
    echo "   ✗ FAIL"
    FAIL=$((FAIL + 1))
  fi
  echo ""
}

# ── Check 1: Rust build (release) ────────────────────────────────────────────
run_check "Rust release build" \
  bash -c "cargo build --release --manifest-path cs-ingest/Cargo.toml --quiet && cargo build --release --example live_observatory --manifest-path core/Cargo.toml --quiet"

# ── Check 2: Timeline fingerprint (frozen substrate loads + aligns) ───────────
run_check "Timeline fingerprint (batch $BATCH_ID)" \
  ./target/release/cs-ingest timeline \
    --batch-id "$BATCH_ID" \
    --cohort "cohorts/batch_$(printf '%03d' "$BATCH_ID").txt"

# ── Check 3: Replay chain certification (Generates Archive) ────────────────────
run_check "Replay chain certification" \
  python3 scripts/certify_replay_chain.py \
    full-replay \
    --batch-id "$BATCH_ID" \
    --run-label "$RUN_LABEL" \
    --max-intervals 50 \
    --fresh

# ── Check 4: Archive integrity + replay consistency ───────────────────────────
run_check "Archive integrity + replay consistency" \
  python3 scripts/verify_cohort_baseline.py \
    --batch-id "$BATCH_ID" \
    --run-label "$RUN_LABEL"

# ── Check 5: Governor smoke test (no archive = NO_DATA/NOMINAL) ───────────────
run_check "Governor smoke test" \
  python3 scripts/governor_refresher.py \
    --archive-dir "$ARCHIVE_DIR" \
    --once

# ── Summary ───────────────────────────────────────────────────────────────────
echo "========================================"
echo " Results: $PASS passed, $FAIL failed"
echo "========================================"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
