#!/bin/bash
set -e

echo "Checking Vocabulary Isolation..."

# Search for domain vocabulary strictly within the optimization source code.
# We exclude tests, docs, and fixtures to avoid false positives.
# If any matches are found, grep will return 0 (success) which triggers the if block.
# We want grep to fail (return 1) to indicate no matches were found.
if grep -R -n -E "PnL|Trade|Signal|Bull|Bear|Regime|Execution|Replay|Chronology" \
    --exclude-dir=tests \
    --exclude-dir=fixtures \
    --exclude-dir=docs \
    --include="*.rs" \
    infrastructure/optimization/src; then
    
    echo "FAIL: Constitutional Violation! Domain vocabulary found in optimization crate."
    exit 1
else
    echo "PASS: Vocabulary Isolation Maintained."
fi
