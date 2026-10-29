#!/usr/bin/env python3
"""
Backfill Prospective V1 recommendation + ABC journals from existing shadow cards.

CRYPTO: rp608_decision_cards.jsonl since eligible_session_boundary (UTC day).
NSE: rp608_nse_decision_cards.jsonl for session date when NSE STARTED.

Idempotent-ish: skips bar_idx/symbol keys already present in the recommendation tape.
0.0% authority — observational only.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Set, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_prospective_v1 import Market, is_market_started, load_start_event
from scripts.rp608_prospective_v1_observer import (
    observe_bar,
    recommendation_journal_path,
    record_recommendation,
)

CRYPTO_CARDS = WORKSPACE / "live_capture" / "shadow_validation" / "crypto" / "rp608_decision_cards.jsonl"
NSE_CARDS = WORKSPACE / "live_capture" / "shadow_validation" / "nse" / "rp608_nse_decision_cards.jsonl"


def _existing_keys(market: Market) -> Set[Tuple[Any, ...]]:
    path = recommendation_journal_path(market)
    keys: Set[Tuple[Any, ...]] = set()
    if not path.exists():
        return keys
    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            keys.add((r.get("symbol"), r.get("bar_idx"), r.get("close_time_ms"), r.get("decision")))
    return keys


def backfill_crypto(*, also_abc: bool = True) -> Dict[str, int]:
    if not is_market_started(Market.CRYPTO):
        return {"skipped": 1, "reason": "CRYPTO_not_started"}  # type: ignore[return-value]
    ev = load_start_event(Market.CRYPTO) or {}
    boundary = str(ev.get("eligible_session_boundary") or "")
    existing = _existing_keys(Market.CRYPTO)
    written = skipped = 0
    if not CRYPTO_CARDS.exists():
        return {"written": 0, "skipped": 0, "missing_cards": 1}
    with CRYPTO_CARDS.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                c = json.loads(line)
            except json.JSONDecodeError:
                continue
            ts = c.get("timestamp_utc") or ""
            try:
                day = datetime.fromisoformat(ts.replace("Z", "+00:00")).strftime("%Y-%m-%d")
            except Exception:
                continue
            if boundary and day < boundary:
                continue
            action_b = str(c.get("policy_b_action") or "NO_TRADE")
            action_c = str(c.get("policy_c_action") or "NO_TRADE")
            key = (c.get("symbol") or "BTCUSDT", c.get("bar_idx"), c.get("close_time_ms"), action_b)
            if key in existing:
                skipped += 1
                continue
            eq = c.get("execution_quote") or {}
            ask = eq.get("best_ask_price")
            bid = eq.get("best_bid_price")
            rec = record_recommendation(
                market=Market.CRYPTO,
                symbol="BTCUSDT",
                decision=action_b,
                secondary_decision=action_c,
                session_date=day,
                bar_idx=int(c.get("bar_idx") or 0),
                close_time_ms=c.get("close_time_ms"),
                reference_price=c.get("reference_price"),
                is_actionable=action_b in ("LONG", "SHORT"),
                regime=c.get("regime_state"),
                extra={
                    "backfill": True,
                    "policy_b_action": action_b,
                    "policy_c_action": action_c,
                    "valid_execution_quote": c.get("valid_execution_quote"),
                    "drop_reason": c.get("drop_reason"),
                    "ret_15m_bps": c.get("ret_15m_bps"),
                    "atr_14": c.get("atr_14"),
                },
            )
            if rec:
                written += 1
                existing.add(key)
            if also_abc and ask and bid and c.get("valid_execution_quote"):
                # Minimal 2-bar OHLC proxy from reference only — skip full ABC if no history.
                # Full ABC for backfill needs candle buffer; recommendation tape is primary here.
                pass
    return {"written": written, "skipped_existing": skipped, "boundary": boundary}  # type: ignore[dict-item]


def backfill_nse() -> Dict[str, Any]:
    if not is_market_started(Market.NSE):
        return {"skipped": 1, "reason": "NSE_not_started"}
    ev = load_start_event(Market.NSE) or {}
    boundary = str(ev.get("eligible_session_boundary") or "")
    existing = _existing_keys(Market.NSE)
    written = skipped = 0
    if not NSE_CARDS.exists():
        return {"written": 0, "skipped": 0, "missing_cards": 1}
    with NSE_CARDS.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                c = json.loads(line)
            except json.JSONDecodeError:
                continue
            # Prefer evaluation block when present
            evl = c.get("evaluation") if isinstance(c.get("evaluation"), dict) else c
            sess = str(evl.get("session_date") or c.get("session_date") or "")
            if boundary and sess and sess < boundary:
                continue
            if boundary and not sess:
                # try iso timestamp
                iso = evl.get("bar_timestamp_iso") or c.get("timestamp_utc") or ""
                try:
                    sess = datetime.fromisoformat(iso.replace("Z", "+00:00")).astimezone(
                        timezone.utc
                    ).strftime("%Y-%m-%d")
                except Exception:
                    pass
                if boundary and sess and sess < boundary:
                    continue
            decision = str(evl.get("decision") or c.get("decision") or "NO_TRADE")
            symbol = str(evl.get("symbol") or c.get("symbol") or "?")
            bar_idx = int(evl.get("bar_idx") or c.get("bar_idx") or 0)
            close_ms = evl.get("bar_timestamp_utc_ms") or c.get("close_time_ms")
            key = (symbol, bar_idx, close_ms, decision)
            if key in existing:
                skipped += 1
                continue
            rec = record_recommendation(
                market=Market.NSE,
                symbol=symbol,
                decision=decision,
                decision_code=evl.get("decision_code") or c.get("decision_code"),
                session_date=sess or boundary,
                bar_idx=bar_idx,
                close_time_ms=close_ms,
                reference_price=evl.get("reference_price") or c.get("reference_price"),
                is_actionable=(decision == "RECOMMEND_LONG"),
                regime=evl.get("regime_state"),
                extra={"backfill": True, "session_idx": evl.get("session_idx") or c.get("session_idx")},
            )
            if rec:
                written += 1
                existing.add(key)
    return {"written": written, "skipped_existing": skipped, "boundary": boundary}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--market", choices=["CRYPTO", "NSE", "ALL"], default="ALL")
    args = ap.parse_args()
    out: Dict[str, Any] = {}
    if args.market in ("CRYPTO", "ALL"):
        out["CRYPTO"] = backfill_crypto()
    if args.market in ("NSE", "ALL"):
        out["NSE"] = backfill_nse()
    print(json.dumps(out, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
