import time
import json
import csv
import sys
import glob
import os
import random
from datetime import datetime

# --- Alpha Injection Settings (Step/Silence Mode) ---
# Goal: Create a discrete jump, then hold. 
# As the rolling window slides, StdDev drops while DeltaPrice remains 100bps.
# This should eventually hit trend_consistency > 0.55.
ALPHA_PROBABILITY = 0.04 
ALPHA_JUMP_SIZE = 0.01   # 1% jump (discrete)
ALPHA_COOLDOWN = 100     # Silence after jump

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
            "alpha_cooldown": 0,
            "current_price": None,
            "avg_vol": 100
        })

    print(f"📡 Mock Streamer: STEP_MODE (1% jumps + 100-tick silence)", file=sys.stderr)

    try:
        while True:
            batch = []
            for t in readers:
                try:
                    row = next(t["reader"])
                    real_price = float(row['close'])
                    if t["current_price"] is None: t["current_price"] = real_price
                    
                    volume = float(row.get('volume', 100))
                    
                    if t["alpha_cooldown"] > 0:
                        t["alpha_cooldown"] -= 1
                        # Maintain the new price level
                    else:
                        if random.random() < ALPHA_PROBABILITY:
                            direction = 1 if random.random() > 0.5 else -1
                            t["current_price"] *= (1.0 + ALPHA_JUMP_SIZE * direction)
                            t["alpha_cooldown"] = ALPHA_COOLDOWN
                            volume = 500 # Volume spike on jump
                            print(f"[ALPHA_EVENT] symbol={t['symbol']} dir={direction} (step jump)", file=sys.stderr)
                        else:
                            # Reverting loosely to real price to keep it bounded
                            t["current_price"] = 0.99 * t["current_price"] + 0.01 * real_price

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
