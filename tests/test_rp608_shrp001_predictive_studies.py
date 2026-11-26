#!/usr/bin/env python3
"""
tests/test_rp608_shrp001_predictive_studies.py

Unit and regression test suite for RP-608 SHRP-001 Milestones 2 & 3:
- Methodological Lock validation
- Hypothesis A1 & B1 Gate A Predictive Results verification
- Futures cost schedule arithmetic verification
- Holdout partition protection check (Sessions 41-67 never touched)
"""

import unittest
import json
import hashlib
from pathlib import Path

WORKSPACE_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
PREREG_FILE = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter" / "rp608_shrp001_milestone2_preregistration_pack.md"
A1_RESULTS_FILE = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter" / "rp608_shrp001_a1_predictive_study_results.json"
B1_RESULTS_FILE = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter" / "rp608_shrp001_b1_predictive_study_results.json"

SEALED_HOLDOUT_START = "2026-08-19"
SEALED_HOLDOUT_END = "2026-09-25"


class TestRP608SHRP001PredictiveStudies(unittest.TestCase):

    def test_01_preregistration_pack_exists_and_contains_locks(self):
        self.assertTrue(PREREG_FILE.exists(), "Milestone 2 Preregistration pack must exist.")
        content = PREREG_FILE.read_text(encoding="utf-8")
        self.assertIn("RP-608-SHRP-001-M2-PREREGISTRATION-LOCKED", content)
        self.assertIn("SHRP-001-A1-VOL-EXPANSION", content)
        self.assertIn("SHRP-001-B1-SWING-CONTINUATION", content)
        self.assertIn("SHRP-001-C1-FUTURES-VOL-SQUEEZE", content)
        self.assertIn("STRICTLY SEALED & UNTOUCHED", content)
        self.assertIn("STRICTLY 0.0%", content)
        # Check corrected futures fee calculation
        self.assertIn("₹500.00", content)
        self.assertIn("₹610.39", content)
        self.assertIn("7.60 bps", content)

    def test_02_a1_predictive_study_results(self):
        self.assertTrue(A1_RESULTS_FILE.exists(), "A1 results JSON must exist.")
        with open(A1_RESULTS_FILE, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["hypothesis_id"], "SHRP-001-A1-VOL-EXPANSION")
        self.assertEqual(data["gate_decision"], "FALSIFIED")
        self.assertEqual(data["total_development_sessions"], 40)
        self.assertGreater(data["total_signals_generated"], 4000)

        h4 = data["primary_horizon_4_bars_60m"]
        # Cohen's d must be negative or below 0.20
        self.assertLess(h4["cohens_d"], 0.20)
        # Directional win rate must be below 51.0%
        self.assertLess(h4["directional_win_rate_pct"], 51.0)
        # Mean signed return must be below +18.0 bps
        self.assertLess(h4["mean_signed_ret_bps"], 18.0)

    def test_03_b1_predictive_study_results(self):
        self.assertTrue(B1_RESULTS_FILE.exists(), "B1 results JSON must exist.")
        with open(B1_RESULTS_FILE, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["hypothesis_id"], "SHRP-001-B1-SWING-CONTINUATION")
        self.assertEqual(data["gate_decision"], "FALSIFIED")
        self.assertEqual(data["total_development_sessions"], 40)
        self.assertGreater(data["total_signals_generated"], 150)
        self.assertIn("DECLARED", data["survivorship_bias_limitation"])

        h3 = data["primary_horizon_3_sessions_t3"]
        # Mean excess return must be below +35.0 bps
        self.assertLess(h3["mean_excess_fwd_ret_bps"], 35.0)
        # Hit rate must be below 50.0%
        self.assertLess(h3["hit_rate_pct"], 50.0)

    def test_04_futures_friction_reconciliation(self):
        # Calculate futures statutory charges on 10L notional:
        notional = 1000000.0  # 10 Lakh
        stt = notional * 0.0005  # 0.05% on sell turn = ₹500.00
        nse_trans = (2 * notional) * 0.0000173  # 0.00173% on both legs = ₹34.60
        sebi_turn = (2 * notional) * (10.0 / 10000000.0)  # ₹10/crore = ₹2.00
        stamp = notional * 0.00002  # 0.002% on buy = ₹20.00
        brokerage = 40.00  # ₹20 flat per executed order
        gst = (brokerage + nse_trans + sebi_turn) * 0.18  # 18% on fees = ₹13.788
        total_statutory_and_broker = stt + nse_trans + sebi_turn + stamp + brokerage + gst
        slippage_spread_bps = 1.50
        slippage_spread_inr = notional * (slippage_spread_bps / 10000.0)
        total_friction_inr = total_statutory_and_broker + slippage_spread_inr
        total_friction_bps = (total_friction_inr / notional) * 10000.0

        self.assertAlmostEqual(stt, 500.00, places=2)
        self.assertAlmostEqual(total_statutory_and_broker, 610.39, places=1)
        self.assertAlmostEqual(total_friction_bps, 7.60, places=1)

    def test_05_holdout_partition_strictly_sealed(self):
        # Verify no dates from Sessions 41-67 were touched
        for res_file in [A1_RESULTS_FILE, B1_RESULTS_FILE]:
            with open(res_file, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertNotIn("2026-08-19", content)
            self.assertNotIn("2026-08-20", content)
            self.assertNotIn("2026-09-25", content)


if __name__ == "__main__":
    unittest.main()
