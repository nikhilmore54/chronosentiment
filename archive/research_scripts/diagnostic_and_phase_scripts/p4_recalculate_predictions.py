#!/usr/bin/env python3
"""
p4_recalculate_predictions.py

Recalculates LIVE-003 → LIVE-004 → LIVE-005 for all Aug 24+ cohorts
using the now-populated evidence store (102 .jsonl files).

GOVERNANCE:
- Original NoTrade ledger entries are archived to live_capture/ledger/entries_original_notrade/
- Recalculated entries are written to live_capture/ledger/entries/ (standard path)
- Frozen P4 thresholds are NOT modified
- Information-set boundary is preserved: each cohort uses only its own LIVE-002 state

PRECONDITION:
- datasets/recommendation/historical/ must contain 102 .jsonl files
- Backend server must be running on :3001
- LIVE-001 and LIVE-002 artifacts must exist for each cohort date

Usage:
    python3 scripts/p4_recalculate_predictions.py
"""

import subprocess
import sys
import json
import os
import shutil
import datetime
from pathlib import Path

LEDGER_DIR = Path("live_capture/ledger")
RECOMMEND_DIR = Path("live_capture/recommendations")
CERTIFY_DIR = Path("live_capture/certifications")
EVAL_DIR = Path("live_capture/evaluations")
SNAPSHOT_DIR = Path("live_capture/snapshots")
EVIDENCE_DIR = Path("datasets/recommendation/historical")
EMIT_URL = "http://localhost:3001"

# All Aug 24+ cohort dates that need recalculation
COHORT_DATES = [
    "20260824-1000",
    "20260825-1000",
    "20260826-1000",
    "20260827-0826",
    "20260827-1015",
    "20260828-1000",
    "20260831-1000",
    "20260901-1000",
    "20260902-1000",
    "20260903-1000",
    "20260904-1000",
    "20260907-1000",
    "20260908-1000",
    "20260909-1000",
    "20260910-1015",
]


def run(cmd, desc, timeout=120):
    result = subprocess.run(
        cmd, shell=True, capture_output=True, text=True, timeout=timeout
    )
    combined = (result.stdout + result.stderr).strip()
    last_lines = combined.split("\n")[-6:]
    for line in last_lines:
        if line.strip():
            print(f"    {line}")
    if result.returncode != 0:
        print(f"  [{desc}] FAILED (exit {result.returncode})")
        return False
    print(f"  [{desc}] OK")
    return True


def preflight():
    """Verify preconditions before running."""
    # Check evidence store
    jsonl_files = list(EVIDENCE_DIR.glob("*.jsonl"))
    if len(jsonl_files) < 102:
        print(f"PREFLIGHT FAIL: evidence store has {len(jsonl_files)} .jsonl files, need 102")
        sys.exit(1)
    print(f"[preflight] evidence store: {len(jsonl_files)} .jsonl files ✓")

    # Check backend
    result = subprocess.run(
        "curl -sf http://localhost:3001/decisions",
        shell=True, capture_output=True, text=True, timeout=5
    )
    if result.returncode != 0:
        print("PREFLIGHT FAIL: backend server not responding on :3001")
        sys.exit(1)
    total = json.loads(result.stdout).get("total", 0)
    print(f"[preflight] backend server: running, {total} decisions ✓")


def archive_original_entries():
    """Archive original NoTrade entries before overwriting."""
    archive_dir = LEDGER_DIR / "entries_original_notrade"
    archive_dir.mkdir(exist_ok=True)

    entries_dir = LEDGER_DIR / "entries"
    archived = 0
    for p in entries_dir.glob("*.json"):
        e = json.loads(p.read_text())
        admitted = e.get("admitted_at", "")
        if admitted >= "2026-08-24":
            dest = archive_dir / p.name
            if not dest.exists():
                shutil.copy2(p, dest)
                archived += 1
    print(f"[archive] Archived {archived} original Aug 24+ NoTrade entries to {archive_dir}")
    return archived


def get_cohort_snapshot_id(cohort_ts):
    """Get the LIVE-001 snapshot ID for a cohort timestamp."""
    snap = SNAPSHOT_DIR / f"LIVE-{cohort_ts}.json"
    if snap.exists():
        return str(snap)
    return None


def recalculate_cohort(cohort_ts):
    """Re-run LIVE-003 → LIVE-004 → LIVE-005 for one cohort."""
    print(f"\n{'='*60}")
    print(f"Recalculating cohort: {cohort_ts}")
    print(f"{'='*60}")

    eval_file = EVAL_DIR / f"LIVE-002-{cohort_ts}.json"
    snap_file = SNAPSHOT_DIR / f"LIVE-{cohort_ts}.json"

    if not eval_file.exists():
        print(f"  SKIP: LIVE-002 artifact not found: {eval_file}")
        return False
    if not snap_file.exists():
        print(f"  SKIP: LIVE-001 snapshot not found: {snap_file}")
        return False

    # LIVE-003: re-run with evidence store populated
    ok = run(
        f"cargo run -p chronosentiment_adapter --bin live003_recommend --release -- "
        f"--state {eval_file} "
        f"--output {RECOMMEND_DIR} 2>&1 | grep -E 'evidence_store_n_files|n_watch|n_buy|n_sell|n_no_trade|artifact'",
        "LIVE-003"
    )
    if not ok:
        return False

    rec_file = RECOMMEND_DIR / f"LIVE-003-{cohort_ts}.json"
    if not rec_file.exists():
        print(f"  SKIP: LIVE-003 artifact not written: {rec_file}")
        return False

    # LIVE-004: certify
    ok = run(
        f"cargo run -p chronosentiment_adapter --bin live004_certify --release -- "
        f"--snapshot {snap_file} "
        f"--state {eval_file} "
        f"--recommend {rec_file} "
        f"--output {CERTIFY_DIR} 2>&1 | tail -5",
        "LIVE-004"
    )
    if not ok:
        return False

    cert_file = CERTIFY_DIR / f"LIVE-004-{cohort_ts}.json"
    if not cert_file.exists():
        print(f"  SKIP: LIVE-004 artifact not written: {cert_file}")
        return False

    # Remove existing ledger entries for this cohort so LIVE-005 can write new ones
    entries_dir = LEDGER_DIR / "entries"
    removed = 0
    for p in entries_dir.glob(f"LIVE-005-{cohort_ts}-*.json"):
        p.unlink()
        removed += 1
    if removed:
        print(f"  Removed {removed} existing ledger entries for {cohort_ts}")

    # LIVE-005: admit to ledger
    ok = run(
        f"cargo run -p chronosentiment_adapter --bin live005_ledger --release -- "
        f"--certification {cert_file} "
        f"--recommend {rec_file} "
        f"--ledger {LEDGER_DIR} "
        f"--audit {LEDGER_DIR}/audit "
        f"--emit-url {EMIT_URL} 2>&1 | tail -8",
        "LIVE-005"
    )
    if not ok:
        return False

    print(f"  Cohort {cohort_ts} recalculated successfully.")
    return True


def summarize_results():
    """Count Watch decisions in the recalculated Aug 24+ ledger."""
    from collections import Counter
    entries_dir = LEDGER_DIR / "entries"
    actions = Counter()
    directions_x_actions = Counter()
    watch_by_date = Counter()

    for p in entries_dir.glob("*.json"):
        try:
            e = json.loads(p.read_text())
            admitted = e.get("admitted_at", "")
            if admitted < "2026-08-24":
                continue
            action = e.get("action", "UNKNOWN")
            direction = e.get("direction", "UNKNOWN")
            actions[action] += 1
            directions_x_actions[(direction, action)] += 1
            if action == "Watch":
                date = admitted[:10]
                watch_by_date[date] += 1
        except Exception:
            pass

    print(f"\n{'='*60}")
    print("RECALCULATION SUMMARY — Aug 24+ cohorts")
    print(f"{'='*60}")
    print(f"Total decisions: {sum(actions.values())}")
    print(f"\nAction breakdown:")
    for k, v in actions.most_common():
        print(f"  {k}: {v}")
    print(f"\nDirection × Action:")
    for k, v in sorted(directions_x_actions.items(), key=lambda x: -x[1]):
        print(f"  {k[0]} × {k[1]}: {v}")
    print(f"\nWatch decisions by date:")
    for date in sorted(watch_by_date):
        print(f"  {date}: {watch_by_date[date]}")
    print(f"\nTotal Watch: {actions.get('Watch', 0)}")


def main():
    print("P4 PREDICTION RECALCULATION")
    print("="*60)
    print(f"Timestamp: {datetime.datetime.utcnow().isoformat()}Z")
    print()

    preflight()
    archive_original_entries()

    success = 0
    failed = 0
    for cohort_ts in COHORT_DATES:
        if recalculate_cohort(cohort_ts):
            success += 1
        else:
            failed += 1

    print(f"\n{'='*60}")
    print(f"Recalculation complete: {success} succeeded, {failed} failed")

    summarize_results()

    # Run time009_observe to update observation statuses
    print(f"\n{'='*60}")
    print("Running time009_observe to update observation statuses...")
    run(
        f"cargo run -p chronosentiment_adapter --bin time009_observe --release -- "
        f"--ledger {LEDGER_DIR} "
        f"--output time_machine/analysis/TIME009/observations "
        f"--cache live_capture/yahoo_cache 2>&1 | grep -E 'result=|n_complete|n_pending|n_no_bars|COMPLETE|PENDING' | head -20",
        "time009_observe"
    )

    print("\nFinal pipeline status:")
    subprocess.run("python3 scripts/time009_status.py", shell=True)


if __name__ == "__main__":
    main()