#!/usr/bin/env python3
"""
RP-609 Indian retail multi-horizon observation primitives.

Read-only research helpers: horizon registry, CNC friction proxy, record shapes.
Does not place orders. live_order_authority is always 0.0.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

SCHEMA_VERSION = "rp609_multi_horizon_v1"
PROTOCOL_ID = "RP-609-INDIAN-RETAIL-MULTI-HORIZON-001"
LIVE_ORDER_AUTHORITY = 0.0
FRICTION_MODEL_ID = "IN_EQ_CNC_FRICTION_V1"

WORKSPACE = Path(__file__).resolve().parent.parent
ARTIFACT_ROOT = WORKSPACE / "artifacts" / "rp609_indian_retail_multi_horizon"
JOURNALS = ARTIFACT_ROOT / "journals"

HORIZONS: Dict[str, Dict[str, Any]] = {
    "H0_INTRADAY": {
        "horizon_id": "H0_INTRADAY",
        "label": "Intraday (MIS shadow — owned by RP-608)",
        "product_type": "MIS",
        "owner_protocol": "RP-608-NSE-SHADOW-001",
        "hold_sessions_min": 0,
        "hold_sessions_max": 0,
        "same_day_only": True,
        "observational_link_only": True,
        "default_review": None,
    },
    "H1_SWING": {
        "horizon_id": "H1_SWING",
        "label": "Swing / positional delivery",
        "product_type": "CNC",
        "owner_protocol": PROTOCOL_ID,
        "hold_sessions_min": 2,
        "hold_sessions_max": 20,
        "same_day_only": False,
        "observational_link_only": False,
        "default_review": "ON_EXIT_OR_MAX_SESSIONS",
    },
    "H2_PORTFOLIO_BUILD": {
        "horizon_id": "H2_PORTFOLIO_BUILD",
        "label": "Staggered portfolio build (CNC tranches)",
        "product_type": "CNC",
        "owner_protocol": PROTOCOL_ID,
        "hold_sessions_min": 5,
        "hold_sessions_max": 60,
        "same_day_only": False,
        "observational_link_only": False,
        "default_review": "ON_BUILD_COMPLETE",
        "default_caps": {
            "max_name_pct": 10.0,
            "max_sector_pct": 30.0,
            "n_tranches_default": 4,
        },
    },
    "H3_LONG_TERM": {
        "horizon_id": "H3_LONG_TERM",
        "label": "Long-term holding (thesis-linked CNC)",
        "product_type": "CNC",
        "owner_protocol": PROTOCOL_ID,
        "hold_sessions_min": 60,
        "hold_sessions_max": None,
        "same_day_only": False,
        "observational_link_only": False,
        "default_review": "QUARTERLY",
    },
}


def ensure_artifact_dirs() -> Path:
    JOURNALS.mkdir(parents=True, exist_ok=True)
    return JOURNALS


def registry_path() -> Path:
    return ensure_artifact_dirs() / "RP609_horizon_registry.json"


def write_horizon_registry() -> Path:
    path = registry_path()
    payload = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "friction_model_id": FRICTION_MODEL_ID,
        "updated_utc": datetime.now(timezone.utc).isoformat(),
        "horizons": HORIZONS,
        "promotion_gates": {
            "G-H1_SWING": "CLOSED",
            "G-H2_PORTFOLIO_BUILD": "CLOSED",
            "G-H3_LONG_TERM": "CLOSED",
            "G-LIVE": "CLOSED",
        },
        "notes": [
            "H0 owned by RP-608; RP-609 must not mutate intraday daemon.",
            "All H1–H3 activity is observation-only until gates open.",
        ],
    }
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    gates = ensure_artifact_dirs() / "RP609_promotion_gates.json"
    gates.write_text(
        json.dumps(
            {
                "protocol_id": PROTOCOL_ID,
                "updated_utc": payload["updated_utc"],
                "gates": payload["promotion_gates"],
                "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    return path


def estimate_cnc_friction_inr(
    *,
    buy_price: float,
    sell_price: float,
    quantity: float,
    half_spread_bps: float = 2.5,
) -> Dict[str, float]:
    """
    Deterministic CNC delivery friction proxy for observation.
    Not a broker invoice; mirrors statutory retail cash-equity shape.
    """
    buy_notional = max(0.0, float(buy_price) * float(quantity))
    sell_notional = max(0.0, float(sell_price) * float(quantity))
    turnover = buy_notional + sell_notional

    # Flat retail brokerage proxy (discount broker style), capped.
    brokerage = min(40.0, 0.0003 * turnover) if turnover else 0.0
    # Delivery STT on sell side ~0.1% (observational constant).
    stt = 0.001 * sell_notional
    exchange = 0.0000297 * turnover
    sebi = 0.000001 * turnover
    stamp = 0.00015 * buy_notional
    gst = 0.18 * (brokerage + exchange)
    spread_cost = turnover * (float(half_spread_bps) / 10000.0)
    total = brokerage + stt + exchange + sebi + stamp + gst + spread_cost
    mid = (buy_notional + sell_notional) / 2.0 if turnover else 0.0
    eff_bps = (total / mid * 10000.0) if mid > 0 else 0.0
    return {
        "brokerage": round(brokerage, 4),
        "stt": round(stt, 4),
        "exchange_charges": round(exchange, 4),
        "sebi_charges": round(sebi, 4),
        "stamp_duty": round(stamp, 4),
        "gst": round(gst, 4),
        "spread_cost": round(spread_cost, 4),
        "total_friction": round(total, 4),
        "effective_friction_bps": round(eff_bps, 4),
        "friction_model_id": FRICTION_MODEL_ID,
    }


def tax_lot_hint_from_hold_sessions(hold_sessions: Optional[int]) -> str:
    """Heuristic calendar proxy only — not tax advice (~250 sessions ≈ 12m)."""
    if hold_sessions is None:
        return "UNKNOWN"
    return "LTCG_GT_12M_HINT" if hold_sessions >= 250 else "STCG_LTE_12M_HINT"


def new_candidate(
    *,
    horizon_id: str,
    symbol: str,
    thesis_tag: str,
    reference_price: float,
    invalidation_price: Optional[float] = None,
    target_price: Optional[float] = None,
    max_sessions: Optional[int] = None,
    sector: Optional[str] = None,
    notes: Optional[str] = None,
    source: str = "manual_observe",
) -> Dict[str, Any]:
    if horizon_id not in HORIZONS:
        raise ValueError(f"unknown horizon_id={horizon_id}")
    if horizon_id == "H0_INTRADAY":
        raise ValueError("H0 candidates belong to RP-608; do not register via RP-609")
    h = HORIZONS[horizon_id]
    now = datetime.now(timezone.utc).isoformat()
    ms = max_sessions
    if ms is None:
        ms = h.get("hold_sessions_max") or h.get("hold_sessions_min")
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event_type": "CANDIDATE_REGISTERED",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "horizon_id": horizon_id,
        "product_type": h["product_type"],
        "symbol": symbol if str(symbol).endswith(".NS") else f"{symbol}.NS",
        "tradingsymbol": str(symbol).replace(".NS", ""),
        "sector": sector,
        "thesis_tag": thesis_tag,
        "reference_price": float(reference_price),
        "invalidation_price": invalidation_price,
        "target_price": target_price,
        "max_sessions": ms,
        "review_cadence": h.get("default_review"),
        "tax_lot_hint": tax_lot_hint_from_hold_sessions(ms if horizon_id == "H3_LONG_TERM" else None),
        "source": source,
        "notes": notes,
        "recorded_utc": now,
        "status": "CANDIDATE",
    }


def new_portfolio_build_plan(
    *,
    plan_id: str,
    legs: List[Dict[str, Any]],
    total_capital_inr: float,
    n_tranches: int = 4,
    max_name_pct: float = 10.0,
    max_sector_pct: float = 30.0,
) -> Dict[str, Any]:
    now = datetime.now(timezone.utc).isoformat()
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event_type": "PORTFOLIO_BUILD_PLAN",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "horizon_id": "H2_PORTFOLIO_BUILD",
        "product_type": "CNC",
        "plan_id": plan_id,
        "total_capital_inr": float(total_capital_inr),
        "n_tranches": int(n_tranches),
        "caps": {
            "max_name_pct": float(max_name_pct),
            "max_sector_pct": float(max_sector_pct),
        },
        "legs": legs,
        "status": "PLAN",
        "recorded_utc": now,
    }


def new_long_term_hold(
    *,
    symbol: str,
    thesis: str,
    thesis_version: str,
    entry_price: float,
    quantity: float,
    sector: Optional[str] = None,
) -> Dict[str, Any]:
    now = datetime.now(timezone.utc).isoformat()
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event_type": "LONG_TERM_HOLD_OPEN",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "horizon_id": "H3_LONG_TERM",
        "product_type": "CNC",
        "symbol": symbol if str(symbol).endswith(".NS") else f"{symbol}.NS",
        "tradingsymbol": str(symbol).replace(".NS", ""),
        "sector": sector,
        "thesis": thesis,
        "thesis_version": thesis_version,
        "entry_price": float(entry_price),
        "quantity": float(quantity),
        "review_cadence": "QUARTERLY",
        "tax_lot_hint": "STCG_LTE_12M_HINT",
        "status": "THESIS_OPEN",
        "recorded_utc": now,
    }


def append_jsonl(path: Path, row: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, default=str) + "\n")


def candidates_path() -> Path:
    return ensure_artifact_dirs() / "RP609_candidates.jsonl"


def swing_path() -> Path:
    return ensure_artifact_dirs() / "RP609_swing_trades.jsonl"


def portfolio_path() -> Path:
    return ensure_artifact_dirs() / "RP609_portfolio_builds.jsonl"


def long_term_path() -> Path:
    return ensure_artifact_dirs() / "RP609_long_term_holds.jsonl"


def daily_log_path() -> Path:
    return ensure_artifact_dirs() / "RP609_daily_observation_log.jsonl"
