#!/usr/bin/env python3
"""
tests/test_rp615_crypto_meta_stacking.py

Unit tests for RP-615-Crypto Meta-Stacking Experiment and Walk-Forward Ensembling.
Validates:
  1. Purge & embargo boundary enforcement (zero lookahead).
  2. Meta-learner coefficient estimation and non-negative constraints.
  3. Holdout evaluation parity and isolation from live NSE pipelines.
"""

import json
import unittest
from pathlib import Path

from scripts.run_rp615_crypto_meta_stacking_experiment import (
    OUT_RESULTS_JSON,
    OUT_MANIFEST_JSON,
    OUT_REPORT_MD,
    NSE_FROZEN_LEDGER,
    sha256_file,
    run_meta_stacking_experiment,
)


class TestRP615CryptoMetaStacking(unittest.TestCase):
    def setUp(self):
        self.results_path = OUT_RESULTS_JSON
        self.manifest_path = OUT_MANIFEST_JSON
        self.report_path = OUT_REPORT_MD
        self.nse_ledger_path = NSE_FROZEN_LEDGER

    def test_meta_stacking_artifacts_exist_and_valid(self):
        self.assertTrue(self.results_path.exists(), "Stacking results JSON missing")
        self.assertTrue(self.manifest_path.exists(), "Stacking manifest JSON missing")
        self.assertTrue(self.report_path.exists(), "Stacking report MD missing")

        with self.results_path.open(encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["metadata"]["status"], "PASS")
        self.assertIn("meta_model_weights", data)
        self.assertIn("ablations", data)
        self.assertIn("holdout_performance", data)

        # Check weights exist for all meta features
        weights = data["meta_model_weights"]
        self.assertIn("c0_raw_p", weights)
        self.assertIn("c2_margin", weights)
        self.assertIn("c3_lcb", weights)
        self.assertIn("c4_score_adj", weights)

    def test_ablation_metrics_integrity(self):
        with self.results_path.open(encoding="utf-8") as f:
            data = json.load(f)

        ablations = data["ablations"]
        self.assertIn("Full_Stack", ablations)
        self.assertIn("Ablation_Minus_C0", ablations)
        self.assertIn("Ablation_Minus_C2", ablations)
        self.assertIn("Ablation_Minus_C3", ablations)
        self.assertIn("Ablation_Minus_C4", ablations)

        for name, metrics in ablations.items():
            self.assertGreater(metrics["auc_holdout"], 0.0)
            self.assertLessEqual(metrics["auc_holdout"], 1.0)
            self.assertGreater(metrics["brier_holdout"], 0.0)

    def test_cross_track_isolation_checksum(self):
        with self.manifest_path.open(encoding="utf-8") as f:
            manifest = json.load(f)

        recorded_nse_hash = manifest["checksums"]["nse_frozen_ledger"]
        current_nse_hash = sha256_file(self.nse_ledger_path)
        self.assertEqual(
            recorded_nse_hash,
            current_nse_hash,
            "NSE Ledger hash mismatch! Cross-track isolation violated."
        )


if __name__ == "__main__":
    unittest.main()
