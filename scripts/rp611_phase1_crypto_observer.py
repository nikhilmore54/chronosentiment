#!/usr/bin/env python3
"""
RP-611C Phase-1 observer — BTC/USDT spot integrity + absorption capture.

Volume: Binance SPOT kline 15m only (futures/perpetual excluded).
L1: spot bookTicker snapshot (observational; does not mutate existing OF).
Journals: artifacts/rp611_crypto_volume_absorption/journals/ only.
Live order authority: 0%.
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.rp611_absorption_core import (
    LIVE_ORDER_AUTHORITY,
    SCHEMA_VERSION,
    Candle,
    candle_from_binance_kline,
    classify_absorption,
    confirm_absorption,
    control_flags,
)
from scripts.rp611_phase1_common import (
    CRYPTO_ROOT,
    append_jsonl,
    load_cursor,
    load_registration,
    save_cursor,
    utc_ms,
    utc_now,
)

PROTOCOL_ID = "RP-611-CRYPTO-VOLUME-ABSORPTION-001"
SHORT_ID = "RP-611C"
SPOT_KLINE_URL = (
    "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=15m&limit=100"
)
SPOT_BOOK_URL = "https://api.binance.com/api/v3/ticker/bookTicker?symbol=BTCUSDT"
CURSOR_PATH = CRYPTO_ROOT / "state" / "phase1_cursor.json"


def _http_json(url: str, timeout: float = 15.0) -> Any:
    req = urllib.request.Request(url, headers={"User-Agent": "RP611C-Phase1/1.0"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def fetch_spot_klines() -> List[List[Any]]:
    rows = _http_json(SPOT_KLINE_URL)
    if not isinstance(rows, list):
        raise RuntimeError("spot_kline_not_list")
    # Drop forming candle (last row may still be open); keep only fully closed:
    # Binance returns close_time in index 6; if close_time > now, drop.
    now = utc_ms()
    completed = [r for r in rows if int(r[6]) < now]
    return completed


def fetch_spot_book() -> Dict[str, Any]:
    raw = _http_json(SPOT_BOOK_URL)
    bid_q = float(raw.get("bidQty") or 0.0)
    ask_q = float(raw.get("askQty") or 0.0)
    den = bid_q + ask_q
    imb = ((bid_q - ask_q) / den) if den > 0 else None
    return {
        "bid_price": float(raw.get("bidPrice") or 0.0),
        "ask_price": float(raw.get("askPrice") or 0.0),
        "bid_qty": bid_q,
        "ask_qty": ask_q,
        "imbalance": imb,
        "source": "binance_spot_bookTicker",
        "futures_excluded": True,
    }


def preflight() -> Dict[str, Any]:
    reasons: List[str] = []
    ok = True
    kline_n = 0
    book = None
    try:
        rows = fetch_spot_klines()
        kline_n = len(rows)
        if kline_n < 25:
            ok = False
            reasons.append("insufficient_spot_klines")
        # sanity: volume field present
        if rows and float(rows[-1][5]) < 0:
            ok = False
            reasons.append("negative_volume")
    except Exception as e:
        ok = False
        reasons.append(f"spot_kline_fetch_failed:{e}")
    try:
        book = fetch_spot_book()
        if book.get("imbalance") is None:
            ok = False
            reasons.append("bookticker_imbalance_unavailable")
    except Exception as e:
        ok = False
        reasons.append(f"spot_bookticker_failed:{e}")
    return {
        "ok": ok,
        "reasons": reasons,
        "spot_klines_completed": kline_n,
        "book": book,
        "volume_source": "binance_spot_kline_15m",
        "futures_perpetual_volume": "excluded",
    }


def run_once() -> Dict[str, Any]:
    reg = load_registration(CRYPTO_ROOT)
    t0_ms = int(reg.get("experiment_clock_started_ms") or 0)
    rows = fetch_spot_klines()
    book = fetch_spot_book()
    candles = [candle_from_binance_kline(r) for r in rows]
    cursor = load_cursor(CURSOR_PATH)
    last_ms = int(cursor.get("last_open_time_ms") or 0)
    # Never process before T0; also never reprocess
    floor_ms = max(t0_ms, last_ms)

    n_int = n_cand = n_conf = 0
    max_ms = last_ms

    # L1 telemetry snapshot each run
    append_jsonl(
        CRYPTO_ROOT / "journals" / "RP611C_l1_telemetry.jsonl",
        {
            "event_type": "L1_SNAPSHOT",
            "protocol_id": PROTOCOL_ID,
            "schema_version": SCHEMA_VERSION,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "timestamp_utc": utc_now(),
            "timestamp_ms": utc_ms(),
            "book": book,
            "note": "Observational only; does not mutate existing OF lifecycle",
        },
    )

    for i, c in enumerate(candles):
        if c.open_time_ms <= floor_ms:
            continue
        if c.open_time_ms < t0_ms:
            continue
        max_ms = max(max_ms, c.open_time_ms)
        prior_vols = [x.volume for x in candles[:i]]
        append_jsonl(
            CRYPTO_ROOT / "journals" / "RP611C_daily_log.jsonl",
            {
                "event_type": "BAR_INTEGRITY",
                "protocol_id": PROTOCOL_ID,
                "schema_version": SCHEMA_VERSION,
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "symbol": "BTCUSDT",
                "open_time_ms": c.open_time_ms,
                "volume": c.volume,
                "ohlc": [c.open, c.high, c.low, c.close],
                "source": c.source,
                "volume_market": "spot",
                "timestamp_utc": utc_now(),
                "timestamp_ms": utc_ms(),
            },
        )
        n_int += 1
        cls, detail = classify_absorption(c, prior_vols)
        flags = control_flags(c, prior_vols)
        if cls is None:
            continue
        cand_id = f"{SHORT_ID}-BTCUSDT-{c.open_time_ms}-{cls}"
        # Arm tagging (A/B/C geometry; L1 support at capture time)
        imb = book.get("imbalance")
        l1_long = imb is not None and imb >= 0.30
        l1_short = imb is not None and imb <= -0.30
        direction = "LONG" if cls == "ACCUMULATION_LONG" else "SHORT"
        same_l1 = (direction == "LONG" and l1_long) or (direction == "SHORT" and l1_short)
        cand = {
            "event_type": "ABSORPTION_CANDIDATE",
            "protocol_id": PROTOCOL_ID,
            "schema_version": SCHEMA_VERSION,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "candidate_id": cand_id,
            "symbol": "BTCUSDT",
            "direction": direction,
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
            "l1_at_capture": book,
            "arms": {
                "A_absorption_only": True,
                "B_l1_only_context": bool(l1_long or l1_short),
                "C_absorption_plus_same_l1": bool(same_l1),
                "D_pending_confirmation": True,
            },
            "timestamp_utc": utc_now(),
            "timestamp_ms": utc_ms(),
        }
        append_jsonl(CRYPTO_ROOT / "journals" / "RP611C_candidates.jsonl", cand)
        append_jsonl(CRYPTO_ROOT / "journals" / "RP611C_arms.jsonl", {**cand, "event_type": "ARM_TAG"})
        n_cand += 1
        if i + 1 < len(candles):
            nxt = candles[i + 1]
            if nxt.open_time_ms <= floor_ms and nxt.open_time_ms < c.open_time_ms:
                pass
            conf = confirm_absorption(cls, c, nxt)
            row = {
                "event_type": "ABSORPTION_CONFIRMATION",
                "protocol_id": PROTOCOL_ID,
                "schema_version": SCHEMA_VERSION,
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "candidate_id": cand_id,
                "symbol": "BTCUSDT",
                "direction": direction,
                "absorption_open_time_ms": c.open_time_ms,
                "confirm_open_time_ms": nxt.open_time_ms,
                "passed": conf["passed"],
                "midpoint": conf["midpoint"],
                "next_close": conf["next_close"],
                "reject_reason": conf["reject_reason"],
                "l1_at_capture": book,
                "arms": {
                    "A": True,
                    "C": bool(same_l1),
                    "D": bool(conf["passed"]),
                },
                "timestamp_utc": utc_now(),
                "timestamp_ms": utc_ms(),
            }
            append_jsonl(CRYPTO_ROOT / "journals" / "RP611C_confirmations.jsonl", row)
            n_conf += 1

    if max_ms > last_ms:
        save_cursor(
            CURSOR_PATH,
            {
                "last_open_time_ms": max_ms,
                "updated_utc": utc_now(),
                "protocol_id": PROTOCOL_ID,
                "volume_source": "binance_spot_kline_15m",
            },
        )
    return {
        "integrity_rows": n_int,
        "candidates": n_cand,
        "confirmations": n_conf,
        "klines_completed": len(candles),
        "floor_ms": floor_ms,
        "t0_ms": t0_ms,
        "book_imbalance": book.get("imbalance"),
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
