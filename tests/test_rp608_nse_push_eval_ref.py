#!/usr/bin/env python3
"""P2 Python reference tests."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.rp608_nse_push_eval_ref import (
    TRAJ15_ENTRY_THRESHOLD,
    WATCHLIST_TOP_N,
    write_p2_fixtures,
)

FIXTURE = (
    ROOT
    / "adapters"
    / "chronosentiment"
    / "tests"
    / "fixtures"
    / "nse_kite_full"
    / "p2_push_eval_differential.json"
)


class TestPushEvalRefP2(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        write_p2_fixtures(FIXTURE.parent)
        cls.fix = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_constants(self):
        self.assertAlmostEqual(TRAJ15_ENTRY_THRESHOLD, 0.05)
        self.assertEqual(WATCHLIST_TOP_N, 20)
        self.assertLessEqual(self.fix["watchlist_n"], 20)

    def test_recommend_and_rejects(self):
        by = {c["name"]: c for c in self.fix["recommendation_cases"]}
        self.assertEqual(by["recommend_long"]["eval"]["decision"], "RECOMMEND_LONG")
        self.assertEqual(by["sub_threshold"]["eval"]["decision_code"], "SUB_THRESHOLD_MOMENTUM")
        self.assertEqual(by["cold_start"]["eval"]["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")
        self.assertEqual(by["entry_cutoff"]["eval"]["decision_code"], "INTRADAY_BOUNDARY_EXCLUDED")
        self.assertEqual(by["square_off"]["eval"]["decision_code"], "INTRADAY_BOUNDARY_EXCLUDED")
        self.assertEqual(by["stale_quote"]["eval"]["decision_code"], "QUOTE_FEASIBILITY_FAILURE")

    def test_stale_engine(self):
        eng = self.fix["engine"]
        self.assertEqual(eng["before_disconnect_decision"], "RECOMMEND_LONG")
        self.assertIsNone(eng["during_stale_current"])
        self.assertIsNone(eng["after_recon_current"])
        self.assertTrue(eng["after_live_is_current"])


if __name__ == "__main__":
    unittest.main()
