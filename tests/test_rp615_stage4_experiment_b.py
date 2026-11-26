#!/usr/bin/env python3
"""
tests/test_rp615_stage4_experiment_b.py

Unit test suite verifying the execution, accounting, and statistical diagnostics
of RP-615 Stage 4: Experiment B Candidate Replay.
"""

import csv
import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

MANIFEST_JSON = LOG_DIR / "rp615_stage4_experiment_b_manifest.json"
OPP_LEDGER_CSV = LOG_DIR / "rp615_stage4_experiment_b_opportunity_ledger.csv"
EXEC_LEDGER_CSV = LOG_DIR / "rp615_stage4_experiment_b_execution_ledger.csv"
REPORT_MD = LOG_DIR / "rp615_stage4_experiment_b_report.md"
MODEL_WEIGHTS_JSON = LOG_DIR / "rp615_stage4_experiment_b_model_weights.json"


class TestRP615Stage4ExperimentB(unittest.TestCase):

    def test_artifacts_exist(self):
        self.assertTrue(MANIFEST_JSON.exists(), f"Missing {MANIFEST_JSON}")
        self.assertTrue(OPP_LEDGER_CSV.exists(), f"Missing {OPP_LEDGER_CSV}")
        self.assertTrue(EXEC_LEDGER_CSV.exists(), f"Missing {EXEC_LEDGER_CSV}")
        self.assertTrue(REPORT_MD.exists(), f"Missing {REPORT_MD}")
        self.assertTrue(MODEL_WEIGHTS_JSON.exists(), f"Missing {MODEL_WEIGHTS_JSON}")

    def test_manifest_and_hash_integrity(self):
        with MANIFEST_JSON.open(encoding="utf-8") as f:
            data = json.load(f)
        
        self.assertIn("hashes", data)
        self.assertEqual(data["hashes"]["benchmark_model_hash"], "212532d3ac12f9c6")
        self.assertIn("spearman_dispersion_correlation", data)
        self.assertGreater(data["spearman_dispersion_correlation"]["rho"], 0.0)
        self.assertIn("lpb_empirical_coverage", data)
        self.assertGreaterEqual(data["lpb_empirical_coverage"]["Pooled_Total"]["coverage_pct"], 90.0)

    def test_opportunity_ledger_coverage(self):
        with OPP_LEDGER_CSV.open(encoding="utf-8") as f:
            reader = csv.DictReader(f)
            rows = list(reader)
        self.assertEqual(len(rows), 19701, f"Expected 19,701 eligible rows, got {len(rows)}")

    def test_governance_authority_boundary(self):
        with MANIFEST_JSON.open(encoding="utf-8") as f:
            data = json.load(f)
        self.assertIn("RP-605 Control (100%)", data["governance_live_authority"])
        self.assertIn("RP-615 Research-Only (0%)", data["governance_live_authority"])


if __name__ == "__main__":
    unittest.main()
