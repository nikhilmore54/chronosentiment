#!/usr/bin/env python3
"""
RP-608 Gate2E B (15m OHLCV trend-agree) scorecard — W/L-first per book.

Books never pooled. Compares B twin vs A (PRIMARY Gate2E) vs BASELINE.
Observational; does not change traj15 / OF ±0.30 / Gate 5 / coralys-*.

Usage:
  python3 scripts/research/rp608_gate2e_trend_agree_status.py
  python3 scripts/research/rp608_gate2e_trend_agree_status.py --json
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
PROTOCOL = "RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001"

# Local shadow roots (Pi pull / mirror). Missing paths → empty book, not error.
CRYPTO_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto"
ETH_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto_eth"
NSE_ROOT = WORKSPACE / "live_capture/shadow_validation/nse"


def _load_jsonl(path: Path, max_bytes: int = 16_000_000) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    with open(path, "rb") as f:
        f.seek(0, 2)
        f.seek(max(0, f.tell() - int(max_bytes)))
        for line in f.read().decode("utf-8", "ignore").splitlines():
            if not line.strip():
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def _pnl_key(row: Dict[str, Any]) -> Optional[float]:
    for k in ("net_pnl_usdt", "net_pnl_inr", "net_pnl", "realized_net"):
        if row.get(k) is not None:
            try:
                return float(row[k])
            except (TypeError, ValueError):
                continue
    return None


def _is_closed(row: Dict[str, Any]) -> bool:
    if row.get("exit_reason") or row.get("exit_timestamp_utc_ms") or row.get("closed"):
        return True
    et = str(row.get("event_type") or "")
    return et.startswith("CLOSED_") or et in {"TIME_EXIT", "ORDER_FLOW_EXIT", "TRADE_CLOSED"}


def _wl(closed: List[Dict[str, Any]]) -> Dict[str, Any]:
    scored: List[Tuple[Dict[str, Any], float]] = []
    for r in closed:
        pnl = _pnl_key(r)
        if pnl is None:
            continue
        scored.append((r, pnl))
    wins = [r for r, p in scored if p > 0]
    losses = [r for r, p in scored if p <= 0]
    w, n = len(wins), len(wins) + len(losses)
    l = n - w
    wa = (
        sum(abs(_pnl_key(r) or 0.0) for r in wins) / w if w else None
    )
    la = (
        sum(abs(_pnl_key(r) or 0.0) for r in losses) / l if l else None
    )
    pay = (wa / la) if wa is not None and la and la > 0 else None
    net = sum(p for _, p in scored)
    return {
        "n": n,
        "W": w,
        "L": l,
        "wr": round(w / n, 4) if n else None,
        "payoff": round(pay, 4) if pay is not None else None,
        "net": round(net, 4),
    }


def _rel(path: Path) -> str:
    try:
        return str(path.relative_to(WORKSPACE))
    except ValueError:
        return str(path)


def _first_existing(*candidates: Path) -> Path:
    for p in candidates:
        if p.exists():
            return p
    return candidates[0]


def _event_counts(path: Path) -> Dict[str, int]:
    rows = _load_jsonl(path, max_bytes=8_000_000)
    out: Dict[str, int] = {}
    for r in rows:
        et = str(r.get("event_type") or "")
        if not et:
            continue
        out[et] = int(out.get(et) or 0) + 1
    return out


def _book_from_journal(path: Path, *, events_path: Optional[Path] = None) -> Dict[str, Any]:
    rows = _load_jsonl(path)
    closed = [r for r in rows if _is_closed(r)]
    # Outcomes files are closed-only; events may carry PLAN_CREATED opens.
    open_hint = 0
    suppress_trend = 0
    plans_created = 0
    ev_path = events_path if events_path is not None else path
    if ev_path.exists():
        counts = _event_counts(ev_path)
        plans_created = int(counts.get("PLAN_CREATED") or 0)
        suppress_trend = int(counts.get("SUPPRESSED_GATE2E_TREND_AGREE") or 0)
        open_hint = max(0, plans_created - sum(
            int(counts.get(k) or 0)
            for k in counts
            if str(k).startswith("CLOSED_") or k in {"TIME_EXIT", "ORDER_FLOW_EXIT"}
        ))
    return {
        "path": _rel(path),
        "exists": path.exists() or (events_path.exists() if events_path else False),
        "rows": len(rows),
        "open_hint": open_hint,
        "plans_created": plans_created,
        "suppressed_trend_agree": suppress_trend,
        "wl": _wl(closed),
    }


def _crypto_pair(root: Path, label: str) -> Dict[str, Any]:
    # Pi layout: trade_plan_overlay_v1/{trade_plan_outcomes,trade_plan_events}.jsonl
    primary_out = _first_existing(
        root / "trade_plan_overlay_v1" / "trade_plan_outcomes.jsonl",
        root / "trade_plan_overlay" / "trade_plan_outcomes.jsonl",
        root / "trade_plan_overlay_v1" / "outcomes.jsonl",
    )
    primary_ev = _first_existing(
        root / "trade_plan_overlay_v1" / "trade_plan_events.jsonl",
        root / "trade_plan_overlay" / "trade_plan_events.jsonl",
    )
    baseline_out = _first_existing(
        root / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_outcomes.jsonl",
        root / "trade_plan_overlay_baseline_parallel_v1" / "outcomes.jsonl",
        root / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_events.jsonl",
    )
    baseline_ev = _first_existing(
        root / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_events.jsonl",
    )
    twin_out = _first_existing(
        root / "trade_plan_overlay_trend_agree_parallel_v1" / "trade_plan_outcomes.jsonl",
        root / "trade_plan_overlay_trend_agree_parallel_v1" / "outcomes.jsonl",
        root / "trade_plan_overlay_trend_agree_parallel_v1" / "trade_plan_events.jsonl",
    )
    twin_ev = _first_existing(
        root / "trade_plan_overlay_trend_agree_parallel_v1" / "trade_plan_events.jsonl",
    )
    twin_dir = root / "trade_plan_overlay_trend_agree_parallel_v1"
    return {
        "venue": label,
        "twin_dir_exists": twin_dir.is_dir(),
        "A_primary": _book_from_journal(primary_out, events_path=primary_ev),
        "BASELINE": _book_from_journal(baseline_out, events_path=baseline_ev),
        "B_trend_agree": _book_from_journal(twin_out, events_path=twin_ev),
    }


def _nse_books() -> Dict[str, Any]:
    primary = _first_existing(
        NSE_ROOT / "rp608_nse_trade_journal.jsonl",
        NSE_ROOT / "estimand_a" / "rp608_nse_trade_journal.jsonl",
    )
    baseline = NSE_ROOT / "estimand_a_baseline_parallel_v1" / "rp608_nse_trade_journal.jsonl"
    twin = NSE_ROOT / "estimand_a_trend_agree_parallel_v1" / "rp608_nse_trade_journal.jsonl"
    twin_dir = NSE_ROOT / "estimand_a_trend_agree_parallel_v1"
    return {
        "venue": "NSE_ESTIMAND_A",
        "twin_dir_exists": twin_dir.is_dir(),
        "A_primary": _book_from_journal(primary),
        "BASELINE": _book_from_journal(baseline),
        "B_trend_agree": _book_from_journal(twin),
    }


def _verdict_line(block: Dict[str, Any]) -> str:
    a = (block.get("A_primary") or {}).get("wl") or {}
    b_book = block.get("B_trend_agree") or {}
    b = b_book.get("wl") or {}
    bl = (block.get("BASELINE") or {}).get("wl") or {}
    if not block.get("twin_dir_exists") and not b_book.get("exists"):
        return "TWIN_NOT_ARMED_OR_NO_JOURNAL"
    if int(b.get("n") or 0) == 0:
        return (
            f"INSUFFICIENT_CLOSED_B "
            f"plans={b_book.get('plans_created')} "
            f"suppress_trend={b_book.get('suppressed_trend_agree')} "
            f"(soak; DO_NOT_ADVANCE)"
        )
    # Predeclared: need soak; this status never auto-ADVANCE.
    return (
        f"OBSERVE n_B={b.get('n')} WR_B={b.get('wr')} | "
        f"n_A={a.get('n')} WR_A={a.get('wr')} | "
        f"n_BL={bl.get('n')} WR_BL={bl.get('wr')} "
        f"(ADVANCE only after holdout floor — see scoped memo)"
    )


def build_status() -> Dict[str, Any]:
    btc = _crypto_pair(CRYPTO_ROOT, "BTCUSDT")
    eth = _crypto_pair(ETH_ROOT, "ETHUSDT")
    nse = _nse_books()
    return {
        "protocol_id": PROTOCOL,
        "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "metric_order": ["W/L", "payoff", "net"],
        "books_never_pooled": True,
        "authority_pct": 0.0,
        "btc": {**btc, "status": _verdict_line(btc)},
        "eth": {**eth, "status": _verdict_line(eth)},
        "nse": {**nse, "status": _verdict_line(nse)},
        "gate5": "NO_CHANGE",
        "coralys": "NO_TOUCH",
        "logic_authority": "rust_ohlcv_trend_agreement",
    }


def _fmt_wl(wl: Dict[str, Any]) -> str:
    if not wl or not wl.get("n"):
        return "n=0"
    return (
        f"n={wl['n']} W={wl['W']} L={wl['L']} "
        f"WR={wl.get('wr')} payoff={wl.get('payoff')} net={wl.get('net')}"
    )


def _fmt_book(book: Dict[str, Any]) -> str:
    return (
        f"{_fmt_wl(book.get('wl') or {})} "
        f"plans={book.get('plans_created')} "
        f"suppress_trend={book.get('suppressed_trend_agree')} "
        f"exists={book.get('exists')}"
    )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    st = build_status()
    if args.json:
        print(json.dumps(st, indent=2, sort_keys=True))
        return 0
    print(f"protocol={st['protocol_id']} generated_utc={st['generated_utc']}")
    print(f"logic={st['logic_authority']} gate5={st['gate5']} coralys={st['coralys']}")
    for key in ("btc", "eth", "nse"):
        block = st[key]
        print(f"\n=== {block.get('venue')} twin_dir={block.get('twin_dir_exists')} ===")
        for role in ("A_primary", "BASELINE", "B_trend_agree"):
            book = block.get(role) or {}
            print(f"  {role}: {_fmt_book(book)}")
        print(f"  status: {block.get('status')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
