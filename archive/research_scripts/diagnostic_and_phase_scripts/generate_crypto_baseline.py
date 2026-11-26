#!/usr/bin/env python3
import json
import gzip
from pathlib import Path
import pandas as pd

def generate_perfect_baseline(batch_id: int, symbol: str):
    data_path = Path(f"state_archive/batches/batch_{batch_id}/data/{symbol.lower()}_frozen.csv.gz")
    if not data_path.exists():
        print(f"❌ Substrate not found at {data_path}")
        return
        
    records = []
    with gzip.open(data_path, "rt", encoding="utf-8") as f:
        for line in f:
            records.append(json.loads(line))
            
    out_dir = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata")
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / "live_session_steps.jsonl"
    
    with open(out_path, "w") as f:
        for r in records:
            step = {
                "barrier_ts": r["ts"],
                "symbol": symbol,
                "admissibility": {
                    "new_entries_allowed": True,
                    "admissibility_reason": "PERFECT_BASELINE"
                },
                "observability": {
                    "acceptance_ratio": 1.0
                }
            }
            f.write(json.dumps(step) + "\n")
            
    print(f"✅ Generated perfect baseline admissibility at {out_path} for {len(records)} ticks.")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch", type=int, default=10001)
    parser.add_argument("--symbol", type=str, default="BTCUSDT")
    args = parser.parse_args()
    generate_perfect_baseline(args.batch, args.symbol)
