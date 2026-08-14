#!/usr/bin/env bash
# CS-P-003 operational tick: current Yahoo data → decide_at(latest session ≤ now).
# Not a B4 replay. No brokerage. Engine remains unfrozen-dev.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

SESSION="${CSP003_SESSION:-product_validation/forward_unfrozen_dev}"
LOG="$SESSION/tick.log"

mkdir -p "$SESSION"
{
  echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) CS-P-003 tick ==="
  cargo run -p chronosentiment_adapter --bin csp003_forward_session -- tick --session "$SESSION"
} 2>&1 | tee -a "$LOG"

echo "Appended $SESSION/ledger.jsonl (see $LOG)"
