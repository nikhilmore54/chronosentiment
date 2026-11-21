#!/usr/bin/env python3
"""Compare CRYPTO_PLAN vs CRYPTO_PLAN_BASELINE_PARALLEL since a restart cutoff.

Books never pooled. W/L-first per book.
"""
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

IST = timezone(timedelta(hours=5, minutes=30))


def ms(v: Any) -> Optional[datetime]:
    try:
        return datetime.fromtimestamp(int(v) / 1000, tz=timezone.utc)
    except Exception:
        return None


def load_unique_plans(path: Path, rst: datetime) -> List[Dict[str, Any]]:
    seen = set()
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except json.JSONDecodeError:
            continue
        et = ms(r.get("entry_time_ms"))
        if not et or et < rst:
            continue
        pid = r.get("plan_id")
        if pid in seen:
            continue
        seen.add(pid)
        rows.append(r)
    # open plan
    st = path.parent / "overlay_state.json"
    if st.exists():
        try:
            ap = json.loads(st.read_text()).get("active_plan")
        except Exception:
            ap = None
        if ap and ap.get("state") == "OPEN":
            et = ms(ap.get("entry_time_ms"))
            pid = ap.get("plan_id")
            if et and et >= rst and pid not in seen:
                rows.append(ap)
                seen.add(pid)
    return sorted(rows, key=lambda r: int(r.get("entry_time_ms") or 0))


def summarize(book: str, rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    closed = [r for r in rows if r.get("exit_reason") or str(r.get("state") or "").startswith("CLOSED")]
    open_n = sum(1 for r in rows if r.get("state") == "OPEN")
    wins = [r for r in closed if float(r.get("net_pnl_usdt") or 0) > 0]
    losses = [r for r in closed if float(r.get("net_pnl_usdt") or 0) <= 0]
    net = sum(float(r.get("net_pnl_usdt") or 0) for r in closed)
    return {
        "book": book,
        "opened": len(rows),
        "open": open_n,
        "closed": len(closed),
        "wins": len(wins),
        "losses": len(losses),
        "wl": f"{len(wins)}/{len(losses)}" if closed else "n/a",
        "net_usdt": round(net, 2),
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--crypto-root",
        default="live_capture/shadow_validation/crypto",
    )
    ap.add_argument(
        "--restart-utc",
        default="",
        help="ISO UTC restart cutoff (default: require explicit)",
    )
    args = ap.parse_args()
    if not args.restart_utc:
        raise SystemExit("--restart-utc required (e.g. 2026-11-20T13:00:00Z)")
    rst = datetime.fromisoformat(args.restart_utc.replace("Z", "+00:00"))
    if rst.tzinfo is None:
        rst = rst.replace(tzinfo=timezone.utc)
    root = Path(args.crypto_root)
    primary = load_unique_plans(
        root / "trade_plan_overlay_v1" / "trade_plan_outcomes.jsonl", rst
    )
    baseline = load_unique_plans(
        root / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_outcomes.jsonl",
        rst,
    )
    s1 = summarize("CRYPTO_PLAN", primary)
    s0 = summarize("CRYPTO_PLAN_BASELINE_PARALLEL", baseline)
    print(json.dumps({"restart_utc": rst.isoformat(), "primary": s1, "baseline": s0}, indent=2))
    print("\n# W/L-first (never pooled)")
    print(f"PRIMARY  opened={s1['opened']} closed={s1['closed']} W/L={s1['wl']} net={s1['net_usdt']}")
    print(f"BASELINE opened={s0['opened']} closed={s0['closed']} W/L={s0['wl']} net={s0['net_usdt']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
