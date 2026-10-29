#!/usr/bin/env python3
"""
tests/test_rp615_stage2c_replay.py

Unit test suite verifying Stage 2C Experiment A Frozen Benchmark Replay.
Asserts multi-session stability metrics, pooled cohorts, and manifest invariants.
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
MANIFEST_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2c_experiment_a_manifest.json"
REPORT_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2c_experiment_a_report.md"
PRED_LEDGER = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2c_experiment_a_prediction_ledger.csv"
COMPARE_CSV = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2c_experiment_a_comparison_dataset.csv"


class TestRP615Stage2CReplay(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not MANIFEST_JSON.exists():
            raise FileNotFoundError(f"Missing manifest JSON: {MANIFEST_JSON}")
        with MANIFEST_JSON.open("r", encoding="utf-8") as f:
            cls.manifest = json.load(f)

    def test_model_hash_and_decision_rule(self):
        self.assertEqual(self.manifest["model_candidate_hash"], "212532d3ac12f9c6")
        self.assertIn("P(Win) >= 0.35", self.manifest["decision_rule"])

    def test_pooled_cohort_counts(self):
        cohorts = self.manifest["pooled_cohorts"]
        added = cohorts["added_discovery"]
        orig = cohorts["original_discovery"]
        expanded = cohorts["expanded_discovery_total"]
        holdout = cohorts["diagnostic_holdout"]

        self.assertEqual(added["selected_n"], 37)
        self.assertEqual(orig["selected_n"], 237)
        self.assertEqual(expanded["selected_n"], 274)
        self.assertEqual(holdout["selected_n"], 51)

    def test_files_exist(self):
        self.assertTrue(REPORT_MD.exists())
        self.assertTrue(PRED_LEDGER.exists())
        self.assertTrue(COMPARE_CSV.exists())

    def test_standing_governance(self):
        gov = self.manifest["standing_governance"]
        self.assertEqual(gov["rp605_control"], "100% Live Execution Authority")
        self.assertEqual(gov["rp615_research"], "0% Live Authority (Research-Only)")


if __name__ == "__main__":
    unittest.main()
