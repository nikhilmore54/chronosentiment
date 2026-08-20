#!/usr/bin/env python3
"""
TIME-009 Status Monitor — time009_status.py

Read-only status report for the TIME-009 prospective observation experiment.
Does NOT modify any observation artifacts.

Usage:
    python3 scripts/time009_status.py \
        --observations time_machine/analysis/TIME009/observations \
        --ledger       live_capture/ledger/entries
"""

import argparse
import json
import sys
from collections import Counter, defaultdict
from datetime import date, datetime, timezone
from pathlib import Path


def parse_args():
    p = argparse.ArgumentParser(description="TIME-009 status monitor (read-only)")
    p.add_argument(
        "--observations",
        default="time_machine/analysis/TIME009/observations",
        help="Directory containing TIME009-OBS-*.json artifacts",
    )
    p.add_argument(
        "--ledger",
        default="live_capture/ledger/entries",
        help="Directory containing LIVE-005 ledger entry JSON files",
    )
    return p.parse_args()


def load_observations(obs_dir: Path):
    obs = []
    if not obs_dir.exists():
        return obs
    for f in sorted(obs_dir.iterdir()):
        if f.suffix != ".json" or f.name == "latest_run.json":
            continue
        try:
            obs.append(json.loads(f.read_text()))
        except Exception as e:
            print(f"  WARN: cannot read {f.name}: {e}", file=sys.stderr)
    return obs


def load_ledger_entries(ledger_dir: Path):
    entries = []
    if not ledger_dir.exists():
        return entries
    for f in sorted(ledger_dir.iterdir()):
        if f.suffix != ".json":
            continue
        try:
            entries.append(json.loads(f.read_text()))
        except Exception as e:
            print(f"  WARN: cannot read {f.name}: {e}", file=sys.stderr)
    return entries


def fmt_ts(ts_str):
    if not ts_str:
        return "—"
    try:
        dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        return dt.strftime("%Y-%m-%d %H:%M UTC")
    except Exception:
        return ts_str[:19]


def main():
    args = parse_args()
    obs_dir = Path(args.observations)
    ledger_dir = Path(args.ledger)

    now_utc = datetime.now(timezone.utc)
    today = now_utc.date()

    print("=" * 60)
    print("TIME-009 PROSPECTIVE OBSERVATION STATUS")
    print(f"Report time: {now_utc.strftime('%Y-%m-%d %H:%M UTC')}")
    print("=" * 60)
    print()

    # ── Load data ─────────────────────────────────────────────────────────────
    observations = load_observations(obs_dir)
    ledger_entries = load_ledger_entries(ledger_dir)

    n_ledger = len(ledger_entries)
    n_obs = len(observations)

    pending = [o for o in observations if o.get("observation_status") == "PENDING"]
    complete = [o for o in observations if o.get("observation_status") == "COMPLETE"]
    other = [o for o in observations if o.get("observation_status") not in ("PENDING", "COMPLETE")]

    print(f"LIVE-005 ledger entries : {n_ledger}")
    print(f"TIME-009 artifacts      : {n_obs}")
    print(f"  PENDING               : {len(pending)}")
    print(f"  COMPLETE              : {len(complete)}")
    if other:
        print(f"  OTHER/ERROR           : {len(other)}")
    print()

    # ── Coverage check ────────────────────────────────────────────────────────
    ledger_ids = {e["decision_id"] for e in ledger_entries}
    obs_ids = {o["decision_id"] for o in observations}
    missing_obs = ledger_ids - obs_ids
    extra_obs = obs_ids - ledger_ids

    if missing_obs:
        print(f"WARNING: {len(missing_obs)} ledger entries have no observation artifact")
        for did in sorted(missing_obs)[:5]:
            print(f"  missing: {did}")
        if len(missing_obs) > 5:
            print(f"  ... and {len(missing_obs) - 5} more")
    else:
        print("Coverage: all ledger entries have observation artifacts ✓")

    if extra_obs:
        print(f"WARNING: {len(extra_obs)} observation artifacts have no ledger entry")

    print()

    # ── Cohort breakdown ──────────────────────────────────────────────────────
    cohort_counts = defaultdict(lambda: {"PENDING": 0, "COMPLETE": 0})
    for o in observations:
        cohort = o.get("cohort_date", "?")
        status = o.get("observation_status", "?")
        cohort_counts[cohort][status] = cohort_counts[cohort].get(status, 0) + 1

    print("Cohort breakdown:")
    print(f"  {'Cohort date':<14} {'PENDING':>8} {'COMPLETE':>10}")
    print(f"  {'-'*14} {'-'*8} {'-'*10}")
    for cohort in sorted(cohort_counts):
        c = cohort_counts[cohort]
        print(f"  {cohort:<14} {c.get('PENDING', 0):>8} {c.get('COMPLETE', 0):>10}")
    print()

    # ── Horizon distribution ──────────────────────────────────────────────────
    horizon_counts = Counter(
        int(o.get("horizon_sessions", 0)) for o in observations
    )
    print("Horizon distribution (sessions):")
    for h in sorted(horizon_counts):
        print(f"  horizon={h}: {horizon_counts[h]} decisions")
    print()

    # ── PENDING: sessions elapsed vs horizon ──────────────────────────────────
    if pending:
        print("PENDING decisions — sessions elapsed:")
        elapsed_dist = Counter()
        for o in pending:
            elapsed = o.get("n_bars_after_t0", 0)
            horizon = o.get("horizon_sessions", "?")
            elapsed_dist[(elapsed, horizon)] += 1
        for (elapsed, horizon), n in sorted(elapsed_dist.items()):
            print(f"  elapsed={elapsed}/{horizon}: {n} decisions")
        print()

        # Oldest pending
        oldest = min(pending, key=lambda o: o.get("admitted_at", ""))
        print(f"Oldest PENDING admitted_at : {fmt_ts(oldest.get('admitted_at'))}")
        print(f"Oldest PENDING ticker      : {oldest.get('ticker', '?')}")
        print(f"Oldest PENDING horizon     : {oldest.get('horizon_sessions', '?')} sessions")
        print()

    # ── COMPLETE: outcome summary ─────────────────────────────────────────────
    if complete:
        print("COMPLETE outcomes:")
        exit_counts = Counter(o.get("exit_reason", "?") for o in complete)
        for reason, n in sorted(exit_counts.items(), key=lambda x: -x[1]):
            print(f"  {reason}: {n}")
        n_eligible = sum(1 for o in complete if o.get("eligible_for_primary_comparison"))
        print(f"  eligible_for_primary_comparison: {n_eligible}/{len(complete)}")
        print()

        # Evidence class breakdown
        ec_counts = Counter(o.get("evidence_class", "?") for o in complete)
        print("COMPLETE by evidence_class:")
        for ec, n in sorted(ec_counts.items()):
            print(f"  {ec}: {n}")
        print()
    else:
        print("COMPLETE outcomes: none yet (waiting for horizons to elapse)")
        print()

    # ── Data availability ─────────────────────────────────────────────────────
    no_bars = [o for o in observations if o.get("n_bars_after_t0", 0) == 0]
    if no_bars:
        print(f"WARNING: {len(no_bars)} observations have 0 bars after T0")
        for o in no_bars[:5]:
            print(f"  {o.get('ticker', '?')} admitted={o.get('admitted_at', '?')[:10]}")
    else:
        print(f"Data availability: all observations have bars after T0 ✓")
    print()

    # ── Summary ───────────────────────────────────────────────────────────────
    print("=" * 60)
    if len(complete) == 0:
        print("STATUS: WAITING — no prospective outcomes yet")
        print("PENDING is not a failure. Horizons have not elapsed.")
    elif len(pending) > 0:
        print(f"STATUS: IN PROGRESS — {len(complete)} COMPLETE, {len(pending)} PENDING")
    else:
        print(f"STATUS: ALL COMPLETE — {len(complete)} observations")
    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())