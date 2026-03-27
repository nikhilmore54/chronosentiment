#!/usr/bin/env bash

API_URL="http://localhost:8000/order/O1"

echo "Fetching API response..."
RESPONSE=$(curl -s "$API_URL")

echo "Raw Response:"
echo "$RESPONSE" | jq .

echo ""
echo "Running contract checks..."

FAIL=0

# Check for forbidden patterns
echo "$RESPONSE" | grep -q '"OrderIntent": {' && echo "FAIL: Found old enum wrapper (OrderIntent)" && FAIL=1
echo "$RESPONSE" | grep -q '"QueueProgression": {' && echo "FAIL: Found old enum wrapper (QueueProgression)" && FAIL=1
echo "$RESPONSE" | grep -q '"ts":' && echo "FAIL: Found legacy field 'ts'" && FAIL=1
echo "$RESPONSE" | grep -q '"quantity_ahead":' && echo "FAIL: Found legacy field 'quantity_ahead'" && FAIL=1
echo "$RESPONSE" | grep -q '"new_quantity_ahead":' && echo "FAIL: Found legacy field 'new_quantity_ahead'" && FAIL=1

# execution must be array
echo "$RESPONSE" | jq -e '.execution | type != "array"' > /dev/null 2>&1 && echo "FAIL: execution is not an array" && FAIL=1

# decision must have type
echo "$RESPONSE" | jq -e '.decision.type == null' > /dev/null 2>&1 && echo "FAIL: decision is not flat (missing type)" && FAIL=1

# execution items must have type
echo "$RESPONSE" | jq -e '.execution[]? | select(.type == null)' > /dev/null 2>&1 && echo "FAIL: execution items missing type" && FAIL=1

# Final result
if [ "$FAIL" -eq 0 ]; then
  echo ""
  echo "SUCCESS: API contract is clean and correct"
else
  echo ""
  echo "CONTRACT VALIDATION FAILED"
  exit 1
fi

echo "🔍 Checking for legacy fields in codebase (excluding tests)..."

grep -rE "\bts:" core services/api/src --exclude-dir=target --exclude-dir=node_modules --exclude-dir=tests && echo "❌ Found 'ts:'" || echo "✅ No 'ts:'"
grep -r "quantity_ahead" core services/api/src --exclude-dir=target --exclude-dir=node_modules --exclude-dir=tests && echo "❌ Found 'quantity_ahead'" || echo "✅ Clean"
grep -r "new_quantity_ahead" core services/api/src --exclude-dir=target --exclude-dir=node_modules --exclude-dir=tests && echo "❌ Found 'new_quantity_ahead'" || echo "✅ Clean"