import time
import json
import csv
import sys
import glob
import os
import math
from datetime import datetime

# --- Deterministic Trend-Cycle Settings ---
TREND_LENGTH = 100
DRIFT_PER_TICK = 0.0001
DRIFT_MULTIPLIER = 2.0
SHOCK_AMPLITUDE = 0.0002 # Reduced shocks to keep momentum in bps range
SHOCK_PERIOD = 20

def parse_ts(ts_str):
    try:
        dt = datetime.strptime(ts_str, '%Y-%m-%d %H:%M:%S')
        return int(dt.timestamp())
    except:
        return 0

def main():
    symbols = ["BTC-USD", "ETH-USD", "SOL-USD", "AXISBANK.NS", "BSE.NS"]
    
    readers = []
    for i, sym in enumerate(symbols):
        readers.append({
            "symbol": sym,
            "current_price": 50000.0 if "BTC" in sym else (3000.0 if "ETH" in sym else (150.0 if "SOL" in sym else 1000.0)),
            "trend_dir": 1 if i % 2 == 0 else -1,
            "trend_step": 0,
            "ts": int(time.time()) - (1000 * 300) # Start with some history
        })

    print(
        f"📡 Mock Streamer (Stable Crypto): drift={DRIFT_PER_TICK * DRIFT_MULTIPLIER:.6f}",
        file=sys.stderr
    )

    try:
        # Emit 500 bars to fill history and buffers
        for tick in range(500):
            batch = []
            for t in readers:
                # Deterministic trend cycle
                phase = t["trend_step"] % SHOCK_PERIOD
                shock = SHOCK_AMPLITUDE * math.sin((2.0 * math.pi * phase) / SHOCK_PERIOD)
                # Cycle drift to test MIXED regime transitions
                cycle_idx = (tick // 100) % 3
                current_drift = DRIFT_PER_TICK
                if cycle_idx == 0:
                    current_drift = DRIFT_PER_TICK * 0.5 # 0.5 bps -> Bootstrap
                elif cycle_idx == 1:
                    current_drift = DRIFT_PER_TICK * 4.0 # 4.0 bps -> Strategy
                else:
                    current_drift = -DRIFT_PER_TICK * 4.0 # -4.0 bps -> Strategy (Sell)
                    
                t["current_price"] *= (1 + current_drift + shock)
                t["trend_step"] += 1
                if t["trend_step"] >= TREND_LENGTH:
                    t["trend_step"] = 0
                    t["trend_dir"] *= -1
                
                batch.append({
                    "symbol": t["symbol"],
                    "timestamp": t["ts"],
                    "open": t["current_price"],
                    "high": t["current_price"],
                    "low": t["current_price"],
                    "close": t["current_price"],
                    "volume": 1000.0
                })
                t["ts"] += 300 # 5m
            
            print(json.dumps(batch), flush=True)
            # time.sleep(0.001) 
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)

if __name__ == "__main__":
    main()
