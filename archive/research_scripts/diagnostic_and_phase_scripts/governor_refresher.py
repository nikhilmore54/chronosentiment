#!/usr/bin/env python3
"""
ChronoSentiment — Minimal Production Governor
==============================================
Reads the last N telemetry records from the live archive and adjusts the
execution multiplier deterministically based on corridor_rate and
instability_rate.

Governor logic (deterministic threshold, no ML):
  if instability_rate > HALT_THRESHOLD   → multiplier=0.0, gate=closed
  elif corridor_rate > THROTTLE_THRESHOLD → multiplier=0.65, gate=open
  else                                    → multiplier=1.0,  gate=open

State is written atomically to analysis/real_live/governor_state.json
via os.replace(tmp, final) — safe for concurrent readers.

Usage:
  python3 scripts/governor_refresher.py
  python3 scripts/governor_refresher.py --archive-dir state_archive/batches/batch_003/runs/replay_equiv
  python3 scripts/governor_refresher.py --window 100 --interval 1.0
"""

from __future__ import annotations

import argparse
import gzip
import json
import os
import sys
import time
from pathlib import Path

# ── Thresholds ────────────────────────────────────────────────────────────────
# These are the starting calibration points. Tune after observing live runs.
HALT_THRESHOLD = 0.40       # instability_rate above this → full halt
THROTTLE_THRESHOLD = 0.20   # corridor_rate above this → throttle to 0.65
DEFAULT_WINDOW = 50         # number of recent records to evaluate
DEFAULT_INTERVAL = 0.5      # seconds between governor refresh cycles
DEFAULT_ARCHIVE_DIR = Path("state_archive")
BRIDGE_PATH = Path("analysis/real_live/governor_state.json")
TMP_PATH = Path("analysis/real_live/governor_state.json.tmp")


def _iter_recent_records(archive_dir: Path, window: int) -> list[dict]:
    """
    Collect the most recent `window` telemetry records from the raw archive.
    Reads latest.json files per symbol (O(symbols), not O(all records)).
    Falls back to scanning .jsonl.gz files if latest.json is absent.
    """
    records: list[dict] = []
    raw_dir = archive_dir / "raw"
    if not raw_dir.exists():
        return records

    # Fast path: each symbol writes a latest.json on every persist
    for sym_dir in raw_dir.iterdir():
        if not sym_dir.is_dir():
            continue
        latest = sym_dir / "latest.json"
        if latest.exists():
            try:
                rec = json.loads(latest.read_text())
                records.append(rec)
            except (json.JSONDecodeError, OSError):
                pass

    if len(records) >= window:
        # Sort by ts descending, take the most recent window records
        records.sort(key=lambda r: r.get("ts", 0), reverse=True)
        return records[:window]

    # Slow path: scan .jsonl.gz files for archives without latest.json
    for sym_dir in raw_dir.iterdir():
        if not sym_dir.is_dir():
            continue
        gz_files = sorted(sym_dir.glob("*.jsonl.gz"), reverse=True)
        for gz_path in gz_files[:2]:  # only scan the 2 most recent files
            try:
                with gzip.open(gz_path, "rt") as f:
                    for line in f:
                        line = line.strip()
                        if line:
                            try:
                                records.append(json.loads(line))
                            except json.JSONDecodeError:
                                pass
            except OSError:
                pass
            if len(records) >= window * 3:
                break

    records.sort(key=lambda r: r.get("ts", 0), reverse=True)
    return records[:window]


def compute_governor_state(records: list[dict]) -> dict:
    """
    Deterministic threshold logic over the observation window.
    Returns the governor state dict ready for JSON serialisation.
    """
    n = len(records)
    if n == 0:
        # No data — default to nominal, but flag the reason
        return {
            "multiplier": 1.0,
            "gate_open": True,
            "reason": "NO_DATA",
            "instability_rate": 0.0,
            "corridor_rate": 0.0,
            "window_size": 0,
            "ts": int(time.time()),
        }

    instability_count = sum(
        1 for r in records if r.get("instability_type") != "STABLE"
    )
    corridor_count = sum(1 for r in records if r.get("corridor", False))

    instability_rate = instability_count / n
    corridor_rate = corridor_count / n

    if instability_rate > HALT_THRESHOLD:
        multiplier = 0.0
        gate_open = False
        reason = f"HALT (instability_rate={instability_rate:.3f} > {HALT_THRESHOLD})"
    elif corridor_rate > THROTTLE_THRESHOLD:
        multiplier = 0.65
        gate_open = True
        reason = f"THROTTLE (corridor_rate={corridor_rate:.3f} > {THROTTLE_THRESHOLD})"
    else:
        multiplier = 1.0
        gate_open = True
        reason = f"NOMINAL (instability={instability_rate:.3f}, corridor={corridor_rate:.3f})"

    return {
        "multiplier": multiplier,
        "gate_open": gate_open,
        "reason": reason,
        "instability_rate": round(instability_rate, 4),
        "corridor_rate": round(corridor_rate, 4),
        "window_size": n,
        "ts": int(time.time()),
    }


def write_governor_state(state: dict) -> None:
    """Atomic write via tmp → replace. Safe for concurrent readers."""
    BRIDGE_PATH.parent.mkdir(parents=True, exist_ok=True)
    TMP_PATH.write_text(json.dumps(state, indent=2))
    os.replace(TMP_PATH, BRIDGE_PATH)


def run_governor(archive_dir: Path, window: int, interval: float) -> None:
    print(f"[governor] starting — archive={archive_dir} window={window} interval={interval}s")
    print(f"[governor] thresholds — halt>{HALT_THRESHOLD} throttle>{THROTTLE_THRESHOLD}")
    print(f"[governor] bridge → {BRIDGE_PATH}")

    prev_reason = None
    while True:
        try:
            records = _iter_recent_records(archive_dir, window)
            state = compute_governor_state(records)
            write_governor_state(state)

            # Only log on state transitions to avoid noise
            if state["reason"] != prev_reason:
                ts_str = time.strftime("%H:%M:%S")
                print(
                    f"[{ts_str}] governor → mult={state['multiplier']:.2f} "
                    f"gate={'OPEN' if state['gate_open'] else 'CLOSED'} | {state['reason']}"
                )
                prev_reason = state["reason"]

        except Exception as exc:
            print(f"[governor] error: {exc}", file=sys.stderr)

        time.sleep(interval)


def main() -> None:
    parser = argparse.ArgumentParser(description="ChronoSentiment minimal production governor")
    parser.add_argument(
        "--archive-dir",
        type=Path,
        default=DEFAULT_ARCHIVE_DIR,
        help="Root of the telemetry archive (default: state_archive)",
    )
    parser.add_argument(
        "--window",
        type=int,
        default=DEFAULT_WINDOW,
        help=f"Number of recent records to evaluate (default: {DEFAULT_WINDOW})",
    )
    parser.add_argument(
        "--interval",
        type=float,
        default=DEFAULT_INTERVAL,
        help=f"Refresh interval in seconds (default: {DEFAULT_INTERVAL})",
    )
    parser.add_argument(
        "--once",
        action="store_true",
        help="Evaluate once and exit (useful for testing)",
    )
    args = parser.parse_args()

    if args.once:
        records = _iter_recent_records(args.archive_dir, args.window)
        state = compute_governor_state(records)
        write_governor_state(state)
        print(json.dumps(state, indent=2))
        return

    run_governor(args.archive_dir, args.window, args.interval)


if __name__ == "__main__":
    main()
