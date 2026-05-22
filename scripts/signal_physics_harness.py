#!/usr/bin/env python3
import pandas as pd
import json
import hashlib
from pathlib import Path
import sys

def load_admissibility(ledger_path: Path):
    adm = {}
    with open(ledger_path, 'r') as f:
        for line in f:
            if line.strip():
                try:
                    row = json.loads(line)
                    adm[row['barrier_ts']] = row.get('admissibility', {})
                except Exception:
                    pass
    return adm

def run_harness():
    from candle_substrate import load_frozen_cohort
    
    batch_id = 9904
    batch_dir = Path("state_archive/batches/batch_9904/runs/live")
    ledger_path = batch_dir / "metadata" / "live_session_steps.jsonl"
    
    # Load historical candles via the substrate loader
    data, _ = load_frozen_cohort(batch_id, ["AAPL"])
    if "AAPL" not in data or data["AAPL"].empty:
        print("❌ No frozen telemetry found for AAPL in batch 9904.")
        return
        
    df = data["AAPL"].sort_index()
    
    admissibility_map = load_admissibility(ledger_path)
    
    print("🔬 SIGNAL PHYSICS HARNESS v1")
    print("Symbol  : AAPL")
    print("Strategy: Deterministic Momentum (2-tick)")
    print("=" * 85)
    
    execution_tape = []
    
    for i in range(2, len(df)):
        # ts in dataframe index is datetime64, convert to integer timestamp
        ts = int(df.index[i].timestamp())
        
        current_price = df['Close'].iloc[i]
        prev_price = df['Close'].iloc[i-2]
        
        # 1. Deterministic Intent Generation
        if current_price > prev_price:
            intent = "ENTER_LONG"
        elif current_price < prev_price:
            intent = "ENTER_SHORT"
        else:
            intent = "HOLD"
            
        # 2. Environmental Intersection
        adm = admissibility_map.get(ts)
        if not adm:
            continue
            
        allowed = adm.get("new_entries_allowed", False)
        
        if intent == "HOLD":
            action = "HELD"
            reason = "No signal"
        elif allowed:
            action = f"EXEC_{intent.split('_')[1]}"
            reason = adm.get("admissibility_reason", "SYNCHRONIZED")
        else:
            action = "BLOCKED"
            reason = adm.get("admissibility_reason", "UNKNOWN_DEGRADATION")
            
        record = {
            "barrier_ts": ts,
            "price": round(float(current_price), 2),
            "intent": intent,
            "action": action,
            "admissibility_reason": reason
        }
        execution_tape.append(record)
        
        print(f"TS: {ts} | Px: {current_price:6.2f} | Intent: {intent:11s} | Action: {action:10s} | Env: {reason}")
        
    print("=" * 85)
    tape_str = json.dumps(execution_tape, sort_keys=True).encode('utf-8')
    tape_hash = hashlib.sha256(tape_str).hexdigest()[:16]
    
    print(f"Total Barriers Evaluated : {len(execution_tape)}")
    print(f"Replay Artifact Hash     : {tape_hash}")
    print("=" * 85)
    
    # Write artifact
    out_file = batch_dir / "metadata" / "signal_physics_ledger.jsonl"
    with open(out_file, 'w') as f:
        for r in execution_tape:
            f.write(json.dumps(r) + "\n")
    print(f"💾 Wrote universal execution tape to {out_file.name}")

if __name__ == "__main__":
    run_harness()
