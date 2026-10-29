#!/usr/bin/env python3
"""Prospective V1 Arm C observer: journal only when STARTED; 0.0% authority; packing."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

import scripts.rp608_prospective_v1 as pv1
import scripts.rp608_prospective_v1_observer as obs
from scripts.rp608_prospective_v1 import Market, PROTOCOL_ID


def _synthetic_bars(n: int = 40, *, trend: bool = True) -> list:
    bars = []
    px = 100.0
    for i in range(n):
        if trend:
            px = 100.0 + i * 1.5
        else:
            px = 100.0 + (0.2 if i % 2 == 0 else -0.2)
        bars.append(
            {
                "open": px - 0.1,
                "high": px + 0.5,
                "low": px - 0.5,
                "close": px,
            }
        )
    return bars


class TestObserverGates(unittest.TestCase):
    def test_noop_when_not_started(self):
        with mock.patch.object(obs, "is_market_started", return_value=False):
            out = obs.observe_bar(
                market=Market.CRYPTO,
                completed_bars_including_signal=_synthetic_bars(),
                ask=160.0,
                bid=159.0,
                quote_age_ms=50.0,
                session_date="2026-10-29",
                symbol="BTCUSDT",
                bar_idx=1,
                arm_a_signal=True,
                arm_a_side="LONG",
                reference_price=159.5,
            )
            self.assertIsNone(out)

    def test_journals_under_prospective_ns(self):
        with tempfile.TemporaryDirectory() as td:
            ns = Path(td) / "prospective_v1"
            ns.mkdir()
            with mock.patch.object(obs, "is_market_started", return_value=True), mock.patch.object(
                obs, "load_start_event", return_value={"eligible_session_boundary": "2026-10-29T00:00:00Z"}
            ), mock.patch.object(obs, "PROSPECTIVE_NS", ns), mock.patch.object(
                pv1, "PROSPECTIVE_NS", ns
            ), mock.patch.object(obs, "assert_write_allowed", lambda p: None), mock.patch.object(
                obs, "ensure_namespace", lambda: ns
            ):
                bars = _synthetic_bars(50, trend=True)
                # Force a clear breakout: ask well above recent highs
                ask = bars[-1]["high"] + 50.0
                bid = ask - 0.5
                out = obs.observe_bar(
                    market=Market.CRYPTO,
                    completed_bars_including_signal=bars,
                    ask=ask,
                    bid=bid,
                    quote_age_ms=40.0,
                    session_date="2026-10-29",
                    symbol="BTCUSDT",
                    bar_idx=42,
                    close_time_ms=1,
                    arm_a_signal=True,
                    arm_a_side="LONG",
                    reference_price=bars[-1]["close"],
                )
                self.assertIsNotNone(out)
                self.assertEqual(out["protocol_id"], PROTOCOL_ID)
                self.assertEqual(out["live_order_authority"], 0.0)
                self.assertEqual(out["packing"], "pack_bars_for_breakout_eval")
                self.assertTrue(out["arm_a_frozen"])
                path = obs.opportunity_journal_path(Market.CRYPTO)
                self.assertTrue(str(path).startswith(str(ns)))
                self.assertTrue(path.exists())
                line = path.read_text(encoding="utf-8").strip().splitlines()[-1]
                rec = json.loads(line)
                self.assertEqual(rec["market"], "CRYPTO")
                self.assertIn(rec["disposition_c"], ("taken", "regime_reject", "breakout_reject", "arm_a_flat"))
                # Arm C intersection flag is DIRECTIONAL ∧ TRADE_QUALIFIED (independent of nesting)
                self.assertEqual(
                    rec["arm_c_intersection"],
                    rec["regime"] == "DIRECTIONAL" and rec["lifecycle_state"] == "TRADE_QUALIFIED",
                )
                if rec["take_c"]:
                    self.assertTrue(rec["take_b"])
                    self.assertEqual(rec["lifecycle_state"], "TRADE_QUALIFIED")
                summary = json.loads(obs.summary_path(Market.CRYPTO).read_text(encoding="utf-8"))
                self.assertEqual(summary["live_order_authority"], 0.0)
                self.assertGreaterEqual(summary["n_bars"], 1)

    def test_arm_a_flat_dispositions(self):
        with tempfile.TemporaryDirectory() as td:
            ns = Path(td) / "prospective_v1"
            ns.mkdir()
            with mock.patch.object(obs, "is_market_started", return_value=True), mock.patch.object(
                obs, "load_start_event", return_value={}
            ), mock.patch.object(obs, "PROSPECTIVE_NS", ns), mock.patch.object(
                obs, "assert_write_allowed", lambda p: None
            ), mock.patch.object(obs, "ensure_namespace", lambda: ns):
                out = obs.observe_bar(
                    market=Market.NSE,
                    completed_bars_including_signal=_synthetic_bars(30, trend=False),
                    ask=100.1,
                    bid=100.0,
                    quote_age_ms=20.0,
                    session_date="2026-10-29",
                    symbol="RELIANCE.NS",
                    bar_idx=5,
                    arm_a_signal=False,
                    reference_price=100.05,
                )
                self.assertIsNotNone(out)
                self.assertFalse(out["take_b"])
                self.assertFalse(out["take_c"])
                self.assertEqual(out["disposition_b"], "arm_a_flat")
                self.assertEqual(out["disposition_c"], "arm_a_flat")

    def test_recommendation_tape_records_flats_and_actions(self):
        with tempfile.TemporaryDirectory() as td:
            ns = Path(td) / "prospective_v1"
            ns.mkdir()
            with mock.patch.object(obs, "is_market_started", return_value=True), mock.patch.object(
                obs, "load_start_event", return_value={"eligible_session_boundary": "2026-10-29"}
            ), mock.patch.object(obs, "PROSPECTIVE_NS", ns), mock.patch.object(
                obs, "assert_write_allowed", lambda p: None
            ), mock.patch.object(obs, "ensure_namespace", lambda: ns):
                flat = obs.record_recommendation(
                    market=Market.CRYPTO,
                    symbol="BTCUSDT",
                    decision="NO_TRADE",
                    secondary_decision="NO_TRADE",
                    session_date="2026-10-29",
                    bar_idx=1,
                    is_actionable=False,
                )
                hit = obs.record_recommendation(
                    market=Market.CRYPTO,
                    symbol="BTCUSDT",
                    decision="SHORT",
                    secondary_decision="LONG",
                    session_date="2026-10-29",
                    bar_idx=2,
                    is_actionable=True,
                )
                self.assertIsNotNone(flat)
                self.assertIsNotNone(hit)
                path = obs.recommendation_journal_path(Market.CRYPTO)
                lines = path.read_text(encoding="utf-8").strip().splitlines()
                self.assertEqual(len(lines), 2)
                summary = json.loads(obs.recommendation_summary_path(Market.CRYPTO).read_text(encoding="utf-8"))
                self.assertEqual(summary["n_recommendations"], 2)
                self.assertEqual(summary["n_actionable"], 1)
                self.assertEqual(summary["live_order_authority"], 0.0)


if __name__ == "__main__":
    unittest.main()
