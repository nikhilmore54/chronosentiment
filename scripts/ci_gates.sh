#!/bin/bash
set -e

echo "Running CI Gates for ChronoSentiment Constitutional Architecture..."

echo "1. Checking Dependency Invariants..."
# Check that optimization has no upward dependencies
if cargo tree -p chronosentiment_optimization | grep -E "chronosentiment_strategies|chronosentiment_financial_core"; then
    echo "FAIL: Constitutional Violation! Optimization layer depends on financial logic."
    exit 1
fi
echo "PASS: Dependency Invariants"

echo "2. Checking Vocabulary Isolation..."
# Check that optimization contains no domain vocabulary
if grep -R -E "PnL|Trade|Signal|Bull|Bear" infrastructure/optimization/src; then
    echo "FAIL: Constitutional Violation! Domain vocabulary found in optimization crate."
    exit 1
fi
echo "PASS: Vocabulary Isolation"

echo "3. Replay Semantic & Structural Certification..."
# Release mode ensures timing/hashing determinism issues are caught early.
# We tolerate warnings temporarily but fail hard on test failures.
RUSTFLAGS="-A unused" cargo test --workspace --release

echo "ALL CI GATES PASSED."
