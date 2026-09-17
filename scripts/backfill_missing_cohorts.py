#!/usr/bin/env python3
"""
backfill_missing_cohorts.py

Replays the LIVE-001 through LIVE-005 pipeline for missing cohort dates
by passing --now <historical_timestamp> to LIVE-001. This uses the Yahoo
cache to reconstruct exactly what the snapshot would have looked like at
market close on each missing date, preserving full temporality.

Missing cohort dates (Aug 20 – Sep 8):
  2026-08-24, 2026-08-25, 2026-08-26, 2026-08-28, 2026-08-31,
  2026-09-02, 2026-09-03, 2026-09-04, 2026-09-07

For each missing date, LIVE-001 is run with:
  --now <date>T10:00:00Z   (NSE market close = 15:30 IST = 10:00 UTC)

The Yahoo cache already contains all bars through Sep 8, so LIVE-001 will
correctly filter to bars <= T and compute reference_price, ATR-14, trend,
momentum, volatility as they were at that historical close.

GOVERNANCE NOTE: source_snapshot_timestamp = market close on that date.
The observer's temporal firewall (AC-T9-02) uses this as T0, so only
bars strictly AFTER that timestamp count as post-T0 bars. This is the
correct T0 for that day's decisions.
"""

import subprocess
import sys
import datetime
import os

CACHE_DIR = "live_capture/yahoo_cache"
SNAPSHOT_DIR = "live_capture/snapshots"
EVAL_DIR = "live_capture/evaluations"
RECOMMEND_DIR = "live_capture/recommendations"
CERTIFY_DIR = "live_capture/certifications"
LEDGER_DIR = "live_capture/ledger"
UNIVERSE_FILE = "datasets/universes/coralys_102_v1.json"
EMIT_URL = "http://localhost:3001"

# NSE market close: 15:30 IST = 10:00 UTC
MARKET_CLOSE_UTC = "10:00:00"

# Missing trading days to backfill
MISSING_DATES = [
    datetime.date(2026, 8, 24),
    datetime.date(2026, 8, 25),
    datetime.date(2026, 8, 26),
    datetime.date(2026, 8, 28),
    datetime.date(2026, 8, 31),
    datetime.date(2026, 9, 2),
    datetime.date(2026, 9, 3),
    datetime.date(2026, 9, 4),
    datetime.date(2026, 9, 7),
]


def run(cmd, desc, timeout=300):
    """Run a shell command, print last lines of output, return success bool."""
    print(f"  [{desc}] running...")
    result = subprocess.run(
        cmd, shell=True, capture_output=True, text=True, timeout=timeout
    )
    combined = (result.stdout + result.stderr).strip()
    last_lines = combined.split("\n")[-8:]
    for line in last_lines:
        if line.strip():
            print(f"    {line}")
    if result.returncode != 0:
        print(f"  [{desc}] FAILED (exit {result.returncode})")
        return False
    print(f"  [{desc}] OK")
    return True


def get_existing_cohort_dates():
    """Get set of dates that already have ledger entries."""
    import glob
    import json
    cohort_dates = set()
    for f in glob.glob(os.path.join(LEDGER_DIR, "entries", "*.json")):
        try:
            d = json.load(open(f))
            admitted = d.get("admitted_at", "")
            if admitted:
                dt = datetime.datetime.fromisoformat(admitted.replace("Z", "+00:00"))
                cohort_dates.add(dt.date())
        except Exception:
            pass
    return cohort_dates


def main():
    os.makedirs(SNAPSHOT_DIR, exist_ok=True)
    os.makedirs(EVAL_DIR, exist_ok=True)
    os.makedirs(RECOMMEND_DIR, exist_ok=True)
    os.makedirs(CERTIFY_DIR, exist_ok=True)
    os.makedirs(os.path.join(LEDGER_DIR, "entries"), exist_ok=True)

    existing = get_existing_cohort_dates()
    print(f"Existing cohort dates: {sorted(existing)}")

    dates_to_run = [d for d in MISSING_DATES if d not in existing]
    if not dates_to_run:
        print("All missing cohort dates already have ledger entries. Nothing to do.")
        return

    print(f"Dates to backfill: {dates_to_run}")

    for target_date in dates_to_run:
        now_str = f"{target_date}T{MARKET_CLOSE_UTC}Z"
        print(f"\n{'='*60}")
        print(f"Backfilling cohort for {target_date} ({target_date.strftime('%A')}) — now={now_str}")
        print(f"{'='*60}")

        # LIVE-001: replay snapshot at historical market close
        ok = run(
            f"CHRONO_YAHOO_CACHE_DIR={CACHE_DIR} "
            f"cargo run -p chronosentiment_adapter --bin live001_snapshot -- "
            f"--universe {UNIVERSE_FILE} "
            f"--output {SNAPSHOT_DIR} "
            f"--now {now_str} 2>&1 | tail -8",
            "LIVE-001"
        )
        if not ok:
            print(f"  Skipping {target_date} due to LIVE-001 failure.")
            continue

        # LIVE-002: evaluate snapshot
        ok = run(
            f"cargo run -p chronosentiment_adapter --bin live002_evaluate -- "
            f"--snapshot {SNAPSHOT_DIR}/latest.json "
            f"--output {EVAL_DIR} 2>&1 | tail -8",
            "LIVE-002"
        )
        if not ok:
            print(f"  Skipping {target_date} due to LIVE-002 failure.")
            continue

        # LIVE-003: generate recommendations
        ok = run(
            f"cargo run -p chronosentiment_adapter --bin live003_recommend -- "
            f"--state {EVAL_DIR}/latest.json "
            f"--output {RECOMMEND_DIR} 2>&1 | tail -5",
            "LIVE-003"
        )
        if not ok:
            print(f"  Skipping {target_date} due to LIVE-003 failure.")
            continue

        # LIVE-004: certify
        ok = run(
            f"cargo run -p chronosentiment_adapter --bin live004_certify -- "
            f"--snapshot {SNAPSHOT_DIR}/latest.json "
            f"--state {EVAL_DIR}/latest.json "
            f"--recommend {RECOMMEND_DIR}/latest.json "
            f"--output {CERTIFY_DIR} 2>&1 | tail -5",
            "LIVE-004"
        )
        if not ok:
            print(f"  Skipping {target_date} due to LIVE-004 failure.")
            continue

        # LIVE-005: admit to ledger
        ok = run(
            f"cargo run -p chronosentiment_adapter --bin live005_ledger -- "
            f"--certification {CERTIFY_DIR}/latest.json "
            f"--recommend {RECOMMEND_DIR}/latest.json "
            f"--ledger {LEDGER_DIR} "
            f"--audit {LEDGER_DIR}/audit "
            f"--emit-url {EMIT_URL} 2>&1 | tail -8",
            "LIVE-005"
        )
        if not ok:
            print(f"  Skipping {target_date} due to LIVE-005 failure.")
            continue

        print(f"  Cohort {target_date} admitted successfully.")

    print(f"\n{'='*60}")
    print("Backfill complete. Running time009_observe to process new cohorts...")
    run(
        f"cargo run -p chronosentiment_adapter --bin time009_observe -- "
        f"--ledger {LEDGER_DIR} "
        f"--output time_machine/analysis/TIME009/observations "
        f"--cache {CACHE_DIR} 2>&1 | grep -E 'result=|n_complete|n_pending|n_no_bars'",
        "LIVE-006 (time009_observe)"
    )

    print("\nFinal status:")
    subprocess.run("python3 scripts/time009_status.py", shell=True)


if __name__ == "__main__":
    main()