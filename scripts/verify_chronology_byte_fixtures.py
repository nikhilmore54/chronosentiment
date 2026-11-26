#!/usr/bin/env python3
"""Compatibility shim — V-006 chronology fixtures are verified by cs-ingest.

Prefer: `cs-ingest verify-chronology-fixtures`
This wrapper keeps release/bootstrap shells working until they call Rust directly.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    candidates = [
        ROOT / "target" / "release" / "cs-ingest",
        ROOT / "bin" / "cs-ingest",
    ]
    cmd_bin = str(next((p for p in candidates if p.exists()), Path("cs-ingest")))
    cmd = [cmd_bin, "verify-chronology-fixtures", "--repo-root", str(ROOT)]
    try:
        return subprocess.call(cmd, cwd=ROOT)
    except FileNotFoundError:
        print("[FAIL] cs-ingest binary not found — build with: cargo build -p cs-ingest --release")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
