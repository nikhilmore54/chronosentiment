import json
import os
import glob
from datetime import datetime, timezone

def extract_window(input_dir, output_dir, target_ts, duration_minutes, prefix):
    os.makedirs(f"core/chronology/historical/{output_dir}", exist_ok=True)
    input_file = glob.glob(f"core/chronology/historical/{input_dir}/*.jsonl")[0]
    
    ticks = []
    with open(input_file, "r") as f:
        for line in f:
            ticks.append(json.loads(line))
            
    end_ts = target_ts + (duration_minutes * 60 * 1000)
    
    output_ticks = []
    for tick in ticks:
        if target_ts <= tick["timestamp"] < end_ts:
            output_ticks.append(tick)
            
    out_path = f"core/chronology/historical/{output_dir}/{prefix}_{target_ts}.jsonl"
    with open(out_path, "w") as f:
        for tick in output_ticks:
            f.write(json.dumps(tick) + "\n")
            
    print(f"Extracted {len(output_ticks)} ticks for {output_dir}")

# SPY Macro shock started at 16:53 UTC on May 21
target_ts = int(datetime(2026, 5, 21, 16, 53, tzinfo=timezone.utc).timestamp() * 1000)
extract_window("2026_spy_macro_1m", "2026_spy_macro_shock_1m", target_ts, 60, "spy")
