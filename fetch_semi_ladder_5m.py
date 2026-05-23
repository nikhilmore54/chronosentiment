import yfinance as yf
import json
import os
from datetime import datetime, timezone

events = {
    "weak": datetime(2026, 4, 7, 13, 30, tzinfo=timezone.utc),
    "moderate": datetime(2026, 5, 22, 13, 30, tzinfo=timezone.utc)
}

for strength, target_dt in events.items():
    target_ts = int(target_dt.timestamp() * 1000)
    end_ts = target_ts + (300 * 60 * 1000)

    for sym in ['NVDA', 'AMD']:
        output_dir = f"core/chronology/historical/2026_{sym.lower()}_sync_drop_{strength}_5m"
        os.makedirs(output_dir, exist_ok=True)
        df = yf.download(sym, period='60d', interval='5m')
        
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
