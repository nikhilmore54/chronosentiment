import yfinance as yf
import json
import hashlib
import os

symbol = "BTC-USD"
name = "2026_crossfeed_state_disagreement_yahoo_1m"
base_dir = f"core/chronology/historical/{name}"
os.makedirs(base_dir, exist_ok=True)

ticker = yf.Ticker(symbol)
df = ticker.history(period="7d", interval="1m")

if df.empty:
    print("No Yahoo data found!")
    exit(1)

start_time = int(df.index[0].timestamp() * 1000)
end_time = int(df.index[-1].timestamp() * 1000)

file_path = os.path.join(base_dir, f"{symbol.lower().replace('-', '')}_{start_time}.jsonl")
hasher = hashlib.sha256()

with open(file_path, "w") as f:
    for index, row in df.iterrows():
        ts = int(index.timestamp() * 1000)
        tick = {
            "symbol": symbol,
            "timestamp": ts,
            "price": float(row["Close"]),
            "volume": float(row["Volume"]),
            "is_buyer_maker": False
        }
        line = json.dumps(tick) + "\n"
        f.write(line)
        hasher.update(line.encode('utf-8'))

hash_hex = hasher.hexdigest()

manifest = {
    "substrate": symbol,
    "capture_start": start_time,
    "capture_end": end_time,
    "total_ticks": len(df),
    "chronology_hash": hash_hex,
    "gaps": [],
    "provenance": "yfinance python OHLCV"
}

meta_path = os.path.join(base_dir, f"{name}_{start_time}_manifest.json")
with open(meta_path, "w") as f:
    json.dump(manifest, f, indent=4)

print(f"✅ Python Yahoo Historical Capture Complete: {len(df)} ticks")
print(f"   Hash: {hash_hex}")
print(f"   Directory: {base_dir}")

