#!/usr/bin/env python3
"""Unit tests for Kite full-mode packet decode (no live network)."""

from __future__ import annotations

import struct
import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_kite_ticker_cache import decode_full_mode_packet


def _pack_full_packet(
    *,
    token: int = 6401,
    ltp_paise: int = 250500,
    ltq: int = 7,
    atp_paise: int = 250450,
    ltt_sec: int = 1_793_247_100,
    bid_paise: int = 250400,
    ask_paise: int = 250500,
    exch_ts: int = 1_793_247_000,
) -> bytes:
    pkt = bytearray(184)
    struct.pack_into(">I", pkt, 0, token)
    struct.pack_into(">I", pkt, 4, ltp_paise)
    struct.pack_into(">I", pkt, 8, ltq)
    struct.pack_into(">I", pkt, 12, atp_paise)
    struct.pack_into(">I", pkt, 16, 1000)  # volume
    struct.pack_into(">I", pkt, 44, ltt_sec)
    struct.pack_into(">I", pkt, 60, exch_ts)
    # five bid levels
    for i, qty in enumerate((10, 20, 30, 40, 50)):
        off = 64 + i * 12
        struct.pack_into(">I", pkt, off, qty)
        struct.pack_into(">I", pkt, off + 4, bid_paise - i)
    # five ask levels
    for i, qty in enumerate((12, 22, 32, 42, 52)):
        off = 124 + i * 12
        struct.pack_into(">I", pkt, off, qty)
        struct.pack_into(">I", pkt, off + 4, ask_paise + i)
    return bytes(pkt)


class TestKiteTickerDecode(unittest.TestCase):
    def test_decode_full_mode_packet(self):
        tick = decode_full_mode_packet(_pack_full_packet())
        self.assertIsNotNone(tick)
        assert tick is not None
        self.assertEqual(tick["instrument_token"], 6401)
        self.assertAlmostEqual(tick["last_price"], 2505.0)
        self.assertAlmostEqual(tick["best_bid"], 2504.0)
        self.assertAlmostEqual(tick["best_ask"], 2505.0)
        self.assertEqual(tick["bid_qty"], 10)
        self.assertEqual(tick["ask_qty"], 12)
        self.assertEqual(tick["last_traded_quantity"], 7)
        self.assertEqual(tick["last_trade_time_sec"], 1_793_247_100)
        self.assertAlmostEqual(tick["average_price"], 2504.5)

    def test_decode_retains_five_level_depth(self):
        tick = decode_full_mode_packet(_pack_full_packet())
        assert tick is not None
        self.assertEqual(len(tick["depth_buy"]), 5)
        self.assertEqual(len(tick["depth_sell"]), 5)
        self.assertEqual(tick["depth_buy"][4]["quantity"], 50)
        self.assertEqual(tick["depth_sell"][4]["quantity"], 52)

    def test_reject_short_packet(self):
        self.assertIsNone(decode_full_mode_packet(b"\x00" * 10))


if __name__ == "__main__":
    unittest.main()
