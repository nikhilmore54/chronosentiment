#!/usr/bin/env python3
"""
Phase 4 — Timestamp-Locked Chronology Recovery Engine (Python shim)

Delegates all logic to the Rust `cs-ingest repair` subcommand.
This script is a thin CLI wrapper — no recovery logic lives here.

Invariant: T_chronology ⊥ S_ecology

Usage:
    # Queue a single repair request
    python3 scripts/repair_chronology_gaps.py queue \\
        --batch-id 3 --symbol RELIANCE.NS --target-ts 1779270900 \\
        --reason quorum_gap

    # Auto-detect gaps from live_session_steps.jsonl and queue them
    python3 scripts/repair_chronology_gaps.py detect \\
        --batch-id 902 --run-label crypto_24h

    # Process all pending repairs (timestamp-locked fetch + provenance write)
    python3 scripts/repair_chronology_gaps.py process --batch-id 902

    # Show repair queue status
    python3 scripts/repair_chronology_gaps.py status --batch-id 902
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

BINARY = Path("cs-ingest/target/release/cs-ingest")
ARCHIVE_ROOT = "state_archive"


def ensure_binary() -> None:
    if not BINARY.exists():
        print(f"❌ cs-ingest binary not found at {BINARY}")
        print("   Run: cd cs-ingest && cargo build --release")
        sys.exit(1)


def run_repair(args: list[str]) -> int:
    """Delegate to cs-ingest repair subcommand."""
    ensure_binary()
    cmd = [str(BINARY), "repair", "--archive-root", ARCHIVE_ROOT] + args
    result = subprocess.run(cmd)
    return result.returncode


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Phase 4 — Timestamp-Locked Chronology Recovery (delegates to cs-ingest)"
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    # queue
    p_queue = sub.add_parser("queue", help="Queue a single repair request")
    p_queue.add_argument("--batch-id", type=int, required=True)
    p_queue.add_argument("--symbol", required=True)
    p_queue.add_argument("--target-ts", type=int, required=True)
    p_queue.add_argument("--reason", default="manual")
    p_queue.add_argument("--bar-sec", type=int, default=300)
    p_queue.add_argument("--provider", default="yfinance")

    # detect
    p_detect = sub.add_parser("detect", help="Auto-detect gaps from live_session_steps.jsonl")
    p_detect.add_argument("--batch-id", type=int, required=True)
    p_detect.add_argument("--run-label", default="live")

    # process
    p_process = sub.add_parser("process", help="Process all pending repair requests")
    p_process.add_argument("--batch-id", type=int, required=True)

    # status
    p_status = sub.add_parser("status", help="Show repair queue status")
    p_status.add_argument("--batch-id", type=int, required=True)

    args = parser.parse_args()

    if args.cmd == "queue":
        return run_repair([
            "--batch-id", str(args.batch_id),
            "queue",
            "--symbol", args.symbol,
            "--target-ts", str(args.target_ts),
            "--reason", args.reason,
            "--bar-sec", str(args.bar_sec),
            "--provider", args.provider,
        ])
    elif args.cmd == "detect":
        return run_repair([
            "--batch-id", str(args.batch_id),
            "detect",
            "--run-label", args.run_label,
        ])
    elif args.cmd == "process":
        return run_repair([
            "--batch-id", str(args.batch_id),
            "process",
        ])
    elif args.cmd == "status":
        return run_repair([
            "--batch-id", str(args.batch_id),
            "status",
        ])
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())