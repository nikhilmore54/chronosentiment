#!/usr/bin/env python3
"""
RP-608-ADAPTER-SELECT-15M-V1 scorecard — W/L-first per book (BTC / ETH).

Books never pooled. Compares twin vs PRIMARY (A) vs BASELINE.
Observational; does not change traj15 / OF ±0.30 / Gate 5 / coralys-*.
Gate 2E B trend-agree is excluded from this promote path (D4).

Usage:
  python3 scripts/research/rp608_adapter_select_15m_status.py
  python3 scripts/research/rp608_adapter_select_15m_status.py --json
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
PROTOCOL = "RP-608-ADAPTER-SELECT-15M-V1"

CRYPTO_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto"
ETH_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto_eth"


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
    wa = sum(abs(_pnl_key(r) or 0.0) for r in wins) / w if w else None
    la = sum(abs(_pnl_key(r) or 0.0) for r in losses) / l if l else None
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


def _book_from_journal(
    outcomes: Path, *, events_path: Optional[Path] = None
) -> Dict[str, Any]:
    rows = _load_jsonl(outcomes)
    closed = [r for r in rows if _is_closed(r)]
    wl = _wl(closed)
    suppress = 0
    if events_path is not None and events_path.exists():
        for ev in _load_jsonl(events_path):
            et = str(ev.get("event_type") or ev.get("type") or "")
            if et == "SUPPRESSED_ADAPTER_SELECT_15M":
                suppress += 1
    # Also count from outcomes_summary if present on last state-ish rows
    for r in reversed(rows[-50:]):
        summ = r.get("outcomes_summary") if isinstance(r, dict) else None
        if isinstance(summ, dict) and "adapter_select_15m_suppressions" in summ:
            try:
                suppress = max(suppress, int(summ["adapter_select_15m_suppressions"]))
            except (TypeError, ValueError):
                pass
            break
    return {
        **wl,
        "outcomes_path": str(outcomes) if outcomes.exists() else None,
        "suppressed_adapter_select_15m": suppress,
    }


def _crypto_block(root: Path) -> Dict[str, Any]:
    primary = root / "trade_plan_overlay_v1" / "trade_plan_outcomes.jsonl"
    baseline = root / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_outcomes.jsonl"
    twin_dir = root / "trade_plan_overlay_adapter_select_15m_v1"
    twin_out = twin_dir / "trade_plan_outcomes.jsonl"
    twin_ev = twin_dir / "trade_plan_events.jsonl"
    twin = _book_from_journal(twin_out, events_path=twin_ev)
    state_path = twin_dir / "overlay_state.json"
    open_info: Dict[str, Any] = {"open": False}
    if state_path.exists():
        try:
            st = json.loads(state_path.read_text(encoding="utf-8"))
            ap = st.get("active_plan") if isinstance(st, dict) else None
            if isinstance(ap, dict) and ap.get("state") == "OPEN":
                sel = ap.get("adapter_select_15m") if isinstance(ap.get("adapter_select_15m"), dict) else {}
                open_info = {
                    "open": True,
                    "plan_id": ap.get("plan_id"),
                    "direction": ap.get("direction"),
                    "band": sel.get("band"),
                    "side_score": sel.get("side_score"),
                    "rust_ok": sel.get("rust_ok"),
                }
        except Exception:
            pass
    twin["active_open"] = open_info
    return {
        "A_primary": _book_from_journal(primary),
        "BASELINE": _book_from_journal(baseline),
        "adapter_select_15m": twin,
        "root": str(root),
    }


def _fmt(book: Dict[str, Any]) -> str:
    wr = book.get("wr")
    wr_s = f"{wr*100:.1f}%" if isinstance(wr, float) else "n/a"
    open_bit = ""
    ao = book.get("active_open") if isinstance(book.get("active_open"), dict) else None
    if ao and ao.get("open"):
        open_bit = (
            f" OPEN={ao.get('direction')}/{ao.get('band')}"
            f" side={ao.get('side_score')} rust_ok={ao.get('rust_ok')}"
        )
    return (
        f"n={book.get('n')} W/L={book.get('W')}/{book.get('L')} WR={wr_s} "
        f"payoff={book.get('payoff')} net={book.get('net')} "
        f"suppress_select={book.get('suppressed_adapter_select_15m')}{open_bit}"
    )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    payload = {
        "protocol_id": PROTOCOL,
        "logic_authority": "rust_adapter_select_15m",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "d1_d5": {
            "D1": "a_filter_on_coralys_side",
            "D2": "enter_abs_0.35",
            "D3": "btc_eth_observational",
            "D4": "leave_gate2e_b_soak_exclude_promote",
            "D5": "design_approved",
        },
        "BTC": _crypto_block(CRYPTO_ROOT),
        "ETH": _crypto_block(ETH_ROOT),
        "NSE": {"status": "NOT_ARMED_SLICE1"},
        "note": (
            "Gate 2E B trend-agree excluded from this promote path (D4). "
            "Pi arm 2026-11-27: RP608_ADAPTER_SELECT_15M_TWIN=1 on BTC+ETH; "
            "n=0 closed means soak started — check OPEN plans / events until first closes."
        ),
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    print(f"[{PROTOCOL}] W/L-first scorecard (books never pooled)")
    for label in ("BTC", "ETH"):
        block = payload[label]
        print(f"\n== {label} ==")
        for role in ("A_primary", "BASELINE", "adapter_select_15m"):
            print(f"  {role}: {_fmt(block.get(role) or {})}")
    print(f"\nNSE: {payload['NSE']['status']}")
    print(payload["note"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
