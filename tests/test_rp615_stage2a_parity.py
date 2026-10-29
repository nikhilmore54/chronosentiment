#!/usr/bin/env python3
"""
tests/test_rp615_stage2a_parity.py

Unit test suite verifying Stage 2A Canonical Reconstruction Parity.
Asserts 100% bijective match of all 7,264 baseline opportunities.
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
PARITY_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2a_parity_audit.json"
PARITY_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2a_parity_report.md"


class TestRP615Stage2AParity(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not PARITY_JSON.exists():
            raise FileNotFoundError(f"Missing parity JSON: {PARITY_JSON}")
        with PARITY_JSON.open("r", encoding="utf-8") as f:
            cls.data = json.load(f)

    def test_population_counts_and_zero_mismatches(self):
        pop = self.data["population_counts"]
        self.assertEqual(pop["canonical_journal_total"], 7264)
        self.assertEqual(pop["canonical_predictive_total"], 7264)
        self.assertEqual(pop["perfect_matches"], 7264)
        self.assertEqual(pop["missing_keys_count"], 0)
        self.assertEqual(pop["orphaned_keys_count"], 0)
        self.assertEqual(pop["field_mismatches_count"], 0)
        self.assertTrue(pop["parity_gate_passed"])

    def test_session_breakdown(self):
        sessions = self.data["session_breakdown"]
        self.assertEqual(len(sessions), 4)
        total_opps = sum(s["total_opportunities"] for s in sessions.values())
        total_eligible = sum(s["eligible_opportunities"] for s in sessions.values())
        self.assertEqual(total_opps, 7264)
        self.assertEqual(total_eligible, 6719)

    def test_as_of_rule_documentation(self):
        rule = self.data["as_of_availability_rule"]
        self.assertIn("t_cutoff", rule["feature_cutoff_formula"])
        self.assertTrue(PARITY_MD.exists())


if __name__ == "__main__":
    unittest.main()
