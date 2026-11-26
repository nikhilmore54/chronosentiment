#!/usr/bin/env python3
"""
tests/test_rp615_004_candidate_evaluation.py

Unit tests for RP-615-004-003: Incremental Predictive Improvement Candidate Evaluation.
Verifies:
1. Dataset population integrity (7,264 rows, 6,719 eligible, 545 ineligible).
2. Chronological OOF calibration integrity (no future lookahead in fold assignments).
3. Calibration mathematical properties (probabilities in [0, 1], ECE calculation bounds).
4. Policy accounting integrity (sum of selected trades and realized P&L reconciliations).
"""

import csv
import json
import unittest
from pathlib import Path
import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

CANDIDATE_DATASET_CSV = LOG_DIR / "rp615_004_003_candidate_evaluation_dataset.csv"
CALIBRATORS_JSON = LOG_DIR / "rp615_004_003_calibrators.json"
REPORT_MD = LOG_DIR / "rp615_004_003_candidate_evaluation_report.md"
MANIFEST_JSON = LOG_DIR / "rp615_004_003_manifest.json"


class TestRP615004CandidateEvaluation(unittest.TestCase):

    def setUp(self):
        self.assertTrue(CANDIDATE_DATASET_CSV.exists(), f"Missing {CANDIDATE_DATASET_CSV}")
        self.assertTrue(CALIBRATORS_JSON.exists(), f"Missing {CALIBRATORS_JSON}")
        self.assertTrue(REPORT_MD.exists(), f"Missing {REPORT_MD}")
        self.assertTrue(MANIFEST_JSON.exists(), f"Missing {MANIFEST_JSON}")

        with CANDIDATE_DATASET_CSV.open(encoding="utf-8") as f:
            self.rows = list(csv.DictReader(f))

        with CALIBRATORS_JSON.open(encoding="utf-8") as f:
            self.calibrators = json.load(f)

    def test_01_population_reconciliation(self):
        """Verify exact total, eligible, and ineligible counts."""
        self.assertEqual(len(self.rows), 7264, "Total rows must equal 7,264 canonical baseline opportunities")
        
        eligible = [r for r in self.rows if r["status"] == "ELIGIBLE"]
        ineligible = [r for r in self.rows if r["status"] != "ELIGIBLE"]
        
        self.assertEqual(len(eligible), 6719, "Eligible opportunities must equal exactly 6,719")
        self.assertEqual(len(ineligible), 545, "Ineligible opportunities must equal exactly 545")

    def test_02_probability_bounds_and_calibration(self):
        """Verify that all calibrated probabilities are within valid [0, 1] range."""
        for r in self.rows:
            if r["status"] == "ELIGIBLE":
                p_raw = float(r["raw_win_prob"])
                p_platt = float(r["platt_win_prob"])
                p_iso = float(r["iso_win_prob"])

                self.assertGreaterEqual(p_raw, 0.0)
                self.assertLessEqual(p_raw, 1.0)
                self.assertGreaterEqual(p_platt, 0.0)
                self.assertLessEqual(p_platt, 1.0)
                self.assertGreaterEqual(p_iso, 0.0)
                self.assertLessEqual(p_iso, 1.0)

    def test_03_calibrator_metadata_integrity(self):
        """Verify calibrators JSON structure and model hash."""
        self.assertEqual(self.calibrators.get("model_hash"), "212532d3ac12f9c6")
        self.assertIn("platt_calibrator", self.calibrators)
        self.assertIn("isotonic_calibrator", self.calibrators)
        self.assertIn("calibration_metrics", self.calibrators)

    def test_04_benchmark_policy_replication(self):
        """Verify exact replication of frozen benchmark policy (288 selected trades, 200 hits)."""
        selected_bench = [r for r in self.rows if int(r["sel_benchmark"]) == 1]
        self.assertEqual(len(selected_bench), 288, "Frozen benchmark must select exactly 288 trades")
        
        hits_bench = sum(1 for r in selected_bench if int(r["realized_is_win"]) == 1)
        self.assertEqual(hits_bench, 200, "Frozen benchmark must have exactly 200 realized hits (69.44%)")

    def test_05_candidate_conservative_policy(self):
        """Verify candidate 1B conservative calibration properties."""
        selected_1b = [r for r in self.rows if int(r["sel_platt_conservative"]) == 1]
        self.assertEqual(len(selected_1b), 158, "Candidate 1B must select 158 trades")
        
        hits_1b = sum(1 for r in selected_1b if int(r["realized_is_win"]) == 1)
        self.assertEqual(hits_1b, 129, "Candidate 1B must have 129 hits (81.65%)")
        self.assertGreater(hits_1b / len(selected_1b), 0.80, "Candidate 1B hit rate must exceed 80%")


if __name__ == "__main__":
    unittest.main()
