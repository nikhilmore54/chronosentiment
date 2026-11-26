#!/usr/bin/env python3
"""
tests/test_rp615_crypto_stage2_model.py

Unit tests for RP-615-Crypto Stage C2:
Crypto Model Adaptation, Walk-Forward Calibration & Candidate Freeze.
"""

import csv
import hashlib
import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

OUT_MODEL_WEIGHTS_JSON = LOG_DIR / "rp615_crypto_stage2_model_weights.json"
OUT_CALIBRATORS_JSON = LOG_DIR / "rp615_crypto_stage2_calibrators.json"
OUT_PREDICTION_LEDGER_CSV = LOG_DIR / "rp615_crypto_stage2_prediction_ledger.csv"
OUT_EXECUTION_LEDGER_CSV = LOG_DIR / "rp615_crypto_stage2_execution_ledger.csv"
OUT_SUMMARY_JSON = LOG_DIR / "rp615_crypto_stage2_summary.json"
OUT_MANIFEST_JSON = LOG_DIR / "rp615_crypto_stage2_manifest.json"
OUT_REPORT_MD = LOG_DIR / "rp615_crypto_stage2_candidate_report.md"
NSE_FROZEN_LEDGER = LOG_DIR / "rp615_tier4_frozen_ledger.csv"


def sha256_file(path: Path) -> str:
    if not path.exists():
        return "FILE_NOT_FOUND"
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


class TestRP615CryptoStage2Model(unittest.TestCase):

    def test_01_artifacts_exist(self):
        """Verify all Stage C2 artifacts are generated and non-empty."""
        for path in [
            OUT_MODEL_WEIGHTS_JSON,
            OUT_CALIBRATORS_JSON,
            OUT_PREDICTION_LEDGER_CSV,
            OUT_EXECUTION_LEDGER_CSV,
            OUT_SUMMARY_JSON,
            OUT_MANIFEST_JSON,
            OUT_REPORT_MD,
        ]:
            self.assertTrue(path.exists(), f"Missing artifact: {path.name}")
            self.assertGreater(path.stat().st_size, 0, f"Empty artifact: {path.name}")

    def test_02_model_weights_and_calibrators(self):
        """Verify model weights and Platt calibrators schema."""
        with OUT_MODEL_WEIGHTS_JSON.open() as f:
            mw = json.load(f)

        self.assertEqual(mw["metadata"]["track"], "RP-615-Crypto")
        self.assertEqual(mw["metadata"]["stage"], "C2")
        self.assertEqual(mw["metadata"]["k_submodels"], 10)
        self.assertIn("platt_calibrator", mw)
        self.assertIn("a", mw["platt_calibrator"])
        self.assertIn("b", mw["platt_calibrator"])
        self.assertIn("feature_scaling", mw)
        self.assertEqual(len(mw["feature_scaling"]["means"]), 17)

        with OUT_CALIBRATORS_JSON.open() as f:
            cal = json.load(f)
        self.assertIn("platt_a", cal)
        self.assertIn("platt_b", cal)
        self.assertIn("residual_scale", cal)

    def test_03_prediction_ledger_geometry(self):
        """Verify prediction ledger contains all 12,219 opportunities and required columns."""
        with OUT_PREDICTION_LEDGER_CSV.open() as f:
            reader = csv.DictReader(f)
            rows = list(reader)

        self.assertEqual(len(rows), 12219, "Prediction ledger row count mismatch")
        required_cols = [
            "trade_id", "session_date", "symbol", "p_calibrated",
            "pred_net_pnl", "sigma_mu", "lcb_mean", "score_adj", "p_be",
            "gate_pass_C0_Reference", "gate_pass_C5_Fixed_Capacity_Top2Pct"
        ]
        for col in required_cols:
            self.assertIn(col, rows[0], f"Missing column: {col}")

    def test_04_execution_ledger_and_candidate_results(self):
        """Verify candidate summary and execution ledger parity."""
        with OUT_SUMMARY_JSON.open() as f:
            summary = json.load(f)

        cands = summary["candidates"]
        self.assertIn("C0_Reference", cands)
        self.assertIn("C5_Fixed_Capacity_Top2Pct", cands)

        # C0 baseline must match Stage C1 exactly (374 trades, -$4,807.89 Net P&L)
        self.assertEqual(cands["C0_Reference"]["executed_trades"], 374)
        self.assertAlmostEqual(cands["C0_Reference"]["net_pnl"], -4807.89, places=2)

        # C5 lead hypothesis must achieve positive net P&L and superior profit factor
        self.assertGreater(cands["C5_Fixed_Capacity_Top2Pct"]["net_pnl"], 0.0)
        self.assertGreater(cands["C5_Fixed_Capacity_Top2Pct"]["profit_factor"], 1.5)

    def test_05_friction_accounting_integrity(self):
        """Verify friction is exactly 18.0 bps ($18.00 per trade) across all executed trades."""
        with OUT_EXECUTION_LEDGER_CSV.open() as f:
            reader = csv.DictReader(f)
            trades = list(reader)

        for t in trades:
            gross = float(t["realized_gross_pnl"])
            net = float(t["realized_net_pnl"])
            diff = gross - net
            self.assertAlmostEqual(diff, 18.00, places=2, msg=f"Friction mismatch for {t['trade_id']}")

    def test_06_cross_track_isolation(self):
        """Verify zero contamination on NSE frozen ledger."""
        with OUT_MANIFEST_JSON.open() as f:
            manifest = json.load(f)

        manifest_nse_hash = manifest["checksums"]["nse_frozen_ledger_csv"]
        actual_nse_hash = sha256_file(NSE_FROZEN_LEDGER)
        self.assertEqual(manifest_nse_hash, actual_nse_hash, "NSE frozen ledger hash mismatch")


if __name__ == "__main__":
    unittest.main()
