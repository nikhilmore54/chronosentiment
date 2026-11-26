#!/usr/bin/env python3
"""
tests/test_rp615_crypto_smoke_test.py

Unit test suite for RP-615-Crypto Stage C0.1 Infrastructure Smoke-Test.
Tests:
  1. Full execution of smoke test runner (run_crypto_smoke_test).
  2. Verification of machine-readable results JSON and report markdown.
  3. Strict zero-contamination assertion against NSE ledgers.
  4. State tracker snapshot round-trip parity.
  5. DES tie-breaking priority.
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

from scripts.run_rp615_crypto_smoke_test import (
    CryptoDESStateTracker,
    run_crypto_smoke_test,
    sha256_file,
)


class TestRP615CryptoSmokeTest(unittest.TestCase):

    def test_crypto_smoke_test_execution_pass(self):
        """
        Executes the full crypto smoke-test harness and asserts 100% PASS status.
        """
        results = run_crypto_smoke_test()
        self.assertEqual(results["status"], "PASS")

        # Verify output files exist
        results_json = LOG_DIR / "rp615_crypto_smoke_test_results.json"
        manifest_json = LOG_DIR / "rp615_crypto_smoke_test_manifest.json"
        report_md = LOG_DIR / "rp615_crypto_smoke_test_report.md"

        self.assertTrue(results_json.exists(), "Results JSON must exist")
        self.assertTrue(manifest_json.exists(), "Manifest JSON must exist")
        self.assertTrue(report_md.exists(), "Report markdown must exist")

        with results_json.open(encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["status"], "PASS")
        self.assertEqual(data["checks"]["feed_integrity"]["status"], "PASS")
        self.assertEqual(data["checks"]["bar_geometry"]["status"], "PASS")
        self.assertEqual(data["checks"]["point_in_time_features"]["status"], "PASS")
        self.assertEqual(data["checks"]["des_tie_breaking"]["status"], "PASS")
        self.assertEqual(data["checks"]["state_management"]["status"], "PASS")
        self.assertEqual(data["isolation_guard"]["status"], "PASS")
        self.assertEqual(data["isolation_guard"]["bytes_modified_on_nse"], 0)

    def test_fault_injection_suite_results(self):
        """
        Asserts that all 4 fault-injection cases pass.
        """
        results_json = LOG_DIR / "rp615_crypto_smoke_test_results.json"
        with results_json.open(encoding="utf-8") as f:
            data = json.load(f)

        fi = data["fault_injections"]
        self.assertEqual(fi["case1_gap_detection"], "PASS")
        self.assertEqual(fi["case2_out_of_order_sorting"], "PASS")
        self.assertEqual(fi["case3_duplicate_rejection"], "PASS")
        self.assertEqual(fi["case4_state_snapshot_recovery"], "PASS")


if __name__ == "__main__":
    unittest.main()
