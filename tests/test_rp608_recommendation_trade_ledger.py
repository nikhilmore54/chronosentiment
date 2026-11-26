#!/usr/bin/env python3
"""Tests for CRYPTO recommendation ↔ trade-plan join ledger."""

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
import scripts.rp608_recommendation_trade_ledger as ledger
from scripts.rp608_prospective_v1 import Market


_START_EV = {
    "state": "STARTED",
    "start_event_utc": "2026-10-28T22:30:01+00:00",
    "eligible_session_boundary": "2026-10-29",
    "eligible_session_open_utc": "2026-10-29T00:00:00+00:00",
}


class TestRecommendationTradeLedger(unittest.TestCase):
    def test_resolve_source_event_time_prefers_close_ms(self):
        out = ledger.resolve_source_event_time(
            close_time_ms=1_793_232_899_999,
            wall_utc=__import__("datetime").datetime(2026, 10, 29, 3, 49, 17, tzinfo=__import__("datetime").timezone.utc),
        )
        self.assertEqual(out["recorded_time_source"], "close_time_ms")
        self.assertTrue(out["recorded_utc"].startswith("2026-10-29T00:14:59"))
        self.assertNotEqual(out["recorded_utc"], out["writer_wall_utc"])

    def test_classify_trade_link(self):
        self.assertEqual(
            ledger.classify_trade_link(decision="NO_TRADE", bar_idx=1, plan=None),
            "NO_TRADE",
        )
        self.assertEqual(
            ledger.classify_trade_link(
                decision="SHORT",
                bar_idx=10,
                plan={"source_bar_idx": 10, "state": "OPEN", "plan_id": "P"},
            ),
            "PLAN_OPENED",
        )
        self.assertEqual(
            ledger.classify_trade_link(
                decision="LONG",
                bar_idx=11,
                plan={"source_bar_idx": 10, "state": "OPEN", "plan_id": "P"},
            ),
            "SUPPRESSED",
        )

    def test_append_and_rebuild(self):
        with tempfile.TemporaryDirectory() as td:
            ns = Path(td) / "prospective_v1"
            ns.mkdir()
            journals = ns / "journals"
            journals.mkdir()
            rec_path = journals / "CRYPTO_recommendations.jsonl"
            events_path = Path(td) / "events.jsonl"
            outcomes_path = Path(td) / "outcomes.jsonl"
            out_path = journals / "CRYPTO_recommendation_trades.jsonl"

            recs = [
                {
                    "bar_idx": 1,
                    "decision": "NO_TRADE",
                    "is_actionable": False,
                    "symbol": "BTCUSDT",
                    "session_date": "2026-10-29",
                    "close_time_ms": 1_793_232_899_999,
                    "recorded_utc": "2026-10-29T00:15:00.100000+00:00",
                },
                {
                    "bar_idx": 2,
                    "decision": "SHORT",
                    "is_actionable": True,
                    "symbol": "BTCUSDT",
                    "session_date": "2026-10-29",
                    "close_time_ms": 1_793_233_799_999,
                    "recorded_utc": "2026-10-29T00:30:00.100000+00:00",
                },
                {
                    "bar_idx": 3,
                    "decision": "LONG",
                    "is_actionable": True,
                    "symbol": "BTCUSDT",
                    "session_date": "2026-10-29",
                    "close_time_ms": 1_793_234_699_999,
                    "recorded_utc": "2026-10-29T00:45:00.100000+00:00",
                },
            ]
            with rec_path.open("w") as fh:
                for r in recs:
                    fh.write(json.dumps(r) + "\n")
            with events_path.open("w") as fh:
                fh.write(json.dumps({
                    "event_type": "PLAN_CREATED",
                    "plan_id": "PLAN-00002-SHORT-1",
                    "direction": "SHORT",
                    "entry_price": 100.0,
                    "stop_loss_price": 101.0,
                    "profit_target_price": 98.0,
                    "state": "OPEN",
                }) + "\n")
                fh.write(json.dumps({
                    "event_type": "SUPPRESSED_OPEN_PLAN",
                    "plan_id": "PLAN-00002-SHORT-1",
                    "suppressed_card_bar_idx": 3,
                    "suppressed_direction": "LONG",
                    "direction": "SHORT",
                    "state": "OPEN",
                }) + "\n")
            with outcomes_path.open("w") as fh:
                fh.write(json.dumps({
                    "plan_id": "PLAN-00002-SHORT-1",
                    "source_bar_idx": 2,
                    "direction": "SHORT",
                    "state": "CLOSED_TP",
                    "exit_reason": "PROFIT_TARGET_HIT",
                    "entry_price": 100.0,
                    "exit_price": 98.0,
                    "net_pnl_usdt": 1.5,
                    "gross_pnl_usdt": 2.0,
                    "holding_duration_seconds": 120.0,
                }) + "\n")

            with mock.patch.object(pv1, "PROSPECTIVE_NS", ns), mock.patch.object(
                ledger, "assert_write_allowed", lambda p: None
            ), mock.patch.object(ledger, "ensure_namespace", lambda: ns), mock.patch.object(
                ledger, "journals_dir", lambda: journals
            ), mock.patch.object(ledger, "load_start_event", return_value=_START_EV):
                path = ledger.rebuild_crypto_ledger(
                    recommendations_path=rec_path,
                    events_path=events_path,
                    outcomes_path=outcomes_path,
                    out_path=out_path,
                )
                rows = [json.loads(l) for l in path.read_text().splitlines() if l.strip()]
                links = [r for r in rows if r["event_type"] == "RECOMMENDATION_TRADE_LINK"]
                closed = [r for r in rows if r["event_type"] == "TRADE_CLOSED"]
                self.assertEqual(len(links), 3)
                self.assertEqual(links[0]["trade_link"], "NO_TRADE")
                self.assertEqual(links[1]["trade_link"], "PLAN_OPENED")
                self.assertTrue(links[1]["outcome_attached"])
                self.assertEqual(links[1]["net_pnl_usdt"], 1.5)
                self.assertEqual(links[2]["trade_link"], "SUPPRESSED")
                self.assertEqual(links[2]["blocking_plan_id"], "PLAN-00002-SHORT-1")
                self.assertEqual(len(closed), 1)
                self.assertEqual(closed[0]["plan_id"], "PLAN-00002-SHORT-1")
                for row in rows:
                    self.assertEqual(row["capture_provenance"], "backfilled")
                    self.assertTrue(row["rebuild"])
                    self.assertEqual(row["start_event_utc"], _START_EV["start_event_utc"])
                    self.assertEqual(row["eligible_session_boundary"], "2026-10-29")
                    self.assertIn("rebuilt_at_utc", row)
                    self.assertNotEqual(row["recorded_utc"], row["rebuilt_at_utc"])
                self.assertEqual(links[0]["recorded_time_source"], "close_time_ms")
                self.assertTrue(links[0]["recorded_utc"].startswith("2026-10-29T00:14:59"))
                self.assertEqual(links[0]["source_recorded_utc"], "2026-10-29T00:15:00.100000+00:00")

    def test_classify_nse_trade_link(self):
        self.assertEqual(
            ledger.classify_nse_trade_link(decision="NO_TRADE", opened_trade={"trade_id": "t1"}),
            "TRADE_OPENED",
        )
        self.assertEqual(
            ledger.classify_nse_trade_link(
                decision="NO_TRADE",
                decision_code="EXISTING_POSITION_ACTIVE",
                blocking_trade={"trade_id": "t0"},
            ),
            "SUPPRESSED",
        )
        self.assertEqual(
            ledger.classify_nse_trade_link(decision="RECOMMEND_LONG"),
            "NO_PLAN",
        )
        self.assertEqual(
            ledger.classify_nse_trade_link(decision="NO_TRADE"),
            "NO_TRADE",
        )

    def test_rebuild_nse_ledger(self):
        with tempfile.TemporaryDirectory() as td:
            ns = Path(td) / "prospective_v1"
            journals = ns / "journals"
            journals.mkdir(parents=True)
            rec_path = journals / "NSE_recommendations.jsonl"
            trades_path = Path(td) / "rp608_nse_trade_journal.jsonl"
            out_path = journals / "NSE_recommendation_trades.jsonl"

            with rec_path.open("w") as fh:
                fh.write(json.dumps({
                    "bar_idx": 5,
                    "decision": "NO_TRADE",
                    "is_actionable": False,
                    "symbol": "AAA.NS",
                    "session_date": "2026-10-29",
                    "decision_code": "SUB_THRESHOLD_MOMENTUM",
                    "close_time_ms": 1_793_245_000_000,
                    "recorded_utc": "2026-10-29T03:10:00+00:00",
                    "extra": {"session_idx": 3},
                }) + "\n")
                fh.write(json.dumps({
                    "bar_idx": 6,
                    "decision": "RECOMMEND_LONG",
                    "is_actionable": True,
                    "symbol": "BBB.NS",
                    "session_date": "2026-10-29",
                    "decision_code": "QUALIFIED_MOMENTUM_BREAKOUT",
                    "close_time_ms": 1_793_245_900_000,
                    "recorded_utc": "2026-10-29T03:25:00+00:00",
                    "extra": {"session_idx": 3},
                }) + "\n")
                fh.write(json.dumps({
                    "bar_idx": 7,
                    "decision": "NO_TRADE",
                    "is_actionable": False,
                    "symbol": "BBB.NS",
                    "session_date": "2026-10-29",
                    "decision_code": "EXISTING_POSITION_ACTIVE",
                    "close_time_ms": 1_793_246_800_000,
                    "recorded_utc": "2026-10-29T03:40:00+00:00",
                    "extra": {"session_idx": 3},
                }) + "\n")
            with trades_path.open("w") as fh:
                fh.write(json.dumps({
                    "trade_id": "nse_trade_3_6_BBB.NS",
                    "symbol": "BBB.NS",
                    "side": "LONG",
                    "session_idx": 3,
                    "entry_bar_idx": 6,
                    "entry_price": 100.0,
                    "exit_bar_idx": 8,
                    "exit_price": 101.5,
                    "exit_reason": "TAKE_PROFIT",
                    "quantity": 10,
                    "estimand": "ESTIMAND_A",
                    "net_pnl_inr": 12.0,
                    "gross_pnl_inr": 15.0,
                    "net_return_bps": 120.0,
                    "gross_return_bps": 150.0,
                }) + "\n")

            nse_start = {
                **_START_EV,
                "start_event_utc": "2026-10-29T02:55:01+00:00",
                "eligible_session_open_utc": "2026-10-29T03:45:00+00:00",
            }
            with mock.patch.object(ledger, "assert_write_allowed", lambda p: None), mock.patch.object(
                ledger, "ensure_namespace", lambda: ns
            ), mock.patch.object(ledger, "journals_dir", lambda: journals), mock.patch.object(
                ledger, "load_start_event", return_value=nse_start
            ):
                path = ledger.rebuild_nse_ledger(
                    recommendations_path=rec_path,
                    trade_journal_path=trades_path,
                    out_path=out_path,
                )
                rows = [json.loads(l) for l in path.read_text().splitlines() if l.strip()]
                links = [r for r in rows if r["event_type"] == "RECOMMENDATION_TRADE_LINK"]
                closed = [r for r in rows if r["event_type"] == "TRADE_CLOSED"]
                self.assertEqual(len(links), 3)
                self.assertEqual(links[0]["trade_link"], "NO_TRADE")
                self.assertEqual(links[1]["trade_link"], "TRADE_OPENED")
                self.assertTrue(links[1]["outcome_attached"])
                self.assertEqual(links[1]["net_pnl_inr"], 12.0)
                self.assertEqual(links[2]["trade_link"], "SUPPRESSED")
                self.assertEqual(len(closed), 1)
                self.assertEqual(closed[0]["trade_id"], "nse_trade_3_6_BBB.NS")
                self.assertEqual(links[0]["capture_provenance"], "backfilled")
                self.assertEqual(links[0]["start_event_utc"], nse_start["start_event_utc"])
                self.assertEqual(links[0]["recorded_time_source"], "close_time_ms")
                self.assertTrue(links[0]["recorded_utc"].startswith("2026-10-29T"))
                self.assertEqual(links[0]["source_recorded_utc"], "2026-10-29T03:10:00+00:00")


if __name__ == "__main__":
    unittest.main()
