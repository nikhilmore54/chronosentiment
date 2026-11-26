#!/usr/bin/env python3
"""
Read-only RP-611 status for Pi Command Center / Gate5.
Does not mutate observers, RP-608, OF, or Gate5 behaviour.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict

WORKSPACE = Path(__file__).resolve().parent.parent
NSE_ROOT = WORKSPACE / "artifacts" / "rp611_nse_volume_absorption"
CRYPTO_ROOT = WORKSPACE / "artifacts" / "rp611_crypto_volume_absorption"


def _load_reg(root: Path) -> Dict[str, Any]:
    path = root / "registration.json"
    if not path.exists():
        return {"available": False, "status": "UNAVAILABLE"}
    try:
        reg = json.loads(path.read_text(encoding="utf-8"))
        reg["available"] = True
        return reg
    except Exception as e:
        return {"available": False, "status": "UNAVAILABLE", "error": str(e)}


def _count_jsonl(path: Path) -> int:
    if not path.exists():
        return 0
    n = 0
    try:
        with path.open("r", encoding="utf-8") as f:
            for line in f:
                if line.strip():
                    n += 1
    except Exception:
        return 0
    return n


def _arm_counts(root: Path, market: str) -> Dict[str, int]:
    journals = root / "journals"
    if market == "NSE":
        return {
            "candidates": _count_jsonl(journals / "RP611N_candidates.jsonl"),
            "confirmations": _count_jsonl(journals / "RP611N_confirmations.jsonl"),
            "session_log_rows": _count_jsonl(journals / "RP611N_session_log.jsonl"),
            "controls": _count_jsonl(journals / "RP611N_controls.jsonl"),
        }
    return {
        "candidates": _count_jsonl(journals / "RP611C_candidates.jsonl"),
        "confirmations": _count_jsonl(journals / "RP611C_confirmations.jsonl"),
        "daily_log_rows": _count_jsonl(journals / "RP611C_daily_log.jsonl"),
        "l1_telemetry_rows": _count_jsonl(journals / "RP611C_l1_telemetry.jsonl"),
        "arms": _count_jsonl(journals / "RP611C_arms.jsonl"),
    }


def get_rp611_dashboard_state() -> Dict[str, Any]:
    nse = _load_reg(NSE_ROOT)
    crypto = _load_reg(CRYPTO_ROOT)
    return {
        "live_order_authority": 0.0,
        "note": (
            "RP-611 Volume Absorption — independent research. "
            "Presentation only. Does not alter RP-608 / OF / Gate5."
        ),
        "nse": {
            **{k: nse.get(k) for k in (
                "available", "protocol_id", "short_id", "status", "disposition",
                "experiment_clock_started", "experiment_clock_started_utc",
                "observer_deployed", "phase", "phase1_deployed_utc",
                "pre_deployment_capture_gap_sec", "phase1_preflight_ok",
                "live_order_authority",
            )},
            "locks": nse.get("locks"),
            "journal_counts": _arm_counts(NSE_ROOT, "NSE") if nse.get("available") else {},
        },
        "crypto": {
            **{k: crypto.get(k) for k in (
                "available", "protocol_id", "short_id", "status", "disposition",
                "experiment_clock_started", "experiment_clock_started_utc",
                "observer_deployed", "phase", "phase1_deployed_utc",
                "pre_deployment_capture_gap_sec", "phase1_preflight_ok",
                "live_order_authority",
            )},
            "locks": crypto.get("locks"),
            "journal_counts": _arm_counts(CRYPTO_ROOT, "CRYPTO") if crypto.get("available") else {},
            "volume_source": (crypto.get("locks") or {}).get("volume_source", "spot_kline_15m"),
            "futures_excluded": True,
        },
    }
