#!/usr/bin/env python3
"""
P2 Python behavioural reference — push watchlist + recommendation evaluation.

Mirrors adapters/chronosentiment/src/nse_market_events/push_eval.rs.
Frozen rules; no production writer; authority 0%.
"""

from __future__ import annotations

import json
import math
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

_ROOT = Path(__file__).resolve().parents[1]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.rp608_nse_next_session_recs import (  # noqa: E402
    _session_return_bps,
    _traj15 as traj15_watch_eod,
)
from scripts.rp608_order_flow_mark import classify_order_flow_support  # noqa: E402

TRAJ15_ENTRY_THRESHOLD = 0.05
MIN_BARS = 5
MAX_NEW_ENTRY_BAR = 19
MANDATORY_SQUARE_OFF_BAR = 23
WATCHLIST_TOP_N = 20


def traj15_live(closes: Sequence[float]) -> float:
    """Match run_rp608_nse_shadow_daemon.evaluate_symbol_bar."""
    if len(closes) < 2:
        return 0.0
    lookback = min(len(closes), 15)
    ret = (closes[-1] - closes[-lookback]) / (closes[-lookback] + 1e-9)
    diffs = [
        (closes[i] - closes[i - 1]) / (closes[i - 1] + 1e-9)
        for i in range(-lookback + 1, 0)
    ]
    vol = (sum(d * d for d in diffs) / len(diffs)) ** 0.5 if diffs else 1e-3
    denom = (vol * (lookback**0.5) * 2.5) + 1e-9
    return float(math.tanh(ret / denom))


def evaluate_recommendation(
    symbol: str,
    bar_idx: int,
    closes: Sequence[float],
    *,
    has_valid_fresh_quote: bool,
    quote_missing: bool = False,
    quote_stale: bool = False,
    quote_bad_prices: bool = False,
    quote_crossed: bool = False,
    existing_position_from_prior_bar: bool = False,
    prior_entry_bar: Optional[int] = None,
) -> Dict[str, Any]:
    traj15 = traj15_live(closes)
    signal_entry = (traj15 >= TRAJ15_ENTRY_THRESHOLD) and (len(closes) >= MIN_BARS)

    if existing_position_from_prior_bar:
        decision_code = "EXISTING_POSITION_ACTIVE"
        explanation = f"Existing active trade open since bar {prior_entry_bar or 0}"
        decision = "NO_TRADE"
    elif bar_idx >= MANDATORY_SQUARE_OFF_BAR:
        decision_code = "INTRADAY_BOUNDARY_EXCLUDED"
        explanation = (
            f"Session bar {bar_idx} >= {MANDATORY_SQUARE_OFF_BAR} "
            f"(15:00–15:15 closing window / mandatory square-off)"
        )
        decision = "NO_TRADE"
    elif bar_idx > MAX_NEW_ENTRY_BAR:
        decision_code = "INTRADAY_BOUNDARY_EXCLUDED"
        explanation = (
            f"Session bar {bar_idx} > {MAX_NEW_ENTRY_BAR} "
            f"(entry cutoff so 60m horizon completes by 15:00 close-window)"
        )
        decision = "NO_TRADE"
    elif len(closes) < MIN_BARS:
        decision_code = "COLD_START_INSUFFICIENT_HISTORY"
        explanation = f"Warmup history: {len(closes)}/5 bars required"
        decision = "NO_TRADE"
    elif traj15 < TRAJ15_ENTRY_THRESHOLD:
        decision_code = "SUB_THRESHOLD_MOMENTUM"
        explanation = f"Momentum traj15={traj15:+.4f} below entry threshold +0.0500"
        decision = "NO_TRADE"
    elif not has_valid_fresh_quote:
        decision_code = "QUOTE_FEASIBILITY_FAILURE"
        if quote_missing:
            explanation = "Missing top-of-book quote fails Gate G4 execution feasibility"
        elif quote_stale:
            explanation = (
                "Stale top-of-book quote (age > 60000.0ms) fails Gate G4 execution feasibility"
            )
        elif quote_bad_prices:
            explanation = "Zero top-of-book bid/ask fails Gate G4 execution feasibility"
        elif quote_crossed:
            explanation = "Crossed or locked top-of-book quote fails Gate G4 execution feasibility"
        else:
            explanation = "Invalid top-of-book quote fails Gate G4 execution feasibility"
        decision = "NO_TRADE"
    else:
        decision_code = "QUALIFIED_MOMENTUM_BREAKOUT"
        explanation = (
            f"Momentum breakout traj15={traj15:+.4f} >= +0.0500 with {len(closes)} bars history"
        )
        decision = "RECOMMEND_LONG"

    return {
        "symbol": symbol,
        "bar_idx": bar_idx,
        "traj15": round(traj15, 6),
        "signal_entry": bool(signal_entry),
        "decision": decision,
        "decision_code": decision_code,
        "explanation": explanation,
        "is_current": True,
        "suppressed_reason": None,
    }


def rank_watchlist(inputs: Sequence[Dict[str, Any]], top_n: int = 20) -> List[Dict[str, Any]]:
    cands: List[Dict[str, Any]] = []
    for inp in inputs:
        closes = list(inp["closes"])
        if len(closes) < MIN_BARS:
            continue
        traj = traj15_watch_eod(closes)
        sess = _session_return_bps(closes)
        if traj < TRAJ15_ENTRY_THRESHOLD and sess < 25.0:
            continue
        from scripts.rp608_nse_next_session_recs import SESSION_RETURN_WEIGHT

        score = traj + (sess / 10000.0) * SESSION_RETURN_WEIGHT
        l1 = {
            "bid_qty": inp.get("bid_qty", 0),
            "ask_qty": inp.get("ask_qty", 0),
            "bid_price": inp.get("bid_price", 0),
            "ask_price": inp.get("ask_price", 0),
            "source": "fixture_replay",
        }
        of = classify_order_flow_support("LONG", l1)
        cands.append(
            {
                "symbol": inp["symbol"],
                "side": "LONG",
                "decision": "WATCH_NEXT_SESSION",
                "status": "WATCH_NEXT_SESSION",
                "score": round(score, 6),
                "traj15_eod": round(traj, 6),
                "session_return_bps": round(sess, 4),
                "close_price": round(closes[-1], 4),
                "session_bars": len(closes),
                "order_flow_supported": bool(of["order_flow_supported"]),
                "order_flow_label": of["order_flow_label"],
                "order_flow_imbalance": of["order_flow_imbalance"],
                "order_flow_source": of["order_flow_source"],
            }
        )
    cands.sort(key=lambda c: (-c["score"], c["symbol"]))
    return cands[: min(int(top_n), WATCHLIST_TOP_N)]


@dataclass
class PushEvalEngineRef:
    watch_symbols: List[str]
    states: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    feed_stale: bool = False
    stale_reason: Optional[str] = None

    def __post_init__(self) -> None:
        self.watch_symbols = sorted(self.watch_symbols)[:WATCHLIST_TOP_N]
        for s in self.watch_symbols:
            self.states[s] = {"closes": [], "last_eval": None}

    def on_disconnect(self) -> None:
        self.feed_stale = True
        self.stale_reason = "disconnect"
        self._suppress()

    def begin_reconcile(self) -> None:
        self.feed_stale = True
        self.stale_reason = "disconnect"
        self._suppress()

    def complete_reconcile(self) -> None:
        self.feed_stale = False
        self.stale_reason = None
        self._suppress()

    def _suppress(self) -> None:
        for st in self.states.values():
            if st["last_eval"]:
                st["last_eval"]["is_current"] = False
                st["last_eval"]["suppressed_reason"] = self.stale_reason

    def on_bar_complete(
        self,
        symbol: str,
        bar_idx: int,
        close: float,
        *,
        has_valid_fresh_quote: bool,
        quote_stale: bool = False,
    ) -> Optional[Dict[str, Any]]:
        st = self.states.get(symbol)
        if st is None:
            return None
        st["closes"].append(float(close))
        eval_ = evaluate_recommendation(
            symbol,
            bar_idx,
            st["closes"],
            has_valid_fresh_quote=has_valid_fresh_quote and not self.feed_stale,
            quote_stale=quote_stale or self.feed_stale,
        )
        if self.feed_stale:
            eval_["is_current"] = False
            eval_["suppressed_reason"] = self.stale_reason
        st["last_eval"] = eval_
        return eval_

    def current_recommendation(self, symbol: str) -> Optional[Dict[str, Any]]:
        if self.feed_stale:
            return None
        ev = self.states.get(symbol, {}).get("last_eval")
        if ev and ev.get("is_current"):
            return ev
        return None


def write_p2_fixtures(out_dir: Path) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)

    # Rising closes → strong traj15_live
    rising = [100.0 + i * 0.5 for i in range(8)]
    flat = [100.0] * 8
    weak = [100.0 - i * 0.1 for i in range(8)]

    rec_cases = [
        {
            "name": "recommend_long",
            "symbol": "AAA.NS",
            "bar_idx": 10,
            "closes": rising,
            "has_valid_fresh_quote": True,
        },
        {
            "name": "sub_threshold",
            "symbol": "BBB.NS",
            "bar_idx": 10,
            "closes": flat,
            "has_valid_fresh_quote": True,
        },
        {
            "name": "cold_start",
            "symbol": "CCC.NS",
            "bar_idx": 3,
            "closes": rising[:3],
            "has_valid_fresh_quote": True,
        },
        {
            "name": "entry_cutoff",
            "symbol": "DDD.NS",
            "bar_idx": 20,
            "closes": rising,
            "has_valid_fresh_quote": True,
        },
        {
            "name": "square_off",
            "symbol": "EEE.NS",
            "bar_idx": 23,
            "closes": rising,
            "has_valid_fresh_quote": True,
        },
        {
            "name": "stale_quote",
            "symbol": "FFF.NS",
            "bar_idx": 10,
            "closes": rising,
            "has_valid_fresh_quote": False,
            "quote_stale": True,
        },
    ]
    rec_out = []
    for c in rec_cases:
        ev = evaluate_recommendation(
            c["symbol"],
            c["bar_idx"],
            c["closes"],
            has_valid_fresh_quote=c["has_valid_fresh_quote"],
            quote_stale=bool(c.get("quote_stale")),
        )
        rec_out.append({**c, "eval": ev, "traj15_live": traj15_live(c["closes"])})

    watch_inputs = [
        {
            "symbol": "W1.NS",
            "closes": rising,
            "bid_qty": 90,
            "ask_qty": 10,
            "bid_price": 99.9,
            "ask_price": 100.1,
        },
        {
            "symbol": "W2.NS",
            "closes": weak,
            "bid_qty": 50,
            "ask_qty": 50,
            "bid_price": 99.9,
            "ask_price": 100.1,
        },
        {
            "symbol": "W3.NS",
            "closes": [100 + i * 0.3 for i in range(10)],
            "bid_qty": 10,
            "ask_qty": 90,
            "bid_price": 99.9,
            "ask_price": 100.1,
        },
        {
            "symbol": "W4.NS",
            "closes": flat[:4],  # skipped <5 bars
            "bid_qty": 90,
            "ask_qty": 10,
            "bid_price": 99.9,
            "ask_price": 100.1,
        },
    ]
    # Add filler to test top_n cap path
    for i in range(25):
        watch_inputs.append(
            {
                "symbol": f"Z{i:02d}.NS",
                "closes": [100.0 + i * 0.01 + j * 0.2 for j in range(8)],
                "bid_qty": 60,
                "ask_qty": 40,
                "bid_price": 99.9,
                "ask_price": 100.1,
            }
        )
    watch = rank_watchlist(watch_inputs, top_n=20)

    eng = PushEvalEngineRef(["AAA.NS", "BBB.NS"])
    for i, c in enumerate(rising[:-1], start=1):
        eng.on_bar_complete("AAA.NS", i, c, has_valid_fresh_quote=True)
    e1 = eng.on_bar_complete("AAA.NS", 10, rising[-1], has_valid_fresh_quote=True)
    before = eng.current_recommendation("AAA.NS")
    eng.on_disconnect()
    during = eng.current_recommendation("AAA.NS")
    eng.begin_reconcile()
    eng.complete_reconcile()
    after_recon = eng.current_recommendation("AAA.NS")
    e2 = eng.on_bar_complete("AAA.NS", 11, rising[-1] + 0.5, has_valid_fresh_quote=True)
    after_live = eng.current_recommendation("AAA.NS")

    golden = {
        "protocol": "RP608-NSE-P2-PUSH-EVAL",
        "constants": {
            "traj15_entry_threshold": TRAJ15_ENTRY_THRESHOLD,
            "max_new_entry_bar": MAX_NEW_ENTRY_BAR,
            "mandatory_square_off_bar": MANDATORY_SQUARE_OFF_BAR,
            "watchlist_top_n": WATCHLIST_TOP_N,
        },
        "recommendation_cases": rec_out,
        "watchlist": watch,
        "watchlist_n": len(watch),
        "engine": {
            "before_disconnect_decision": (before or {}).get("decision"),
            "during_stale_current": during,
            "after_recon_current": after_recon,
            "after_live_decision": (after_live or {}).get("decision"),
            "after_live_is_current": (after_live or {}).get("is_current"),
            "e1_decision_code": (e1 or {}).get("decision_code"),
            "e2_decision_code": (e2 or {}).get("decision_code"),
        },
    }
    path = out_dir / "p2_push_eval_differential.json"
    path.write_text(json.dumps(golden, indent=2) + "\n", encoding="utf-8")
    return path


if __name__ == "__main__":
    out = (
        _ROOT
        / "adapters"
        / "chronosentiment"
        / "tests"
        / "fixtures"
        / "nse_kite_full"
    )
    print(write_p2_fixtures(out))
