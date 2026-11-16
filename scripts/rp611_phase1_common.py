#!/usr/bin/env python3
"""Shared Phase-1 journal / registration helpers for RP-611."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any, Dict, Optional

WORKSPACE = Path(__file__).resolve().parent.parent

NSE_ROOT = WORKSPACE / "artifacts" / "rp611_nse_volume_absorption"
CRYPTO_ROOT = WORKSPACE / "artifacts" / "rp611_crypto_volume_absorption"

T0_UTC = "2026-11-01T10:47:24Z"
T0_MS = 1761994044000  # will recompute from T0 string if needed


def utc_now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def utc_ms() -> int:
    return int(time.time() * 1000)


def parse_utc_ms(s: str) -> int:
    # YYYY-MM-DDTHH:MM:SSZ
    import calendar

    t = time.strptime(s, "%Y-%m-%dT%H:%M:%SZ")
    return int(calendar.timegm(t) * 1000)


def load_registration(root: Path) -> Dict[str, Any]:
    return json.loads((root / "registration.json").read_text(encoding="utf-8"))


def save_registration(root: Path, reg: Dict[str, Any]) -> None:
    path = root / "registration.json"
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(reg, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def append_jsonl(path: Path, row: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, separators=(",", ":")) + "\n")


def load_cursor(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def save_cursor(path: Path, data: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def stamp_deployment(
    root: Path,
    *,
    protocol_id: str,
    short_id: str,
    preflight_ok: bool,
    receipt: Dict[str, Any],
) -> Dict[str, Any]:
    reg = load_registration(root)
    t0 = reg.get("experiment_clock_started_utc") or T0_UTC
    t0_ms = int(reg.get("experiment_clock_started_ms") or parse_utc_ms(t0))
    deployed_utc = utc_now()
    deployed_ms = utc_ms()
    gap_ms = max(0, deployed_ms - t0_ms)
    reg["disposition"] = (
        "REGISTERED_CLOCK_RUNNING_PHASE1_DEPLOYED"
        if preflight_ok
        else "REGISTERED_CLOCK_RUNNING_PHASE1_DEPLOY_BLOCKED"
    )
    reg["observer_deployed"] = bool(preflight_ok)
    reg["phase"] = "1_data_integrity"
    reg["phase1_deployed_utc"] = deployed_utc
    reg["phase1_deployed_ms"] = deployed_ms
    reg["pre_deployment_capture_gap_ms"] = gap_ms
    reg["pre_deployment_capture_gap_sec"] = round(gap_ms / 1000.0, 3)
    reg["phase1_preflight_ok"] = bool(preflight_ok)
    reg["live_order_authority"] = 0.0
    # Preserve T0
    reg["experiment_clock_started"] = True
    reg["experiment_clock_started_utc"] = t0
    reg["experiment_clock_started_ms"] = t0_ms
    save_registration(root, reg)

    event = {
        "event_type": "PHASE1_DEPLOY",
        "protocol_id": protocol_id,
        "short_id": short_id,
        "schema_version": "1.0.0",
        "live_order_authority": 0.0,
        "timestamp_utc": deployed_utc,
        "timestamp_ms": deployed_ms,
        "experiment_t0_utc": t0,
        "experiment_t0_ms": t0_ms,
        "pre_deployment_capture_gap_ms": gap_ms,
        "pre_deployment_capture_gap_sec": round(gap_ms / 1000.0, 3),
        "preflight_ok": bool(preflight_ok),
        "receipt": receipt,
    }
    append_jsonl(root / "journals" / "RP611_clock_events.jsonl", event)
    append_jsonl(root / "journals" / "RP611_phase1_deploy_receipt.jsonl", event)
    return event
