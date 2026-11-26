#!/usr/bin/env python3
"""
RP-608-POLICY-D-PREOPEN-WS-CONT-001

Observational WebSocket continuity probe for NSE pre-open (default 09:00–09:15 IST).
Records per-symbol packet counts, depth validity, ages, and control messages.

Does NOT feed G3/G4 or change live trading rules. Authority: 0.0%.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import struct
import sys
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

import websockets

WORKSPACE_DIR = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_nse_shadow_daemon import NSE_119_SYMBOLS
from scripts.test_kite_probe import get_nse_instrument_map, load_credentials

IST = timezone(timedelta(hours=5, minutes=30))
OUT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse" / "preopen_ws_continuity"
OUT_DIR.mkdir(parents=True, exist_ok=True)


def decode_full(pkt: bytes) -> Optional[Dict[str, Any]]:
    if len(pkt) != 184:
        return None
    tok = struct.unpack(">I", pkt[0:4])[0]
    ltp = struct.unpack(">I", pkt[4:8])[0] / 100.0
    vol = struct.unpack(">I", pkt[16:20])[0]
    exch_ts = struct.unpack(">I", pkt[60:64])[0]
    bid_qty = struct.unpack(">I", pkt[64:68])[0]
    bid = struct.unpack(">I", pkt[68:72])[0] / 100.0
    ask_qty = struct.unpack(">I", pkt[124:128])[0]
    ask = struct.unpack(">I", pkt[128:132])[0] / 100.0
    return {
        "instrument_token": tok,
        "ltp": ltp,
        "volume": vol,
        "exchange_timestamp_sec": exch_ts,
        "best_bid": bid,
        "best_ask": ask,
        "bid_qty": bid_qty,
        "ask_qty": ask_qty,
        "valid_spread": bool(ask > bid > 0),
        "zero_depth": bool(bid == 0 or ask == 0),
        "crossed": bool(bid >= ask and bid > 0 and ask > 0),
    }


async def run_probe(*, until_ist: datetime) -> Dict[str, Any]:
    creds = load_credentials()
    api_key = creds.get("api_key") or ""
    access_token = creds.get("access_token") or ""
    if not api_key or not access_token:
        raise RuntimeError("MISSING_KITE_CREDENTIALS")

    instrument_map = get_nse_instrument_map()
    token_to_sym: Dict[int, str] = {}
    tokens: List[int] = []
    for full_sym in NSE_119_SYMBOLS:
        mapping = instrument_map.get(full_sym)
        if not mapping:
            continue
        tok = int(mapping[0])
        tokens.append(tok)
        token_to_sym[tok] = full_sym
    tokens = sorted(set(tokens))

    per: Dict[str, Dict[str, Any]] = {
        token_to_sym[t]: {
            "instrument_token": t,
            "packets": 0,
            "valid_spread_packets": 0,
            "zero_depth_packets": 0,
            "crossed_packets": 0,
            "first_receipt_mono": None,
            "last_receipt_mono": None,
            "last_exchange_ts": None,
            "last_ltp": None,
            "last_bid": None,
            "last_ask": None,
            "last_bid_qty": None,
            "last_ask_qty": None,
        }
        for t in tokens
        if t in token_to_sym
    }
    control_messages = 0
    binary_frames = 0
    reconnects = 0
    started = datetime.now(timezone.utc)
    url = f"wss://ws.kite.trade?api_key={api_key}&access_token={access_token}"

    async def _session() -> None:
        nonlocal control_messages, binary_frames
        async with websockets.connect(url, ping_interval=15, ping_timeout=10) as ws:
            await ws.send(json.dumps({"a": "subscribe", "v": tokens}))
            await ws.send(json.dumps({"a": "mode", "v": ["full", tokens]}))
            while datetime.now(IST) < until_ist:
                try:
                    msg = await asyncio.wait_for(ws.recv(), timeout=2.0)
                except asyncio.TimeoutError:
                    continue
                now_m = time.monotonic()
                if isinstance(msg, str):
                    control_messages += 1
                    continue
                if not isinstance(msg, bytes) or len(msg) < 2:
                    continue
                binary_frames += 1
                n = struct.unpack(">H", msg[:2])[0]
                off = 2
                for _ in range(n):
                    if off + 2 > len(msg):
                        break
                    plen = struct.unpack(">H", msg[off : off + 2])[0]
                    off += 2
                    pkt = msg[off : off + plen]
                    off += plen
                    if plen != 184:
                        continue
                    tick = decode_full(pkt)
                    if not tick:
                        continue
                    sym = token_to_sym.get(int(tick["instrument_token"]))
                    if not sym or sym not in per:
                        continue
                    row = per[sym]
                    row["packets"] += 1
                    if tick["valid_spread"]:
                        row["valid_spread_packets"] += 1
                    if tick["zero_depth"]:
                        row["zero_depth_packets"] += 1
                    if tick["crossed"]:
                        row["crossed_packets"] += 1
                    if row["first_receipt_mono"] is None:
                        row["first_receipt_mono"] = now_m
                    row["last_receipt_mono"] = now_m
                    row["last_exchange_ts"] = tick["exchange_timestamp_sec"]
                    row["last_ltp"] = tick["ltp"]
                    row["last_bid"] = tick["best_bid"]
                    row["last_ask"] = tick["best_ask"]
                    row["last_bid_qty"] = tick["bid_qty"]
                    row["last_ask_qty"] = tick["ask_qty"]

    # reconnect loop until wall clock
    while datetime.now(IST) < until_ist:
        try:
            await _session()
            break
        except Exception as exc:
            reconnects += 1
            print(f"[preopen-ws] reconnect after error: {type(exc).__name__}: {exc}")
            await asyncio.sleep(2.0)

    ended = datetime.now(timezone.utc)
    symbols_with_packets = sum(1 for r in per.values() if r["packets"] > 0)
    symbols_with_valid = sum(1 for r in per.values() if r["valid_spread_packets"] > 0)
    symbols_all_zero_depth = sum(
        1 for r in per.values() if r["packets"] > 0 and r["zero_depth_packets"] == r["packets"]
    )

    report = {
        "protocol_id": "RP-608-POLICY-D-PREOPEN-WS-CONT-001",
        "authority": "0.0%",
        "session_date_ist": datetime.now(IST).date().isoformat(),
        "started_utc": started.isoformat(),
        "ended_utc": ended.isoformat(),
        "until_ist": until_ist.isoformat(),
        "subscribed_tokens": len(tokens),
        "binary_frames": binary_frames,
        "control_messages": control_messages,
        "reconnects": reconnects,
        "symbols_with_packets": symbols_with_packets,
        "symbols_with_valid_spread": symbols_with_valid,
        "symbols_all_zero_depth_when_seen": symbols_all_zero_depth,
        "coverage_pct_packets": round(100.0 * symbols_with_packets / max(1, len(per)), 2),
        "per_symbol": per,
    }
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description="Pre-open WS continuity probe (0% authority)")
    parser.add_argument("--until-ist", default="09:15:00", help="Stop time HH:MM:SS IST")
    parser.add_argument("--duration-sec", type=float, default=None, help="Override: run N seconds from now")
    args = parser.parse_args()

    now_ist = datetime.now(IST)
    if args.duration_sec is not None:
        until = now_ist + timedelta(seconds=float(args.duration_sec))
    else:
        hh, mm, ss = [int(x) for x in str(args.until_ist).split(":")]
        until = now_ist.replace(hour=hh, minute=mm, second=ss, microsecond=0)
        if until <= now_ist:
            print(f"[preopen-ws] until {until.isoformat()} already passed at {now_ist.isoformat()}; abort")
            return 2

    print(f"[preopen-ws] running until {until.isoformat()} (now {now_ist.isoformat()})")
    report = asyncio.run(run_probe(until_ist=until))
    out = OUT_DIR / f"preopen_ws_continuity_{report['session_date_ist']}.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    summary = {k: report[k] for k in report if k != "per_symbol"}
    print(json.dumps(summary, indent=2))
    print(f"[wrote] {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
