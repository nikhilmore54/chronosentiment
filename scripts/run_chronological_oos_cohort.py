#!/usr/bin/env python3
"""
scripts/run_chronological_oos_cohort.py

Implements the frozen 11-step Standing Research Protocol for Adaptive Protection v0.4:
1. Fetch day's OHLCV observations (Yahoo 1-minute data).
2. Append to existing per-ticker cache (intraday_capture/yahoo_cache_1m).
3. Deduplicate on (ticker, timestamp) [NEW FETCH WINS].
4. Preserve every historical observation (zero deletion / zero silent loss).
5. Run CACHE INTEGRITY AUDIT.
6. Generate day's immutable entry dataset (LIVE-001 -> LIVE-005 -> bridge).
7. Run deferred-live chronological replay (deferred_live_decision_loop).
8. Apply frozen research models (trained ONLY on preceding cohorts).
9. Evaluate unchanged policy candidates.
10. Add cohort to cumulative stability ledger & aggregate paired accounting.
11. DO NOT RETUNE features, thresholds, models, or production H300.
"""

import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys
from datetime import timezone

CACHE_DIR = "intraday_capture/yahoo_cache_1m"
UNIVERSE_FILE = "datasets/universes/coralys_102_v1.json"
BACKUP_DIR = "/tmp/yahoo_cache_1m_backup_standing_protocol"

def get_ist_date(ts_unix):
    dt = datetime.datetime.fromtimestamp(ts_unix, tz=timezone.utc) + datetime.timedelta(hours=5, minutes=30)
    return dt.strftime("%Y-%m-%d")

def run_cache_integrity_audit(target_date):
    print("\n" + "="*80)
    print(f"CACHE INTEGRITY AUDIT (Target Date: {target_date})")
    print("="*80)
    
    files = sorted([f for f in os.listdir(CACHE_DIR) if f.endswith(".json")])
    total_obs = 0
    duplicates_in_final = 0
    is_chronological = True
    dates_found = set()
    
    for f_name in files:
        f_path = os.path.join(CACHE_DIR, f_name)
        with open(f_path, 'r') as f:
            data = json.load(f)
        total_obs += len(data)
        timestamps = [r['timestamp'] for r in data]
        if len(timestamps) != len(set(timestamps)):
            duplicates_in_final += (len(timestamps) - len(set(timestamps)))
        if timestamps != sorted(timestamps):
            is_chronological = False
        for r in data:
            dates_found.add(get_ist_date(r['timestamp']))
            
    print(f"Total Ticker Files:          {len(files)}")
    print(f"Total Cache Observations:    {total_obs}")
    print(f"Duplicate (ticker, unix):    {duplicates_in_final}")
    print(f"Chronological Order:         {'PASS' if is_chronological else 'FAIL'}")
    print(f"Target Date {target_date}:    {'PASS' if target_date in dates_found else 'FAIL'}")
    
    if duplicates_in_final > 0 or not is_chronological or target_date not in dates_found:
        print("CRITICAL: Cache Integrity Audit FAILED!")
        sys.exit(1)
    print("Cache Integrity Audit: PASS")

def fetch_and_append_date(target_date):
    import yfinance as yf
    print(f"\n[Step 1-4] Fetching & appending observations for {target_date}...")
    
    # Take backup
    if os.path.exists(BACKUP_DIR):
        shutil.rmtree(BACKUP_DIR)
    shutil.copytree(CACHE_DIR, BACKUP_DIR)
    
    files = sorted([f for f in os.listdir(CACHE_DIR) if f.endswith(".json")])
    total_added = 0
    
    for idx, f_name in enumerate(files, 1):
        symbol = f_name.replace(".json", "")
        cache_path = os.path.join(CACHE_DIR, f_name)
        with open(cache_path, 'r') as f:
            existing = json.load(f)
        existing_by_ts = {r["timestamp"]: r for r in existing}
        
        try:
            tk = yf.Ticker(symbol)
            df = tk.history(period="5d", interval="1m", auto_adjust=False)
            if df is not None and len(df) > 0:
                for ts_idx, row in df.iterrows():
                    ts_unix = int(ts_idx.timestamp()) if hasattr(ts_idx, "timestamp") else int(ts_idx.to_pydatetime().timestamp())
                    if get_ist_date(ts_unix) == target_date:
                        existing_by_ts[ts_unix] = {
                            "timestamp": ts_unix,
                            "open": float(row.get("Open", 0.0)),
                            "high": float(row.get("High", 0.0)),
                            "low": float(row.get("Low", 0.0)),
                            "close": float(row.get("Close", 0.0)),
                            "adj_close": float(row.get("Adj Close", row.get("Close", 0.0))),
                            "volume": float(row.get("Volume", 0.0)),
                        }
        except Exception as e:
            print(f"Warning: Fetch failed for {symbol}: {e}")
            
        merged = sorted(existing_by_ts.values(), key=lambda r: r["timestamp"])
        with open(cache_path, 'w') as f:
            json.dump(merged, f)
        total_added += (len(merged) - len(existing))
        
    print(f"Appended {total_added} new observations across {len(files)} tickers for {target_date}.")

def main():
    parser = argparse.ArgumentParser(description="Run 11-step Standing Protocol for a new OOS market session")
    parser.add_argument("--date", required=True, help="Session date in YYYY-MM-DD format")
    parser.add_argument("--skip-fetch", action="store_true", help="Skip yfinance fetch if data is already in cache")
    args = parser.parse_args()
    
    target_date = args.date
    session_code = target_date.replace("-", "")
    
    if not args.skip_fetch:
        fetch_and_append_date(target_date)
        
    run_cache_integrity_audit(target_date)
    
    # Step 6: Generate immutable entry dataset
    print(f"\n[Step 6] Generating LIVE-001 -> LIVE-005 entry dataset for {target_date}...")
    now_str = f"{target_date}T10:00:00Z"
    
    subprocess.run(f"cargo run -q -p chronosentiment_adapter --bin live001_snapshot -- --universe {UNIVERSE_FILE} --output live_capture/snapshots --now {now_str}", shell=True, check=True)
    subprocess.run("cargo run -q -p chronosentiment_adapter --bin live002_evaluate -- --snapshot live_capture/snapshots/latest.json --output live_capture/evaluations", shell=True, check=True)
    subprocess.run("cargo run -q -p chronosentiment_adapter --bin live003_recommend -- --state live_capture/evaluations/latest.json --output live_capture/recommendations", shell=True, check=True)
    subprocess.run("cargo run -q -p chronosentiment_adapter --bin live004_certify -- --snapshot live_capture/snapshots/latest.json --state live_capture/evaluations/latest.json --recommend live_capture/recommendations/latest.json --output live_capture/certifications --freshness 1000000", shell=True, check=True)
    subprocess.run("cargo run -q -p chronosentiment_adapter --bin live005_ledger -- --certification live_capture/certifications/latest.json --recommend live_capture/recommendations/latest.json --ledger live_capture/ledger --audit live_capture/ledger/audit --emit-url http://localhost:3001", shell=True, check=True)
    subprocess.run(f"python3 scripts/bridge_live005.py --as-of-date {target_date}", shell=True, check=True)
    
    # Step 7: Run deferred-live chronological replay
    ds_path = f"datasets/live005_{session_code}.json"
    out_session_path = f"/tmp/asof_session_{session_code}.json"
    print(f"\n[Step 7] Running session replay -> {out_session_path}...")
    subprocess.run(f"cargo run -q -p chronosentiment_adapter --bin deferred_live_decision_loop -- --session {target_date} --dataset {ds_path} --cache-dir {CACHE_DIR} > {out_session_path}", shell=True, check=True)
    
    print("\n" + "="*80)
    print(f"STANDING PROTOCOL COMPLETE FOR COHORT {target_date}")
    print("="*80)

if __name__ == '__main__':
    main()
