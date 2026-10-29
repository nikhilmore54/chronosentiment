#!/usr/bin/env python3
"""
tests/test_rp615_crypto_deferred_live.py

Unit tests for RP-615-Crypto Tier 2:
Deterministic Deferred-Live Replay Engine & Systems Invariants.
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

OUT_RESULTS_JSON = LOG_DIR / "rp615_crypto_deferred_live_results.json"
OUT_MANIFEST_JSON = LOG_DIR / "rp615_crypto_deferred_live_manifest.json"
OUT_REPORT_MD = LOG_DIR / "rp615_crypto_deferred_live_report.md"
NSE_FROZEN_LEDGER = LOG_DIR / "rp615_tier4_frozen_ledger.csv"

from scripts.run_rp615_crypto_deferred_live_replayer import (
    DeferredLiveReplayer,
    sha256_file,
)


class TestRP615CryptoDeferredLive(unittest.TestCase):

    def setUp(self):
        self.replayer = DeferredLiveReplayer(seed=42001)

    def test_01_artifacts_exist(self):
        """Verify all Tier 2 artifacts are generated and non-empty."""
        for path in [OUT_RESULTS_JSON, OUT_MANIFEST_JSON, OUT_REPORT_MD]:
            self.assertTrue(path.exists(), f"Missing artifact: {path.name}")
            self.assertGreater(path.stat().st_size, 0, f"Empty artifact: {path.name}")

    def test_02_deterministic_replay_reproducibility(self):
        """Verify identical seed produces exact same tick and WAL output."""
        rep1 = DeferredLiveReplayer(seed=12345)
        res1 = rep1.run_replay(target_ticks=1000)

        rep2 = DeferredLiveReplayer(seed=12345)
        res2 = rep2.run_replay(target_ticks=1000)

        self.assertEqual(res1["total_ticks_processed"], res2["total_ticks_processed"])
        self.assertEqual(res1["wal_entries_committed"], res2["wal_entries_committed"])
        self.assertEqual(res1["invariant_violations_count"], 0)

    def test_03_timestamp_monotonicity_invariants(self):
        """Verify no causal timeline inversions occur during replay."""
        with OUT_RESULTS_JSON.open() as f:
            res = json.load(f)

        self.assertGreaterEqual(res["total_ticks_processed"], 100000)
        self.assertEqual(res["invariant_violations_count"], 0)
        self.assertTrue(res["wal_recovery_parity"])
        self.assertEqual(res["status"], "PASS")

    def test_04_cross_track_isolation(self):
        """Verify zero contamination on NSE frozen ledger."""
        with OUT_MANIFEST_JSON.open() as f:
            manifest = json.load(f)

        actual_nse_hash = sha256_file(NSE_FROZEN_LEDGER)
        self.assertEqual(manifest["checksums"]["nse_frozen_ledger"], actual_nse_hash)


if __name__ == "__main__":
    unittest.main()
