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
    classify_prior_watch_status,
    gate_enabled,
    is_preopen_turbulent,
    session_phase_ist,
)


class TestSessionPhase(unittest.TestCase):
    def test_phases(self):
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 9, 15, tzinfo=IST)), "OPEN")
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 9, 30, tzinfo=IST)), "EARLY")
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 10, 45, tzinfo=IST)), "MID")
        self.assertEqual(session_phase_ist(datetime(2026, 11, 16, 13, 0, tzinfo=IST)), "LATE")


class TestPriorStaleLadder(unittest.TestCase):
    def test_ok(self):
        self.assertEqual(classify_prior_watch_status("2026-11-16", "2026-11-16"), "OK")

    def test_stale_recent_weekend_and_holiday_buffer(self):
        # Fri→Mon and up to 6 calendar days remain STALE_RECENT (unseen holidays).
        self.assertEqual(classify_prior_watch_status("2026-11-16", "2026-11-13"), "STALE_RECENT")
        self.assertEqual(classify_prior_watch_status("2026-11-16", "2026-11-10"), "STALE_RECENT")
        self.assertEqual(classify_prior_watch_status("2026-11-16", "2026-11-15"), "STALE_RECENT")

    def test_stale_expired(self):
        self.assertEqual(classify_prior_watch_status("2026-11-16", "2026-11-09"), "STALE_EXPIRED")

    def test_future_mismatch(self):
        self.assertEqual(classify_prior_watch_status("2026-11-16", "2026-11-17"), "MISMATCH_FUTURE")

    def test_missing(self):
        self.assertEqual(classify_prior_watch_status("2026-11-16", None), "MISSING")


class TestPreopenTurbulence(unittest.TestCase):
    def test_large_gap(self):
        ok, tag = is_preopen_turbulent(-80.0, 0.0)
        self.assertTrue(ok)
        self.assertEqual(tag, "LARGE_ABS_GAP")

    def test_calm(self):
        ok, tag = is_preopen_turbulent(25.0, 0.1)
        self.assertFalse(ok)
        self.assertEqual(tag, "")


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
            {
                "symbol": "DDD.NS",
                "opening_gap_bps": 55.0,
                "imbalance_ratio": 0.50,
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
        self.assertEqual(ctx["policy_id"], "CORALYS_INPUT_GATE_V3")

    def test_open_gap_down_standaside(self):
        ctx = self.store.evaluate(
            symbol="BBB.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 9, 15, tzinfo=IST),
            traj15=0.06,
            signal_long=True,
        )
        self.assertFalse(ctx["allow_long_entry"])

    def test_open_turbulence_weak_traj_standaside(self):
        ctx = self.store.evaluate(
            symbol="DDD.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 9, 20, tzinfo=IST),
            traj15=0.06,
            signal_long=True,
        )
        self.assertFalse(ctx["allow_long_entry"])
        self.assertTrue(str(ctx["standaside_reason"]).startswith("STANDASIDE_PREOPEN_TURBULENCE_"))

    def test_open_turbulence_strong_traj_may_pass_alignment(self):
        # Turbulent but strong traj — still needs OPEN alignment; DDD gap+imb not aligned for longs.
        ctx = self.store.evaluate(
            symbol="DDD.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 9, 15, tzinfo=IST),
            traj15=0.15,
            signal_long=True,
        )
        # gap 55 aligned (>=20), imb 0.5 ok; not on watch → OPEN_NO_PRIOR_WATCH
        self.assertFalse(ctx["allow_long_entry"])
        self.assertEqual(ctx["standaside_reason"], "STANDASIDE_OPEN_NO_PRIOR_WATCH")

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
        self.assertEqual(ctx["standaside_reason"], "STANDASIDE_CONTINUOUS_WEAK_TRAJ")

    def test_mid_not_on_watch_needs_strong_traj(self):
        ctx = self.store.evaluate(
            symbol="CCC.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 10, 45, tzinfo=IST),
            traj15=0.10,
            signal_long=True,
        )
        self.assertFalse(ctx["allow_long_entry"])
        self.assertEqual(ctx["standaside_reason"], "STANDASIDE_CONTINUOUS_NO_PRIOR_WEAK_TRAJ")

    def test_mid_opposed_strong_traj_pass(self):
        ctx = self.store.evaluate(
            symbol="CCC.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 10, 45, tzinfo=IST),
            traj15=0.12,
            signal_long=True,
        )
        self.assertTrue(ctx["allow_long_entry"])

    def test_mid_positive_gap_on_watch_floor(self):
        ctx_fail = self.store.evaluate(
            symbol="AAA.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 10, 45, tzinfo=IST),
            traj15=0.06,
            signal_long=True,
        )
        self.assertFalse(ctx_fail["allow_long_entry"])
        ctx_pass = self.store.evaluate(
            symbol="AAA.NS",
            session_date="2026-11-16",
            decision_ts_ist=datetime(2026, 11, 16, 10, 45, tzinfo=IST),
            traj15=0.08,
            signal_long=True,
        )
        self.assertTrue(ctx_pass["allow_long_entry"])

    def test_stale_recent_loads_symbols(self):
        recs = {
            "next_session_date": "2026-11-15",
            "from_session_date": "2026-11-14",
            "n_recommendations": 1,
            "recommendations": [{"symbol": "EEE.NS"}],
        }
        (self.root / "rp608_nse_next_session_recommendations.json").write_text(
            json.dumps(recs), encoding="utf-8"
        )
        store = CoralysInputContextStore(self.root)
        watch = store.load_watchlist_for_session("2026-11-16")
        self.assertIn("EEE.NS", watch)
        self.assertEqual(store._watch_meta.get("status"), "STALE_RECENT")

    def test_stale_expired_does_not_load_symbols(self):
        recs = {
            "next_session_date": "2026-11-09",
            "from_session_date": "2026-11-08",
            "n_recommendations": 1,
            "recommendations": [{"symbol": "FFF.NS"}],
        }
        (self.root / "rp608_nse_next_session_recommendations.json").write_text(
            json.dumps(recs), encoding="utf-8"
        )
        store = CoralysInputContextStore(self.root)
        watch = store.load_watchlist_for_session("2026-11-16")
        self.assertNotIn("FFF.NS", watch)
        self.assertEqual(store._watch_meta.get("status"), "STALE_EXPIRED")

    def test_watch_resolves_from_parent_dir(self):
        nested = self.root / "nse" / "test_run"
        nested.mkdir(parents=True)
        watch_parent = self.root / "nse"
        recs = {
            "next_session_date": "2026-11-16",
            "from_session_date": "2026-11-15",
            "n_recommendations": 1,
            "recommendations": [{"symbol": "GGG.NS"}],
        }
        (watch_parent / "rp608_nse_next_session_recommendations.json").write_text(
            json.dumps(recs), encoding="utf-8"
        )
        store = CoralysInputContextStore(nested)
        watch = store.load_watchlist_for_session("2026-11-16")
        self.assertIn("GGG.NS", watch)
        self.assertEqual(store._watch_meta.get("status"), "OK")


if __name__ == "__main__":
    unittest.main()
