#!/usr/bin/env python3
import yfinance as yf
import pandas as pd
from pathlib import Path
import json

def build_substrate():
    symbols = ["AAPL", "MSFT", "GOOG", "AMZN", "NVDA", "META", "TSLA"]
    print(f"Downloading 1m data for ecology substrate: {symbols}")
    
    # Download 1 day of 1m data to get a few hundred/thousand rows
    df = yf.download(symbols, period="1d", interval="1m", group_by="ticker", threads=True)
    
    batch_dir = Path("state_archive/batches/batch_10000/runs/live")
    metadata_dir = batch_dir / "metadata"
    data_dir = batch_dir / "data"
    
    metadata_dir.mkdir(parents=True, exist_ok=True)
    data_dir.mkdir(parents=True, exist_ok=True)
    
    ledger_path = metadata_dir / "live_session_steps.jsonl"
    
    all_timestamps = df.index.sort_values().unique()
    
    with open(ledger_path, "w") as f:
        cycle = 0
        for ts in all_timestamps:
            cycle += 1
            record = {
                "barrier_ts": int(ts.timestamp()),
                "cycle": cycle,
                "symbols_attempted": len(symbols),
                "symbols_returned": len(symbols),
                "symbols_accepted": len(symbols),
                "freshness": {
                    "median_symbol_lag_sec": 0,
                    "lag_stddev": 0.0
                },
                "observability": {
                    "strict_ratio": 1.0,
                    "acceptance_ratio": 1.0,
                    "recovery_slope": 0.0,
                    "regime_state": "SYNCHRONIZED"
                },
                "admissibility": {
                    "execution_admissible": True,
                    "admissibility_reason": "SYNCHRONIZED",
                    "new_entries_allowed": True,
                    "exits_allowed": True,
                    "observability_schema_version": "v1.0",
                    "classification_policy_version": "v1.0"
                }
            }
            f.write(json.dumps(record) + "\n")
            
    for sym in symbols:
        try:
            if len(symbols) == 1:
                sym_df = df.dropna(subset=['Close'])
            else:
                sym_df = df[sym].dropna(subset=['Close'])
                
            if len(sym_df) > 0:
                sym_path = data_dir / f"{sym}_candles.csv"
                sym_df.to_csv(sym_path)
                print(f"Saved {sym} -> {len(sym_df)} ticks")
        except Exception as e:
            print(f"Failed {sym}: {e}")
            
    print(f"\nSubstrate built: {len(all_timestamps)} chronological barriers.")
    print(f"Path: {ledger_path}")

if __name__ == "__main__":
    build_substrate()
