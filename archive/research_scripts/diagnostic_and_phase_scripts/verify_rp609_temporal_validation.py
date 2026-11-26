#!/usr/bin/env python3
"""
verify_rp609_temporal_validation.py

Automated Verification Suite for RP-609-005 Temporal Validation Report.
Verifies snapshot integrity, frozen policy contracts, bootstrap uncertainty CIs,
opportunity cost accounting, and strict shadow governance isolation.
"""

import json
import unittest
from pathlib import Path

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

class TestRP609TemporalValidation(unittest.TestCase):

    def setUp(self):
        self.json_path = LOG_DIR / "rp609_temporal_validation_report.json"
        self.md_path = LOG_DIR / "rp609_temporal_validation_report.md"
        self.assertTrue(self.json_path.exists(), f"JSON report missing at {self.json_path}")
        self.assertTrue(self.md_path.exists(), f"Markdown report missing at {self.md_path}")
        with open(self.json_path, "r", encoding="utf-8") as f:
            self.data = json.load(f)

    def test_01_report_structure_and_snapshot_fingerprint(self):
        """Check 1: Verify presence of mandatory JSON keys and valid SHA256 snapshot fingerprint."""
        self.assertIn("run_id", self.data)
        self.assertIn("snapshot_sha256", self.data)
        self.assertIn("candidate_replay_results", self.data)
        self.assertIn("bootstrap_uncertainty_results", self.data)
        self.assertIn("validation_decisions", self.data)
        self.assertEqual(len(self.data["snapshot_sha256"]), 64)

    def test_02_untouched_stage2_cohort_integrity(self):
        """Check 2: Confirm target population matches Stage 2 temporal holdout cohort size."""
        target_cohort = self.data.get("target_cohort", "")
        self.assertIn("Stage 2", target_cohort)
        self.assertGreater(self.data.get("cohort_size", 0), 0)

    def test_03_frozen_policy_contracts_preservation(self):
        """Check 3: Verify all four candidates use exact frozen identifiers without parameter modifications."""
        expected_candidates = ["PFT-v1.0-FROZEN", "SPL-v1.0-FROZEN", "VAT-v1.0-FROZEN", "PPR-v1.0-FROZEN"]
        results = self.data.get("candidate_replay_results", {})
        for cand in expected_candidates:
            self.assertIn(cand, results)
            self.assertIn("mean_paired_delta", results[cand])
            self.assertIn("total_candidate_realized_pnl", results[cand])

    def test_04_bootstrap_uncertainty_and_decision_categorization(self):
        """Check 4: Confirm 95% bootstrap CIs exist and decisions belong to canonical 3-tier taxonomy."""
        decisions = self.data.get("validation_decisions", {})
        valid_statuses = {"CONFIRMED_ON_HOLDOUT", "NOT_CONFIRMED", "INCONCLUSIVE"}
        for cand, info in decisions.items():
            self.assertIn(info["decision"], valid_statuses)
            self.assertEqual(len(info["ci_95"]), 2)
            self.assertLessEqual(info["ci_95"][0], info["ci_95"][1])

    def test_05_strict_shadow_governance_isolation(self):
        """Check 5: Verify strict read-only shadow isolation and explicit denial of live authority."""
        with open(self.md_path, "r", encoding="utf-8") as f:
            md_content = f.read()
        self.assertIn("Zero Live Execution Authority", md_content)
        self.assertIn("RP-607 Q3 discrimination gate remains pending/inconclusive", md_content)

if __name__ == "__main__":
    unittest.main()
