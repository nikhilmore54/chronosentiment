#!/usr/bin/env python3
"""
ChronoSentiment — Safe state_archive cleanup (Phase A verification prep)

Removes:
  - legacy uncompressed telemetry_stream.jsonl when .jsonl.gz exists
  - stale uncompressed corridor/collapse logs when .jsonl.gz counterpart exists
  - optional partial batch dirs (--remove-partial-batch N)

Verifies gzip integrity before deleting anything.
"""

from __future__ import annotations

import argparse
import gzip
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / "state_archive"


def verify_gzip(path: Path) -> bool:
    try:
        with gzip.open(path, "rt", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                json.loads(line)
        return True
    except Exception as e:
        print(f"  ❌ gzip corrupt: {path} ({e})")
        return False


def remove_legacy_telemetry(archive: Path, dry_run: bool) -> tuple[int, int]:
    removed, bytes_freed = 0, 0
    for legacy in archive.rglob("telemetry_stream.jsonl"):
        sym_dir = legacy.parent
        gz_files = list(sym_dir.glob("telemetry_stream_*.jsonl.gz"))
        if not gz_files:
            print(f"  ⚠️  skip (no gzip yet): {legacy}")
            continue
        if not all(verify_gzip(g) for g in gz_files):
            print(f"  ⚠️  skip (gzip verify failed): {sym_dir}")
            continue
        size = legacy.stat().st_size
        if dry_run:
            print(f"  [dry-run] would remove {legacy} ({size/1e6:.2f} MB)")
        else:
            legacy.unlink()
            print(f"  removed {legacy.name} ({size/1e6:.2f} MB)")
        removed += 1
        bytes_freed += size
    return removed, bytes_freed


def remove_stale_transition_dupes(archive: Path, dry_run: bool) -> tuple[int, int]:
    """Remove uncompressed corridor/collapse logs if gz mirror exists."""
    removed, bytes_freed = 0, 0
    for sub in ("corridor_events", "collapse_events"):
        base = archive / "transitions" / sub
        if not base.exists():
            continue
        for plain in base.glob("*.jsonl"):
            gz = plain.with_suffix(plain.suffix + ".gz")
            if not gz.exists():
                continue
            if not verify_gzip(gz):
                print(f"  ⚠️  skip stale plain (gz bad): {plain}")
                continue
            size = plain.stat().st_size
            if dry_run:
                print(f"  [dry-run] would remove {plain}")
            else:
                plain.unlink()
            removed += 1
            bytes_freed += size
    return removed, bytes_freed


def compress_transition_logs(archive: Path, dry_run: bool) -> tuple[int, int]:
    """Gzip-compress legacy plain corridor/collapse logs (no .gz yet)."""
    compressed, bytes_before = 0, 0
    for sub in ("corridor_events", "collapse_events"):
        base = archive / "transitions" / sub
        if not base.exists():
            continue
        for plain in base.glob("*.jsonl"):
            gz = Path(str(plain) + ".gz")
            if gz.exists():
                continue
            raw = plain.read_bytes()
            bytes_before += len(raw)
            if dry_run:
                print(f"  [dry-run] would gzip {plain.name} ({len(raw)/1e3:.0f} KB)")
            else:
                with gzip.open(gz, "wb") as out:
                    out.write(raw)
                if verify_gzip(gz):
                    plain.unlink()
                    print(f"  gzipped {plain.name} -> {gz.name}")
                else:
                    gz.unlink(missing_ok=True)
                    print(f"  ❌ failed verify, kept {plain.name}")
                    continue
            compressed += 1
    return compressed, bytes_before


def remove_partial_batch(batch_id: int, dry_run: bool) -> tuple[int, int]:
    import shutil

    path = ARCHIVE / "batches" / f"batch_{batch_id:03d}"
    if not path.exists():
        return 0, 0
    total = sum(f.stat().st_size for f in path.rglob("*") if f.is_file())
    if dry_run:
        print(f"  [dry-run] would remove partial batch dir {path} ({total/1e6:.1f} MB)")
    else:
        shutil.rmtree(path)
        print(f"  removed partial batch {path} ({total/1e6:.1f} MB)")
    return 1, total


def audit_duplication(archive: Path) -> None:
    print("\n📊 Archive audit")
    raw = archive / "raw"
    if not raw.exists():
        print("  no raw/ layer")
        return
    legacy = list(raw.rglob("telemetry_stream.jsonl"))
    gz = list(raw.rglob("telemetry_stream_*.jsonl.gz"))
    print(f"  legacy .jsonl files     : {len(legacy)}")
    print(f"  gzip stream files       : {len(gz)}")
    corr = archive / "transitions" / "corridor_events"
    if corr.exists():
        plain = list(corr.glob("*.jsonl"))
        gz_c = list(corr.glob("*.jsonl.gz"))
        print(f"  corridor plain          : {len(plain)}")
        print(f"  corridor gzip           : {len(gz_c)}")


def main():
    parser = argparse.ArgumentParser(description="Safe state_archive cleanup before verification")
    parser.add_argument("--dry-run", action="store_true", help="Report only, do not delete")
    parser.add_argument(
        "--archive-root",
        default=str(ARCHIVE),
        help="Archive root (default: state_archive/)",
    )
    parser.add_argument(
        "--remove-partial-batch",
        type=int,
        default=0,
        help="Remove interrupted batch_NNN dir under batches/",
    )
    parser.add_argument(
        "--compress-corridors",
        action="store_true",
        help="Gzip legacy uncompressed corridor/collapse event logs",
    )
    args = parser.parse_args()

    archive = Path(args.archive_root)
    if not archive.exists():
        print(f"❌ Archive not found: {archive}", file=sys.stderr)
        sys.exit(1)

    print("=" * 60)
    print("CHRONOSENTIMENT — STATE ARCHIVE CLEANUP (Phase A)")
    print("=" * 60)
    print(f"Archive: {archive}")
    print(f"Mode:    {'DRY RUN' if args.dry_run else 'LIVE DELETE'}")
    print()

    audit_duplication(archive)
    print()

    total_removed = 0
    total_bytes = 0

    print("🧹 Legacy telemetry_stream.jsonl (gzip must exist + verify)")
    n, b = remove_legacy_telemetry(archive, args.dry_run)
    total_removed += n
    total_bytes += b

    print("\n🧹 Stale uncompressed transition logs (gzip mirror exists)")
    n, b = remove_stale_transition_dupes(archive, args.dry_run)
    total_removed += n
    total_bytes += b

    if args.remove_partial_batch:
        print(f"\n🧹 Partial batch {args.remove_partial_batch:03d}")
        n, b = remove_partial_batch(args.remove_partial_batch, args.dry_run)
        total_removed += n
        total_bytes += b

    if args.compress_corridors:
        print("\n🗜️  Compressing legacy corridor/collapse logs")
        n, b = compress_transition_logs(archive, args.dry_run)
        total_removed += n
        total_bytes += b

    print("\n" + "=" * 60)
    print(f"Files removed : {total_removed}")
    print(f"Space reclaimed: {total_bytes/1e6:.2f} MB")
    print("=" * 60)

    if not args.dry_run:
        audit_duplication(archive)


if __name__ == "__main__":
    main()
