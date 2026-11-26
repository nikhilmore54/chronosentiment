#!/usr/bin/env python3
"""
RP-610 Crypto multi-horizon observation primitives.

H0 (≤60m) remains owned by RP-608. H1 multi-day, H2 multi-week, H3 long-term
are observation-only spot-sim paper books. live_order_authority = 0.0.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Optional

SCHEMA_VERSION = "rp610_crypto_multi_horizon_v1"
PROTOCOL_ID = "RP-610-CRYPTO-MULTI-HORIZON-001"
LIVE_ORDER_AUTHORITY = 0.0
FRICTION_MODEL_ID = "CRYPTO_SPOT_RT_BPS_V1"
DEFAULT_RT_FRICTION_BPS = 12.0
DEFAULT_NOTIONAL_USDT = 1000.0

WORKSPACE = Path(__file__).resolve().parent.parent
ARTIFACT_ROOT = WORKSPACE / "artifacts" / "rp610_crypto_multi_horizon"
JOURNALS = ARTIFACT_ROOT / "journals"
DEFAULT_CRYPTO_DIR = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"

HORIZONS: Dict[str, Dict[str, Any]] = {
    "H0_INTRADAY_60M": {
        "horizon_id": "H0_INTRADAY_60M",
        "label": "Intraday / ≤60m trade-plan overlay (owned by RP-608)",
        "product_type": "SPOT_SIM",
        "owner_protocol": "RP-608-TRADE-PLAN-OVERLAY-001",
        "hold_days_min": 0,
        "hold_days_max": 0,
        "observational_link_only": True,
        "default_review": None,
    },
    "H1_MULTI_DAY": {
        "horizon_id": "H1_MULTI_DAY",
        "label": "Multi-day swing",
        "product_type": "SPOT_SIM",
        "owner_protocol": PROTOCOL_ID,
        "hold_days_min": 2,
        "hold_days_max": 14,
        "observational_link_only": False,
        "default_review": "ON_EXIT_OR_MAX_DAYS",
        "default_stop_pct": 0.04,
        "default_target_pct": 0.08,
    },
    "H2_MULTI_WEEK": {
        "horizon_id": "H2_MULTI_WEEK",
        "label": "Multi-week position",
        "product_type": "SPOT_SIM",
        "owner_protocol": PROTOCOL_ID,
        "hold_days_min": 14,
        "hold_days_max": 56,
        "observational_link_only": False,
        "default_review": "WEEKLY_MARK",
        "default_stop_pct": 0.08,
        "default_target_pct": 0.18,
    },
    "H3_LONG_TERM": {
        "horizon_id": "H3_LONG_TERM",
        "label": "Long-term thesis hold",
        "product_type": "SPOT_SIM",
        "owner_protocol": PROTOCOL_ID,
        "hold_days_min": 90,
        "hold_days_max": None,
        "observational_link_only": False,
        "default_review": "QUARTERLY",
        "review_cadence_days": 90,
    },
}


def ensure_artifact_dirs() -> Path:
    JOURNALS.mkdir(parents=True, exist_ok=True)
    return JOURNALS


def write_horizon_registry() -> Path:
    ensure_artifact_dirs()
    path = JOURNALS / "RP610_horizon_registry.json"
    now = datetime.now(timezone.utc).isoformat()
    payload = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "market": "CRYPTO",
        "symbol_default": "BTCUSDT",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "friction_model_id": FRICTION_MODEL_ID,
        "default_rt_friction_bps": DEFAULT_RT_FRICTION_BPS,
        "default_notional_usdt": DEFAULT_NOTIONAL_USDT,
        "updated_utc": now,
        "horizons": HORIZONS,
        "promotion_gates": {
            "G-H1_MULTI_DAY": "CLOSED",
            "G-H2_MULTI_WEEK": "CLOSED",
            "G-H3_LONG_TERM": "CLOSED",
            "G-LIVE": "CLOSED",
        },
        "notes": [
            "H0 owned by RP-608 trade-plan overlay; RP-610 must not mutate 60m exits.",
            "H1–H3 are observation-only spot-sim; no perpetual funding in v1.",
        ],
    }
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    (JOURNALS / "RP610_promotion_gates.json").write_text(
        json.dumps(
            {
                "protocol_id": PROTOCOL_ID,
                "updated_utc": now,
                "gates": payload["promotion_gates"],
                "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    return path


def estimate_crypto_friction_usdt(
    *,
    entry_price: float,
    exit_price: float,
    quantity: float,
    rt_friction_bps: float = DEFAULT_RT_FRICTION_BPS,
) -> Dict[str, float]:
    notional = abs(float(entry_price) * float(quantity)) + abs(float(exit_price) * float(quantity))
    mid = notional / 2.0 if notional else 0.0
    total = mid * (float(rt_friction_bps) / 10000.0)
    return {
        "round_trip_friction_bps": float(rt_friction_bps),
        "total_friction_usdt": round(total, 6),
        "friction_model_id": FRICTION_MODEL_ID,
    }


def append_jsonl(path: Path, row: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, default=str) + "\n")


def candidates_path() -> Path:
    return ensure_artifact_dirs() / "RP610_candidates.jsonl"


def day_path() -> Path:
    return ensure_artifact_dirs() / "RP610_day_trades.jsonl"


def week_path() -> Path:
    return ensure_artifact_dirs() / "RP610_week_trades.jsonl"


def long_path() -> Path:
    return ensure_artifact_dirs() / "RP610_long_term_holds.jsonl"


def daily_log_path() -> Path:
    return ensure_artifact_dirs() / "RP610_daily_observation_log.jsonl"
