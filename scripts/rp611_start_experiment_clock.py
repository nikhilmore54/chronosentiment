#!/usr/bin/env python3
"""
RP-611 experiment clock start (governance).

Stamps T0 for RP-611N and RP-611C, appends CLOCK_START integrity rows.
Does NOT mutate RP-608, OF lifecycle, Gate5, RP-609, or RP-610.
Does NOT deploy long-running observers (Phase-1 integrity logging only at T0).

Live order authority: 0.0%
"""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any, Dict

WORKSPACE = Path(__file__).resolve().parent.parent

NSE_ROOT = WORKSPACE / "artifacts" / "rp611_nse_volume_absorption"
CRYPTO_ROOT = WORKSPACE / "artifacts" / "rp611_crypto_volume_absorption"

NSE_PROTOCOL = "RP-611-NSE-VOLUME-ABSORPTION-001"
CRYPTO_PROTOCOL = "RP-611-CRYPTO-VOLUME-ABSORPTION-001"


def _utc_now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def _utc_ms() -> int:
    return int(time.time() * 1000)


def _load_json(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _save_json(path: Path, data: Dict[str, Any]) -> None:
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def _append_jsonl(path: Path, row: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, separators=(",", ":")) + "\n")


def start_clock(root: Path, protocol_id: str, short_id: str, log_name: str) -> Dict[str, Any]:
    reg_path = root / "registration.json"
    reg = _load_json(reg_path)
    if reg.get("experiment_clock_started") is True and reg.get("experiment_clock_started_utc"):
        return {
            "protocol_id": protocol_id,
            "already_started": True,
            "experiment_clock_started_utc": reg.get("experiment_clock_started_utc"),
        }

    started_utc = _utc_now()
    started_ms = _utc_ms()
    reg["status"] = "REGISTERED"
    reg["disposition"] = "REGISTERED_CLOCK_RUNNING_NOT_FULLY_DEPLOYED"
    reg["experiment_clock_started"] = True
    reg["experiment_clock_started_utc"] = started_utc
    reg["experiment_clock_started_ms"] = started_ms
    reg["experiment_clock_start_instruction"] = "start the experiment clock"
    reg["observer_deployed"] = False
    reg["phase"] = "1_data_integrity"
    reg["live_order_authority"] = 0.0
    _save_json(reg_path, reg)

    journals = root / "journals"
    journals.mkdir(parents=True, exist_ok=True)
    event = {
        "event_type": "CLOCK_START",
        "protocol_id": protocol_id,
        "short_id": short_id,
        "schema_version": "1.0.0",
        "live_order_authority": 0.0,
        "timestamp_utc": started_utc,
        "timestamp_ms": started_ms,
        "phase": "1_data_integrity",
        "observer_deployed": False,
        "note": (
            "Experiment evaluation window opened. Full absorption signal observer "
            "not yet deployed; Phase-1 integrity logging authorized at T0."
        ),
    }
    _append_jsonl(journals / log_name, event)
    _append_jsonl(
        journals / "RP611_clock_events.jsonl",
        event,
    )
    return {
        "protocol_id": protocol_id,
        "already_started": False,
        "experiment_clock_started_utc": started_utc,
        "registration": str(reg_path),
        "log": str(journals / log_name),
    }


def main() -> int:
    nse = start_clock(NSE_ROOT, NSE_PROTOCOL, "RP-611N", "RP611N_session_log.jsonl")
    crypto = start_clock(CRYPTO_ROOT, CRYPTO_PROTOCOL, "RP-611C", "RP611C_daily_log.jsonl")
    print(json.dumps({"nse": nse, "crypto": crypto}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
