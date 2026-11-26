import yfinance as yf
import json
import os
import time
from datetime import datetime, timezone

target_dt = datetime(2026, 5, 15, 14, 55, tzinfo=timezone.utc)
target_ts = int(target_dt.timestamp() * 1000)
end_ts = target_ts + (300 * 60 * 1000)

for sym in ['NVDA', 'AMD']:
    output_dir = f"core/chronology/historical/2026_{sym.lower()}_sync_gap_fade_5m"
    os.makedirs(output_dir, exist_ok=True)
    
    success = False
    retries = 3
    while not success and retries > 0:
        df = yf.download(sym, period='60d', interval='5m')
        if df.empty:
            print(f"Empty df for {sym}, retrying...")
            retries -= 1
            time.sleep(2)
            continue
            
        success = True
        output_ticks = []
        for index, row in df.iterrows():
            ts = int(index.timestamp() * 1000)
            if target_ts <= ts <= end_ts:
                output_ticks.append({
                    "timestamp": ts,
                    "price": float(row['Close'].iloc[0] if hasattr(row['Close'], 'iloc') else row['Close']),
                    "volume": float(row['Volume'].iloc[0] if hasattr(row['Volume'], 'iloc') else row['Volume'])
                })
                
        out_path = f"{output_dir}/{sym.lower()}_{target_ts}.jsonl"
        with open(out_path, "w") as f:
            for tick in output_ticks:
                f.write(json.dumps(tick) + "\n")
                
        print(f"Extracted {len(output_ticks)} ticks to {out_path}")
