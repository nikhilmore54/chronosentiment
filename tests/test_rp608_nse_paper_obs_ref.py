#!/usr/bin/env python3
"""P3 Python reference — paper lifecycle observations (no ledger write)."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.rp608_nse_paper_obs_ref import (
    OF_PAPER_MIN_HOLD_MS,
    OF_PAPER_OPPOSITE_PERSIST_MS,
    write_p3_fixtures,
)

FIXTURE = (
    ROOT
    / "adapters"
    / "chronosentiment"
    / "tests"
    / "fixtures"
    / "nse_kite_full"
    / "p3_paper_obs_differential.json"
)


class TestPaperObsRefP3(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        write_p3_fixtures(FIXTURE.parent)
        cls.fix = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_constants(self):
        self.assertEqual(OF_PAPER_MIN_HOLD_MS, 300_000)
        self.assertEqual(OF_PAPER_OPPOSITE_PERSIST_MS, 30_000)

    def test_tick_actions(self):
        acts = [o["action"] for o in self.fix["tick_sequence"]]
        self.assertEqual(
            acts,
            [
                "hold",
                "close_suppressed",
                "close_suppressed",
                "close_opposite_side_order_flow",
            ],
        )
        self.assertEqual(self.fix["tick_sequence"][3]["ledger_writer"], "python_authoritative")

    def test_stale_and_governance(self):
        self.assertEqual(self.fix["stale_obs"]["action"], "suppressed_stale")
        self.assertFalse(self.fix["stale_obs"]["is_current"])
        self.assertFalse(self.fix["governance"]["dual_write"])
        self.assertFalse(self.fix["governance"]["cutover"])
        self.assertEqual(self.fix["governance"]["authority_pct"], 0.0)


if __name__ == "__main__":
    unittest.main()
