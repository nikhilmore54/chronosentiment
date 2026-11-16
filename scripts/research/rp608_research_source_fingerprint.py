#!/usr/bin/env python3
"""Fingerprint a research source file (sha256, bytes, mtime, line count)."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path


def fingerprint(path: Path) -> dict:
    if not path.exists():
        return {"path": str(path), "exists": False}
    data = path.read_bytes()
    lines = data.count(b"\n") + (1 if data and not data.endswith(b"\n") else 0)
    nonempty = sum(1 for line in data.splitlines() if line.strip())
    return {
        "path": str(path.resolve()),
        "exists": True,
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "mtime_utc": datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc).isoformat(),
        "line_count": lines,
        "nonempty_line_count": nonempty,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+")
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    reps = [fingerprint(Path(p)) for p in args.paths]
    text = json.dumps(reps if len(reps) > 1 else reps[0], indent=2)
    if args.out:
        Path(args.out).write_text(text + "\n", encoding="utf-8")
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
