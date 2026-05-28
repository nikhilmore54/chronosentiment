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

echo "FAST CI GATES PASSED."
