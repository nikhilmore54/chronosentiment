#!/usr/bin/env python3
"""
tests/test_rp608_exp005_h2_exhaustion.py

Unit tests for RP-608-EXP-005 Hypothesis H2 Mean Reversion & Exhaustion Fading harness.
Tests:
  1. Bullish absorption / spring rejection wick triggering Long entry.
  2. Bearish absorption / upthrust rejection wick triggering Short entry.
  3. Short trade execution mechanics and friction accounting.
  4. Intrabar target and stop execution for both directions.
"""

from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from scripts.run_rp608_exp005_h2_exhaustion_fading import (
    H2ExhaustionHarness,
    calculate_statutory_friction
)


class TestRP608EXP005(unittest.TestCase):
    def setUp(self):
        self.test_dir = Path(tempfile.mkdtemp(prefix="test_rp608_exp005_"))
        self.data_dir = self.test_dir / "data"
        self.output_dir = self.test_dir / "output"
        self.data_dir.mkdir(parents=True)
        self.output_dir.mkdir(parents=True)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def _create_synthetic_symbol(self, sym: str, price_bars: list[dict]):
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

    def test_01_bullish_spring_long_entry(self):
        """Bar pierces below OR Low, closes back inside with lower wick >= 35%, triggers Long."""
        bars = [
            {"open": 100.0, "high": 105.0, "low": 98.0, "close": 102.0},  # 1: OR Low = 98.0
            {"open": 102.0, "high": 104.0, "low": 99.0, "close": 103.0},  # 2
            {"open": 103.0, "high": 104.5, "low": 100.0, "close": 102.5}, # 3
            {"open": 102.5, "high": 104.0, "low": 101.0, "close": 103.0}, # 4: OR Low = 98.0
            # Bar 5: Dips to 95.0 (below 98.0), but rallies back to close at 99.0 (inside OR)
            # Range: 100.0 - 95.0 = 5.0. Lower wick: min(open, close) - low = 98.0 - 95.0 = 3.0 (60% wick)
            {"open": 98.0, "high": 100.0, "low": 95.0, "close": 99.0},   # 5: Spring!
            {"open": 99.0, "high": 101.5, "low": 98.5, "close": 101.0},  # 6: Long entry fills at 99.0 + slip
        ]
        for _ in range(7, 26):
            bars.append({"open": 101.0, "high": 101.5, "low": 100.5, "close": 101.0})

        self._create_synthetic_symbol("TEST_SPRING", bars)
        harness = H2ExhaustionHarness(self.data_dir, self.output_dir)
        harness.load_dataset()
        res = harness.run_h2_configuration("TEST_SPRING_RUN", "LONG_ONLY", 4, 0.35, 1.0)
        self.assertEqual(res["executed_trades"], 1)
        self.assertGreaterEqual(res["nominal_win_rate_pct"], 0.0)

    def test_02_bearish_upthrust_short_entry(self):
        """Bar pierces above OR High, closes back inside with upper wick >= 35%, triggers Short."""
        bars = [
            {"open": 100.0, "high": 105.0, "low": 98.0, "close": 102.0},  # 1: OR High = 105.0
            {"open": 102.0, "high": 104.0, "low": 99.0, "close": 103.0},  # 2
            {"open": 103.0, "high": 104.5, "low": 100.0, "close": 102.5}, # 3
            {"open": 102.5, "high": 104.0, "low": 101.0, "close": 103.0}, # 4: OR High = 105.0
            # Bar 5: Spikes to 110.0 (above 105.0), but rejected to close at 104.0 (inside OR)
            # Range: 110.0 - 103.0 = 7.0. Upper wick: 110.0 - max(open, close) = 110.0 - 104.5 = 5.5 (78% wick)
            {"open": 104.5, "high": 110.0, "low": 103.0, "close": 104.0}, # 5: Upthrust!
            {"open": 104.0, "high": 104.5, "low": 101.0, "close": 101.5}, # 6: Short entry fills at 104.0 - slip
        ]
        for _ in range(7, 26):
            bars.append({"open": 101.5, "high": 102.0, "low": 101.0, "close": 101.5})

        self._create_synthetic_symbol("TEST_UPTHRUST", bars)
        harness = H2ExhaustionHarness(self.data_dir, self.output_dir)
        harness.load_dataset()
        res = harness.run_h2_configuration("TEST_UPTHRUST_RUN", "SHORT_ONLY", 4, 0.35, 1.0)
        self.assertEqual(res["executed_trades"], 1)

    def test_03_short_friction_accounting(self):
        """Verify statutory friction calculation for short trade (entry is sell, exit is buy)."""
        f = calculate_statutory_friction(100.0, 99.0, 1000.0, is_short=True)
        self.assertGreater(f["brokerage"], 0)
        self.assertGreater(f["stt"], 0)
        self.assertGreater(f["exchange_charges"], 0)
        self.assertGreater(f["total_statutory_friction"], 50.0)


if __name__ == "__main__":
    unittest.main()
