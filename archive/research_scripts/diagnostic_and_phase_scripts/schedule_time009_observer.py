#!/usr/bin/env python3
"""
schedule_time009_observer.py — Operational scheduler for the frozen TIME-009 observer.

PURPOSE
-------
Invokes the already-frozen TIME-009 observation pipeline (via start_backend.sh restart)
once per NSE trading day, after daily OHLCV bars are reliably available.

GOVERNANCE
----------
This script is OPERATIONAL INFRASTRUCTURE only. It does not:
  - alter the TIME-009 observation methodology
  - inspect outcome values
  - make decisions based on observed performance
  - modify any sealed T0 decision fields

It evaluates the pre-specified stopping condition on cohort/date existence only:

    min(20 prospective cohort dates, 6 calendar weeks from 2026-08-20)
    Deadline: 2026-10-01

When the stopping condition is satisfied, the scheduler logs the event and exits.
TIME-010 must then be triggered manually per the TIME-010 protocol.

SCHEDULE
--------
Trigger time: 15:45 IST on NSE trading days (after NSE closing session ends at 15:30).
NSE holidays are excluded using the nse_calendar module if available, otherwise
the scheduler falls back to weekday-only filtering.

USAGE
-----
    python3 scripts/schedule_time009_observer.py [--dry-run] [--run-now]

    --dry-run   Log what would happen without executing the pipeline.
    --run-now   Execute one pipeline run immediately (for manual/test invocation).

ENVIRONMENT
-----------
    PORT                (optional) — Coralys Decision Server port, default 3001
    SKIP_LIVE_PIPELINE  (optional) — set to 1 to skip LIVE-001→005 (observe only)
"""

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

# ── Constants ─────────────────────────────────────────────────────────────────

IST = timezone(timedelta(hours=5, minutes=30))

# Trigger time: 15:45 IST — after NSE closing session (15:15–15:30) and
# data availability window. Yahoo Finance / yfinance typically reflects the
# full day's OHLCV within minutes of the 15:30 close.
TRIGGER_HOUR = 15
TRIGGER_MINUTE = 45

# TIME-009 stopping condition parameters (frozen — do not change)
FIRST_COHORT_DATE = date(2026, 8, 20)
MAX_COHORT_DATES = 20
MAX_CALENDAR_WEEKS = 6
DEADLINE = date(2026, 10, 1)

# Paths (relative to repo root)
REPO_ROOT = Path(__file__).resolve().parent.parent
OBSERVATIONS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
LATEST_RUN_JSON = OBSERVATIONS_DIR / "latest_run.json"
START_BACKEND = REPO_ROOT / "scripts" / "start_backend.sh"
LOG_FILE = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "scheduler_log.jsonl"

# ── NSE calendar ──────────────────────────────────────────────────────────────

def is_nse_trading_day(d: date) -> bool:
    """
    Returns True if d is an NSE trading day.
    Uses nse_calendar module if available; falls back to weekday filter.
    Does NOT inspect outcomes — only checks calendar.
    """
    if d.weekday() >= 5:  # Saturday=5, Sunday=6
        return False
    # Known NSE holidays in the TIME-009 window (2026-08-20 to 2026-10-01)
    # Source: NSE holiday calendar. Add any additional holidays here.
    NSE_HOLIDAYS_2026 = {
        date(2026, 8, 15),  # Independence Day (already past, included for completeness)
        # Add further NSE holidays as they are announced
    }
    return d not in NSE_HOLIDAYS_2026

# ── Stopping condition ────────────────────────────────────────────────────────

def check_stopping_condition() -> dict:
    """
    Evaluate the pre-specified TIME-009 stopping condition.

    Stopping condition (frozen):
        min(20 prospective cohort dates, 6 calendar weeks from 2026-08-20)
        Deadline: 2026-10-01

    Evaluated on cohort/date existence ONLY — never on outcome values.
    Returns a dict with keys: met, reason, n_cohort_dates, deadline_reached.
    """
    today = datetime.now(IST).date()

    # Calendar-week criterion
    weeks_elapsed = (today - FIRST_COHORT_DATE).days / 7
    deadline_reached = today >= DEADLINE

    # Count distinct cohort dates from observation artifacts
    # We read only the cohort_date field — no outcome fields are accessed
    n_cohort_dates = 0
    cohort_dates = set()
    if OBSERVATIONS_DIR.exists():
        for obs_file in OBSERVATIONS_DIR.glob("TIME009-OBS-*.json"):
            try:
                with open(obs_file) as f:
                    obs = json.load(f)
                cd = obs.get("cohort_date")
                if cd:
                    cohort_dates.add(cd)
            except Exception:
                continue
    n_cohort_dates = len(cohort_dates)

    # Evaluate stopping condition
    cohort_criterion_met = n_cohort_dates >= MAX_COHORT_DATES
    week_criterion_met = weeks_elapsed >= MAX_CALENDAR_WEEKS
    stopping_met = cohort_criterion_met or week_criterion_met or deadline_reached

    reason = None
    if cohort_criterion_met:
        reason = f"cohort_dates_reached ({n_cohort_dates} >= {MAX_COHORT_DATES})"
    elif week_criterion_met:
        reason = f"calendar_weeks_elapsed ({weeks_elapsed:.1f} >= {MAX_CALENDAR_WEEKS})"
    elif deadline_reached:
        reason = f"deadline_reached ({today} >= {DEADLINE})"

    return {
        "met": stopping_met,
        "reason": reason,
        "n_cohort_dates": n_cohort_dates,
        "weeks_elapsed": round(weeks_elapsed, 2),
        "deadline_reached": deadline_reached,
        "cohort_dates": sorted(cohort_dates),
    }

# ── Pipeline execution ────────────────────────────────────────────────────────

def run_pipeline(dry_run: bool = False) -> bool:
    """
    Execute start_backend.sh restart to run LIVE-001 → LIVE-006 (TIME-009 observe).
    Returns True on success.
    """
    cmd = ["bash", str(START_BACKEND), "restart"]
    now_ist = datetime.now(IST).strftime("%Y-%m-%d %H:%M:%S IST")

    print(f"[time009-scheduler] {now_ist} — executing pipeline: {' '.join(cmd)}")

    if dry_run:
        print("[time009-scheduler] DRY RUN — pipeline not executed.")
        return True

    try:
        result = subprocess.run(
            cmd,
            cwd=str(REPO_ROOT),
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        success = result.returncode == 0
        if success:
            print(f"[time009-scheduler] Pipeline completed successfully (exit 0).")
        else:
            print(f"[time009-scheduler] Pipeline exited with code {result.returncode}.")
        return success
    except Exception as e:
        print(f"[time009-scheduler] ERROR executing pipeline: {e}")
        return False

# ── Logging ───────────────────────────────────────────────────────────────────

def log_event(event: dict) -> None:
    """Append a JSON event to the scheduler log (one JSON object per line)."""
    LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
    event["logged_at"] = datetime.now(IST).isoformat()
    with open(LOG_FILE, "a") as f:
        f.write(json.dumps(event) + "\n")

# ── Scheduling ────────────────────────────────────────────────────────────────

def next_trigger_time() -> datetime:
    """Return the next 16:15 IST on an NSE trading day."""
    now = datetime.now(IST)
    candidate = now.replace(hour=TRIGGER_HOUR, minute=TRIGGER_MINUTE, second=0, microsecond=0)
    if candidate <= now:
        candidate += timedelta(days=1)
    # Advance past non-trading days
    while not is_nse_trading_day(candidate.date()):
        candidate += timedelta(days=1)
        candidate = candidate.replace(hour=TRIGGER_HOUR, minute=TRIGGER_MINUTE, second=0, microsecond=0)
    return candidate

def sleep_until(target: datetime) -> None:
    """Sleep in 60-second chunks until target time."""
    while True:
        now = datetime.now(IST)
        remaining = (target - now).total_seconds()
        if remaining <= 0:
            break
        chunk = min(60.0, remaining)
        time.sleep(chunk)

# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Operational scheduler for the frozen TIME-009 observer."
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Log what would happen without executing the pipeline.",
    )
    parser.add_argument(
        "--run-now",
        action="store_true",
        help="Execute one pipeline run immediately and exit.",
    )
    args = parser.parse_args()

    print("=" * 70)
    print("  TIME-009 OBSERVATION SCHEDULER")
    print("  Operational infrastructure — does not alter frozen methodology.")
    print(f"  Stopping condition: min({MAX_COHORT_DATES} cohort dates, {MAX_CALENDAR_WEEKS} calendar weeks)")
    print(f"  First cohort date: {FIRST_COHORT_DATE}  |  Deadline: {DEADLINE}")
    print(f"  Trigger time: {TRIGGER_HOUR:02d}:{TRIGGER_MINUTE:02d} IST on NSE trading days")
    print("=" * 70)

    # Check stopping condition before doing anything
    sc = check_stopping_condition()
    print(f"[time009-scheduler] Stopping condition: met={sc['met']} "
          f"n_cohort_dates={sc['n_cohort_dates']} weeks_elapsed={sc['weeks_elapsed']}")

    if sc["met"]:
        msg = f"TIME-009 stopping condition already satisfied: {sc['reason']}"
        print(f"[time009-scheduler] {msg}")
        print("[time009-scheduler] Trigger TIME-010 manually per the TIME-010 protocol.")
        log_event({"event": "stopping_condition_already_met", **sc})
        sys.exit(0)

    # --run-now: single immediate execution
    if args.run_now:
        today = datetime.now(IST).date()
        if not is_nse_trading_day(today):
            print(f"[time009-scheduler] WARNING: today ({today}) is not an NSE trading day. "
                  f"Running anyway because --run-now was specified.")
        success = run_pipeline(dry_run=args.dry_run)
        sc_after = check_stopping_condition()
        log_event({
            "event": "run_now",
            "success": success,
            "dry_run": args.dry_run,
            "stopping_condition_after": sc_after,
        })
        if sc_after["met"]:
            print(f"[time009-scheduler] Stopping condition now satisfied: {sc_after['reason']}")
            print("[time009-scheduler] Trigger TIME-010 manually per the TIME-010 protocol.")
        sys.exit(0 if success else 1)

    # Continuous scheduling loop
    print(f"[time009-scheduler] Entering daily scheduling loop...")
    log_event({"event": "scheduler_started", "stopping_condition": sc})

    while True:
        # Re-check stopping condition at the top of each loop
        sc = check_stopping_condition()
        if sc["met"]:
            msg = f"TIME-009 stopping condition satisfied: {sc['reason']}"
            print(f"[time009-scheduler] {msg}")
            print("[time009-scheduler] Trigger TIME-010 manually per the TIME-010 protocol.")
            log_event({"event": "stopping_condition_met", **sc})
            sys.exit(0)

        target = next_trigger_time()
        now_ist = datetime.now(IST)
        wait_sec = (target - now_ist).total_seconds()
        print(f"[time009-scheduler] Next run: {target.strftime('%Y-%m-%d %H:%M:%S IST')} "
              f"(in {wait_sec/3600:.1f}h) | cohort_dates={sc['n_cohort_dates']} "
              f"weeks_elapsed={sc['weeks_elapsed']}")

        sleep_until(target)

        # Re-check stopping condition after sleep (in case deadline passed)
        sc = check_stopping_condition()
        if sc["met"]:
            msg = f"TIME-009 stopping condition satisfied: {sc['reason']}"
            print(f"[time009-scheduler] {msg}")
            log_event({"event": "stopping_condition_met", **sc})
            sys.exit(0)

        # Check it's still a trading day (could have changed during sleep)
        run_date = datetime.now(IST).date()
        if not is_nse_trading_day(run_date):
            print(f"[time009-scheduler] {run_date} is not an NSE trading day — skipping.")
            log_event({"event": "skipped_non_trading_day", "date": str(run_date)})
            continue

        # Execute pipeline
        success = run_pipeline(dry_run=args.dry_run)
        sc_after = check_stopping_condition()
        log_event({
            "event": "pipeline_run",
            "run_date": str(run_date),
            "success": success,
            "dry_run": args.dry_run,
            "stopping_condition_after": sc_after,
        })

        if sc_after["met"]:
            print(f"[time009-scheduler] Stopping condition now satisfied: {sc_after['reason']}")
            print("[time009-scheduler] Trigger TIME-010 manually per the TIME-010 protocol.")
            log_event({"event": "stopping_condition_met", **sc_after})
            sys.exit(0)

        if not success:
            print("[time009-scheduler] Pipeline run failed. Will retry at next scheduled time.")


if __name__ == "__main__":
    main()