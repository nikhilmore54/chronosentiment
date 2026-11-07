#!/usr/bin/env python3
"""
scripts/rp608_trade_plan_overlay.py

RP-608 Trade-Plan Overlay v1: Shadow Execution & Lifecycle State Engine.
Implements Protocol: RP-608-TRADE-PLAN-OVERLAY-001
Strictly Read-Only · Zero Order Authority (0.0%)

Generates and monitors complete trade plans from Policy B recommendations:
  - Eligible Entry: Ask for LONG, Bid for SHORT
  - Stop Loss:  1.0 x ATR(14)
  - Profit Target: 1.5 x ATR(14)
  - Holding Horizon: 60 minutes (3,600s) from simulated entry
  - Net Reward/Risk: Post-12.0 bps round-trip friction and spread crossing
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import sys
import time
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

from scripts.rp608_order_flow_mark import (
    ORDER_FLOW_EXIT_LABEL,
    ORDER_FLOW_LABEL,
    classify_order_flow_support,
    is_order_flow_exit_signal,
    l1_from_book_ticker,
    l1_from_tick_features,
)

CANDLE_15M_MS = 900_000


def build_of_candle_mark(
    *,
    now_ms: int,
    card: Optional[Dict[str, Any]] = None,
    bar_idx: Optional[int] = None,
    decision_event_idx: Optional[int] = None,
) -> Dict[str, Any]:
    """
    Exact 15m candle identity for OF entry/exit UI markers.
    open = floor(t / 15m); close = open + 15m (Binance kline convention).
    """
    card = card or {}
    mark_ms = int(card.get("close_time_ms") or now_ms)
    open_ms = (mark_ms // CANDLE_15M_MS) * CANDLE_15M_MS
    close_ms = open_ms + CANDLE_15M_MS
    b_idx = bar_idx if bar_idx is not None else card.get("bar_idx")
    e_idx = (
        decision_event_idx
        if decision_event_idx is not None
        else card.get("decision_event_idx")
    )
    try:
        b_idx = int(b_idx) if b_idx is not None else None
    except (TypeError, ValueError):
        b_idx = None
    try:
        e_idx = int(e_idx) if e_idx is not None else None
    except (TypeError, ValueError):
        e_idx = None

    def _iso(ms: int) -> str:
        return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(ms / 1000.0))

    return {
        "bar_idx": b_idx,
        "decision_event_idx": e_idx,
        "candle_open_time_ms": open_ms,
        "candle_close_time_ms": close_ms,
        "candle_open_utc": _iso(open_ms),
        "candle_close_utc": _iso(close_ms),
        "mark_time_ms": mark_ms,
        "mark_time_utc": _iso(mark_ms),
        "candle_label": (
            f"15m #{b_idx}" if b_idx is not None else "15m"
        )
        + f" {_iso(open_ms)[11:16]}–{_iso(close_ms)[11:16]} UTC",
    }

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
CRYPTO_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
OVERLAY_DIR = CRYPTO_DIR / "trade_plan_overlay_v1"
OVERLAY_DIR.mkdir(parents=True, exist_ok=True)

DECISION_CARDS_FILE = CRYPTO_DIR / "rp608_decision_cards.jsonl"
STATE_FILE = OVERLAY_DIR / "overlay_state.json"
EVENTS_FILE = OVERLAY_DIR / "trade_plan_events.jsonl"
OUTCOMES_FILE = OVERLAY_DIR / "trade_plan_outcomes.jsonl"
OF_OUTCOMES_FILE = OVERLAY_DIR / "of_trade_outcomes.jsonl"
OF_PAPER_OPENS_FILE = OVERLAY_DIR / "of_paper_trade_opens.jsonl"
OF_PAPER_OPEN_STATE_FILE = OVERLAY_DIR / "of_paper_trades_open.json"
OF_PAPER_PROTOCOL_ID = "RP-608-OF-PAPER-TRADE-001"
OF_DEPTH_QUALITY_PROTOCOL_ID = "RP-608-OF-DEPTH-QUALITY-001"
CONFIG_FILE = OVERLAY_DIR / "overlay_config.json"


def _iso_utc_ms(ms: int) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(ms / 1000.0))


def _l1_book_qtys(quote: Optional[Dict[str, Any]]) -> tuple:
    q = quote if isinstance(quote, dict) else {}
    try:
        bid_qty = float(
            q.get("best_bid_qty") if q.get("best_bid_qty") is not None else q.get("bidQty") or 0.0
        )
    except (TypeError, ValueError):
        bid_qty = 0.0
    try:
        ask_qty = float(
            q.get("best_ask_qty") if q.get("best_ask_qty") is not None else q.get("askQty") or 0.0
        )
    except (TypeError, ValueError):
        ask_qty = 0.0
    return bid_qty, ask_qty


def of_entry_signal_qty(direction: str, quote: Optional[Dict[str, Any]]) -> float:
    """L1 qty on the OF entry candle side: LONG→bidQty, SHORT→askQty."""
    d = str(direction or "").upper()
    bid_qty, ask_qty = _l1_book_qtys(quote)
    if d == "LONG":
        return bid_qty
    if d == "SHORT":
        return ask_qty
    return 0.0


def of_exit_signal_qty(direction: str, quote: Optional[Dict[str, Any]]) -> float:
    """
    L1 qty on the reverse-OF exit candle side (informational only).
    LONG flat→opposite SHORT confirm uses askQty; SHORT flat→bidQty.
    Never sizes the paper exit — exit fill qty must equal entry paper qty.
    """
    d = str(direction or "").upper()
    bid_qty, ask_qty = _l1_book_qtys(quote)
    if d == "LONG":
        return ask_qty
    if d == "SHORT":
        return bid_qty
    return 0.0


def compute_of_depth_quality(
    *,
    direction: str,
    quote: Optional[Dict[str, Any]] = None,
    imbalance: Optional[float] = None,
    phase: str = "entry",
) -> Dict[str, Any]:
    """
    RP-608-OF-DEPTH-QUALITY-001 — TELEMETRY ONLY.

    Imbalance is relative dominance; these fields measure absolute liquidity so
    thin-touch extremes (imb≈±1 with empty opposite) can be studied without
    changing frozen OF ±0.30 entry / reverse-flow exit rules.
    """
    d = str(direction or "").upper()
    q = quote if isinstance(quote, dict) else {}
    bid_qty, ask_qty = _l1_book_qtys(q)
    try:
        bid_px = float(q.get("best_bid_price") if q.get("best_bid_price") is not None else q.get("bidPrice") or 0.0)
    except (TypeError, ValueError):
        bid_px = 0.0
    try:
        ask_px = float(q.get("best_ask_price") if q.get("best_ask_price") is not None else q.get("askPrice") or 0.0)
    except (TypeError, ValueError):
        ask_px = 0.0
    mid = (bid_px + ask_px) / 2.0 if bid_px > 0 and ask_px > 0 else max(bid_px, ask_px, 0.0)

    bid_notional = bid_qty * bid_px if bid_px > 0 else bid_qty * mid
    ask_notional = ask_qty * ask_px if ask_px > 0 else ask_qty * mid
    combined_qty = bid_qty + ask_qty
    combined_notional = bid_notional + ask_notional

    if d == "LONG":
        same_qty, same_notional = bid_qty, bid_notional
        opp_qty, opp_notional = ask_qty, ask_notional
        same_side, opp_side = "bid", "ask"
    elif d == "SHORT":
        same_qty, same_notional = ask_qty, ask_notional
        opp_qty, opp_notional = bid_qty, bid_notional
        same_side, opp_side = "ask", "bid"
    else:
        same_qty = same_notional = opp_qty = opp_notional = 0.0
        same_side = opp_side = "none"

    if imbalance is not None:
        try:
            imb = float(imbalance)
        except (TypeError, ValueError):
            imb = None
    else:
        denom = combined_qty
        imb = ((bid_qty - ask_qty) / denom) if denom > 0 else None

    # Diagnostic only — does NOT gate entry/exit
    thin_opposite = bool(same_notional > 0 and opp_notional < max(1.0, 0.05 * same_notional))

    return {
        "protocol_id": OF_DEPTH_QUALITY_PROTOCOL_ID,
        "phase": phase,
        "direction": d,
        "l1_imbalance": round(imb, 6) if imb is not None else None,
        "best_bid_qty": round(bid_qty, 8),
        "best_ask_qty": round(ask_qty, 8),
        "best_bid_price": round(bid_px, 8) if bid_px else None,
        "best_ask_price": round(ask_px, 8) if ask_px else None,
        "mid_price": round(mid, 8) if mid else None,
        "combined_touch_qty": round(combined_qty, 8),
        "combined_touch_notional_usdt": round(combined_notional, 4),
        "same_side": same_side,
        "same_side_touch_qty": round(same_qty, 8),
        "same_side_touch_notional_usdt": round(same_notional, 4),
        "opposing_side": opp_side,
        "opposing_side_touch_qty": round(opp_qty, 8),
        "opposing_side_touch_notional_usdt": round(opp_notional, 4),
        "thin_opposite_diagnostic": thin_opposite,
        "primary_depth_metric": "combined_touch_notional_usdt",
        "gates_entry": False,
        "gates_exit": False,
        "note": "Telemetry only — frozen OF ±0.30 entry / reverse-flow exit unchanged",
    }


def size_of_paper_from_entry_signal(
    *,
    direction: str,
    entry_price: float,
    quote: Optional[Dict[str, Any]] = None,
    participation_pct: Optional[float] = None,
) -> Dict[str, Any]:
    """
    PAPER size from the OF entry-candle L1 signal qty (not fixed $1000).

    LONG uses best_bid_qty; SHORT uses best_ask_qty on that entry mark.
    Target participation defaults to 10%, hard-clamped to [5%, 15%] of signal qty,
    then notional-clamped to [of_paper_min_notional, of_paper_max_notional].

    Contract: exit fill qty MUST equal this simulated_quantity. Exit-book
    signal qty is recorded separately and must not resize the close.
    """
    d = str(direction or "").upper()
    bid_qty, ask_qty = _l1_book_qtys(quote)
    signal_qty = of_entry_signal_qty(d, quote)
    pct_min = float(CONFIG.get("of_paper_participation_pct_min", 0.05))
    pct_max = float(CONFIG.get("of_paper_participation_pct_max", 0.15))
    pct_default = float(CONFIG.get("of_paper_participation_pct", 0.10))
    pct = float(participation_pct) if participation_pct is not None else pct_default
    pct = max(pct_min, min(pct_max, pct))

    raw_qty = max(0.0, signal_qty) * pct
    px = float(entry_price) if entry_price and entry_price > 0 else 0.0
    min_notional = float(CONFIG.get("of_paper_min_notional_usdt", 25.0))
    max_notional = float(CONFIG.get("of_paper_max_notional_usdt", 500.0))
    fallback_notional = float(CONFIG.get("of_paper_fallback_notional_usdt", 100.0))

    if px <= 0:
        return {
            "simulated_quantity": 0.0,
            "simulated_notional_usdt": 0.0,
            "entry_signal_qty": signal_qty,
            "participation_pct": pct,
            "sizing_source": "invalid_entry_price",
        }

    if raw_qty <= 0:
        qty = fallback_notional / px
        notional = fallback_notional
        source = "fallback_notional_no_signal_qty"
    else:
        notional = raw_qty * px
        if notional < min_notional:
            qty = min_notional / px
            notional = min_notional
            source = "clamped_min_notional"
        elif notional > max_notional:
            qty = max_notional / px
            notional = max_notional
            source = "clamped_max_notional"
        else:
            qty = raw_qty
            source = "entry_signal_pct"

    return {
        "simulated_quantity": round(qty, 8),
        "simulated_notional_usdt": round(notional, 4),
        "entry_signal_qty": round(signal_qty, 8),
        "entry_bid_qty": round(bid_qty, 8),
        "entry_ask_qty": round(ask_qty, 8),
        "participation_pct": pct,
        "participation_pct_min": pct_min,
        "participation_pct_max": pct_max,
        "sizing_source": source,
    }


def compute_of_paper_cost_breakdown(
    *,
    direction: str,
    entry_price: float,
    exit_price: float,
    notional: float,
    entry_bid: Optional[float] = None,
    entry_ask: Optional[float] = None,
    exit_bid: Optional[float] = None,
    exit_ask: Optional[float] = None,
    fee_bps_per_leg: float = 6.0,
    quantity: Optional[float] = None,
) -> Dict[str, Any]:
    """
    Deterministic PAPER cost + P&L for OF trades (RP-608-OF-PAPER-TRADE-001).

    Gross is mid→mid; spread crossing, fees (6 bps/leg), and modeled slippage
    are explicit. friction = spread + fees + slippage. Net = gross_mid − friction.
    Slippage v1: slippage_model=none → 0.0 (field always present).

    If quantity is provided (preferred), exit uses that exact entry qty — never
    re-sizes from exit-book depth.
    """
    if quantity is not None and float(quantity) > 0 and entry_price > 0:
        qty = float(quantity)
        entry_notional = qty * float(entry_price)
    else:
        entry_notional = float(notional)
        qty = entry_notional / entry_price if entry_price > 0 else 0.0
    d = str(direction or "").upper()

    def _mid(bid: Optional[float], ask: Optional[float], fallback: float) -> float:
        if bid is not None and ask is not None and ask >= bid > 0:
            return (bid + ask) / 2.0
        return fallback

    entry_mid = _mid(entry_bid, entry_ask, entry_price)
    exit_mid = _mid(exit_bid, exit_ask, exit_price)
    if d == "LONG":
        gross_mid = (exit_mid - entry_mid) * qty
    else:
        gross_mid = (entry_mid - exit_mid) * qty

    spread_entry = abs(entry_price - entry_mid) * qty
    spread_exit = abs(exit_price - exit_mid) * qty
    spread_cost = spread_entry + spread_exit
    exit_notional = exit_price * qty
    fees = (entry_notional + exit_notional) * (fee_bps_per_leg / 10000.0)
    slippage = 0.0
    friction = spread_cost + fees + slippage
    net = gross_mid - friction
    net_bps = (net / entry_notional) * 10000.0 if entry_notional > 0 else 0.0
    return {
        "simulated_quantity": round(qty, 8),
        "simulated_notional_usdt": round(entry_notional, 4),
        "entry_mid": round(entry_mid, 8),
        "exit_mid": round(exit_mid, 8),
        "gross_pnl_usdt": round(gross_mid, 4),
        "spread_cost_usdt": round(spread_cost, 4),
        "fees_usdt": round(fees, 4),
        "slippage_usdt": round(slippage, 4),
        "slippage_model": "none",
        "friction_usdt": round(friction, 4),
        "net_pnl_usdt": round(net, 4),
        "net_return_bps": round(net_bps, 2),
        "fee_bps_per_leg": fee_bps_per_leg,
    }

# Fixed Governance Parameters (Protocol RP-608-TRADE-PLAN-OVERLAY-001)
CONFIG: Dict[str, Any] = {
    "protocol_id": "RP-608-TRADE-PLAN-OVERLAY-001",
    "version": "1.2.3",
    "instrument": "BTC/USDT",
    "signal_policy": "B",
    # Signals open on tick bookTicker decisions (RP-608-SHADOW-TICK-001);
    # ATR(14) SL/TP geometry still frozen from the 15m bar buffer at open;
    # TP/SL lifecycle remains continuous L1 (not 15m-gated).
    "lifecycle_monitor": "continuous_l1_bookTicker",
    "signal_cadence": "tick_book_ticker",
    "atr_geometry_source": "completed_15m_atr14",
    "stop_multiplier": 1.0,
    "target_multiplier": 1.5,
    "max_holding_seconds": 3600,  # 60 minutes
    "simulated_notional_usdt": 1000.0,
    "price_tick_size": 0.01,
    "taker_fee_bps": 5.0,
    "slippage_bps": 1.0,
    "round_trip_friction_bps": 12.0,  # 2 * (5.0 + 1.0) bps
    "max_quote_age_ms": 500,
    "order_authority_pct": 0.0,
    # OF paper anti-churn (PAPER only; does not raise live authority)
    # L1 imbalance can flip ±0.30 within seconds — without these guards the
    # paper ledger round-trips on noise (fee-dominated "sells" with ~0 hold).
    # Stage-1 ledger (n=7): all sub-5m reverse-OF exits were fee-dominated losses;
    # RT fee floor ≈12 bps — require sustained reverse OF after meaningful hold.
    "of_paper_min_hold_ms": 300_000,           # 5m min hold before reverse-OF close
    "of_paper_opposite_persist_ms": 30_000,    # 30s continuous opposite OF
    "of_paper_one_per_15m_bar": True,
    # PAPER open quality (does NOT change OF ±0.30 mark — only whether a paper
    # trade is booked). Rejects micro/hollow books that Stage-1 showed as bleed.
    "of_paper_reject_thin_opposite": True,
    "of_paper_min_entry_signal_qty": 0.25,     # BTC L1 same-side qty floor
    "of_paper_min_same_side_notional_usdt": 5_000.0,
    # OF paper size = % of entry-side L1 qty (LONG=bidQty, SHORT=askQty)
    "of_paper_participation_pct": 0.10,       # target 10%
    "of_paper_participation_pct_min": 0.05,   # floor 5%
    "of_paper_participation_pct_max": 0.15,   # cap 15%
    "of_paper_min_notional_usdt": 25.0,
    "of_paper_max_notional_usdt": 500.0,
    "of_paper_fallback_notional_usdt": 100.0,  # if signal qty missing
}

CONFIG_HASH = hashlib.sha256(json.dumps(CONFIG, sort_keys=True).encode("utf-8")).hexdigest()
CONFIG["config_hash"] = CONFIG_HASH


def save_config_if_missing():
    if not CONFIG_FILE.exists():
        with open(CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(CONFIG, f, indent=2)


def round_tick(price: float, tick_size: float = 0.01) -> float:
    """Rounds price to nearest instrument tick increment."""
    return round(round(price / tick_size) * tick_size, 2)


def fetch_live_btcusdt_book(timeout_sec: float = 2.0) -> Optional[Dict[str, Any]]:
    """
    Live Binance BTCUSDT bookTicker (REST). Used for TP/SL/MTM — never the
    entry-bar decision-card quote, which goes stale within seconds.
    """
    url = "https://api.binance.com/api/v3/ticker/bookTicker?symbol=BTCUSDT"
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "rp608-trade-plan-overlay/1.0"})
        with urllib.request.urlopen(req, timeout=timeout_sec) as resp:
            data = json.loads(resp.read().decode("utf-8"))
        bid = float(data["bidPrice"])
        ask = float(data["askPrice"])
        if bid <= 0 or ask <= 0 or ask < bid:
            return None
        mid = (bid + ask) / 2.0
        now_ms = int(time.time() * 1000)
        return {
            "best_bid_price": bid,
            "best_ask_price": ask,
            "best_bid_qty": float(data.get("bidQty") or 0.0),
            "best_ask_qty": float(data.get("askQty") or 0.0),
            "spread_bps": ((ask - bid) / mid) * 10000.0 if mid > 0 else 0.0,
            "local_wallclock_utc_ms": now_ms,
            "quote_source": "binance_rest_bookTicker",
        }
    except Exception:
        return None


def normalize_book_quote(quote: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    """Accept daemon bookTicker or REST shapes; require valid bid/ask."""
    if not quote:
        return None
    try:
        bid = float(quote.get("best_bid_price") or quote.get("b") or 0.0)
        ask = float(quote.get("best_ask_price") or quote.get("a") or 0.0)
    except (TypeError, ValueError):
        return None
    if bid <= 0.0 or ask <= 0.0 or ask < bid:
        return None
    out = dict(quote)
    out["best_bid_price"] = bid
    out["best_ask_price"] = ask
    return out


def calculate_trade_plan(
    decision_card: Dict[str, Any],
    quote: Optional[Dict[str, Any]] = None,
    notional_usdt: float = 1000.0
) -> Optional[Dict[str, Any]]:
    """
    Formulates a deterministic Trade Plan from a Policy B decision card.
    """
    direction = decision_card.get("policy_b_action")
    if direction not in ("LONG", "SHORT"):
        return None

    atr = float(decision_card.get("atr_14", 0.0))
    ref_price = float(decision_card.get("reference_price", 0.0))
    if atr <= 0.0 or ref_price <= 0.0:
        return None

    # Use execution quote if provided, otherwise fallback to reference price
    exec_quote = quote or decision_card.get("execution_quote") or {}
    bid = float(exec_quote.get("best_bid_price", ref_price))
    ask = float(exec_quote.get("best_ask_price", ref_price))

    # Bid/Ask-aware entry
    if direction == "LONG":
        entry_price = round_tick(ask if ask > 0 else ref_price)
        stop_price = round_tick(entry_price - (CONFIG["stop_multiplier"] * atr))
        target_price = round_tick(entry_price + (CONFIG["target_multiplier"] * atr))
        invalidation_price = stop_price
    else:  # SHORT
        entry_price = round_tick(bid if bid > 0 else ref_price)
        stop_price = round_tick(entry_price + (CONFIG["stop_multiplier"] * atr))
        target_price = round_tick(entry_price - (CONFIG["target_multiplier"] * atr))
        invalidation_price = stop_price

    # Validate price monotonicity
    if direction == "LONG" and not (stop_price < entry_price < target_price):
        return None
    if direction == "SHORT" and not (target_price < entry_price < stop_price):
        return None

    # Simulated Sizing
    qty = round(notional_usdt / entry_price, 6)

    # Distances
    gross_risk_dist = abs(entry_price - stop_price)
    gross_reward_dist = abs(target_price - entry_price)
    gross_risk_usdt = gross_risk_dist * qty
    gross_reward_usdt = gross_reward_dist * qty
    gross_rr_ratio = round(gross_reward_dist / gross_risk_dist, 2) if gross_risk_dist > 0 else 0.0

    # Friction accounting: 6 bps entry + 6 bps exit = 12 bps total on notional
    entry_friction_usdt = round(notional_usdt * (6.0 / 10000.0), 4)
    exit_friction_target_usdt = round((target_price * qty) * (6.0 / 10000.0), 4)
    exit_friction_stop_usdt = round((stop_price * qty) * (6.0 / 10000.0), 4)

    total_target_friction_usdt = round(entry_friction_usdt + exit_friction_target_usdt, 4)
    total_stop_friction_usdt = round(entry_friction_usdt + exit_friction_stop_usdt, 4)

    net_target_profit_usdt = round(gross_reward_usdt - total_target_friction_usdt, 2)
    net_stop_loss_usdt = round(gross_risk_usdt + total_stop_friction_usdt, 2)
    net_rr_ratio = round(net_target_profit_usdt / net_stop_loss_usdt, 2) if net_stop_loss_usdt > 0 else 0.0

    # Timestamps
    signal_close_ms = int(decision_card.get("close_time_ms", time.time() * 1000))
    entry_time_ms = int(exec_quote.get("exchange_event_time_ms", signal_close_ms))
    expiry_time_ms = entry_time_ms + (CONFIG["max_holding_seconds"] * 1000)

    event_idx = decision_card.get("decision_event_idx", decision_card.get("bar_idx", 0))
    plan_id = f"PLAN-{int(event_idx):06d}-{direction}-{entry_time_ms}"

    # Order-flow confirm: prefer card stamp, else bookTicker qty, else tick features.
    # Latched at open — stays on the plan until TP/SL/TIME_EXIT (signal expiry).
    of: Dict[str, Any]
    if decision_card.get("order_flow_supported") is not None and (
        decision_card.get("order_flow_imbalance") is not None
        or decision_card.get("order_flow_label") is not None
        or decision_card.get("order_flow_supported") is False
    ):
        of = {
            "order_flow_supported": bool(decision_card.get("order_flow_supported")),
            "order_flow_label": decision_card.get("order_flow_label"),
            "order_flow_imbalance": decision_card.get("order_flow_imbalance"),
            "order_flow_source": str(decision_card.get("order_flow_source") or "decision_card"),
        }
    else:
        of = classify_order_flow_support(
            direction,
            l1_from_book_ticker(exec_quote if isinstance(exec_quote, dict) else None),
        )
        if not of["order_flow_supported"]:
            of = classify_order_flow_support(
                direction,
                l1_from_tick_features(
                    decision_card.get("features_tick")
                    if isinstance(decision_card.get("features_tick"), dict)
                    else None
                ),
            )

    return {
        "plan_id": plan_id,
        "protocol_id": CONFIG["protocol_id"],
        "config_hash": CONFIG_HASH,
        "signal_cadence": CONFIG.get("signal_cadence", "tick_book_ticker"),
        "source_bar_idx": decision_card.get("bar_idx"),
        "source_decision_event_idx": decision_card.get("decision_event_idx"),
        "source_decision_clock": decision_card.get("decision_clock", CONFIG.get("signal_cadence")),
        "source_timestamp_utc": decision_card.get("timestamp_utc"),
        "direction": direction,
        "reference_price": ref_price,
        "atr_14": atr,
        "entry_price": entry_price,
        "stop_loss_price": stop_price,
        "profit_target_price": target_price,
        "invalidation_price": invalidation_price,
        "simulated_quantity": qty,
        "simulated_notional_usdt": notional_usdt,
        "gross_risk_usdt": round(gross_risk_usdt, 2),
        "gross_reward_usdt": round(gross_reward_usdt, 2),
        "gross_rr_ratio": gross_rr_ratio,
        "round_trip_friction_bps": CONFIG["round_trip_friction_bps"],
        "est_friction_usdt": total_target_friction_usdt,
        "net_target_profit_usdt": net_target_profit_usdt,
        "net_stop_loss_usdt": net_stop_loss_usdt,
        "net_rr_ratio": net_rr_ratio,
        "max_holding_seconds": CONFIG["max_holding_seconds"],
        "entry_time_ms": entry_time_ms,
        "expiry_time_ms": expiry_time_ms,
        "state": "OPEN",
        "data_quality_status": "VALID",
        "order_authority": "0.0% (SIMULATED)",
        "horizon_class": "INTRADAY",
        "horizon_id": "H0_INTRADAY_60M",
        "horizon_label": "Intraday",
        "horizon_term": "≤60 minutes (trade-plan overlay)",
        "order_flow_supported": bool(of.get("order_flow_supported")),
        "order_flow_label": of.get("order_flow_label"),
        "order_flow_imbalance": of.get("order_flow_imbalance"),
        "order_flow_source": of.get("order_flow_source"),
        # OF mark at open; live OF recommendation lifecycle is exit-signal driven
        # (reverse flow), not the 60m plan horizon.
        "order_flow_latched_at_open": bool(of.get("order_flow_supported")),
    }


class TradePlanOverlayManager:
    """Manages lifecycle states, state persistence, and outcome recording."""

    def __init__(self, base_dir: Optional[Path] = None):
        self.base_dir = Path(base_dir) if base_dir else OVERLAY_DIR
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.state_file = self.base_dir / "overlay_state.json"
        self.events_file = self.base_dir / "trade_plan_events.jsonl"
        self.outcomes_file = self.base_dir / "trade_plan_outcomes.jsonl"
        self.of_outcomes_file = self.base_dir / "of_trade_outcomes.jsonl"
        self.of_opens_file = self.base_dir / "of_paper_trade_opens.jsonl"
        self.of_open_state_file = self.base_dir / "of_paper_trades_open.json"
        self.config_file = self.base_dir / "overlay_config.json"
        self._save_config_if_missing()
        self.state: Dict[str, Any] = self._load_state()
        if "of_outcomes_summary" not in self.state:
            self.state["of_outcomes_summary"] = {
                "n_trades": 0,
                "n_wins": 0,
                "n_losses": 0,
                "cumulative_gross_pnl_usdt": 0.0,
                "cumulative_net_pnl_usdt": 0.0,
                "cumulative_net_return_bps": 0.0,
                "cumulative_friction_usdt": 0.0,
                "cumulative_spread_cost_usdt": 0.0,
                "cumulative_fees_usdt": 0.0,
                "cumulative_slippage_usdt": 0.0,
            }
        elif "cumulative_net_return_bps" not in self.state["of_outcomes_summary"]:
            self.state["of_outcomes_summary"]["cumulative_net_return_bps"] = 0.0
        self._reconstruct_open_of_paper_trade()
        self.reconcile_of_paper_already_closed(persist=True)
        self._repair_last_closed_from_ledger(persist=True)
        # Entry-quality gates apply only at activate-time. Never scrub a booked
        # PAPER open on load (that orphaned OF ENTRY #664 / painted OF EXIT #—).
        self._restore_scrubbed_open_from_journal(persist=True)

    def _save_config_if_missing(self):
        if not self.config_file.exists():
            with open(self.config_file, "w", encoding="utf-8") as f:
                json.dump(CONFIG, f, indent=2)

    def _load_state(self) -> Dict[str, Any]:
        if self.state_file.exists():
            try:
                with open(self.state_file, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                pass
        return {
            "schema_version": "1.0.0",
            "active_plan": None,
            "active_order_flow_recommendation": None,
            "total_plans_created": 0,
            "closed_plans_count": 0,
            "suppressed_signals_count": 0,
            "outcomes_summary": {
                "tp_hits": 0,
                "sl_hits": 0,
                "time_exits": 0,
                "ambiguous_hits": 0,
                "cumulative_net_pnl_usdt": 0.0
            },
            "of_outcomes_summary": {
                "n_trades": 0,
                "n_wins": 0,
                "n_losses": 0,
                "cumulative_gross_pnl_usdt": 0.0,
                "cumulative_net_pnl_usdt": 0.0,
                "cumulative_friction_usdt": 0.0,
            },
        }

    def _quote_bid_ask(
        self, quote: Optional[Dict[str, Any]]
    ) -> Tuple[Optional[float], Optional[float]]:
        q = quote if isinstance(quote, dict) else {}
        bid = q.get("best_bid_price", q.get("bid"))
        ask = q.get("best_ask_price", q.get("ask"))
        try:
            bid_f = float(bid) if bid is not None else None
        except (TypeError, ValueError):
            bid_f = None
        try:
            ask_f = float(ask) if ask is not None else None
        except (TypeError, ValueError):
            ask_f = None
        return bid_f, ask_f

    def _of_entry_exec_price(self, direction: str, quote: Optional[Dict[str, Any]], fallback: Optional[float]) -> Optional[float]:
        """Shadow OF entry: pay ask for LONG, hit bid for SHORT (same as plan overlay)."""
        bid, ask = self._quote_bid_ask(quote)
        d = str(direction or "").upper()
        if d == "LONG" and ask is not None:
            return ask
        if d == "SHORT" and bid is not None:
            return bid
        try:
            return float(fallback) if fallback is not None else None
        except (TypeError, ValueError):
            return None

    def _of_exit_exec_price(self, direction: str, quote: Optional[Dict[str, Any]], fallback: Optional[float]) -> Optional[float]:
        """Shadow OF exit: sell bid for LONG, cover ask for SHORT."""
        bid, ask = self._quote_bid_ask(quote)
        d = str(direction or "").upper()
        if d == "LONG" and bid is not None:
            return bid
        if d == "SHORT" and ask is not None:
            return ask
        try:
            return float(fallback) if fallback is not None else None
        except (TypeError, ValueError):
            return None

    def _compute_of_trade_pnl(
        self,
        *,
        direction: str,
        entry_price: float,
        exit_price: float,
        notional: float,
        entry_bid: Optional[float] = None,
        entry_ask: Optional[float] = None,
        exit_bid: Optional[float] = None,
        exit_ask: Optional[float] = None,
        quantity: Optional[float] = None,
    ) -> Dict[str, Any]:
        """PAPER OF P&L with explicit spread / fees / slippage (RP-608-OF-PAPER-TRADE-001)."""
        return compute_of_paper_cost_breakdown(
            direction=direction,
            entry_price=entry_price,
            exit_price=exit_price,
            notional=notional,
            entry_bid=entry_bid,
            entry_ask=entry_ask,
            exit_bid=exit_bid,
            exit_ask=exit_ask,
            fee_bps_per_leg=6.0,
            quantity=quantity,
        )

    def _read_of_open_state_file(self) -> Optional[Dict[str, Any]]:
        if not self.of_open_state_file.exists():
            return None
        try:
            data = json.loads(self.of_open_state_file.read_text(encoding="utf-8"))
            return data if isinstance(data, dict) and data.get("of_trade_id") else None
        except Exception:
            return None

    def _write_of_open_state_file(self, rec: Optional[Dict[str, Any]]) -> None:
        # Unique tmp — shared *.tmp races between dashboard/daemon caused FileNotFoundError
        tmp = self.of_open_state_file.with_name(
            f"{self.of_open_state_file.stem}.{os.getpid()}.{time.time_ns()}.tmp"
        )
        if rec is None:
            payload: Dict[str, Any] = {
                "open": False,
                "of_trade_id": None,
                "protocol_id": OF_PAPER_PROTOCOL_ID,
                "execution_mode": "PAPER",
                "live_order_authority": 0.0,
            }
        else:
            payload = {
                "open": True,
                "protocol_id": OF_PAPER_PROTOCOL_ID,
                "execution_mode": "PAPER",
                "live_order_authority": 0.0,
                "of_trade_id": rec.get("of_trade_id"),
                "trade": rec,
            }
        try:
            with open(tmp, "w", encoding="utf-8") as f:
                json.dump(payload, f, indent=2)
                f.flush()
                os.fsync(f.fileno())
            os.replace(str(tmp), str(self.of_open_state_file))
        finally:
            if tmp.exists():
                try:
                    tmp.unlink()
                except OSError:
                    pass

    def _log_of_paper_open(self, rec: Dict[str, Any]) -> None:
        # Idempotent on of_trade_id — dual writers must not spam opens.jsonl
        tid = rec.get("of_trade_id")
        if tid and self.of_opens_file.exists():
            try:
                with open(self.of_opens_file, "r", encoding="utf-8") as f:
                    for line in f:
                        try:
                            prev = json.loads(line)
                        except Exception:
                            continue
                        if str(prev.get("of_trade_id") or "") == str(tid):
                            return
            except Exception:
                pass
        row = {
            "event": "OF_PAPER_OPEN",
            "protocol_id": OF_PAPER_PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "of_trade_id": rec.get("of_trade_id"),
            "symbol": rec.get("symbol"),
            "direction": rec.get("direction"),
            "entry_price": rec.get("entry_price"),
            "entry_bid": rec.get("entry_bid"),
            "entry_ask": rec.get("entry_ask"),
            "simulated_notional_usdt": rec.get("simulated_notional_usdt"),
            "simulated_quantity": rec.get("simulated_quantity"),
            "entry_signal_qty": rec.get("entry_signal_qty"),
            "participation_pct": rec.get("participation_pct"),
            "sizing_source": rec.get("sizing_source"),
            "depth_quality": rec.get("depth_quality"),
            "activated_ms": rec.get("activated_ms"),
            "paper_open_timestamp": rec.get("paper_open_timestamp"),
            "plan_id": rec.get("plan_id"),
            "entry_candle": rec.get("entry_candle"),
            "order_flow_imbalance": rec.get("order_flow_imbalance"),
            "order_flow_source": rec.get("order_flow_source"),
            "independent_of_trade_plan_ttl": True,
        }
        with open(self.of_opens_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(row) + "\n")

    def _of_outcome_row_by_id(self, tid: str) -> Optional[Dict[str, Any]]:
        """Load durable closed outcome for of_trade_id (last matching row)."""
        if not tid or not self.of_outcomes_file.exists():
            return None
        found: Optional[Dict[str, Any]] = None
        try:
            with open(self.of_outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except Exception:
                        continue
                    if str(row.get("of_trade_id") or "") == str(tid):
                        found = row
        except Exception:
            return None
        return found

    def reconcile_of_paper_already_closed(self, *, persist: bool = True) -> bool:
        """
        Drop ghost OPEN paper trades whose of_trade_id is already in the closed ledger.

        Dual writers / stale overlay_state can resurrect an id after the first close;
        ledger is source of truth for closed ids (OPEN must never mirror a booked row).
        Hydrate last_closed from the outcomes row so UI keeps exit imb / candles.
        Returns True if a ghost was dropped.
        """
        rec = self.state.get("active_order_flow_recommendation")
        if not isinstance(rec, dict) or not rec.get("active"):
            # Also scrub disk open file if it points at a closed id
            disk = self._read_of_open_state_file()
            disk_trade = (disk or {}).get("trade") if disk and disk.get("open") else None
            tid_disk = (disk_trade or {}).get("of_trade_id") if disk_trade else None
            if tid_disk and str(tid_disk) in self._known_of_outcome_ids():
                ledger = self._of_outcome_row_by_id(str(tid_disk))
                if ledger:
                    closed = dict(ledger)
                    closed["active"] = False
                    closed["execution_mode"] = "PAPER"
                    closed["live_order_authority"] = 0.0
                    closed["protocol_id"] = OF_PAPER_PROTOCOL_ID
                    self.state["last_closed_order_flow_recommendation"] = closed
                    if persist:
                        self._save_state()
                self._write_of_open_state_file(None)
                return True
            return False
        tid = rec.get("of_trade_id")
        if not tid or str(tid) not in self._known_of_outcome_ids():
            return False
        ledger = self._of_outcome_row_by_id(str(tid))
        if ledger:
            closed = dict(ledger)
            closed["active"] = False
        else:
            closed = dict(rec)
            closed["active"] = False
            closed["exit_reason"] = closed.get("exit_reason") or "RECONCILED_ALREADY_CLOSED"
        closed["execution_mode"] = "PAPER"
        closed["live_order_authority"] = 0.0
        closed["protocol_id"] = OF_PAPER_PROTOCOL_ID
        self.state["last_closed_order_flow_recommendation"] = closed
        self.state["active_order_flow_recommendation"] = None
        self._write_of_open_state_file(None)
        if persist:
            self._save_state()
        return True

    def _repair_last_closed_from_ledger(self, *, persist: bool = True) -> bool:
        """Always replace incomplete RECONCILED stubs with durable outcome rows."""
        last = self.state.get("last_closed_order_flow_recommendation")
        needs = False
        if not isinstance(last, dict):
            needs = True
            last = {}
        elif (
            last.get("exit_reason") == "RECONCILED_ALREADY_CLOSED"
            or not last.get("exit_candle")
            or last.get("exit_imbalance") is None
            or last.get("exit_price") is None
        ):
            needs = True
        if not needs:
            return False
        tid_last = last.get("of_trade_id")
        ledger = self._of_outcome_row_by_id(str(tid_last)) if tid_last else None
        if ledger is None:
            recent = self.load_recent_of_outcomes(1)
            ledger = recent[-1] if recent else None
        if not ledger:
            return False
        closed = dict(ledger)
        closed["active"] = False
        closed["execution_mode"] = "PAPER"
        closed["live_order_authority"] = 0.0
        closed["protocol_id"] = OF_PAPER_PROTOCOL_ID
        self.state["last_closed_order_flow_recommendation"] = closed
        if persist:
            self._save_state()
        return True

    def _of_open_row_by_id(self, tid: str) -> Optional[Dict[str, Any]]:
        """Last OF_PAPER_OPEN journal row for of_trade_id."""
        if not tid or not self.of_opens_file.exists():
            return None
        found: Optional[Dict[str, Any]] = None
        try:
            with open(self.of_opens_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except Exception:
                        continue
                    if str(row.get("of_trade_id") or "") == str(tid):
                        found = row
        except Exception:
            return None
        return found

    def _restore_scrubbed_open_from_journal(self, *, persist: bool = True) -> bool:
        """
        Heal opens dropped by legacy scrub_on_load_* (entry-quality misapplied post-book).
        Restores from opens journal when no outcome exists — no new of_trade_id.
        """
        active = self.state.get("active_order_flow_recommendation")
        if isinstance(active, dict) and active.get("active") and active.get("of_trade_id"):
            return False
        supp = self.state.get("last_of_paper_open_suppressed")
        if not isinstance(supp, dict):
            return False
        reason = str(supp.get("reason") or "")
        if not reason.startswith("scrub_on_load_"):
            return False
        tid = supp.get("of_trade_id")
        if not tid or str(tid) in self._known_of_outcome_ids():
            return False
        row = self._of_open_row_by_id(str(tid))
        if not row:
            return False
        act_ms = row.get("activated_ms")
        try:
            act_ms_i = int(act_ms) if act_ms is not None else None
        except (TypeError, ValueError):
            act_ms_i = None
        # Recover bar/event from of_trade_id PLAN-0NNNNN or decision cards by time
        evt_from_id: Optional[int] = None
        try:
            import re as _re

            m = _re.search(r"PLAN-0*(\d+)-", str(tid))
            if m:
                evt_from_id = int(m.group(1))
        except Exception:
            evt_from_id = None
        card_hit: Optional[Dict[str, Any]] = None
        if DECISION_CARDS_FILE.exists() and (act_ms_i is not None or evt_from_id is not None):
            try:
                with open(DECISION_CARDS_FILE, "r", encoding="utf-8") as f:
                    for line in f:
                        line = line.strip()
                        if not line:
                            continue
                        try:
                            c = json.loads(line)
                        except Exception:
                            continue
                        if evt_from_id is not None and c.get("decision_event_idx") == evt_from_id:
                            card_hit = c
                            break
                        if (
                            act_ms_i is not None
                            and c.get("close_time_ms") is not None
                            and abs(int(c["close_time_ms"]) - act_ms_i) <= 5
                        ):
                            card_hit = c
            except Exception:
                card_hit = None
        entry_candle = row.get("entry_candle")
        if not isinstance(entry_candle, dict) or entry_candle.get("bar_idx") is None:
            if card_hit is not None:
                entry_candle = build_of_candle_mark(
                    now_ms=act_ms_i or int(card_hit.get("close_time_ms") or 0),
                    card=card_hit,
                )
            elif act_ms_i is not None:
                entry_candle = build_of_candle_mark(
                    now_ms=act_ms_i,
                    decision_event_idx=evt_from_id,
                )
            else:
                entry_candle = entry_candle if isinstance(entry_candle, dict) else {}
        restored = {
            "active": True,
            "symbol": row.get("symbol") or "BTCUSDT",
            "direction": row.get("direction"),
            "order_flow_label": ORDER_FLOW_LABEL,
            "order_flow_supported": True,
            "order_flow_imbalance": supp.get("order_flow_imbalance") or row.get("order_flow_imbalance"),
            "order_flow_source": row.get("order_flow_source") or "restored_after_scrub",
            "plan_id": row.get("plan_id"),
            "of_trade_id": tid,
            "entry_price": row.get("entry_price"),
            "entry_bid": row.get("entry_bid"),
            "entry_ask": row.get("entry_ask"),
            "entry_signal_qty": row.get("entry_signal_qty"),
            "participation_pct": row.get("participation_pct"),
            "sizing_source": row.get("sizing_source"),
            "depth_quality": row.get("depth_quality") or supp.get("depth_quality"),
            "simulated_notional_usdt": row.get("simulated_notional_usdt"),
            "simulated_quantity": row.get("simulated_quantity"),
            "entry_paper_quantity": row.get("simulated_quantity"),
            "entry_time_ms": act_ms_i,
            "activated_ms": act_ms_i,
            "paper_open_timestamp": row.get("paper_open_timestamp"),
            "entry_candle": entry_candle,
            "exit_rule": (
                "opposite_side_order_flow_confirm"
                f"+min_hold_{int(CONFIG.get('of_paper_min_hold_ms', 300_000))}ms"
                f"+persist_{int(CONFIG.get('of_paper_opposite_persist_ms', 30_000))}ms"
            ),
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "protocol_id": OF_PAPER_PROTOCOL_ID,
            "independent_of_trade_plan_ttl": True,
            "stop_loss_price": None,
            "profit_target_price": None,
            "expiry_time_ms": None,
            "opposite_exit_seen_ms": None,
            "restored_after_scrub_on_load": True,
            "note": (
                "PAPER OF trade restored after legacy scrub_on_load — "
                "entry quality is activate-time only; exit still reverse-OF."
            ),
        }
        self.state["active_order_flow_recommendation"] = restored
        self.state.pop("last_of_paper_open_suppressed", None)
        bar_idx = (entry_candle or {}).get("bar_idx")
        if bar_idx is not None:
            self.state["last_of_paper_entry_bar_idx"] = int(bar_idx)
        self._write_of_open_state_file(restored)
        if persist:
            self._save_state()
        return True

    def _reconstruct_open_of_paper_trade(self) -> None:
        """Restart-safe: restore at most one open OF paper trade; never mint a new id."""
        active = self.state.get("active_order_flow_recommendation")
        disk = self._read_of_open_state_file()
        disk_trade = (disk or {}).get("trade") if disk and disk.get("open") else None
        known = self._known_of_outcome_ids()

        if active and active.get("active") and active.get("of_trade_id"):
            if str(active.get("of_trade_id")) in known:
                # Closed in ledger — do not keep / re-persist as open
                self.state["active_order_flow_recommendation"] = None
                self._write_of_open_state_file(None)
                return
            self._write_of_open_state_file(active)
            return

        if disk_trade and disk_trade.get("active") and disk_trade.get("of_trade_id"):
            if str(disk_trade.get("of_trade_id")) in known:
                self._write_of_open_state_file(None)
                return
            restored = dict(disk_trade)
            restored["active"] = True
            restored["execution_mode"] = "PAPER"
            restored["live_order_authority"] = 0.0
            restored["protocol_id"] = OF_PAPER_PROTOCOL_ID
            restored["independent_of_trade_plan_ttl"] = True
            restored["reconstructed_from_open_journal"] = True
            self.state["active_order_flow_recommendation"] = restored
            self._save_state()
            return

        if not active or not active.get("active"):
            self._write_of_open_state_file(None)

    def build_of_shadow_trade_snapshot(
        self,
        quote: Optional[Dict[str, Any]] = None,
        *,
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """Open OF paper trade MTM for /api/state (null if none)."""
        rec = self.state.get("active_order_flow_recommendation")
        if not rec or not rec.get("active"):
            return None
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        direction = str(rec.get("direction") or "")
        try:
            entry_f = float(rec["entry_price"]) if rec.get("entry_price") is not None else None
        except (TypeError, ValueError):
            entry_f = None
        notional = float(rec.get("simulated_notional_usdt") or CONFIG.get("of_paper_fallback_notional_usdt", 100.0))
        try:
            locked_qty = float(rec["simulated_quantity"]) if rec.get("simulated_quantity") is not None else None
        except (TypeError, ValueError):
            locked_qty = None
        bid, ask = self._quote_bid_ask(quote)
        mark = self._of_exit_exec_price(direction, quote, entry_f)
        hold_s = None
        act_ms = rec.get("activated_ms") or rec.get("entry_time_ms")
        if act_ms is not None:
            try:
                hold_s = round((now - int(act_ms)) / 1000.0, 3)
            except (TypeError, ValueError):
                hold_s = None
        pnl: Dict[str, Any] = {}
        if entry_f is not None and mark is not None and direction in ("LONG", "SHORT"):
            pnl = self._compute_of_trade_pnl(
                direction=direction,
                entry_price=entry_f,
                exit_price=float(mark),
                notional=notional,
                entry_bid=rec.get("entry_bid"),
                entry_ask=rec.get("entry_ask"),
                exit_bid=bid,
                exit_ask=ask,
                quantity=locked_qty,
            )
        return {
            "open": True,
            "protocol_id": OF_PAPER_PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "of_trade_id": rec.get("of_trade_id"),
            "symbol": rec.get("symbol") or "BTCUSDT",
            "direction": direction,
            "entry_price": entry_f,
            "entry_bid": rec.get("entry_bid"),
            "entry_ask": rec.get("entry_ask"),
            "simulated_quantity": locked_qty if locked_qty is not None else pnl.get("simulated_quantity"),
            "simulated_notional_usdt": notional,
            "entry_signal_qty": rec.get("entry_signal_qty"),
            "participation_pct": rec.get("participation_pct"),
            "sizing_source": rec.get("sizing_source"),
            "depth_quality": rec.get("depth_quality"),
            "activated_ms": act_ms,
            "paper_open_timestamp": rec.get("paper_open_timestamp")
            or (_iso_utc_ms(int(act_ms)) if act_ms is not None else None),
            "mark_price": mark,
            "mark_bid": bid,
            "mark_ask": ask,
            "holding_duration_seconds": hold_s,
            "unrealized_gross_pnl_usdt": pnl.get("gross_pnl_usdt"),
            "unrealized_net_pnl_usdt": pnl.get("net_pnl_usdt"),
            "unrealized_net_return_bps": pnl.get("net_return_bps"),
            "unrealized_gross_return_bps": (
                round((float(pnl["gross_pnl_usdt"]) / notional) * 10000.0, 2)
                if pnl.get("gross_pnl_usdt") is not None and notional > 0
                else None
            ),
            "spread_cost_usdt": pnl.get("spread_cost_usdt"),
            "fees_usdt": pnl.get("fees_usdt"),
            "slippage_usdt": pnl.get("slippage_usdt"),
            "slippage_model": pnl.get("slippage_model", "none"),
            "friction_usdt": pnl.get("friction_usdt"),
            "exit_rule": "opposite_side_order_flow_confirm",
            "independent_of_trade_plan_ttl": True,
            "plan_id": rec.get("plan_id"),
            "order_flow_imbalance": rec.get("order_flow_imbalance"),
            "note": (
                "PAPER OF trade — reverse OF closes; TP/SL/60m do not. "
                "uGross=mark mid P&L; uNet=gross−spread−fees (projected RT)."
            ),
        }

    def _of_paper_entry_quality_ok(
        self,
        *,
        size: Dict[str, Any],
        depth_q: Dict[str, Any],
    ) -> Tuple[bool, str]:
        """
        PAPER-only open filter. Does not change OF ±0.30 classification —
        only whether a paper trade is booked against a confirmed OF latch.
        """
        if CONFIG.get("of_paper_reject_thin_opposite", True) and depth_q.get(
            "thin_opposite_diagnostic"
        ):
            return False, "thin_opposite_diagnostic"
        min_sig = float(CONFIG.get("of_paper_min_entry_signal_qty", 0.0) or 0.0)
        try:
            sig = float(size.get("entry_signal_qty") or 0.0)
        except (TypeError, ValueError):
            sig = 0.0
        if min_sig > 0.0 and sig < min_sig:
            return False, f"entry_signal_qty_{sig}<{min_sig}"
        min_same = float(CONFIG.get("of_paper_min_same_side_notional_usdt", 0.0) or 0.0)
        try:
            same_n = float(depth_q.get("same_side_touch_notional_usdt") or 0.0)
        except (TypeError, ValueError):
            same_n = 0.0
        if min_same > 0.0 and same_n < min_same:
            return False, f"same_side_notional_{same_n}<{min_same}"
        return True, "ok"

    def _activate_order_flow_recommendation(
        self,
        *,
        direction: str,
        of: Dict[str, Any],
        plan_id: Optional[str] = None,
        entry_price: Optional[float] = None,
        entry_time_ms: Optional[int] = None,
        now_ms: Optional[int] = None,
        candle_mark: Optional[Dict[str, Any]] = None,
        quote: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Start OF recommendation as a PAPER trade; stays until reverse-flow exit."""
        self.reconcile_of_paper_already_closed(persist=True)
        existing = self.state.get("active_order_flow_recommendation")
        if existing and existing.get("active") and existing.get("of_trade_id"):
            if str(existing.get("direction")) == str(direction).upper():
                existing["order_flow_imbalance"] = of.get("order_flow_imbalance")
                existing["order_flow_source"] = of.get("order_flow_source")
                existing["last_of_eval_ms"] = int(now_ms if now_ms is not None else time.time() * 1000)
                if plan_id:
                    existing["plan_id"] = plan_id
                self.state["active_order_flow_recommendation"] = existing
                self._write_of_open_state_file(existing)
                return existing

        now = int(now_ms if now_ms is not None else time.time() * 1000)
        entry_candle = candle_mark or build_of_candle_mark(now_ms=now)

        # Anti-churn: at most one OF paper open per 15m bar
        if CONFIG.get("of_paper_one_per_15m_bar", True):
            bar_idx = (entry_candle or {}).get("bar_idx")
            last_bar = self.state.get("last_of_paper_entry_bar_idx")
            if bar_idx is not None and last_bar is not None and int(bar_idx) == int(last_bar):
                return existing if isinstance(existing, dict) else None

        bid, ask = self._quote_bid_ask(quote)
        exec_entry = self._of_entry_exec_price(direction, quote, entry_price)
        size = size_of_paper_from_entry_signal(
            direction=direction,
            entry_price=float(exec_entry if exec_entry is not None else (entry_price or 0.0)),
            quote=quote if isinstance(quote, dict) else None,
        )
        depth_q = compute_of_depth_quality(
            direction=direction,
            quote=quote if isinstance(quote, dict) else None,
            imbalance=of.get("order_flow_imbalance"),
            phase="entry",
        )
        ok_q, deny_q = self._of_paper_entry_quality_ok(size=size, depth_q=depth_q)
        if not ok_q:
            self.state["last_of_paper_open_suppressed"] = {
                "reason": deny_q,
                "direction": str(direction).upper(),
                "order_flow_imbalance": of.get("order_flow_imbalance"),
                "entry_signal_qty": size.get("entry_signal_qty"),
                "depth_quality": depth_q,
                "suppressed_ms": now,
                "protocol_id": OF_PAPER_PROTOCOL_ID,
                "note": "PAPER open suppressed — OF ±0.30 mark unchanged; no paper trade booked",
            }
            return existing if isinstance(existing, dict) else None

        notional = float(size["simulated_notional_usdt"])
        qty = float(size["simulated_quantity"])
        # Stamp entry-candle OF signal qty (source of paper size; exit must match paper qty)
        entry_candle = dict(entry_candle or {})
        entry_candle["signal_qty"] = size.get("entry_signal_qty")
        entry_candle["signal_side"] = "bid" if str(direction).upper() == "LONG" else "ask"
        entry_candle["paper_quantity"] = qty
        entry_candle["participation_pct"] = size.get("participation_pct")
        entry_candle["depth_quality"] = depth_q
        fill_ms = int(entry_time_ms) if entry_time_ms is not None else now
        of_trade_id = f"OF-PAPER-{(plan_id or 'NOPLAN')}-{fill_ms}"
        paper_open_ts = _iso_utc_ms(fill_ms)
        rec = {
            "active": True,
            "symbol": "BTCUSDT",
            "direction": direction,
            "order_flow_label": ORDER_FLOW_LABEL,
            "order_flow_supported": True,
            "order_flow_imbalance": of.get("order_flow_imbalance"),
            "order_flow_source": of.get("order_flow_source"),
            "plan_id": plan_id,
            "of_trade_id": of_trade_id,
            "entry_price": exec_entry if exec_entry is not None else entry_price,
            "entry_bid": bid,
            "entry_ask": ask,
            "entry_bid_qty": size.get("entry_bid_qty"),
            "entry_ask_qty": size.get("entry_ask_qty"),
            "entry_signal_qty": size.get("entry_signal_qty"),
            "participation_pct": size.get("participation_pct"),
            "participation_pct_min": size.get("participation_pct_min"),
            "participation_pct_max": size.get("participation_pct_max"),
            "sizing_source": size.get("sizing_source"),
            "depth_quality": depth_q,
            "entry_quote_source": (quote or {}).get("quote_source") if isinstance(quote, dict) else None,
            "simulated_notional_usdt": notional,
            "simulated_quantity": qty,
            "entry_paper_quantity": qty,  # immutable entry size; exit fill must equal this
            "entry_time_ms": fill_ms,
            "activated_ms": fill_ms,
            "paper_open_timestamp": paper_open_ts,
            "entry_candle": entry_candle,
            "exit_rule": (
                "opposite_side_order_flow_confirm"
                f"+min_hold_{int(CONFIG.get('of_paper_min_hold_ms', 300_000))}ms"
                f"+persist_{int(CONFIG.get('of_paper_opposite_persist_ms', 30_000))}ms"
            ),
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "protocol_id": OF_PAPER_PROTOCOL_ID,
            "independent_of_trade_plan_ttl": True,
            "stop_loss_price": None,
            "profit_target_price": None,
            "expiry_time_ms": None,
            "opposite_exit_seen_ms": None,
            "note": (
                "PAPER OF trade (RP-608-OF-PAPER-TRADE-001). Closes on sustained reverse OF "
                "after min hold — not on L1 flicker. Not tied to 60m plan TP/SL. 0% authority."
            ),
        }
        self.state["active_order_flow_recommendation"] = rec
        self.state.pop("last_of_paper_open_suppressed", None)
        bar_idx = (entry_candle or {}).get("bar_idx")
        if bar_idx is not None:
            self.state["last_of_paper_entry_bar_idx"] = int(bar_idx)
        self._log_of_paper_open(rec)
        self._write_of_open_state_file(rec)
        return rec

    def _of_paper_close_allowed(
        self,
        prev: Dict[str, Any],
        *,
        now_ms: int,
        exit_meta: Optional[Dict[str, Any]] = None,
    ) -> Tuple[bool, str]:
        """
        Anti-churn gate for PAPER closes only.

        Requires min hold since paper open AND opposite-OF signal sustained
        for of_paper_opposite_persist_ms. Prevents fee-dominated micro flips.
        """
        min_hold = int(CONFIG.get("of_paper_min_hold_ms", 300_000))
        persist = int(CONFIG.get("of_paper_opposite_persist_ms", 30_000))
        act = prev.get("activated_ms") or prev.get("entry_time_ms")
        try:
            act_i = int(act) if act is not None else None
        except (TypeError, ValueError):
            act_i = None
        if act_i is not None:
            held = now_ms - act_i
            if held < 0:
                # Clock skew / test injection — treat as zero hold
                held = 0
            if held < min_hold:
                return False, f"min_hold_not_met_{held}ms<{min_hold}ms"

        # Track continuous opposite-OF observation window on the open rec
        if exit_meta and exit_meta.get("exit"):
            seen = prev.get("opposite_exit_seen_ms")
            if seen is None:
                prev["opposite_exit_seen_ms"] = now_ms
                self.state["active_order_flow_recommendation"] = prev
                return False, f"opposite_persist_arming_need_{persist}ms"
            try:
                seen_i = int(seen)
            except (TypeError, ValueError):
                prev["opposite_exit_seen_ms"] = now_ms
                self.state["active_order_flow_recommendation"] = prev
                return False, "opposite_persist_rearm"
            if (now_ms - seen_i) < persist:
                return False, f"opposite_persist_{(now_ms - seen_i)}ms<{persist}ms"
            return True, "ok"

        # Non-opposite clear reasons (should be rare): still require min hold
        return True, "ok"

    def _clear_order_flow_recommendation(
        self,
        *,
        reason: str,
        exit_meta: Optional[Dict[str, Any]] = None,
        now_ms: Optional[int] = None,
        candle_mark: Optional[Dict[str, Any]] = None,
        quote: Optional[Dict[str, Any]] = None,
    ) -> None:
        prev = self.state.get("active_order_flow_recommendation")
        if not prev:
            return
        # Already booked → drop ghost without re-journaling / EXIT spam
        tid_prev = prev.get("of_trade_id")
        if tid_prev and str(tid_prev) in self._known_of_outcome_ids():
            closed_ghost = dict(prev)
            closed_ghost["active"] = False
            closed_ghost["exit_reason"] = closed_ghost.get("exit_reason") or "RECONCILED_ALREADY_CLOSED"
            self.state["last_closed_order_flow_recommendation"] = closed_ghost
            self.state["active_order_flow_recommendation"] = None
            self._write_of_open_state_file(None)
            return
        signal_exit_ms = int(now_ms if now_ms is not None else time.time() * 1000)
        allowed, deny = self._of_paper_close_allowed(
            prev, now_ms=signal_exit_ms, exit_meta=exit_meta
        )
        if not allowed:
            prev = dict(prev)
            prev["close_suppressed_reason"] = deny
            prev["close_suppressed_ms"] = signal_exit_ms
            if exit_meta:
                prev["pending_exit_imbalance"] = exit_meta.get("order_flow_imbalance")
                prev["pending_exit_opposite_side"] = exit_meta.get("opposite_side")
            self.state["active_order_flow_recommendation"] = prev
            self._write_of_open_state_file(prev)
            return

        paper_close_ms = signal_exit_ms
        paper_close_lag_ms = paper_close_ms - signal_exit_ms
        signal_exit_ts = _iso_utc_ms(signal_exit_ms)
        paper_close_ts = _iso_utc_ms(paper_close_ms)

        closed = dict(prev)
        closed["active"] = False
        closed["exited_ms"] = paper_close_ms
        closed["signal_exit_ms"] = signal_exit_ms
        closed["signal_exit_timestamp"] = signal_exit_ts
        closed["paper_close_ms"] = paper_close_ms
        closed["paper_close_timestamp"] = paper_close_ts
        closed["paper_close_lag_ms"] = paper_close_lag_ms
        closed["exit_reason"] = reason
        closed["order_flow_exit_label"] = ORDER_FLOW_EXIT_LABEL
        closed["execution_mode"] = "PAPER"
        closed["live_order_authority"] = 0.0
        closed["protocol_id"] = OF_PAPER_PROTOCOL_ID
        # Exit mark identity must be the EXIT event — never reuse entry bar_idx
        # (that produced OF EXIT #— / wrong-bar badges on the next 15m window).
        if isinstance(candle_mark, dict) and (
            candle_mark.get("bar_idx") is not None
            or candle_mark.get("decision_event_idx") is not None
            or candle_mark.get("mark_time_ms") is not None
        ):
            exit_candle = dict(candle_mark)
        else:
            exit_candle = build_of_candle_mark(now_ms=signal_exit_ms)
        closed["exit_candle"] = exit_candle
        if exit_meta:
            closed["exit_imbalance"] = exit_meta.get("order_flow_imbalance")
            closed["exit_opposite_side"] = exit_meta.get("opposite_side")

        direction = str(closed.get("direction") or "")
        entry_price = closed.get("entry_price")
        try:
            entry_f = float(entry_price) if entry_price is not None else None
        except (TypeError, ValueError):
            entry_f = None
        bid, ask = self._quote_bid_ask(quote)
        exit_px = self._of_exit_exec_price(direction, quote, entry_f)
        # Exit-book OF signal qty (observability only — never sizes the close)
        exit_sig_qty = of_exit_signal_qty(direction, quote if isinstance(quote, dict) else None)
        exit_bid_qty, exit_ask_qty = _l1_book_qtys(quote if isinstance(quote, dict) else None)
        exit_depth = compute_of_depth_quality(
            direction=direction,
            quote=quote if isinstance(quote, dict) else None,
            imbalance=(exit_meta or {}).get("order_flow_imbalance") if exit_meta else closed.get("exit_imbalance"),
            phase="exit",
        )
        # Persistence diagnostic (telemetry): hold length + opposite-OF arm duration
        opp_seen = prev.get("opposite_exit_seen_ms")
        persist_ms = None
        if opp_seen is not None:
            try:
                persist_ms = max(0, int(signal_exit_ms) - int(opp_seen))
            except (TypeError, ValueError):
                persist_ms = None
        exit_depth["signal_persistence_hold_seconds"] = None  # filled below once hold_s known
        exit_depth["opposite_persist_ms_at_exit"] = persist_ms
        closed["exit_bid"] = bid
        closed["exit_ask"] = ask
        closed["exit_price"] = exit_px
        closed["exit_signal_qty"] = round(exit_sig_qty, 8)
        closed["exit_bid_qty"] = round(exit_bid_qty, 8)
        closed["exit_ask_qty"] = round(exit_ask_qty, 8)
        closed["exit_depth_quality"] = exit_depth
        closed["exit_quote_source"] = (quote or {}).get("quote_source") if isinstance(quote, dict) else None
        if isinstance(exit_candle, dict):
            exit_candle = dict(exit_candle)
            exit_candle["signal_qty"] = round(exit_sig_qty, 8)
            exit_candle["signal_side"] = "ask" if str(direction).upper() == "LONG" else "bid"
            exit_candle["depth_quality"] = exit_depth
            closed["exit_candle"] = exit_candle
        hold_s = None
        act_ms = closed.get("activated_ms") or closed.get("entry_time_ms")
        if act_ms is not None:
            try:
                hold_s = round((paper_close_ms - int(act_ms)) / 1000.0, 3)
            except (TypeError, ValueError):
                hold_s = None
        closed["holding_duration_seconds"] = hold_s
        if isinstance(exit_depth, dict):
            exit_depth["signal_persistence_hold_seconds"] = hold_s
            closed["exit_depth_quality"] = exit_depth

        pnl: Dict[str, Any] = {}
        notional = float(
            closed.get("simulated_notional_usdt")
            or CONFIG.get("of_paper_fallback_notional_usdt", 100.0)
        )
        # Exit fill qty = entry paper qty (from entry OF candle signal). Never exit-book sized.
        locked_qty = None
        for key in ("entry_paper_quantity", "simulated_quantity"):
            if closed.get(key) is not None:
                try:
                    locked_qty = float(closed[key])
                    break
                except (TypeError, ValueError):
                    continue
        if locked_qty is None and closed.get("entry_signal_qty") is not None and entry_f:
            # Recover from stamped entry signal if paper qty missing (legacy rows)
            try:
                pct = float(closed.get("participation_pct") or CONFIG.get("of_paper_participation_pct", 0.10))
                pct = max(
                    float(CONFIG.get("of_paper_participation_pct_min", 0.05)),
                    min(float(CONFIG.get("of_paper_participation_pct_max", 0.15)), pct),
                )
                locked_qty = float(closed["entry_signal_qty"]) * pct
            except (TypeError, ValueError):
                locked_qty = None
        if entry_f is not None and exit_px is not None and direction in ("LONG", "SHORT"):
            pnl = self._compute_of_trade_pnl(
                direction=direction,
                entry_price=entry_f,
                exit_price=float(exit_px),
                notional=notional,
                entry_bid=closed.get("entry_bid"),
                entry_ask=closed.get("entry_ask"),
                exit_bid=bid,
                exit_ask=ask,
                quantity=locked_qty,
            )
            closed.update(pnl)
            # Force exit qty == entry paper qty (ignore any pnl recompute / exit signal)
            if locked_qty is not None:
                closed["simulated_quantity"] = locked_qty
                closed["exit_quantity"] = locked_qty
                closed["entry_paper_quantity"] = locked_qty
                closed["simulated_notional_usdt"] = round(locked_qty * entry_f, 4)
                closed["qty_matched_entry_to_exit"] = True
                notional = float(closed["simulated_notional_usdt"])
            if isinstance(exit_candle, dict):
                exit_candle["paper_quantity"] = locked_qty
                exit_candle["qty_matched_to_entry"] = True
            closed["exit_candle"] = exit_candle
            outcome = {
                "protocol_id": OF_PAPER_PROTOCOL_ID,
                "execution_mode": "PAPER",
                "of_trade_id": closed.get("of_trade_id"),
                "plan_id": closed.get("plan_id"),
                "symbol": closed.get("symbol") or "BTCUSDT",
                "direction": direction,
                "entry_price": entry_f,
                "exit_price": float(exit_px),
                "entry_bid": closed.get("entry_bid"),
                "entry_ask": closed.get("entry_ask"),
                "entry_signal_qty": closed.get("entry_signal_qty"),
                "entry_paper_quantity": locked_qty,
                "exit_signal_qty": closed.get("exit_signal_qty"),
                "exit_quantity": locked_qty if locked_qty is not None else pnl.get("simulated_quantity"),
                "qty_matched_entry_to_exit": locked_qty is not None,
                "participation_pct": closed.get("participation_pct"),
                "sizing_source": closed.get("sizing_source"),
                "depth_quality_entry": closed.get("depth_quality"),
                "depth_quality_exit": closed.get("exit_depth_quality"),
                "exit_bid": bid,
                "exit_ask": ask,
                "exit_bid_qty": closed.get("exit_bid_qty"),
                "exit_ask_qty": closed.get("exit_ask_qty"),
                "simulated_notional_usdt": notional,
                "simulated_quantity": locked_qty if locked_qty is not None else pnl.get("simulated_quantity"),
                "gross_pnl_usdt": pnl.get("gross_pnl_usdt"),
                "spread_cost_usdt": pnl.get("spread_cost_usdt"),
                "fees_usdt": pnl.get("fees_usdt"),
                "slippage_usdt": pnl.get("slippage_usdt"),
                "slippage_model": pnl.get("slippage_model", "none"),
                "friction_usdt": pnl.get("friction_usdt"),
                "net_pnl_usdt": pnl.get("net_pnl_usdt"),
                "net_return_bps": pnl.get("net_return_bps"),
                "holding_duration_seconds": hold_s,
                "activated_ms": act_ms,
                "paper_open_timestamp": closed.get("paper_open_timestamp"),
                "signal_exit_ms": signal_exit_ms,
                "signal_exit_timestamp": signal_exit_ts,
                "paper_close_ms": paper_close_ms,
                "paper_close_timestamp": paper_close_ts,
                "paper_close_lag_ms": paper_close_lag_ms,
                "exited_ms": paper_close_ms,
                "exit_reason": reason,
                "exit_imbalance": closed.get("exit_imbalance"),
                "exit_opposite_side": closed.get("exit_opposite_side"),
                "entry_candle": closed.get("entry_candle"),
                "exit_candle": exit_candle,
                "order_flow_imbalance_entry": closed.get("order_flow_imbalance"),
                "live_order_authority": 0.0,
                "independent_of_trade_plan_ttl": True,
                "depth_quality_protocol_id": OF_DEPTH_QUALITY_PROTOCOL_ID,
            }
            if self._log_of_outcome(outcome):
                summary = self.state.setdefault(
                    "of_outcomes_summary",
                    {
                        "n_trades": 0,
                        "n_wins": 0,
                        "n_losses": 0,
                        "cumulative_gross_pnl_usdt": 0.0,
                        "cumulative_net_pnl_usdt": 0.0,
                        "cumulative_net_return_bps": 0.0,
                        "cumulative_friction_usdt": 0.0,
                        "cumulative_spread_cost_usdt": 0.0,
                        "cumulative_fees_usdt": 0.0,
                        "cumulative_slippage_usdt": 0.0,
                    },
                )
                net = float(pnl.get("net_pnl_usdt") or 0.0)
                bps = float(pnl.get("net_return_bps") or 0.0)
                summary["n_trades"] = int(summary.get("n_trades") or 0) + 1
                if net > 0:
                    summary["n_wins"] = int(summary.get("n_wins") or 0) + 1
                else:
                    summary["n_losses"] = int(summary.get("n_losses") or 0) + 1
                summary["cumulative_gross_pnl_usdt"] = round(
                    float(summary.get("cumulative_gross_pnl_usdt") or 0.0)
                    + float(pnl.get("gross_pnl_usdt") or 0.0),
                    4,
                )
                summary["cumulative_net_pnl_usdt"] = round(
                    float(summary.get("cumulative_net_pnl_usdt") or 0.0) + net,
                    4,
                )
                summary["cumulative_net_return_bps"] = round(
                    float(summary.get("cumulative_net_return_bps") or 0.0) + bps,
                    2,
                )
                summary["cumulative_friction_usdt"] = round(
                    float(summary.get("cumulative_friction_usdt") or 0.0)
                    + float(pnl.get("friction_usdt") or 0.0),
                    4,
                )
                summary["cumulative_spread_cost_usdt"] = round(
                    float(summary.get("cumulative_spread_cost_usdt") or 0.0)
                    + float(pnl.get("spread_cost_usdt") or 0.0),
                    4,
                )
                summary["cumulative_fees_usdt"] = round(
                    float(summary.get("cumulative_fees_usdt") or 0.0)
                    + float(pnl.get("fees_usdt") or 0.0),
                    4,
                )
                summary["cumulative_slippage_usdt"] = round(
                    float(summary.get("cumulative_slippage_usdt") or 0.0)
                    + float(pnl.get("slippage_usdt") or 0.0),
                    4,
                )
                self.state["of_outcomes_summary"] = summary

        self.state["last_closed_order_flow_recommendation"] = closed
        self.state["active_order_flow_recommendation"] = None
        self._write_of_open_state_file(None)
        self._log_event(
            "ORDER_FLOW_EXIT",
            {
                "plan_id": closed.get("plan_id"),
                "direction": closed.get("direction"),
                "entry_price": closed.get("entry_price"),
                "state": "OF_EXIT",
            },
            {
                "reason": reason,
                "exit_imbalance": (exit_meta or {}).get("order_flow_imbalance"),
                "opposite_side": (exit_meta or {}).get("opposite_side"),
                "entry_candle": closed.get("entry_candle"),
                "exit_candle": exit_candle,
                "of_trade_id": closed.get("of_trade_id"),
                "exit_price": closed.get("exit_price"),
                "net_pnl_usdt": closed.get("net_pnl_usdt"),
                "net_return_bps": closed.get("net_return_bps"),
                "holding_duration_seconds": hold_s,
                "signal_exit_ms": signal_exit_ms,
                "paper_close_ms": paper_close_ms,
                "paper_close_lag_ms": paper_close_lag_ms,
                "execution_mode": "PAPER",
                "protocol_id": OF_PAPER_PROTOCOL_ID,
            },
        )

    def evaluate_order_flow_tick(
        self,
        quote: Optional[Dict[str, Any]] = None,
        *,
        features_tick: Optional[Dict[str, Any]] = None,
        now_ms: Optional[int] = None,
        persist: bool = True,
        candle_ctx: Optional[Dict[str, Any]] = None,
    ) -> Optional[Dict[str, Any]]:
        """
        Update active OF recommendation on each L1 tick.
        Exit only when opposite-side order-flow confirm fires (not on 60m / TP / SL).
        """
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        if self.reconcile_of_paper_already_closed(persist=persist):
            return None
        rec = self.state.get("active_order_flow_recommendation")
        if not rec or not rec.get("active"):
            return None

        l1 = l1_from_book_ticker(quote if isinstance(quote, dict) else None)
        if l1.get("source") == "none" and features_tick:
            l1 = l1_from_tick_features(features_tick)

        direction = str(rec.get("direction") or "")
        exit_sig = is_order_flow_exit_signal(direction, l1)
        # Keep live imbalance telemetry on the active rec
        of_same = classify_order_flow_support(direction, l1)
        if of_same.get("order_flow_imbalance") is not None:
            rec["last_book_imbalance"] = of_same["order_flow_imbalance"]
            rec["last_of_eval_ms"] = now
            # Keep entry/confirm imbalance stable; only refresh while same-side still supports
            if of_same.get("order_flow_supported"):
                rec["order_flow_imbalance"] = of_same["order_flow_imbalance"]
            self.state["active_order_flow_recommendation"] = rec

        if exit_sig.get("exit"):
            ctx = candle_ctx or {}
            exit_mark = build_of_candle_mark(
                now_ms=now,
                bar_idx=ctx.get("bar_idx"),
                decision_event_idx=ctx.get("decision_event_idx"),
                card={"close_time_ms": ctx.get("close_time_ms") or now},
            )
            self._clear_order_flow_recommendation(
                reason="OPPOSITE_SIDE_ORDER_FLOW",
                exit_meta=exit_sig,
                now_ms=now,
                candle_mark=exit_mark,
                quote=quote if isinstance(quote, dict) else None,
            )
            if persist:
                self._save_state()
            # May still be open if anti-churn suppressed the close
            still = self.state.get("active_order_flow_recommendation")
            if still and still.get("active"):
                return dict(still)
            return None

        # Opposite OF not present — reset persistence arming
        if rec.get("opposite_exit_seen_ms") is not None:
            rec["opposite_exit_seen_ms"] = None
            rec.pop("close_suppressed_reason", None)
            self.state["active_order_flow_recommendation"] = rec

        if persist:
            self._save_state()
        return dict(self.state.get("active_order_flow_recommendation") or {})

    def get_active_order_flow_recommendation(
        self, *, now_ms: Optional[int] = None, quote: Optional[Dict[str, Any]] = None
    ) -> Optional[Dict[str, Any]]:
        """Return active OF rec after applying any pending exit signal on quote."""
        if quote is not None:
            return self.evaluate_order_flow_tick(quote, now_ms=now_ms, persist=True)
        rec = self.state.get("active_order_flow_recommendation")
        if rec and rec.get("active"):
            return dict(rec)
        return None

    def _save_state(self):
        tmp_file = self.state_file.with_suffix(".tmp")
        with open(tmp_file, "w", encoding="utf-8") as f:
            json.dump(self.state, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        tmp_file.replace(self.state_file)

    def _log_event(self, event_type: str, plan_data: Dict[str, Any], extra: Optional[Dict[str, Any]] = None):
        evt = {
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "timestamp_ms": int(time.time() * 1000),
            "event_type": event_type,
            "plan_id": plan_data.get("plan_id"),
            "direction": plan_data.get("direction"),
            "entry_price": plan_data.get("entry_price"),
            "stop_loss_price": plan_data.get("stop_loss_price"),
            "profit_target_price": plan_data.get("profit_target_price"),
            "state": plan_data.get("state")
        }
        if extra:
            evt.update(extra)
        with open(self.events_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(evt) + "\n")

    def _log_outcome(self, outcome_data: Dict[str, Any]):
        with open(self.outcomes_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(outcome_data) + "\n")

    def _known_of_outcome_ids(self) -> set:
        """of_trade_ids already journaled (idempotent close)."""
        cached = getattr(self, "_of_outcome_id_cache", None)
        if cached is not None:
            return cached
        ids: set = set()
        if self.of_outcomes_file.exists():
            try:
                with open(self.of_outcomes_file, "r", encoding="utf-8") as f:
                    for line in f:
                        line = line.strip()
                        if not line:
                            continue
                        try:
                            row = json.loads(line)
                        except Exception:
                            continue
                        tid = row.get("of_trade_id")
                        if tid:
                            ids.add(str(tid))
            except Exception:
                pass
        self._of_outcome_id_cache = ids
        return ids

    def _log_of_outcome(self, outcome_data: Dict[str, Any]) -> bool:
        """
        Append one closed OF paper trade. Idempotent on of_trade_id so dual
        writers (daemon + dashboard) cannot inflate the ledger.
        Returns True if a new row was written.
        """
        tid = outcome_data.get("of_trade_id")
        if tid is not None:
            known = self._known_of_outcome_ids()
            if str(tid) in known:
                return False
        with open(self.of_outcomes_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(outcome_data) + "\n")
        if tid is not None:
            self._known_of_outcome_ids().add(str(tid))
        return True

    @staticmethod
    def _dedupe_of_outcomes_keep_first(rows: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Stable unique by of_trade_id (first occurrence = first close)."""
        seen: set = set()
        out: List[Dict[str, Any]] = []
        for row in rows:
            tid = row.get("of_trade_id")
            key = str(tid) if tid is not None else None
            if key is not None:
                if key in seen:
                    continue
                seen.add(key)
            out.append(row)
        return out

    def rebuild_of_outcomes_summary(self, *, persist: bool = True) -> Dict[str, Any]:
        """Recompute summary from deduped journal (repairs dual-writer inflation)."""
        rows: List[Dict[str, Any]] = []
        if self.of_outcomes_file.exists():
            try:
                with open(self.of_outcomes_file, "r", encoding="utf-8") as f:
                    for line in f:
                        line = line.strip()
                        if not line:
                            continue
                        try:
                            rows.append(json.loads(line))
                        except Exception:
                            continue
            except Exception:
                rows = []
        uniq = self._dedupe_of_outcomes_keep_first(rows)
        summary = {
            "n_trades": 0,
            "n_wins": 0,
            "n_losses": 0,
            "cumulative_gross_pnl_usdt": 0.0,
            "cumulative_net_pnl_usdt": 0.0,
            "cumulative_net_return_bps": 0.0,
            "cumulative_friction_usdt": 0.0,
            "cumulative_spread_cost_usdt": 0.0,
            "cumulative_fees_usdt": 0.0,
            "cumulative_slippage_usdt": 0.0,
        }
        for row in uniq:
            net = float(row.get("net_pnl_usdt") or 0.0)
            bps = float(row.get("net_return_bps") or 0.0)
            summary["n_trades"] += 1
            if net > 0:
                summary["n_wins"] += 1
            else:
                summary["n_losses"] += 1
            summary["cumulative_gross_pnl_usdt"] = round(
                summary["cumulative_gross_pnl_usdt"] + float(row.get("gross_pnl_usdt") or 0.0), 4
            )
            summary["cumulative_net_pnl_usdt"] = round(
                summary["cumulative_net_pnl_usdt"] + net, 4
            )
            summary["cumulative_net_return_bps"] = round(
                summary["cumulative_net_return_bps"] + bps, 2
            )
            summary["cumulative_friction_usdt"] = round(
                summary["cumulative_friction_usdt"] + float(row.get("friction_usdt") or 0.0), 4
            )
            summary["cumulative_spread_cost_usdt"] = round(
                summary["cumulative_spread_cost_usdt"]
                + float(row.get("spread_cost_usdt") or 0.0),
                4,
            )
            summary["cumulative_fees_usdt"] = round(
                summary["cumulative_fees_usdt"] + float(row.get("fees_usdt") or 0.0), 4
            )
            summary["cumulative_slippage_usdt"] = round(
                summary["cumulative_slippage_usdt"] + float(row.get("slippage_usdt") or 0.0), 4
            )
        self.state["of_outcomes_summary"] = summary
        self._of_outcome_id_cache = {
            str(r["of_trade_id"]) for r in uniq if r.get("of_trade_id") is not None
        }
        if persist:
            self._save_state()
        return summary

    def compact_of_outcomes_journal(self) -> Dict[str, Any]:
        """Rewrite outcomes jsonl with one row per of_trade_id; rebuild summary."""
        rows: List[Dict[str, Any]] = []
        if self.of_outcomes_file.exists():
            with open(self.of_outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        rows.append(json.loads(line))
                    except Exception:
                        continue
        uniq = self._dedupe_of_outcomes_keep_first(rows)
        tmp = self.of_outcomes_file.with_suffix(".jsonl.tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            for row in uniq:
                f.write(json.dumps(row) + "\n")
            f.flush()
            os.fsync(f.fileno())
        tmp.replace(self.of_outcomes_file)
        self._of_outcome_id_cache = None
        summary = self.rebuild_of_outcomes_summary(persist=True)
        return {"rows_before": len(rows), "rows_after": len(uniq), "summary": summary}

    def load_recent_of_outcomes(self, n: int = 24) -> List[Dict[str, Any]]:
        """Newest-first tail of OF P&L ledger (deduped by of_trade_id)."""
        if n <= 0 or not self.of_outcomes_file.exists():
            return []
        rows: List[Dict[str, Any]] = []
        try:
            with open(self.of_outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        rows.append(json.loads(line))
                    except Exception:
                        continue
        except Exception:
            return []
        uniq = self._dedupe_of_outcomes_keep_first(rows)
        return list(reversed(uniq[-n:]))

    def process_decision_card(self, card: Dict[str, Any], current_quote: Optional[Dict[str, Any]] = None) -> Optional[Dict[str, Any]]:
        """Evaluates a new decision card for Trade Plan creation or suppression."""
        # Strict Boundary Check: Exclude pre-window startup cards (< 05:30 IST / 1792972800000 ms)
        close_time_ms = int(card.get("close_time_ms", 0))
        if close_time_ms < 1792972800000:
            return None

        direction = card.get("policy_b_action")
        if direction not in ("LONG", "SHORT"):
            # If no new signal, preserve and return existing active plan
            active = self.state.get("active_plan")
            if active and active.get("state") == "OPEN":
                return active
            return None

        # Check concurrency: 1 plan max
        if self.state.get("active_plan"):
            active = self.state["active_plan"]
            if active.get("state") == "OPEN":
                self.state["suppressed_signals_count"] += 1
                self._log_event("SUPPRESSED_OPEN_PLAN", active, {
                    "suppressed_card_bar_idx": card.get("bar_idx"),
                    "suppressed_direction": direction
                })
                # Plan suppressed, but OF recommendation can still activate / refresh / exit
                self.ingest_order_flow_signal(
                    direction=str(direction),
                    card=card,
                    quote=current_quote,
                    plan_id=active.get("plan_id"),
                    entry_price=active.get("entry_price"),
                    entry_time_ms=active.get("entry_time_ms"),
                    persist=True,
                )
                return active

        # Create new plan
        plan = calculate_trade_plan(card, quote=current_quote)
        if not plan:
            return None

        self.state["active_plan"] = plan
        self.state["total_plans_created"] += 1
        self._log_event(
            "PLAN_CREATED",
            plan,
            {
                "order_flow_supported": plan.get("order_flow_supported"),
                "order_flow_imbalance": plan.get("order_flow_imbalance"),
            },
        )
        # OF recommendation: activate on OF entry; independent of 60m plan clock
        self.ingest_order_flow_signal(
            direction=str(direction),
            card=card,
            quote=(
                current_quote
                or (card.get("execution_quote") if isinstance(card.get("execution_quote"), dict) else None)
                or plan.get("execution_quote")
            ),
            plan_id=plan.get("plan_id"),
            entry_price=plan.get("entry_price"),
            entry_time_ms=plan.get("entry_time_ms"),
            persist=False,
        )
        self._save_state()
        return plan

    def ingest_order_flow_signal(
        self,
        *,
        direction: str,
        card: Optional[Dict[str, Any]] = None,
        quote: Optional[Dict[str, Any]] = None,
        plan_id: Optional[str] = None,
        entry_price: Optional[float] = None,
        entry_time_ms: Optional[int] = None,
        persist: bool = True,
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """
        Activate OF recommendation when L1 confirms direction.
        Remains active until opposite-side OF exit signal (not 60m / TP / SL).
        """
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        card = card or {}
        # Size/entry need L1 qty — prefer live quote, else card execution_quote
        if not isinstance(quote, dict) or not quote:
            cq = card.get("execution_quote")
            quote = cq if isinstance(cq, dict) else None
        entry_mark = build_of_candle_mark(now_ms=now, card=card)
        candle_ctx = {
            "bar_idx": entry_mark.get("bar_idx"),
            "decision_event_idx": entry_mark.get("decision_event_idx"),
            "close_time_ms": entry_mark.get("mark_time_ms"),
        }
        direction_u = str(direction or "").upper()
        if direction_u not in ("LONG", "SHORT"):
            # Still allow exit evaluation on quote
            return self.evaluate_order_flow_tick(
                quote,
                features_tick=card.get("features_tick"),
                now_ms=now,
                persist=persist,
                candle_ctx=candle_ctx,
            )

        # Prefer stamped card OF, else book, else tick features
        if card.get("order_flow_supported") and card.get("order_flow_label") == ORDER_FLOW_LABEL:
            of = {
                "order_flow_supported": True,
                "order_flow_label": ORDER_FLOW_LABEL,
                "order_flow_imbalance": card.get("order_flow_imbalance"),
                "order_flow_source": str(card.get("order_flow_source") or "decision_card"),
            }
        else:
            of = classify_order_flow_support(
                direction_u, l1_from_book_ticker(quote if isinstance(quote, dict) else None)
            )
            if not of["order_flow_supported"]:
                of = classify_order_flow_support(
                    direction_u,
                    l1_from_tick_features(
                        card.get("features_tick")
                        if isinstance(card.get("features_tick"), dict)
                        else None
                    ),
                )

        active = self.state.get("active_order_flow_recommendation")
        if active and active.get("active"):
            # First apply exit on current book (may clear before re-entry)
            self.evaluate_order_flow_tick(
                quote,
                features_tick=card.get("features_tick") if isinstance(card.get("features_tick"), dict) else None,
                now_ms=now,
                persist=False,
                candle_ctx=candle_ctx,
            )
            active = self.state.get("active_order_flow_recommendation")

        if not of.get("order_flow_supported"):
            if persist:
                self._save_state()
            return self.state.get("active_order_flow_recommendation")

        # Opposite active OF → exit first, then activate new side
        if active and active.get("active") and str(active.get("direction")) != direction_u:
            self._clear_order_flow_recommendation(
                reason="OPPOSITE_SIDE_ORDER_FLOW_ENTRY",
                exit_meta={
                    "order_flow_imbalance": of.get("order_flow_imbalance"),
                    "opposite_side": direction_u,
                },
                now_ms=now,
                candle_mark=entry_mark,
                quote=quote if isinstance(quote, dict) else None,
            )

        active = self.state.get("active_order_flow_recommendation")
        if active and active.get("active") and str(active.get("direction")) == direction_u:
            # Refresh telemetry only
            active["order_flow_imbalance"] = of.get("order_flow_imbalance")
            active["order_flow_source"] = of.get("order_flow_source")
            active["last_of_eval_ms"] = now
            if plan_id:
                active["plan_id"] = plan_id
            self.state["active_order_flow_recommendation"] = active
        else:
            rec = self._activate_order_flow_recommendation(
                direction=direction_u,
                of=of,
                plan_id=plan_id,
                entry_price=entry_price,
                entry_time_ms=entry_time_ms,
                now_ms=now,
                candle_mark=entry_mark,
                quote=quote if isinstance(quote, dict) else None,
            )
            self._log_event(
                "ORDER_FLOW_RECOMMENDATION_ACTIVE",
                {
                    "plan_id": plan_id,
                    "direction": direction_u,
                    "entry_price": entry_price,
                    "state": "OF_ACTIVE",
                },
                {
                    "order_flow_imbalance": of.get("order_flow_imbalance"),
                    "order_flow_source": of.get("order_flow_source"),
                    "entry_candle": entry_mark,
                },
            )
            if persist:
                self._save_state()
            return rec

        if persist:
            self._save_state()
        return self.state.get("active_order_flow_recommendation")

    def evaluate_market_tick(
        self,
        quote: Dict[str, Any],
        *,
        reload_state: bool = True,
        now_ms: Optional[int] = None,
        evaluate_of: bool = True,
    ) -> Optional[Dict[str, Any]]:
        """
        Monitors open trade plan against live top-of-book (continuous, not 15m).

        Executable-side triggers (never the stale entry-bar decision quote):
          LONG  TP: bid >= target · SL: bid <= stop
          SHORT TP: ask <= target · SL: ask >= stop

        Hit-then-reverse invariant: the first observed touch of TP/SL closes the
        plan immediately. A later reverse inside the same 15m bar cannot unwind
        a fill that already latched. Optional low_price/high_price on the quote
        (e.g. forming kline extremes) count as path-through observations.

        evaluate_of: when False (dashboard read path), do not mutate OF paper trades.
        """
        if reload_state:
            self.state = self._load_state()
        plan = self.state.get("active_plan")
        if not plan or plan.get("state") != "OPEN":
            return None

        q = normalize_book_quote(quote)
        if not q:
            return None

        direction = plan["direction"]
        sl = float(plan["stop_loss_price"])
        tp = float(plan["profit_target_price"])
        qty = plan["simulated_quantity"]
        entry_price = plan["entry_price"]
        entry_ms = plan["entry_time_ms"]
        expiry_ms = plan["expiry_time_ms"]

        now_ms = int(now_ms if now_ms is not None else time.time() * 1000)
        bid = float(q["best_bid_price"])
        ask = float(q["best_ask_price"])

        # Optional OHLC extremes (forming kline) — path-through inside the 15m bar.
        low = float(q["low_price"]) if q.get("low_price") is not None else min(bid, ask)
        high = float(q["high_price"]) if q.get("high_price") is not None else max(bid, ask)

        # Track observed executable extremes (audit + reverse-proof latch).
        exec_px = bid if direction == "LONG" else ask
        if direction == "LONG":
            fav = max(bid, high)
            adv = min(bid, low)
            prev_mfe = plan.get("mfe_exec_price")
            prev_mae = plan.get("mae_exec_price")
            plan["mfe_exec_price"] = round_tick(max(float(prev_mfe), fav) if prev_mfe is not None else fav)
            plan["mae_exec_price"] = round_tick(min(float(prev_mae), adv) if prev_mae is not None else adv)
        else:
            fav = min(ask, low)  # favorable for SHORT = lower cover ask
            adv = max(ask, high)
            prev_mfe = plan.get("mfe_exec_price")
            prev_mae = plan.get("mae_exec_price")
            plan["mfe_exec_price"] = round_tick(min(float(prev_mfe), fav) if prev_mfe is not None else fav)
            plan["mae_exec_price"] = round_tick(max(float(prev_mae), adv) if prev_mae is not None else adv)
        plan["last_mark_price"] = round_tick(exec_px)
        plan["last_mark_time_ms"] = now_ms
        plan["lifecycle_monitor"] = CONFIG.get("lifecycle_monitor", "continuous_l1_bookTicker")
        self.state["active_plan"] = plan

        # OF exit is L1 reverse-flow only — independent of plan TP/SL/60m
        if evaluate_of:
            self.evaluate_order_flow_tick(
                q,
                now_ms=now_ms,
                persist=False,
                candle_ctx={
                    "bar_idx": plan.get("source_bar_idx"),
                    "decision_event_idx": plan.get("source_decision_event_idx"),
                    "close_time_ms": now_ms,
                },
            )

        hit_tp = False
        hit_sl = False
        exit_price = 0.0
        terminal_state = None
        exit_reason = None

        if direction == "LONG":
            if bid >= tp or high >= tp or float(plan["mfe_exec_price"]) >= tp:
                hit_tp = True
            if bid <= sl or low <= sl or float(plan["mae_exec_price"]) <= sl:
                hit_sl = True
            exit_price = sl if hit_sl else (tp if hit_tp else bid)
        else:  # SHORT — buy-to-cover on ask
            if ask <= tp or low <= tp or float(plan["mfe_exec_price"]) <= tp:
                hit_tp = True
            if ask >= sl or high >= sl or float(plan["mae_exec_price"]) >= sl:
                hit_sl = True
            exit_price = sl if hit_sl else (tp if hit_tp else ask)

        if hit_tp and hit_sl:
            terminal_state = "CLOSED_SL"
            exit_reason = "AMBIGUOUS_STOP_TARGET"
            self.state["outcomes_summary"]["ambiguous_hits"] += 1
        elif hit_tp:
            terminal_state = "CLOSED_TP"
            exit_reason = "PROFIT_TARGET_HIT"
            self.state["outcomes_summary"]["tp_hits"] += 1
        elif hit_sl:
            terminal_state = "CLOSED_SL"
            exit_reason = "STOP_LOSS_HIT"
            self.state["outcomes_summary"]["sl_hits"] += 1
        elif now_ms >= expiry_ms:
            terminal_state = "TIME_EXIT"
            exit_reason = "MAX_HORIZON_60M_EXPIRED"
            self.state["outcomes_summary"]["time_exits"] += 1

        if terminal_state:
            plan["state"] = terminal_state
            plan["exit_price"] = round_tick(exit_price)
            plan["exit_time_ms"] = now_ms
            plan["holding_duration_seconds"] = round((now_ms - entry_ms) / 1000.0, 1)

            if direction == "LONG":
                gross_pnl = (plan["exit_price"] - entry_price) * qty
            else:
                gross_pnl = (entry_price - plan["exit_price"]) * qty

            friction = (plan["simulated_notional_usdt"] + (plan["exit_price"] * qty)) * (6.0 / 10000.0)
            net_pnl = round(gross_pnl - friction, 2)

            plan["gross_pnl_usdt"] = round(gross_pnl, 2)
            plan["friction_usdt"] = round(friction, 4)
            plan["net_pnl_usdt"] = net_pnl
            plan["exit_reason"] = exit_reason

            self.state["closed_plans_count"] += 1
            self.state["outcomes_summary"]["cumulative_net_pnl_usdt"] += net_pnl
            self.state["last_closed_plan"] = dict(plan)
            self.state["active_plan"] = None  # reverse after latch cannot reopen this plan

            self._log_event(
                terminal_state,
                plan,
                {
                    "exit_price": plan["exit_price"],
                    "net_pnl": net_pnl,
                    "reason": exit_reason,
                    "mfe_exec_price": plan.get("mfe_exec_price"),
                    "mae_exec_price": plan.get("mae_exec_price"),
                    "quote_source": q.get("quote_source"),
                },
            )
            self._log_outcome(plan)
            self._save_state()
            return plan

        return plan

    def compute_mtm_valuation(self, plan: Optional[Dict[str, Any]], current_quote: Optional[Dict[str, Any]], now_ms: Optional[int] = None) -> Dict[str, Any]:
        """Calculates executable-side mark-to-market valuation after friction."""
        if not plan:
            return {}

        now_ms = now_ms if now_ms is not None else int(time.time() * 1000)
        direction = plan.get("direction", "LONG")
        entry_p = float(plan.get("entry_price", 0.0))
        qty = float(plan.get("simulated_quantity", 0.0))
        notional = float(plan.get("simulated_notional_usdt", 1000.0))

        q = current_quote or {}
        bid_p = float(q.get("best_bid_price", entry_p))
        ask_p = float(q.get("best_ask_price", entry_p))

        # Executable exit quote: bid for LONG, ask for SHORT
        exec_exit_p = bid_p if direction == "LONG" else ask_p

        if plan.get("state") in ("CLOSED_TP", "CLOSED_SL", "TIME_EXIT"):
            exec_exit_p = float(plan.get("exit_price", exec_exit_p))
            net_pnl = float(plan.get("net_pnl_usdt", 0.0))
            net_ret_bps = (net_pnl / notional) * 10000.0 if notional > 0 else 0.0
            return {
                "entry_price": entry_p,
                "current_bid": bid_p,
                "current_ask": ask_p,
                "executable_exit_price": exec_exit_p,
                "unrealized_gross_pnl_usdt": plan.get("gross_pnl_usdt", 0.0),
                "unrealized_net_pnl_usdt": net_pnl,
                "unrealized_net_return_bps": round(net_ret_bps, 2),
                "time_remaining_seconds": 0,
                "is_expired": True
            }

        # Open position mark-to-market
        if direction == "LONG":
            gross_pnl = (exec_exit_p - entry_p) * qty
        else:
            gross_pnl = (entry_p - exec_exit_p) * qty

        # Round-trip 12 bps friction on notional
        friction = (notional + (exec_exit_p * qty)) * (6.0 / 10000.0)
        net_pnl = gross_pnl - friction
        net_ret_bps = (net_pnl / notional) * 10000.0 if notional > 0 else 0.0

        time_remaining = max(0, int((plan.get("expiry_time_ms", now_ms) - now_ms) / 1000))

        return {
            "entry_price": entry_p,
            "current_bid": bid_p,
            "current_ask": ask_p,
            "executable_exit_price": exec_exit_p,
            "unrealized_gross_pnl_usdt": round(gross_pnl, 2),
            "unrealized_net_pnl_usdt": round(net_pnl, 2),
            "unrealized_net_return_bps": round(net_ret_bps, 2),
            "time_remaining_seconds": time_remaining,
            "is_expired": (time_remaining == 0)
        }

    def sync_lifecycle(
        self,
        current_quote: Optional[Dict[str, Any]] = None,
        now_ms: Optional[int] = None,
        *,
        mutate_of: bool = True,
        mutate_plans: bool = True,
    ) -> Dict[str, Any]:
        """
        Reconciles lifecycle state against latest quote and current time.
        Transitions expired or hit plans to terminal states.

        Prefer a live bookTicker; fall back to provided quote only if live fetch fails.
        Never evaluate TP/SL solely against a stale decision-card execution quote when
        a fresher market book is available.

        mutate_of / mutate_plans: dashboard should pass False so only the shadow
        daemon owns OF paper-trade and plan terminal transitions.
        """
        now_ms = now_ms if now_ms is not None else int(time.time() * 1000)
        self.state = self._load_state()
        # Ledger wins: never present a closed of_trade_id as OPEN (dashboard + daemon)
        self.reconcile_of_paper_already_closed(persist=mutate_of)
        active = self.state.get("active_plan")

        live = fetch_live_btcusdt_book()
        eval_quote = normalize_book_quote(live) or normalize_book_quote(current_quote)

        # Reconcile OF exit on live book (independent of plan / 60m) — daemon only
        if mutate_of and eval_quote:
            of_bar = None
            if active:
                of_bar = active.get("source_bar_idx")
            elif self.state.get("active_order_flow_recommendation"):
                of_bar = (
                    (self.state["active_order_flow_recommendation"].get("entry_candle") or {})
                    .get("bar_idx")
                )
            self.evaluate_order_flow_tick(
                eval_quote,
                now_ms=now_ms,
                persist=True,
                candle_ctx={"bar_idx": of_bar, "close_time_ms": now_ms},
            )

        if mutate_plans and active and active.get("state") == "OPEN":
            if eval_quote:
                resolved = self.evaluate_market_tick(
                    eval_quote,
                    reload_state=False,
                    now_ms=now_ms,
                    evaluate_of=mutate_of,
                )
                if resolved and resolved.get("state") != "OPEN":
                    return {
                        "active_plan": None,
                        "display_plan": resolved,
                        "is_active": False,
                        "state": resolved["state"],
                        "valuation": self.compute_mtm_valuation(resolved, eval_quote, now_ms),
                        "quote_source": eval_quote.get("quote_source") or "live_or_provided",
                    }

            # Refresh after tick (evaluate reloads/persists state)
            active = self.state.get("active_plan")
            if not active or active.get("state") != "OPEN":
                last_closed = self.state.get("last_closed_plan")
                return {
                    "active_plan": None,
                    "display_plan": last_closed,
                    "is_active": False,
                    "state": (last_closed or {}).get("state", "NO_PLAN"),
                    "valuation": self.compute_mtm_valuation(last_closed, eval_quote, now_ms) if last_closed else None,
                }

            # Check direct expiry
            if now_ms >= active.get("expiry_time_ms", 0):
                q = eval_quote or {
                    "best_bid_price": active["entry_price"],
                    "best_ask_price": active["entry_price"],
                }
                resolved = self.evaluate_market_tick(q, reload_state=False, now_ms=now_ms)
                return {
                    "active_plan": None,
                    "display_plan": resolved,
                    "is_active": False,
                    "state": resolved.get("state", "TIME_EXIT") if resolved else "TIME_EXIT",
                    "valuation": self.compute_mtm_valuation(resolved, q, now_ms),
                }

            # Still genuinely open — MTM against live book
            return {
                "active_plan": active,
                "display_plan": active,
                "is_active": True,
                "state": "OPEN",
                "valuation": self.compute_mtm_valuation(active, eval_quote, now_ms),
                "quote_source": (eval_quote or {}).get("quote_source") or "provided",
            }

        # Read-only path (dashboard): MTM only — do not terminalize plan/OF
        if active and active.get("state") == "OPEN":
            return {
                "active_plan": active,
                "display_plan": active,
                "is_active": True,
                "state": "OPEN",
                "valuation": self.compute_mtm_valuation(active, eval_quote, now_ms),
                "quote_source": (eval_quote or {}).get("quote_source") or "provided",
            }

        last_closed = self.state.get("last_closed_plan")
        return {
            "active_plan": None,
            "display_plan": last_closed,
            "is_active": False,
            "state": last_closed.get("state", "NO_PLAN") if last_closed else "NO_PLAN",
            "valuation": self.compute_mtm_valuation(last_closed, eval_quote or current_quote, now_ms) if last_closed else None,
        }


def run_standalone_sync():
    """Reads latest decision card from disk and synchronizes the Trade Plan overlay."""
    mgr = TradePlanOverlayManager()
    if not DECISION_CARDS_FILE.exists():
        print("[-] No decision cards file found.")
        return

    latest_card = None
    with open(DECISION_CARDS_FILE, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    latest_card = json.loads(line)
                except Exception:
                    continue

    if not latest_card:
        print("[-] No valid cards in log.")
        return

    print("=" * 80)
    print("  RP-608 TRADE-PLAN OVERLAY v1 — SYNCHRONIZATION")
    print(f"  Source Card: Bar #{latest_card.get('bar_idx')} ({latest_card.get('timestamp_utc')})")
    print("=" * 80)

    plan = mgr.process_decision_card(latest_card)
    if plan:
        print(f"[+] Active Trade Plan: {plan['plan_id']}")
        print(f"    Direction:      {plan['direction']}")
        print(f"    Eligible Entry: ${plan['entry_price']:,.2f}")
        print(f"    Stop Loss:      ${plan['stop_loss_price']:,.2f} (-${plan['gross_risk_usdt']:,.2f})")
        print(f"    Profit Target:  ${plan['profit_target_price']:,.2f} (+${plan['gross_reward_usdt']:,.2f})")
        print(f"    Reward/Risk:    Gross {plan['gross_rr_ratio']}:1 | Net {plan['net_rr_ratio']}:1")
        print(f"    Est Friction:   ${plan['est_friction_usdt']:.2f} ({plan['round_trip_friction_bps']} bps)")
        print(f"    Horizon:        {plan['max_holding_seconds']}s (60m) [Expires: {time.strftime('%H:%M:%S UTC', time.gmtime(plan['expiry_time_ms'] / 1000))}]")
        print(f"    State:          {plan['state']}")
    else:
        print("[*] No active trade plan issued (Policy B is NO_TRADE or criteria unfulfilled).")


if __name__ == "__main__":
    run_standalone_sync()
