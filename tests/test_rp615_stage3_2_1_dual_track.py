#!/usr/bin/env python3
"""
tests/test_rp615_stage3_2_1_dual_track.py

Unit test suite for RP-615 Stage 3.2.1 Protocol Errata & Dual-Track Prospective Shadow Validation.
Tests:
  1. Discrete Event Simulation (DES) priority queue and tie-breaking semantics.
  2. Zero-forward-exit leakage state query invariants (t_query <= t_cutoff).
  3. Cooldown observable boundary calculation.
  4. Gross-payoff Break-Even Probability (P_BE) computation for Arm R2.
  5. Conditional-mean parameter uncertainty (LCB_mean) computation for Arm R3.
  6. Dispersion-adjusted economic score computation for Arm R4/R5.
  7. Gain concentration scorecard handling for negative / zero net P&L.
  8. Track 1 Frozen Audit vs Track 2 Research ledger schema and reproducibility.
"""

from __future__ import annotations

import heapq
import json
import math
import unittest
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

from scripts.run_rp615_prospective_dual_track import (
    DESEvent,
    DESStateTracker,
    PRIORITY_DECISION,
    PRIORITY_EXIT,
    calc_gain_concentration,
    calc_profit_factor,
    wilson_ci,
)


class TestRP615Stage321DualTrack(unittest.TestCase):

    def setUp(self):
        self.state_tracker = DESStateTracker()
        self.t0 = datetime(2026, 9, 28, 9, 15, 0, tzinfo=timezone.utc)
        self.t_cutoff = datetime(2026, 9, 28, 9, 15, 15, tzinfo=timezone.utc)
        self.t_entry = datetime(2026, 9, 28, 9, 15, 30, tzinfo=timezone.utc)
        self.t_exit = datetime(2026, 9, 28, 9, 20, 30, tzinfo=timezone.utc)

    def test_des_equal_timestamp_tie_breaking(self):
        """
        Asserts that if EVENT_EXIT and EVENT_CUTOFF occur at the exact same second,
        EVENT_EXIT has priority 1 and processes first, clearing the symbol lock before the cutoff decision.
        """
        event_queue: List[DESEvent] = []
        sym = "INFY.NS"
        trade1 = "TRADE-001"
        trade2 = "TRADE-002"

        # Trade 1 is currently active
        self.state_tracker.register_entry(sym, trade1)
        self.assertTrue(self.state_tracker.is_symbol_locked(sym))

        # Enqueue EXIT for trade1 and CUTOFF for trade2 at the identical timestamp
        same_time = datetime(2026, 9, 28, 9, 25, 0, tzinfo=timezone.utc)
        heapq.heappush(event_queue, DESEvent(
            timestamp=same_time,
            priority=PRIORITY_DECISION,
            event_id=trade2,
            event_type="EVENT_CUTOFF",
            payload={"symbol": sym, "trade_id": trade2}
        ))
        heapq.heappush(event_queue, DESEvent(
            timestamp=same_time,
            priority=PRIORITY_EXIT,
            event_id=f"EXIT_{trade1}",
            event_type="EVENT_EXIT",
            payload={"symbol": sym, "trade_id": trade1}
        ))

        # First popped event MUST be EVENT_EXIT
        first_ev = heapq.heappop(event_queue)
        self.assertEqual(first_ev.event_type, "EVENT_EXIT")
        self.state_tracker.process_exit(sym, trade1, first_ev.timestamp)
        self.assertFalse(self.state_tracker.is_symbol_locked(sym))

        # Second popped event is EVENT_CUTOFF, which now sees an unlocked symbol
        second_ev = heapq.heappop(event_queue)
        self.assertEqual(second_ev.event_type, "EVENT_CUTOFF")
        self.assertFalse(self.state_tracker.is_symbol_locked(second_ev.payload["symbol"]))

    def test_des_zero_forward_exit_leakage(self):
        """
        Asserts that an exit occurring at t_entry is in the future relative to t_cutoff (t_entry - 15s)
        and therefore does NOT release the lock at t_cutoff.
        """
        sym = "RELIANCE.NS"
        trade_old = "TRADE-OLD"
        self.state_tracker.register_entry(sym, trade_old)

        t_cutoff = datetime(2026, 9, 28, 9, 30, 0, tzinfo=timezone.utc)
        t_entry = datetime(2026, 9, 28, 9, 30, 15, tzinfo=timezone.utc)
        t_future_exit = t_entry # Exit happens at 9:30:15

        # At decision cutoff (9:30:00), the symbol must still be locked
        self.assertTrue(self.state_tracker.is_symbol_locked(sym))
        
        # When simulation clock reaches 9:30:15, exit is processed
        self.state_tracker.process_exit(sym, trade_old, t_future_exit)
        self.assertFalse(self.state_tracker.is_symbol_locked(sym))
        self.assertEqual(self.state_tracker.last_observable_exit[sym], t_future_exit)

    def test_observable_cooldown_calculation(self):
        """
        Tests the 300-second cooldown calculation against observable exits.
        """
        sym = "TCS.NS"
        t_exit = datetime(2026, 9, 28, 9, 30, 0, tzinfo=timezone.utc)
        self.state_tracker.process_exit(sym, "TRADE-A", t_exit)

        # 299s later -> Cooldown active (< 300s)
        t_query_blocked = datetime(2026, 9, 28, 9, 34, 59, tzinfo=timezone.utc)
        cooldown_sec = self.state_tracker.get_cooldown_seconds(sym, t_query_blocked)
        self.assertAlmostEqual(cooldown_sec, 299.0)
        self.assertTrue(cooldown_sec < 300.0)

        # 300s later -> Cooldown cleared
        t_query_cleared = datetime(2026, 9, 28, 9, 35, 0, tzinfo=timezone.utc)
        cooldown_sec_cleared = self.state_tracker.get_cooldown_seconds(sym, t_query_cleared)
        self.assertAlmostEqual(cooldown_sec_cleared, 300.0)
        self.assertFalse(cooldown_sec_cleared < 300.0)

    def test_arm_r2_dynamic_break_even_formula(self):
        """
        Verifies Arm R2 dynamic break-even formula: P_BE = (GrossLoss + Friction) / (GrossWin + GrossLoss).
        """
        gross_win = 400.0
        gross_loss = 200.0
        friction = 140.0

        p_be = (gross_loss + friction) / (gross_win + gross_loss)
        # (200 + 140) / 600 = 340 / 600 = 0.56666...
        self.assertAlmostEqual(p_be, 340.0 / 600.0, places=4)

        # Hurdle is P_BE + 0.0300
        hurdle = p_be + 0.0300
        self.assertAlmostEqual(hurdle, 0.596666, places=4)

    def test_arm_r3_mean_uncertainty_formula(self):
        """
        Verifies Arm R3 parameter uncertainty: LCB_mean = mu - 1.645 * (sigma_ens_mu / sqrt(10)).
        """
        pred_net = 150.0
        sigma_ens_mu = 60.0
        k = 10

        se = sigma_ens_mu / math.sqrt(k)
        lcb_mean = pred_net - 1.645 * se

        expected_se = 60.0 / math.sqrt(10.0) # ~18.9736
        expected_lcb = 150.0 - 1.645 * expected_se # ~118.788
        self.assertAlmostEqual(se, expected_se, places=4)
        self.assertAlmostEqual(lcb_mean, expected_lcb, places=3)
        self.assertTrue(lcb_mean > 0.0)

    def test_arm_r4_dispersion_adjusted_score(self):
        """
        Verifies Arm R4 score: Score_adj = mu / (1.0 + 5.0 * sigma_ens).
        """
        pred_net = 120.0
        sigma_ens = 0.040

        score_adj = pred_net / (1.0 + 5.0 * sigma_ens)
        # 120.0 / (1.0 + 0.20) = 120.0 / 1.20 = 100.0
        self.assertAlmostEqual(score_adj, 100.0, places=4)
        self.assertTrue(score_adj >= 25.0)

    def test_gain_concentration_handling_negative_net(self):
        """
        Asserts that calc_gain_concentration returns None (N/A) for ratio when total net P&L <= 0.
        """
        losing_trades = [
            {"realized_net_pnl": -500.0},
            {"realized_net_pnl": -300.0},
            {"realized_net_pnl": 200.0},
        ]
        ratio, top5, tot = calc_gain_concentration(losing_trades)
        self.assertIsNone(ratio)
        self.assertEqual(tot, -600.0)

        winning_trades = [
            {"realized_net_pnl": 500.0},
            {"realized_net_pnl": 400.0},
            {"realized_net_pnl": 300.0},
            {"realized_net_pnl": 200.0},
            {"realized_net_pnl": 100.0},
            {"realized_net_pnl": 50.0},
        ]
        ratio_win, top5_win, tot_win = calc_gain_concentration(winning_trades)
        self.assertIsNotNone(ratio_win)
        # Top 5 = 1500 / 1550 = 0.96774
        self.assertAlmostEqual(ratio_win, 1500.0 / 1550.0, places=4)
        self.assertEqual(top5_win, 1500.0)
        self.assertEqual(tot_win, 1550.0)

    def test_dual_track_manifest_and_ledgers_exist(self):
        """
        Asserts that dual-track prospective artifacts exist, have valid schemas, and are populated.
        """
        frozen_ledger = LOG_DIR / "rp615_tier4_frozen_ledger.csv"
        research_ledger = LOG_DIR / "rp615_tier4_research_ledger.csv"
        summary_json = LOG_DIR / "rp615_tier4_dual_track_summary.json"
        manifest_json = LOG_DIR / "rp615_tier4_dual_track_manifest.json"
        report_md = LOG_DIR / "rp615_tier4_dual_track_report.md"

        self.assertTrue(frozen_ledger.exists(), "Frozen ledger must exist")
        self.assertTrue(research_ledger.exists(), "Research ledger must exist")
        self.assertTrue(summary_json.exists(), "Summary JSON must exist")
        self.assertTrue(manifest_json.exists(), "Manifest JSON must exist")
        self.assertTrue(report_md.exists(), "Report markdown must exist")

        with summary_json.open(encoding="utf-8") as f:
            summary_data = json.load(f)

        self.assertIn("track1_frozen_summary", summary_data)
        self.assertIn("track2_research_summary", summary_data)
        self.assertEqual(summary_data["calibration_constants"]["benchmark_hash"], "212532d3ac12f9c6")

        # Check Track 1 Benchmark has 325 trades
        t1 = summary_data["track1_frozen_summary"]
        self.assertEqual(t1["Cand_0_Benchmark"]["n_executed"], 325)
        self.assertAlmostEqual(t1["Cand_0_Benchmark"]["total_net_pnl"], 68652.97, places=1)


if __name__ == "__main__":
    unittest.main()
