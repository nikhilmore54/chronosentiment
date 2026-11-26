#!/usr/bin/env python3
from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.research.rp608_nse_of_book_intel_phase2_features import (
    annotate_features,
    detect_bid_support_while_falling,
    i5_imbalance,
    summarize,
)


class TestPhase2Features(unittest.TestCase):
    def test_i5(self):
        imb = i5_imbalance(
            [{"quantity": 80}] * 5,
            [{"quantity": 20}] * 5,
        )
        self.assertAlmostEqual(imb or 0.0, 0.6, places=6)

    def test_detect_support_while_falling(self):
        rows = []
        base = 100.0
        for i in range(5):
            rows.append(
                {
                    "symbol": "T.NS",
                    "sample_utc_ms": 1_000_000 + i * 5_000,
                    "sample_iso": f"t{i}",
                    "_i5": 0.4,
                    "_bid_sum": 1000.0,
                    "_ask_sum": 400.0,
                    "_mid": base - i * 0.05,
                    "last_price": base - i * 0.05,
                    "best_bid": base - i * 0.05 - 0.01,
                    "best_ask": base - i * 0.05 + 0.01,
                    "depth_buy": [{"price": 1, "quantity": 200}] * 5,
                    "depth_sell": [{"price": 1, "quantity": 80}] * 5,
                }
            )
        feats = annotate_features(rows)
        evs = detect_bid_support_while_falling(feats, persist=3, i5_min=0.2)
        self.assertGreaterEqual(len(evs), 1)
        self.assertLess(evs[0]["price_move_bps"], 0)

    def test_independence_counters_separate(self):
        events = [
            {
                "symbol": "A.NS",
                "event_utc_ms": 1_793_604_000_000,
                "candle_red": True,
                "forming_vs_open_bps": -10,
                "replenish_in_window": True,
                "withdraw_in_window": False,
                "fwd_1_available": True,
                "fwd_1_bar_net_bps": 5.0,
            },
            {
                "symbol": "A.NS",
                "event_utc_ms": 1_793_604_100_000,
                "candle_red": True,
                "forming_vs_open_bps": -5,
                "replenish_in_window": False,
                "withdraw_in_window": True,
                "fwd_1_available": True,
                "fwd_1_bar_net_bps": -3.0,
            },
            {
                "symbol": "B.NS",
                "event_utc_ms": 1_793_604_000_000,
                "candle_red": False,
                "forming_vs_open_bps": -1,
                "replenish_in_window": False,
                "withdraw_in_window": False,
                "fwd_1_available": False,
                "fwd_1_bar_net_bps": None,
            },
        ]
        s = summarize(events, {"A.NS": 24, "B.NS": 24})
        ind = s["independence"]
        self.assertEqual(ind["n_events"], 3)
        self.assertEqual(ind["n_unique_symbols"], 2)
        self.assertEqual(ind["n_independent_sessions"], 1)
        self.assertEqual(ind["n_symbol_session_pairs"], 2)
        self.assertFalse(s["verdict"]["actionable_claim_authorized"])


if __name__ == "__main__":
    unittest.main()
