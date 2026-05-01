#!/usr/bin/env python3
"""
Blue/Green log writer for streaming pipelines.

Reads stdin and writes to alternating files:
  <base>_A.log and <base>_B.log

Also maintains <base>.log as a symlink to the active file so existing
consumers can continue reading a stable path.

Example:
  ... | python3 scripts/blue_green_log_writer.py analysis/awr_grid/live_run.log
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("base_log_path", help="Base log path, e.g. analysis/awr_grid/live_run.log")
    ap.add_argument(
        "--max-bytes",
        type=int,
        default=5 * 1024 * 1024,
        help="Rotate when active file exceeds this size (default: 5MB)",
    )
    return ap.parse_args()


def make_variant_path(base: Path, suffix: str) -> Path:
    stem = base.stem
    ext = base.suffix or ".log"
    return base.with_name(f"{stem}_{suffix}{ext}")


def set_active_symlink(link_path: Path, target: Path) -> None:
    try:
        if link_path.exists() or link_path.is_symlink():
            link_path.unlink()
        # Use relative target so moving parent directory preserves link.
        rel_target = os.path.relpath(target, start=link_path.parent)
        link_path.symlink_to(rel_target)
    except OSError:
        # Fallback for environments where symlink creation may fail.
        # Keep behavior deterministic by writing directly to link path later.
        pass


def main() -> int:
    args = parse_args()
    # Avoid resolve(): it follows existing symlinks and can mutate base naming.
    base = Path(args.base_log_path).expanduser()
    if not base.is_absolute():
        base = (Path.cwd() / base).absolute()
    base.parent.mkdir(parents=True, exist_ok=True)

    a_path = make_variant_path(base, "A")
    b_path = make_variant_path(base, "B")
    active_path = a_path
    standby_path = b_path

    # Start fresh on first active file.
    active_path.write_text("", encoding="utf-8")
    set_active_symlink(base, active_path)

    out = active_path.open("a", encoding="utf-8")
    try:
        for line in sys.stdin:
            out.write(line)
            out.flush()

            if active_path.stat().st_size >= args.max_bytes:
                out.close()
                # Flip active/standby and truncate new active file.
                active_path, standby_path = standby_path, active_path
                active_path.write_text("", encoding="utf-8")
                set_active_symlink(base, active_path)
                out = active_path.open("a", encoding="utf-8")
    finally:
        out.close()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
