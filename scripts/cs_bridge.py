#!/usr/bin/env python3
"""Thin Python helpers for remaining yfinance/live orchestration.

Causal ingest/verify/governor logic lives in `cs-ingest` (Rust).
This module only resolves archive paths and shells to the Rust binary.
"""

from __future__ import annotations

import subprocess
import sys
import time
from pathlib import Path

DEFAULT_ARCHIVE_ROOT = Path("state_archive")
ROOT = Path(__file__).resolve().parents[1]


def resolve_archive_dir(
    batch_id: int,
    shared_archive: bool = False,
    run_label: str = "",
    archive_root: Path | None = None,
) -> Path:
    root = archive_root or DEFAULT_ARCHIVE_ROOT
    if shared_archive:
        return root
    base = root / "batches" / f"batch_{batch_id:03d}"
    if run_label:
        return base / "runs" / run_label
    return base


def cs_ingest_binary() -> Path:
    return ROOT / "target" / "release" / "cs-ingest"


def run_frozen_via_cs_ingest(
    *,
    batch_id: int,
    cohort_file: Path | None = None,
    archive_dir: Path | None = None,
    start_interval: int = 0,
    max_intervals: int | None = None,
    fresh: bool = False,
    resume: bool = False,
    rebuild_dedupe: bool = False,
    run_label: str = "",
    shared_archive: bool = False,
) -> str:
    """Canonical frozen replay — delegates to `cs-ingest cohort`."""
    binary = cs_ingest_binary()
    if not binary.exists():
        print(f"❌ cs-ingest not built: {binary}", file=sys.stderr)
        print("   Run: cargo build -p cs-ingest --release", file=sys.stderr)
        sys.exit(1)

    if archive_dir is None:
        archive_dir = resolve_archive_dir(batch_id, shared_archive, run_label)

    # Prefer the convenience command when path matches standard layout;
    # otherwise call replay-step with an explicit --archive.
    cmd = [
        str(binary),
        "cohort",
        "--batch-id",
        str(batch_id),
        "--start-interval",
        str(start_interval),
    ]
    if run_label:
        cmd.extend(["--run-label", run_label])
    if shared_archive:
        cmd.append("--shared-archive")
    if max_intervals is not None:
        cmd.extend(["--max-intervals", str(max_intervals)])
    if fresh:
        cmd.append("--fresh")
    if resume:
        cmd.append("--resume")
    if rebuild_dedupe:
        cmd.append("--rebuild-dedupe")

    # If caller passed a non-standard archive_dir, fall back to replay-step.
    expected = resolve_archive_dir(batch_id, shared_archive, run_label)
    if archive_dir.resolve() != expected.resolve() or cohort_file is not None:
        cohort = cohort_file or Path(f"cohorts/batch_{batch_id:03d}.txt")
        cmd = [
            str(binary),
            "replay-step",
            "--batch-id",
            str(batch_id),
            "--cohort",
            str(cohort),
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
    print("  Backend            : cs-ingest")
    print(f"  Archive            : {archive_dir}")
    if fresh:
        print("  Certification      : --fresh (isolated archive)")
    print("=" * 60)

    t0 = time.time()
    try:
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        print(proc.stdout)
    except subprocess.CalledProcessError as e:
        print(e.stdout or "")
        print(e.stderr or "", file=sys.stderr)
        print(f"❌ cs-ingest failed with exit code {e.returncode}", file=sys.stderr)
        sys.exit(e.returncode)

    duration = time.time() - t0
    print("\n" + "=" * 60)
    print("BATCH INGESTION COMPLETE")
    print("=" * 60)
    print(f"  Execution Time      : {duration:.2f} seconds")
    print(f"  Substrate Location  : {archive_dir}/raw/")
    print("=" * 60)
    return proc.stdout
