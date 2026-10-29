#!/usr/bin/env python3
"""
tests/test_rp615_stage2b_reconstruction.py

Unit test suite verifying Stage 2B Historical Reconstruction (Sep 17–25, 2026).
Asserts population integrity, 10-session dataset compilation, and manifest consistency.
"""

import csv
import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
MANIFEST_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2b_reconstruction_manifest.json"
REPORT_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2b_reconstruction_report.md"
RECON_CSV = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2b_reconstructed_opportunities.csv"
EXPANDED_CSV = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage2b_expanded_discovery_dataset.csv"


class TestRP615Stage2BReconstruction(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not MANIFEST_JSON.exists():
            raise FileNotFoundError(f"Missing manifest JSON: {MANIFEST_JSON}")
        with MANIFEST_JSON.open("r", encoding="utf-8") as f:
            cls.manifest = json.load(f)

    def test_reconstructed_population_counts(self):
        census = self.manifest["eligibility_census"]
        self.assertEqual(census["added_discovery_total"], 13480)
        self.assertEqual(census["added_discovery_eligible"], 12982)
        self.assertEqual(census["added_discovery_ineligible"], 498)
        self.assertEqual(census["original_discovery_total"], 5456)
        self.assertEqual(census["expanded_discovery_total"], 18936)

    def test_reconstructed_csv_row_count(self):
        with RECON_CSV.open(encoding="utf-8") as f:
            reader = csv.DictReader(f)
            rows = list(reader)
        self.assertEqual(len(rows), 13480)

    def test_expanded_discovery_csv_row_count(self):
        with EXPANDED_CSV.open(encoding="utf-8") as f:
            reader = csv.DictReader(f)
            rows = list(reader)
        self.assertEqual(len(rows), 18936)
        
        # Verify dates in expanded discovery: strictly Sep 17 to Sep 30 (no Oct 1)
        dates = set(r["session_date"] for r in rows)
        self.assertEqual(len(dates), 10)
        self.assertNotIn("2026-10-01", dates)

    def test_standing_governance(self):
        gov = self.manifest["standing_governance"]
        self.assertEqual(gov["rp605_control"], "100% Live Execution Authority")
        self.assertEqual(gov["rp615_research"], "0% Live Authority (Research-Only)")
        self.assertEqual(gov["benchmark_model_hash"], "212532d3ac12f9c6")
        self.assertTrue(REPORT_MD.exists())


if __name__ == "__main__":
    unittest.main()
