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
    release_bin = ROOT / "target" / "release" / "cs-ingest"
    packaged_bin = ROOT / "bin" / "cs-ingest"

    if release_bin.exists():
        cmd = [str(release_bin), "verify-chronology-fixtures", "--repo-root", str(ROOT)]
    elif packaged_bin.exists():
        cmd = [str(packaged_bin), "verify-chronology-fixtures", "--repo-root", str(ROOT)]
    else:
        # CI often invokes this before an explicit release build of cs-ingest.
        cmd = [
            "cargo",
            "run",
            "--quiet",
            "--release",
            "-p",
            "cs-ingest",
            "--",
            "verify-chronology-fixtures",
            "--repo-root",
            str(ROOT),
        ]

    try:
        return subprocess.call(cmd, cwd=ROOT)
    except FileNotFoundError:
        print(
            "[FAIL] cs-ingest binary not found — build with: "
            "cargo build -p cs-ingest --release"
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
