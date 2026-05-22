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
                    adm[row['barrier_ts']] = {
                        "admissibility": row.get('admissibility', {}),
                        "observability": row.get('observability', {})
                    }
                except Exception:
                    pass
    return adm

def deterministic_acceptance(ts, symbol, acceptance_ratio):
    # Deterministic pseudo-random check to see if this symbol was accepted in this barrier
    # based purely on the macro acceptance_ratio.
    hash_int = int(hashlib.md5(f"{ts}_{symbol}".encode()).hexdigest(), 16)
    normalized = (hash_int % 1000) / 1000.0
    return normalized <= acceptance_ratio

def run_harness(ledger_path: Path, strategy_id: str):
    from candle_substrate import load_frozen_cohort
    
    batch_id = 10000
    batch_dir = Path("state_archive/batches/batch_10000/runs/live")
    
    data, _ = load_frozen_cohort(batch_id, ["AAPL"])
    if "AAPL" not in data or data["AAPL"].empty:
        print("❌ No frozen telemetry found for AAPL in batch 10000.")
        return
        
    df = data["AAPL"].sort_index()
    
    admissibility_map = load_admissibility(ledger_path)
    baseline_map = load_admissibility(batch_dir / "metadata" / "live_session_steps.jsonl")
    
    print("🔬 STATE DIVERGENCE TRACER v1")
    print("Symbol  : AAPL")
    print(f"Strategy: {strategy_id}")
    print("=" * 85)
    
    execution_tape = []
    
    # Prefill windows with the first two prices to ensure sufficient state for i=2
    window_baseline = []
    window_fragmented = []
    if len(df) >= 2:
        for idx in range(0, 2):
            px = float(df['Close'].iloc[idx])
            window_baseline.append(px)
            window_fragmented.append(px)
            
    WINDOW_SIZE = 3
    
    divergence_count = 0
    
    for i in range(2, len(df)):
        ts = int(df.index[i].timestamp())
        
        real_price = float(df['Close'].iloc[i])
        
        baseline_adm = baseline_map.get(ts, {})
        current_adm = admissibility_map.get(ts, {})
        
        if not current_adm:
            continue
            
        base_accept_ratio = baseline_adm.get("observability", {}).get("acceptance_ratio", 1.0)
        curr_accept_ratio = current_adm.get("observability", {}).get("acceptance_ratio", 1.0)
        
        # Deterministic tick acceptance
        base_accepted = deterministic_acceptance(ts, "AAPL", base_accept_ratio)
        curr_accepted = deterministic_acceptance(ts, "AAPL", curr_accept_ratio)
        
        base_price = real_price if base_accepted else (window_baseline[-1] if window_baseline else real_price)
        curr_price = real_price if curr_accepted else (window_fragmented[-1] if window_fragmented else real_price)
        
        # Accumulate fixed windows (blind to admissibility blockades)
        window_baseline.append(base_price)
        window_fragmented.append(curr_price)
        if len(window_baseline) > WINDOW_SIZE: window_baseline.pop(0)
        if len(window_fragmented) > WINDOW_SIZE: window_fragmented.pop(0)
        
        if len(window_baseline) < WINDOW_SIZE:
            continue
            
        # Intent Generation
        def generate_intent(window):
            if strategy_id == "rolling_window_momentum_v1":
                delta = window[-1] - window[0]
                if delta > 5.0: return "ENTER_LONG"
                if delta < -5.0: return "ENTER_SHORT"
                return "HOLD"
            else:
                # Fallback to 2-tick stateless
                delta = window[-1] - window[-2]
                if delta > 0: return "ENTER_LONG"
                if delta < 0: return "ENTER_SHORT"
                return "HOLD"
                
        intent_live = generate_intent(window_baseline)
        intent_fragmented = generate_intent(window_fragmented)
        
        is_divergent = (intent_live != intent_fragmented)
        if is_divergent:
            divergence_count += 1
            
        allowed = current_adm.get("admissibility", {}).get("new_entries_allowed", False)
        reason = current_adm.get("admissibility", {}).get("admissibility_reason", "UNKNOWN_DEGRADATION")
        
        if intent_fragmented == "HOLD":
            action = "HELD"
        elif allowed:
            action = f"EXEC_{intent_fragmented.split('_')[1]}"
        else:
            action = "BLOCKED"
            
        # Memory Coherence Index (MCI)
        overlap_count = sum(1 for a, b in zip(window_baseline, window_fragmented) if a == b)
        state_overlap_ratio = round(overlap_count / WINDOW_SIZE, 2)
        window_distance = round(sum(abs(a - b) for a, b in zip(window_baseline, window_fragmented)), 2)
        
        record = {
            "barrier_ts": ts,
            "intent": intent_fragmented,
            "action": action,
            "admissibility_reason": reason,
            "state_divergence_trace": {
                "window_state_live": [round(x, 2) for x in window_baseline],
                "window_state_fragmented": [round(x, 2) for x in window_fragmented],
                "intent_live": intent_live,
                "intent_fragmented": intent_fragmented,
                "divergence_reason": "chronological discontinuity" if is_divergent else None,
                "memory_coherence_index": {
                    "window_distance": window_distance,
                    "state_overlap_ratio": state_overlap_ratio
                }
            }
        }
        execution_tape.append(record)
        
        div_flag = "⚠️ DIVERGED" if is_divergent else "✅ COHERENT"
        print(f"TS: {ts} | Intent: {intent_fragmented:11s} | Action: {action:10s} | State: {div_flag}")

    print("=" * 85)
    
    # Deterministic Hashing
    tape_str = json.dumps(execution_tape, sort_keys=True).encode('utf-8')
    tape_hash = hashlib.sha256(tape_str).hexdigest()[:16]
    
    divergence_str = json.dumps([r["state_divergence_trace"] for r in execution_tape if r["state_divergence_trace"]["divergence_reason"]], sort_keys=True).encode('utf-8')
    divergence_hash = hashlib.sha256(divergence_str).hexdigest()[:16]
    
    print(f"Total Barriers          : {len(execution_tape)}")
    print(f"Cognitive Divergences   : {divergence_count}")
    print(f"Replay Artifact Hash    : {tape_hash}")
    print(f"State Divergence Hash   : {divergence_hash}")
    print("=" * 85)
    
    out_file = ledger_path.parent / f"physics_ledger_{strategy_id}_{ledger_path.stem}.jsonl"
    with open(out_file, 'w') as f:
        for r in execution_tape:
            f.write(json.dumps(r) + "\n")
    print(f"💾 Wrote state-traced execution tape to {out_file.name}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=str, default="state_archive/batches/batch_10000/runs/live/metadata/live_session_steps.jsonl")
    parser.add_argument("--strategy", type=str, default="rolling_window_momentum_v1")
    args = parser.parse_args()
    run_harness(Path(args.ledger), args.strategy)
