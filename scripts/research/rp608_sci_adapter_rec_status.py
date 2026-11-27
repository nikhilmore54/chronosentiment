#!/usr/bin/env python3
"""
RP-608-SCI-ADAPTER-REC-001 P3 scorecard — W/L-first (BTC book).

Compares SCI twin vs PRIMARY (A) vs BASELINE vs adapter_select / Gate2E B
as observational baselines. Books never pooled. Authority 0%.

Reads the live journal filenames used by TradePlanOverlayManager:
  trade_plan_outcomes.jsonl · trade_plan_events.jsonl · overlay_state.json

Usage:
  python3 scripts/research/rp608_sci_adapter_rec_status.py
  python3 scripts/research/rp608_sci_adapter_rec_status.py --json
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
PROTOCOL = "RP-608-SCI-ADAPTER-REC-001"
CRYPTO_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto"

# Event types that are SCI-twin-specific or useful soak diagnostics.
_SCI_SUPPRESS = "SUPPRESSED_SCI_ADAPTER_REC"
_DIAG_EVENTS = (
    _SCI_SUPPRESS,
    "SUPPRESSED_GATE2E_DI_SKEW",
    "SUPPRESSED_GATE2E_REGIME_DIRECTIONAL",
    "SUPPRESSED_GATE2E_PATH_INVALID",
    "SUPPRESSED_GATE2E_TREND_AGREE",
    "SUPPRESSED_ADAPTER_SELECT_15M",
    "SUPPRESSED_OPEN_PLAN",
    "PLAN_CREATED",
)


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


def _event_counts(events_path: Path) -> Dict[str, int]:
    c: Counter[str] = Counter()
    for ev in _load_jsonl(events_path):
        et = str(ev.get("event_type") or ev.get("type") or "")
        if et:
            c[et] += 1
    return dict(c)


def _book(
    root: Path,
    twin_dir: str,
    *,
    primary_suppress_event: str = "",
) -> Dict[str, Any]:
    base = root / twin_dir
    outcomes = base / "trade_plan_outcomes.jsonl"
    events = base / "trade_plan_events.jsonl"
    state_path = base / "overlay_state.json"

    rows = _load_jsonl(outcomes)
    # Fallback: closed rows sometimes only appear as CLOSED_* in events.
    if not rows:
        rows = [r for r in _load_jsonl(events) if _is_closed(r)]
    closed = [r for r in rows if _is_closed(r)]
    wl = _wl(closed)

    ev_counts = _event_counts(events) if events.exists() else {}
    suppress = int(ev_counts.get(primary_suppress_event, 0)) if primary_suppress_event else 0

    open_n = 0
    state_summ: Dict[str, Any] = {}
    if state_path.exists():
        try:
            st = json.loads(state_path.read_text(encoding="utf-8"))
            if isinstance(st, dict):
                ap = st.get("active_plan")
                if isinstance(ap, dict) and str(ap.get("state") or "").upper() == "OPEN":
                    open_n = 1
                summ = st.get("outcomes_summary")
                if isinstance(summ, dict):
                    state_summ = {
                        k: summ.get(k)
                        for k in (
                            "sci_adapter_rec_suppressions",
                            "gate2e_di_skew_suppressions",
                            "gate2e_regime_align_suppressions",
                            "gate2e_trend_agree_suppressions",
                            "adapter_select_15m_suppressions",
                            "cumulative_net_pnl_usdt",
                            "tp_hits",
                            "sl_hits",
                            "time_exits",
                        )
                        if k in summ
                    }
                    if primary_suppress_event == _SCI_SUPPRESS:
                        try:
                            suppress = max(
                                suppress,
                                int(summ.get("sci_adapter_rec_suppressions") or 0),
                            )
                        except (TypeError, ValueError):
                            pass
        except Exception:
            pass

    diag = {k: int(ev_counts.get(k, 0)) for k in _DIAG_EVENTS if ev_counts.get(k)}
    # Include any other SUPPRESSED_* / CLOSED_* for soak visibility (capped).
    for k, v in sorted(ev_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        if k.startswith("SUPPRESSED_") or k.startswith("CLOSED_") or k == "PLAN_CREATED":
            diag.setdefault(k, int(v))
        if len(diag) >= 16:
            break

    return {
        "path": str(base),
        "armed": base.exists(),
        **wl,
        "suppress": suppress,
        "open": open_n,
        "event_diag": diag,
        "state_summary": state_summ,
        "outcomes_path": str(outcomes) if outcomes.exists() else None,
        "events_path": str(events) if events.exists() else None,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--root", type=Path, default=CRYPTO_ROOT)
    args = ap.parse_args()
    generated = datetime.now(timezone.utc).isoformat()

    sci = _book(
        args.root,
        "trade_plan_overlay_sci_adapter_rec_v1",
        primary_suppress_event=_SCI_SUPPRESS,
    )
    primary = _book(args.root, "trade_plan_overlay_v1")
    baseline = _book(args.root, "trade_plan_overlay_baseline_parallel_v1")
    adapter_select = _book(
        args.root,
        "trade_plan_overlay_adapter_select_15m_v1",
        primary_suppress_event="SUPPRESSED_ADAPTER_SELECT_15M",
    )
    trend_agree = _book(
        args.root,
        "trade_plan_overlay_trend_agree_parallel_v1",
        primary_suppress_event="SUPPRESSED_GATE2E_TREND_AGREE",
    )

    out = {
        "protocol_id": PROTOCOL,
        "generated_utc": generated,
        "book": "BTCUSDT",
        "sci_adapter_rec": sci,
        "primary_a": primary,
        "baseline": baseline,
        "adapter_select_baseline": adapter_select,
        "gate2e_b_baseline": trend_agree,
        "notes": [
            "W/L-first per book; never pooled (.cursor/rules chronosentiment-core §3b)",
            "SCI twin env RP608_SCI_ADAPTER_REC_TWIN=1 on BTC Pi observer (P3)",
            "Gate 2E B + adapter_select left observational (S6)",
            "Journals: trade_plan_outcomes.jsonl + trade_plan_events.jsonl",
        ],
    }
    if args.json:
        print(json.dumps(out, indent=2))
        return 0

    print(f"[{PROTOCOL}] BTC scorecard  generated_utc={generated}")
    for name, block in (
        ("SCI_ADAPTER_REC", sci),
        ("PRIMARY_A", primary),
        ("BASELINE", baseline),
        ("ADAPTER_SELECT", adapter_select),
        ("GATE2E_B", trend_agree),
    ):
        print(
            f"  {name:16} armed={block['armed']}  "
            f"W/L={block['W']}/{block['L']} n={block['n']} wr={block['wr']}  "
            f"payoff={block['payoff']} net={block['net']}  "
            f"suppress={block['suppress']} open={block['open']}"
        )
        diag = block.get("event_diag") or {}
        if diag and name in {"SCI_ADAPTER_REC", "ADAPTER_SELECT", "GATE2E_B"}:
            top = ", ".join(f"{k}={v}" for k, v in list(diag.items())[:6])
            print(f"    events: {top}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
