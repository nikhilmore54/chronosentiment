import json
import glob
import os

def filter_file(directory, start_time, end_time):
    files = glob.glob(f"core/chronology/historical/{directory}/*.jsonl")
    if not files: return
    file_path = files[0]
    filtered_lines = []
    
    with open(file_path, "r") as f:
        for line in f:
            data = json.loads(line)
            if start_time <= data["timestamp"] < end_time:
                filtered_lines.append(line)
                
    with open(file_path, "w") as f:
        f.writelines(filtered_lines)
    print(f"Filtered {directory} to {len(filtered_lines)} ticks.")

filter_file("2026_recent_crossfeed_1h_yahoo_1m", 1779285600000, 1779289200000)
filter_file("2026_recent_discontinuity_1h_yahoo_1m", 1779332400000, 1779336000000)
