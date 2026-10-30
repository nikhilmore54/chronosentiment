"""Post-close next-session watchlist unit tests."""

from __future__ import annotations

import unittest
from datetime import date
from pathlib import Path
import sys

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_nse_next_session_recs import (
    build_next_session_recommendations,
    next_weekday_session,
)


class TestNextSessionRecs(unittest.TestCase):
    def test_next_weekday_skips_weekend(self):
        # Friday → Monday
        self.assertEqual(next_weekday_session(date(2026, 10, 30)), date(2026, 11, 2))
        # Thursday → Friday
        self.assertEqual(next_weekday_session(date(2026, 10, 29)), date(2026, 10, 30))

    def test_ranks_strong_eod_momentum(self):
        hist = {
            "WEAK.NS": [{"close": 100.0 + i * 0.01} for i in range(10)],
            "STRONG.NS": [{"close": 100.0 * (1.01 ** i)} for i in range(12)],
        }
        payload = build_next_session_recommendations(
            hist,
            from_session_date="2026-10-30",
            next_session_date="2026-11-02",
            top_n=5,
        )
        self.assertEqual(payload["protocol_id"], "RP-608-NSE-NEXT-SESSION-001")
        self.assertEqual(payload["live_order_authority"], 0.0)
        self.assertGreaterEqual(payload["n_recommendations"], 1)
        top = payload["recommendations"][0]
        self.assertEqual(top["symbol"], "STRONG.NS")
        self.assertEqual(top["decision"], "WATCH_NEXT_SESSION")
        self.assertEqual(top["actionable_from_session_date"], "2026-11-02")
        self.assertEqual(top["actionable_from_bar"], 5)

    def test_deterministic(self):
        hist = {
            "AAA.NS": [{"close": 100 + i} for i in range(8)],
            "BBB.NS": [{"close": 50 + i * 0.5} for i in range(8)],
        }
        a = build_next_session_recommendations(
            hist, from_session_date="2026-10-30", next_session_date="2026-11-02"
        )
        b = build_next_session_recommendations(
            hist, from_session_date="2026-10-30", next_session_date="2026-11-02"
        )
        self.assertEqual(a["recommendations"], b["recommendations"])


if __name__ == "__main__":
    unittest.main()
