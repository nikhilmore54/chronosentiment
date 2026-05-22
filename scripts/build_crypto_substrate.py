#!/usr/bin/env python3
import json
import gzip
import hashlib
from pathlib import Path
import urllib.request
from datetime import datetime

# Binance API endpoint for Klines (OHLCV)
BINANCE_KLINE_URL = "https://api.binance.com/api/v3/klines"

def fetch_binance_klines(symbol: str, interval: str, total_limit: int):
    records = []
    end_time = None
    
    while len(records) < total_limit:
        limit = min(1000, total_limit - len(records))
        url = f"{BINANCE_KLINE_URL}?symbol={symbol}&interval={interval}&limit={limit}"
        if end_time:
            url += f"&endTime={end_time}"
            
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            
        if not data:
            break
            
        batch = []
        for row in data:
            batch.append({
                "ts": int(row[0] / 1000),
                "open": float(row[1]),
                "high": float(row[2]),
                "low": float(row[3]),
                "close": float(row[4]),
                "volume": float(row[5])
            })
            
        # Binance returns data ascending.
        # Since we use endTime, we get the oldest chunks relative to endTime.
        records = batch + records
        
        # New end_time is the start of this batch minus 1ms
        end_time = data[0][0] - 1
        
    return records

def freeze_crypto_substrate(batch_id: int, symbol: str, interval: str, limit: int):
    print(f"🧊 FREEZING CRYPTO SUBSTRATE: BATCH {batch_id}")
    print(f"Venue      : Binance")
    print(f"Asset      : {symbol}")
    print(f"Resolution : {interval}")
    
    # 1. Fetch raw chronology
    print("⏳ Acquiring continuous chronological state...")
    try:
        records = fetch_binance_klines(symbol, interval, limit)
    except Exception as e:
        print(f"❌ Failed to acquire data from Binance: {e}")
        return
        
    if not records:
        print("❌ No records returned.")
        return
        
    print(f"✅ Downloaded {len(records)} continuous chronological events.")
    
    # 2. Deterministic normalization
    records.sort(key=lambda x: x["ts"])
    
    # 3. Create Immutable Replay Hash
    # Stringify in a strict deterministic order
    chronology_str = json.dumps(records, sort_keys=True).encode('utf-8')
    timeline_fingerprint = hashlib.sha256(chronology_str).hexdigest()[:16]
    
    # 4. Write to disk
    base_dir = Path(f"state_archive/batches/batch_{batch_id}")
    data_dir = base_dir / "data"
    data_dir.mkdir(parents=True, exist_ok=True)
    
    out_path = data_dir / f"{symbol.lower()}_frozen.csv.gz"
    with gzip.open(out_path, "wt", encoding="utf-8") as f:
        for rec in records:
            f.write(json.dumps(rec, sort_keys=True) + "\n")
            
    # Write metadata
    metadata = {
        "batch_id": batch_id,
        "venue": "Binance",
        "symbol": symbol,
        "resolution": interval,
        "total_ticks": len(records),
        "start_ts": records[0]["ts"],
        "end_ts": records[-1]["ts"],
        "timeline_fingerprint": timeline_fingerprint,
        "freeze_timestamp": datetime.utcnow().isoformat() + "Z",
        "contract": "CRYPTO_SUBSTRATE_CONTRACT_v1",
        "notes": "Immutable 24/7 continuous session data for long-memory ecology."
    }
    
    meta_path = base_dir / "manifest.json"
    with open(meta_path, "w") as f:
        json.dump(metadata, f, indent=2)
        
    print(f"💾 Immutable Substrate Written:")
    print(f"   Timeline Hash : {timeline_fingerprint}")
    print(f"   Start Time    : {datetime.utcfromtimestamp(records[0]['ts']).isoformat()}")
    print(f"   End Time      : {datetime.utcfromtimestamp(records[-1]['ts']).isoformat()}")
    print(f"   Location      : {out_path}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch", type=int, default=10001)
    parser.add_argument("--symbol", type=str, default="BTCUSDT")
    parser.add_argument("--interval", type=str, default="1m")
    # 72h continuous map at 1m resolution = 72 * 60 = 4320 ticks.
    parser.add_argument("--limit", type=int, default=4320)
    args = parser.parse_args()
    
    freeze_crypto_substrate(args.batch, args.symbol, args.interval, args.limit)
