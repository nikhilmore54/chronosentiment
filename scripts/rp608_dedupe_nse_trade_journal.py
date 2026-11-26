#!/usr/bin/env python3
"""
Deduplicate RP-608 NSE Estimand A trade journal by trade_id.

Keeps the first exit chronologically per trade_id (preserves original PnL).
Writes atomic replace; leaves .bak alongside.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import time
from pathlib import Path


def dedupe_journal(path: Path, *, dry_run: bool = False) -> dict:
    path = Path(path)
    rows = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue

    best: dict[str, dict] = {}
    order: list[str] = []
    no_id = []
    for r in rows:
        tid = str(r.get("trade_id") or "")
        if not tid:
            no_id.append(r)
            continue
        if tid not in best:
            best[tid] = r
            order.append(tid)
            continue
        prev = best[tid]
        prev_x = int(prev.get("exit_timestamp_utc_ms") or 0)
        cur_x = int(r.get("exit_timestamp_utc_ms") or 0)
        if cur_x and (not prev_x or cur_x < prev_x):
            best[tid] = r

    kept = [best[t] for t in order] + no_id
    meta = {
        "input_rows": len(rows),
        "output_rows": len(kept),
        "removed": len(rows) - len(kept),
        "unique_trade_ids": len(best),
        "no_trade_id": len(no_id),
    }
    if dry_run:
        return meta

    bak = path.with_suffix(path.suffix + f".bak_dedupe_{int(time.time())}")
    shutil.copy2(path, bak)
    tmp = path.with_suffix(path.suffix + ".tmp")
    with tmp.open("w", encoding="utf-8") as f:
        for r in kept:
            f.write(json.dumps(r, separators=(",", ":")) + "\n")
        f.flush()
        os.fsync(f.fileno())
    os.replace(tmp, path)
    meta["backup"] = str(bak)
    return meta


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--path",
        default="/home/ultraspeed/ChronoSentiment_MEGA_FINAL/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl",
    )
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    meta = dedupe_journal(Path(args.path), dry_run=args.dry_run)
    print(json.dumps(meta, indent=2))


if __name__ == "__main__":
    main()
