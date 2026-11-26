#!/usr/bin/env python3
"""
tests/test_rp615_prospective_audit.py

Unit tests for RP-615 Prospective Shadow Audit & Integrity Verification.
Verifies all 6 Audit Gates:
1. Manifest SHA-256 Checksums.
2. Exact Population Accounting (7,264 / 6,719 / 545).
3. 1:1 Composite Key Linkages Across C1–C4.
4. Causal Temporal Non-Leakage (C4 strictly prior resolved exits).
5. Financial Accounting Reconciliation (Gross - Friction = Net).
6. Deterministic Policy Selection Replay (288 / 303 / 158).
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

AUDIT_JSON = LOG_DIR / "rp615_prospective_audit_results.json"
AUDIT_REPORT_MD = LOG_DIR / "rp615_prospective_audit_report.md"


class TestRP615ProspectiveAudit(unittest.TestCase):

    def setUp(self):
        self.assertTrue(AUDIT_JSON.exists(), f"Missing {AUDIT_JSON}")
        self.assertTrue(AUDIT_REPORT_MD.exists(), f"Missing {AUDIT_REPORT_MD}")

        with AUDIT_JSON.open(encoding="utf-8") as f:
            self.audit = json.load(f)

    def test_gate1_manifest_and_hashes(self):
        """Verify all 4 compartment digests match manifest."""
        self.assertTrue(self.audit["all_digests_match"], "All compartment SHA-256 digests must match manifest")
        self.assertEqual(self.audit["model_hash"], "212532d3ac12f9c6")

    def test_gate2_population_accounting(self):
        """Verify exact population accounting."""
        self.assertEqual(self.audit["total_records"], 7264)
        self.assertEqual(self.audit["eligible_records"], 6719)
        self.assertEqual(self.audit["ineligible_records"], 545)

    def test_gate3_key_linkages(self):
        """Verify zero composite key mismatches across C1-C4."""
        self.assertEqual(self.audit["key_alignment_errors"], 0)

    def test_gate4_causal_non_leakage(self):
        """Verify zero temporal non-leakage violations."""
        self.assertEqual(self.audit["c4_causal_leakage_violations"], 0)

    def test_gate5_accounting_reconciliation(self):
        """Verify gross P&L - friction = net P&L invariant."""
        self.assertEqual(self.audit["accounting_errors"], 0)
        self.assertLessEqual(self.audit["max_accounting_delta"], 0.05)

    def test_gate6_deterministic_policy_replay(self):
        """Verify deterministic policy selection counts."""
        bench = self.audit["policy_metrics"]["benchmark"]
        platt_std = self.audit["policy_metrics"]["platt_std"]
        platt_cons = self.audit["policy_metrics"]["platt_cons"]

        self.assertEqual(bench["n"], 288)
        self.assertEqual(bench["hits"], 200)

        self.assertEqual(platt_std["n"], 303)
        self.assertEqual(platt_std["hits"], 210)

        self.assertEqual(platt_cons["n"], 158)
        self.assertEqual(platt_cons["hits"], 129)

    def test_overall_status(self):
        """Verify overall audit status is 100% passed."""
        self.assertEqual(self.audit["audit_status"], "PASSED_100_PERCENT")


if __name__ == "__main__":
    unittest.main()
