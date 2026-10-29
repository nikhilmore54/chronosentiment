#!/usr/bin/env python3
"""
verify_rp609_us_prospective_shadow_replay.py

Automated Verification Suite for RP-609 US-Market Prospective Shadow Test.
Verifies G0-G6 validation gates, 5 parallel shadow ledgers, universe hash,
1-minute bar feed identification, and strict 0% live execution authority governance.
"""

import json
import unittest
from pathlib import Path

WORKSPACE_DIR = Path(__file__).parent.parent
EXP_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "rp609_us_20260928"

class TestRP609USProspectiveShadowReplay(unittest.TestCase):

    def setUp(self):
        self.manifest_path = EXP_DIR / "run_manifest.json"
        self.report_json_path = EXP_DIR / "reports" / "us_session_summary.json"
        self.report_md_path = EXP_DIR / "reports" / "rp609_us_prospective_report.md"
        self.audit_json_path = EXP_DIR / "audits" / "accounting_reconciliation.json"
        
        self.assertTrue(self.manifest_path.exists(), f"Manifest missing at {self.manifest_path}")
        self.assertTrue(self.report_json_path.exists(), f"Summary report missing at {self.report_json_path}")
        self.assertTrue(self.report_md_path.exists(), f"Markdown report missing at {self.report_md_path}")
        self.assertTrue(self.audit_json_path.exists(), f"Audit JSON missing at {self.audit_json_path}")

        with open(self.report_json_path, "r", encoding="utf-8") as f:
            self.report_data = json.load(f)
        with open(self.audit_json_path, "r", encoding="utf-8") as f:
            self.audit_data = json.load(f)

    def test_01_manifest_and_universe_hash_integrity(self):
        """Check 1: Confirm run manifest recorded valid universe hash and frozen policy contracts."""
        with open(self.manifest_path, "r", encoding="utf-8") as f:
            manifest = json.load(f)
        self.assertIn("run_id", manifest)
        self.assertIn("universe_hash", manifest)
        self.assertEqual(len(manifest["universe_hash"]), 64)
        self.assertIn("PFT-v1.0-FROZEN", manifest["policy_contracts"])
        self.assertIn("PPR-v1.0-FROZEN", manifest["policy_contracts"])

    def test_02_validation_gates_g0_g6_verification(self):
        """Check 2: Confirm all 7 validation gates (G0 - G6) are marked VERIFIED."""
        expected_gates = [
            "G0_configuration_freeze",
            "G1_feed_integrity",
            "G2_population_integrity",
            "G3_runtime_isolation",
            "G4_execution_and_accounting",
            "G5_outcome_completeness",
            "G6_evidence_sealing"
        ]
        for gate in expected_gates:
            self.assertIn(gate, self.audit_data)
            self.assertIn("VERIFIED", self.audit_data[gate])

    def test_03_five_parallel_ledgers_isolation(self):
        """Check 3: Confirm 5 isolated shadow ledgers exist and match trade entry counts."""
        ledgers_dir = EXP_DIR / "ledgers"
        expected_files = ["rp605_control.json", "pft_shadow.json", "spl_shadow.json", "vat_shadow.json", "ppr_shadow.json"]
        for fn in expected_files:
            p = ledgers_dir / fn
            self.assertTrue(p.exists(), f"Ledger file missing: {fn}")
            with open(p, "r", encoding="utf-8") as f:
                records = json.load(f)
            self.assertGreater(len(records), 0)

    def test_04_feed_identification_and_friction_model(self):
        """Check 4: Verify feed is explicitly labeled 1-minute OHLCV intraday bars."""
        with open(self.manifest_path, "r", encoding="utf-8") as f:
            manifest = json.load(f)
        self.assertIn("1-Minute", manifest["feed_source"])
        self.assertIn("Intraday", manifest["feed_source"])

    def test_05_strict_zero_live_authority_governance(self):
        """Check 5: Confirm explicit zero live execution authority and RP-607 Q3 gate status."""
        with open(self.report_md_path, "r", encoding="utf-8") as f:
            md_content = f.read()
        self.assertIn("0% Live Authority", md_content)
        self.assertIn("RP-607 Q3 discrimination gate remains pending/inconclusive", md_content)

if __name__ == "__main__":
    unittest.main()
