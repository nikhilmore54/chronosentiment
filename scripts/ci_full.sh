#!/bin/bash
set -e

echo "Running Full CI Constitutional Gates..."

echo "1. Validating Topology and Dependencies..."
if cargo tree -p chronosentiment_optimization | grep -E "chronosentiment_strategies|chronosentiment_financial_core"; then
    echo "FAIL: Constitutional Violation! Optimization layer depends on financial logic."
    exit 1
fi

echo "2. Validating Vocabulary Isolation..."
./scripts/check_vocabulary.sh

echo "3. Running Standard Test Suite..."
RUSTFLAGS="-A unused" cargo test --workspace

echo "4. Running Release Mode Certification & Single-Threaded Replay..."
RUSTFLAGS="-A unused" cargo test --workspace --release
RUSTFLAGS="-A unused" cargo test replay --release -- --test-threads=1

echo "FULL CI GATES PASSED."
