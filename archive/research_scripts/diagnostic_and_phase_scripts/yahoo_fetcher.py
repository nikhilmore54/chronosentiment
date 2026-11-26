import yfinance as yf
import argparse
import os
import json
import hashlib
import time

parser = argparse.ArgumentParser()
parser.add_argument("--symbol", default="BTC-USD")
parser.add_argument("--name", required=True)
args = parser.parse_args()

print(f"Starting Python Yahoo Importer for {args.symbol} ({args.name})")

data = yf.download(args.symbol, period='7d', interval='1m')

if len(data) == 0:
    print("No quotes returned.")
    exit(1)

base_dir = os.path.join("chronology", "historical", args.name)
os.makedirs(base_dir, exist_ok=True)

# data index is datetime
start_time = int(data.index[0].timestamp() * 1000)
end_time = int(data.index[-1].timestamp() * 1000)

file_path = os.path.join(base_dir, f"{args.symbol.lower().replace('-', '')}_{start_time}.jsonl")

gaps = []
total_ticks = 0
hasher = hashlib.sha256()

with open(file_path, "w") as f:
    for idx, row in data.iterrows():
        ts = int(idx.timestamp() * 1000)
        # Extract scalar values from the pandas series
        try:
            price = float(row[('Close', args.symbol)]) if isinstance(data.columns, __import__('pandas').MultiIndex) else float(row['Close'])
            volume = float(row[('Volume', args.symbol)]) if isinstance(data.columns, __import__('pandas').MultiIndex) else float(row['Volume'])
        except KeyError:
            price = float(row.iloc[3])
            volume = float(row.iloc[4])
        
        tick = {
            "symbol": args.symbol,
            "timestamp": ts,
            "price": price,
            "volume": volume,
            "is_buyer_maker": False
        }
        line = json.dumps(tick) + "\n"
        f.write(line)
        hasher.update(line.encode())
        total_ticks += 1

hash_hex = hasher.hexdigest()

manifest = {
    "substrate": args.symbol,
    "capture_start": start_time,
    "capture_end": end_time,
    "total_ticks": total_ticks,
    "chronology_hash": hash_hex,
    "gaps": gaps,
    "provenance": "Python Yahoo API (7-day rolling)"
}

with open(os.path.join(base_dir, f"{args.symbol.lower().replace('-', '')}_{start_time}_manifest.json"), "w") as mf:
    json.dumps(manifest)
    mf.write(json.dumps(manifest, indent=2))

print(f"✅ Python Historical Capture Complete: {total_ticks} ticks")
print(f"   Hash: {hash_hex}")
print(f"   Directory: {base_dir}")
