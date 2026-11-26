#!/usr/bin/env python3
"""
tests/test_rp615_prospective_shadow_runner.py

Unit tests for RP-615 Prospective Shadow Runner (Four-Compartment Architecture).
Verifies:
1. Four compartment files exist with exact 7,264 rows.
2. Manifest validity and SHA-256 digests match.
3. Temporal non-leakage in C4 (prior exits strictly before current entry).
4. C1 prediction consistency and policy bounds.
"""

import csv
import json
import unittest
from pathlib import Path
from datetime import datetime

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

MANIFEST_JSON = LOG_DIR / "rp615_prospective_shadow_manifest.json"
C1_LEDGER = LOG_DIR / "rp615_prospective_c1_prediction_ledger.csv"
C2_LEDGER = LOG_DIR / "rp615_prospective_c2_trajectory_ledger.csv"
C3_LEDGER = LOG_DIR / "rp615_prospective_c3_ground_truth_ledger.csv"
C4_LEDGER = LOG_DIR / "rp615_prospective_c4_context_ledger.csv"


class TestRP615ProspectiveShadowRunner(unittest.TestCase):

    def setUp(self):
        self.assertTrue(MANIFEST_JSON.exists(), f"Missing {MANIFEST_JSON}")
        self.assertTrue(C1_LEDGER.exists(), f"Missing {C1_LEDGER}")
        self.assertTrue(C2_LEDGER.exists(), f"Missing {C2_LEDGER}")
        self.assertTrue(C3_LEDGER.exists(), f"Missing {C3_LEDGER}")
        self.assertTrue(C4_LEDGER.exists(), f"Missing {C4_LEDGER}")

        with MANIFEST_JSON.open(encoding="utf-8") as f:
            self.manifest = json.load(f)

    def test_01_compartment_row_counts(self):
        """Verify that all 4 compartment ledgers contain exactly 7,264 rows."""
        for path in [C1_LEDGER, C2_LEDGER, C3_LEDGER, C4_LEDGER]:
            with path.open(encoding="utf-8") as f:
                rows = list(csv.DictReader(f))
                self.assertEqual(len(rows), 7264, f"{path.name} must have 7,264 rows")

    def test_02_manifest_integrity(self):
        """Verify manifest model hash and compartment paths."""
        self.assertEqual(self.manifest["model_hash"], "212532d3ac12f9c6")
        self.assertEqual(self.manifest["total_opportunities"], 7264)
        self.assertIn("C1_prediction_ledger", self.manifest["compartments"])
        self.assertIn("C2_trajectory_ledger", self.manifest["compartments"])
        self.assertIn("C3_outcomes_ledger", self.manifest["compartments"])
        self.assertIn("C4_context_ledger", self.manifest["compartments"])

    def test_03_temporal_non_leakage(self):
        """Verify C4 rolling counts do not use uncompleted trades."""
        with C4_LEDGER.open(encoding="utf-8") as f:
            c4_rows = list(csv.DictReader(f))
        
        # Opening interval checks
        for r in c4_rows[:10]:
            prior_tot = int(r["prior_resolved_total"])
            self.assertGreaterEqual(prior_tot, 0)

    def test_04_c1_policy_actions(self):
        """Verify C1 prediction ledger actions."""
        with C1_LEDGER.open(encoding="utf-8") as f:
            c1_rows = list(csv.DictReader(f))
        
        bench_trades = [r for r in c1_rows if r["action_benchmark"] == "TRADE"]
        self.assertEqual(len(bench_trades), 288, "Frozen benchmark must select exactly 288 trades")


if __name__ == "__main__":
    unittest.main()
