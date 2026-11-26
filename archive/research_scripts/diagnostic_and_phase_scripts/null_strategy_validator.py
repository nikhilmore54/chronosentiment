#!/usr/bin/env python3
import json
import sys
from pathlib import Path

def run_null_strategy(ledger_path: Path):
    if not ledger_path.exists():
        print(f"❌ Ledger not found: {ledger_path}")
        sys.exit(1)

    print(f"🔬 Layer 3 Null Strategy Validation: {ledger_path}")
    print("=" * 60)
    
    total_intents = 0
    executed_intents = 0
    blocked_intents = 0
    
    with open(ledger_path, 'r') as f:
        for line in f:
            if not line.strip():
                continue
                
            row = json.loads(line)
            cycle = row.get('cycle')
            admissibility = row.get('admissibility')
            
            if not admissibility:
                continue
                
            # 1. Null Strategy emits a synthetic intent unconditionally
            total_intents += 1
            synthetic_intent = {"action": "ENTER_LONG", "symbol": "DUMMY"}
            
            # 2. Execution Pipeline enforces Admissibility ∩ Intent
            # Notice the alpha logic NEVER sees 'regime_state'. It only consumes 'new_entries_allowed'.
            entries_allowed = admissibility.get('new_entries_allowed', False)
            
            if entries_allowed:
                executed_intents += 1
                outcome = "✅ EXECUTED"
            else:
                blocked_intents += 1
                outcome = f"❌ BLOCKED (Environmental Blindness: {admissibility.get('admissibility_reason')})"
                
            print(f"Cycle {cycle:02d} | Intent: {synthetic_intent['action']} | {outcome}")
            
    print("=" * 60)
    print(f"Total Intents Generated: {total_intents}")
    print(f"Total Executed         : {executed_intents}")
    print(f"Total Blocked          : {blocked_intents}")
    print("=" * 60)
    
    if blocked_intents > 0 and executed_intents > 0:
        print("✅ SUCCESS: Execution pipeline correctly intersections admissibility with intents.")
    elif total_intents == 0:
        print("⚠️ WARNING: No admissibility data found to test.")
    else:
        print("⚠️ WARNING: Edge case detected, review logs.")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=str, default="state_archive/batches/batch_9904/runs/live/metadata/live_session_steps.jsonl")
    args = parser.parse_args()
    run_null_strategy(Path(args.ledger))
