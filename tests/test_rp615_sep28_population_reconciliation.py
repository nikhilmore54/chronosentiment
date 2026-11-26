#!/usr/bin/env python3
"""
tests/test_rp615_sep28_population_reconciliation.py

Unit test suite verifying the September 28 population reconciliation bridge.
Asserts accounting provenance between the canonical live journal (1,782 trades)
and full-day continuous synthetic replay (2,053 trades).
"""

import json
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
RECON_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_sep28_population_reconciliation.json"
RECON_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_sep28_population_reconciliation_report.md"


class TestRP615Sep28PopulationReconciliation(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not RECON_JSON.exists():
            raise FileNotFoundError(f"Missing reconciliation JSON: {RECON_JSON}")
        with RECON_JSON.open("r", encoding="utf-8") as f:
            cls.data = json.load(f)

    def test_canonical_journal_provenance(self):
        canon = self.data["canonical_journal"]
        self.assertEqual(canon["session_date"], "2026-09-28")
        self.assertEqual(canon["total_trades"], 1782)
        self.assertEqual(canon["first_entry_ist"], "10:20:50")

    def test_full_day_replay_provenance(self):
        replay = self.data["full_day_synthetic_replay"]
        self.assertEqual(replay["session_date"], "2026-09-28")
        self.assertEqual(replay["total_trades"], 2053)
        self.assertEqual(replay["first_entry_ist"], "09:30:15")

    def test_accounting_bridge_delta(self):
        bridge = self.data["accounting_bridge"]
        self.assertEqual(bridge["delta_count"], 271)
        self.assertTrue(RECON_MD.exists())


if __name__ == "__main__":
    unittest.main()
