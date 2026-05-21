#!/usr/bin/env python3
"""
Incremental synchronized observatory step — one global timestamp only.

Requires frozen candle substrate. Advances manifold by a single chronosynchrony
barrier (default: latest timestamp in frozen manifest).

Usage:
  python3 scripts/freeze_cohort_candles.py --batch-id 3
  python3 scripts/run_incremental_cohort.py --batch-id 3
  python3 scripts/run_incremental_cohort.py --batch-id 3 --bootstrap-archive  # first warm pass
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import pandas as pd

sys.path.insert(0, str(Path(__file__).resolve().parent))
from candle_substrate import build_timeline_fingerprint, load_frozen_cohort
from run_nse_cohort import NSEIngestionEngine, fresh_wipe_archive, resolve_archive_dir


def latest_global_timestamp(data: dict[str, pd.DataFrame]) -> int | None:
    latest = None
    for df in data.values():
        if df.empty:
            continue
        ts = int(df.index[-1].timestamp())
        if latest is None or ts > latest:
            latest = ts
    return latest


def build_interval_batch(data: dict[str, pd.DataFrame], ts: int) -> list[dict]:
    batch = []
    dt = pd.to_datetime(ts, unit="s", utc=True)
    for sym, df in data.items():
        if dt not in df.index:
            continue
        row = df.loc[dt]
        if isinstance(row, pd.DataFrame):
            row = row.iloc[0]

        def get_val(k: str) -> float:
            raw = row.get(k, row.get(k.lower(), 0.0))
            if hasattr(raw, "iloc"):
                raw = raw.iloc[0]
            return float(raw)

        batch.append(
            {
                "symbol": sym,
                "timestamp": ts,
                "open": get_val("Open"),
                "high": get_val("High"),
                "low": get_val("Low"),
                "close": get_val("Close"),
                "volume": get_val("Volume"),
            }
        )
    return batch


def main():
    parser = argparse.ArgumentParser(description="Single-interval incremental cohort observatory step")
    parser.add_argument("--batch-id", type=int, required=True)
    parser.add_argument("--at-ts", type=int, default=0, help="Unix ts (default: latest in frozen substrate)")
    parser.add_argument("--run-label", default="incremental")
    parser.add_argument("--bootstrap-archive", action="store_true", help="Wipe archive metadata before step")
    parser.add_argument("--shared-archive", action="store_true")
    args = parser.parse_args()

    cohort_file = Path(f"cohorts/batch_{args.batch_id:03d}.txt")
    archive_dir = resolve_archive_dir(args.batch_id, args.shared_archive, args.run_label)

    if args.bootstrap_archive and archive_dir.exists():
        print(f"🧹 bootstrap: wiping {archive_dir}")
        fresh_wipe_archive(archive_dir)
    archive_dir.mkdir(parents=True, exist_ok=True)

    symbols = [line.strip() for line in cohort_file.read_text().splitlines() if line.strip()]
    data, frozen_manifest = load_frozen_cohort(args.batch_id, symbols)

    ts = args.at_ts or latest_global_timestamp(data)
    if ts is None:
        print("❌ No timestamps in frozen substrate", file=sys.stderr)
        sys.exit(1)

    batch = build_interval_batch(data, ts)
    if not batch:
        print(f"❌ No symbols have a bar at ts={ts}", file=sys.stderr)
        sys.exit(1)

    print(f"📋 Incremental step batch={args.batch_id:03d} ts={ts} symbols={len(batch)}")
    print(f"   Frozen fingerprint: {frozen_manifest.get('timeline_fingerprint')}")
    print(f"   Substrate hash: {frozen_manifest.get('substrate_hash')}")

    engine = NSEIngestionEngine(
        cohort_file=cohort_file,
        archive_dir=archive_dir,
        batch_id=args.batch_id,
        run_label=args.run_label,
    )
    engine.init_dedupe_index()

    import subprocess

    proc = subprocess.Popen(
        ["./target/release/examples/live_observatory"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    t0 = time.time()
    proc.stdin.write(json.dumps(batch) + "\n")
    proc.stdin.flush()

    processed = 0
    corridors = 0
    for _ in range(len(batch)):
        while True:
            line = proc.stdout.readline()
            if not line:
                break
            if line.startswith("[TELEMETRY]"):
                rec = engine.process_telemetry_line(line)
                if rec:
                    processed += 1
                    if rec.get("corridor"):
                        corridors += 1
                break

    proc.stdin.close()
    proc.terminate()
    engine.dedupe.save()
    engine._gzip_pool.flush_all()
    engine._gzip_pool.close_all()

    duration = time.time() - t0
    engine._sorted_timestamps = [ts]
    manifest_path = engine.write_ingestion_manifest(
        symbols_downloaded=len(data),
        processed_ticks=processed,
        corridors_detected=corridors,
        duration_sec=duration,
    )

    inc_manifest = archive_dir / "metadata" / "incremental_steps.jsonl"
    inc_manifest.parent.mkdir(parents=True, exist_ok=True)
    step = {
        "ts": ts,
        "processed_ticks": processed,
        "corridors": corridors,
        "duration_sec": round(duration, 3),
        "frozen_substrate_hash": frozen_manifest.get("substrate_hash"),
        "completed_at_utc": datetime.now(timezone.utc).isoformat(),
    }
    with open(inc_manifest, "a") as f:
        f.write(json.dumps(step) + "\n")

    print("\n" + "=" * 60)
    print("⚡ INCREMENTAL STEP COMPLETE")
    print("=" * 60)
    print(f"  Timestamp           : {ts}")
    print(f"  Symbols in batch    : {len(batch)}")
    print(f"  State ticks         : {processed:,}")
    print(f"  Corridors           : {corridors:,}")
    print(f"  Wall time           : {duration:.2f}s ({processed / max(duration, 1e-6):.0f} states/sec)")
    print(f"  Archive             : {archive_dir}")
    print(f"  Manifest            : {manifest_path}")
    print("=" * 60)


if __name__ == "__main__":
    main()
