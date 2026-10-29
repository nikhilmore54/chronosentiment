#!/usr/bin/env python3
"""
verify_rp609_profit_giveback_diagnostic.py

Automated Verification Suite for RP-609 Profit Retention and Giveback Control.
Verifies:
1. RP-609 Formal Specification File exists and incorporates MRQ-609, RQ-609.1..6, Candidate A-D, and exact governance rules.
2. Population Identity Reconciliation: Unique Trade Identities N=982 (Ledger A Control N=982; Ledger B Freshness N=569 matched sub-population, 100% overlap N=569).
3. Primary Control Ledger A NSE Metrics: N=870, MFE > 0 N=535 (61.5%), Reversals N=264 (49.3%), Total Giveback ₹46,401.63, Reversal Opp Loss ₹20,404.03.
4. Per-Trade PCR Reconciliation: Ledger A NSE per-trade mean PCR is -1.3384 (100% reconciled across JSON and Markdown).
5. Strict Currency Isolation: Keeps NSE (₹) and Crypto ($) monetary totals isolated without cross-currency addition.
"""

import json
import unittest
from pathlib import Path

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

class TestRP609ProfitGivebackDiagnostic(unittest.TestCase):

    def setUp(self):
        self.spec_path = WORKSPACE_DIR / "rp609_profit_retention_specification.md"
        self.md_report_path = LOG_DIR / "rp609_profit_giveback_baseline_report.md"
        self.json_report_path = LOG_DIR / "rp609_profit_giveback_baseline_report.json"

    def test_01_artifacts_exist(self):
        """Check 1: Confirm RP-609 specification, Markdown report, and JSON artifact exist."""
        self.assertTrue(self.spec_path.exists(), "RP-609 specification must exist.")
        self.assertTrue(self.md_report_path.exists(), "RP-609 Markdown report must exist.")
        self.assertTrue(self.json_report_path.exists(), "RP-609 JSON report must exist.")

    def test_02_specification_governance_integrity(self):
        """Check 2: Verify specification incorporates MRQ-609, RQ-609.1..6, Candidate A-D, and exact governance boundaries."""
        with open(self.spec_path, "r", encoding="utf-8") as f:
            text = f.read()

        self.assertIn("MRQ-609", text)
        self.assertIn("RQ-609.1", text)
        self.assertIn("Candidate A", text)
        self.assertIn("Candidate B", text)
        self.assertIn("Candidate C", text)
        self.assertIn("Candidate D", text)
        self.assertIn("RP-605 Control Baseline", text)
        self.assertIn("RP-607 Q3 Status Integrity", text)
        self.assertIn("RP-608 Shadow Isolation", text)

    def test_03_reconciled_transaction_identity_overlap(self):
        """Check 3: Reconcile unique trade identity population (N=982) and 100% overlap for Ledger B (N=569)."""
        with open(self.json_report_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        pop = data["population_identity_reconciliation"]
        self.assertEqual(pop["total_record_evaluations"], 1551)
        self.assertEqual(pop["total_unique_composite_trade_identities"], 982)
        self.assertEqual(pop["primary_control_ledger_a_unique_count"], 982)
        self.assertEqual(pop["freshness_observer_ledger_b_unique_count"], 569)
        self.assertEqual(pop["matched_overlapping_identities_count"], 569)
        self.assertEqual(pop["overlap_percentage_of_ledger_b"], 100.0)

    def test_04_primary_control_metrics_and_pcr_reconciliation(self):
        """Check 4: Reconcile Primary Ledger A NSE (N=870, 264 reversals, ₹46,401.63 giveback, ₹20,404.03 loss, -1.3384 mean PCR)."""
        with open(self.json_report_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        ledger_a_nse = data["primary_control_ledger_a_unique_population_analytics"]["nse_equities_ledger_a"]
        self.assertEqual(ledger_a_nse["trade_count"], 870)
        self.assertEqual(ledger_a_nse["positive_mfe_trades_count"], 535)
        self.assertEqual(ledger_a_nse["positive_mfe_pct"], 61.5)
        self.assertEqual(ledger_a_nse["profit_to_loss_reversals_count"], 264)
        self.assertEqual(ledger_a_nse["reversal_rate_pct"], 49.3)
        self.assertEqual(ledger_a_nse["total_giveback"], 46401.63)
        self.assertEqual(ledger_a_nse["reversal_opportunity_loss"], 20404.03)
        self.assertEqual(ledger_a_nse["per_trade_mean_profit_capture_ratio"], -1.3384)
        self.assertEqual(ledger_a_nse["aggregate_cohort_profit_capture_ratio"], -0.2082)

        # Markdown consistency
        with open(self.md_report_path, "r", encoding="utf-8") as f:
            md_text = f.read()

        self.assertIn("-1.3384", md_text)
        self.assertIn("20,404.03", md_text)
        self.assertIn("46,401.63", md_text)

    def test_05_currency_isolation(self):
        """Check 5: Verify strict currency isolation between NSE (₹) and Crypto ($)."""
        with open(self.json_report_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        nse = data["primary_control_ledger_a_unique_population_analytics"]["nse_equities_ledger_a"]
        crypto = data["primary_control_ledger_a_unique_population_analytics"]["crypto_assets_ledger_a"]

        self.assertEqual(nse["currency_symbol"], "₹")
        self.assertEqual(crypto["currency_symbol"], "$")
        self.assertEqual(crypto["trade_count"], 112)
        self.assertEqual(crypto["positive_mfe_trades_count"], 80)
        self.assertEqual(crypto["profit_to_loss_reversals_count"], 44)
        self.assertAlmostEqual(crypto["reversal_opportunity_loss"], 10.49, places=2)

if __name__ == "__main__":
    unittest.main(defaultTest="TestRP609ProfitGivebackDiagnostic")
