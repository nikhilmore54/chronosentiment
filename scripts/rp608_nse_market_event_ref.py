#!/usr/bin/env python3
"""
RP-608 NSE P0 — Python behavioural reference for market-event integrity.

Mirrors adapters/chronosentiment/src/nse_market_events (Rust).
Not a production writer. Live authority 0%. No OF strategy logic.
"""

from __future__ import annotations

import json
import struct
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

_ROOT = Path(__file__).resolve().parents[1]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.rp608_kite_ticker_cache import decode_full_mode_packet
from scripts.rp608_nse_of_book_depth_history import snapshot_row_from_tick

FULL_MODE_PACKET_LEN = 184


def encode_full_mode_packet(
    *,
    instrument_token: int,
    last_price: float,
    volume: int,
    exchange_timestamp_sec: int,
    depth_buy: Sequence[Tuple[float, int]],
    depth_sell: Sequence[Tuple[float, int]],
) -> bytes:
    """Build a 184-byte full-mode body matching the Python decoder layout."""
    buf = bytearray(FULL_MODE_PACKET_LEN)
    struct.pack_into(">I", buf, 0, int(instrument_token))
    struct.pack_into(">I", buf, 4, int(round(float(last_price) * 100)))
    struct.pack_into(">I", buf, 16, int(volume))
    struct.pack_into(">I", buf, 60, int(exchange_timestamp_sec))
    for i in range(5):
        px, qty = depth_buy[i] if i < len(depth_buy) else (0.0, 0)
        off = 64 + i * 12
        struct.pack_into(">I", buf, off, int(qty))
        struct.pack_into(">I", buf, off + 4, int(round(float(px) * 100)))
    for i in range(5):
        px, qty = depth_sell[i] if i < len(depth_sell) else (0.0, 0)
        off = 124 + i * 12
        struct.pack_into(">I", buf, off, int(qty))
        struct.pack_into(">I", buf, off + 4, int(round(float(px) * 100)))
    return bytes(buf)


def encode_full_mode_frame(packets: Sequence[bytes]) -> bytes:
    parts = [struct.pack(">H", len(packets))]
    for pkt in packets:
        parts.append(struct.pack(">H", len(pkt)))
        parts.append(pkt)
    return b"".join(parts)


def fnv1a64_fields(tick: Dict[str, Any]) -> int:
    """Match Rust content_fingerprint mixing (FNV-1a over f64 bits)."""
    h = 0xCBF29CE484222325

    def mix_u64(x: int) -> None:
        nonlocal h
        h ^= x & 0xFFFFFFFFFFFFFFFF
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF

    def f64_bits(x: float) -> int:
        return struct.unpack(">Q", struct.pack(">d", float(x)))[0]

    mix_u64(int(tick["instrument_token"]))
    mix_u64(int(tick["exchange_timestamp_sec"]))
    mix_u64(f64_bits(tick["last_price"]))
    mix_u64(f64_bits(tick["volume"]))
    for side in ("depth_buy", "depth_sell"):
        for lvl in tick[side]:
            mix_u64(f64_bits(lvl["price"]))
            mix_u64(f64_bits(lvl["quantity"]))
    return h


@dataclass
class FeedHealthRef:
    max_quote_age_ms: float = 60_000.0
    status: str = "live"
    stale_reason: Optional[str] = None
    stale_tokens: set = field(default_factory=set)
    last_exchange_ms: Dict[int, int] = field(default_factory=dict)

    def on_disconnect(self, tokens: Sequence[int]) -> None:
        self.status = "stale"
        self.stale_reason = "disconnect"
        self.stale_tokens.update(int(t) for t in tokens)

    def begin_reconcile(self) -> None:
        self.status = "reconciling"

    def complete_reconcile(self) -> None:
        if self.status == "reconciling":
            self.status = "live"
            self.stale_reason = None
            self.stale_tokens.clear()

    def note_age(self, token: int, age_ms: float) -> None:
        if age_ms > self.max_quote_age_ms:
            self.status = "stale"
            self.stale_reason = "quote_age_exceeded"
            self.stale_tokens.add(int(token))

    def token_stale(self, token: int) -> bool:
        return self.status != "live" or int(token) in self.stale_tokens


@dataclass
class MarketEventEngineRef:
    health: FeedHealthRef = field(default_factory=FeedHealthRef)
    next_seq: int = 1
    subscribed: set = field(default_factory=set)
    recent_fp: Dict[int, List[int]] = field(default_factory=dict)
    dedupe_window: int = 64
    accepted: List[Dict[str, Any]] = field(default_factory=list)
    depth_history: List[Dict[str, Any]] = field(default_factory=list)
    dropped_duplicates: int = 0

    def on_disconnect(self) -> None:
        self.health.on_disconnect(list(self.subscribed))

    def begin_reconcile(self) -> None:
        self.health.begin_reconcile()

    def complete_reconcile(self) -> None:
        self.health.complete_reconcile()

    def ingest_packet(
        self,
        pkt: bytes,
        *,
        receipt_utc_ms: int,
        symbol: str = "UNKNOWN",
        source: str = "fixture_replay",
    ) -> Optional[Dict[str, Any]]:
        tick = decode_full_mode_packet(pkt)
        if not tick:
            return None
        tok = int(tick["instrument_token"])
        self.subscribed.add(tok)
        exch_ms = int(tick["exchange_timestamp_sec"]) * 1000
        tick["receipt_utc_ms"] = int(receipt_utc_ms)
        tick["quote_age_ms"] = max(0.0, float(receipt_utc_ms - exch_ms))
        tick["quote_source"] = source
        self.health.note_age(tok, float(tick["quote_age_ms"]))

        fp = fnv1a64_fields(tick)
        window = self.recent_fp.setdefault(tok, [])
        if fp in window:
            self.dropped_duplicates += 1
            return None
        window.append(fp)
        while len(window) > self.dedupe_window:
            window.pop(0)

        ingest_seq = self.next_seq
        self.next_seq += 1
        observed_while_stale = self.health.token_stale(tok)
        snap = snapshot_row_from_tick(symbol=symbol, tick=tick, sample_utc_ms=receipt_utc_ms)
        snap["ingest_seq"] = ingest_seq
        snap["content_fp"] = fp
        # Align source string with Rust enum for fixture_replay
        if source == "fixture_replay":
            snap["quote_source"] = "fixture_replay"
        row = {
            "event": {
                "instrument_token": tok,
                "last_price": tick["last_price"],
                "volume": tick["volume"],
                "exchange_timestamp_sec": tick["exchange_timestamp_sec"],
                "exchange_timestamp_utc_ms": exch_ms,
                "receipt_utc_ms": receipt_utc_ms,
                "quote_age_ms": tick["quote_age_ms"],
                "best_bid": tick["best_bid"],
                "best_ask": tick["best_ask"],
                "bid_qty": tick["bid_qty"],
                "ask_qty": tick["ask_qty"],
                "depth_buy": tick["depth_buy"],
                "depth_sell": tick["depth_sell"],
                "source": source,
                "ingest_seq": ingest_seq,
                "content_fp": fp,
            },
            "depth_snapshot": snap,
            "observed_while_stale": observed_while_stale,
        }
        self.accepted.append(row)
        self.depth_history.append(snap)
        return row


def write_p0_fixtures(out_dir: Path) -> Path:
    """Generate recorded-style packets + Python golden outputs for Rust differential."""
    out_dir.mkdir(parents=True, exist_ok=True)
    buy = [(100.0 - i * 0.05, 10 * (i + 1)) for i in range(5)]
    sell = [(100.1 + i * 0.05, 12 * (i + 1)) for i in range(5)]
    pkt_a = encode_full_mode_packet(
        instrument_token=6401,
        last_price=100.05,
        volume=12345,
        exchange_timestamp_sec=1_700_000_000,
        depth_buy=buy,
        depth_sell=sell,
    )
    pkt_b = encode_full_mode_packet(
        instrument_token=6401,
        last_price=100.10,
        volume=12350,
        exchange_timestamp_sec=1_700_000_001,
        depth_buy=[(100.05 - i * 0.05, 11 * (i + 1)) for i in range(5)],
        depth_sell=[(100.15 + i * 0.05, 13 * (i + 1)) for i in range(5)],
    )
    # Duplicate of A (dedupe)
    pkt_a_dup = pkt_a
    # Second symbol
    pkt_c = encode_full_mode_packet(
        instrument_token=738561,
        last_price=2500.0,
        volume=99,
        exchange_timestamp_sec=1_700_000_000,
        depth_buy=[(2499.5 - i, 5) for i in range(5)],
        depth_sell=[(2500.5 + i, 6) for i in range(5)],
    )

    frame = encode_full_mode_frame([pkt_a, pkt_b])
    engine = MarketEventEngineRef()
    receipt0 = 1_700_000_000_500
    engine.ingest_packet(pkt_a, receipt_utc_ms=receipt0, symbol="TEST.NS")
    engine.ingest_packet(pkt_a_dup, receipt_utc_ms=receipt0 + 1, symbol="TEST.NS")
    engine.ingest_packet(pkt_b, receipt_utc_ms=receipt0 + 1_000, symbol="TEST.NS")
    engine.on_disconnect()
    stale_try = engine.health.status
    engine.begin_reconcile()
    recon_status = engine.health.status
    engine.complete_reconcile()
    engine.ingest_packet(pkt_c, receipt_utc_ms=receipt0 + 2_000, symbol="RELIANCE.NS")

    # Age-stale packet
    pkt_old = encode_full_mode_packet(
        instrument_token=6401,
        last_price=99.0,
        volume=1,
        exchange_timestamp_sec=1_700_000_000 - 120,
        depth_buy=buy,
        depth_sell=sell,
    )
    engine_age = MarketEventEngineRef()
    engine_age.ingest_packet(
        pkt_old, receipt_utc_ms=1_700_000_000_000, symbol="TEST.NS"
    )

    golden = {
        "protocol": "RP608-NSE-P0-MARKET-EVENTS",
        "packets": {
            "pkt_a_hex": pkt_a.hex(),
            "pkt_b_hex": pkt_b.hex(),
            "pkt_c_hex": pkt_c.hex(),
            "pkt_old_hex": pkt_old.hex(),
            "frame_ab_hex": frame.hex(),
        },
        "python_engine": {
            "accepted": engine.accepted,
            "depth_history_len": len(engine.depth_history),
            "dropped_duplicates": engine.dropped_duplicates,
            "ordered_tokens_prices": [
                (a["event"]["instrument_token"], a["event"]["last_price"], a["event"]["ingest_seq"])
                for a in sorted(
                    engine.accepted,
                    key=lambda r: (
                        r["event"]["exchange_timestamp_utc_ms"],
                        r["event"]["ingest_seq"],
                    ),
                )
            ],
            "after_disconnect_status": stale_try,
            "after_begin_reconcile": recon_status,
            "after_complete_reconcile": engine.health.status,
            "final_feed_status": engine.health.status,
        },
        "age_stale": {
            "status": engine_age.health.status,
            "reason": engine_age.health.stale_reason,
            "observed_while_stale": engine_age.accepted[0]["observed_while_stale"]
            if engine_age.accepted
            else None,
            "quote_age_ms": engine_age.accepted[0]["event"]["quote_age_ms"]
            if engine_age.accepted
            else None,
        },
        "decoded_pkt_a": decode_full_mode_packet(pkt_a),
    }
    path = out_dir / "p0_kite_full_differential.json"
    path.write_text(json.dumps(golden, indent=2) + "\n", encoding="utf-8")
    return path


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[1]
    out = root / "adapters" / "chronosentiment" / "tests" / "fixtures" / "nse_kite_full"
    p = write_p0_fixtures(out)
    print(p)
