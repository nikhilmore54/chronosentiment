#!/usr/bin/env python3
import json
import hashlib
from pathlib import Path
import pandas as pd
from candle_substrate import load_frozen_cohort

HOLD_PERIOD_TICKS = 5

def build_economic_ledger(physics_ledger_path: Path, df: pd.DataFrame, topology_id: str):
    strategy_id = physics_ledger_path.stem.replace("physics_ledger_", "").replace(f"_{topology_id}", "")
    
    # Extract the replay hash from the atlas (or compute it, but let's just grab it from the jsonl directly by computing it)
    execution_tape = []
    with open(physics_ledger_path, 'r') as f:
        for line in f:
            if line.strip():
                execution_tape.append(json.loads(line))
                
    # Deterministic Hashing to inherit replay_artifact_hash
    tape_str = json.dumps(execution_tape, sort_keys=True).encode('utf-8')
    replay_hash = hashlib.sha256(tape_str).hexdigest()[:16]

    economic_tape = []
    
    for row in execution_tape:
        ts = row["barrier_ts"]
        action = row["action"]
        
        trace = row.get("state_divergence_trace", {})
        intent_live = trace.get("intent_live", "HOLD")
        active_intent = row["intent"]
        
        # Find entry index
        try:
            entry_idx = df.index.get_loc(pd.to_datetime(ts, unit='s', utc=True))
        except KeyError:
            continue
            
        entry_price = float(df['Close'].iloc[entry_idx])
        exit_idx = min(entry_idx + HOLD_PERIOD_TICKS, len(df) - 1)
        exit_price = float(df['Close'].iloc[exit_idx])
        
        # Calculate Canonical PnL (If we executed the pristine intent perfectly)
        canonical_pnl = 0.0
        if intent_live == "ENTER_LONG":
            canonical_pnl = round(exit_price - entry_price, 2)
        elif intent_live == "ENTER_SHORT":
            canonical_pnl = round(entry_price - exit_price, 2)
            
        # Calculate Fragmented PnL (The actual execution physics)
        fragmented_pnl = 0.0
        if action == "EXEC_LONG":
            fragmented_pnl = round(exit_price - entry_price, 2)
        elif action == "EXEC_SHORT":
            fragmented_pnl = round(entry_price - exit_price, 2)
            
        economic_tape.append({
            "barrier_ts": ts,
            "symbol": "AAPL",
            "intent": active_intent,
            "action": action,
            "entry_price": entry_price,
            "exit_price": exit_price,
            "canonical_pnl_ticks": canonical_pnl,
            "fragmented_pnl_ticks": fragmented_pnl,
            "topology_id": topology_id,
            "replay_hash": replay_hash
        })
        
    out_file = physics_ledger_path.parent / physics_ledger_path.name.replace("physics_", "economic_")
    with open(out_file, 'w') as f:
        for r in economic_tape:
            f.write(json.dumps(r) + "\n")
            
    print(f"[{strategy_id} | {topology_id}] -> Wrote Economic Ledger (Replay Hash: {replay_hash})")
    
    # Compute summary for comparison
    total_canonical = round(sum(r["canonical_pnl_ticks"] for r in economic_tape), 2)
    total_fragmented = round(sum(r["fragmented_pnl_ticks"] for r in economic_tape), 2)
    print(f"   Canonical PnL : {total_canonical:8.2f} ticks")
    print(f"   Fragmented PnL: {total_fragmented:8.2f} ticks")
    print(f"   Economic Divergence: {round(total_canonical - total_fragmented, 2):8.2f} ticks\n")


def run_all():
    batch_id = 10000
    data, _ = load_frozen_cohort(batch_id, ["AAPL"])
    if "AAPL" not in data or data["AAPL"].empty:
        print("❌ No frozen telemetry found for AAPL in batch 10000.")
        return
        
    df = data["AAPL"].sort_index()
    
    metadata_dir = Path("state_archive/batches/batch_10000/runs/live/metadata")
    for f in metadata_dir.glob("physics_ledger_*.jsonl"):
        # The filename format is physics_ledger_{strategy_id}_{topology_ledger_name}.jsonl
        # The topology_ledger_name contains the topology_id.
        topology_id = "unknown"
        if "uniform" in f.name: topology_id = "topo_uniform_60"
        elif "bimodal" in f.name: topology_id = "topo_bimodal_180"
        elif "wavefront" in f.name: topology_id = "topo_wavefront"
        elif "anticipatory" in f.name: topology_id = "topo_anticipatory"
        elif "collapse" in f.name: topology_id = "topo_collapse_300"
        
        build_economic_ledger(f, df, topology_id)

if __name__ == "__main__":
    run_all()
