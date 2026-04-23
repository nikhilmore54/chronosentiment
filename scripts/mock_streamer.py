import time
import json
import csv
import sys
import glob
import os
from datetime import datetime

# --- Deterministic Trend-Cycle Settings ---
# Goal: guarantee persistent directional structure for geometry validation.
TREND_LENGTH = 60
DRIFT_PER_TICK = 0.0010

def parse_ts(ts_str):
    try:
        dt = datetime.strptime(ts_str, '%Y-%m-%d %H:%M:%S')
        return int(dt.timestamp())
    except:
        return 0

def main():
    files = glob.glob("data/nse/5m/*.csv")
    if not files: return

    readers = []
    for path in sorted(files)[:5]: 
        f = open(path, 'r')
        readers.append({
            "symbol": os.path.basename(path).replace(".csv", ""),
            "reader": csv.DictReader(f),
            "current_price": None,
            "avg_vol": 100,
            "trend_dir": 1,
            "trend_step": 0,
        })

    print(
        f"📡 Mock Streamer: TREND_CYCLE_MODE (drift={DRIFT_PER_TICK:.4f}, len={TREND_LENGTH})",
        file=sys.stderr
    )

    try:
        while True:
            batch = []
            for t in readers:
                try:
                    row = next(t["reader"])
                    real_price = float(row['close'])
                    if t["current_price"] is None: t["current_price"] = real_price
                    
                    volume = float(row.get('volume', 100))

                    # Deterministic trend cycle with periodic reversal.
                    t["current_price"] *= (1.0 + (DRIFT_PER_TICK * t["trend_dir"]))
                    t["trend_step"] += 1
                    if t["trend_step"] >= TREND_LENGTH:
                        t["trend_step"] = 0
                        t["trend_dir"] *= -1
                        volume = 500
                        print(
                            f"[ALPHA_EVENT] symbol={t['symbol']} dir={t['trend_dir']} (trend flip)",
                            file=sys.stderr
                        )
                    # Keep bounded to source regime while preserving deterministic drift.
                    t["current_price"] = (0.995 * t["current_price"]) + (0.005 * real_price)

                    batch.append({
                        "symbol": t["symbol"],
                        "timestamp": parse_ts(row.get('timestamp', row.get('date', ''))),
                        "open": t["current_price"],
                        "high": t["current_price"],
                        "low": t["current_price"],
                        "close": t["current_price"],
                        "volume": volume
                    })
                except StopIteration: continue
            
            if not batch: break
            print(json.dumps(batch), flush=True)
            time.sleep(0.001) 
    except: pass

if __name__ == "__main__":
    main()
