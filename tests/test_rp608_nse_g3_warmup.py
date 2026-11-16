"""Unit tests for RP-608 G3 cross-session gap-neutral warmup."""

from __future__ import annotations

import unittest
from pathlib import Path
import sys

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_nse_g3_warmup import (
    POLICY_CROSS_GAP_NEUTRAL_V1,
    POLICY_SAME_SESSION,
    evaluate_g3_momentum,
    traj15_from_closes,
    traj15_gap_neutral,
)


def _bar(ts_ms: int, o: float, h: float, l: float, c: float, session_date: str):
    return {
        "timestamp_utc_ms": ts_ms,
        "open": o,
        "high": h,
        "low": l,
        "close": c,
        "volume": 1000,
        "session_date": session_date,
    }


class TestG3Warmup(unittest.TestCase):
    def test_same_session_cold_with_4_today(self):
        today = [
            _bar(1_000_000 + i * 900_000, 100 + i, 101 + i, 99 + i, 100 + i, "2026-11-03")
            for i in range(4)
        ]
        out = evaluate_g3_momentum(
            policy=POLICY_SAME_SESSION,
            session_bars=today,
            feature_bars=today,
            today_session_date="2026-11-03",
        )
        self.assertFalse(out["g3_ready"])
        self.assertEqual(out["g3_history_length"], 4)

    def test_gap_neutral_ready_with_prior_plus_today(self):
        # Timestamps must be chronological across sessions (sort key in builder).
        prior = [
            _bar(1_700_000_000_000 + i * 900_000, 90 + i * 0.1, 91, 89, 90 + i * 0.1, "2026-11-02")
            for i in range(20)
        ]
        today = [
            _bar(1_700_000_000_000 + 100 * 900_000, 110.0, 111.0, 109.0, 110.5, "2026-11-03"),
            _bar(1_700_000_000_000 + 101 * 900_000, 110.5, 111.0, 110.0, 110.8, "2026-11-03"),
        ]
        out = evaluate_g3_momentum(
            policy=POLICY_CROSS_GAP_NEUTRAL_V1,
            session_bars=today,
            feature_bars=prior + today,
            today_session_date="2026-11-03",
        )
        self.assertTrue(out["g3_ready"])
        self.assertGreaterEqual(out["g3_history_length"], 5)
        self.assertEqual(out["g3_session_history_length"], 2)
        self.assertIsNotNone(out["overnight_gap_bps"])
        self.assertGreater(out["overnight_gap_bps"], 500)

    def test_gap_neutral_zeros_overnight_vs_naive(self):
        # Flat prior closes @100; +10% gap into a flat first today bar (open=close).
        prior = [
            _bar(1_700_000_000_000 + i * 900_000, 100, 101, 99, 100.0, "2026-11-02")
            for i in range(10)
        ]
        today = [
            _bar(1_700_000_000_000 + 50 * 900_000, 110.0, 110.5, 109.5, 110.0, "2026-11-03"),
            _bar(1_700_000_000_000 + 51 * 900_000, 110.0, 110.3, 109.8, 110.1, "2026-11-03"),
        ]
        hist = prior + today
        naive = traj15_from_closes([b["close"] for b in hist])
        gap_n, gap_bps = traj15_gap_neutral(hist)
        self.assertIsNotNone(gap_bps)
        self.assertGreater(gap_bps, 500)
        # Gap-neutral must differ from naive when a large overnight gap is present.
        self.assertNotAlmostEqual(gap_n, naive, places=3)
        # And must not equal the raw overnight close-to-close move as traj input:
        # first today bar open==close ⇒ overnight contribution to adj path is ~0.
        self.assertLess(abs(gap_n), abs(naive) + 1e-9)


if __name__ == "__main__":
    unittest.main()
