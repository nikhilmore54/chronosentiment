import json
import os
import glob
from datetime import datetime, timezone

def extract_window(input_dir, output_dir, target_hour_utc, target_minute_utc, duration_minutes, prefix):
    os.makedirs(f"chronology/historical/{output_dir}", exist_ok=True)
    input_file = glob.glob(f"chronology/historical/{input_dir}/*.jsonl")[0]
    
    # We will find the latest day in the dataset
    ticks = []
    with open(input_file, "r") as f:
        for line in f:
            ticks.append(json.loads(line))
            
    if not ticks:
        print(f"No ticks found in {input_dir}")
        return
        
    last_tick_ts = ticks[-1]["timestamp"]
    last_dt = datetime.fromtimestamp(last_tick_ts / 1000, tz=timezone.utc)
    
    # Target date is the same date as the last tick
    target_dt = datetime(last_dt.year, last_dt.month, last_dt.day, target_hour_utc, target_minute_utc, tzinfo=timezone.utc)
    target_ts = int(target_dt.timestamp() * 1000)
    end_ts = target_ts + (duration_minutes * 60 * 1000)
    
    # If the target time hasn't happened on the last day, use the previous day
    if last_tick_ts < target_ts:
        target_dt = datetime(last_dt.year, last_dt.month, last_dt.day - 1, target_hour_utc, target_minute_utc, tzinfo=timezone.utc)
        target_ts = int(target_dt.timestamp() * 1000)
        end_ts = target_ts + (duration_minutes * 60 * 1000)
        
    output_ticks = []
    for tick in ticks:
        if target_ts <= tick["timestamp"] < end_ts:
            output_ticks.append(tick)
            
    out_path = f"chronology/historical/{output_dir}/{prefix}_{target_ts}.jsonl"
    with open(out_path, "w") as f:
        for tick in output_ticks:
            f.write(json.dumps(tick) + "\n")
            
    print(f"Extracted {len(output_ticks)} ticks for {output_dir}")

# AAPL open is 9:30 AM EST -> 13:30 UTC
# Let's extract 13:00 to 14:30 UTC (90 minutes)
extract_window("2026_aapl_overnight_gap_1m", "2026_aapl_open_auction_1m", 13, 0, 90, "aapl")

# RELIANCE open is 9:15 AM IST -> 03:45 UTC
# Let's extract 03:15 to 04:45 UTC (90 minutes)
extract_window("2026_reliance_overnight_gap_1m", "2026_reliance_open_auction_1m", 3, 15, 90, "reliance")
