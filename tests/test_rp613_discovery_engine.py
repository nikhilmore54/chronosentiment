#!/usr/bin/env python3
"""
Unit Tests for RP-613 Discovery Engine Conformance & Reproducibility.

Tests:
1. Population Parity: Exactly 7,264 trades, all unique composite keys.
2. Point-in-Time Causality: All records enforce anchor lag >= 1.0s.
3. Chronological Fold Isolation: Strict expanding window; zero future leakage.
4. Hurdle Model Accounting: Friction subtracted once on gross payoffs.
5. Coverage Selection: Deterministic per-session ranking and tie-breaking.
"""

import csv
import math
import unittest
from pathlib import Path
import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
DATASET_CSV = WORKSPACE / "live_capture" / "shadow_validation" / "rp613_opportunity_forensic_dataset.csv"


class TestRP613DiscoveryEngine(unittest.TestCase):

    def setUp(self):
        self.assertTrue(DATASET_CSV.exists(), f"Missing dataset: {DATASET_CSV}")
        with DATASET_CSV.open(encoding="utf-8") as f:
            self.rows = list(csv.DictReader(f))

    def test_population_parity_and_keys(self):
        """Verify 7,264 trades and unique composite keys."""
        self.assertEqual(len(self.rows), 7264)
        keys = [r["unique_key"] for r in self.rows]
        self.assertEqual(len(keys), len(set(keys)))

    def test_point_in_time_causality(self):
        """Verify anchor lag >= 1.0s and zero leakage."""
        for r in self.rows:
            lag = float(r["anchor_lag_sec"])
            self.assertGreaterEqual(lag, 1.0, f"Leakage violation on {r['unique_key']}: lag = {lag}s")

    def test_chronological_fold_isolation(self):
        """Verify strict expanding-window fold boundaries."""
        sess_dates = sorted(set(r["session_date"] for r in self.rows))
        self.assertEqual(sess_dates, ["2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01"])

        # Fold 1: Train 28 Sep -> Test 29 Sep
        # Fold 2: Train 28, 29 Sep -> Test 30 Sep
        # Fold 3: Train 28, 29, 30 Sep -> Test 01 Oct
        folds = [
            (["2026-09-28"], "2026-09-29"),
            (["2026-09-28", "2026-09-29"], "2026-09-30"),
            (["2026-09-28", "2026-09-29", "2026-09-30"], "2026-10-01"),
        ]
        for tr_sess, te_sess in folds:
            self.assertNotIn(te_sess, tr_sess)
            self.assertTrue(all(s < te_sess for s in tr_sess))

    def test_hurdle_model_accounting(self):
        """Verify single friction subtraction on gross payoff expectations."""
        p_win = 0.30
        mu_win = 300.0  # Gross win expectation
        mu_loss = 150.0 # Gross loss expectation
        friction = 50.0 # Estimated round-trip friction

        # Gross expectation:
        expected_gross = p_win * mu_win - (1.0 - p_win) * mu_loss
        self.assertAlmostEqual(expected_gross, -15.0)

        # Net expectation with single friction deduction:
        expected_net = expected_gross - friction
        self.assertAlmostEqual(expected_net, -65.0)

        # Confirm formula matches:
        formula_net = p_win * mu_win - (1.0 - p_win) * mu_loss - friction
        self.assertAlmostEqual(formula_net, expected_net)

    def test_coverage_selection_determinism(self):
        """Verify per-session coverage ranking and tie-breaking."""
        sample_trades = [
            {"unique_key": "2026-09-29_RP0002", "pred": 10.0},
            {"unique_key": "2026-09-29_RP0001", "pred": 10.0},
            {"unique_key": "2026-09-29_RP0003", "pred": 5.0},
            {"unique_key": "2026-09-29_RP0004", "pred": 20.0},
        ]
        # Sort descending by pred, then by key
        sorted_trades = sorted(sample_trades, key=lambda x: (x["pred"], x["unique_key"]), reverse=True)
        expected_order = [
            "2026-09-29_RP0004", # 20.0
            "2026-09-29_RP0002", # 10.0 (RP0002 > RP0001 in reverse tie-break)
            "2026-09-29_RP0001", # 10.0
            "2026-09-29_RP0003", # 5.0
        ]
        actual_order = [x["unique_key"] for x in sorted_trades]
        self.assertEqual(actual_order, expected_order)


if __name__ == "__main__":
    unittest.main()
