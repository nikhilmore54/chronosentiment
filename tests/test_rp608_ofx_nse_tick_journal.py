#!/usr/bin/env python3
"""Unit tests for OFX Stage 0D inferred-aggressor tick journal (no network)."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_ofx_nse_tick_journal import (
    OfxNseInferredAggressorTickJournal,
    classify_trade_like,
    infer_aggressor_vs_quote,
)


class TestOfxTickJournal(unittest.TestCase):
    def test_infer_buy_sell_ambiguous(self):
        buy = infer_aggressor_vs_quote(ltp=100.05, bid=100.0, ask=100.05, quote_age_ms=10, quote_source="same_packet")
        self.assertEqual(buy["label"], "BUY_INITIATED_CANDIDATE")
        sell = infer_aggressor_vs_quote(ltp=99.95, bid=99.95, ask=100.0, quote_age_ms=10, quote_source="prior_packet_quote")
        self.assertEqual(sell["label"], "SELL_INITIATED_CANDIDATE")
        amb = infer_aggressor_vs_quote(ltp=100.02, bid=100.0, ask=100.05, quote_age_ms=10, quote_source="same_packet")
        self.assertEqual(amb["label"], "AMBIGUOUS")

    def test_trade_like_vs_quote_repeat(self):
        prev = {"last_price": 1.0, "last_traded_quantity": 2.0, "last_trade_time_sec": 10, "volume": 100.0}
        cls, tl = classify_trade_like(prev=prev, ltp=1.0, ltq=2.0, ltt_sec=10, volume=100.0)
        self.assertFalse(tl)
        self.assertIn("QUOTE_REPEAT", cls)
        cls2, tl2 = classify_trade_like(prev=prev, ltp=1.1, ltq=3.0, ltt_sec=11, volume=105.0)
        self.assertTrue(tl2)

    def test_journal_writes_rows(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            j = OfxNseInferredAggressorTickJournal(
                root, enabled=True, token_to_symbol={6401: "ADANIENT.NS"}, flush_every=1
            )
            tick = {
                "instrument_token": 6401,
                "last_price": 2505.0,
                "last_traded_quantity": 7.0,
                "average_price": 2504.5,
                "volume": 1000.0,
                "last_trade_time_sec": 100,
                "exchange_timestamp_sec": 100,
                "best_bid": 2504.0,
                "best_ask": 2505.0,
                "bid_qty": 10,
                "ask_qty": 12,
                "depth_buy": [{"price": 2504.0 - i, "quantity": 10} for i in range(5)],
                "depth_sell": [{"price": 2505.0 + i, "quantity": 12} for i in range(5)],
                "receipt_utc_ms": 1_700_000_000_000,
                "quote_age_ms": 5.0,
                "quote_source": "kite_ws_full",
            }
            j.on_tick(tick)
            j.on_tick({**tick, "volume": 1010.0, "last_traded_quantity": 8.0, "last_trade_time_sec": 101})
            j.close()
            lines = (root / "inferred_aggressor_ticks.jsonl").read_text().strip().splitlines()
            self.assertEqual(len(lines), 2)
            r0 = json.loads(lines[0])
            self.assertEqual(r0["symbol"], "ADANIENT.NS")
            self.assertIn("aggressor_inference_primary", r0)
            cov = json.loads((root / "capture_coverage.json").read_text())
            self.assertEqual(cov["ticks_written"], 2)


if __name__ == "__main__":
    unittest.main()
