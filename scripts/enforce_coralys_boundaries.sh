#!/usr/bin/env bash
set -e

echo "Running Coralys Dependency Enforcement..."

CORALYS_DIRS=("coralys-ecology" "coralys-moga" "coralys-simulation" "coralys-decision" "coralys-recommendation")

# 1. Invariant A: No direct dependencies on chronosentiment
for dir in "${CORALYS_DIRS[@]}"; do
    if [ -f "$dir/Cargo.toml" ]; then
        if grep -q "chronosentiment" "$dir/Cargo.toml"; then
            echo "ERROR: $dir depends on chronosentiment"
            exit 1
        fi
    fi
done
echo "✓ Invariant A passed: No Coralys crate depends on ChronoSentiment."

# 2. Invariant B: No forbidden vocabulary
FORBIDDEN_WORDS=("Trade" "Order" "Portfolio" "Position" "Market" "Price" "PnL" "Sharpe" "BullTrend" "BearTrend" "SignalCluster")

VIOLATIONS=0
for word in "${FORBIDDEN_WORDS[@]}"; do
    for dir in "${CORALYS_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            MATCHES=$(grep -rnw "$dir/src" -e "$word" || true)
            if [ ! -z "$MATCHES" ]; then
                echo "ERROR: Forbidden vocabulary '$word' found in $dir:"
                echo "$MATCHES"
                VIOLATIONS=$((VIOLATIONS + 1))
            fi
        fi
    done
done

if [ $VIOLATIONS -gt 0 ]; then
    echo "Found $VIOLATIONS vocabulary violations."
    exit 1
fi
echo "✓ Invariant B passed: No forbidden vocabulary found in Coralys crates."

echo "All Coralys boundaries are strictly enforced."
exit 0
