#!/usr/bin/env python3
"""
ChronoSentiment — High-Fidelity Physics Backfiller
Fetches historical candles, aligns them, and pipes them into the live_observatory
to fill gaps in the physics_divergence.csv archive.
"""

import sys
import json
import time
import subprocess
from datetime import datetime, timedelta
from pathlib import Path

# Add project root to path
_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(_ROOT))

from scripts.fetch_candles import fetch_latest

def backfill(symbols, interval="1m", n_candles=1540):
    csv_path = _ROOT / "archive" / "physics_divergence.csv"
    existing_pairs = set()
    if csv_path.exists():
        try:
            import pandas as pd
            print(f"📖 Loading existing archive for warmup-aware incremental check...")
            df_existing = pd.read_csv(csv_path, header=None, usecols=[0, 1])
            existing_pairs = set(zip(df_existing[0], df_existing[1]))
            print(f"✅ Loaded {len(existing_pairs)} existing pairs.")
        except Exception as e:
            print(f"⚠️ Could not load archive: {e}")

    print(f"🚀 Starting backfill for {symbols} | Interval: {interval} | Window: {n_candles}")
    
    # 1. Fetch data
    all_data = {}
    for symbol in symbols:
        candles = fetch_latest(symbol, interval, n_candles)
        all_data[symbol] = sorted(candles, key=lambda c: c['timestamp'])
        print(f"✅ Received {len(candles)} candles for {symbol}")

    # 2. Synchronize: UNION
    # We allow asynchronous ingestion of every available candle.
    all_timestamps = set()
    for symbol in symbols:
        for c in all_data[symbol]:
            all_timestamps.add(c['timestamp'])
    
    if not all_timestamps:
        print("❌ No data found across all symbols. Check fetch logic.")
        return

    sorted_timestamps = sorted(list(all_timestamps))
    
    # Create lookup map for speed: {timestamp: {symbol: candle}}
    lookup = {ts: {} for ts in sorted_timestamps}
    for symbol in symbols:
        for c in all_data[symbol]:
            lookup[c['timestamp']][symbol] = c

    # 3. Warmup-Aware Ingestion: 
    # Find the earliest timestamp where at least one symbol is missing from the archive
    first_missing_idx = 0
    for i, ts in enumerate(sorted_timestamps):
        is_missing = False
        for symbol in symbols:
            if (ts, symbol) not in existing_pairs:
                is_missing = True
                break
        if is_missing:
            first_missing_idx = i
            break
    else:
        print("✅ Archive is already up to date. No new ticks to backfill.")
        return

    # Start 100 bars before the first missing tick to allow engine warmup
    start_idx = max(0, first_missing_idx - 100)
    ingestion_window = sorted_timestamps[start_idx:]
    
    print(f"🧩 Synchronized {len(sorted_timestamps)} timesteps (Union).")
    print(f"🔥 Warmup-Aware: Starting ingestion at index {start_idx} (100-bar lead-in).")

    # 4. Engine Process
    engine_cmd = ["cargo", "run", "--release", "--example", "live_observatory"]
    import os
    env = os.environ.copy()
    env["SOURCE_TYPE"] = "REPLAY"
    env["REPLAY_GENERATION"] = "1"

    process = subprocess.Popen(
        engine_cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
        bufsize=1,
        cwd=str(_ROOT),
        env=env
    )

    print(f"🏗️  Engine started. Replaying {len(ingestion_window)} timesteps...")
    
    try:
        emitted_ticks = 0
        for ts in ingestion_window:
            batch = []
            # We only send symbols that exist for this timestamp
            for symbol in symbols:
                if symbol in lookup[ts]:
                    c = lookup[ts][symbol]
                    batch.append({
                        "symbol": symbol, "timestamp": ts,
                        "open": c["open"], "high": c["high"],
                        "low": c["low"], "close": c["close"], "volume": c["volume"]
                    })
            
            if batch:
                process.stdin.write(json.dumps(batch) + "\n")
                process.stdin.flush()
                emitted_ticks += len(batch)

        process.stdin.close()
        print(f"🏁 Replay complete ({emitted_ticks} ticks sent). Waiting for engine...")
        process.wait()
        print("✅ Backfill complete.")

    except Exception as e:
        print(f"❌ Error: {e}")
        process.kill()

if __name__ == "__main__":
    symbols = ["BTC-USD", "ETH-USD", "SOL-USD"]
    # 1540 = 1440 (1 day) + 100 (warmup buffer)
    backfill(symbols, n_candles=1540)
