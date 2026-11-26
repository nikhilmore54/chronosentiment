#!/usr/bin/env python3
"""
RP-608-POLICY-C-ECON-001 — Multi-session economic evaluation (diagnostic).

Compares Policy A (same-session) vs Policy C (gap-neutral cross-session) on a
watchlist cohort or full universe. Frozen thresholds: traj15 >= +0.05,
entry bars 1..19, square-off bar 23. Authority: 0.0%.

Timing-advance pairs: early C entry vs later A entry for the same symbol/day
are compared on entry price and subsequent MFE/MAE — not double-counted as
incremental opportunities.

Does NOT change live G3.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.research.rp608_cross_session_warmup_diagnostic import (  # noqa: E402
    DEFAULT_SYMBOLS,
    Bar,
    IST,
    TRAJ_THRESHOLD,
    _bar_idx_for_session,
    _ist_date,
    bars_from_kite,
    evaluate_at,
    prior_session_date,
    split_sessions,
    subsequent_trajectory,
)

MAX_ENTRY_BAR = 19
SQUARE_OFF_BAR = 23
FRICTION_BPS = 12.0  # observational proxy: fees+spread+slippage combined


@dataclass
class TradeSim:
    symbol: str
    session_date: str
    policy: str
    entry_bar: int
    entry_price: float
    exit_bar: int
    exit_price: float
    classification: str  # incremental_early_only | timing_advance | baseline
    paired_baseline_entry_bar: Optional[int]
    paired_baseline_entry_price: Optional[float]
    raw_return_bps: float
    net_return_bps: float
    mfe_bps: float
    mae_bps: float
    hold_bars: int
    still_signal_at_plus_4: Optional[bool]


def _filter_session_bars(bars: Sequence[Bar]) -> List[Bar]:
    out: List[Bar] = []
    for b in bars:
        dt = datetime.fromtimestamp(b.timestamp_utc_ms / 1000.0, tz=timezone.utc).astimezone(IST)
        mins = dt.hour * 60 + dt.minute
        if 9 * 60 + 15 <= mins <= 15 * 60 + 30:
            out.append(b)
    return out


def _path_mfe_mae(entry: float, path_closes: Sequence[float]) -> Tuple[float, float]:
    if entry <= 0 or not path_closes:
        return 0.0, 0.0
    mfe = max(((c / entry) - 1.0) * 1e4 for c in path_closes)
    mae = min(((c / entry) - 1.0) * 1e4 for c in path_closes)
    return float(mfe), float(mae)


def simulate_long(
    *,
    symbol: str,
    session_date: str,
    today_bars: Sequence[Bar],
    prev_bars: Sequence[Bar],
    entry_bar: int,
    policy: str,
    classification: str,
    paired_baseline_entry_bar: Optional[int] = None,
    paired_baseline_entry_price: Optional[float] = None,
) -> Optional[TradeSim]:
    by_idx = {_bar_idx_for_session(b.timestamp_utc_ms, session_date): b for b in today_bars}
    if entry_bar not in by_idx or entry_bar > MAX_ENTRY_BAR:
        return None
    entry_px = float(by_idx[entry_bar].close)
    # Exit: first bar >= entry where traj fails under same policy history, else square-off.
    exit_bar = min(SQUARE_OFF_BAR, max(by_idx.keys()))
    exit_px = float(by_idx[exit_bar].close)
    path = []
    for b in range(entry_bar, exit_bar + 1):
        if b not in by_idx:
            continue
        today_prefix = [by_idx[i] for i in range(1, b + 1) if i in by_idx]
        hist = list(today_prefix) if policy == "same_session" else list(prev_bars) + list(today_prefix)
        ev = evaluate_at(policy=("cross_gap_neutral" if policy == "cross_gap_neutral" else "same_session"), history=hist, bar_idx=b)
        path.append(float(by_idx[b].close))
        if b > entry_bar and not ev["g4_pass"]:
            exit_bar = b
            exit_px = float(by_idx[b].close)
            break
    raw = ((exit_px / entry_px) - 1.0) * 1e4 if entry_px > 0 else 0.0
    mfe, mae = _path_mfe_mae(entry_px, path)
    # persistence probe
    today_prefix_e = [by_idx[i] for i in range(1, entry_bar + 1) if i in by_idx]
    # Build synthetic rows for subsequent_trajectory helper
    rows = []
    for b in sorted(by_idx):
        if b > MAX_ENTRY_BAR:
            break
        pref = [by_idx[i] for i in range(1, b + 1) if i in by_idx]
        hist = pref if policy == "same_session" else list(prev_bars) + pref
        pol = "same_session" if policy == "same_session" else "cross_gap_neutral"
        rows.append(evaluate_at(policy=pol, history=hist, bar_idx=b))
    pers = subsequent_trajectory(policy=("same_session" if policy == "same_session" else "cross_gap_neutral"), rows=rows, from_bar=entry_bar, horizon=4)
    return TradeSim(
        symbol=symbol,
        session_date=session_date,
        policy=policy,
        entry_bar=entry_bar,
        entry_price=entry_px,
        exit_bar=exit_bar,
        exit_price=exit_px,
        classification=classification,
        paired_baseline_entry_bar=paired_baseline_entry_bar,
        paired_baseline_entry_price=paired_baseline_entry_price,
        raw_return_bps=round(raw, 2),
        net_return_bps=round(raw - FRICTION_BPS, 2),
        mfe_bps=round(mfe, 2),
        mae_bps=round(mae, 2),
        hold_bars=int(exit_bar - entry_bar),
        still_signal_at_plus_4=pers.get("still_signal_at_horizon") if pers.get("available") else None,
    )


def analyze_symbol_session(symbol: str, all_bars: Sequence[Bar], session_date: str) -> Dict[str, Any]:
    by_sess = split_sessions(all_bars)
    today = by_sess.get(session_date, [])
    prev = prior_session_date(by_sess, session_date)
    prev_bars = by_sess.get(prev, []) if prev else []
    by_idx = {_bar_idx_for_session(b.timestamp_utc_ms, session_date): b for b in today}

    # First signals
    first_a = first_c = None
    for b in sorted(by_idx):
        if b < 1 or b > MAX_ENTRY_BAR:
            continue
        pref = [by_idx[i] for i in range(1, b + 1) if i in by_idx]
        ea = evaluate_at(policy="same_session", history=pref, bar_idx=b)
        ec = evaluate_at(policy="cross_gap_neutral", history=list(prev_bars) + pref, bar_idx=b)
        if first_a is None and ea["signal"]:
            first_a = b
        if first_c is None and ec["signal"]:
            first_c = b

    trades: List[TradeSim] = []
    if first_a is not None:
        t = simulate_long(
            symbol=symbol,
            session_date=session_date,
            today_bars=today,
            prev_bars=prev_bars,
            entry_bar=first_a,
            policy="same_session",
            classification="baseline",
        )
        if t:
            trades.append(t)
    if first_c is not None:
        if first_a is None:
            cls = "incremental_early_only"
            paired_b, paired_px = None, None
        elif first_c < first_a:
            cls = "timing_advance"
            paired_b = first_a
            paired_px = float(by_idx[first_a].close) if first_a in by_idx else None
        else:
            cls = "same_or_later_than_baseline"
            paired_b, paired_px = first_a, float(by_idx[first_a].close) if first_a in by_idx else None
        t = simulate_long(
            symbol=symbol,
            session_date=session_date,
            today_bars=today,
            prev_bars=prev_bars,
            entry_bar=first_c,
            policy="cross_gap_neutral",
            classification=cls,
            paired_baseline_entry_bar=paired_b,
            paired_baseline_entry_price=paired_px,
        )
        if t:
            trades.append(t)

    timing_edge_bps = None
    if first_c is not None and first_a is not None and first_c < first_a and first_c in by_idx and first_a in by_idx:
        # Price improvement of early entry vs later baseline entry (positive => early cheaper for LONG)
        timing_edge_bps = round((1.0 - (by_idx[first_c].close / by_idx[first_a].close)) * 1e4, 2)

    return {
        "symbol": symbol,
        "session_date": session_date,
        "prior_session": prev,
        "first_signal_bar_A": first_a,
        "first_signal_bar_C": first_c,
        "timing_advance_entry_edge_bps": timing_edge_bps,
        "trades": [asdict(t) for t in trades],
    }


def summarize(results: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    trades = [t for r in results for t in r.get("trades") or []]
    by_pol: Dict[str, List[Dict[str, Any]]] = {"same_session": [], "cross_gap_neutral": []}
    for t in trades:
        by_pol.setdefault(t["policy"], []).append(t)

    def _stats(ts: List[Dict[str, Any]]) -> Dict[str, Any]:
        if not ts:
            return {"n": 0}
        nets = [float(t["net_return_bps"]) for t in ts]
        wins = sum(1 for x in nets if x > 0)
        losses = sum(1 for x in nets if x <= 0)
        gp = sum(x for x in nets if x > 0)
        gl = abs(sum(x for x in nets if x <= 0))
        return {
            "n": len(ts),
            "mean_net_bps": round(sum(nets) / len(nets), 2),
            "win_rate": round(wins / len(ts), 4),
            "profit_factor": round(gp / gl, 4) if gl > 0 else None,
            "mean_mfe_bps": round(sum(float(t["mfe_bps"]) for t in ts) / len(ts), 2),
            "mean_mae_bps": round(sum(float(t["mae_bps"]) for t in ts) / len(ts), 2),
            "class_counts": {
                k: sum(1 for t in ts if t.get("classification") == k)
                for k in ("baseline", "incremental_early_only", "timing_advance", "same_or_later_than_baseline")
            },
        }

    edges = [r.get("timing_advance_entry_edge_bps") for r in results if r.get("timing_advance_entry_edge_bps") is not None]
    return {
        "n_symbol_sessions": len(results),
        "friction_bps_proxy": FRICTION_BPS,
        "policy_A_same_session": _stats(by_pol.get("same_session") or []),
        "policy_C_gap_neutral": _stats(by_pol.get("cross_gap_neutral") or []),
        "timing_advance_entry_edge_bps": {
            "n": len(edges),
            "mean": round(sum(edges) / len(edges), 2) if edges else None,
            "note": "Positive => early C LONG entry price below later A entry (favorable for long).",
        },
        "direction_bias_note": "Watchlist/cohort is LONG-oriented; SHORT not evaluated in this harness yet.",
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Policy C multi-session economics (0% authority)")
    ap.add_argument("--sessions", nargs="+", required=True, help="IST session dates YYYY-MM-DD")
    ap.add_argument("--symbols", nargs="*", default=None)
    ap.add_argument("--lookback-days", type=int, default=12)
    ap.add_argument("--max-symbols", type=int, default=20)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    symbols = tuple(args.symbols) if args.symbols else DEFAULT_SYMBOLS[: args.max_symbols]
    sessions = list(args.sessions)
    # Fetch once spanning min session - lookback through max session
    d0 = min(datetime.fromisoformat(s).date() for s in sessions)
    d1 = max(datetime.fromisoformat(s).date() for s in sessions)
    from_utc = datetime(d0.year, d0.month, d0.day, tzinfo=IST) - timedelta(days=int(args.lookback_days))
    to_utc = datetime(d1.year, d1.month, d1.day, 15, 30, tzinfo=IST)

    results: List[Dict[str, Any]] = []
    errors: Dict[str, str] = {}
    cache: Dict[str, List[Bar]] = {}
    for sym in symbols:
        try:
            raw = bars_from_kite(sym, from_utc.astimezone(timezone.utc), to_utc.astimezone(timezone.utc))
            cache[sym] = _filter_session_bars(raw)
            print(f"[ok] fetch {sym} bars={len(cache[sym])}")
        except Exception as exc:
            errors[sym] = f"{type(exc).__name__}: {exc}"
            print(f"[err] {sym}: {errors[sym]}")

    for sess in sessions:
        for sym, bars in cache.items():
            results.append(analyze_symbol_session(sym, bars, sess))

    payload = {
        "protocol_id": "RP-608-POLICY-C-ECON-001",
        "authority": "0.0%",
        "disposition": "DIAGNOSTIC_ONLY_NO_LIVE_G3_CHANGE",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "sessions": sessions,
        "symbols": list(symbols),
        "summary": summarize(results),
        "errors": errors,
        "results": results,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(json.dumps(payload["summary"], indent=2))
    print(f"[wrote] {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
