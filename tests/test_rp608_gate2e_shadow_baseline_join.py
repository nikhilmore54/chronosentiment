"""Unit tests for Gate2E shadow → BASELINE join (observational)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.research.rp608_gate2e_shadow_baseline_join import join_shadow_to_baseline


class TestGate2eShadowBaselineJoin(unittest.TestCase):
    def test_join_same_direction_within_tol(self):
        shadows = [
            {
                "shadow_rule_id": "SHADOW_ALLOW_IF_N10_WR_GT_035",
                "plan_id": "PLAN-S1",
                "direction": "LONG",
                "suppressed_ms": 1_000_000,
                "hist_wr": 0.5,
                "cohort_n": 12,
                "pattern_id": "P1",
            }
        ]
        baselines = [
            {
                "plan_id": "PLAN-B1",
                "direction": "LONG",
                "entry_time_ms": 1_000_050,
                "exit_time_ms": 1_100_000,
                "net_pnl_usdt": 2.5,
                "exit_reason": "TRAILING_STOP_HIT",
            },
            {
                "plan_id": "PLAN-B2",
                "direction": "SHORT",
                "entry_time_ms": 1_000_010,
                "exit_time_ms": 1_100_000,
                "net_pnl_usdt": -9.0,
                "exit_reason": "STOP_LOSS_HIT",
            },
        ]
        report = join_shadow_to_baseline(shadows, baselines, tol_ms=120_000)
        self.assertEqual(report["n_joined"], 1)
        self.assertEqual(report["baseline_wl"]["wins"], 1)
        self.assertEqual(report["joined"][0]["baseline_plan_id"], "PLAN-B1")
        self.assertAlmostEqual(report["net_pnl_usdt_sum"], 2.5)

    def test_unmatched_when_outside_tol(self):
        shadows = [
            {
                "shadow_rule_id": "SHADOW_ALLOW_IF_N10_WR_GT_035",
                "plan_id": "PLAN-S1",
                "direction": "LONG",
                "suppressed_ms": 1_000_000,
                "hist_wr": 0.5,
                "cohort_n": 12,
            }
        ]
        baselines = [
            {
                "plan_id": "PLAN-B1",
                "direction": "LONG",
                "entry_time_ms": 1_000_000 + 500_000,
                "net_pnl_usdt": 1.0,
            }
        ]
        report = join_shadow_to_baseline(shadows, baselines, tol_ms=120_000)
        self.assertEqual(report["n_joined"], 0)
        self.assertEqual(report["n_unmatched"], 1)


if __name__ == "__main__":
    unittest.main()
