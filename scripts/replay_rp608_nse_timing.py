#!/usr/bin/env python3
"""
scripts/replay_rp608_nse_timing.py

RP-608-TIMING-EXP-001: Horizon-Aware Timing & Pre-Closeout Excursion Replay Engine.
Strictly Read-Only (0.0% Order Authority) with Single-Pass Forward-Only Accounting.

Implements:
  1. Horizon-Aware Entry Eligibility:
     - T_available = 15:15:00 IST - T_decision
     - Eligible(H) <=> H <= T_available
  2. Completed 15m Candle Anchor:
     - Decision timestamps strictly at completed candle closes T_close(k).
  3. Decoupled Estimand Accounting:
     - Estimand A: Simulated Trade Lifecycle (Target +1.5%, Stop -1.0%, Timeout H, 15:15 Square-off, Statutory Costs).
     - Estimand B: Fixed-Horizon Directional Prediction with Right-Censoring for intervals extending past 15:30 IST.
  4. Conservative Resolution:
     - When intra-interval price range breaches both target and stop, resolve conservatively as STOP_LOSS_HIT.
     - Raw MFE and MAE distributions reported with 0.50 bps regularization floor for finite aggregate ratio.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from datetime import datetime, time as dtime, timedelta, timezone
import hashlib
import json
import math
from pathlib import Path
import sys
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_nse_shadow_daemon import (
    BarOHLCV,
    TopOfBookQuote,
    calc_indian_equity_friction,
    NSE_119_SYMBOLS
)

IST_OFFSET = timedelta(hours=5, minutes=30)
SESSION_OPEN_TIME = dtime(9, 15)
SESSION_SQUAREOFF_TIME = dtime(15, 15)
SESSION_CLOSE_TIME = dtime(15, 30)

DEFAULT_NOTIONAL_INR = 100000.0
TARGET_BPS = 150.0  # +1.50%
STOP_BPS = 100.0    # -1.00%


def get_bar_interval_info(bar_idx: int, session_date_iso: str) -> Dict[str, Any]:
    """
    Computes exact timestamps and interval span for 1-based completed 15m bar index k in {1..25}.
    bar_idx 1: [09:15, 09:30 IST), closes at 09:30:00 IST.
    """
    if not (1 <= bar_idx <= 25):
        raise ValueError(f"Invalid bar_idx {bar_idx}, must be 1..25")

    year, month, day = [int(x) for x in session_date_iso.split("-")[:3]]
    base_dt = datetime(year, month, day, 9, 15, tzinfo=timezone(IST_OFFSET))

    start_dt = base_dt + timedelta(minutes=(bar_idx - 1) * 15)
    close_dt = base_dt + timedelta(minutes=bar_idx * 15)
    sqoff_dt = datetime(year, month, day, 15, 15, tzinfo=timezone(IST_OFFSET))

    available_sec = max(0, int((sqoff_dt - close_dt).total_seconds()))
    available_min = available_sec // 60

    return {
        "bar_idx": bar_idx,
        "interval_span_ist": f"[{start_dt.strftime('%H:%M')}, {close_dt.strftime('%H:%M')} IST)",
        "close_timestamp_ist": close_dt.isoformat(),
        "close_timestamp_utc_ms": int(close_dt.astimezone(timezone.utc).timestamp() * 1000),
        "available_minutes_to_squareoff": available_min,
        "squareoff_timestamp_utc_ms": int(sqoff_dt.astimezone(timezone.utc).timestamp() * 1000),
    }


def is_horizon_eligible(bar_idx: int, horizon_minutes: int) -> Tuple[bool, str]:
    """
    Evaluates dynamic eligibility: H <= T_available.
    """
    info = get_bar_interval_info(bar_idx, "2026-10-26")
    avail = info["available_minutes_to_squareoff"]
    if horizon_minutes <= avail:
        return True, "ELIGIBLE_WITHIN_SESSION_RUNWAY"
    return False, f"INSUFFICIENT_RUNWAY_TO_SQUARE_OFF: {horizon_minutes}m required > {avail}m available"


def evaluate_lifecycle_trade(
    entry_price: float,
    entry_time_ms: int,
    horizon_minutes: int,
    session_date_iso: str,
    subsequent_quotes: List[Dict[str, Any]],
    half_spread_bps: float = 2.5,
    quantity: Optional[float] = None,
    target_bps: float = TARGET_BPS,
    stop_bps: float = STOP_BPS,
    prices_are_executable: bool = False
) -> Dict[str, Any]:
    """
    Evaluates Estimand A: Simulated Trade Lifecycle.
    Exit Priority:
      1. STOP_LOSS_HIT (conservative priority when both hit simultaneously)
      2. TARGET_HIT
      3. MAX_HOLDING_EXPIRED
      4. MANDATORY_INTRADAY_SQUARE_OFF
      5. DATA_QUALITY_UNRESOLVED
    """
    year, month, day = [int(x) for x in session_date_iso.split("-")[:3]]
    sqoff_dt = datetime(year, month, day, 15, 15, tzinfo=timezone(IST_OFFSET))
    sqoff_ms = int(sqoff_dt.astimezone(timezone.utc).timestamp() * 1000)

    expiry_ms = entry_time_ms + (horizon_minutes * 60 * 1000)
    effective_deadline_ms = min(expiry_ms, sqoff_ms)

    target_price = round(entry_price * (1.0 + (target_bps / 10000.0)), 2)
    stop_price = round(entry_price * (1.0 - (stop_bps / 10000.0)), 2)

    qty = quantity if quantity is not None else max(1.0, math.floor(DEFAULT_NOTIONAL_INR / entry_price))

    mfe_bps = 0.0
    mae_bps = 0.0

    resolved = False
    exit_price = None
    exit_reason = None
    exit_time_ms = None

    valid_subsequent = [q for q in subsequent_quotes if q["timestamp_utc_ms"] > entry_time_ms]

    if not valid_subsequent:
        return {
            "status": "DATA_QUALITY_UNRESOLVED",
            "entry_price": entry_price,
            "exit_price": None,
            "exit_reason": "NO_SUBSEQUENT_QUOTES",
            "exit_timestamp_iso": None,
            "gross_return_bps": None,
            "statutory_friction_bps": None,
            "net_return_bps": None,
            "mfe_bps": 0.0,
            "mae_bps": 0.0,
            "holding_seconds": 0
        }

    for q in valid_subsequent:
        q_time = q["timestamp_utc_ms"]
        bid = float(q.get("bid_price", 0.0))
        ask = float(q.get("ask_price", 0.0))

        # Check crossed/invalid quote
        if bid <= 0.0 or ask <= 0.0 or bid > ask:
            continue

        # Excursion tracking based on executable exit quote (bid for long)
        fav = ((bid - entry_price) / entry_price) * 10000.0
        adv = ((entry_price - bid) / entry_price) * 10000.0
        if fav > mfe_bps:
            mfe_bps = fav
        if adv > mae_bps:
            mae_bps = adv

        target_reached = (bid >= target_price)
        stop_reached = (bid <= stop_price)

        # Conservative ambiguity rule: if both reached, stop first
        if target_reached and stop_reached:
            resolved = True
            exit_price = stop_price
            exit_reason = "STOP_LOSS_HIT"
            exit_time_ms = q_time
            break

        if stop_reached:
            resolved = True
            exit_price = stop_price
            exit_reason = "STOP_LOSS_HIT"
            exit_time_ms = q_time
            break

        if target_reached:
            resolved = True
            exit_price = target_price
            exit_reason = "TARGET_HIT"
            exit_time_ms = q_time
            break

        # Check square-off deadline first
        if q_time >= sqoff_ms:
            resolved = True
            exit_price = bid
            exit_reason = "MANDATORY_INTRADAY_SQUARE_OFF"
            exit_time_ms = q_time
            break

        # Check horizon timeout
        if q_time >= expiry_ms:
            resolved = True
            exit_price = bid
            exit_reason = "MAX_HOLDING_EXPIRED"
            exit_time_ms = q_time
            break

    # If subsequent quotes ended without crossing deadline
    if not resolved:
        last_q = valid_subsequent[-1]
        exit_price = float(last_q.get("bid_price", entry_price))
        exit_time_ms = last_q["timestamp_utc_ms"]
        if exit_time_ms >= sqoff_ms:
            exit_reason = "MANDATORY_INTRADAY_SQUARE_OFF"
        elif exit_time_ms >= expiry_ms:
            exit_reason = "MAX_HOLDING_EXPIRED"
        else:
            exit_reason = "DATA_QUALITY_UNRESOLVED"
        resolved = (exit_reason != "DATA_QUALITY_UNRESOLVED")

    if resolved and exit_price is not None:
        gross_ret = ((exit_price - entry_price) / entry_price) * 10000.0
        # Spread double-counting safeguard: if prices are already executable (Ask entry, Bid exit),
        # the gross return already pays the spread; pass 0.0 to friction to avoid double-charging.
        effective_spread = 0.0 if prices_are_executable else half_spread_bps
        friction = calc_indian_equity_friction(
            buy_price=entry_price,
            sell_price=exit_price,
            quantity=qty,
            half_spread_bps=effective_spread
        )
        eff_fric = friction.get("effective_friction_bps", 4.8)
        net_ret = gross_ret - eff_fric
        holding_sec = max(0, int((exit_time_ms - entry_time_ms) / 1000)) if exit_time_ms else 0
        exit_iso = datetime.fromtimestamp(exit_time_ms / 1000.0, timezone.utc).isoformat() if exit_time_ms else None

        return {
            "status": exit_reason,
            "entry_price": entry_price,
            "exit_price": exit_price,
            "exit_reason": exit_reason,
            "exit_timestamp_iso": exit_iso,
            "gross_return_bps": round(gross_ret, 2),
            "statutory_friction_bps": round(eff_fric, 2),
            "net_return_bps": round(net_ret, 2),
            "mfe_bps": round(mfe_bps, 2),
            "mae_bps": round(mae_bps, 2),
            "holding_seconds": holding_sec
        }
    else:
        return {
            "status": "DATA_QUALITY_UNRESOLVED",
            "entry_price": entry_price,
            "exit_price": None,
            "exit_reason": "UNRESOLVED_EXHAUSTED_TIMELINE",
            "exit_timestamp_iso": None,
            "gross_return_bps": None,
            "statutory_friction_bps": None,
            "net_return_bps": None,
            "mfe_bps": round(mfe_bps, 2),
            "mae_bps": round(mae_bps, 2),
            "holding_seconds": 0
        }


def evaluate_directional_prediction(
    decision_price: float,
    decision_time_ms: int,
    horizon_minutes: int,
    session_date_iso: str,
    subsequent_quotes: List[Dict[str, Any]]
) -> Dict[str, Any]:
    """
    Evaluates Estimand B: Fixed-Horizon Directional Prediction.
    Right-censoring invariant: If decision_time + horizon > 15:30 IST, mark RIGHT_CENSORED_INCOMPLETE_HORIZON.
    Reports empirical directional hit rate and signed return; omits uncalibrated Brier scores.
    """
    year, month, day = [int(x) for x in session_date_iso.split("-")[:3]]
    close_dt = datetime(year, month, day, 15, 30, tzinfo=timezone(IST_OFFSET))
    close_ms = int(close_dt.astimezone(timezone.utc).timestamp() * 1000)

    target_outcome_ms = decision_time_ms + (horizon_minutes * 60 * 1000)

    if target_outcome_ms > close_ms:
        return {
            "status": "RIGHT_CENSORED_INCOMPLETE_HORIZON",
            "horizon_minutes": horizon_minutes,
            "forward_return_bps": None,
            "directional_label": None,
            "directional_hit": None
        }

    # Find the nearest quote occurring on or after target_outcome_ms
    outcome_quotes = [q for q in subsequent_quotes if q["timestamp_utc_ms"] >= target_outcome_ms]
    if not outcome_quotes:
        return {
            "status": "RIGHT_CENSORED_INCOMPLETE_HORIZON",
            "horizon_minutes": horizon_minutes,
            "forward_return_bps": None,
            "directional_label": None,
            "directional_hit": None
        }

    outcome_quote = outcome_quotes[0]
    p_outcome = (outcome_quote["bid_price"] + outcome_quote["ask_price"]) / 2.0
    fwd_ret = ((p_outcome - decision_price) / decision_price) * 10000.0
    label = 1 if p_outcome > decision_price else 0

    return {
        "status": "EVALUATED",
        "horizon_minutes": horizon_minutes,
        "forward_return_bps": round(fwd_ret, 2),
        "directional_label": label,
        "directional_hit": 1 if label == 1 else 0
    }


def construct_replay_row(
    bar_idx: int,
    session_date_iso: str,
    symbol: str,
    decision_price: float,
    traj15: float,
    history_length: int,
    horizon_minutes: int,
    subsequent_quotes: List[Dict[str, Any]],
    half_spread_bps: float = 2.5
) -> Dict[str, Any]:
    """
    Constructs a fully auditable Replay Row adhering strictly to Section 4.4 Schema.
    """
    info = get_bar_interval_info(bar_idx, session_date_iso)
    eligible_runway, runway_reason = is_horizon_eligible(bar_idx, horizon_minutes)

    if history_length < 5:
        eligible = False
        exclusion_reason = "COLD_START_INSUFFICIENT_HISTORY"
    elif not eligible_runway:
        eligible = False
        exclusion_reason = runway_reason
    elif traj15 < 0.05:
        eligible = False
        exclusion_reason = "SUB_THRESHOLD_MOMENTUM"
    else:
        eligible = True
        exclusion_reason = "NONE_QUALIFIED_ENTRY"

    if eligible:
        est_a = evaluate_lifecycle_trade(
            entry_price=decision_price,
            entry_time_ms=info["close_timestamp_utc_ms"],
            horizon_minutes=horizon_minutes,
            session_date_iso=session_date_iso,
            subsequent_quotes=subsequent_quotes,
            half_spread_bps=half_spread_bps
        )
    else:
        est_a = {
            "status": "NON_ENTERED",
            "entry_price": None,
            "exit_price": None,
            "exit_reason": exclusion_reason,
            "exit_timestamp_iso": None,
            "gross_return_bps": None,
            "statutory_friction_bps": None,
            "net_return_bps": None,
            "mfe_bps": 0.0,
            "mae_bps": 0.0,
            "holding_seconds": 0
        }

    # Directional prediction evaluated across all valid history bars
    if history_length >= 5:
        est_b = evaluate_directional_prediction(
            decision_price=decision_price,
            decision_time_ms=info["close_timestamp_utc_ms"],
            horizon_minutes=horizon_minutes,
            session_date_iso=session_date_iso,
            subsequent_quotes=subsequent_quotes
        )
    else:
        est_b = {
            "status": "COLD_START_EXCLUDED",
            "horizon_minutes": horizon_minutes,
            "forward_return_bps": None,
            "directional_label": None,
            "brier_score": None
        }

    return {
        "schema_version": "rp608_timing_replay_v1",
        "symbol": symbol,
        "bar_index": bar_idx,
        "candle_interval_ist": info["interval_span_ist"],
        "decision_timestamp_ist": info["close_timestamp_ist"],
        "decision_timestamp_utc_ms": info["close_timestamp_utc_ms"],
        "available_minutes_to_squareoff": info["available_minutes_to_squareoff"],
        "horizon_minutes": horizon_minutes,
        "score_traj15": round(traj15, 4),
        "history_length": history_length,
        "entry_eligible": eligible,
        "entry_exclusion_reason": exclusion_reason,
        "estimand_a": est_a,
        "estimand_b": est_b
    }


def compute_aggregate_metrics(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    """
    Computes summary metrics per (window, horizon) partition.
    """
    eligible_entries = [r for r in rows if r["entry_eligible"] and r["estimand_a"]["status"] in ("TARGET_HIT", "STOP_LOSS_HIT", "MAX_HOLDING_EXPIRED", "MANDATORY_INTRADAY_SQUARE_OFF")]
    n_entered = len(eligible_entries)

    if n_entered == 0:
        return {
            "total_candidates": len(rows),
            "entered_count": 0,
            "win_rate_pct": 0.0,
            "net_expectancy_bps": 0.0,
            "gross_expectancy_bps": 0.0,
            "mean_mfe_bps": 0.0,
            "mean_mae_bps": 0.0,
            "edge_ratio": 0.0,
            "exit_distribution": {}
        }

    wins = sum(1 for r in eligible_entries if r["estimand_a"]["status"] == "TARGET_HIT")
    win_rate = (wins / n_entered) * 100.0

    net_returns = [r["estimand_a"]["net_return_bps"] for r in eligible_entries]
    gross_returns = [r["estimand_a"]["gross_return_bps"] for r in eligible_entries]
    mfes = [r["estimand_a"]["mfe_bps"] for r in eligible_entries]
    maes = [r["estimand_a"]["mae_bps"] for r in eligible_entries]

    mean_mfe = sum(mfes) / n_entered
    mean_mae = sum(maes) / n_entered

    # Regularized Edge Ratio using 0.50 bps floor
    edge_ratio = mean_mfe / max(mean_mae, 0.50)

    exit_dist = {}
    for r in eligible_entries:
        s = r["estimand_a"]["status"]
        exit_dist[s] = exit_dist.get(s, 0) + 1

    return {
        "total_candidates": len(rows),
        "entered_count": n_entered,
        "win_rate_pct": round(win_rate, 2),
        "net_expectancy_bps": round(sum(net_returns) / n_entered, 2),
        "gross_expectancy_bps": round(sum(gross_returns) / n_entered, 2),
        "mean_mfe_bps": round(mean_mfe, 2),
        "mean_mae_bps": round(mean_mae, 2),
        "edge_ratio": round(edge_ratio, 2),
        "exit_distribution": exit_dist
    }
