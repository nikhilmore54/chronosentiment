#!/usr/bin/env python3
"""
RP-608 Track A — plan_lifecycle_authority soak audit (books never pooled).

Counts closed trade-plan outcomes by plan_lifecycle_authority per book.
Default roots: crypto PRIMARY + BASELINE under shadow_validation/crypto.

Usage:
  python3 scripts/research/rp608_plan_lifecycle_authority_audit.py
  python3 scripts/research/rp608_plan_lifecycle_authority_audit.py --since-hours 24
  python3 scripts/research/rp608_plan_lifecycle_authority_audit.py --today-ist
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
CRYPTO = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"
IST = timezone(timedelta(hours=5, minutes=30))

BOOKS = (
    ("CRYPTO_PRIMARY", CRYPTO / "trade_plan_overlay_v1" / "trade_plan_outcomes.jsonl"),
    (
        "CRYPTO_BASELINE",
        CRYPTO / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_outcomes.jsonl",
    ),
    (
        "CRYPTO_ETH_PRIMARY",
        WORKSPACE
        / "live_capture"
        / "shadow_validation"
        / "crypto_eth"
        / "trade_plan_overlay_v1"
        / "trade_plan_outcomes.jsonl",
    ),
)


def _exit_ms(row: Dict[str, Any]) -> Optional[int]:
    for k in ("exit_time_ms", "closed_ms", "close_time_ms"):
        v = row.get(k)
        if v is None:
            continue
        try:
            return int(v)
        except (TypeError, ValueError):
            continue
    return None


def _authority(row: Dict[str, Any]) -> str:
    a = row.get("plan_lifecycle_authority")
    if a:
        return str(a)
    return "UNSET"


def _iter_rows(path: Path) -> Iterable[Dict[str, Any]]:
    if not path.exists():
        return
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(row, dict):
                yield row


def audit_book(
    path: Path, *, since_ms: Optional[int]
) -> Tuple[Counter, Counter, List[Dict[str, Any]]]:
    auth: Counter = Counter()
    exits: Counter = Counter()
    kept: List[Dict[str, Any]] = []
    for row in _iter_rows(path) or []:
        xt = _exit_ms(row)
        if since_ms is not None:
            if xt is None or xt < since_ms:
                continue
        a = _authority(row)
        auth[a] += 1
        exits[str(row.get("exit_reason") or row.get("state") or "?")] += 1
        kept.append(row)
    return auth, exits, kept


def _since_ms(args: argparse.Namespace) -> Optional[int]:
    if args.today_ist:
        start = datetime.now(IST).replace(hour=0, minute=0, second=0, microsecond=0)
        return int(start.timestamp() * 1000)
    if args.since_hours is not None:
        return int((datetime.now(timezone.utc).timestamp() - float(args.since_hours) * 3600) * 1000)
    return None


def main() -> int:
    ap = argparse.ArgumentParser(description="Plan-lifecycle authority soak audit (never pooled)")
    ap.add_argument("--since-hours", type=float, default=None)
    ap.add_argument("--today-ist", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    since = _since_ms(args)

    report: Dict[str, Any] = {
        "protocol": "RP-608-PLAN-LIFECYCLE-AUTHORITY-AUDIT",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "since_ms": since,
        "books": {},
        "note": "Books never pooled; rust_rp608_plan_lifecycle = Track A soak stamp",
    }

    print("RP-608 plan_lifecycle_authority audit (books never pooled)")
    if since is not None:
        print(f"  since_ms={since} ({datetime.fromtimestamp(since/1000, tz=IST).isoformat()})")
    print()

    all_rust = True
    any_closed = False
    for book_id, path in BOOKS:
        auth, exits, rows = audit_book(path, since_ms=since)
        n = sum(auth.values())
        rust_n = int(auth.get("rust_rp608_plan_lifecycle", 0))
        unset_n = int(auth.get("UNSET", 0))
        other = {k: v for k, v in auth.items() if k not in ("rust_rp608_plan_lifecycle", "UNSET")}
        rust_pct = (100.0 * rust_n / n) if n else None
        if n:
            any_closed = True
            if rust_n != n:
                all_rust = False
        else:
            # empty window is not a fail for that book
            pass
        block = {
            "path": str(path),
            "exists": path.exists(),
            "n_closed": n,
            "authority": dict(auth),
            "rust_n": rust_n,
            "unset_n": unset_n,
            "other_authority": other,
            "rust_pct": rust_pct,
            "exits": dict(exits),
            "soak_ok": (n == 0) or (rust_n == n and unset_n == 0 and not other),
        }
        report["books"][book_id] = block
        print(f"=== {book_id} ===")
        print(f"  path: {path}")
        if not path.exists():
            print("  MISSING ledger")
            print()
            continue
        print(f"  n_closed={n}  rust={rust_n}  UNSET={unset_n}  other={other or '{}'}")
        if rust_pct is not None:
            print(f"  rust_pct={rust_pct:.1f}%  soak_ok={block['soak_ok']}")
        if exits:
            print(f"  exits={dict(exits)}")
        if rows:
            last = rows[-1]
            print(
                f"  last: {last.get('plan_id')} {last.get('exit_reason')} "
                f"auth={_authority(last)} net={last.get('net_pnl_usdt')}"
            )
        print()

    report["all_nonempty_books_rust_only"] = bool(any_closed and all_rust)
    report["track_a_soak_hint"] = (
        "PASS_PARTIAL" if report["all_nonempty_books_rust_only"] else "NEED_MORE_RUST_OR_UNSET"
    )
    if args.json:
        json.dump(report, sys.stdout, indent=2)
        print()
    else:
        print(
            f"summary: nonempty_books_all_rust={report['all_nonempty_books_rust_only']} "
            f"hint={report['track_a_soak_hint']}"
        )
        print(
            "next: confirm systemd REQUIRE_RUST=1 + multi-day rust authority; "
            "then retire Python Adaptive/Gate2X manage bodies "
            "(see docs/research/rp608_rust_migration_abc_plan_2026_11_22.md)"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
