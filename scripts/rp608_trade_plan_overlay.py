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
from datetime import datetime, timedelta, timezone
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
from scripts.rp608_crypto_trade_pattern_match import (
    build_gate2e_shadow_allow_stamp,
    entry_gate_mode as crypto_pattern_entry_gate_mode,
    exit_gate_enabled as crypto_pattern_exit_gate_enabled,
    features_from_card_or_plan as crypto_pattern_features,
    gate2e_shadow_thin_wr_enabled,
    load_library as load_crypto_pattern_library,
    match_entry as match_crypto_pattern_entry,
    match_exit as match_crypto_pattern_exit,
    matches_shadow_allow_if_n10_wr_gt_035,
    normalize_entry_gate_mode as crypto_normalize_entry_gate_mode,
)
from scripts.rp608_gate2e_shadow_insufficient import (
    build_gate2e_shadow_insufficient_stamp,
    gate2e_shadow_insufficient_enabled,
    matches_shadow_observe_abstain_insufficient,
)
from scripts.rp608_gate2e_regime_align import (
    evaluate_regime_align,
    regime_dict_from_card,
)
from scripts.rp608_gate2e_rejected_cf import (
    build_rejected_cf_stamp,
    rejected_cf_applies_to_book,
)
from scripts.rp608_adapter_select_15m import (
    PROTOCOL_ID as ADAPTER_SELECT_15M_PROTOCOL_ID,
    recommend as adapter_select_15m_recommend,
)
from scripts.rp608_gate2e_15m_trend_agree import (
    PROTOCOL_ID as TREND_AGREE_PROTOCOL_ID,
    agreement as trend_agreement,
)
from scripts.rp608_sci_adapter_rec import (
    PROTOCOL_ID as SCI_ADAPTER_REC_PROTOCOL_ID,
    recommend as sci_adapter_rec_recommend,
)
from scripts.rp608_crypto_trade_pattern_library import (
    default_baseline_library_path as crypto_baseline_pattern_library_path,
    default_library_path as crypto_pattern_library_path,
    session_date_utc as crypto_pattern_session_date_utc,
    vol_surge_bucket as crypto_vol_surge_bucket,
)
from scripts.rp608_gate2x_adapter_client import (
    DEFAULT_CRYPTO_EVIDENCE,
    apply_soft_breach_to_trade,
    build_snapshot as gate2x_build_snapshot,
    decide as gate2x_adapter_decide,
)
from scripts.rp608_plan_lifecycle_client import (
    get_session as plan_lifecycle_session,
    lifecycle_enabled as plan_lifecycle_enabled,
    log_require_rust_block as plan_lifecycle_log_require_rust,
    map_lifecycle_close,
    merge_lifecycle_into_plan,
    plan_from_overlay,
    require_rust_lifecycle as plan_lifecycle_require_rust,
)

CANDLE_15M_MS = 900_000


def _plan_volume_fields(decision_card: Dict[str, Any]) -> Dict[str, Any]:
    """Latch candle OHLCV surge for Gate 2E/2X patterns (not OFX aggression)."""
    burst = decision_card.get("vol_burst_20")
    surge = decision_card.get("vol_surge")
    if surge not in {"LOW", "MED", "HIGH", "UNK"}:
        try:
            surge = crypto_vol_surge_bucket(float(burst) if burst is not None else None)
        except (TypeError, ValueError):
            surge = "UNK"
    burst_f = None
    if burst is not None:
        try:
            burst_f = float(burst)
        except (TypeError, ValueError):
            burst_f = None
    return {
        "vol_burst_20": burst_f,
        "vol_surge": str(surge or "UNK"),
        "exit_vol_burst_20": burst_f,
        "exit_vol_surge": str(surge or "UNK"),
    }


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
OF_ENTRY_QUALITY_PROTOCOL_ID = "RP-608-OF-PAPER-ENTRY-QUALITY-001"


def of_paper_quality_gate_enabled() -> bool:
    """RP608_OF_PAPER_QUALITY_GATE: 0=stamp-only, 1=suppress open (default 1)."""
    raw = os.environ.get("RP608_OF_PAPER_QUALITY_GATE", "1").strip().lower()
    return raw not in {"0", "off", "false", "no"}


def plan_require_of_enabled() -> bool:
    """
    Gate 2E: RP608_CRYPTO_PLAN_REQUIRE_OF — suppress plan opens without L1 OF support.

    Evidence (BTC BASELINE 24h 2026-11-25): OF_FALSE 0/30 WR · −$92; OF_TRUE still
    mixed but not 0%. Does NOT change OF ±0.30 construct (`.cursor/rules/rp608-ofx.mdc`).
    Applies uniformly so BL/FL/R50 pairing stays intact. Default off until Pi arms it.
    """
    raw = os.environ.get("RP608_CRYPTO_PLAN_REQUIRE_OF", "0").strip().lower()
    return raw in {"1", "true", "yes", "on"}


def plan_b_persist_min_fires() -> int:
    """
    Gate 2E: RP608_CRYPTO_PLAN_B_PERSIST_MIN_FIRES — min same-direction Policy B
    fires in the lookback window (including this card) required to open.

    Evidence (BTC BL 8h 2026-11-25): 26/26 opens were isolated one-shot B flickers
    (99.3% of tradeable B cards have NO_TRADE neighbors). fires60≥2 CF: PASS n=6
    net −$2.50 vs FAIL n=20 net −$49.90. Default 0 = stamp-only.
    """
    raw = os.environ.get("RP608_CRYPTO_PLAN_B_PERSIST_MIN_FIRES", "0").strip()
    try:
        return max(0, int(raw))
    except (TypeError, ValueError):
        return 0


def plan_b_persist_window_s() -> float:
    raw = os.environ.get("RP608_CRYPTO_PLAN_B_PERSIST_WINDOW_S", "60").strip()
    try:
        return max(1.0, float(raw))
    except (TypeError, ValueError):
        return 60.0


CONFIG_FILE = OVERLAY_DIR / "overlay_config.json"
PLAN_REQUIRE_OF_PROTOCOL_ID = "RP-608-GATE2E-PLAN-REQUIRE-OF-001"
PLAN_B_PERSIST_PROTOCOL_ID = "RP-608-GATE2E-PLAN-B-PERSIST-001"


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
    "version": "1.2.6",
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
    # Gate 5 Adaptive (2026-11-23): Coralys/Gate2X intelligent trail.
    # Exit on Coralys Protect/Exit (reversal doubt); SL ratchets only to Coralys
    # protect/ratchet floors — not blind ATR. TP runner-only under Adaptive.
    "adaptive_tp_enabled": True,
    "adaptive_tp_extend_atr_mult": 0.5,
    "adaptive_tp_max_extensions": 3,
    "adaptive_tp_protocol": "RP-608-GATE5-CRYPTO-ADAPTIVE-TP-001",
    "adaptive_partial_latch_enabled": False,
    "adaptive_dumb_atr_trail": False,
    "adaptive_partial_latch_arm_tp_frac": 0.70,
    "adaptive_partial_latch_lock_tp_frac": 0.50,
    # Standalone ATR trail only when Adaptive is off.
    "trailing_sl_enabled": True,
    "trailing_sl_atr_mult": 0.5,
    "trailing_sl_arm_risk_frac": 0.30,
    "trailing_sl_protocol": "RP-608-TRAILING-SL-001",
    "max_holding_seconds": 3600,  # 60 minutes
    # Gate 5 (2026-11-15): ambiguous TP∧SL same observation → TP (policy G).
    # Prior: conservative SL / AMBIGUOUS_STOP_TARGET. Authority remains 0%.
    "ambiguous_tie_break": "TP",  # TP | SL
    "gate5_policy_g": "RP-608-GATE5-CRYPTO-G-001",
    # Gate 5 H120_PATH (2026-11-16): early ATR SL (<120s) → scratch@entry only if
    # L1 mid left entry±6bps adverse then revisited the band (Gate3X PATH, not OPT).
    "gate5_policy_h": "RP-608-GATE5-CRYPTO-H-001",
    "early_sl_path_scratch_enabled": True,
    "early_sl_path_scratch_max_hold_s": 120.0,
    "early_sl_path_friction_bps_side": 6.0,
    # Gate 2E (2026-11-20): twin ABSTAIN study — W/L damage was re-entry after
    # instant SL / path-through stop, not TP/SL geometry (Gate 5 frozen).
    # Empty book > churn. Does not change OF ±0.30 / ATR stops / targets.
    "gate2e_post_fast_sl_cooldown_enabled": True,
    "gate2e_post_fast_sl_max_hold_s": 120.0,  # arm cooldown only after fast SL/scratch
    "gate2e_post_fast_sl_cooldown_ms": 300_000,  # 5m same-direction standaside
    "gate2e_preentry_path_invalid_enabled": True,
    # Observational dual-stamp (2026-11-21): compare forming 15m H/L path-invalid
    # vs tick/last mid. Live open authority stays forming-H/L only (no Gate 5).
    "gate2e_path_dual_stamp_observational": True,
    # Gate 2X (2026-11-21): promote quality path toward TP — arm only after peak
    # tp_frac clears arm_min (default 0.40) AND MFE clears cost+floor; then lock a
    # cost-clear floor (exit when cur_mfe <= threshold). Instant <30s TP cascade is
    # observational non-promote (not suppressed — path-through, not entry alpha).
    # v1.4: after adaptive TP arm, Hold|Protect|Exit decides continue vs take profit.
    # Does not change OF ±0.30 / initial ATR open multipliers.
    "gate2x_mfe_giveback_protect_enabled": True,
    "gate2x_mfe_meaningful_floor_bps": 5.0,
    "gate2x_arm_min_tp_frac": 0.40,  # quality progress toward TP before arm
    "gate2x_min_hold_s_before_exit": 120.0,  # avoid wick giveback exits
    "gate2x_protect_floor": "COST_CLEAR",  # COST_CLEAR | ENTRY (cur_mfe<=0)
    # v1.2: fake/small reversal guard — HIGH candle surge confirms exit now;
    # LOW/MED/UNK soft-breach must dwell before protect fires.
    "gate2x_reversal_volume_confirm_enabled": True,
    "gate2x_reversal_immediate_vol_surges": ["HIGH"],
    "gate2x_reversal_med_vol_dwell_ms": 40_000,
    "gate2x_reversal_low_vol_dwell_ms": 90_000,  # LOW + UNK
    # v1.3: ratchet floor with peak MFE; shorten dwell on deep giveback.
    # R4: Rust Gate2xPresetId — baseline_r30 | r50_booking | floor_lock_r50
    "gate2x_preset": "baseline_r30",
    "gate2x_ratchet_peak_frac": 0.30,  # floor = max(cost_clear, 0.30×peak_mfe)
    # Gate 2X r50 booking twin (2026-11-24): sensible lock of printed MFE.
    # Live PRIMARY/BASELINE stay 0.30 until Gate 5; r50 twin only.
    "gate2x_ratchet_peak_frac_r50": 0.50,
    # Gate 2X floor-lock twin (2026-11-24): exit at protect floor immediately —
    # no dwell/bleed to COST_CLEAR HARD_FLOOR. Observational only (authority 0%).
    "gate2x_floor_lock_immediate": False,
    "gate2x_floor_lock_protocol": "RP-608-GATE2X-FLOOR-LOCK-TWIN-001",
    "gate2x_deep_giveback_frac": 0.50,  # (peak-cur)/peak
    "gate2x_deep_giveback_dwell_scale": 0.40,  # multiply dwell when deep
    # v1.5: once through cost_clear during soft-breach, Exit now (no dwell bleed).
    # Stamped in Rust gate2x_decide as MFE_GIVEBACK_TO_COST_CLEAR_HARD_FLOOR.
    "gate2x_mfe_giveback_protocol": "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001",
    "gate2x_r50_booking_protocol": "RP-608-GATE2X-R50-BOOKING-TWIN-001",
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
    "of_paper_entry_persist_ms": 30_000,       # 30s continuous same-side OF before paper open
    "of_paper_one_per_15m_bar": True,
    # PAPER open quality (does NOT change OF ±0.30 mark — only whether a paper
    # trade is booked). Rejects micro/hollow books that Stage-1 showed as bleed.
    "of_paper_reject_thin_opposite": True,
    "of_paper_min_entry_signal_qty": 0.25,     # BTC L1 same-side qty floor
    "of_paper_min_same_side_notional_usdt": 5_000.0,
    # 2026-11-21 diag: QUALITY vs WEAK same_side notional did NOT separate
    # (WEAK denser) — do not raise floor; entry persist is the quality lever.
    "of_paper_min_combined_touch_notional_usdt": 0.0,
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


def fetch_live_book_ticker(
    symbol: str = "BTCUSDT", timeout_sec: float = 2.0
) -> Optional[Dict[str, Any]]:
    """
    Live Binance spot bookTicker (REST). Used for TP/SL/MTM — never the
    entry-bar decision-card quote, which goes stale within seconds.
    """
    sym = str(symbol or "BTCUSDT").strip().upper() or "BTCUSDT"
    url = f"https://api.binance.com/api/v3/ticker/bookTicker?symbol={sym}"
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
            "symbol": sym,
        }
    except Exception:
        return None


def fetch_live_btcusdt_book(timeout_sec: float = 2.0) -> Optional[Dict[str, Any]]:
    """Back-compat alias — BTCUSDT bookTicker REST."""
    return fetch_live_book_ticker("BTCUSDT", timeout_sec=timeout_sec)


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
        "original_stop_loss_price": stop_price,
        "profit_target_price": target_price,
        "tp_level": 0,
        "adaptive_tp_armed": False,
        "adaptive_tp_protocol": CONFIG.get("adaptive_tp_protocol"),
        "trailing_sl_armed": False,
        "trailing_sl_protocol": CONFIG.get("trailing_sl_protocol"),
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
        # Gate 5 H120_PATH live path stamps (L1 mid)
        "path_h_adverse_seen": False,
        "path_h_revisit_seen": False,
        "symbol": str(decision_card.get("symbol") or "BTCUSDT").strip().upper() or "BTCUSDT",
        # Candle OHLCV volume surge (pattern Gate 2E/2X; not OFX aggression)
        **_plan_volume_fields(decision_card),
    }


class TradePlanOverlayManager:
    """Manages lifecycle states, state persistence, and outcome recording."""

    def __init__(
        self,
        base_dir: Optional[Path] = None,
        *,
        book_id: str = "CRYPTO_PLAN",
        symbol: str = "BTCUSDT",
        of_paper_enabled: bool = True,
        volume_pattern_axes: bool = True,
        pattern_library_path: Optional[Path] = None,
        pattern_entry_gate_mode: Optional[str] = None,
        config_overrides: Optional[Dict[str, Any]] = None,
    ):
        self.base_dir = Path(base_dir) if base_dir else OVERLAY_DIR
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.book_id = str(book_id)
        self.symbol = str(symbol or "BTCUSDT").strip().upper() or "BTCUSDT"
        self.of_paper_enabled = bool(of_paper_enabled)
        self.volume_pattern_axes = bool(volume_pattern_axes)
        # Per-book Gate 2E mode (PRIMARY allow_only vs BASELINE off). None → env.
        self._pattern_entry_gate_mode_override = pattern_entry_gate_mode
        self._pattern_library_path_override = (
            Path(pattern_library_path) if pattern_library_path else None
        )
        # Per-manager Gate 2X / Adaptive overrides (r50 booking twin). Never mutates global CONFIG.
        self._config_overrides: Dict[str, Any] = dict(config_overrides or {})
        self.state_file = self.base_dir / "overlay_state.json"
        self.events_file = self.base_dir / "trade_plan_events.jsonl"
        self.outcomes_file = self.base_dir / "trade_plan_outcomes.jsonl"
        self.of_outcomes_file = self.base_dir / "of_trade_outcomes.jsonl"
        self.of_opens_file = self.base_dir / "of_paper_trade_opens.jsonl"
        self.of_open_state_file = self.base_dir / "of_paper_trades_open.json"
        # Gate 2E observational thin-WR shadow allows (stamp-only; never opens).
        self.gate2e_shadow_allows_file = self.base_dir / "gate2e_shadow_allows.jsonl"
        # Gate 2E cold-start ABSTAIN_INSUFFICIENT observe (incl. cohort_n=0).
        self.gate2e_shadow_insufficient_file = (
            self.base_dir / "gate2e_shadow_insufficient.jsonl"
        )
        # Gate 2E rejected-trade CF ledger (PRIMARY standaside → twin PnL join).
        self.gate2e_rejected_cf_file = self.base_dir / "gate2e_rejected_cf.jsonl"
        self.config_file = self.base_dir / "overlay_config.json"
        self._save_config_if_missing()
        self._sync_runtime_config_to_disk()
        # Gate 2E B-persist: recent (ts_ms, policy_b_action) for fire-count CF / suppress.
        self._recent_policy_b: List[Tuple[int, str]] = []
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
        if self.of_paper_enabled:
            self._reconstruct_open_of_paper_trade()
            self.reconcile_of_paper_already_closed(persist=True)
            self._repair_last_closed_from_ledger(persist=True)
            # Entry-quality gates apply only at activate-time. Never scrub a booked
            # PAPER open on load (that orphaned OF ENTRY #664 / painted OF EXIT #—).
            self._restore_scrubbed_open_from_journal(persist=True)
        # Past-trade pattern library (Gate 2E entry / Gate 2X exit observational).
        self._pattern_library: Optional[Dict[str, Any]] = None
        self._pattern_library_mtime: float = 0.0
        self._pattern_stop_dirs_by_day: Dict[str, set] = {}
        self._bootstrap_pattern_stop_dirs()

    def _cfg(self, key: str, default: Any = None) -> Any:
        """Instance config: overrides first, then global CONFIG (PRIMARY defaults)."""
        if key in self._config_overrides:
            return self._config_overrides[key]
        return CONFIG.get(key, default)

    def _plan_require_of(self) -> bool:
        """Override `plan_require_of` wins; else RP608_CRYPTO_PLAN_REQUIRE_OF env."""
        if "plan_require_of" in self._config_overrides:
            return bool(self._config_overrides["plan_require_of"])
        return plan_require_of_enabled()

    def _require_adapter_select_15m(self) -> bool:
        """Clean-slate twin only: require Rust 15m soft-score ENTER (config override)."""
        return bool(self._config_overrides.get("require_adapter_select_15m"))

    def _require_sci_adapter_rec(self) -> bool:
        """SCI-ADAPTER-REC P2 twin: S1=b generator (config override; default off)."""
        return bool(self._config_overrides.get("require_sci_adapter_rec"))

    def _require_trend_agree(self) -> bool:
        """B twin only: require L3/L8/L20 15m OHLCV agreement (config override)."""
        return bool(self._config_overrides.get("require_trend_agree"))

    def _b_persist_min_fires(self) -> int:
        if "b_persist_min_fires" in self._config_overrides:
            try:
                return max(0, int(self._config_overrides["b_persist_min_fires"]))
            except (TypeError, ValueError):
                return 0
        return plan_b_persist_min_fires()

    def _note_policy_b(self, now_ms: int, action: str) -> None:
        a = str(action or "").upper()
        if a not in ("LONG", "SHORT"):
            return
        self._recent_policy_b.append((int(now_ms), a))
        window_ms = int(plan_b_persist_window_s() * 1000) * 3  # keep extra headroom
        cutoff = int(now_ms) - window_ms
        self._recent_policy_b = [x for x in self._recent_policy_b if x[0] >= cutoff]

    def _b_persist_stamp(self, now_ms: int, direction: str) -> Dict[str, Any]:
        d = str(direction or "").upper()
        window_s = plan_b_persist_window_s()
        window_ms = int(window_s * 1000)
        cutoff = int(now_ms) - window_ms
        fires = [
            ts
            for ts, act in self._recent_policy_b
            if ts >= cutoff and act == d
        ]
        prior = [ts for ts in fires if ts < int(now_ms)]
        min_fires = self._b_persist_min_fires()
        count = len(fires)
        return {
            "protocol_id": PLAN_B_PERSIST_PROTOCOL_ID,
            "direction": d,
            "window_s": window_s,
            "fires_in_window": count,
            "prior_fires": len(prior),
            "min_fires": min_fires,
            "isolated_oneshot": count <= 1,
            "blocks_entry": bool(min_fires > 0 and count < min_fires),
            "block_reason": (
                "B_PERSIST_INSUFFICIENT_FIRES"
                if (min_fires > 0 and count < min_fires)
                else None
            ),
        }

    def _stamp_gate2x_runtime_on_plan(self, plan: Dict[str, Any]) -> None:
        """Persist twin preset / floor-lock flags on OPEN so state matches Rust tick."""
        preset = self._cfg("gate2x_preset")
        if preset:
            plan["gate2x_preset"] = str(preset).strip()
        if bool(self._cfg("gate2x_floor_lock_immediate")):
            plan["gate2x_floor_lock_immediate"] = True
            plan["gate2x_preset"] = plan.get("gate2x_preset") or "floor_lock_r50"
            plan["gate2x_protocol"] = self._cfg("gate2x_floor_lock_protocol") or (
                "RP-608-GATE2X-FLOOR-LOCK-TWIN-001"
            )
            try:
                plan["gate2x_ratchet_peak_frac"] = float(
                    self._cfg("gate2x_ratchet_peak_frac") or 0.50
                )
            except (TypeError, ValueError):
                plan["gate2x_ratchet_peak_frac"] = 0.50
        elif preset or self._cfg("gate2x_ratchet_peak_frac") is not None:
            try:
                plan["gate2x_ratchet_peak_frac"] = float(
                    self._cfg("gate2x_ratchet_peak_frac")
                    if self._cfg("gate2x_ratchet_peak_frac") is not None
                    else CONFIG.get("gate2x_ratchet_peak_frac")
                    or 0.30
                )
            except (TypeError, ValueError):
                pass
            proto = self._cfg("gate2x_mfe_giveback_protocol")
            if proto:
                plan["gate2x_protocol"] = proto

    def _crypto_pattern_lib(self) -> Dict[str, Any]:
        if self._pattern_library_path_override is not None:
            path = self._pattern_library_path_override
        elif self.volume_pattern_axes:
            path = crypto_pattern_library_path(self.base_dir.parent)
        else:
            path = crypto_baseline_pattern_library_path(self.base_dir.parent)
        try:
            mtime = path.stat().st_mtime if path.exists() else 0.0
        except OSError:
            mtime = 0.0
        if self._pattern_library is not None and mtime == self._pattern_library_mtime:
            return self._pattern_library
        lib = load_crypto_pattern_library(path)
        self._pattern_library = lib
        self._pattern_library_mtime = mtime
        return lib

    def _bootstrap_pattern_stop_dirs(self) -> None:
        """UTC-day × direction stop families (not symbol bans)."""
        self._pattern_stop_dirs_by_day = {}
        if not self.outcomes_file.exists():
            return
        try:
            with open(self.outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        r = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if "STOP" not in str(r.get("exit_reason") or ""):
                        continue
                    et = r.get("entry_time_ms")
                    if not et:
                        continue
                    day = crypto_pattern_session_date_utc(int(et))
                    d = str(r.get("direction") or "UNK").upper()
                    self._pattern_stop_dirs_by_day.setdefault(day, set()).add(d)
        except OSError:
            pass

    def _note_pattern_stop(self, plan: Dict[str, Any]) -> None:
        reason = str(plan.get("exit_reason") or "")
        # Trailing-stop fills are not initial protective-stop losses for pattern day marks.
        if "STOP" not in reason or reason == "TRAILING_STOP_HIT":
            return
        et = plan.get("entry_time_ms")
        if not et:
            return
        day = crypto_pattern_session_date_utc(int(et))
        d = str(plan.get("direction") or "UNK").upper()
        self._pattern_stop_dirs_by_day.setdefault(day, set()).add(d)

    def _note_gate2e_fast_sl_cooldown(self, plan: Dict[str, Any]) -> None:
        """Arm same-direction cooldown after fast SL / path-scratch (Gate 2E)."""
        if not bool(CONFIG.get("gate2e_post_fast_sl_cooldown_enabled")):
            return
        reason = str(plan.get("exit_reason") or "")
        if reason not in ("STOP_LOSS_HIT", "EARLY_SL_PATH_SCRATCH"):
            return
        hold_s = float(plan.get("holding_duration_seconds") or 0.0)
        max_hold = float(CONFIG.get("gate2e_post_fast_sl_max_hold_s") or 120.0)
        if hold_s >= max_hold:
            return
        direction = str(plan.get("direction") or "").upper()
        if direction not in ("LONG", "SHORT"):
            return
        exit_ms = int(plan.get("exit_time_ms") or time.time() * 1000)
        cool_ms = int(CONFIG.get("gate2e_post_fast_sl_cooldown_ms") or 300_000)
        self.state["gate2e_cooldown"] = {
            "direction": direction,
            "armed_ms": exit_ms,
            "until_ms": exit_ms + cool_ms,
            "from_plan_id": plan.get("plan_id"),
            "from_exit_reason": reason,
            "from_hold_s": hold_s,
            "protocol": "RP-608-GATE2E-CRYPTO-FAST-SL-COOLDOWN-001",
        }

    def _gate2e_cooldown_blocks(self, direction: str, now_ms: int) -> Optional[Dict[str, Any]]:
        if not bool(CONFIG.get("gate2e_post_fast_sl_cooldown_enabled")):
            return None
        cool = self.state.get("gate2e_cooldown")
        if not isinstance(cool, dict):
            return None
        until = int(cool.get("until_ms") or 0)
        if until <= 0 or now_ms >= until:
            if cool:
                self.state["gate2e_cooldown"] = None
            return None
        if str(cool.get("direction") or "").upper() != str(direction or "").upper():
            return None
        return cool

    @staticmethod
    def _gate2e_quote_last_px(quote: Optional[Dict[str, Any]]) -> Optional[float]:
        """Tick/last mark for observational path stamp (not forming candle extremes)."""
        q = quote or {}
        for key in ("last_price", "last_trade_price", "mid_price", "price"):
            v = q.get(key)
            if v is not None:
                try:
                    px = float(v)
                except (TypeError, ValueError):
                    continue
                if px > 0:
                    return px
        try:
            bid = q.get("best_bid_price")
            ask = q.get("best_ask_price")
            if bid is not None and ask is not None:
                b = float(bid)
                a = float(ask)
                if b > 0 and a > 0:
                    return (b + a) / 2.0
            if bid is not None and float(bid) > 0:
                return float(bid)
            if ask is not None and float(ask) > 0:
                return float(ask)
        except (TypeError, ValueError):
            return None
        return None

    def _gate2e_path_already_invalid(
        self, plan: Dict[str, Any], quote: Optional[Dict[str, Any]]
    ) -> Optional[str]:
        """Reject open if forming/path extremes already through proposed stop."""
        if not bool(CONFIG.get("gate2e_preentry_path_invalid_enabled")):
            return None
        q = quote or {}
        direction = str(plan.get("direction") or "")
        try:
            stop = float(plan.get("stop_loss_price") or plan.get("invalidation_price") or 0.0)
        except (TypeError, ValueError):
            return None
        if stop <= 0:
            return None
        high = q.get("high_price")
        low = q.get("low_price")
        try:
            if direction == "SHORT" and high is not None and float(high) >= stop:
                return "FORMING_HIGH_THROUGH_STOP"
            if direction == "LONG" and low is not None and float(low) <= stop:
                return "FORMING_LOW_THROUGH_STOP"
        except (TypeError, ValueError):
            return None
        return None

    def _gate2e_tick_last_path_invalid(
        self, plan: Dict[str, Any], quote: Optional[Dict[str, Any]]
    ) -> Optional[str]:
        """Observational: would tick/last mark alone be through proposed stop?"""
        direction = str(plan.get("direction") or "")
        try:
            stop = float(plan.get("stop_loss_price") or plan.get("invalidation_price") or 0.0)
        except (TypeError, ValueError):
            return None
        if stop <= 0:
            return None
        last_px = self._gate2e_quote_last_px(quote)
        if last_px is None:
            return None
        try:
            if direction == "SHORT" and last_px >= stop:
                return "TICK_LAST_THROUGH_STOP"
            if direction == "LONG" and last_px <= stop:
                return "TICK_LAST_THROUGH_STOP"
        except (TypeError, ValueError):
            return None
        return None

    def _gate2e_path_dual_stamp(
        self, plan: Dict[str, Any], quote: Optional[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """
        Compare forming-H/L (live authority) vs tick/last path.
        Observational only — does not change open/suppress authority.
        """
        forming = self._gate2e_path_already_invalid(plan, quote)
        tick = self._gate2e_tick_last_path_invalid(plan, quote)
        forming_blocks = forming is not None
        tick_blocks = tick is not None
        diverge = forming_blocks != tick_blocks
        if forming_blocks and not tick_blocks:
            diverge_kind = "FORMING_BLOCKS_TICK_ALLOWS"
        elif tick_blocks and not forming_blocks:
            diverge_kind = "TICK_BLOCKS_FORMING_ALLOWS"
        else:
            diverge_kind = None
        q = quote or {}
        return {
            "protocol_id": "RP-608-GATE2E-PATH-DUAL-STAMP-001",
            "live_authority": "FORMING_HL",
            "observational": True,
            "live_order_authority": 0.0,
            "forming_path_invalid": forming,
            "tick_path_invalid": tick,
            "forming_blocks": forming_blocks,
            "tick_blocks": tick_blocks,
            "diverge": diverge,
            "diverge_kind": diverge_kind,
            "tick_last_px": self._gate2e_quote_last_px(quote),
            "high_price": q.get("high_price"),
            "low_price": q.get("low_price"),
            "stop_loss_price": plan.get("stop_loss_price"),
            "book_id": self.book_id,
        }

    def _maybe_log_gate2e_path_dual_stamp(
        self, plan: Dict[str, Any], quote: Optional[Dict[str, Any]]
    ) -> Dict[str, Any]:
        stamp = self._gate2e_path_dual_stamp(plan, quote)
        if not bool(CONFIG.get("gate2e_path_dual_stamp_observational")):
            return stamp
        if stamp.get("diverge"):
            summ = self.state.setdefault("outcomes_summary", {})
            summ["gate2e_path_dual_stamp_divergences"] = int(
                summ.get("gate2e_path_dual_stamp_divergences") or 0
            ) + 1
            self._log_event("GATE2E_PATH_DUAL_STAMP", plan, stamp)
        return stamp

    @staticmethod
    def _favorable_excursion_usdt(
        direction: str, entry_price: float, qty: float, fav_exec_px: Optional[float]
    ) -> Optional[float]:
        if fav_exec_px is None or qty <= 0:
            return None
        try:
            e = float(entry_price)
            px = float(fav_exec_px)
        except (TypeError, ValueError):
            return None
        if direction == "LONG":
            return (px - e) * qty
        return (e - px) * qty

    def _gate2x_mfe_arm_threshold_usdt(self, plan: Dict[str, Any]) -> float:
        """Peak MFE must clear round-trip friction + meaningful-movement floor."""
        notional = float(
            plan.get("simulated_notional_usdt") or CONFIG.get("simulated_notional_usdt") or 1000.0
        )
        rt_bps = float(CONFIG.get("round_trip_friction_bps") or 12.0)
        floor_bps = float(self._cfg("gate2x_mfe_meaningful_floor_bps") or 5.0)
        friction = notional * (rt_bps / 10000.0)
        floor = notional * (floor_bps / 10000.0)
        return max(friction, floor)

    @staticmethod
    def _gate2x_peak_tp_frac(
        direction: str, entry_price: float, tp_price: Optional[float], mfe_exec_px: Optional[float]
    ) -> Optional[float]:
        if tp_price is None or mfe_exec_px is None:
            return None
        try:
            e = float(entry_price)
            tp = float(tp_price)
            mfe_px = float(mfe_exec_px)
        except (TypeError, ValueError):
            return None
        if direction == "SHORT":
            denom = e - tp
            if denom <= 0:
                return None
            return (e - mfe_px) / denom
        denom = tp - e
        if denom <= 0:
            return None
        return (mfe_px - e) / denom

    def _try_rust_plan_lifecycle(
        self,
        plan: Dict[str, Any],
        q: Dict[str, Any],
        *,
        now_ms: int,
        bid: float,
        ask: float,
        low: float,
        high: float,
    ) -> Optional[Dict[str, Any]]:
        """Rust Adaptive TP + Gate 2X + SL/TP authority. Returns closed plan or None."""
        if not plan_lifecycle_enabled():
            if plan_lifecycle_require_rust():
                plan_lifecycle_log_require_rust(
                    "crypto",
                    "RP608_PLAN_LIFECYCLE disabled while REQUIRE_RUST=1",
                )
            return None
        sess = plan_lifecycle_session(book="crypto")
        if sess is None:
            if plan_lifecycle_require_rust():
                plan_lifecycle_log_require_rust(
                    "crypto",
                    "lifecycle bin missing or lacks --plan-lifecycle-loop",
                )
            return None
        # Persist twin preset onto long-lived OPEN plans opened before stamp landed.
        if bool(self._cfg("gate2x_floor_lock_immediate")) and not plan.get(
            "gate2x_floor_lock_immediate"
        ):
            self._stamp_gate2x_runtime_on_plan(plan)
        elif self._cfg("gate2x_preset") and not plan.get("gate2x_preset"):
            self._stamp_gate2x_runtime_on_plan(plan)

        plan_for_rust = dict(plan)
        # R4: named Rust preset (preferred). Legacy field overrides kept as fallback.
        preset = self._cfg("gate2x_preset")
        if preset:
            plan_for_rust["gate2x_preset"] = str(preset).strip()
        elif bool(self._cfg("gate2x_floor_lock_immediate")):
            plan_for_rust["gate2x_preset"] = "floor_lock_r50"
            plan_for_rust["gate2x_floor_lock_immediate"] = True
            try:
                plan_for_rust["gate2x_ratchet_peak_frac"] = float(
                    self._cfg("gate2x_ratchet_peak_frac") or 0.50
                )
            except (TypeError, ValueError):
                plan_for_rust["gate2x_ratchet_peak_frac"] = 0.50
            plan_for_rust["gate2x_protocol"] = (
                self._cfg("gate2x_floor_lock_protocol")
                or "RP-608-GATE2X-FLOOR-LOCK-TWIN-001"
            )
        else:
            try:
                r = float(self._cfg("gate2x_ratchet_peak_frac") or 0.30)
            except (TypeError, ValueError):
                r = 0.30
            if abs(r - 0.50) < 1e-9:
                plan_for_rust["gate2x_preset"] = "r50_booking"
                plan_for_rust["gate2x_ratchet_peak_frac"] = r
                plan_for_rust["gate2x_protocol"] = (
                    self._cfg("gate2x_r50_booking_protocol")
                    or self._cfg("gate2x_mfe_giveback_protocol")
                    or "RP-608-GATE2X-R50-BOOKING-TWIN-001"
                )
            else:
                plan_for_rust["gate2x_preset"] = "baseline_r30"
        rust_plan = plan_from_overlay(plan_for_rust, book="crypto")
        quote = {
            "best_bid_price": bid,
            "best_ask_price": ask,
            "low_price": low,
            "high_price": high,
            "now_ms": now_ms,
            "vol_surge": plan.get("exit_vol_surge") or plan.get("vol_surge"),
        }
        result = sess.tick(rust_plan, quote)
        if not result or not isinstance(result, dict):
            if plan_lifecycle_require_rust():
                plan_lifecycle_log_require_rust(
                    "crypto",
                    "plan-lifecycle-loop tick failed",
                )
            return None
        merged = merge_lifecycle_into_plan(plan, result, book="crypto")
        self.state["active_plan"] = merged
        if not result.get("closed"):
            # Persist Adaptive TP / Gate2X / trail / excursion fields from Rust hold.
            # Marks/MFE must survive daemon periodic disk reload (same plan_id OPEN).
            _hold_keys = (
                "tp_level",
                "stop_loss_price",
                "profit_target_price",
                "gate2x_mfe_armed",
                "trailing_sl_armed",
                "adaptive_tp_armed",
                "mfe_exec_price",
                "mae_exec_price",
                "gate2x_peak_mfe_usdt",
                "gate2x_peak_tp_frac",
                "last_mark_price",
                "last_mark_time_ms",
            )
            if any(merged.get(k) != plan.get(k) for k in _hold_keys):
                self._save_state()
            return None  # hold — caller skips Python lifecycle when authority set

        direction = str(merged.get("direction") or "SHORT").upper()
        entry_price = float(merged["entry_price"])
        qty = float(merged["simulated_quantity"])
        entry_ms = int(merged.get("entry_time_ms") or 0)
        terminal_state, exit_reason = map_lifecycle_close(result)
        _fallback_px = ask if direction == "SHORT" else bid
        exit_price = float(
            result["exit_price"] if result.get("exit_price") is not None else _fallback_px
        )
        merged["state"] = terminal_state
        merged["exit_price"] = round_tick(exit_price)
        merged["exit_time_ms"] = now_ms
        merged["holding_duration_seconds"] = round((now_ms - entry_ms) / 1000.0, 1)
        if result.get("exit_reason_detail"):
            merged["exit_reason_detail"] = result["exit_reason_detail"]
        if terminal_state == "CLOSED_GIVEBACK":
            if bool(self._cfg("gate2x_floor_lock_immediate")):
                merged["gate2x_protocol"] = self._cfg("gate2x_floor_lock_protocol") or (
                    self._cfg("gate2x_mfe_giveback_protocol")
                )
            else:
                merged["gate2x_protocol"] = self._cfg("gate2x_mfe_giveback_protocol")
            merged["gate2x_ratchet_peak_frac"] = float(
                self._cfg("gate2x_ratchet_peak_frac") or 0.30
            )
        net_pnl = result.get("net_pnl")
        if net_pnl is None:
            if direction == "LONG":
                gross_pnl = (merged["exit_price"] - entry_price) * qty
            else:
                gross_pnl = (entry_price - merged["exit_price"]) * qty
            friction = (merged["simulated_notional_usdt"] + (merged["exit_price"] * qty)) * (
                6.0 / 10000.0
            )
            net_pnl = round(gross_pnl - friction, 2)
            merged["gross_pnl_usdt"] = round(gross_pnl, 2)
            merged["friction_usdt"] = round(friction, 4)
        else:
            net_pnl = round(float(net_pnl), 2)
            if direction == "LONG":
                gross_pnl = (merged["exit_price"] - entry_price) * qty
            else:
                gross_pnl = (entry_price - merged["exit_price"]) * qty
            merged["gross_pnl_usdt"] = round(gross_pnl, 2)
            friction = (merged["simulated_notional_usdt"] + (merged["exit_price"] * qty)) * (
                6.0 / 10000.0
            )
            merged["friction_usdt"] = round(friction, 4)
        merged["net_pnl_usdt"] = net_pnl
        merged["exit_reason"] = exit_reason

        self.state["closed_plans_count"] += 1
        summ = self.state.setdefault("outcomes_summary", {})
        summ["cumulative_net_pnl_usdt"] = float(
            summ.get("cumulative_net_pnl_usdt") or 0.0
        ) + net_pnl
        if terminal_state == "CLOSED_TP":
            summ["tp_hits"] = int(summ.get("tp_hits") or 0) + 1
        elif terminal_state == "CLOSED_TRAIL":
            summ["trail_hits"] = int(summ.get("trail_hits") or 0) + 1
        elif terminal_state == "CLOSED_SL":
            summ["sl_hits"] = int(summ.get("sl_hits") or 0) + 1
        elif terminal_state == "TIME_EXIT":
            summ["time_exits"] = int(summ.get("time_exits") or 0) + 1
        elif terminal_state == "CLOSED_GIVEBACK":
            summ["gate2x_mfe_giveback_exits"] = int(
                summ.get("gate2x_mfe_giveback_exits") or 0
            ) + 1
        elif terminal_state == "CLOSED_SCRATCH":
            summ["early_sl_path_scratches"] = int(
                summ.get("early_sl_path_scratches") or 0
            ) + 1
        self.state["last_closed_plan"] = dict(merged)
        self.state["active_plan"] = None
        self._log_event(
            terminal_state,
            merged,
            {
                "exit_price": merged["exit_price"],
                "net_pnl": net_pnl,
                "reason": exit_reason,
                "authority": "rust_rp608_plan_lifecycle",
            },
        )
        self._log_outcome(merged)
        try:
            self._note_pattern_stop(merged)
        except Exception:
            pass
        try:
            self._note_gate2e_fast_sl_cooldown(merged)
        except Exception:
            pass
        self._save_state()
        return merged

    def _adaptive_tp_try_extend(
        self,
        plan: Dict[str, Any],
        *,
        direction: str,
        hit_tp: bool,
        now_ms: int,
        bid: float,
        ask: float,
        low: float,
        high: float,
    ) -> bool:
        """
        Gate 5 Adaptive: Coralys floor trail (not TP latch / not blind ATR).

        Ratchets SL to Gate2X protect|ratchet floors. `hit_tp` ignored.
        Parity with Rust `apply_adaptive_coralys_floor_trail`.
        Track A: no-op when PLAN_LIFECYCLE=1 (Rust owns Adaptive).
        """
        if plan_lifecycle_enabled():
            return False
        _ = (hit_tp, low, high, bid, ask)
        if not bool(CONFIG.get("adaptive_tp_enabled")):
            return False
        try:
            entry = float(plan["entry_price"])
            sl = float(plan["stop_loss_price"])
            qty = float(plan.get("simulated_quantity") or 0.0)
        except (TypeError, ValueError, KeyError):
            return False
        orig = plan.get("original_stop_loss_price")
        try:
            orig_f = float(orig) if orig is not None else sl
        except (TypeError, ValueError):
            orig_f = sl
        plan["original_stop_loss_price"] = orig_f
        if entry <= 0 or qty <= 0:
            return False
        mfe_px = plan.get("mfe_exec_price")
        try:
            mfe_f = float(mfe_px) if mfe_px is not None else entry
        except (TypeError, ValueError):
            mfe_f = entry

        floor = plan.get("gate2x_protect_floor_usdt")
        if floor is None:
            floor = plan.get("gate2x_ratchet_floor_usdt")
        armed = bool(plan.get("gate2x_mfe_armed") or plan.get("adaptive_tp_armed"))
        proposed = None
        if armed and floor is not None:
            try:
                floor_f = float(floor)
            except (TypeError, ValueError):
                floor_f = 0.0
            if floor_f > 0:
                if direction == "LONG":
                    proposed = round_tick(max(orig_f, entry + floor_f / qty))
                else:
                    proposed = round_tick(min(orig_f, entry - floor_f / qty))

        if proposed is None and bool(CONFIG.get("adaptive_dumb_atr_trail")):
            atr = float(plan.get("atr_14") or 0.0)
            risk = (entry - orig_f) if direction == "LONG" else (orig_f - entry)
            fav = (mfe_f - entry) if direction == "LONG" else (entry - mfe_f)
            arm = risk * float(CONFIG.get("trailing_sl_arm_risk_frac") or 0.30)
            if risk > 0 and fav >= arm:
                trail_dist = (
                    float(CONFIG.get("trailing_sl_atr_mult") or 0.5) * atr
                    if atr > 0
                    else risk * 0.5
                )
                if trail_dist > 0:
                    if direction == "LONG":
                        proposed = round_tick(max(orig_f, mfe_f - trail_dist))
                    else:
                        proposed = round_tick(min(orig_f, mfe_f + trail_dist))

        if proposed is None:
            if armed:
                plan["adaptive_tp_armed"] = True
            return False
        if direction == "LONG":
            if proposed <= sl or proposed >= mfe_f:
                return False
        elif proposed >= sl or proposed <= mfe_f:
            return False
        plan["stop_loss_price"] = proposed
        plan["adaptive_tp_armed"] = True
        plan["trailing_sl_armed"] = True
        plan["adaptive_partial_latch_armed"] = False
        plan["adaptive_tp_touch_ms"] = now_ms
        plan["adaptive_tp_protocol"] = CONFIG.get("adaptive_tp_protocol")
        self.state["active_plan"] = plan
        self._save_state()
        return True

    def _gate2x_update_mfe_giveback(
        self, plan: Dict[str, Any], *, exec_px: float, now_ms: Optional[int] = None
    ) -> Optional[str]:
        """
        Gate 2X feature construction + adapter decide (Coralys via chronosentiment).

        Python tracks peak MFE / tp_frac only; Hold|Protect|Exit comes from
        `rp608_position_decide` → `gate2x_decide` → RecommendationEngineV1.
        Track A: no-op when PLAN_LIFECYCLE=1 (Rust owns Gate2X manage).
        """
        if plan_lifecycle_enabled():
            return None
        if not bool(self._cfg("gate2x_mfe_giveback_protect_enabled")):
            return None
        direction = str(plan.get("direction") or "")
        try:
            entry_price = float(plan.get("entry_price") or 0.0)
            qty = float(plan.get("simulated_quantity") or 0.0)
        except (TypeError, ValueError):
            return None
        if entry_price <= 0 or qty <= 0 or direction not in ("LONG", "SHORT"):
            return None

        peak_mfe = self._favorable_excursion_usdt(
            direction, entry_price, qty, plan.get("mfe_exec_price")
        )
        if peak_mfe is None:
            return None
        prev_peak = plan.get("gate2x_peak_mfe_usdt")
        try:
            prev_peak_f = float(prev_peak) if prev_peak is not None else None
        except (TypeError, ValueError):
            prev_peak_f = None
        if prev_peak_f is None or peak_mfe > prev_peak_f:
            plan["gate2x_peak_mfe_usdt"] = round(peak_mfe, 4)

        tp_frac = self._gate2x_peak_tp_frac(
            direction,
            entry_price,
            plan.get("profit_target_price"),
            plan.get("mfe_exec_price"),
        )
        if tp_frac is not None:
            prev_tf = plan.get("gate2x_peak_tp_frac")
            try:
                prev_tf_f = float(prev_tf) if prev_tf is not None else None
            except (TypeError, ValueError):
                prev_tf_f = None
            if prev_tf_f is None or tp_frac > prev_tf_f:
                plan["gate2x_peak_tp_frac"] = round(tp_frac, 4)

        threshold = self._gate2x_mfe_arm_threshold_usdt(plan)
        plan.setdefault("gate2x_mfe_armed_threshold_usdt", round(threshold, 4))

        try:
            entry_ms = int(plan.get("entry_time_ms") or 0)
            ts = int(now_ms if now_ms is not None else time.time() * 1000)
            hold_s = (ts - entry_ms) / 1000.0 if entry_ms > 0 else 0.0
        except (TypeError, ValueError):
            hold_s = 0.0
            ts = int(time.time() * 1000)

        cur_mfe = self._favorable_excursion_usdt(direction, entry_price, qty, exec_px)
        if cur_mfe is None:
            return None
        plan["gate2x_current_mfe_usdt"] = round(cur_mfe, 4)

        # R3: floor-lock Exit lives in Rust gate2x_decide (floor_lock_immediate).
        # Python giveback path is PRIMARY/BASELINE/r50 fallback only.

        since = plan.get("gate2x_soft_breach_since_ms")
        try:
            since_i = int(since) if since is not None else None
        except (TypeError, ValueError):
            since_i = None

        protocol = str(
            self._cfg("gate2x_mfe_giveback_protocol")
            or "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001"
        )
        try:
            tp_level_i = int(plan.get("tp_level") or 0)
        except (TypeError, ValueError):
            tp_level_i = 0
        snap = gate2x_build_snapshot(
            decision_id=str(plan.get("plan_id") or plan.get("decision_id") or "CRYPTO"),
            instrument=str(plan.get("symbol") or plan.get("instrument") or "BTCUSDT"),
            direction=direction,
            reference_price=entry_price,
            peak_mfe=float(plan.get("gate2x_peak_mfe_usdt") or peak_mfe),
            current_mfe=float(cur_mfe),
            peak_tp_frac=float(plan.get("gate2x_peak_tp_frac") or 0.0),
            hold_s=hold_s,
            arm_threshold=threshold,
            already_armed=bool(plan.get("gate2x_mfe_armed")),
            vol_surge=str(
                plan.get("exit_vol_surge") or plan.get("vol_surge") or "UNK"
            ),
            soft_breach_since_ms=since_i,
            now_ms=ts,
            protocol=protocol,
            tp_level=tp_level_i,
            adaptive_armed=bool(plan.get("adaptive_tp_armed")) or tp_level_i >= 1,
            params={
                "enabled": True,
                "arm_min_tp_frac": float(self._cfg("gate2x_arm_min_tp_frac") or 0.40),
                "min_hold_s_before_exit": float(
                    self._cfg("gate2x_min_hold_s_before_exit") or 120.0
                ),
                "meaningful_floor_bps": float(
                    self._cfg("gate2x_mfe_meaningful_floor_bps") or 5.0
                ),
                "protect_floor": str(self._cfg("gate2x_protect_floor") or "COST_CLEAR"),
                "reversal_volume_confirm": bool(
                    self._cfg("gate2x_reversal_volume_confirm_enabled")
                ),
                "immediate_vol_surges": list(
                    self._cfg("gate2x_reversal_immediate_vol_surges") or ["HIGH"]
                ),
                "med_vol_dwell_ms": int(
                    self._cfg("gate2x_reversal_med_vol_dwell_ms") or 40_000
                ),
                "low_vol_dwell_ms": int(
                    self._cfg("gate2x_reversal_low_vol_dwell_ms") or 90_000
                ),
                "ratchet_peak_frac": float(self._cfg("gate2x_ratchet_peak_frac") or 0.30),
                "deep_giveback_frac": float(self._cfg("gate2x_deep_giveback_frac") or 0.50),
                "deep_giveback_dwell_scale": float(
                    self._cfg("gate2x_deep_giveback_dwell_scale") or 0.40
                ),
            },
        )
        evidence = os.environ.get(
            "RP608_CRYPTO_REC001H_DIR", DEFAULT_CRYPTO_EVIDENCE
        )
        action, reason, soft = gate2x_adapter_decide(snap, evidence_dir=evidence)
        apply_soft_breach_to_trade(plan, soft)
        if soft.get("protect_floor") is not None:
            plan["gate2x_protect_floor_usdt"] = round(float(soft["protect_floor"]), 4)
        if soft.get("ratchet_floor") is not None:
            plan["gate2x_ratchet_floor_usdt"] = round(float(soft["ratchet_floor"]), 4)
        if soft.get("armed") and not plan.get("gate2x_mfe_armed"):
            plan["gate2x_mfe_armed"] = True
            plan["gate2x_mfe_armed_threshold_usdt"] = round(threshold, 4)
            plan["gate2x_mfe_armed_tp_frac"] = round(
                float(plan.get("gate2x_peak_tp_frac") or 0.0), 4
            )

        if action in ("Protect", "Exit"):
            detail = reason or "MFE_GIVEBACK_TO_COST_CLEAR"
            plan["exit_reason_detail"] = detail
            plan["gate2x_adapter_action"] = action
            plan["gate2x_adapter_authority"] = "chronosentiment_adapter+coralys"
            return detail
        return None

    def _pattern_entry_gate_mode(self) -> str:
        if self._pattern_entry_gate_mode_override is not None:
            return crypto_normalize_entry_gate_mode(
                self._pattern_entry_gate_mode_override
            )
        return crypto_pattern_entry_gate_mode()

    def _match_entry_pattern(
        self, card: Dict[str, Any], plan_preview: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        et = int(
            (plan_preview or {}).get("entry_time_ms")
            or card.get("close_time_ms")
            or time.time() * 1000
        )
        day = crypto_pattern_session_date_utc(et)
        direction = str(
            (plan_preview or {}).get("direction")
            or card.get("policy_b_action")
            or "UNK"
        ).upper()
        prior = direction in self._pattern_stop_dirs_by_day.get(day, set())
        feats = crypto_pattern_features(
            card=card,
            plan=plan_preview,
            prior_stop_same_utc_day_dir=prior,
        )
        return match_crypto_pattern_entry(
            library=self._crypto_pattern_lib(),
            feats=feats,
            include_volume=self.volume_pattern_axes,
            gate_mode=self._pattern_entry_gate_mode(),
        )

    def _save_config_if_missing(self):
        if not self.config_file.exists():
            with open(self.config_file, "w", encoding="utf-8") as f:
                json.dump(CONFIG, f, indent=2)

    def _sync_runtime_config_to_disk(self) -> None:
        """
        Merge instance overrides into overlay_config.json so ops/readers see the
        live twin preset (floor_lock_immediate, ratchet, gate2x_preset) — not stale
        PRIMARY defaults left by a first-write of global CONFIG.
        """
        if not self._config_overrides:
            return
        try:
            disk: Dict[str, Any] = {}
            if self.config_file.exists():
                with open(self.config_file, "r", encoding="utf-8") as f:
                    loaded = json.load(f)
                if isinstance(loaded, dict):
                    disk = loaded
            merged = dict(CONFIG)
            merged.update(disk)
            merged.update(self._config_overrides)
            with open(self.config_file, "w", encoding="utf-8") as f:
                json.dump(merged, f, indent=2)
        except Exception as exc:
            print(f"[overlay_config] sync overrides skipped: {exc}", flush=True)

    def reload_state_from_disk(self, *, preserve_open_progress: bool = True) -> None:
        """
        Reload overlay JSON from disk without clobbering in-memory OPEN mark/MFE.

        Daemon bookTicker path reloads every few seconds so external readers see
        durable state — but hold-tick marks were often not yet flushed. Replacing
        an identical OPEN plan_id with a staler disk copy resets Gate2X peaks and
        last_mark (Track A soak fidelity). Keep memory plan when plan_id matches.
        """
        mem_plan = (self.state or {}).get("active_plan")
        disk = self._load_state()
        disk_plan = disk.get("active_plan") if isinstance(disk, dict) else None
        if (
            preserve_open_progress
            and isinstance(mem_plan, dict)
            and mem_plan.get("state") == "OPEN"
            and isinstance(disk_plan, dict)
            and disk_plan.get("state") == "OPEN"
            and disk_plan.get("plan_id") == mem_plan.get("plan_id")
        ):
            disk["active_plan"] = mem_plan
        self.state = disk

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
            "gate2e_cooldown": None,
            "outcomes_summary": {
                "tp_hits": 0,
                "sl_hits": 0,
                "time_exits": 0,
                "ambiguous_hits": 0,
                "early_sl_path_scratches": 0,
                "gate2e_cooldown_suppressions": 0,
                "gate2e_path_invalid_suppressions": 0,
                "gate2e_pattern_suppressions": 0,
                "gate2e_path_dual_stamp_divergences": 0,
                "gate2x_mfe_giveback_exits": 0,
                "cumulative_net_pnl_usdt": 0.0
            },
            "of_outcomes_summary": {
                "n_trades": 0,
                "n_wins": 0,
                "n_losses": 0,
                "cumulative_gross_pnl_usdt": 0.0,
                "cumulative_net_pnl_usdt": 0.0,
                "cumulative_friction_usdt": 0.0,
                "of_entry_quality_suppressions": 0,
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

    def _of_entry_persist_ok(
        self, *, direction: str, now_ms: int
    ) -> Tuple[bool, str, Dict[str, Any]]:
        """
        Same-side OF must persist continuously before paper open (mirror exit persist).
        Does not change ±0.30 classify — only paper booking.
        """
        persist = int(CONFIG.get("of_paper_entry_persist_ms", 30_000) or 0)
        direction_u = str(direction or "").upper()
        arm = self.state.get("of_entry_persist_arm")
        if not isinstance(arm, dict):
            arm = {}
        if persist <= 0:
            return True, "ok", {"persist_ms": 0, "armed_ms": now_ms, "direction": direction_u}
        if str(arm.get("direction") or "") != direction_u or arm.get("seen_ms") is None:
            self.state["of_entry_persist_arm"] = {
                "direction": direction_u,
                "seen_ms": now_ms,
            }
            return (
                False,
                f"entry_persist_arming_need_{persist}ms",
                {"persist_ms": persist, "armed_ms": now_ms, "direction": direction_u},
            )
        try:
            seen_i = int(arm["seen_ms"])
        except (TypeError, ValueError):
            self.state["of_entry_persist_arm"] = {
                "direction": direction_u,
                "seen_ms": now_ms,
            }
            return False, "entry_persist_rearm", {"persist_ms": persist, "armed_ms": now_ms}
        held = now_ms - seen_i
        if held < persist:
            return (
                False,
                f"entry_persist_{held}ms<{persist}ms",
                {
                    "persist_ms": persist,
                    "armed_ms": seen_i,
                    "held_ms": held,
                    "direction": direction_u,
                },
            )
        return (
            True,
            "ok",
            {
                "persist_ms": persist,
                "armed_ms": seen_i,
                "held_ms": held,
                "direction": direction_u,
            },
        )

    def _clear_of_entry_persist_arm(self) -> None:
        self.state.pop("of_entry_persist_arm", None)

    def _of_paper_entry_quality_ok(
        self,
        *,
        size: Dict[str, Any],
        depth_q: Dict[str, Any],
        persist_meta: Optional[Dict[str, Any]] = None,
        persist_ok: bool = True,
        persist_reason: str = "ok",
    ) -> Tuple[bool, str, Dict[str, Any]]:
        """
        PAPER-only open filter. Does not change OF ±0.30 classification —
        only whether a paper trade is booked against a confirmed OF latch.
        """
        depth_ok = True
        depth_reason = "ok"
        if CONFIG.get("of_paper_reject_thin_opposite", True) and depth_q.get(
            "thin_opposite_diagnostic"
        ):
            depth_ok = False
            depth_reason = "thin_opposite_diagnostic"
        min_sig = float(CONFIG.get("of_paper_min_entry_signal_qty", 0.0) or 0.0)
        try:
            sig = float(size.get("entry_signal_qty") or 0.0)
        except (TypeError, ValueError):
            sig = 0.0
        if depth_ok and min_sig > 0.0 and sig < min_sig:
            depth_ok = False
            depth_reason = f"entry_signal_qty_{sig}<{min_sig}"
        min_same = float(CONFIG.get("of_paper_min_same_side_notional_usdt", 0.0) or 0.0)
        try:
            same_n = float(depth_q.get("same_side_touch_notional_usdt") or 0.0)
        except (TypeError, ValueError):
            same_n = 0.0
        if depth_ok and min_same > 0.0 and same_n < min_same:
            depth_ok = False
            depth_reason = f"same_side_notional_{same_n}<{min_same}"
        min_comb = float(
            CONFIG.get("of_paper_min_combined_touch_notional_usdt", 0.0) or 0.0
        )
        try:
            comb_n = float(depth_q.get("combined_touch_notional_usdt") or 0.0)
        except (TypeError, ValueError):
            comb_n = 0.0
        if depth_ok and min_comb > 0.0 and comb_n < min_comb:
            depth_ok = False
            depth_reason = f"combined_touch_notional_{comb_n}<{min_comb}"

        ok = bool(persist_ok and depth_ok)
        reason = "ok"
        if not persist_ok:
            reason = persist_reason
        elif not depth_ok:
            reason = depth_reason
        stamp = {
            "protocol_id": OF_ENTRY_QUALITY_PROTOCOL_ID,
            "persist_ok": bool(persist_ok),
            "depth_ok": bool(depth_ok),
            "ok": ok,
            "reason": reason,
            "persist": persist_meta or {},
            "gate_enabled": of_paper_quality_gate_enabled(),
        }
        return ok, reason, stamp

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

        # Gate 2E regime-align on OF paper opens (PRIMARY; ±0.30 mark unchanged).
        of_regime_card = {}
        if isinstance(candle_mark, dict):
            of_regime_card = candle_mark
        # Prefer last decision-card regime fields parked on state when present.
        parked = self.state.get("last_decision_regime")
        if isinstance(parked, dict):
            of_regime_card = {**of_regime_card, **parked}
        of_align = evaluate_regime_align(
            direction=str(direction).upper(),
            regime=regime_dict_from_card(of_regime_card),
            book_id=self.book_id,
        )
        if of_align.get("blocks_entry"):
            summ = self.state.setdefault("of_outcomes_summary", {})
            summ["gate2e_regime_align_suppressions"] = int(
                summ.get("gate2e_regime_align_suppressions") or 0
            ) + 1
            guard = str(of_align.get("guard") or "DIRECTIONAL")
            if guard == "DI_SKEW":
                summ["gate2e_di_skew_suppressions"] = int(
                    summ.get("gate2e_di_skew_suppressions") or 0
                ) + 1
            self.state["last_of_paper_open_suppressed"] = {
                "reason": of_align.get("block_reason"),
                "direction": str(direction).upper(),
                "regime_align": of_align,
                "guard": guard,
                "suppressed_ms": now,
                "would_have_opened": True,
                "protocol_id": of_align.get("protocol_id"),
                "note": (
                    "OF paper open suppressed — adverse DI skew / DIRECTIONAL; "
                    "OF ±0.30 unchanged"
                ),
            }
            self._log_event(
                "OF_PAPER_OPEN_SUPPRESSED_DI_SKEW"
                if guard == "DI_SKEW"
                else "OF_PAPER_OPEN_SUPPRESSED_REGIME_ALIGN",
                {
                    "direction": str(direction).upper(),
                    "plan_id": plan_id,
                    "state": "OF_REGIME_ALIGN_SUPPRESSED",
                },
                {"regime_align": of_align, "book_id": self.book_id},
            )
            return existing if isinstance(existing, dict) else None

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
        persist_ok, persist_reason, persist_meta = self._of_entry_persist_ok(
            direction=str(direction).upper(), now_ms=now
        )
        ok_q, deny_q, quality_stamp = self._of_paper_entry_quality_ok(
            size=size,
            depth_q=depth_q,
            persist_meta=persist_meta,
            persist_ok=persist_ok,
            persist_reason=persist_reason,
        )
        gate_on = of_paper_quality_gate_enabled()
        if not ok_q and gate_on:
            suppressed = {
                "reason": deny_q,
                "direction": str(direction).upper(),
                "order_flow_imbalance": of.get("order_flow_imbalance"),
                "entry_signal_qty": size.get("entry_signal_qty"),
                "depth_quality": depth_q,
                "of_entry_quality": quality_stamp,
                "suppressed_ms": now,
                "would_have_opened": True,
                "protocol_id": OF_PAPER_PROTOCOL_ID,
                "note": (
                    "PAPER open suppressed by entry quality — OF ±0.30 mark unchanged; "
                    "no paper trade booked"
                ),
            }
            self.state["last_of_paper_open_suppressed"] = suppressed
            summ = self.state.setdefault("of_outcomes_summary", {})
            summ["of_entry_quality_suppressions"] = int(
                summ.get("of_entry_quality_suppressions") or 0
            ) + 1
            self._log_event(
                "OF_PAPER_OPEN_SUPPRESSED",
                {
                    "direction": str(direction).upper(),
                    "plan_id": plan_id,
                    "state": "OF_QUALITY_SUPPRESSED",
                },
                {
                    "reason": deny_q,
                    "of_entry_quality": quality_stamp,
                    "depth_quality": depth_q,
                    "order_flow_imbalance": of.get("order_flow_imbalance"),
                    "would_have_opened": True,
                    "protocol_id": OF_ENTRY_QUALITY_PROTOCOL_ID,
                },
            )
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
        # Stamp-only mode: still open but record that quality would have blocked.
        open_quality = dict(quality_stamp)
        if not ok_q and not gate_on:
            open_quality["stamp_only_would_suppress"] = True
        rec = {
            "active": True,
            "symbol": self.symbol,
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
            "of_entry_quality": open_quality,
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
                f"+entry_persist_{int(CONFIG.get('of_paper_entry_persist_ms', 30_000))}ms"
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
        self._clear_of_entry_persist_arm()
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

    def continue_of_paper_entry_persist(
        self,
        *,
        quote: Optional[Dict[str, Any]] = None,
        now_ms: Optional[int] = None,
        persist: bool = True,
        plan_id: Optional[str] = None,
        entry_price: Optional[float] = None,
    ) -> Optional[Dict[str, Any]]:
        """
        WS/tick continuation for OF paper entry persist.

        Decision cards arm `of_entry_persist_arm`; live L1 must keep same-side OF
        for of_paper_entry_persist_ms before paper open. Does not change ±0.30.
        """
        if not self.of_paper_enabled:
            return None
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        active = self.state.get("active_order_flow_recommendation")
        if active and active.get("active"):
            return dict(active)
        arm = self.state.get("of_entry_persist_arm")
        if not isinstance(arm, dict):
            return None
        direction_u = str(arm.get("direction") or "").upper()
        if direction_u not in ("LONG", "SHORT"):
            self._clear_of_entry_persist_arm()
            return None
        l1 = l1_from_book_ticker(quote if isinstance(quote, dict) else None)
        of = classify_order_flow_support(direction_u, l1)
        if not of.get("order_flow_supported"):
            self._clear_of_entry_persist_arm()
            if persist:
                self._save_state()
            return None
        px = entry_price
        if px is None and isinstance(quote, dict):
            raw_px = (
                quote.get("best_bid_price")
                if direction_u == "LONG"
                else quote.get("best_ask_price")
            )
            try:
                px = float(raw_px) if raw_px is not None else None
            except (TypeError, ValueError):
                px = None
        rec = self._activate_order_flow_recommendation(
            direction=direction_u,
            of=of,
            plan_id=plan_id,
            entry_price=px,
            now_ms=now,
            candle_mark=build_of_candle_mark(now_ms=now),
            quote=quote if isinstance(quote, dict) else None,
        )
        if (
            isinstance(rec, dict)
            and rec.get("active")
            and rec.get("of_trade_id")
        ):
            self._log_event(
                "ORDER_FLOW_RECOMMENDATION_ACTIVE",
                {
                    "plan_id": plan_id or rec.get("plan_id"),
                    "direction": direction_u,
                    "entry_price": rec.get("entry_price"),
                    "state": "OF_ACTIVE",
                },
                {
                    "order_flow_imbalance": of.get("order_flow_imbalance"),
                    "order_flow_source": of.get("order_flow_source") or "entry_persist_ws",
                    "of_entry_quality": rec.get("of_entry_quality"),
                    "entry_persist_continued": True,
                },
            )
        if persist:
            self._save_state()
        return rec if isinstance(rec, dict) else None

    def _save_state(self):
        # Unique tmp — fixed overlay_state.tmp races when duplicate daemons
        # (or dashboard) write the same book dir (Errno 2 on replace).
        tmp_file = self.state_file.parent / (
            f"{self.state_file.stem}.{os.getpid()}.{time.time_ns()}.tmp"
        )
        try:
            with open(tmp_file, "w", encoding="utf-8") as f:
                json.dump(self.state, f, indent=2)
                f.flush()
                os.fsync(f.fileno())
            os.replace(str(tmp_file), str(self.state_file))
        finally:
            if tmp_file.exists():
                try:
                    tmp_file.unlink()
                except OSError:
                    pass

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

    def _maybe_log_gate2e_shadow_allow(
        self,
        pattern_stamp: Dict[str, Any],
        plan: Dict[str, Any],
        suppressed_ms: int,
    ) -> Optional[Dict[str, Any]]:
        """Stamp-only counterfactual when allow_only suppresses thin-high-WR ABSTAIN.

        Never opens a trade. Does not change live RP608_CRYPTO_PATTERN_GATE.
        Default env ON (RP608_CRYPTO_GATE2E_SHADOW_THIN_WR=1).
        """
        if not gate2e_shadow_thin_wr_enabled():
            return None
        if not matches_shadow_allow_if_n10_wr_gt_035(pattern_stamp):
            return None
        stamp = build_gate2e_shadow_allow_stamp(
            pattern_stamp=pattern_stamp,
            plan_id=plan.get("plan_id"),
            direction=plan.get("direction"),
            suppressed_ms=suppressed_ms,
            book_id=self.book_id,
            plan=plan,
        )
        stamp["timestamp_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        with open(self.gate2e_shadow_allows_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(stamp) + "\n")
        summ = self.state.setdefault("outcomes_summary", {})
        summ["gate2e_shadow_allow_stamps"] = int(
            summ.get("gate2e_shadow_allow_stamps") or 0
        ) + 1
        self.state["last_gate2e_shadow_allow"] = {
            "shadow_rule_id": stamp.get("shadow_rule_id"),
            "pattern_id": stamp.get("pattern_id"),
            "feature_pattern_id": stamp.get("feature_pattern_id"),
            "plan_id": stamp.get("plan_id"),
            "hist_wr": stamp.get("hist_wr"),
            "cohort_n": stamp.get("cohort_n"),
            "suppressed_ms": stamp.get("suppressed_ms"),
            "live_vol_surge": stamp.get("live_vol_surge"),
            "vol_burst_20": stamp.get("vol_burst_20"),
        }
        return stamp

    def _maybe_log_gate2e_shadow_insufficient(
        self,
        pattern_stamp: Dict[str, Any],
        plan: Dict[str, Any],
        suppressed_ms: int,
    ) -> Optional[Dict[str, Any]]:
        """Stamp-only cold-start ABSTAIN_INSUFFICIENT (incl. cohort_n=0).

        Never opens a trade. Does not change live allow_only.
        Default env ON (RP608_GATE2E_SHADOW_INSUFFICIENT=1).
        """
        if not gate2e_shadow_insufficient_enabled():
            return None
        if not matches_shadow_observe_abstain_insufficient(pattern_stamp):
            return None
        _bid = str(self.book_id)
        _crypto_twins = {
            "CRYPTO_PLAN": "CRYPTO_PLAN_BASELINE_PARALLEL",
            "CRYPTO_PLAN_ETHUSDT": "CRYPTO_PLAN_BASELINE_PARALLEL_ETHUSDT",
        }
        twin_book = str(
            getattr(self, "twin_book_id", None)
            or plan.get("twin_book")
            or _crypto_twins.get(_bid)
            or (
                f"{_bid}_BASELINE_PARALLEL"
                if "BASELINE" not in _bid.upper()
                else _bid
            )
        )
        stamp = build_gate2e_shadow_insufficient_stamp(
            pattern_stamp=pattern_stamp,
            book_id=str(self.book_id),
            twin_book=twin_book,
            suppressed_ms=suppressed_ms,
            symbol=plan.get("symbol"),
            direction=plan.get("direction"),
            plan_id=plan.get("plan_id"),
            decision_bar_idx=plan.get("bar_idx") or plan.get("entry_bar_idx"),
            source="live_suppress",
            extra={
                "live_vol_surge": pattern_stamp.get("live_vol_surge")
                or plan.get("vol_surge"),
                "live_vol_burst_20": pattern_stamp.get("live_vol_burst_20")
                if pattern_stamp.get("live_vol_burst_20") is not None
                else plan.get("vol_burst_20"),
                "entry_price": plan.get("entry_price"),
            },
        )
        stamp["timestamp_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        with open(self.gate2e_shadow_insufficient_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(stamp) + "\n")
        summ = self.state.setdefault("outcomes_summary", {})
        summ["gate2e_shadow_insufficient_stamps"] = int(
            summ.get("gate2e_shadow_insufficient_stamps") or 0
        ) + 1
        self.state["last_gate2e_shadow_insufficient"] = {
            "shadow_rule_id": stamp.get("shadow_rule_id"),
            "pattern_id": stamp.get("pattern_id"),
            "plan_id": stamp.get("plan_id"),
            "cohort_n": stamp.get("cohort_n"),
            "suppressed_ms": stamp.get("suppressed_ms"),
        }
        return stamp

    def _log_gate2e_rejected_cf(
        self,
        *,
        reject_reason: str,
        suppressed_ms: int,
        direction: Any = None,
        plan: Optional[Dict[str, Any]] = None,
        detail: Optional[str] = None,
        pattern_stamp: Optional[Dict[str, Any]] = None,
        regime_align: Optional[Dict[str, Any]] = None,
        extra: Optional[Dict[str, Any]] = None,
    ) -> Optional[Dict[str, Any]]:
        """Append PRIMARY reject stamp for later twin CF PnL join (authority 0%)."""
        if not rejected_cf_applies_to_book(self.book_id):
            return None
        plan_d = dict(plan or {})
        if not plan_d.get("symbol"):
            plan_d["symbol"] = self.symbol
        stamp = build_rejected_cf_stamp(
            reject_reason=reject_reason,
            book_id=str(self.book_id),
            suppressed_ms=int(suppressed_ms),
            direction=direction or plan_d.get("direction"),
            plan=plan_d,
            detail=detail,
            pattern_stamp=pattern_stamp,
            regime_align=regime_align,
            extra=extra,
        )
        stamp["timestamp_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        with open(self.gate2e_rejected_cf_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(stamp) + "\n")
        summ = self.state.setdefault("outcomes_summary", {})
        summ["gate2e_rejected_cf_stamps"] = int(
            summ.get("gate2e_rejected_cf_stamps") or 0
        ) + 1
        by_reason = summ.setdefault("gate2e_rejected_cf_by_reason", {})
        rr = str(stamp.get("reject_reason") or "?")
        by_reason[rr] = int(by_reason.get(rr) or 0) + 1
        self.state["last_gate2e_rejected_cf"] = {
            "reject_reason": stamp.get("reject_reason"),
            "direction": stamp.get("direction"),
            "plan_id": stamp.get("plan_id"),
            "suppressed_ms": stamp.get("suppressed_ms"),
            "detail": stamp.get("detail") or stamp.get("block_reason"),
            "twin_book": stamp.get("twin_book"),
        }
        return stamp

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

    @staticmethod
    def _plan_outcome_net_bps(row: Dict[str, Any]) -> Optional[float]:
        if row.get("net_return_bps") is not None:
            try:
                return float(row["net_return_bps"])
            except (TypeError, ValueError):
                pass
        try:
            net = float(row.get("net_pnl_usdt") or 0.0)
            notional = float(row.get("simulated_notional_usdt") or 1000.0)
            if notional <= 0:
                return None
            return round((net / notional) * 10000.0, 4)
        except (TypeError, ValueError):
            return None

    def load_recent_plan_outcomes(self, n: int = 24) -> List[Dict[str, Any]]:
        """Newest-first closed 60m trade-plan outcomes (deduped by plan_id, last wins)."""
        if n <= 0 or not self.outcomes_file.exists():
            return []
        by_id: Dict[str, Dict[str, Any]] = {}
        order: List[str] = []
        try:
            with open(self.outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except Exception:
                        continue
                    pid = str(row.get("plan_id") or "")
                    if not pid:
                        continue
                    if pid not in by_id:
                        order.append(pid)
                    by_id[pid] = row
        except Exception:
            return []
        uniq = [by_id[pid] for pid in order]
        recent = list(reversed(uniq[-n:]))
        out: List[Dict[str, Any]] = []
        for row in recent:
            r = dict(row)
            bps = self._plan_outcome_net_bps(r)
            if bps is not None:
                r["net_return_bps"] = bps
            # Display helpers for UI (IST open/close)
            et = r.get("entry_time_ms")
            xt = r.get("exit_time_ms")
            try:
                if et is not None:
                    r["open_time_ist"] = datetime.fromtimestamp(
                        int(et) / 1000.0, tz=timezone.utc
                    ).astimezone(timezone(timedelta(hours=5, minutes=30))).strftime(
                        "%Y-%m-%d %H:%M:%S IST"
                    )
                    r["open_time_utc"] = datetime.fromtimestamp(
                        int(et) / 1000.0, tz=timezone.utc
                    ).strftime("%Y-%m-%d %H:%M:%S UTC")
            except Exception:
                pass
            try:
                if xt is not None:
                    r["close_time_ist"] = datetime.fromtimestamp(
                        int(xt) / 1000.0, tz=timezone.utc
                    ).astimezone(timezone(timedelta(hours=5, minutes=30))).strftime(
                        "%Y-%m-%d %H:%M:%S IST"
                    )
                    r["close_time_utc"] = datetime.fromtimestamp(
                        int(xt) / 1000.0, tz=timezone.utc
                    ).strftime("%Y-%m-%d %H:%M:%S UTC")
            except Exception:
                pass
            out.append(r)
        return out

    def build_plan_trade_pnl_summary(self) -> Dict[str, Any]:
        """W/L-first summary from trade_plan_outcomes.jsonl (books never pooled)."""
        summary = {
            "n_trades": 0,
            "n_wins": 0,
            "n_losses": 0,
            "cumulative_gross_pnl_usdt": 0.0,
            "cumulative_net_pnl_usdt": 0.0,
            "cumulative_net_return_bps": 0.0,
            "cumulative_friction_usdt": 0.0,
            "tp_hits": 0,
            "sl_hits": 0,
            "time_exits": 0,
            "giveback_exits": 0,
        }
        if not self.outcomes_file.exists():
            return summary
        by_id: Dict[str, Dict[str, Any]] = {}
        try:
            with open(self.outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except Exception:
                        continue
                    pid = str(row.get("plan_id") or "")
                    if pid:
                        by_id[pid] = row
        except Exception:
            return summary
        for row in by_id.values():
            net = float(row.get("net_pnl_usdt") or 0.0)
            gross = float(row.get("gross_pnl_usdt") or 0.0)
            fric = float(row.get("friction_usdt") or 0.0)
            bps = self._plan_outcome_net_bps(row) or 0.0
            summary["n_trades"] += 1
            if net > 0:
                summary["n_wins"] += 1
            else:
                summary["n_losses"] += 1
            summary["cumulative_net_pnl_usdt"] += net
            summary["cumulative_gross_pnl_usdt"] += gross
            summary["cumulative_friction_usdt"] += fric
            summary["cumulative_net_return_bps"] += bps
            reason = str(row.get("exit_reason") or "")
            state = str(row.get("state") or "")
            if "PROFIT_TARGET" in reason or state == "CLOSED_TP":
                summary["tp_hits"] += 1
            elif (
                "TRAILING_STOP" in reason
                or state == "CLOSED_TRAIL"
                or reason == "TRAILING_STOP_HIT"
            ):
                summary["trail_hits"] = int(summary.get("trail_hits") or 0) + 1
            elif "STOP_LOSS" in reason or "AMBIGUOUS" in reason or state == "CLOSED_SL":
                summary["sl_hits"] += 1
            elif "TIME" in reason or "HORIZON" in reason or state == "TIME_EXIT":
                summary["time_exits"] += 1
            elif "GIVEBACK" in reason or state == "CLOSED_GIVEBACK":
                summary["giveback_exits"] += 1
        for k in (
            "cumulative_net_pnl_usdt",
            "cumulative_gross_pnl_usdt",
            "cumulative_friction_usdt",
            "cumulative_net_return_bps",
        ):
            summary[k] = round(float(summary[k]), 4)
        return summary

    def build_trade_plan_pnl_payload(self, n: int = 24) -> Dict[str, Any]:
        """API payload for Crypto trade plan ledger (independent of OF paper)."""
        return {
            "protocol_id": "RP-608-CRYPTO-TRADE-PLAN-001",
            "book": "CRYPTO_PLAN",
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "summary": self.build_plan_trade_pnl_summary(),
            "recent": self.load_recent_plan_outcomes(n),
        }

    def process_decision_card(self, card: Dict[str, Any], current_quote: Optional[Dict[str, Any]] = None) -> Optional[Dict[str, Any]]:
        """Evaluates a new decision card for Trade Plan creation or suppression."""
        # Strict Boundary Check: Exclude pre-window startup cards (< 05:30 IST / 1792972800000 ms)
        close_time_ms = int(card.get("close_time_ms", 0))
        if close_time_ms < 1792972800000:
            return None

        # Park Stage-1 regime telemetry for OF paper align (same card fields).
        if isinstance(card, dict) and card.get("regime_state") is not None:
            self.state["last_decision_regime"] = {
                "regime_state": card.get("regime_state"),
                "regime_reason": card.get("regime_reason"),
                "directional_efficiency_er": card.get("directional_efficiency_er"),
                "relative_volatility": card.get("relative_volatility"),
                "adx_14": card.get("adx_14"),
                "plus_di": card.get("plus_di"),
                "minus_di": card.get("minus_di"),
                "regime_trend_dir": card.get("regime_trend_dir"),
            }

        direction = card.get("policy_b_action")
        # Always note tradeable B for persist CF (even if we later suppress / busy).
        _note_ms = int(
            (current_quote or {}).get("exchange_event_time_ms")
            or card.get("close_time_ms")
            or time.time() * 1000
        )
        if direction in ("LONG", "SHORT"):
            self._note_policy_b(_note_ms, str(direction))

        now_ms = int(
            (current_quote or {}).get("exchange_event_time_ms")
            or card.get("close_time_ms")
            or time.time() * 1000
        )

        # SCI-ADAPTER-REC P2: S1=b side generator (observational twin; default off).
        # May open even when Coralys/Estimand emits NO_TRADE. Does not touch
        # Gate 2E B, adapter_select, PRIMARY, or coralys-*.
        if self._require_sci_adapter_rec():
            closes_raw = (
                card.get("ohlcv_closes_15m")
                or card.get("completed_closes")
                or []
            )
            vols_raw = (
                card.get("ohlcv_volumes_15m")
                or card.get("completed_volumes")
                or []
            )
            closes: List[float] = []
            volumes: List[float] = []
            if isinstance(closes_raw, list):
                for x in closes_raw:
                    try:
                        closes.append(float(x))
                    except (TypeError, ValueError):
                        continue
            if isinstance(vols_raw, list):
                for x in vols_raw:
                    try:
                        volumes.append(float(x))
                    except (TypeError, ValueError):
                        continue
            pack_override = self._config_overrides.get("sci_adapter_rec_pack")
            pack_path = Path(str(pack_override)) if pack_override else None
            sci = sci_adapter_rec_recommend(closes, volumes, pack_path=pack_path)
            card["sci_adapter_rec"] = sci
            if sci.get("blocks_entry") or str(sci.get("side") or "").upper() not in {
                "LONG",
                "SHORT",
            }:
                self.state["suppressed_signals_count"] = int(
                    self.state.get("suppressed_signals_count") or 0
                ) + 1
                summ = self.state.setdefault("outcomes_summary", {})
                summ["sci_adapter_rec_suppressions"] = int(
                    summ.get("sci_adapter_rec_suppressions") or 0
                ) + 1
                self.state["last_suppressed_sci_adapter_rec"] = {
                    "suppressed_ms": now_ms,
                    "book_id": self.book_id,
                    **sci,
                }
                self._log_event(
                    "SUPPRESSED_SCI_ADAPTER_REC",
                    {"book_id": self.book_id, "side": sci.get("side")},
                    {
                        "sci_adapter_rec": sci,
                        "protocol_id": SCI_ADAPTER_REC_PROTOCOL_ID,
                        "card_bar_idx": card.get("bar_idx"),
                        "n_closes": len(closes),
                        "n_volumes": len(volumes),
                    },
                )
                self._save_state()
                return None
            direction = str(sci.get("side")).upper()
            card["policy_b_action"] = direction
            self._note_policy_b(now_ms, direction)

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
                if self.of_paper_enabled:
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

        # Gate 2E: same-direction cooldown after fast SL / path-scratch.
        cool = self._gate2e_cooldown_blocks(str(direction), now_ms)
        if cool:
            self.state["suppressed_signals_count"] = int(
                self.state.get("suppressed_signals_count") or 0
            ) + 1
            summ = self.state.setdefault("outcomes_summary", {})
            summ["gate2e_cooldown_suppressions"] = int(
                summ.get("gate2e_cooldown_suppressions") or 0
            ) + 1
            self._log_event(
                "SUPPRESSED_GATE2E_FAST_SL_COOLDOWN",
                {"direction": direction, "book_id": self.book_id},
                {
                    "cooldown": cool,
                    "now_ms": now_ms,
                    "card_bar_idx": card.get("bar_idx"),
                },
            )
            self._log_gate2e_rejected_cf(
                reject_reason="COOLDOWN",
                suppressed_ms=now_ms,
                direction=direction,
                plan={"direction": direction, "symbol": self.symbol, "bar_idx": card.get("bar_idx")},
                detail=str((cool or {}).get("reason") or "FAST_SL_COOLDOWN"),
                extra={"cooldown": cool},
            )
            self._save_state()
            return None

        # Clean-slate 15m adapter select (observational twin; D1=a filter on candidate).
        # Does not touch Gate 2E B trend-agree or PRIMARY Gate 2E A stack.
        if self._require_adapter_select_15m():
            closes_raw = (
                card.get("ohlcv_closes_15m")
                or card.get("completed_closes")
                or []
            )
            closes: List[float] = []
            if isinstance(closes_raw, list):
                for x in closes_raw:
                    try:
                        closes.append(float(x))
                    except (TypeError, ValueError):
                        continue
            sel = adapter_select_15m_recommend(closes, str(direction))
            card["adapter_select_15m"] = sel
            if sel.get("blocks_entry"):
                self.state["suppressed_signals_count"] = int(
                    self.state.get("suppressed_signals_count") or 0
                ) + 1
                summ = self.state.setdefault("outcomes_summary", {})
                summ["adapter_select_15m_suppressions"] = int(
                    summ.get("adapter_select_15m_suppressions") or 0
                ) + 1
                self.state["last_suppressed_adapter_select_15m"] = {
                    "direction": direction,
                    "suppressed_ms": now_ms,
                    "book_id": self.book_id,
                    **sel,
                }
                self._log_event(
                    "SUPPRESSED_ADAPTER_SELECT_15M",
                    {"direction": direction, "book_id": self.book_id},
                    {
                        "adapter_select_15m": sel,
                        "protocol_id": ADAPTER_SELECT_15M_PROTOCOL_ID,
                        "card_bar_idx": card.get("bar_idx"),
                        "n_closes": len(closes),
                    },
                )
                self._save_state()
                return None

        # Gate 2E B: 15m multi-lookback OHLCV trend agreement (observational twin).
        if self._require_trend_agree():
            closes_raw = card.get("ohlcv_closes_15m") or card.get("completed_closes") or []
            closes: List[float] = []
            if isinstance(closes_raw, list):
                for x in closes_raw:
                    try:
                        closes.append(float(x))
                    except (TypeError, ValueError):
                        continue
            ta = trend_agreement(closes, str(direction))
            card["trend_agree"] = ta
            if ta.get("blocks_entry"):
                self.state["suppressed_signals_count"] = int(
                    self.state.get("suppressed_signals_count") or 0
                ) + 1
                summ = self.state.setdefault("outcomes_summary", {})
                summ["gate2e_trend_agree_suppressions"] = int(
                    summ.get("gate2e_trend_agree_suppressions") or 0
                ) + 1
                self.state["last_suppressed_trend_agree"] = {
                    "direction": direction,
                    "suppressed_ms": now_ms,
                    "book_id": self.book_id,
                    **ta,
                }
                self._log_event(
                    "SUPPRESSED_GATE2E_TREND_AGREE",
                    {"direction": direction, "book_id": self.book_id},
                    {
                        "trend_agree": ta,
                        "protocol_id": TREND_AGREE_PROTOCOL_ID,
                        "card_bar_idx": card.get("bar_idx"),
                        "n_closes": len(closes),
                    },
                )
                self._save_state()
                return None

        # Create new plan
        plan = calculate_trade_plan(card, quote=current_quote)
        if not plan:
            return None
        plan["book_id"] = self.book_id
        plan["volume_pattern_axes"] = bool(self.volume_pattern_axes)
        if isinstance(card.get("adapter_select_15m"), dict):
            plan["adapter_select_15m"] = card.get("adapter_select_15m")
        if isinstance(card.get("sci_adapter_rec"), dict):
            plan["sci_adapter_rec"] = card.get("sci_adapter_rec")
        if isinstance(card.get("trend_agree"), dict):
            plan["trend_agree"] = card.get("trend_agree")
        self._stamp_gate2x_runtime_on_plan(plan)
        if not self.volume_pattern_axes:
            # Baseline twin: keep pre-volume plan geometry; no OHLCV surge latch.
            plan["vol_burst_20"] = None
            plan["vol_surge"] = "UNK"
            plan["exit_vol_burst_20"] = None
            plan["exit_vol_surge"] = "UNK"

        # Gate 2E: Policy B persist (multi-fire). Stamp always; suppress when env min>0.
        b_persist = self._b_persist_stamp(now_ms, str(direction))
        plan["b_persist"] = b_persist
        card["b_persist"] = b_persist
        if b_persist.get("blocks_entry"):
            self.state["suppressed_signals_count"] = int(
                self.state.get("suppressed_signals_count") or 0
            ) + 1
            summ = self.state.setdefault("outcomes_summary", {})
            summ["gate2e_b_persist_suppressions"] = int(
                summ.get("gate2e_b_persist_suppressions") or 0
            ) + 1
            self.state["last_suppressed_b_persist"] = {
                "direction": direction,
                "suppressed_ms": now_ms,
                "book_id": self.book_id,
                **b_persist,
            }
            self._log_event(
                "SUPPRESSED_GATE2E_B_PERSIST",
                plan,
                {
                    "b_persist": b_persist,
                    "book_id": self.book_id,
                    "card_bar_idx": card.get("bar_idx"),
                },
            )
            self._log_gate2e_rejected_cf(
                reject_reason="B_PERSIST",
                suppressed_ms=now_ms,
                direction=direction,
                plan=plan,
                detail=str(b_persist.get("block_reason") or "B_PERSIST"),
                extra={"b_persist": b_persist},
            )
            self._save_state()
            return None

        # Gate 2E: require L1 OF support at open (env / override). Twin-safe — same
        # filter on every manager when armed. No OF ±0.30 / Gate 5 geometry change.
        if self._plan_require_of() and not bool(plan.get("order_flow_supported")):
            self.state["suppressed_signals_count"] = int(
                self.state.get("suppressed_signals_count") or 0
            ) + 1
            summ = self.state.setdefault("outcomes_summary", {})
            summ["gate2e_of_require_suppressions"] = int(
                summ.get("gate2e_of_require_suppressions") or 0
            ) + 1
            of_stamp = {
                "order_flow_supported": bool(plan.get("order_flow_supported")),
                "order_flow_label": plan.get("order_flow_label"),
                "order_flow_imbalance": plan.get("order_flow_imbalance"),
                "order_flow_source": plan.get("order_flow_source"),
                "protocol_id": PLAN_REQUIRE_OF_PROTOCOL_ID,
                "block_reason": "OF_NOT_SUPPORTED_AT_OPEN",
            }
            plan["of_require_align"] = of_stamp
            self.state["last_suppressed_of_require"] = {
                "direction": direction,
                "suppressed_ms": now_ms,
                "book_id": self.book_id,
                **of_stamp,
            }
            self._log_event(
                "SUPPRESSED_GATE2E_OF_REQUIRE",
                plan,
                {
                    "of_require_align": of_stamp,
                    "book_id": self.book_id,
                    "card_bar_idx": card.get("bar_idx"),
                },
            )
            self._log_gate2e_rejected_cf(
                reject_reason="OF_REQUIRE",
                suppressed_ms=now_ms,
                direction=direction,
                plan=plan,
                detail="OF_NOT_SUPPORTED_AT_OPEN",
                extra={"of_require_align": of_stamp},
            )
            self._save_state()
            return None

        # Gate 2E: do not open if forming path already through proposed stop.
        # Observational dual-stamp (tick/last vs forming H/L) — live authority unchanged.
        dual = self._maybe_log_gate2e_path_dual_stamp(plan, current_quote)
        path_invalid = dual.get("forming_path_invalid")
        if path_invalid:
            self.state["suppressed_signals_count"] = int(
                self.state.get("suppressed_signals_count") or 0
            ) + 1
            summ = self.state.setdefault("outcomes_summary", {})
            summ["gate2e_path_invalid_suppressions"] = int(
                summ.get("gate2e_path_invalid_suppressions") or 0
            ) + 1
            self._log_event(
                "SUPPRESSED_GATE2E_PATH_INVALID",
                plan,
                {
                    "reason": path_invalid,
                    "stop_loss_price": plan.get("stop_loss_price"),
                    "high_price": (current_quote or {}).get("high_price"),
                    "low_price": (current_quote or {}).get("low_price"),
                    "book_id": self.book_id,
                    "tick_path_invalid": dual.get("tick_path_invalid"),
                    "tick_last_px": dual.get("tick_last_px"),
                    "diverge": dual.get("diverge"),
                    "diverge_kind": dual.get("diverge_kind"),
                    "path_dual_stamp_protocol": dual.get("protocol_id"),
                },
            )
            self._log_gate2e_rejected_cf(
                reject_reason="PATH",
                suppressed_ms=now_ms,
                direction=direction,
                plan=plan,
                detail=str(path_invalid),
                extra={
                    "tick_path_invalid": dual.get("tick_path_invalid"),
                    "diverge_kind": dual.get("diverge_kind"),
                },
            )
            self._save_state()
            return None

        # Gate 2E regime-align + DI skew (decision-engine tune):
        # DIRECTIONAL adverse, or SIDEWAYS/TRANSITIONING with large DI skew fighting side.
        # PRIMARY only by default; BASELINE/r50/SCI-clean twins remain ungated for CF.
        # P1a: config skip_gate2e_regime_align forces non-blocking (stamp still written).
        regime_stamp = evaluate_regime_align(
            direction=str(direction),
            regime=regime_dict_from_card(card),
            book_id=self.book_id,
        )
        if bool(self._config_overrides.get("skip_gate2e_regime_align")):
            regime_stamp = dict(regime_stamp)
            regime_stamp["blocks_entry"] = False
            regime_stamp["p1a_sci_clean_bypass"] = True
        plan["regime_align"] = regime_stamp
        card["regime_align"] = regime_stamp
        if regime_stamp.get("blocks_entry"):
            self.state["suppressed_signals_count"] = int(
                self.state.get("suppressed_signals_count") or 0
            ) + 1
            summ = self.state.setdefault("outcomes_summary", {})
            summ["gate2e_regime_align_suppressions"] = int(
                summ.get("gate2e_regime_align_suppressions") or 0
            ) + 1
            guard = str(regime_stamp.get("guard") or "DIRECTIONAL")
            if guard == "DI_SKEW":
                summ["gate2e_di_skew_suppressions"] = int(
                    summ.get("gate2e_di_skew_suppressions") or 0
                ) + 1
                event_name = "SUPPRESSED_GATE2E_DI_SKEW"
            else:
                event_name = "SUPPRESSED_GATE2E_REGIME_ALIGN"
            self.state["last_suppressed_regime_align"] = {
                "direction": direction,
                "regime_state": regime_stamp.get("regime_state"),
                "trend_dir": regime_stamp.get("trend_dir"),
                "block_reason": regime_stamp.get("block_reason"),
                "guard": guard,
                "di_skew": regime_stamp.get("di_skew"),
                "suppressed_ms": now_ms,
                "book_id": self.book_id,
                "protocol_id": regime_stamp.get("protocol_id"),
            }
            self._log_event(
                event_name,
                plan,
                {
                    "regime_align": regime_stamp,
                    "book_id": self.book_id,
                    "card_bar_idx": card.get("bar_idx"),
                },
            )
            self._log_gate2e_rejected_cf(
                reject_reason="DI_SKEW" if guard == "DI_SKEW" else "REGIME_DIRECTIONAL",
                suppressed_ms=now_ms,
                direction=direction,
                plan=plan,
                detail=str(regime_stamp.get("block_reason") or guard),
                regime_align=regime_stamp,
            )
            self._save_state()
            return None

        # Gate 2E observational entry pattern (env-gated standaside; no symbol bans).
        try:
            pattern_stamp = self._match_entry_pattern(card, plan)
            plan["trade_pattern_match"] = pattern_stamp
            card["trade_pattern_match"] = pattern_stamp
            if pattern_stamp.get("blocks_entry"):
                self.state["suppressed_signals_count"] = int(
                    self.state.get("suppressed_signals_count") or 0
                ) + 1
                summ = self.state.setdefault("outcomes_summary", {})
                summ["gate2e_pattern_suppressions"] = int(
                    summ.get("gate2e_pattern_suppressions") or 0
                ) + 1
                suppress_snap = {
                    "prediction": pattern_stamp.get("prediction"),
                    "pattern_id": pattern_stamp.get("pattern_id"),
                    "feature_pattern_id": pattern_stamp.get("feature_pattern_id"),
                    "match_mode": pattern_stamp.get("match_mode"),
                    "gate_mode": pattern_stamp.get("gate_mode")
                    or self._pattern_entry_gate_mode(),
                    "direction": plan.get("direction"),
                    "plan_id": plan.get("plan_id"),
                    "suppressed_ms": now_ms,
                    "hist_wr": pattern_stamp.get("hist_wr"),
                    "cohort_n": pattern_stamp.get("cohort_n"),
                    "live_vol_surge": pattern_stamp.get("live_vol_surge")
                    or plan.get("vol_surge"),
                    "live_vol_burst_20": pattern_stamp.get("live_vol_burst_20")
                    if pattern_stamp.get("live_vol_burst_20") is not None
                    else plan.get("vol_burst_20"),
                }
                self.state["last_suppressed_trade_pattern"] = suppress_snap
                self._log_event(
                    "SUPPRESSED_TRADE_PATTERN",
                    plan,
                    {
                        "pattern_id": pattern_stamp.get("pattern_id"),
                        "feature_pattern_id": pattern_stamp.get(
                            "feature_pattern_id"
                        ),
                        "prediction": pattern_stamp.get("prediction"),
                        "match_mode": pattern_stamp.get("match_mode"),
                        "hist_wr": pattern_stamp.get("hist_wr"),
                        "cohort_n": pattern_stamp.get("cohort_n"),
                        "gate_enabled": pattern_stamp.get("gate_enabled"),
                        "gate_mode": pattern_stamp.get("gate_mode")
                        or self._pattern_entry_gate_mode(),
                        "book_id": self.book_id,
                        # Live Gate2E candle vol (≠ pattern_id mined V*; ≠ OFX)
                        "live_vol_surge": suppress_snap.get("live_vol_surge"),
                        "live_vol_burst_20": suppress_snap.get(
                            "live_vol_burst_20"
                        ),
                        "vol_surge": plan.get("vol_surge"),
                        "vol_burst_20": plan.get("vol_burst_20"),
                    },
                )
                # Observational thin-WR counterfactual (stamp-only; authority 0%).
                self._maybe_log_gate2e_shadow_allow(
                    pattern_stamp, plan, suppressed_ms=now_ms
                )
                # Observational cold-start INSUFFICIENT (incl. n=0; authority 0%).
                self._maybe_log_gate2e_shadow_insufficient(
                    pattern_stamp, plan, suppressed_ms=now_ms
                )
                # Rejected-trade CF ledger → join BASELINE twin PnL later.
                self._log_gate2e_rejected_cf(
                    reject_reason="PATTERN",
                    suppressed_ms=now_ms,
                    direction=plan.get("direction"),
                    plan=plan,
                    detail=str(
                        pattern_stamp.get("prediction")
                        or "STANDASIDE_TRADE_PATTERN"
                    ),
                    pattern_stamp=pattern_stamp,
                    extra={
                        "live_vol_surge": suppress_snap.get("live_vol_surge"),
                        "live_vol_burst_20": suppress_snap.get("live_vol_burst_20"),
                    },
                )
                self._save_state()
                return None
        except Exception as pat_exc:
            print(f"[crypto_pattern] entry match skipped: {pat_exc}")

        self.state["active_plan"] = plan
        self.state["total_plans_created"] += 1
        self.state.pop("last_suppressed_trade_pattern", None)
        self._log_event(
            "PLAN_CREATED",
            plan,
            {
                "order_flow_supported": plan.get("order_flow_supported"),
                "order_flow_imbalance": plan.get("order_flow_imbalance"),
                "trade_pattern_match": plan.get("trade_pattern_match"),
                "b_persist": plan.get("b_persist"),
                "book_id": self.book_id,
            },
        )
        # OF recommendation: activate on OF entry; independent of 60m plan clock
        # Baseline parallel twin keeps OF inert (primary book owns L1 OF paper).
        if self.of_paper_enabled:
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
        if isinstance(card, dict) and card.get("regime_state") is not None:
            self.state["last_decision_regime"] = {
                "regime_state": card.get("regime_state"),
                "regime_reason": card.get("regime_reason"),
                "directional_efficiency_er": card.get("directional_efficiency_er"),
                "relative_volatility": card.get("relative_volatility"),
                "adx_14": card.get("adx_14"),
                "plus_di": card.get("plus_di"),
                "minus_di": card.get("minus_di"),
                "regime_trend_dir": card.get("regime_trend_dir"),
            }
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
            # Same-side OF dropped — reset entry persist arm (mirror exit persist).
            self._clear_of_entry_persist_arm()
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
            if isinstance(rec, dict) and rec.get("active") and rec.get("of_trade_id"):
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
                        "of_entry_quality": rec.get("of_entry_quality"),
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

        # Refresh exit volume surge from forming/completed candle burst when present.
        if self.volume_pattern_axes:
            if q.get("vol_burst_20") is not None:
                try:
                    burst = float(q["vol_burst_20"])
                    plan["exit_vol_burst_20"] = burst
                    plan["exit_vol_surge"] = crypto_vol_surge_bucket(burst)
                except (TypeError, ValueError):
                    pass
            elif q.get("vol_surge") in {"LOW", "MED", "HIGH", "UNK"}:
                plan["exit_vol_surge"] = str(q["vol_surge"])

        # Gate 5 H120_PATH: track L1 mid adverse → revisit around entry (±side friction bps)
        mid = 0.5 * (bid + ask)
        band = float(entry_price) * (
            float(CONFIG.get("early_sl_path_friction_bps_side") or 6.0) / 10000.0
        )
        prev_adv = bool(plan.get("path_h_adverse_seen"))
        prev_rev = bool(plan.get("path_h_revisit_seen"))
        if direction == "LONG":
            if mid < float(entry_price) - band:
                plan["path_h_adverse_seen"] = True
            if plan.get("path_h_adverse_seen") and mid >= float(entry_price) - band:
                plan["path_h_revisit_seen"] = True
        else:
            if mid > float(entry_price) + band:
                plan["path_h_adverse_seen"] = True
            if plan.get("path_h_adverse_seen") and mid <= float(entry_price) + band:
                plan["path_h_revisit_seen"] = True
        path_stamp_flipped = (bool(plan.get("path_h_adverse_seen")) != prev_adv) or (
            bool(plan.get("path_h_revisit_seen")) != prev_rev
        )
        self.state["active_plan"] = plan
        # Persist path stamps + throttled marks so periodic daemon reload cannot
        # wipe excursion / last_mark (REQUIRE_RUST Gate2X peaks need continuity).
        last_mark_flush = int(getattr(self, "_last_mark_flush_ms", 0) or 0)
        mark_flush_due = (now_ms - last_mark_flush) >= 1000
        if path_stamp_flipped or mark_flush_due:
            self._last_mark_flush_ms = now_ms
            self._save_state()

        # R3: floor-lock twin is Rust-owned (gate2x_floor_lock_immediate params
        # stamped in _try_rust_plan_lifecycle). Adverse extremes in quote L/H.
        giveback_reason = None
        rust_closed = self._try_rust_plan_lifecycle(
            plan, q, now_ms=now_ms, bid=bid, ask=ask, low=low, high=high
        )
        if rust_closed is not None:
            return rust_closed
        plan = self.state.get("active_plan") or plan
        _rust_hold = plan.get("plan_lifecycle_authority") == "rust_rp608_plan_lifecycle"
        # Track A retire: when PLAN_LIFECYCLE=1, Rust owns Adaptive/Gate2X/SL-TP.
        # Python manage bodies only for RP608_PLAN_LIFECYCLE=0 (unit tests).
        # REQUIRE_RUST: missing bin → fail-closed (no silent Python manage).
        _require_rust_blocked = bool(plan_lifecycle_require_rust() and not _rust_hold)
        _skip_python_lifecycle = bool(
            plan_lifecycle_enabled() or _rust_hold or _require_rust_blocked
        )

        if not _skip_python_lifecycle and not giveback_reason:
            # Coralys/Gate2X first — Adaptive intelligence (doubt → Protect/Exit).
            giveback_reason = self._gate2x_update_mfe_giveback(
                plan, exec_px=exec_px, now_ms=now_ms
            )
            if not giveback_reason:
                _tp0 = float(plan["profit_target_price"])
                _hit_tp0 = False
                if direction == "LONG":
                    _hit_tp0 = (
                        bid >= _tp0
                        or high >= _tp0
                        or float(plan["mfe_exec_price"]) >= _tp0
                    )
                else:
                    _hit_tp0 = (
                        ask <= _tp0
                        or low <= _tp0
                        or float(plan["mfe_exec_price"]) <= _tp0
                    )
                # Coralys floor trail on Hold (not blind ATR / not TP latch).
                if self._adaptive_tp_try_extend(
                    plan,
                    direction=direction,
                    hit_tp=_hit_tp0,
                    now_ms=now_ms,
                    bid=bid,
                    ask=ask,
                    low=low,
                    high=high,
                ):
                    plan = self.state.get("active_plan") or plan
                    sl = float(plan["stop_loss_price"])
                    tp = float(plan["profit_target_price"])
            self.state["active_plan"] = plan
        if giveback_reason:
            terminal_state = "CLOSED_GIVEBACK"
            exit_reason = "GATE2X_MFE_GIVEBACK_PROTECT"
            exit_price = exec_px
            plan["state"] = terminal_state
            plan["exit_price"] = round_tick(exit_price)
            plan["exit_time_ms"] = now_ms
            plan["holding_duration_seconds"] = round((now_ms - entry_ms) / 1000.0, 1)
            _fl_stamp = "FLOOR_LOCK" in str(giveback_reason)
            if _fl_stamp:
                # Keep raw twin stamp for scorecard (do not Adaptive-prefix).
                plan["exit_reason_detail"] = giveback_reason
                plan["gate2x_protocol"] = self._cfg("gate2x_floor_lock_protocol") or (
                    self._cfg("gate2x_mfe_giveback_protocol")
                )
            elif bool(CONFIG.get("adaptive_tp_enabled")):
                plan["adaptive_tp_armed"] = True
                plan["exit_reason_detail"] = f"ADAPTIVE_CORALYS_{giveback_reason}"
                plan["gate2x_protocol"] = self._cfg("gate2x_mfe_giveback_protocol")
            else:
                plan["exit_reason_detail"] = giveback_reason
                plan["gate2x_protocol"] = self._cfg("gate2x_mfe_giveback_protocol")
            plan["gate2x_ratchet_peak_frac"] = float(
                self._cfg("gate2x_ratchet_peak_frac") or 0.30
            )
            if direction == "LONG":
                gross_pnl = (plan["exit_price"] - entry_price) * qty
            else:
                gross_pnl = (entry_price - plan["exit_price"]) * qty
            friction = (plan["simulated_notional_usdt"] + (plan["exit_price"] * qty)) * (
                6.0 / 10000.0
            )
            net_pnl = round(gross_pnl - friction, 2)
            plan["gross_pnl_usdt"] = round(gross_pnl, 2)
            plan["friction_usdt"] = round(friction, 4)
            plan["net_pnl_usdt"] = net_pnl
            plan["exit_reason"] = exit_reason
            self.state["closed_plans_count"] += 1
            summ = self.state.setdefault("outcomes_summary", {})
            summ["cumulative_net_pnl_usdt"] = float(
                summ.get("cumulative_net_pnl_usdt") or 0.0
            ) + net_pnl
            summ["gate2x_mfe_giveback_exits"] = int(
                summ.get("gate2x_mfe_giveback_exits") or 0
            ) + 1
            self.state["last_closed_plan"] = dict(plan)
            self.state["active_plan"] = None
            self._log_event(
                terminal_state,
                plan,
                {
                    "exit_price": plan["exit_price"],
                    "net_pnl": net_pnl,
                    "reason": exit_reason,
                    "detail": giveback_reason,
                    "peak_mfe_usdt": plan.get("gate2x_peak_mfe_usdt"),
                    "armed_threshold_usdt": plan.get("gate2x_mfe_armed_threshold_usdt"),
                },
            )
            self._log_outcome(plan)
            try:
                self._note_pattern_stop(plan)
            except Exception:
                pass
            try:
                self._note_gate2e_fast_sl_cooldown(plan)
            except Exception:
                pass
            self._save_state()
            return plan

        # Gate 2X observational exit-pattern stamp; optional paper reverse exit.
        try:
            exit_stamp = match_crypto_pattern_exit(
                library=self._crypto_pattern_lib(),
                plan=plan,
                include_volume=self.volume_pattern_axes,
            )
            plan["trade_pattern_exit"] = exit_stamp
            self.state["active_plan"] = plan
            if exit_stamp.get("apply_early_exit") and crypto_pattern_exit_gate_enabled():
                terminal_state = "CLOSED_PATTERN"
                exit_reason = "PATTERN_REVERSE"
                exit_price = exec_px
                plan["state"] = terminal_state
                plan["exit_price"] = round_tick(exit_price)
                plan["exit_time_ms"] = now_ms
                plan["holding_duration_seconds"] = round((now_ms - entry_ms) / 1000.0, 1)
                if direction == "LONG":
                    gross_pnl = (plan["exit_price"] - entry_price) * qty
                else:
                    gross_pnl = (entry_price - plan["exit_price"]) * qty
                friction = (plan["simulated_notional_usdt"] + (plan["exit_price"] * qty)) * (
                    6.0 / 10000.0
                )
                net_pnl = round(gross_pnl - friction, 2)
                plan["gross_pnl_usdt"] = round(gross_pnl, 2)
                plan["friction_usdt"] = round(friction, 4)
                plan["net_pnl_usdt"] = net_pnl
                plan["exit_reason"] = exit_reason
                self.state["closed_plans_count"] += 1
                self.state["outcomes_summary"]["cumulative_net_pnl_usdt"] += net_pnl
                self.state["last_closed_plan"] = dict(plan)
                self.state["active_plan"] = None
                self._log_event(
                    terminal_state,
                    plan,
                    {
                        "exit_price": plan["exit_price"],
                        "net_pnl": net_pnl,
                        "reason": exit_reason,
                        "trade_pattern_exit": exit_stamp,
                    },
                )
                self._log_outcome(plan)
                self._note_pattern_stop(plan)
                self._save_state()
                return plan
        except Exception as pat_exc:
            print(f"[crypto_pattern] exit match skipped: {pat_exc}")

        # OF exit is L1 reverse-flow only — independent of plan TP/SL/60m
        if evaluate_of and self.of_paper_enabled:
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

        # Refresh TP/SL after possible adaptive ratchet earlier in this tick.
        # Skipped when Rust plan-lifecycle owned Adaptive/Gate2X/SL/TP, or when
        # REQUIRE_RUST blocked Python manage (TIME_EXIT safety valve still allowed).
        terminal_state = None
        exit_reason = None
        exit_price = 0.0
        ambiguous = False
        tie_break = "TP"

        if not _skip_python_lifecycle:
            sl = float(plan["stop_loss_price"])
            tp = float(plan["profit_target_price"])
            try:
                tp_level = int(plan.get("tp_level") or 0)
            except (TypeError, ValueError):
                tp_level = 0
            adaptive_on = bool(CONFIG.get("adaptive_tp_enabled"))
            try:
                orig_sl = float(plan.get("original_stop_loss_price") or sl)
            except (TypeError, ValueError):
                orig_sl = sl
            sl_still_original = (
                sl <= orig_sl + 1e-9 if direction == "LONG" else sl + 1e-9 >= orig_sl
            )

            hit_tp = False
            hit_sl = False

            if direction == "LONG":
                if bid >= tp or high >= tp or float(plan["mfe_exec_price"]) >= tp:
                    hit_tp = True
                mae = float(plan["mae_exec_price"])
                if bid <= sl or low <= sl or (sl_still_original and mae <= sl):
                    hit_sl = True
                exit_price = sl if hit_sl else (tp if hit_tp else bid)
            else:  # SHORT — buy-to-cover on ask
                if ask <= tp or low <= tp or float(plan["mfe_exec_price"]) <= tp:
                    hit_tp = True
                mae = float(plan["mae_exec_price"])
                if ask >= sl or high >= sl or (sl_still_original and mae >= sl):
                    hit_sl = True
                exit_price = sl if hit_sl else (tp if hit_tp else ask)

            # Adaptive tick-trail: TP is runner-only — no hard TP / no TP latch.
            if adaptive_on:
                hit_tp = False

            ambiguous = bool(hit_tp and hit_sl)
            tie_break = str(CONFIG.get("ambiguous_tie_break") or "TP").upper()
            if ambiguous:
                self.state["outcomes_summary"]["ambiguous_hits"] += 1
                if tie_break == "SL":
                    terminal_state = "CLOSED_SL"
                    exit_reason = "AMBIGUOUS_STOP_TARGET"
                    exit_price = sl
                    self.state["outcomes_summary"]["sl_hits"] += 1
                else:
                    terminal_state = "CLOSED_TP"
                    exit_reason = "PROFIT_TARGET_HIT"
                    exit_price = tp
                    plan["ambiguous_resolved_as"] = "TP"
                    plan["gate5_policy_g"] = CONFIG.get("gate5_policy_g")
                    self.state["outcomes_summary"]["tp_hits"] += 1
            elif hit_tp:
                terminal_state = "CLOSED_TP"
                exit_reason = "PROFIT_TARGET_HIT"
                self.state["outcomes_summary"]["tp_hits"] += 1
                exit_price = tp
            elif hit_sl:
                hold_s = (now_ms - entry_ms) / 1000.0
                h_enabled = bool(CONFIG.get("early_sl_path_scratch_enabled"))
                h_max = float(CONFIG.get("early_sl_path_scratch_max_hold_s") or 120.0)
                path_ok = bool(plan.get("path_h_adverse_seen") and plan.get("path_h_revisit_seen"))
                if h_enabled and hold_s < h_max and path_ok:
                    # Gate 5 H120_PATH: scratch@entry (not optimistic OPT-all early SL)
                    terminal_state = "CLOSED_SCRATCH"
                    exit_reason = "EARLY_SL_PATH_SCRATCH"
                    exit_price = float(entry_price)
                    plan["gate5_policy_h"] = CONFIG.get("gate5_policy_h")
                    plan["exit_reason_detail"] = "EARLY_SL_PATH_SCRATCH_GATE5_H120"
                    summ = self.state.setdefault("outcomes_summary", {})
                    summ["early_sl_path_scratches"] = int(summ.get("early_sl_path_scratches") or 0) + 1
                else:
                    orig_sl = float(
                        plan.get("original_stop_loss_price")
                        if plan.get("original_stop_loss_price") is not None
                        else sl
                    )
                    trailed = bool(plan.get("trailing_sl_armed")) and abs(sl - orig_sl) > 1e-9
                    if trailed:
                        # Industry/OMS style (IBKR/TT): Trailing Stop fill ≠ initial SL.
                        terminal_state = "CLOSED_TRAIL"
                        exit_reason = "TRAILING_STOP_HIT"
                        plan["exit_reason_detail"] = "TRAILING_STOP_HIT"
                        summ = self.state.setdefault("outcomes_summary", {})
                        summ["trail_hits"] = int(summ.get("trail_hits") or 0) + 1
                    else:
                        terminal_state = "CLOSED_SL"
                        exit_reason = "STOP_LOSS_HIT"
                        self.state["outcomes_summary"]["sl_hits"] += 1
                    exit_price = sl
            elif now_ms >= expiry_ms:
                terminal_state = "TIME_EXIT"
                exit_reason = "MAX_HORIZON_60M_EXPIRED"
                self.state["outcomes_summary"]["time_exits"] += 1
        elif _require_rust_blocked and now_ms >= expiry_ms:
            # Cutover allows Python TIME_EXIT when Rust lifecycle is unavailable.
            terminal_state = "TIME_EXIT"
            exit_reason = "MAX_HORIZON_60M_EXPIRED"
            exit_price = exec_px
            self.state["outcomes_summary"]["time_exits"] += 1

        if terminal_state:
            plan["state"] = terminal_state
            plan["exit_price"] = round_tick(exit_price)
            if ambiguous and tie_break != "SL":
                plan["exit_reason_detail"] = "AMBIGUOUS_RESOLVED_TP_GATE5_G"
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
            try:
                self._note_pattern_stop(plan)
            except Exception:
                pass
            try:
                self._note_gate2e_fast_sl_cooldown(plan)
            except Exception:
                pass
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

        live = fetch_live_book_ticker(self.symbol)
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
