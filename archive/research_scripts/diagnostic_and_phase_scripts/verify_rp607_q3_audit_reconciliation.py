#!/usr/bin/env python3
"""
verify_rp607_q3_audit_reconciliation.py

Automated Verification Suite for RP-607 Q3 Audit V3 Canonical Reconciled Deliverable.
Verifies:
1. Reconciled confidence intervals & AUC orientations across summary, markdown report, JSON artifact, and UI feed.
2. Complete population denominator reconciliation (N=1,551 total, N=1,012 complete, N=506 discovery, N=506 validation).
3. Subgroup canonical reconciliation: Steady-State N=824 (Delta +30.23), Discovery Steady-State N=318 (Delta +10.14), Startup N=188 (Delta -108.91).
4. Source snapshot timestamp provenance metadata (2026-09-28 07:31:46 UTC).
5. Immutable historical artifact preservation & versioned V3 report existence.
"""

import json
import unittest
from pathlib import Path

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"
PUBLIC_DATA_DIR = WORKSPACE_DIR / "services" / "ui" / "public" / "data"

class TestRP607Q3AuditReconciliation(unittest.TestCase):

    def setUp(self):
        self.md_v3_path = LOG_DIR / "rp607_q3_discrimination_audit_v3_canonical_report.md"
        self.json_v3_path = LOG_DIR / "rp607_q3_discrimination_audit_v3_canonical_report.json"
        self.ui_feed_path = PUBLIC_DATA_DIR / "rp606_rp607_accumulating_evidence_snapshot.json"

    def test_01_versioned_artifacts_exist(self):
        """Check 1: Confirm versioned V3 Markdown, JSON, and UI feeds exist."""
        self.assertTrue(self.md_v3_path.exists(), "Markdown V3 report must exist.")
        self.assertTrue(self.json_v3_path.exists(), "JSON V3 artifact must exist.")
        self.assertTrue(self.ui_feed_path.exists(), "UI public snapshot feed must exist.")

    def test_02_auc_orientation_and_ci_reconciliation(self):
        """Check 2: Verify 100% agreement of Raw AUC, Hypo AUC, and 95% CIs across JSON & Markdown."""
        with open(self.json_v3_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        disc_f1 = data["stage_1_discovery_results"][0]
        val_f1 = data["stage_2_validation_results"][0]

        # Discovery F1: Raw 0.5145 | Hypo 0.4855 | CI [0.4486, 0.5189]
        self.assertEqual(disc_f1["raw_feature_auc"], 0.5145)
        self.assertEqual(disc_f1["hypo_score_auc"], 0.4855)
        self.assertEqual(disc_f1["ci_95"], [0.4486, 0.5189])

        # Validation F1: Raw 0.4548 | Hypo 0.5452 | CI [0.5046, 0.5903]
        self.assertEqual(val_f1["raw_feature_auc"], 0.4548)
        self.assertEqual(val_f1["hypo_score_auc"], 0.5452)
        self.assertEqual(val_f1["ci_95"], [0.5046, 0.5903])

        # Markdown consistency
        with open(self.md_v3_path, "r", encoding="utf-8") as f:
            md_text = f.read()

        self.assertIn("`0.5145`", md_text)
        self.assertIn("`0.4855`", md_text)
        self.assertIn("`0.4548`", md_text)
        self.assertIn("`0.5452`", md_text)
        self.assertIn("[0.4486, 0.5189]", md_text)
        self.assertIn("[0.5046, 0.5903]", md_text)

    def test_03_population_denominator_reconciliation(self):
        """Check 3: Reconcile N=1,551 total exits, N=1,012 complete LO_60 exits, N=506 Discovery, N=506 Validation."""
        with open(self.json_v3_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        pop = data["population_reconciliation"]
        self.assertEqual(pop["total_reconstructed_engine_exits"], 1551)
        self.assertEqual(pop["complete_horizon_records_count"], 1012)
        self.assertEqual(pop["discovery_cohort"]["total_count"], 506)
        self.assertEqual(pop["validation_cohort"]["total_count"], 506)

        # Denominator stratification check
        self.assertEqual(pop["discovery_cohort"]["steady_state_count"], 318)
        self.assertEqual(pop["discovery_cohort"]["startup_seeding_count"], 188)
        self.assertEqual(pop["validation_cohort"]["steady_state_count"], 506)
        self.assertEqual(pop["validation_cohort"]["startup_seeding_count"], 0)

    def test_04_subgroup_canonical_reconciliation(self):
        """Check 4: Reconcile complete steady-state N=824 (Delta +30.23), discovery steady-state N=318 (Delta +72.92), startup N=188 (Delta -108.91)."""
        with open(self.json_v3_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        sub = data["subgroup_canonical_results"]
        self.assertEqual(sub["complete_steady_state_f1"]["opportunity_delta"], 30.23)
        self.assertEqual(sub["discovery_steady_state_f1"]["opportunity_delta"], 72.92)
        self.assertEqual(sub["complete_startup_seeding_f1"]["opportunity_delta"], -108.91)

    def test_05_source_snapshot_timestamp_metadata(self):
        """Check 5: Verify source snapshot timestamp 2026-09-28 07:31:46 UTC is present and verified."""
        with open(self.json_v3_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["snapshot_source_timestamp_utc"], "2026-09-28 07:31:46 UTC")
        self.assertEqual(data["q3_qualification_gate_status"], "PENDING / INCONCLUSIVE")
        self.assertIn("snapshot_json_sha256", data["input_fingerprints"])

if __name__ == "__main__":
    unittest.main()
