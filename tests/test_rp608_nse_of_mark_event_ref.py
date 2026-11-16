#!/usr/bin/env python3
"""P1 Python reference — OF ±0.30 + event/depth linkage + stale suppression."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.rp608_nse_of_mark_event_ref import write_p1_fixtures
from scripts.rp608_order_flow_mark import ORDER_FLOW_IMBALANCE_THRESHOLD

FIXTURE = (
    ROOT
    / "adapters"
    / "chronosentiment"
    / "tests"
    / "fixtures"
    / "nse_kite_full"
    / "p1_of_depth_differential.json"
)


class TestNseOfMarkEventRefP1(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        write_p1_fixtures(FIXTURE.parent)
        cls.fix = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_threshold_frozen(self):
        self.assertAlmostEqual(ORDER_FLOW_IMBALANCE_THRESHOLD, 0.30)
        self.assertAlmostEqual(self.fix["threshold"], 0.30)

    def test_boundary_cases(self):
        by_name = {c["name"]: c for c in self.fix["classify_cases"]}
        self.assertTrue(by_name["long_boundary_030"]["classify"]["order_flow_supported"])
        self.assertFalse(by_name["long_below"]["classify"]["order_flow_supported"])
        self.assertTrue(by_name["short_boundary"]["classify"]["order_flow_supported"])
        self.assertTrue(by_name["exit_from_long"]["exit"] if False else by_name["short_strong"]["exit_from_long"]["exit"])

    def test_stale_suppresses_current_of(self):
        eng = self.fix["engine"]
        self.assertTrue(eng["before_disconnect_has_current"])
        self.assertIsNone(eng["during_stale_current"])
        self.assertFalse(eng["while_stale_of_is_current"])
        self.assertEqual(eng["while_stale_suppressed_reason"], "disconnect")
        self.assertTrue(eng["after_recon_current_cleared"])
        self.assertTrue(eng["after_live_has_current"])

    def test_linkage_identities(self):
        for row in self.fix["engine"]["linked"]:
            self.assertEqual(row["ingest_seq"], row["depth_ingest_seq"])
            self.assertEqual(row["content_fp"], row["depth_content_fp"])
            self.assertEqual(row["ingest_seq"], row["of_ingest_seq"])
            self.assertEqual(row["content_fp"], row["of_content_fp"])
            self.assertEqual(row["depth_levels_buy"], 5)
            self.assertEqual(row["depth_levels_sell"], 5)


if __name__ == "__main__":
    unittest.main()
