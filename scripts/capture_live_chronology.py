import requests
import json
import hashlib
from pathlib import Path
from datetime import datetime, timezone

def capture_binance_chronology(symbol="BTCUSDT", interval="1m", limit=4320):
    # Binance klines limit is 1000 per request, so we need to page if limit > 1000
    # For simplicity, we can fetch the last 1000 candles or implement paging.
    # 72 hours * 60 mins = 4320 candles.
    
    url = "https://api.binance.com/api/v3/klines"
    
    klines = []
    end_time = None
    
    remaining = limit
    while remaining > 0:
        fetch_limit = min(remaining, 1000)
        params = {
            "symbol": symbol,
            "interval": interval,
            "limit": fetch_limit
        }
        if end_time:
            params["endTime"] = end_time
            
        response = requests.get(url, params=params)
        response.raise_for_status()
        
        batch = response.json()
        if not batch:
            break
            
        klines = batch + klines
        end_time = batch[0][0] - 1 # prior to the first candle in this batch
        remaining -= len(batch)

    output_dir = Path("core/chronology")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    output_file = output_dir / "live_capture_0001.jsonl"
    
    hasher = hashlib.sha256()
    
    with open(output_file, "w") as f:
        for k in klines:
            # kline format: [open_time, open, high, low, close, volume, close_time, quote_asset_volume, trades, taker_buy_base, taker_buy_quote, ignore]
            event = {
                "symbol": symbol,
                "timestamp": k[0],
                "open": float(k[1]),
                "high": float(k[2]),
                "low": float(k[3]),
                "close": float(k[4]),
                "volume": float(k[5])
            }
            line = json.dumps(event) + "\n"
            f.write(line)
            hasher.update(line.encode("utf-8"))
            
    chronology_hash = hasher.hexdigest()
    
    manifest = {
        "substrate": symbol,
        "interval": interval,
        "total_klines": len(klines),
        "start_timestamp": klines[0][0],
        "end_timestamp": klines[-1][0],
        "chronology_hash": chronology_hash,
        "capture_time": datetime.now(timezone.utc).isoformat()
    }
    
    manifest_file = output_dir / "live_capture_0001_manifest.json"
    with open(manifest_file, "w") as f:
        json.dump(manifest, f, indent=4)
        
    print(f"Captured {len(klines)} chronology ticks for {symbol}.")
    print(f"Chronology Hash: {chronology_hash}")
    print(f"Output: {output_file}")

if __name__ == "__main__":
    capture_binance_chronology()
