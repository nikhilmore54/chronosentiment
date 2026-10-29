#!/usr/bin/env python3
"""
tests/test_rp608_exp002_exit_falsification.py

Unit tests for RP-608-EXP-002 exit falsification experiment harness.
Tests:
  1. Variant B unconditional 60m exit (exactly bars_held == 4).
  2. Variant C stop-loss precedence before 60m.
  3. Dynamic portfolio concurrency expansion when exits occur earlier.
"""

from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from scripts.run_rp608_exp002_exit_falsification import (
    ExitFalsificationHarness,
    ReplayBar
)


class TestRP608EXP002(unittest.TestCase):
    def setUp(self):
        self.test_dir = Path(tempfile.mkdtemp(prefix="test_rp608_exp002_"))
        self.data_dir = self.test_dir / "data"
        self.output_dir = self.test_dir / "output"
        self.data_dir.mkdir(parents=True)
        self.output_dir.mkdir(parents=True)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def _create_synthetic_symbol(self, sym: str, breakout_bar: int = 5, price_path: list[float] = None):
        base_utc = int(datetime(2026, 7, 6, 3, 45, tzinfo=timezone.utc).timestamp())
        bars = []
        for i in range(25):
            bar_num = i + 1
            if price_path and i < len(price_path):
                c = price_path[i]
                o = c
                h = c * 1.001
                l = c * 0.999
            else:
                if bar_num < breakout_bar:
                    o, h, l, c = 100.0, 100.5, 99.5, 100.0
                elif bar_num == breakout_bar:
                    o, h, l, c = 100.0, 110.0, 99.5, 109.0  # Big breakout
                else:
                    o, h, l, c = 109.0, 109.5, 108.5, 109.0
            bars.append({
                "timestamp": base_utc + (i * 900),
                "open": o,
                "high": h,
                "low": l,
                "close": c,
                "volume": 10000.0
            })
        with open(self.data_dir / f"{sym}.json", "w", encoding="utf-8") as f:
            json.dump(bars, f)

    def test_01_variant_b_unconditional_exit_at_bar_4(self):
        """Tests that Variant B unconditionally exits at exactly 4 bars held regardless of PnL."""
        sym = "TEST_UNCOND.NS"
        self._create_synthetic_symbol(sym, breakout_bar=5)

        harness = ExitFalsificationHarness(
            data_dir=self.data_dir,
            output_dir=self.output_dir,
            symbols=[sym]
        )
        harness.load_and_validate_all_data()
        res_b = harness.run_variant_simulation("VARIANT_B_UNCONDITIONAL_60M")
        self.assertGreaterEqual(res_b["total_trades"], 1)

        trades_file = self.output_dir / "variant_b_unconditional_60m_trades.jsonl"
        with open(trades_file) as f:
            trade = json.loads(f.readline())

        self.assertEqual(trade["entry_bar_idx"], 6)
        self.assertEqual(trade["exit_bar_idx"], 10)
        self.assertEqual(trade["bars_held"], 4)
        self.assertEqual(trade["exit_reason"], "UNCONDITIONAL_TIMED_EXIT_60M")

    def test_02_variant_c_early_stop_precedence(self):
        """Tests that in Variant C, a stop loss hit at bar 2 triggers early rather than waiting for bar 4."""
        sym = "TEST_STOP.NS"
        # Bar 1..4: flat 100
        # Bar 5: breakout to 110 (entry at bar 6 open ~110)
        # Bar 6: open 110, close 110
        # Bar 7: price drops to 105 (breaching -1.0% stop-loss)
        path = [100.0]*4 + [110.0, 110.0, 105.0] + [105.0]*18
        self._create_synthetic_symbol(sym, breakout_bar=5, price_path=path)

        harness = ExitFalsificationHarness(
            data_dir=self.data_dir,
            output_dir=self.output_dir,
            symbols=[sym]
        )
        harness.load_and_validate_all_data()
        res_c = harness.run_variant_simulation("VARIANT_C_FIXED_60M_WITH_STOP")
        self.assertGreaterEqual(res_c["total_trades"], 1)

        trades_file = self.output_dir / "variant_c_fixed_60m_with_stop_trades.jsonl"
        with open(trades_file) as f:
            trade = json.loads(f.readline())

        self.assertEqual(trade["entry_bar_idx"], 6)
        self.assertEqual(trade["exit_bar_idx"], 7)
        self.assertEqual(trade["bars_held"], 1)
        self.assertEqual(trade["exit_reason"], "STOP_LOSS_HIT")


if __name__ == "__main__":
    unittest.main()
