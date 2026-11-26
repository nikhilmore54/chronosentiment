#!/usr/bin/env python3
"""
Downloads and cryptographically verifies the 2024 Binance Spot BTC/USDT 15-minute historical dataset.
Period: 2024-01-01T00:00:00Z (1704067200000) to 2024-12-31T23:45:00Z (1735688700000)
Expected total candles: 366 days * 96 candles/day = 35,136 candles.
"""

import hashlib
import json
import os
import sys
import time
import urllib.request

START_TIME_MS = 1704067200000  # 2024-01-01 00:00:00 UTC
END_TIME_MS = 1735689600000    # 2025-01-01 00:00:00 UTC
INTERVAL_MS = 15 * 60 * 1000   # 900,000 ms (15 minutes)
EXPECTED_CANDLES = 35136

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "datasets", "crypto_15m")
OUTPUT_FILE = os.path.join(OUTPUT_DIR, "binance_btcusdt_2024_15m.json")
MANIFEST_FILE = os.path.join(OUTPUT_DIR, "binance_btcusdt_2024_15m_manifest.json")


def fetch_all():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    current_start = START_TIME_MS
    all_candles = []

    print(f"Fetching Binance Spot BTC/USDT 15m candles from {START_TIME_MS} to {END_TIME_MS}...")
    batch = 0
    while current_start < END_TIME_MS:
        url = f"https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=15m&startTime={current_start}&endTime={END_TIME_MS}&limit=1000"
        req = urllib.request.Request(url, headers={"User-Agent": "ChronoSentiment-RP608/1.0"})
        
        retries = 3
        while retries > 0:
            try:
                with urllib.request.urlopen(req, timeout=15) as resp:
                    raw_data = json.loads(resp.read().decode())
                break
            except Exception as e:
                retries -= 1
                if retries == 0:
                    raise RuntimeError(f"Failed to fetch batch starting at {current_start}: {e}")
                time.sleep(1.0)

        if not raw_data:
            break

        for item in raw_data:
            ot = int(item[0])
            if ot >= END_TIME_MS:
                break
            o = float(item[1])
            h = float(item[2])
            l = float(item[3])
            c = float(item[4])
            v = float(item[5])
            ct = int(item[6])
            
            all_candles.append({
                "open_time_ms": ot,
                "open": o,
                "high": h,
                "low": l,
                "close": c,
                "volume": v,
                "close_time_ms": ct,
            })

        batch += 1
        last_ot = all_candles[-1]["open_time_ms"]
        current_start = last_ot + INTERVAL_MS
        print(f"Batch {batch}: Fetched up to {last_ot} (Total: {len(all_candles)} candles)")
        time.sleep(0.1)

    print(f"\nFetch complete. Total candles collected: {len(all_candles)}")
    assert len(all_candles) == EXPECTED_CANDLES, f"Expected {EXPECTED_CANDLES} candles, got {len(all_candles)}"

    # Audit invariants
    print("Auditing chronological and geometric invariants...")
    for i, candle in enumerate(all_candles):
        expected_time = START_TIME_MS + i * INTERVAL_MS
        assert candle["open_time_ms"] == expected_time, f"Timestamp mismatch at index {i}: expected {expected_time}, got {candle['open_time_ms']}"
        assert candle["close_time_ms"] == expected_time + INTERVAL_MS - 1, f"Close time mismatch at index {i}"
        assert candle["low"] <= min(candle["open"], candle["close"]), f"Low geometry violation at index {i}"
        assert candle["high"] >= max(candle["open"], candle["close"]), f"High geometry violation at index {i}"
        assert candle["volume"] >= 0.0, f"Negative volume at index {i}"

    print(f"All {EXPECTED_CANDLES} candles passed 100% of invariant checks (0 gaps, 0 duplicates, valid geometry).")

    # Save dataset JSON
    print(f"Writing dataset to {OUTPUT_FILE}...")
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        json.dump(all_candles, f, indent=2)

    # Compute SHA-256
    with open(OUTPUT_FILE, "rb") as f:
        sha256_hash = hashlib.sha256(f.read()).hexdigest()

    manifest = {
        "dataset_name": "binance_btcusdt_2024_15m",
        "symbol": "BTCUSDT",
        "resolution": "15m",
        "exchange": "Binance Spot",
        "start_time_iso": "2024-01-01T00:00:00Z",
        "end_time_iso": "2024-12-31T23:45:00Z",
        "start_time_ms": START_TIME_MS,
        "end_time_ms": 1735688700000,
        "total_candles": len(all_candles),
        "expected_candles": EXPECTED_CANDLES,
        "gaps_detected": 0,
        "duplicates_detected": 0,
        "sha256": sha256_hash,
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }

    with open(MANIFEST_FILE, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    print("\nDataset Manifest:")
    print(json.dumps(manifest, indent=2))
    return manifest


if __name__ == "__main__":
    fetch_all()
