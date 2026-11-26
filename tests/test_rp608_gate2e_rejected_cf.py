"""Gate2E rejected-trade CF stamps (authority 0%)."""
from __future__ import annotations

import os
import unittest


class TestRejectedCf(unittest.TestCase):
    def setUp(self) -> None:
        os.environ["RP608_GATE2E_REJECTED_CF"] = "1"

    def test_primary_stamp_pattern(self):
        from scripts.rp608_gate2e_rejected_cf import (
            build_rejected_cf_stamp,
            rejected_cf_applies_to_book,
        )

        self.assertTrue(rejected_cf_applies_to_book("CRYPTO_PLAN"))
        s = build_rejected_cf_stamp(
            reject_reason="PATTERN",
            book_id="CRYPTO_PLAN",
            suppressed_ms=1,
            direction="SHORT",
            plan={"plan_id": "p1", "entry_price": 100.0, "symbol": "BTCUSDT"},
            pattern_stamp={"prediction": "STANDASIDE_ADVERSE_PATTERN"},
        )
        self.assertEqual(s["reject_reason"], "PATTERN")
        self.assertEqual(s["twin_book"], "CRYPTO_PLAN_BASELINE_PARALLEL")
        self.assertFalse(s["opens_trade"])
        self.assertEqual(s["entry_price"], 100.0)

    def test_di_skew_stamp(self):
        from scripts.rp608_gate2e_rejected_cf import build_rejected_cf_stamp

        s = build_rejected_cf_stamp(
            reject_reason="DI_SKEW",
            book_id="CRYPTO_PLAN_ETHUSDT",
            suppressed_ms=2,
            direction="SHORT",
            plan={"entry_price": 3000.0},
            regime_align={
                "guard": "DI_SKEW",
                "block_reason": "ADVERSE_TO_DI_SKEW_BULLISH",
                "di_skew": 18.0,
                "regime_state": "SIDEWAYS",
            },
        )
        self.assertEqual(s["twin_book"], "CRYPTO_PLAN_BASELINE_PARALLEL_ETHUSDT")
        self.assertEqual(s["guard"], "DI_SKEW")
        self.assertEqual(s["di_skew"], 18.0)

    def test_baseline_book_not_stamped(self):
        from scripts.rp608_gate2e_rejected_cf import rejected_cf_applies_to_book

        self.assertFalse(
            rejected_cf_applies_to_book("CRYPTO_PLAN_BASELINE_PARALLEL")
        )
        self.assertFalse(
            rejected_cf_applies_to_book("CRYPTO_PLAN_GATE2X_R50_PARALLEL")
        )


if __name__ == "__main__":
    unittest.main()
