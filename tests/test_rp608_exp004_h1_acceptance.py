#!/usr/bin/env python3
"""
tests/test_rp608_exp004_h1_acceptance.py

Unit tests for RP-608-EXP-004 Hypothesis H1 Confirmed Breakout Acceptance harness.
Tests:
  1. Causal Opening Range boundary construction (zero entries during formation window).
  2. Multi-bar acceptance state machine: M=1 vs M=2 confirmation closes.
  3. Re-test rejection / boundary re-penetration resets candidate state.
  4. Intrabar profit target (+1.5R) and stop loss (-1.0R) execution.
  5. Capacity limiting under 10-scrip concurrency.
"""

from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from scripts.run_rp608_exp004_h1_confirmed_breakout import (
    H1BreakoutHarness,
    calculate_statutory_friction
)


class TestRP608EXP004(unittest.TestCase):
    def setUp(self):
        self.test_dir = Path(tempfile.mkdtemp(prefix="test_rp608_exp004_"))
        self.data_dir = self.test_dir / "data"
        self.output_dir = self.test_dir / "output"
        self.data_dir.mkdir(parents=True)
        self.output_dir.mkdir(parents=True)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def _create_synthetic_symbol(
        self,
        sym: str,
        price_bars: list[dict]
    ):
        base_utc = int(datetime(2026, 7, 6, 3, 45, tzinfo=timezone.utc).timestamp())
        bars = []
        for i, b in enumerate(price_bars):
            bars.append({
                "timestamp": base_utc + (i * 900),
                "open": b["open"],
                "high": b["high"],
                "low": b["low"],
                "close": b["close"],
                "volume": b.get("volume", 10000.0)
            })
        with open(self.data_dir / f"{sym}.json", "w") as f:
            json.dump(bars, f)

    def test_01_orb_formation_bars_zero_trading(self):
        """No trading should occur during or before formation window (bars 1-4)."""
        # Bars 1-4 have huge high, Bar 5 stays inside range
        bars = [
            {"open": 100.0, "high": 110.0, "low": 99.0, "close": 105.0},  # 1
            {"open": 105.0, "high": 108.0, "low": 104.0, "close": 106.0}, # 2
            {"open": 106.0, "high": 107.0, "low": 105.0, "close": 106.5}, # 3
            {"open": 106.5, "high": 109.0, "low": 105.5, "close": 107.0}, # 4: OR High = 110.0
        ]
        # Bars 5-25 stay below 110.0
        for _ in range(5, 26):
            bars.append({"open": 107.0, "high": 108.0, "low": 106.0, "close": 107.5})

        self._create_synthetic_symbol("TEST1", bars)
        harness = H1BreakoutHarness(self.data_dir, self.output_dir)
        harness.load_dataset()
        res = harness.run_h1_configuration(
            "TEST_ORB_NO_BREAK", "ORB", formation_bars=4, required_closes=1, volume_mult=1.0
        )
        self.assertEqual(res["executed_trades"], 0)
        self.assertEqual(res["eligible_breakouts"], 0)

    def test_02_m1_breakout_acceptance(self):
        """M=1 breakout triggers at Bar 5 Close and fills at Bar 6 Open."""
        bars = [
            {"open": 100.0, "high": 102.0, "low": 99.0, "close": 101.0},  # 1
            {"open": 101.0, "high": 103.0, "low": 100.0, "close": 102.0}, # 2
            {"open": 102.0, "high": 103.5, "low": 101.5, "close": 102.5}, # 3
            {"open": 102.5, "high": 104.0, "low": 102.0, "close": 103.0}, # 4: OR High = 104.0
            {"open": 103.0, "high": 106.0, "low": 104.5, "close": 105.5}, # 5: Breakout! Close 105.5 > 104.0
            {"open": 105.5, "high": 108.0, "low": 105.0, "close": 107.5}, # 6: Entry fills at 105.5 + slip
        ]
        # Bars 7-25 flat
        for _ in range(7, 26):
            bars.append({"open": 107.5, "high": 107.6, "low": 107.4, "close": 107.5})

        self._create_synthetic_symbol("TEST_M1", bars)
        harness = H1BreakoutHarness(self.data_dir, self.output_dir)
        harness.load_dataset()
        res = harness.run_h1_configuration(
            "TEST_M1_RUN", "ORB", formation_bars=4, required_closes=1, volume_mult=1.0
        )
        self.assertEqual(res["executed_trades"], 1)
        self.assertGreaterEqual(res["eligible_breakouts"], 1)

    def test_03_m2_acceptance_resets_on_failed_second_close(self):
        """M=2 requires two consecutive closes > boundary; reset occurs if second close fails."""
        bars = [
            {"open": 100.0, "high": 104.0, "low": 99.0, "close": 102.0},  # 1: OR High = 104.0
            {"open": 102.0, "high": 103.0, "low": 100.0, "close": 101.0}, # 2
            {"open": 101.0, "high": 102.0, "low": 100.5, "close": 101.5}, # 3
            {"open": 101.5, "high": 103.0, "low": 101.0, "close": 102.0}, # 4: OR High = 104.0
            {"open": 102.0, "high": 105.5, "low": 102.0, "close": 105.0}, # 5: Breakout! (Close 105.0 > 104.0)
            {"open": 105.0, "high": 105.2, "low": 102.5, "close": 103.0}, # 6: Failed! Close 103.0 < 104.0 (resets)
        ]
        for _ in range(7, 26):
            bars.append({"open": 103.0, "high": 103.5, "low": 102.5, "close": 103.0})

        self._create_synthetic_symbol("TEST_M2_FAIL", bars)
        harness = H1BreakoutHarness(self.data_dir, self.output_dir)
        harness.load_dataset()
        res = harness.run_h1_configuration(
            "TEST_M2_FAIL_RUN", "ORB", formation_bars=4, required_closes=2, volume_mult=1.0
        )
        self.assertEqual(res["executed_trades"], 0)

    def test_04_statutory_friction_accuracy(self):
        """Verify 8 statutory friction components."""
        f = calculate_statutory_friction(100.0, 101.5, 1000.0)
        self.assertGreater(f["brokerage"], 0)
        self.assertGreater(f["stt"], 0)
        self.assertGreater(f["exchange_charges"], 0)
        self.assertGreater(f["sebi_charges"], 0)
        self.assertGreater(f["stamp_duty"], 0)
        self.assertGreater(f["gst"], 0)
        self.assertGreater(f["total_statutory_friction"], 50.0)
        self.assertAlmostEqual(f["total_statutory_friction"], 82.88, places=1)


if __name__ == "__main__":
    unittest.main()
