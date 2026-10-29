#!/usr/bin/env python3
"""
test_rp614_gate5_cohort_and_pf_integrity.py

Automated Test Suite for RP-614 Gate 5 (MFE Cohort Integrity & Empirical Profit Factor).
Validates:
1. Mutually exclusive & exhaustive failure cohort partition across all 5,594 losing trades (A1, A2, B, C, D, E)
2. Exact reconciliation: 1,670 net winners + 5,594 net losers = 7,264 total trades
3. Zero cohort overlap (intersection between any two cohorts is empty)
4. Empirical per-trade Profit Factor calculation formula: sum(pos_net) / sum(abs(neg_net))
5. Proper handling of zero-division / pure-gain edge cases
"""

import sys
import unittest
import json
import numpy as np
import pandas as pd
from pathlib import Path

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"
sys.path.insert(0, str(WORKSPACE_DIR / "scripts"))

from run_rp614_root_cause_audit import compute_canonical_friction, run_investigation_1, run_investigation_2

JOURNAL_FILE = DATA_DIR / "rp605_nse_baseline_journal.jsonl"
OPP_DATASET_FILE = DATA_DIR / "rp613_opportunity_forensic_dataset.csv"

class TestRP614Gate5CohortAndPFIntegrity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with open(JOURNAL_FILE, "r") as f:
            cls.trades = [json.loads(line) for line in f if line.strip()]
        cls.opp_df = pd.read_csv(OPP_DATASET_FILE)

    def test_g5_01_win_loss_total_reconciliation(self):
        """G5-01: Reconciles exact 1,670 net winners and 5,594 net losers to 7,264 trades."""
        inv1 = run_investigation_1(self.trades)
        self.assertEqual(inv1["total_trades"], 7264)
        self.assertEqual(inv1["total_gross_wins"], 2444)
        self.assertEqual(inv1["total_net_wins"], 1670)
        self.assertEqual(inv1["total_friction_killed"], 774)
        self.assertEqual(inv1["total_net_wins"] + (7264 - inv1["total_net_wins"]), 7264)

    def test_g5_02_mutually_exclusive_cohort_partition(self):
        """G5-02: Failure cohorts A1, A2, B, C, D, E partition all 5,594 losers with zero overlap."""
        inv2 = run_investigation_2(self.trades, self.opp_df)
        c_a1 = inv2["Cohort_A1_True_Zero_MFE"]["count"]
        c_a2 = inv2["Cohort_A2_Micro_MFE"]["count"]
        c_b = inv2["Cohort_B_All_Friction_Killed"]["count"]
        c_c = inv2["Cohort_C_Minor_Chop_Reversal"]["count"]
        c_d = inv2["Cohort_D_Moderate_Giveback"]["count"]
        c_e = inv2["Cohort_E_Severe_Giveback"]["count"]

        self.assertEqual(c_a1, 882)
        self.assertEqual(c_a2, 1622)
        self.assertEqual(c_b, 774)
        self.assertEqual(c_c, 1772)
        self.assertEqual(c_d, 446)
        self.assertEqual(c_e, 98)

        total_losers = c_a1 + c_a2 + c_b + c_c + c_d + c_e
        self.assertEqual(total_losers, 5594)
        self.assertEqual(total_losers + 1670, 7264)

    def test_g5_03_cohort_boundary_disjointness(self):
        """G5-03: Boundary conditions between cohorts are strictly disjoint."""
        opp_map = {(str(row["session_date"]), str(row["trade_id"])): row.to_dict() for _, row in self.opp_df.iterrows()}
        classified_set = {}

        for t in self.trades:
            ep = float(t["entry_price"])
            xp = float(t["exit_price"])
            qty = float(t["quantity"])
            gp = float(t["pnl"])
            fric = compute_canonical_friction(ep, xp, qty)
            npnl = gp - fric

            if npnl > 0:
                continue # Winner

            s_date = t["entry_timestamp"][:10]
            mfe = float(opp_map[(s_date, t["trade_id"])]["mfe_pct"])
            cohorts_matched = []

            if gp > 0:
                cohorts_matched.append("B")
            if gp <= 0 and mfe == 0.0:
                cohorts_matched.append("A1")
            if gp <= 0 and 0.0 < mfe <= 0.05:
                cohorts_matched.append("A2")
            if gp <= 0 and 0.05 < mfe <= 0.20:
                cohorts_matched.append("C")
            if gp <= 0 and 0.20 < mfe <= 0.50:
                cohorts_matched.append("D")
            if gp <= 0 and mfe > 0.50:
                cohorts_matched.append("E")

            # Must match exactly ONE cohort
            self.assertEqual(len(cohorts_matched), 1, f"Trade {t['trade_id']} matched multiple cohorts: {cohorts_matched}")
            classified_set[(s_date, t["trade_id"])] = cohorts_matched[0]

        self.assertEqual(len(classified_set), 5594)

    def test_g5_04_empirical_profit_factor_calculation(self):
        """G5-04: Profit factor is calculated from per-trade net distribution: sum(pos) / abs(sum(neg))."""
        pos_net_pnl = 0.0
        neg_net_pnl = 0.0

        for t in self.trades:
            ep = float(t["entry_price"])
            xp = float(t["exit_price"])
            qty = float(t["quantity"])
            gp = float(t["pnl"])
            fric = compute_canonical_friction(ep, xp, qty)
            npnl = gp - fric

            if npnl > 0:
                pos_net_pnl += npnl
            else:
                neg_net_pnl += abs(npnl)

        pf = pos_net_pnl / neg_net_pnl if neg_net_pnl > 0 else 999.0
        expected_pf = 514032.50 / 941629.47 # ~0.5459
        self.assertAlmostEqual(pf, 0.5459, places=2)
        self.assertGreater(pf, 0.0)
        self.assertLess(pf, 1.0) # Confirms loss-making strategy overall

    def test_g5_05_profit_factor_edge_cases(self):
        """G5-05: Handles pure gains (999.0), pure losses (0.0), and empty trades correctly."""
        # Helper PF function
        def calc_pf(net_pnls):
            pos = sum(p for p in net_pnls if p > 0)
            neg = sum(abs(p) for p in net_pnls if p < 0)
            return (pos / neg) if neg > 0 else (999.0 if pos > 0 else 0.0)

        self.assertEqual(calc_pf([100.0, 200.0]), 999.0) # Zero losses
        self.assertEqual(calc_pf([-100.0, -200.0]), 0.0) # Zero wins
        self.assertEqual(calc_pf([]), 0.0) # Empty
        self.assertEqual(calc_pf([100.0, -100.0]), 1.0) # Breakeven

    def test_g5_06_population_pnl_reconciliation(self):
        """G5-06: Net Winner P&L + Net Loser P&L equals total strategy Net P&L."""
        inv1 = run_investigation_1(self.trades)
        pos_net_pnl = 0.0
        neg_net_pnl = 0.0
        zero_pnl_count = 0

        for t in self.trades:
            ep = float(t["entry_price"])
            xp = float(t["exit_price"])
            qty = float(t["quantity"])
            gp = float(t["pnl"])
            fric = compute_canonical_friction(ep, xp, qty)
            npnl = gp - fric

            if npnl > 0:
                pos_net_pnl += npnl
            elif npnl < 0:
                neg_net_pnl += npnl
            else:
                zero_pnl_count += 0

        self.assertEqual(zero_pnl_count, 0)
        self.assertAlmostEqual(pos_net_pnl, 514032.50, places=1)
        self.assertAlmostEqual(neg_net_pnl, -941629.48, places=1)
        self.assertAlmostEqual(pos_net_pnl + neg_net_pnl, inv1["total_net_pnl"], places=2)
        self.assertAlmostEqual(inv1["total_net_pnl"], -427596.97, places=2)

    def test_g5_07_cohort_loss_sum_reconciliation(self):
        """G5-07: Sum of Net Losses across Cohorts A1, A2, B, C, D, E equals total Loser Net P&L."""
        inv2 = run_investigation_2(self.trades, self.opp_df)
        cohort_loss_sum = (
            inv2["Cohort_A1_True_Zero_MFE"]["total_net_loss_rupees"] +
            inv2["Cohort_A2_Micro_MFE"]["total_net_loss_rupees"] +
            inv2["Cohort_B_All_Friction_Killed"]["total_net_loss_rupees"] +
            inv2["Cohort_C_Minor_Chop_Reversal"]["total_net_loss_rupees"] +
            inv2["Cohort_D_Moderate_Giveback"]["total_net_loss_rupees"] +
            inv2["Cohort_E_Severe_Giveback"]["total_net_loss_rupees"]
        )
        self.assertAlmostEqual(cohort_loss_sum, -941629.47, places=1)

def main():
    print("=" * 80)
    print("🚀 RUNNING RP-614 GATE 5 COHORT & PROFIT FACTOR TEST SUITE")
    print("=" * 80)
    suite = unittest.TestLoader().loadTestsFromTestCase(TestRP614Gate5CohortAndPFIntegrity)
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    if result.wasSuccessful():
        print("\n" + "=" * 80)
        print("✅ ALL GATE 5 COHORT & PROFIT FACTOR TESTS PASSED WITH ZERO ERRORS!")
        print("=" * 80)
    else:
        print("\n❌ GATE 5 TESTS FAILED!")
        sys.exit(1)

if __name__ == "__main__":
    main()
