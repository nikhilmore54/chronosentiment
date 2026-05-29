#!/bin/bash
set -e

echo "Running Fast CI Gates..."

cargo check --workspace
cargo test -p chronosentiment_strategies replay
./scripts/check_vocabulary.sh

# Check dependency invariants
if cargo tree -p chronosentiment_optimization | grep -E "chronosentiment_strategies|chronosentiment_financial_core"; then
    echo "FAIL: Constitutional Violation! Optimization layer depends on financial logic."
    exit 1
fi

# ── Candle Substrate Certification ───────────────────────────────────────────
# Every substrate change must preserve fingerprint and chronology guarantees.
# The 41-test suite encodes operational contracts (dedup, tz-normalization,
# merge semantics, serial execution) — not just implementation details.
echo "Running candle substrate certification (41 tests)..."
VENV_PYTHON=""
if [ -f ".venv_test/bin/python" ]; then
    VENV_PYTHON=".venv_test/bin/python"
elif command -v python3 &>/dev/null; then
    VENV_PYTHON="python3"
else
    echo "FAIL: No Python interpreter found for substrate certification."
    exit 1
fi
$VENV_PYTHON -m pytest tests/test_candle_substrate.py -q --tb=short
echo "PASS: Candle substrate certification (41/41)"

echo "FAST CI GATES PASSED."
