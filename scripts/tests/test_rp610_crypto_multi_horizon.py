#!/usr/bin/env python3
"""Unit tests for RP-610 crypto multi-horizon observation."""

from __future__ import annotations

import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import scripts.rp610_crypto_multi_horizon as horizons
from scripts.rp610_crypto_multi_horizon import estimate_crypto_friction_usdt
from scripts.rp610_crypto_multi_horizon_engine import (
    admit_candidate,
    amend_hold,
    build_candidate,
    evaluate_open_hold,
    find_open_hold,
    open_holds_from_book,
    write_daily_observation,
)

UTC = timezone.utc


class Rp610CryptoTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.journals = self.root / "journals"
        self.journals.mkdir(parents=True)
        horizons.ARTIFACT_ROOT = self.root
        horizons.JOURNALS = self.journals

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_friction_positive(self) -> None:
        fr = estimate_crypto_friction_usdt(entry_price=100.0, exit_price=110.0, quantity=1.0)
        self.assertGreater(fr["total_friction_usdt"], 0.0)
        self.assertEqual(fr["round_trip_friction_bps"], 12.0)

    def test_h0_reject(self) -> None:
        with self.assertRaises(ValueError):
            build_candidate(
                symbol="BTCUSDT",
                side="LONG",
                entry_price=68000.0,
                entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
                horizon_id="H0_INTRADAY_60M",
            )

    def test_h1_target_exit(self) -> None:
        c = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=100.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H1_MULTI_DAY",
            stop_pct=0.05,
            target_pct=0.10,
            notional_usdt=1000.0,
        )
        admit_candidate(c)
        closed = evaluate_open_hold(
            c,
            mark_ts=datetime(2026, 10, 3, tzinfo=UTC),
            mark_price=112.0,
        )
        self.assertIsNotNone(closed)
        assert closed is not None
        self.assertEqual(closed["exit_reason"], "TARGET")
        self.assertGreater(closed["net_pnl_usdt"], 0.0)

    def test_h1_invalidation_and_max_hold(self) -> None:
        c = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=100.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H1_MULTI_DAY",
            stop_pct=0.05,
            target_pct=0.50,
            notional_usdt=1000.0,
        )
        stopped = evaluate_open_hold(
            c,
            mark_ts=datetime(2026, 10, 2, tzinfo=UTC),
            mark_price=94.0,
        )
        self.assertEqual(stopped["exit_reason"], "INVALIDATION")

        c2 = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=100.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H1_MULTI_DAY",
            stop_pct=0.50,
            target_pct=0.50,
            notional_usdt=1000.0,
        )
        # hold_days_max for H1 is 14
        maxed = evaluate_open_hold(
            c2,
            mark_ts=datetime(2026, 10, 1, tzinfo=UTC) + timedelta(days=14),
            mark_price=101.0,
        )
        self.assertEqual(maxed["exit_reason"], "MAX_HOLD_DAYS")

    def test_h2_defaults_and_eod(self) -> None:
        c = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=68000.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H2_MULTI_WEEK",
        )
        self.assertEqual(c["hold_days_min"], 14)
        self.assertEqual(c["hold_days_max"], 56)
        admit_candidate(c)
        self.assertEqual(len(open_holds_from_book("H2_MULTI_WEEK")), 1)
        summary = write_daily_observation(
            mark_ts=datetime(2026, 10, 2, tzinfo=UTC),
            marks={"BTCUSDT": 68500.0},
        )
        self.assertEqual(summary["live_order_authority"], 0.0)
        self.assertEqual(summary["decision_status"], "OBSERVE_ONLY_NO_CONCLUSION")
        self.assertEqual(summary["open_counts"]["H2_MULTI_WEEK"], 1)

    def test_h3_no_max_hold_exit(self) -> None:
        c = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=68000.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H3_LONG_TERM",
        )
        self.assertIsNone(c["hold_days_max"])
        self.assertIsNone(c["max_exit_ts"])
        # far future mark should not force close without stop/target
        still = evaluate_open_hold(
            c,
            mark_ts=datetime(2027, 10, 1, tzinfo=UTC),
            mark_price=70000.0,
        )
        self.assertIsNone(still)

    def test_registry_gates_closed(self) -> None:
        path = horizons.write_horizon_registry()
        data = path.read_text(encoding="utf-8")
        self.assertIn("G-LIVE", data)
        self.assertIn("CLOSED", data)
        self.assertIn("H1_MULTI_DAY", data)

    def test_amend_target_then_exit(self) -> None:
        c = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=100.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H1_MULTI_DAY",
            stop_pct=0.05,
            target_pct=0.20,
            notional_usdt=1000.0,
        )
        admit_candidate(c)
        out = amend_hold(c["candidate_id"], reason="raise_target", target_pct=0.08)
        self.assertEqual(out["amendment"]["prior"]["target_pct"], 0.20)
        self.assertEqual(out["effective"]["target_pct"], 0.08)
        # Marks must not wipe amended target
        write_daily_observation(
            mark_ts=datetime(2026, 10, 2, tzinfo=UTC),
            marks={"BTCUSDT": 105.0},
        )
        hold = find_open_hold(c["candidate_id"])
        assert hold is not None
        self.assertEqual(hold["target_pct"], 0.08)
        self.assertEqual(hold["entry_price"], 100.0)
        closed = evaluate_open_hold(
            hold,
            mark_ts=datetime(2026, 10, 3, tzinfo=UTC),
            mark_price=109.0,
        )
        self.assertEqual(closed["exit_reason"], "TARGET")

    def test_horizon_roll_preserves_entry(self) -> None:
        c = build_candidate(
            symbol="BTCUSDT",
            side="LONG",
            entry_price=68000.0,
            entry_ts=datetime(2026, 10, 1, tzinfo=UTC),
            horizon_id="H1_MULTI_DAY",
        )
        admit_candidate(c)
        out = amend_hold(
            c["candidate_id"],
            reason="extend_to_multi_week",
            new_horizon_id="H2_MULTI_WEEK",
            mark_price=68500.0,
            mark_ts=datetime(2026, 10, 5, tzinfo=UTC),
        )
        self.assertEqual(out["supersede"]["status"], "SUPERSEDED")
        child = out["child"]
        self.assertEqual(child["parent_candidate_id"], c["candidate_id"])
        self.assertEqual(child["entry_price"], 68000.0)
        self.assertEqual(child["horizon_id"], "H2_MULTI_WEEK")
        self.assertIsNone(find_open_hold(c["candidate_id"]))
        self.assertIsNotNone(find_open_hold(child["candidate_id"]))
        self.assertEqual(len(open_holds_from_book("H1_MULTI_DAY")), 0)
        self.assertEqual(len(open_holds_from_book("H2_MULTI_WEEK")), 1)


if __name__ == "__main__":
    unittest.main()
