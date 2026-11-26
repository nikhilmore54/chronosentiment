#!/usr/bin/env python3
"""
tests/test_rp615_stage1_ohlcv_audit.py

Unit tests verifying the Stage 1 OHLCV Data-Quality & Completeness Audit.
Asserts schema compliance, zero duplicates, price validity, and exact population totals.
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
AUDIT_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage1_ohlcv_data_audit.json"
AUDIT_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage1_ohlcv_data_audit.md"
INPUT_CSV = WORKSPACE / "datasets" / "rp615_ohlcv_since_20260901.csv"


class TestRP615Stage1OHLCVAudit(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not AUDIT_JSON.exists():
            raise FileNotFoundError(f"Audit JSON missing: {AUDIT_JSON}")
        with AUDIT_JSON.open("r", encoding="utf-8") as f:
            cls.audit = json.load(f)

    def test_gate1_schema_and_nulls(self):
        """Verify schema match and zero null values."""
        schema = self.audit["schema_validation"]
        self.assertTrue(schema["schema_match"])
        for col, null_count in schema["null_counts"].items():
            self.assertEqual(null_count, 0, f"Column {col} has {null_count} nulls")

    def test_gate2_deduplication_and_invariants(self):
        """Verify zero duplicates and valid OHLCV price/volume physics."""
        checks = self.audit["integrity_checks"]
        self.assertEqual(checks["duplicates_count"], 0)
        self.assertEqual(checks["invalid_high_low"], 0)
        self.assertEqual(checks["invalid_high_open_close"], 0)
        self.assertEqual(checks["invalid_low_open_close"], 0)
        self.assertEqual(checks["non_positive_prices"], 0)
        self.assertEqual(checks["negative_volumes"], 0)

    def test_gate3_exact_population_reconciliation(self):
        """Verify the arithmetic reconciliation across all 11 sessions."""
        recon = self.audit["reconciliation"]
        self.assertEqual(recon["added_discovery_bars_sep17_25"], 287967)
        self.assertEqual(recon["original_discovery_bars_sep28_30"], 128849)
        self.assertEqual(recon["expanded_discovery_total_bars_sep17_30"], 416816)
        self.assertEqual(recon["diagnostic_holdout_bars_oct01"], 42995)
        self.assertEqual(recon["all_sessions_total_bars"], 459811)
        self.assertTrue(recon["reconciliation_exact"])

    def test_gate4_session_and_symbol_coverage(self):
        """Verify session count is 11 and symbol universe is 119."""
        self.assertEqual(self.audit["total_sessions"], 11)
        self.assertEqual(self.audit["total_unique_symbols"], 119)
        self.assertTrue(AUDIT_MD.exists())


if __name__ == "__main__":
    unittest.main()
