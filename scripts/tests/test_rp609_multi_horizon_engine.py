#!/usr/bin/env python3
"""Unit tests for RP-609 multi-horizon observation engine."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import scripts.rp609_indian_retail_horizons as horizons
from scripts.rp609_indian_retail_horizons import estimate_cnc_friction_inr
from scripts.rp609_multi_horizon_engine import MultiHorizonEngine


class Rp609EngineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.journals = self.root / "journals"
        self.journals.mkdir(parents=True)
        self.nse = self.root / "nse"
        self.nse.mkdir()
        # Redirect artifact paths
        horizons.ARTIFACT_ROOT = self.root
        horizons.JOURNALS = self.journals
        import scripts.rp609_multi_horizon_engine as eng_mod

        eng_mod.JOURNALS = self.journals
        eng_mod.ARTIFACT_ROOT = self.root

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_cnc_friction_positive(self) -> None:
        fr = estimate_cnc_friction_inr(buy_price=1000.0, sell_price=1010.0, quantity=10)
        self.assertGreater(fr["total_friction"], 0.0)
        self.assertIn("stt", fr)

    def test_swing_target_exit(self) -> None:
        eng = MultiHorizonEngine(nse_dir=self.nse)
        t = eng.register_swing_candidate(
            symbol="AAA.NS",
            thesis_tag="t",
            reference_price=100.0,
            invalidation_price=90.0,
            target_price=110.0,
            quantity=10,
            max_sessions=5,
            session_date="2026-10-01",
        )
        tid = t["trade_id"]
        closed = eng.mark_swing(tid, price=111.0, session_date="2026-10-02")
        self.assertEqual(closed["status"], "CLOSED")
        self.assertEqual(closed["exit_reason"], "TARGET")
        self.assertGreater(closed["net_pnl_inr"], 0.0)
        self.assertEqual(len(eng.swing["open"]), 0)

    def test_swing_stop_and_max_sessions(self) -> None:
        eng = MultiHorizonEngine(nse_dir=self.nse)
        t = eng.register_swing_candidate(
            symbol="BBB.NS",
            thesis_tag="t",
            reference_price=100.0,
            invalidation_price=95.0,
            target_price=120.0,
            quantity=5,
            max_sessions=2,
            session_date="2026-10-01",
        )
        tid = t["trade_id"]
        stopped = eng.mark_swing(tid, price=94.0, session_date="2026-10-02")
        self.assertEqual(stopped["exit_reason"], "INVALIDATION")

        t2 = eng.register_swing_candidate(
            symbol="CCC.NS",
            thesis_tag="t",
            reference_price=100.0,
            invalidation_price=80.0,
            target_price=150.0,
            quantity=5,
            max_sessions=2,
            session_date="2026-10-01",
        )
        tid2 = t2["trade_id"]
        eng.mark_swing(tid2, price=101.0, session_date="2026-10-02")
        closed = eng.mark_swing(tid2, price=102.0, session_date="2026-10-03")
        self.assertEqual(closed["exit_reason"], "MAX_SESSIONS")
        self.assertEqual(closed["sessions_held"], 2)

    def test_portfolio_caps_and_tranches(self) -> None:
        eng = MultiHorizonEngine(nse_dir=self.nse)
        with self.assertRaises(ValueError):
            eng.register_portfolio_plan(
                plan_id="bad",
                total_capital_inr=100000,
                legs=[{"symbol": "X.NS", "sector": "A", "target_weight_pct": 20.0}],
                max_name_pct=10.0,
            )
        plan = eng.register_portfolio_plan(
            plan_id="ok",
            total_capital_inr=100000,
            n_tranches=2,
            legs=[
                {"symbol": "X.NS", "sector": "A", "target_weight_pct": 8.0},
                {"symbol": "Y.NS", "sector": "B", "target_weight_pct": 8.0},
            ],
            session_date="2026-10-01",
        )
        prices = {"X.NS": 100.0, "Y.NS": 50.0}
        eng.advance_portfolio_tranche("ok", prices=prices, session_date="2026-10-01")
        self.assertEqual(eng.portfolio["plans"]["ok"]["tranches_filled"], 1)
        # same session should not double-fill
        eng.advance_portfolio_tranche("ok", prices=prices, session_date="2026-10-01")
        self.assertEqual(eng.portfolio["plans"]["ok"]["tranches_filled"], 1)
        eng.advance_portfolio_tranche("ok", prices=prices, session_date="2026-10-02")
        self.assertEqual(eng.portfolio["plans"]["ok"]["status"], "BUILT")
        self.assertEqual(eng.portfolio["plans"]["ok"]["tranches_filled"], 2)

    def test_long_term_review_due(self) -> None:
        eng = MultiHorizonEngine(nse_dir=self.nse)
        h = eng.open_long_term(
            symbol="ZZ.NS",
            thesis="observe",
            thesis_version="v1",
            entry_price=200.0,
            quantity=3,
            review_cadence_sessions=2,
            session_date="2026-10-01",
        )
        hid = h["hold_id"]
        eng.mark_long_term(hid, price=205.0, session_date="2026-10-02")
        marked = eng.mark_long_term(hid, price=210.0, session_date="2026-10-03")
        self.assertEqual(marked["status"], "REVIEW_DUE")
        cont = eng.complete_long_term_review(hid, decision="HOLD_CONTINUE")
        self.assertEqual(cont["status"], "HOLDING")

    def test_eod_uses_price_map(self) -> None:
        eng = MultiHorizonEngine(nse_dir=self.nse)
        eng.register_swing_candidate(
            symbol="EEE.NS",
            thesis_tag="t",
            reference_price=50.0,
            invalidation_price=40.0,
            target_price=80.0,
            quantity=10,
            max_sessions=10,
            session_date="2026-10-01",
        )
        summary = eng.run_eod(
            session_date="2026-10-02",
            prices={"EEE.NS": 55.0},
        )
        self.assertEqual(summary["swing_marked"], 1)
        open_pos = list(eng.swing["open"].values())[0]
        self.assertEqual(open_pos["last_mark_price"], 55.0)
        self.assertEqual(open_pos["sessions_held"], 1)


if __name__ == "__main__":
    unittest.main()
