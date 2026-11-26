"""Unit tests for OFX crypto aggTrade durable journal (no network)."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.research.rp608_ofx_crypto_aggtrade_dq_report import dq_report, sample_aggregate
from scripts.rp608_ofx_crypto_aggtrade_journal import (
    OfxCryptoAggTradeJournal,
    normalize_aggtrade_event,
)


class TestNormalizeAggTrade(unittest.TestCase):
    def test_buy_aggressor_m_false(self):
        row = normalize_aggtrade_event(
            {"e": "aggTrade", "s": "BTCUSDT", "a": 1, "p": "100.0", "q": "0.5", "m": False, "E": 1000, "T": 1001}
        )
        self.assertIsNotNone(row)
        assert row is not None
        self.assertEqual(row["aggressor"], "BUY")
        self.assertEqual(row["side_classification"], "DIRECT")
        self.assertEqual(row["quantity"], 0.5)
        self.assertTrue(row["baseline_of_inert"])

    def test_sell_aggressor_m_true(self):
        row = normalize_aggtrade_event(
            {"s": "BTCUSDT", "a": 2, "p": "100.0", "q": "1.25", "m": True, "E": 2000, "T": 2001}
        )
        assert row is not None
        self.assertEqual(row["aggressor"], "SELL")
        self.assertEqual(row["is_buyer_maker"], True)

    def test_missing_m_unavailable(self):
        row = normalize_aggtrade_event({"s": "BTCUSDT", "a": 3, "p": "1", "q": "1", "E": 1, "T": 1})
        assert row is not None
        self.assertEqual(row["side_classification"], "UNAVAILABLE")


class TestJournalPersist(unittest.TestCase):
    def test_writes_and_dedupes(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            j = OfxCryptoAggTradeJournal(root, enabled=True, flush_every=1)
            raw = {"s": "BTCUSDT", "a": 10, "p": "1", "q": "2", "m": False, "E": 100, "T": 100}
            self.assertIsNotNone(j.on_raw_event(raw, receipt_utc_ms=1000))
            self.assertIsNone(j.on_raw_event(raw, receipt_utc_ms=1001))  # duplicate a
            j.on_raw_event({**raw, "a": 11, "m": True, "q": "3", "T": 200}, receipt_utc_ms=1002)
            j.close()
            lines = (root / "aggtrades.jsonl").read_text().strip().splitlines()
            self.assertEqual(len(lines), 2)
            cov = json.loads((root / "capture_coverage.json").read_text())
            self.assertEqual(cov["events_written"], 2)
            self.assertEqual(cov["duplicates_skipped"], 1)
            self.assertEqual(cov["buy_aggressor"], 1)
            self.assertEqual(cov["sell_aggressor"], 1)


class TestDqAndAggregatorFeed(unittest.TestCase):
    def test_dq_and_sample_flow(self):
        rows = [
            {
                "symbol": "BTCUSDT",
                "agg_trade_id": 1,
                "price": 100.0,
                "quantity": 2.0,
                "is_buyer_maker": False,
                "aggressor": "BUY",
                "trade_time_ms": 1000,
                "event_time_ms": 1000,
                "raw_ref": {"s": "BTCUSDT", "a": 1, "p": "100", "q": "2", "m": False, "T": 1000},
            },
            {
                "symbol": "BTCUSDT",
                "agg_trade_id": 2,
                "price": 100.0,
                "quantity": 1.0,
                "is_buyer_maker": True,
                "aggressor": "SELL",
                "trade_time_ms": 1100,
                "event_time_ms": 1100,
                "raw_ref": {"s": "BTCUSDT", "a": 2, "p": "100", "q": "1", "m": True, "T": 1100},
            },
        ]
        dq = dq_report(rows, gap_flag_ms=5000)
        self.assertEqual(dq["status"], "OK")
        self.assertEqual(dq["qty_side_mapping_ok"], 2)
        sample = sample_aggregate(rows, symbol="BTCUSDT")
        self.assertIsNotNone(sample)
        assert sample is not None
        self.assertEqual(sample["signed_delta_qty"], 1.0)  # 2 buy - 1 sell
        self.assertEqual(sample["source"], "binance_aggTrade_journal")


if __name__ == "__main__":
    unittest.main()
