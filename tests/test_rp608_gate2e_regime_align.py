"""Gate2E regime-align + DI-skew decision tune (authority 0%)."""
from __future__ import annotations

import os
import unittest


class TestRegimeAlign(unittest.TestCase):
    def setUp(self) -> None:
        os.environ["RP608_GATE2E_REGIME_ALIGN"] = "1"
        os.environ["RP608_GATE2E_REGIME_ALIGN_BASELINE"] = "0"
        os.environ["RP608_GATE2E_DI_SKEW"] = "1"
        os.environ["RP608_GATE2E_DI_SKEW_MIN_ABS"] = "10"

    def test_blocks_short_in_bullish_directional_primary(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="SHORT",
            regime={
                "regime": "DIRECTIONAL",
                "trend_dir": "BULLISH",
                "plus_di": 30.0,
                "minus_di": 10.0,
            },
            book_id="CRYPTO_PLAN",
        )
        self.assertTrue(s["blocks_entry"])
        self.assertEqual(s["align"], "ADVERSE")
        self.assertEqual(s["guard"], "DIRECTIONAL")

    def test_allows_long_in_bullish_directional(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="LONG",
            regime={"regime": "DIRECTIONAL", "trend_dir": "BULLISH"},
            book_id="CRYPTO_PLAN",
        )
        self.assertFalse(s["blocks_entry"])
        self.assertEqual(s["align"], "ALIGNED")

    def test_sideways_weak_di_skew_no_block(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="SHORT",
            regime={"regime": "SIDEWAYS", "plus_di": 22.0, "minus_di": 20.0},
            book_id="CRYPTO_PLAN",
        )
        self.assertFalse(s["blocks_entry"])
        self.assertEqual(s["align"], "NO_DIRECTIONAL_REGIME")

    def test_sideways_strong_di_skew_blocks_adverse_short(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="SHORT",
            regime={"regime": "SIDEWAYS", "plus_di": 30.0, "minus_di": 10.0},
            book_id="CRYPTO_PLAN",
        )
        self.assertTrue(s["blocks_entry"])
        self.assertEqual(s["guard"], "DI_SKEW")
        self.assertEqual(s["block_reason"], "ADVERSE_TO_DI_SKEW_BULLISH")
        self.assertEqual(s["align"], "ADVERSE")

    def test_transitioning_di_skew_allows_aligned_long(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="LONG",
            regime={"regime": "TRANSITIONING", "plus_di": 28.0, "minus_di": 12.0},
            book_id="CRYPTO_PLAN",
        )
        self.assertFalse(s["blocks_entry"])
        self.assertEqual(s["guard"], "DI_SKEW")
        self.assertEqual(s["align"], "ALIGNED")

    def test_di_skew_disabled_no_block_sideways(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        os.environ["RP608_GATE2E_DI_SKEW"] = "0"
        s = evaluate_regime_align(
            direction="SHORT",
            regime={"regime": "SIDEWAYS", "plus_di": 30.0, "minus_di": 10.0},
            book_id="CRYPTO_PLAN",
        )
        self.assertFalse(s["blocks_entry"])

    def test_baseline_twin_ungated_by_default(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="SHORT",
            regime={"regime": "DIRECTIONAL", "trend_dir": "BULLISH"},
            book_id="CRYPTO_PLAN_BASELINE_PARALLEL",
        )
        self.assertFalse(s["blocks_entry"])
        self.assertEqual(s["align"], "ADVERSE")

    def test_baseline_twin_ungated_on_di_skew(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="SHORT",
            regime={"regime": "SIDEWAYS", "plus_di": 30.0, "minus_di": 10.0},
            book_id="CRYPTO_PLAN_BASELINE_PARALLEL",
        )
        self.assertFalse(s["blocks_entry"])
        self.assertEqual(s["guard"], "DI_SKEW")
        self.assertEqual(s["align"], "ADVERSE")

    def test_nse_long_blocked_in_bearish_directional(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="LONG",
            regime={"regime": "DIRECTIONAL", "trend_dir": "BEARISH"},
            book_id="NSE_ESTIMAND_A",
        )
        self.assertTrue(s["blocks_entry"])

    def test_nse_long_blocked_sideways_bearish_di_skew(self):
        from scripts.rp608_gate2e_regime_align import evaluate_regime_align

        s = evaluate_regime_align(
            direction="LONG",
            regime={"regime": "SIDEWAYS", "plus_di": 8.0, "minus_di": 25.0},
            book_id="NSE_ESTIMAND_A",
        )
        self.assertTrue(s["blocks_entry"])
        self.assertEqual(s["block_reason"], "ADVERSE_TO_DI_SKEW_BEARISH")


if __name__ == "__main__":
    unittest.main()
