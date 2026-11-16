#!/usr/bin/env python3
"""Tests for Coralys input-context gate (adapter-side)."""

from __future__ import annotations

import json
import os
import tempfile
import unittest
from datetime import datetime
from pathlib import Path

from scripts.rp608_nse_coralys_input_context import (
    CoralysInputContextStore,
    IST,
    gate_enabled,
    session_phase_ist,
)


class TestSessionPhase(unittest.TestCase):
    def test_phases(self):
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 9, 15, tzinfo=IST)), "OPEN")
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 9, 30, tzinfo=IST)), "EARLY")
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 10, 45, tzinfo=IST)), "MID")
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 13, 0, tzinfo=IST)), "LATE")


class TestCoralysInputGate(unittest.TestCase):
    def setUp(self):
        self._tmpdir = tempfile.TemporaryDirectory()
        self.root = Path(self._tmpdir.name)
        (self.root / "test_run").mkdir()
        day = "20261116"
        snap = self.root / "test_run" / f"preopen_discovery_snapshot_{day}T091430.jsonl"
        rows = [
            {
                "symbol": "AAA.NS",
                "opening_gap_bps": 40.0,
                "imbalance_ratio": 0.1,
                "indicative_source": "TEST",
                "previous_close": 100.0,
            },
            {
                "symbol": "BBB.NS",
                "opening_gap_bps": -80.0,
                "imbalance_ratio": -0.5,
                "indicative_source": "TEST",
                "previous_close": 100.0,
            },
            {
                "symbol": "CCC.NS",
                "opening_gap_bps": -30.0,
                "imbalance_ratio": -0.1,
                "indicative_source": "TEST",
                "previous_close": 100.0,
            },
        ]
        snap.write_text("\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8")
        recs = {
            "next_session_date": "2026-11-16",
            "from_session_date": "2026-11-15",
            "n_recommendations": 1,
            "recommendations": [{"symbol": "AAA.NS", "side": "LONG", "traj15_eod": 0.2}],
        }
        (self.root / "rp608_nse_next_session_recommendations.json").write_text(
            json.dumps(recs), encoding="utf-8"
        )
        self.store = CoralysInputContextStore(self.root)
        os.environ["RP608_NSE_CORALYS_INPUT_GATE"] = "1"

    def tearDown(self):
        self._tmpdir.cleanup()
        os.environ.pop("RP608_NSE_CORALYS_INPUT_GATE", None)

    def test_gate_enabled_default(self):
        os.environ["RP608_NSE_CORALYS_INPUT_GATE"] = "1"
        self.assertTrue(gate_enabled())
        os.environ["RP608_NSE_CORALYS_INPUT_GATE"] = "0"
        self.assertFalse(gate_enabled())

    def test_open_aligned_pass(self):
        ctx = self.store.evaluate(
            symbol="AAA.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 9, 15, tzinfo=IST),
            traj15=0.06,
            signal_long=True,
        )
        self.assertTrue(ctx["allow_long_entry"])
        self.assertEqual(ctx["session_phase"], "OPEN")
        self.assertIn("PASS_OPEN", ctx["standaside_reason"])

    def test_open_gap_down_standaside(self):
        ctx = self.store.evaluate(
            symbol="BBB.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 9, 15, tzinfo=IST),
            traj15=0.06,
            signal_long=True,
        )
        self.assertFalse(ctx["allow_long_entry"])

    def test_mid_opposed_weak_traj_standaside(self):
        ctx = self.store.evaluate(
            symbol="CCC.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 10, 45, tzinfo=IST),
            traj15=0.05,
            signal_long=True,
        )
        self.assertFalse(ctx["allow_long_entry"])
        self.assertEqual(ctx["session_phase"], "MID")

    def test_mid_opposed_strong_traj_pass(self):
        # gap<-20 + not on prior watch requires traj15 >= 0.10
        ctx = self.store.evaluate(
            symbol="CCC.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 10, 45, tzinfo=IST),
            traj15=0.10,
            signal_long=True,
        )
        self.assertTrue(ctx["allow_long_entry"])

    def test_watch_resolves_from_parent_dir(self):
        # Mimic daemon output_dir = .../nse/test_run with watch in .../nse/
        nested = self.root / "nse" / "test_run"
        nested.mkdir(parents=True)
        watch_parent = self.root / "nse"
        recs = {
            "next_session_date": "2026-11-16",
            "from_session_date": "2026-11-15",
            "n_recommendations": 1,
            "recommendations": [{"symbol": "DDD.NS"}],
        }
        (watch_parent / "rp608_nse_next_session_recommendations.json").write_text(
            json.dumps(recs), encoding="utf-8"
        )
        store = CoralysInputContextStore(nested)
        watch = store.load_watchlist_for_session("2026-11-16")
        self.assertIn("DDD.NS", watch)
        self.assertEqual(store._watch_meta.get("status"), "OK")


if __name__ == "__main__":
    unittest.main()
