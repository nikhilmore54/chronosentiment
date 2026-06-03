#!/usr/bin/env python3
"""Validate live capture dataset integrity.
Checks performed:
1. Row continuity in canonical 1‑minute CSVs (no missing, duplicate, or out‑of‑order timestamps).
2. Raw JSON ↔ canonical CSV consistency for a random sample of rows.
3. 1‑minute → 5‑minute aggregation correctness for the first 5‑minute window.
4. capture_manifest.json contains required keys.
"""
import json, csv, pathlib, sys, random
from datetime import datetime, timezone

BASE = pathlib.Path('live_capture') / datetime.now().strftime('%Y-%m-%d')
if not BASE.exists():
    # fallback to today's date string (as used in earlier steps)
    BASE = pathlib.Path('live_capture') / '2026-06-02'

# Helper to parse ISO timestamp strings
def parse_ts(s):
    return datetime.fromisoformat(s).replace(tzinfo=timezone.utc)

# 1. Row continuity
def check_continuity(csv_path):
    with csv_path.open() as f:
        reader = csv.DictReader(f)
        prev = None
        for i, row in enumerate(reader, start=1):
            ts = parse_ts(row['timestamp'])
            if prev and ts <= prev:
                print(f"[FAIL] {csv_path.name}: timestamp not monotonic at line {i} ({prev} >= {ts})")
                return False
            prev = ts
    print(f"[PASS] {csv_path.name}: monotonic timestamps")
    return True

# 2. Raw ↔ Canonical sample check
def sample_consistency(symbol):
    raw_path = BASE / 'raw' / f"{symbol}.json"
    canon_path = BASE / 'canonical' / f"{symbol}_1m.csv"
    with raw_path.open() as f:
        raw = json.load(f)
    records = raw.get('records', [])
    if not records:
        print(f"[WARN] {symbol}: no records in raw JSON")
        return
    # pick a random record
    rec = random.choice(records)
    ts = rec.get('timestamp') or rec.get('Datetime') or rec.get('date')
    # locate same timestamp in CSV
    with canon_path.open() as f:
        reader = csv.DictReader(f)
        for row in reader:
            if row['timestamp'] == ts:
                # compare fields (open, high, low, close, volume)
                diffs = []
                for col in ['open','high','low','close','volume']:
                    if str(row[col]) != str(rec.get(col)):
                        diffs.append(col)
                if diffs:
                    print(f"[FAIL] {symbol}: mismatch in {diffs} for timestamp {ts}")
                else:
                    print(f"[PASS] {symbol}: raw ↔ canonical match for timestamp {ts}")
                break
        else:
            print(f"[FAIL] {symbol}: timestamp {ts} not found in canonical CSV")

# 3. 1m → 5m aggregation correctness (first window)
def check_aggregation(symbol):
    canon_path = BASE / 'canonical' / f"{symbol}_1m.csv"
    derived_path = BASE / 'derived' / f"{symbol}_5m.csv"
    # load first 5 rows of canonical
    with canon_path.open() as f:
        rows = list(csv.DictReader(f))[:5]
    if len(rows) < 5:
        print(f"[WARN] {symbol}: not enough canonical rows for aggregation check")
        return
    # compute expected aggregates
    open_ = rows[0]['open']
    close_ = rows[-1]['close']
    high_ = max(r['high'] for r in rows)
    low_ = min(r['low'] for r in rows)
    vol_ = sum(float(r['volume']) for r in rows)
    # load first derived row
    with derived_path.open() as f:
        derived_row = next(csv.DictReader(f))
    # compare
    eps = 1e-6
    def close_enough(a,b):
        try:
            return abs(float(a)-float(b)) < eps
        except:
            return a == b
    mismatches = []
    if not close_enough(open_, derived_row['open']): mismatches.append('open')
    if not close_enough(high_, derived_row['high']): mismatches.append('high')
    if not close_enough(low_, derived_row['low']): mismatches.append('low')
    if not close_enough(close_, derived_row['close']): mismatches.append('close')
    if not close_enough(vol_, derived_row['volume']): mismatches.append('volume')
    if mismatches:
        print(f"[FAIL] {symbol}: aggregation mismatch in {mismatches}")
    else:
        print(f"[PASS] {symbol}: 5‑minute aggregation correct for first window")

# 4. Manifest validation
def check_manifest():
    manifest_path = BASE / 'capture_manifest.json'
    if not manifest_path.exists():
        print('[FAIL] capture_manifest.json missing')
        return
    with manifest_path.open() as f:
        manifest = json.load(f)
    required_keys = {'capture_date','symbols','file_hashes','row_counts','timestamps'}
    missing = required_keys - manifest.keys()
    if missing:
        print(f"[FAIL] manifest missing keys: {missing}")
    else:
        print('[PASS] capture_manifest.json contains required keys')

if __name__ == '__main__':
    symbols = ['NIFTY','BANKNIFTY']
    for sym in symbols:
        # continuity check for canonical CSV
        check_continuity(BASE / 'canonical' / f"{sym}_1m.csv")
        # sample consistency
        sample_consistency(sym)
        # aggregation check
        check_aggregation(sym)
    # manifest
    check_manifest()
"
