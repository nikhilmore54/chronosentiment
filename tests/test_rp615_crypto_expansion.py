#!/usr/bin/env python3
"""
tests/test_rp615_crypto_expansion.py

Unit tests for RP-615-Crypto Track B:
Multi-Cohort Architecture, Expansion Instrument Audit, and Ledger Isolation.
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

EXPANSION_REPORT_MD = LOG_DIR / "rp615_crypto_expansion_audit_report.md"
EXPANSION_MANIFEST_JSON = LOG_DIR / "rp615_crypto_expansion_manifest.json"
EXPANSION_JOURNAL_JSONL = LOG_DIR / "rp615_crypto_expansion_trade_journal.jsonl"
EXPANSION_CARDS_JSONL = LOG_DIR / "rp615_crypto_expansion_decision_cards.jsonl"
NSE_FROZEN_LEDGER = LOG_DIR / "rp615_tier4_frozen_ledger.csv"

from scripts.run_rp615_crypto_expansion_audit import sha256_file


class TestRP615CryptoExpansion(unittest.TestCase):

    def test_01_expansion_artifacts_exist(self):
        """Verify Track B expansion audit artifacts exist and are non-empty."""
        for path in [EXPANSION_REPORT_MD, EXPANSION_MANIFEST_JSON]:
            self.assertTrue(path.exists(), f"Missing artifact: {path.name}")
            self.assertGreater(path.stat().st_size, 0, f"Empty artifact: {path.name}")

    def test_02_expansion_manifest_structure(self):
        """Verify Track B manifest structure and batch admissions."""
        with EXPANSION_MANIFEST_JSON.open() as f:
            manifest = json.load(f)

        self.assertEqual(manifest["track"], "RP-615-Crypto Track B (Expansion Research)")
        self.assertIn("SOLUSDT", manifest["batch_a_admitted"])
        self.assertIn("SOLUSDT", manifest["results"])
        self.assertIn("BNBUSDT", manifest["results"])
        self.assertIn("XRPUSDT", manifest["results"])

    def test_03_cohort_ledger_isolation(self):
        """Verify Track B expansion records are logged to dedicated files with cohort tag."""
        if EXPANSION_JOURNAL_JSONL.exists():
            with EXPANSION_JOURNAL_JSONL.open() as f:
                lines = [json.loads(line) for line in f if line.strip()]
            for item in lines:
                self.assertEqual(item["cohort"], "TRACK_B_EXPANSION")
                self.assertIn(item["symbol"], ["SOLUSDT", "BNBUSDT", "XRPUSDT", "ADAUSDT", "DOGEUSDT"])

    def test_04_nse_cross_track_isolation(self):
        """Verify zero contamination on NSE frozen ledger."""
        with EXPANSION_MANIFEST_JSON.open() as f:
            manifest = json.load(f)

        actual_nse_hash = sha256_file(NSE_FROZEN_LEDGER)
        self.assertEqual(manifest["checksums"]["nse_frozen_ledger"], actual_nse_hash)


if __name__ == "__main__":
    unittest.main()
