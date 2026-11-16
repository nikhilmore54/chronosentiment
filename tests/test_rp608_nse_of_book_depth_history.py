#!/usr/bin/env python3
"""Phase 1 durable L1–L5 depth history — observational journal tests."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_nse_of_book_depth_history import (
    NseOfBookDepthHistoryJournal,
    snapshot_row_from_tick,
)


class TestOfBookDepthHistory(unittest.TestCase):
    def test_snapshot_retains_l1_l5(self):
        tick = {
            "instrument_token": 6401,
            "last_price": 100.0,
            "best_bid": 99.9,
            "best_ask": 100.1,
            "bid_qty": 10,
            "ask_qty": 12,
            "exchange_timestamp_sec": 1_700_000_000,
            "receipt_utc_ms": 1_700_000_000_500,
            "quote_age_ms": 500.0,
            "depth_buy": [{"price": 99.9 - i * 0.05, "quantity": 10 * (i + 1)} for i in range(5)],
            "depth_sell": [{"price": 100.1 + i * 0.05, "quantity": 12 * (i + 1)} for i in range(5)],
            "quote_source": "kite_ws_full",
        }
        row = snapshot_row_from_tick(symbol="TEST.NS", tick=tick, sample_utc_ms=1_700_000_001_000)
        self.assertEqual(len(row["depth_buy"]), 5)
        self.assertEqual(len(row["depth_sell"]), 5)
        self.assertEqual(row["bid_levels_qty_nonzero"], 5)
        self.assertFalse(row["stale"])
        self.assertEqual(row["protocol_id"], "RP-608-NSE-OF-BOOK-INTELLIGENCE-001")

    def test_journal_writes_and_rate_limits(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            j = NseOfBookDepthHistoryJournal(root, min_interval_ms=60_000, enabled=True)
            ticks = {
                "AAA.NS": {
                    "instrument_token": 1,
                    "last_price": 10.0,
                    "best_bid": 9.9,
                    "best_ask": 10.1,
                    "bid_qty": 1,
                    "ask_qty": 1,
                    "exchange_timestamp_sec": 1_700_000_000,
                    "receipt_utc_ms": 1_700_000_000_100,
                    "quote_age_ms": 100.0,
                    "depth_buy": [{"price": 9.9, "quantity": 1}] * 5,
                    "depth_sell": [{"price": 10.1, "quantity": 1}] * 5,
                }
            }
            n1 = j.maybe_sample(ticks_by_symbol=ticks, now_ms=1_700_000_000_000)
            n2 = j.maybe_sample(ticks_by_symbol=ticks, now_ms=1_700_000_001_000)
            self.assertEqual(n1, 1)
            self.assertEqual(n2, 0)
            lines = j.path.read_text(encoding="utf-8").strip().splitlines()
            self.assertEqual(len(lines), 1)
            row = json.loads(lines[0])
            self.assertEqual(len(row["depth_buy"]), 5)
            cov = json.loads(j.coverage_path.read_text(encoding="utf-8"))
            self.assertEqual(cov["snapshots_written"], 1)
            self.assertGreaterEqual(cov["samples_skipped_interval"], 1)


if __name__ == "__main__":
    unittest.main()
