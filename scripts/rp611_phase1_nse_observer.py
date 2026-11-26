#!/usr/bin/env python3
"""
RP-611N Phase-1 observer — NSE integrity + absorption candidate/confirmation capture.

Reads completed bars from rp608_nse_bar_history.json (read-only).
Writes only to artifacts/rp611_nse_volume_absorption/journals/.
Does not mutate RP-608 daemons, OF, or Gate5.
Live order authority: 0%.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.rp611_absorption_core import (
    LIVE_ORDER_AUTHORITY,
    SCHEMA_VERSION,
    Candle,
    candle_from_nse_bar,
    classify_absorption,
    confirm_absorption,
    control_flags,
)
from scripts.rp611_phase1_common import (
    NSE_ROOT,
    WORKSPACE,
    append_jsonl,
    load_cursor,
    load_registration,
    save_cursor,
    utc_ms,
    utc_now,
)

PROTOCOL_ID = "RP-611-NSE-VOLUME-ABSORPTION-001"
SHORT_ID = "RP-611N"
BAR_HISTORY = (
    WORKSPACE / "live_capture" / "shadow_validation" / "nse" / "rp608_nse_bar_history.json"
)
CURSOR_PATH = NSE_ROOT / "state" / "phase1_cursor.json"


def preflight() -> Dict[str, Any]:
    ok = True
    reasons: List[str] = []
    if not BAR_HISTORY.exists():
        ok = False
        reasons.append("missing_bar_history")
        return {"ok": ok, "reasons": reasons, "symbols": 0, "bars": 0}
    try:
        data = json.loads(BAR_HISTORY.read_text(encoding="utf-8"))
    except Exception as e:
        return {"ok": False, "reasons": [f"bar_history_unreadable:{e}"], "symbols": 0, "bars": 0}
    if not isinstance(data, dict) or len(data) < 100:
        ok = False
        reasons.append(f"universe_size_{len(data) if isinstance(data, dict) else 0}")
    n_bars = 0
    completed = 0
    for sym, bars in (data or {}).items():
        if not isinstance(bars, list):
            continue
        n_bars += len(bars)
        completed += sum(1 for b in bars if isinstance(b, dict) and b.get("is_completed") is not False)
    if completed < 20:
        ok = False
        reasons.append("insufficient_completed_bars")
    # volume present?
    sample_ok = False
    for bars in list(data.values())[:5]:
        for b in bars[-3:]:
            if float((b or {}).get("volume") or 0) > 0:
                sample_ok = True
                break
    if not sample_ok:
        ok = False
        reasons.append("volume_missing_in_sample")
    return {
        "ok": ok,
        "reasons": reasons,
        "symbols": len(data) if isinstance(data, dict) else 0,
        "bars": n_bars,
        "completed_bars": completed,
        "source": str(BAR_HISTORY),
    }


def _process_symbol(
    symbol: str,
    bars: List[Dict[str, Any]],
    last_ms: int,
    t0_ms: int,
) -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]], List[Dict[str, Any]], int]:
    integrity: List[Dict[str, Any]] = []
    candidates: List[Dict[str, Any]] = []
    confirmations: List[Dict[str, Any]] = []
    candles: List[Candle] = []
    for b in bars:
        c = candle_from_nse_bar(b)
        if c is None:
            continue
        candles.append(c)

    floor_ms = max(last_ms, t0_ms)
    max_ms = last_ms
    for i, c in enumerate(candles):
        if c.open_time_ms <= floor_ms:
            continue
        max_ms = max(max_ms, c.open_time_ms)
        prior_vols = [x.volume for x in candles[:i]]
        # integrity row for each newly seen completed bar
        integrity.append(
            {
                "event_type": "BAR_INTEGRITY",
                "protocol_id": PROTOCOL_ID,
                "schema_version": SCHEMA_VERSION,
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "symbol": symbol,
                "open_time_ms": c.open_time_ms,
                "volume": c.volume,
                "ohlc": [c.open, c.high, c.low, c.close],
                "source": c.source,
                "timestamp_utc": utc_now(),
                "timestamp_ms": utc_ms(),
            }
        )
        cls, detail = classify_absorption(c, prior_vols)
        flags = control_flags(c, prior_vols)
        if cls is None and not flags["C_absorption"]:
            # still record control-eligible colour/rvol for diagnostics in session log only via counts
            pass
        if cls is not None:
            cand_id = f"{SHORT_ID}-{symbol}-{c.open_time_ms}-{cls}"
            cand = {
                "event_type": "ABSORPTION_CANDIDATE",
                "protocol_id": PROTOCOL_ID,
                "schema_version": SCHEMA_VERSION,
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "candidate_id": cand_id,
                "symbol": symbol,
                "direction": "LONG" if cls == "ACCUMULATION_LONG" else "SHORT",
                "classification": cls,
                "open_time_ms": c.open_time_ms,
                "ohlcv": {
                    "open": c.open,
                    "high": c.high,
                    "low": c.low,
                    "close": c.close,
                    "volume": c.volume,
                },
                "metrics": detail,
                "controls": flags,
                "timestamp_utc": utc_now(),
                "timestamp_ms": utc_ms(),
            }
            candidates.append(cand)
            # confirmation requires next completed candle
            if i + 1 < len(candles):
                nxt = candles[i + 1]
                conf = confirm_absorption(cls, c, nxt)
                confirmations.append(
                    {
                        "event_type": "ABSORPTION_CONFIRMATION",
                        "protocol_id": PROTOCOL_ID,
                        "schema_version": SCHEMA_VERSION,
                        "live_order_authority": LIVE_ORDER_AUTHORITY,
                        "candidate_id": cand_id,
                        "symbol": symbol,
                        "direction": cand["direction"],
                        "absorption_open_time_ms": c.open_time_ms,
                        "confirm_open_time_ms": nxt.open_time_ms,
                        "passed": conf["passed"],
                        "midpoint": conf["midpoint"],
                        "next_close": conf["next_close"],
                        "reject_reason": conf["reject_reason"],
                        "cohort_D_eligible": bool(conf["passed"]),
                        "cohort_C_candidate": True,
                        "timestamp_utc": utc_now(),
                        "timestamp_ms": utc_ms(),
                    }
                )
    return integrity, candidates, confirmations, max_ms


def run_once() -> Dict[str, Any]:
    reg = load_registration(NSE_ROOT)
    t0_ms = int(reg.get("experiment_clock_started_ms") or 0)
    data = json.loads(BAR_HISTORY.read_text(encoding="utf-8"))
    cursor = load_cursor(CURSOR_PATH)
    per_sym = cursor.get("last_open_time_ms_by_symbol") or {}
    n_int = n_cand = n_conf = 0
    n_pass = 0
    new_cursor = dict(per_sym)
    # One census row per run (integrity of source file, including pre-T0 history)
    max_bar_ms = 0
    for bars in data.values():
        if isinstance(bars, list):
            for b in bars:
                max_bar_ms = max(max_bar_ms, int((b or {}).get("timestamp_utc_ms") or 0))
    append_jsonl(
        NSE_ROOT / "journals" / "RP611N_session_log.jsonl",
        {
            "event_type": "SOURCE_CENSUS",
            "protocol_id": PROTOCOL_ID,
            "schema_version": SCHEMA_VERSION,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "symbols": len(data),
            "max_bar_open_time_ms": max_bar_ms,
            "t0_ms": t0_ms,
            "source_covers_post_t0": bool(max_bar_ms >= t0_ms),
            "source": str(BAR_HISTORY),
            "timestamp_utc": utc_now(),
            "timestamp_ms": utc_ms(),
        },
    )
    for symbol, bars in data.items():
        if not isinstance(bars, list):
            continue
        last = int(per_sym.get(symbol) or 0)
        integ, cands, confs, max_ms = _process_symbol(symbol, bars, last, t0_ms)
        for row in integ:
            append_jsonl(NSE_ROOT / "journals" / "RP611N_session_log.jsonl", row)
        for row in cands:
            append_jsonl(NSE_ROOT / "journals" / "RP611N_candidates.jsonl", row)
        for row in confs:
            append_jsonl(NSE_ROOT / "journals" / "RP611N_confirmations.jsonl", row)
            # control cohort D/C markers
            append_jsonl(
                NSE_ROOT / "journals" / "RP611N_controls.jsonl",
                {
                    **row,
                    "event_type": "CONTROL_OUTCOME",
                    "cohorts": {
                        "C": True,
                        "D": bool(row.get("passed")),
                    },
                },
            )
        n_int += len(integ)
        n_cand += len(cands)
        n_conf += len(confs)
        n_pass += sum(1 for r in confs if r.get("passed"))
        if max_ms > last:
            new_cursor[symbol] = max_ms
    save_cursor(
        CURSOR_PATH,
        {
            "last_open_time_ms_by_symbol": new_cursor,
            "updated_utc": utc_now(),
            "protocol_id": PROTOCOL_ID,
            "t0_ms": t0_ms,
        },
    )
    return {
        "symbols_scanned": len(data),
        "integrity_rows": n_int,
        "candidates": n_cand,
        "confirmations": n_conf,
        "passed_confirmations": n_pass,
        "t0_ms": t0_ms,
        "source_covers_post_t0": bool(max_bar_ms >= t0_ms),
        "max_bar_open_time_ms": max_bar_ms,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--preflight-only", action="store_true")
    ap.add_argument("--run", action="store_true")
    args = ap.parse_args()
    pf = preflight()
    print(json.dumps({"preflight": pf}, indent=2))
    if args.preflight_only or not args.run:
        return 0 if pf.get("ok") else 2
    if not pf.get("ok"):
        return 2
    result = run_once()
    print(json.dumps({"run": result}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
