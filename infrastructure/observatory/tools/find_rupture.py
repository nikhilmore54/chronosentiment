import json
import glob
from datetime import datetime, timezone

def find_max_rupture(input_dir):
    input_file = glob.glob(f"core/chronology/historical/{input_dir}/*.jsonl")[0]
    ticks = []
    with open(input_file, "r") as f:
        for line in f:
            ticks.append(json.loads(line))
            
    max_diff = 0
    best_window = []
    best_start = 0
    
    # scan for 60-minute continuous windows
    for i in range(len(ticks) - 60):
        window = ticks[i:i+60]
        # ensure it's a continuous session (no overnight gap inside the window)
        # 60 minutes should span roughly 60 * 60 * 1000 ms
        time_span = window[-1]["timestamp"] - window[0]["timestamp"]
        if time_span > 120 * 60 * 1000:
            continue # skipped overnight
            
        prices = [t["price"] for t in window]
        diff = max(prices) - min(prices)
        if diff > max_diff:
            max_diff = diff
            best_window = window
            best_start = window[0]["timestamp"]
            
    dt = datetime.fromtimestamp(best_start / 1000, tz=timezone.utc)
    print(f"{input_dir} Max Rupture: {max_diff:.2f} at {dt}")
    
    return best_start

find_max_rupture("2026_aapl_overnight_gap_1m")
find_max_rupture("2026_reliance_overnight_gap_1m")
