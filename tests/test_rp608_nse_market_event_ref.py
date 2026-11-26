#!/usr/bin/env python3
"""P0 Python reference — decode/dedupe/stale parity fixtures."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.rp608_kite_ticker_cache import decode_full_mode_packet
from scripts.rp608_nse_market_event_ref import (
    MarketEventEngineRef,
    encode_full_mode_packet,
    write_p0_fixtures,
)

FIXTURE = (
    ROOT
    / "adapters"
    / "chronosentiment"
    / "tests"
    / "fixtures"
    / "nse_kite_full"
    / "p0_kite_full_differential.json"
)


class TestNseMarketEventRefP0(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        write_p0_fixtures(FIXTURE.parent)
        cls.fix = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_decode_roundtrip(self):
        pkt = bytes.fromhex(self.fix["packets"]["pkt_a_hex"])
        tick = decode_full_mode_packet(pkt)
        self.assertIsNotNone(tick)
        self.assertEqual(tick["instrument_token"], 6401)
        self.assertAlmostEqual(tick["last_price"], 100.05)
        self.assertEqual(len(tick["depth_buy"]), 5)
        self.assertEqual(len(tick["depth_sell"]), 5)

    def test_dedupe_and_disconnect_reconcile(self):
        eng = self.fix["python_engine"]
        self.assertEqual(eng["dropped_duplicates"], 1)
        self.assertEqual(len(eng["accepted"]), 3)
        self.assertEqual(eng["after_disconnect_status"], "stale")
        self.assertEqual(eng["after_begin_reconcile"], "reconciling")
        self.assertEqual(eng["after_complete_reconcile"], "live")

    def test_age_stale(self):
        self.assertEqual(self.fix["age_stale"]["status"], "stale")
        self.assertEqual(self.fix["age_stale"]["reason"], "quote_age_exceeded")
        self.assertTrue(self.fix["age_stale"]["observed_while_stale"])

    def test_encode_decode_level_qty(self):
        pkt = encode_full_mode_packet(
            instrument_token=1,
            last_price=10.0,
            volume=5,
            exchange_timestamp_sec=100,
            depth_buy=[(9.9, 3)] * 5,
            depth_sell=[(10.1, 4)] * 5,
        )
        tick = decode_full_mode_packet(pkt)
        self.assertEqual(tick["bid_qty"], 3.0)
        self.assertEqual(tick["ask_qty"], 4.0)


if __name__ == "__main__":
    unittest.main()
