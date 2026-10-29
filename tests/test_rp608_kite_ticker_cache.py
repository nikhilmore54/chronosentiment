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
    bid_paise: int = 250400,
    ask_paise: int = 250500,
    exch_ts: int = 1_793_247_000,
) -> bytes:
    pkt = bytearray(184)
    struct.pack_into(">I", pkt, 0, token)
    struct.pack_into(">I", pkt, 4, ltp_paise)
    struct.pack_into(">I", pkt, 16, 1000)  # volume
    struct.pack_into(">I", pkt, 60, exch_ts)
    # best bid
    struct.pack_into(">I", pkt, 64, 10)
    struct.pack_into(">I", pkt, 68, bid_paise)
    # best ask
    struct.pack_into(">I", pkt, 124, 12)
    struct.pack_into(">I", pkt, 128, ask_paise)
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

    def test_reject_short_packet(self):
        self.assertIsNone(decode_full_mode_packet(b"\x00" * 10))


if __name__ == "__main__":
    unittest.main()
