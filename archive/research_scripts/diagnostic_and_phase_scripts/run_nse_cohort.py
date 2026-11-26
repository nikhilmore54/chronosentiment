#!/usr/bin/env python3
"""
ChronoSentiment — Canonical Substrate Ingestion Engine (Rust Backend)
======================================================================
Delegates all ingestion telemetry and archive generation to the Rust `cs-ingest` binary.
Removes legacy Python-side telemetry hydration and execution abstractions.

Usage:
    python3 scripts/run_nse_cohort.py --batch-id 3 --fresh
"""

import os
import sys
import json
import time
import argparse
import subprocess
from pathlib import Path

# --- Global Configurations ---
DEFAULT_ARCHIVE_ROOT = Path("state_archive")

def resolve_archive_dir(batch_id: int, shared_archive: bool, run_label: str = "") -> Path:
    if shared_archive:
        return DEFAULT_ARCHIVE_ROOT
    base = DEFAULT_ARCHIVE_ROOT / "batches" / f"batch_{batch_id:03d}"
    if run_label:
        return base / "runs" / run_label
    return base

def cs_ingest_binary() -> Path:
    return Path(__file__).resolve().parents[1] / "target" / "release" / "cs-ingest"

def run_frozen_via_cs_ingest(
    *,
    batch_id: int,
    cohort_file: Path,
    archive_dir: Path,
    start_interval: int,
    max_intervals: int | None,
    fresh: bool,
    resume: bool,
    rebuild_dedupe: bool,
) -> str:
    """Canonical frozen replay path — validated replay-step in Rust. Returns stdout string."""
    binary = cs_ingest_binary()
    if not binary.exists():
        print(f"❌ cs-ingest not built: {binary}", file=sys.stderr)
        print("   Run: cargo build -p cs-ingest --release", file=sys.stderr)
        sys.exit(1)

    cmd = [
        str(binary),
        "replay-step",
        "--batch-id",
        str(batch_id),
        "--cohort",
        str(cohort_file),
        "--archive",
        str(archive_dir),
        "--start-interval",
        str(start_interval),
    ]
    if max_intervals is not None:
        cmd.extend(["--max-intervals", str(max_intervals)])
    if fresh:
        cmd.append("--fresh")
    if resume:
        cmd.append("--resume")
    if rebuild_dedupe:
        cmd.append("--rebuild-dedupe")

    print("=" * 60)
    print("CHRONOSENTIMENT — FROZEN REPLAY (cs-ingest)")
    print("=" * 60)
    print(f"  Backend            : cs-ingest replay-step")
    print(f"  Archive            : {archive_dir}")
    if fresh:
        print("  Certification      : --fresh (isolated archive)")
    print("=" * 60)

    t0 = time.time()
    try:
        proc = subprocess.run(
            cmd,
            cwd=Path(__file__).resolve().parents[1],
            check=True,
            capture_output=True,
            text=True,
        )
        print(proc.stdout)
    except subprocess.CalledProcessError as e:
        print(e.stdout)
        print(f"❌ cs-ingest failed with exit code {e.returncode}", file=sys.stderr)
        sys.exit(e.returncode)

    duration = time.time() - t0
    print("\n" + "=" * 60)
    print("🏆 BATCH INGESTION COMPLETE")
    print("=" * 60)
    print(f"  Execution Time      : {duration:.2f} seconds")
    print(f"  Substrate Location  : {archive_dir}/raw/")
    print("=" * 60)
    
    return proc.stdout

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Ingest an alphabetical cohort batch of symbols via Rust cs-ingest.")
    parser.add_argument("--batch-id", type=int, default=1, help="Cohort batch ID to run (default: 1)")
    parser.add_argument(
        "--fresh",
        action="store_true",
        help="Wipe isolated batch archive before ingest (required for replay verification)",
    )
    parser.add_argument(
        "--shared-archive",
        action="store_true",
        help="Write to legacy state_archive/ root (not recommended during verification)",
    )
    parser.add_argument(
        "--run-label",
        default="",
        help="Label for isolated replay under batch_NNN/runs/LABEL (optional)",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Load dedupe index; skip duplicate (symbol, ts) writes (full replay still runs)",
    )
    parser.add_argument(
        "--rebuild-dedupe",
        action="store_true",
        help="Rebuild dedupe index from existing gzip streams before ingest",
    )
    parser.add_argument(
        "--from-frozen",
        action="store_true",
        help="(DEPRECATED) All ingests are now from frozen. This flag is a no-op.",
    )
    parser.add_argument(
        "--start-interval",
        type=int,
        default=0,
        help="First barrier index in aligned timeline (for bounded parity runs)",
    )
    parser.add_argument(
        "--max-intervals",
        type=int,
        default=None,
        help="Max barriers to process from start-interval",
    )
    args = parser.parse_args()

    if args.fresh and args.resume:
        print("❌ Cannot use --fresh and --resume together", file=sys.stderr)
        sys.exit(1)

    cohort_file = Path(f"cohorts/batch_{args.batch_id:03d}.txt")
    run_label = args.run_label
    archive_dir = resolve_archive_dir(args.batch_id, args.shared_archive, run_label)
    manifest_label = run_label or f"run_{int(time.time())}"

    run_frozen_via_cs_ingest(
        batch_id=args.batch_id,
        cohort_file=cohort_file,
        archive_dir=archive_dir,
        start_interval=args.start_interval,
        max_intervals=args.max_intervals,
        fresh=args.fresh,
        resume=args.resume,
        rebuild_dedupe=args.rebuild_dedupe,
    )
    sys.exit(0)
