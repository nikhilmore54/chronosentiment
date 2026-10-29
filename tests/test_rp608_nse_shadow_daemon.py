#!/usr/bin/env python3
"""
tests/test_rp608_nse_shadow_daemon.py

Standard-library unit test suite for the RP-608 NSE prospective shadow daemon.
Zero external test framework dependencies (uses standard unittest).
"""

from __future__ import annotations

import copy
import json
import os
import shutil
import tempfile
import time
import unittest
import unittest.mock
from datetime import datetime, timezone, timedelta
from pathlib import Path

from scripts.run_rp608_nse_shadow_daemon import (
    NSE_119_SYMBOLS,
    PROTOCOL_ID,
    REGISTERED_WINDOW_START_UTC_MS,
    TOTAL_BARS_PER_SESSION,
    ESTIMAND_B_ELIGIBLE_BARS,
    ESTIMAND_A_TRAIN_PREVALENCE,
    ESTIMAND_B_TRAIN_PREVALENCE,
    FORECAST_LONG_PROB,
    FORECAST_STANDASIDE_PROB,
    BarOHLCV,
    TopOfBookQuote,
    NSECheckpointState,
    KiteDataFeedAdapter,
    RP608NSEShadowDaemon,
    calc_indian_equity_friction,
    compute_payload_checksum,
    resolve_output_dir
)


class TestRP608NSEShadowDaemon(unittest.TestCase):

    def setUp(self):
        self.test_dir = tempfile.mkdtemp(prefix="rp608_nse_test_")

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_01_registered_universe_census(self):
        """Validates that the universe contains exactly 119 distinct NSE symbols."""
        self.assertEqual(len(NSE_119_SYMBOLS), 119)
        self.assertEqual(len(set(NSE_119_SYMBOLS)), 119)
        self.assertEqual(NSE_119_SYMBOLS[0], "ADANIENT.NS")
        self.assertEqual(NSE_119_SYMBOLS[-1], "WIPRO.NS")
        for sym in NSE_119_SYMBOLS:
            self.assertTrue(sym.endswith(".NS"), f"Symbol {sym} does not end with .NS")

    def test_02_indian_equity_friction_schedule(self):
        """Verifies 8-component statutory friction arithmetic against known tax slabs."""
        # Scenario: ₹100,000 buy, ₹101,000 sell (100 shares @ ₹1000 buy, ₹1010 sell)
        res = calc_indian_equity_friction(
            buy_price=1000.0,
            sell_price=1010.0,
            quantity=100.0,
            half_spread_bps=2.5
        )

        buy_turnover = 100_000.0
        sell_turnover = 101_000.0
        tot_turnover = buy_turnover + sell_turnover

        # 1. Brokerage: min(20, 100000*0.0003=30 -> 20) + min(20, 101000*0.0003=30.3 -> 20) = ₹40.0
        self.assertEqual(res["brokerage"], 40.0)

        # 2. STT: 0.025% on sell only = 101,000 * 0.00025 = ₹25.25
        self.assertEqual(res["stt"], 25.25)

        # 3. Exchange: 0.00297% on 201,000 = ₹5.9697
        self.assertAlmostEqual(res["exchange_charges"], tot_turnover * 0.0000297, places=3)

        # 4. SEBI: ₹10 per Cr (0.0001%) on 201,000 = ₹0.201
        self.assertAlmostEqual(res["sebi_charges"], tot_turnover * 0.000001, places=3)

        # 5. GST: 18% on (40 + exch + sebi)
        expected_gst = 0.18 * (40.0 + (tot_turnover * 0.0000297) + (tot_turnover * 0.000001))
        self.assertAlmostEqual(res["gst"], expected_gst, places=2)

        # 6. Stamp duty: 0.003% on buy only = 100,000 * 0.00003 = ₹3.0
        self.assertEqual(res["stamp_duty"], 3.0)

        # 7. Spread cost: 2.5 bps on total turnover = 201,000 * 0.00025 = ₹50.25
        self.assertEqual(res["spread_cost"], 50.25)

        # 8. Slippage: 2.0 bps on total turnover = 201,000 * 0.0002 = ₹40.20
        self.assertEqual(res["slippage_cost"], 40.20)

        # Total turnover and friction sum
        self.assertEqual(res["total_turnover"], 201_000.0)
        self.assertGreater(res["total_friction"], 170.0)
        self.assertLess(res["total_friction"], 220.0)

    def test_03_session_schedule_mapping(self):
        """Verifies 15-minute interval indexing and boundary phase transitions."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)

        # Monday 09:00 IST -> Pre-open
        dt_pre = datetime(2026, 10, 26, 3, 30, tzinfo=timezone.utc)  # 09:00 IST
        active, bar_idx, phase = daemon.is_ist_session_active(dt_pre)
        self.assertFalse(active)
        self.assertEqual(bar_idx, 0)
        self.assertEqual(phase, "PRE_OPEN")

        # Monday 09:20 IST -> Bar 1
        dt_bar1 = datetime(2026, 10, 26, 3, 50, tzinfo=timezone.utc)  # 09:20 IST
        active, bar_idx, phase = daemon.is_ist_session_active(dt_bar1)
        self.assertTrue(active)
        self.assertEqual(bar_idx, 1)
        self.assertEqual(phase, "ACTIVE_SESSION")

        # Monday 14:25 IST -> Bar 21 (Eligible horizon boundary)
        dt_bar21 = datetime(2026, 10, 26, 8, 55, tzinfo=timezone.utc)  # 14:25 IST
        active, bar_idx, phase = daemon.is_ist_session_active(dt_bar21)
        self.assertTrue(active)
        self.assertEqual(bar_idx, 21)

        # Monday 15:10 IST -> Bar 24 (Mandatory square-off bar)
        dt_bar24 = datetime(2026, 10, 26, 9, 40, tzinfo=timezone.utc)  # 15:10 IST
        active, bar_idx, phase = daemon.is_ist_session_active(dt_bar24)
        self.assertTrue(active)
        self.assertEqual(bar_idx, 24)

        # Monday 15:35 IST -> Post-close
        dt_post = datetime(2026, 10, 26, 10, 5, tzinfo=timezone.utc)  # 15:35 IST
        active, bar_idx, phase = daemon.is_ist_session_active(dt_post)
        self.assertFalse(active)
        self.assertEqual(bar_idx, 0)
        self.assertEqual(phase, "POST_CLOSE")

    def test_04_estimand_b_eligibility_and_outcome_resolution(self):
        """Verifies Estimand B eligibility rules and 4-bar forward outcome resolution."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        sym = "RELIANCE.NS"

        # Bar 1 decision at 09:30 IST (P = 1000.0)
        b1 = BarOHLCV(timestamp_utc_ms=1000, open=995.0, high=1005.0, low=990.0, close=1000.0, volume=1000.0)
        card1 = daemon.evaluate_symbol_bar(sym, bar_idx=1, latest_bar=b1, quote=None)
        self.assertTrue(card1["eligible"])
        self.assertEqual(card1["status"], "PENDING_OUTCOME")
        self.assertEqual(card1["forward_target_bar_idx"], 5)

        # Simulate bars 2, 3, 4
        for idx, p in [(2, 1005.0), (3, 1010.0), (4, 1015.0)]:
            bx = BarOHLCV(timestamp_utc_ms=idx*1000, open=p-5, high=p+5, low=p-5, close=p, volume=1000.0)
            daemon.evaluate_symbol_bar(sym, bar_idx=idx, latest_bar=bx, quote=None)

        # Bar 5 outcome arrived (P = 1020.0 > 1000.0 => Label = 1)
        b5 = BarOHLCV(timestamp_utc_ms=5000, open=1015.0, high=1025.0, low=1010.0, close=1020.0, volume=1000.0)
        daemon.evaluate_symbol_bar(sym, bar_idx=5, latest_bar=b5, quote=None)

        # Verify resolved decision card written to ledger
        with open(daemon.decision_cards_path, "r", encoding="utf-8") as f:
            cards = [json.loads(line) for line in f]

        resolved_cards = [c for c in cards if c["bar_idx"] == 1 and c["status"] == "EVALUATED"]
        self.assertEqual(len(resolved_cards), 1)
        rc = resolved_cards[0]
        self.assertEqual(rc["outcome_price"], 1020.0)
        self.assertEqual(rc["directional_label"], 1)
        self.assertIsNotNone(rc["brier_score"])

        # Bar 22 decision (Boundary excluded)
        b22 = BarOHLCV(timestamp_utc_ms=22000, open=1000.0, high=1010.0, low=990.0, close=1005.0, volume=1000.0)
        card22 = daemon.evaluate_symbol_bar(sym, bar_idx=22, latest_bar=b22, quote=None)
        self.assertFalse(card22["eligible"])
        self.assertEqual(card22["status"], "INTRADAY_BOUNDARY_EXCLUDED")

    def test_05_estimand_a_statistical_invariant_and_mandatory_close(self):
        """Verifies Estimand A balanced accuracy invariant (None) and 15:15 mandatory close."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        sym = "TCS.NS"

        # Prime with 5 bars of upward momentum to trigger entry
        for i in range(1, 6):
            p = 3000.0 + (i * 20.0)
            b = BarOHLCV(timestamp_utc_ms=i*1000, open=p-5, high=p+10, low=p-5, close=p, volume=1000.0)
            daemon.evaluate_symbol_bar(sym, bar_idx=i, latest_bar=b, quote=None)

        # Verify active trade initiated
        self.assertIn(sym, daemon.active_trades)
        trade = daemon.active_trades[sym]
        self.assertIsNone(trade["balanced_accuracy"])  # Protocol invariant: strictly None/NA
        self.assertEqual(trade["reference_training_prevalence"], ESTIMAND_A_TRAIN_PREVALENCE)

        # Advance to Bar 24 (15:15 IST) -> Mandatory square-off
        b24 = BarOHLCV(timestamp_utc_ms=24000, open=3150.0, high=3160.0, low=3140.0, close=3155.0, volume=1000.0)
        daemon.evaluate_symbol_bar(sym, bar_idx=24, latest_bar=b24, quote=None)

        # Trade must be closed and written to trade journal
        self.assertNotIn(sym, daemon.active_trades)
        with open(daemon.trade_journal_path, "r", encoding="utf-8") as f:
            trades = [json.loads(line) for line in f]
        self.assertEqual(len(trades), 1)
        t = trades[0]
        self.assertEqual(t["exit_reason"], "MANDATORY_INTRADAY_SQUARE_OFF")
        self.assertEqual(t["exit_bar_idx"], 24)
        self.assertIn("friction", t)
        self.assertGreater(t["friction"]["total_friction"], 0.0)

    def test_06_atomic_checkpoint_tamper_rejection(self):
        """Verifies atomic checkpoint persistence and SHA-256 tamper rejection."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        self.assertTrue(daemon.checkpoint_path.exists())

        # Tamper with checkpoint file
        with open(daemon.checkpoint_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        data["active_positions_count"] = 99999  # Unauthorized modification
        with open(daemon.checkpoint_path, "w", encoding="utf-8") as f:
            json.dump(data, f)

        # Restart daemon should fail closed with RuntimeError
        with self.assertRaises(RuntimeError) as ctx:
            RP608NSEShadowDaemon(output_dir=self.test_dir)
        self.assertIn("Checkpoint tamper/corruption", str(ctx.exception))

    def test_07_recovery_idempotency_and_decoupled_cursors(self):
        """Verifies repeated restarts do not duplicate drop records."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        # Advance state
        daemon.state.last_durable_bar_idx = 5
        daemon.state.last_accounted_bar_idx = 5
        # Simulate last heartbeat in the past
        past_ms = REGISTERED_WINDOW_START_UTC_MS + (30 * 60 * 1000)  # 2 intervals past window start
        daemon.state.last_heartbeat_utc_ms = past_ms
        daemon.atomic_commit_checkpoint(daemon.state)

        # First restart
        with unittest.mock.patch("time.time", return_value=(past_ms + 1000) / 1000.0):
            d1 = RP608NSEShadowDaemon(output_dir=self.test_dir)
            drops1 = d1.state.unexecutable_drops_count
            outages1 = len(d1.state.accounted_outage_ids)

            # Immediate second restart without time jump
            d2 = RP608NSEShadowDaemon(output_dir=self.test_dir)
            drops2 = d2.state.unexecutable_drops_count
            outages2 = len(d2.state.accounted_outage_ids)

        # Must not duplicate outage accounting
        self.assertEqual(drops1, drops2)
        self.assertEqual(outages1, outages2)

    def test_08_pre_activation_boundary_exclusion(self):
        """Verifies gaps before 09:15 IST (2026-10-26T03:45:00Z) do not produce dropped bars."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        # Heartbeat 1 hour before window start
        pre_start_ms = REGISTERED_WINDOW_START_UTC_MS - (3600 * 1000)
        daemon.state.last_heartbeat_utc_ms = pre_start_ms
        daemon.atomic_commit_checkpoint(daemon.state)

        # Restart still before or at window start
        with unittest.mock.patch("time.time", return_value=(pre_start_ms + 1000) / 1000.0):
            d = RP608NSEShadowDaemon(output_dir=self.test_dir)
            # Drops must remain zero because window has not opened
            self.assertEqual(d.state.unexecutable_drops_count, 0)
            self.assertEqual(len(d.state.accounted_outage_ids), 0)

    def test_09_consecutive_outage_fail_closed(self):
        """Verifies protocol TG-4 fail-closed when consecutive missing bars >= 1."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        # Directly record consecutive outage breach in checkpoint
        daemon.state.consecutive_outages = 2
        daemon.atomic_commit_checkpoint(daemon.state)

        # Non-test restart must fail closed
        with self.assertRaises(RuntimeError) as ctx:
            RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=False)
        self.assertIn("Protocol TG-4 violation", str(ctx.exception))

        # Checkpoint status must be recorded as HALTED_CONSECUTIVE_OUTAGES
        with open(daemon.checkpoint_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        self.assertEqual(data["status"], "HALTED_CONSECUTIVE_OUTAGES")

    def test_10_isolated_single_bar_execution(self):
        """Executes a complete single-bar mock iteration in isolated test directory."""
        class MockFeedAdapter:
            def fetch_all_symbols(self):
                # Return synthetic bars for all 119 symbols
                res = {}
                for s in NSE_119_SYMBOLS:
                    b = BarOHLCV(timestamp_utc_ms=1000, open=100.0, high=102.0, low=99.0, close=101.0, volume=500.0)
                    q = TopOfBookQuote(timestamp_utc_ms=1000, bid_price=100.9, ask_price=101.1)
                    res[s] = {"bars": [b], "quote": q, "error": None}
                return res

        daemon = RP608NSEShadowDaemon(
            output_dir=self.test_dir,
            feed_adapter=MockFeedAdapter(),
            single_bar_test=True
        )
        # Freeze time to 10:00 IST (Bar 4) to ensure hermetic execution regardless of current time of day
        ist_offset = timedelta(hours=5, minutes=30)
        session_time_utc = (datetime(2026, 10, 27, 10, 0, 0) - ist_offset).replace(tzinfo=timezone.utc)
        with unittest.mock.patch("scripts.run_rp608_nse_shadow_daemon.datetime") as mock_dt:
            mock_dt.now.return_value = session_time_utc
            mock_dt.fromtimestamp = datetime.fromtimestamp
            mock_dt.fromisoformat = datetime.fromisoformat
            res = daemon.run_single_iteration()

        self.assertGreaterEqual(res["bar_idx"], 1)
        self.assertEqual(res["successful_symbols"], 119)
        self.assertEqual(res["failed_symbols"], 0)
        self.assertEqual(daemon.state.last_durable_bar_idx, res["bar_idx"])

        # Verify telemetry log written
        self.assertTrue(daemon.telemetry_path.exists())
        with open(daemon.telemetry_path, "r", encoding="utf-8") as f:
            lines = [json.loads(l) for l in f if l.strip()]
        self.assertGreaterEqual(len(lines), 1)
        self.assertEqual(lines[-1]["successful_symbols"], 119)

        # Verify scrip evaluations ledger written with exactly 119 records
        self.assertTrue(daemon.evaluations_path.exists())
        with open(daemon.evaluations_path, "r", encoding="utf-8") as f:
            eval_records = [json.loads(l) for l in f if l.strip()]
        self.assertEqual(len(eval_records), 119)

        # Verify latest atomic JSON snapshot exists and matches
        self.assertTrue(daemon.latest_evaluations_path.exists())
        with open(daemon.latest_evaluations_path, "r", encoding="utf-8") as f:
            latest = json.load(f)
        self.assertEqual(latest["total_evaluated"], 119)
        self.assertIn("rejection_breakdown", latest)
        self.assertEqual(latest["rejection_breakdown"].get("COLD_START_INSUFFICIENT_HISTORY"), 119)

    def test_11_zero_order_authority_verification(self):
        """Verifies strictly 0.0% order authority in code and configuration."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        # Assert no order credentials exist in daemon or environment
        for forbidden in ("API_KEY", "API_SECRET", "ORDER_URL", "KITE_API", "BROKER_TOKEN"):
            self.assertFalse(hasattr(daemon, forbidden))
            self.assertFalse(hasattr(daemon.feed_adapter, forbidden))
            self.assertNotIn(forbidden, os.environ)
        # Assert no execution/order placement methods exist
        for m in ("place_order", "submit_order", "send_order", "execute_order"):
            self.assertFalse(hasattr(daemon, m))

    def test_12_scrip_level_decision_ledger_and_rejections(self):
        """Verifies deterministic decision codes, explanations, ranking, and statutory friction."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        sym = "TCS.NS"

        # 1. Bar 1: Cold start rejection (1/5 bars)
        b1 = BarOHLCV(timestamp_utc_ms=1000, open=3000.0, high=3020.0, low=2990.0, close=3010.0, volume=100.0)
        q1 = TopOfBookQuote(timestamp_utc_ms=1000, bid_price=3009.0, ask_price=3011.0, bid_qty=50, ask_qty=50)
        c1 = daemon.evaluate_symbol_bar(sym, bar_idx=1, latest_bar=b1, quote=q1)
        e1 = daemon.last_evaluations[sym]
        self.assertEqual(e1["decision"], "NO_TRADE")
        self.assertEqual(e1["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")
        self.assertIn("1/5 bars required", e1["explanation"])
        self.assertIsNone(e1["provisional_rank"])
        self.assertIsNone(e1["risk_reward"])

        # Feed 3 more flat bars
        for idx, p in [(2, 3010.0), (3, 3010.0), (4, 3010.0)]:
            bx = BarOHLCV(timestamp_utc_ms=idx*1000, open=p, high=p+2, low=p-2, close=p, volume=100.0)
            daemon.evaluate_symbol_bar(sym, bar_idx=idx, latest_bar=bx, quote=q1)

        # 2. Bar 5 flat: Sub-threshold momentum rejection (5 bars, but traj15 ~ 0 < 0.05)
        b5_flat = BarOHLCV(timestamp_utc_ms=5000, open=3010.0, high=3011.0, low=3009.0, close=3010.0, volume=100.0)
        daemon.evaluate_symbol_bar(sym, bar_idx=5, latest_bar=b5_flat, quote=q1)
        e5 = daemon.last_evaluations[sym]
        self.assertEqual(e5["decision"], "NO_TRADE")
        self.assertEqual(e5["decision_code"], "SUB_THRESHOLD_MOMENTUM")

        # 3. Bar 6 strong breakout: traj15 >= 0.05 -> RECOMMEND_LONG
        b6_up = BarOHLCV(timestamp_utc_ms=6000, open=3010.0, high=3100.0, low=3010.0, close=3090.0, volume=5000.0)
        daemon.evaluate_symbol_bar(sym, bar_idx=6, latest_bar=b6_up, quote=q1)
        e6 = daemon.last_evaluations[sym]
        self.assertEqual(e6["decision"], "RECOMMEND_LONG")
        self.assertEqual(e6["decision_code"], "QUALIFIED_MOMENTUM_BREAKOUT")
        self.assertGreaterEqual(e6["score"], 0.05)
        self.assertIsNotNone(e6["risk_reward"])
        self.assertGreater(e6["risk_reward"]["net_reward_bps"], 120.0)
        self.assertGreater(e6["risk_reward"]["net_rr_ratio"], 1.0)

        # 4. Bar 24 (after 15:00 IST): Intraday boundary excluded takes precedence
        b24 = BarOHLCV(timestamp_utc_ms=24000, open=3090.0, high=3150.0, low=3090.0, close=3140.0, volume=5000.0)
        daemon.evaluate_symbol_bar(sym, bar_idx=24, latest_bar=b24, quote=q1)
        e24 = daemon.last_evaluations[sym]
        self.assertEqual(e24["decision"], "NO_TRADE")
        self.assertEqual(e24["decision_code"], "INTRADAY_BOUNDARY_EXCLUDED")
        self.assertIn("bar 24 >= 24", e24["explanation"])

    def test_13_bar_history_persistence_and_restore_across_restarts(self):
        """Verifies bar-history persists to rp608_nse_bar_history.json and restores without reset."""
        sym = "INFY.NS"
        q = TopOfBookQuote(timestamp_utc_ms=1000, bid_price=1500.0, ask_price=1501.0, bid_qty=10, ask_qty=10)

        # Session 1: Process Bar 1
        d1 = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        b1 = BarOHLCV(timestamp_utc_ms=1000, open=1500.0, high=1505.0, low=1495.0, close=1502.0, volume=100.0)
        d1.evaluate_symbol_bar(sym, bar_idx=1, latest_bar=b1, quote=q)
        d1.save_bar_history()
        self.assertEqual(len(d1.price_history[sym]), 1)
        self.assertEqual(d1.last_evaluations[sym]["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")

        # Session 2: Daemon restart (new instance loading existing output_dir)
        d2 = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        # Verify history restored from disk
        self.assertEqual(len(d2.price_history[sym]), 1)
        self.assertEqual(d2.price_history[sym][0].close, 1502.0)

        # Process Bar 2 (different timestamp)
        b2 = BarOHLCV(timestamp_utc_ms=2000, open=1502.0, high=1506.0, low=1500.0, close=1503.0, volume=100.0)
        d2.evaluate_symbol_bar(sym, bar_idx=2, latest_bar=b2, quote=q)
        d2.save_bar_history()
        self.assertEqual(len(d2.price_history[sym]), 2)
        self.assertIn("2/5 bars required", d2.last_evaluations[sym]["explanation"])

        # Session 3: Another restart -> restores 2 bars, progresses to 3, 4, 5
        d3 = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        self.assertEqual(len(d3.price_history[sym]), 2)

        for idx, p in [(3, 1504.0), (4, 1505.0), (5, 1506.0)]:
            bx = BarOHLCV(timestamp_utc_ms=idx*1000, open=p-1, high=p+2, low=p-2, close=p, volume=100.0)
            d3.evaluate_symbol_bar(sym, bar_idx=idx, latest_bar=bx, quote=q)
        d3.save_bar_history()

        # At 5 bars, cold-start is resolved!
        self.assertEqual(len(d3.price_history[sym]), 5)
        self.assertNotEqual(d3.last_evaluations[sym]["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")

        # Session 4: Restart preserves 5 bars
        d4 = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        self.assertEqual(len(d4.price_history[sym]), 5)

    def test_14_opening_warmup_rejection_no_bootstrap_opening(self):
        """
        Scenario 1: Opening warm-up rejection.
        Historical bootstrap must NOT turn the first 4 bars into eligible observations.
        Before the 5th completed bar (10:30 IST), Gate 3 must remain cold.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        # 10:15 IST (Bar 4 close): only 4 bars completed today
        now_ist = datetime(2026, 10, 27, 10, 15, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        adapter = MockKiteHistoricalAdapter()
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)

        res = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res["status"], "OPENING_WARMUP_COLD")
        self.assertEqual(res["completed_bars_today"], 4)
        self.assertEqual(res["recovered_symbols"], 0)

        # Feed 4 live bars; all 4 must reject as COLD_START_INSUFFICIENT_HISTORY
        sym = "TCS.NS"
        q = TopOfBookQuote(timestamp_utc_ms=1000, bid_price=3000.0, ask_price=3002.0, bid_qty=10, ask_qty=10)
        for idx in range(1, 5):
            b = BarOHLCV(timestamp_utc_ms=idx * 900000, open=3000.0, high=3010.0, low=2995.0, close=3005.0, volume=100.0, source="kite_live")
            daemon.evaluate_symbol_bar(sym, bar_idx=idx, latest_bar=b, quote=q)
            eval_rec = daemon.last_evaluations[sym]
            self.assertEqual(eval_rec["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")
            self.assertEqual(eval_rec["decision"], "NO_TRADE")
            self.assertIn(f"{idx}/5 bars required", eval_rec["explanation"])

    def test_15_mid_session_restart_recovery_and_eligibility(self):
        """
        Scenario 2: Mid-session restart recovery.
        When restarting at Bar 6 (10:45 IST), bootstrap recovers 5 completed same-day bars.
        Evaluating Bar 6 reaches 6 bars history >= 5, satisfying warm-up and allowing eligibility.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        # 10:35 IST (after 5 completed bars)
        now_ist = datetime(2026, 10, 27, 10, 35, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        sym = "INFY.NS"
        adapter = MockKiteHistoricalAdapter()

        # Build 5 completed 15m bars for INFY (09:15, 09:30, 09:45, 10:00, 10:15)
        bars_5 = []
        for idx, (h, m, p) in enumerate([(9, 15, 1500.0), (9, 30, 1502.0), (9, 45, 1504.0), (10, 0, 1506.0), (10, 15, 1508.0)], start=1):
            dt_ist = datetime(2026, 10, 27, h, m, 0)
            dt_utc = (dt_ist - ist_offset).replace(tzinfo=timezone.utc)
            ts = int(dt_utc.timestamp() * 1000)
            bars_5.append(BarOHLCV(timestamp_utc_ms=ts, open=p-2, high=p+2, low=p-2, close=p, volume=1000.0, source="kite_historical_same_day"))
        adapter.historical_data[sym] = bars_5

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)
        res = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res["status"], "BOOTSTRAP_SUCCESS")
        self.assertEqual(res["completed_bars_today"], 5)
        self.assertEqual(len(daemon.price_history[sym]), 5)
        for b in daemon.price_history[sym]:
            self.assertEqual(b.source, "kite_historical_same_day")

        # Now evaluate Bar 6 at 10:45 IST (strong breakout)
        dt_bar6 = datetime(2026, 10, 27, 10, 45, 0)
        ts_bar6 = int(((dt_bar6 - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
        b6 = BarOHLCV(timestamp_utc_ms=ts_bar6, open=1508.0, high=1560.0, low=1508.0, close=1555.0, volume=10000.0, source="kite_live")
        q6 = TopOfBookQuote(timestamp_utc_ms=ts_bar6, bid_price=1554.0, ask_price=1556.0, bid_qty=50, ask_qty=50)

        daemon.evaluate_symbol_bar(sym, bar_idx=6, latest_bar=b6, quote=q6)
        e6 = daemon.last_evaluations[sym]

        self.assertEqual(len(daemon.price_history[sym]), 6)
        self.assertEqual(e6["history_length"], 6)
        self.assertNotEqual(e6["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")
        self.assertEqual(e6["decision"], "RECOMMEND_LONG")
        self.assertEqual(e6["decision_code"], "QUALIFIED_MOMENTUM_BREAKOUT")
        self.assertEqual(e6["bar_provenance"], "kite_live")
        self.assertEqual(e6["history_provenance"]["historical_bootstrap_count"], 5)
        self.assertEqual(e6["history_provenance"]["live_count"], 1)

    def test_16_exact_candle_boundary_and_forming_candle_exclusion(self):
        """
        Scenario 3: Exact candle boundary handling.
        At 10:30:00 IST: 10:15–10:30 candle is complete; 10:30–10:45 candle is forming and excluded.
        At 10:29:59 IST: 10:15–10:30 candle is incomplete; returns OPENING_WARMUP_COLD.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        sym = "RELIANCE.NS"
        adapter = MockKiteHistoricalAdapter()

        # Build 6 candles: 5 completed (09:15 to 10:15 start) and 1 forming (10:30 start)
        all_6_bars = []
        for idx, (h, m) in enumerate([(9, 15), (9, 30), (9, 45), (10, 0), (10, 15), (10, 30)], start=1):
            dt_ist = datetime(2026, 10, 27, h, m, 0)
            ts = int(((dt_ist - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
            all_6_bars.append(BarOHLCV(timestamp_utc_ms=ts, open=2500.0, high=2510.0, low=2490.0, close=2505.0, volume=100.0, source="kite_historical_same_day"))
        adapter.historical_data[sym] = all_6_bars

        # Sub-test A: At 10:30:00 IST -> Exactly 5 completed candles; 10:30 candle excluded
        now_ist_1030 = datetime(2026, 10, 27, 10, 30, 0)
        now_utc_1030 = (now_ist_1030 - ist_offset).replace(tzinfo=timezone.utc)
        daemon_a = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)
        res_a = daemon_a.bootstrap_same_day_history(now_utc_1030)
        self.assertEqual(res_a["status"], "BOOTSTRAP_SUCCESS")
        self.assertEqual(res_a["completed_bars_today"], 5)
        self.assertEqual(len(daemon_a.price_history[sym]), 5)
        # Verify 10:30 candle was excluded
        last_bar_ist = datetime.fromtimestamp(daemon_a.price_history[sym][-1].timestamp_utc_ms / 1000.0, tz=timezone.utc) + ist_offset
        self.assertEqual(last_bar_ist.hour, 10)
        self.assertEqual(last_bar_ist.minute, 15)

        # Sub-test B: At 10:29:59 IST -> 4 completed candles; opening warm-up active
        now_ist_102959 = datetime(2026, 10, 27, 10, 29, 59)
        now_utc_102959 = (now_ist_102959 - ist_offset).replace(tzinfo=timezone.utc)
        daemon_b = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)
        res_b = daemon_b.bootstrap_same_day_history(now_utc_102959)
        self.assertEqual(res_b["status"], "OPENING_WARMUP_COLD")
        self.assertEqual(res_b["completed_bars_today"], 4)
        self.assertEqual(len(daemon_b.price_history[sym]), 0)

    def test_17_deduplication_and_live_precedence_and_repeated_calls(self):
        """
        Scenario 4: Deduplication and live observation precedence.
        Live bars are preserved over historical duplicates; repeated bootstrap calls do not duplicate bars.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = datetime(2026, 10, 27, 10, 45, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        sym = "HDFCBANK.NS"
        adapter = MockKiteHistoricalAdapter()

        # Build 5 candles in historical adapter
        bars_5 = []
        for idx, (h, m) in enumerate([(9, 15), (9, 30), (9, 45), (10, 0), (10, 15)], start=1):
            dt_ist = datetime(2026, 10, 27, h, m, 0)
            ts = int(((dt_ist - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
            bars_5.append(BarOHLCV(timestamp_utc_ms=ts, open=1600.0, high=1610.0, low=1590.0, close=1605.0, volume=100.0, source="kite_historical_same_day"))
        adapter.historical_data[sym] = bars_5

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)

        # Pre-seed Bar 1 from live observation
        bar1_live = copy.deepcopy(bars_5[0])
        bar1_live.source = "kite_live"
        bar1_live.close = 1609.99  # Distinct live close price
        daemon.price_history[sym] = [bar1_live]

        # Call bootstrap #1
        res1 = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res1["status"], "BOOTSTRAP_SUCCESS")
        self.assertEqual(len(daemon.price_history[sym]), 5)

        # Invariant check: Bar 1 live observation MUST be preserved (not overwritten by historical 1605.0)
        self.assertEqual(daemon.price_history[sym][0].source, "kite_live")
        self.assertEqual(daemon.price_history[sym][0].close, 1609.99)
        # Bars 2 to 5 are backfilled from historical
        for b in daemon.price_history[sym][1:]:
            self.assertEqual(b.source, "kite_historical_same_day")

        # Call bootstrap #2 (repeated call): history length must remain strictly 5
        res2 = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(len(daemon.price_history[sym]), 5)
        self.assertEqual(daemon.price_history[sym][0].source, "kite_live")

    def test_18_overnight_boundary_isolation(self):
        """
        Scenario 5: Overnight boundary isolation.
        Bars from prior trading session are strictly excluded by load_bar_history.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        sym = "SBIN.NS"

        # Create bars dated yesterday (2026-10-26)
        yesterday_ist = datetime(2026, 10, 26, 14, 0, 0)
        ts_yest = int(((yesterday_ist - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
        b_yest = BarOHLCV(timestamp_utc_ms=ts_yest, open=600.0, high=605.0, low=595.0, close=602.0, volume=500.0, source="persisted_live")

        daemon.price_history[sym] = [b_yest]
        daemon.save_bar_history()

        # Load with today's session date (2026-10-27)
        today_date = datetime(2026, 10, 27).date()
        fresh_history = daemon.load_bar_history(session_date=today_date)

        # Yesterday's bar must be purged from today's session
        self.assertEqual(len(fresh_history[sym]), 0)

    def test_19_api_failure_fail_closed_and_fallback(self):
        """
        Scenario 6: API failure fail-closed behavior.
        When historical API throws HTTP 500 / network error, daemon does NOT crash,
        retains valid persisted history, and fails closed with COLD_START if < 5 bars.
        """
        sym = "ITC.NS"
        q = TopOfBookQuote(timestamp_utc_ms=1000, bid_price=450.0, ask_price=451.0, bid_qty=10, ask_qty=10)

        adapter = MockKiteHistoricalAdapter()
        adapter.should_raise = True  # Simulates API outage

        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = datetime(2026, 10, 27, 10, 45, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)

        # Daemon has 2 persisted bars on disk
        b1 = BarOHLCV(timestamp_utc_ms=1000, open=450.0, high=452.0, low=449.0, close=451.0, volume=100.0, source="persisted_live")
        b2 = BarOHLCV(timestamp_utc_ms=2000, open=451.0, high=453.0, low=450.0, close=452.0, volume=100.0, source="persisted_live")
        daemon.price_history[sym] = [b1, b2]

        # Bootstrap call must catch exception and NOT crash
        res = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res["status"], "BOOTSTRAP_SUCCESS")
        self.assertEqual(res["total_backfilled_bars"], 0)
        # History retains the 2 persisted bars
        self.assertEqual(len(daemon.price_history[sym]), 2)

        # Evaluate Bar 3: must evaluate COLD_START_INSUFFICIENT_HISTORY (3/5 bars)
        b3 = BarOHLCV(timestamp_utc_ms=3000, open=452.0, high=454.0, low=451.0, close=453.0, volume=100.0, source="kite_live")
        daemon.evaluate_symbol_bar(sym, bar_idx=3, latest_bar=b3, quote=q)
        e3 = daemon.last_evaluations[sym]

        self.assertEqual(e3["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")
        self.assertEqual(e3["decision"], "NO_TRADE")
        self.assertIn("3/5 bars required", e3["explanation"])

    def test_20_zero_order_authority_remains_strictly_zero(self):
        """Verifies strictly 0.0% order authority is preserved after recovery additions."""
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir)
        for forbidden in ("API_KEY", "API_SECRET", "ORDER_URL", "KITE_API", "BROKER_TOKEN"):
            self.assertFalse(hasattr(daemon, forbidden))
            self.assertFalse(hasattr(daemon.feed_adapter, forbidden))
            self.assertNotIn(forbidden, os.environ)
        for m in ("place_order", "submit_order", "send_order", "execute_order"):
            self.assertFalse(hasattr(daemon, m))

    def test_21_auto_recovery_during_live_iteration_at_bar_5_plus(self):
        """
        Verifies that run_single_iteration automatically triggers bootstrap_same_day_history
        when bar_idx >= 5 and history is deficient (< 5 bars), recovering the session without manual intervention.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        # Mock 10:45 IST (Bar 6)
        now_ist = datetime(2026, 10, 27, 10, 45, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        adapter = MockKiteHistoricalAdapter(symbols=("INFY.NS",))

        # Build 5 historical candles (09:15 to 10:15)
        bars_5 = []
        for idx, (h, m, p) in enumerate([(9, 15, 1500.0), (9, 30, 1502.0), (9, 45, 1504.0), (10, 0, 1506.0), (10, 15, 1508.0)], start=1):
            dt_ist = datetime(2026, 10, 27, h, m, 0)
            ts = int(((dt_ist - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
            bars_5.append(BarOHLCV(timestamp_utc_ms=ts, open=p-2, high=p+2, low=p-2, close=p, volume=1000.0, source="kite_historical_same_day"))
        adapter.historical_data["INFY.NS"] = bars_5

        # Feed adapter live quote for Bar 6
        ts_bar6 = int(now_utc.timestamp() * 1000)
        b6_live = BarOHLCV(timestamp_utc_ms=ts_bar6, open=1508.0, high=1550.0, low=1508.0, close=1545.0, volume=5000.0, source="kite_live")
        q6_live = TopOfBookQuote(timestamp_utc_ms=ts_bar6, bid_price=1544.0, ask_price=1546.0, bid_qty=50, ask_qty=50)

        adapter.fetch_all_symbols = lambda: {"INFY.NS": {"bars": [b6_live], "quote": q6_live, "error": None}}

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=False)
        daemon.symbols = ("INFY.NS",)

        # Mock is_ist_session_active to return active session Bar 6
        daemon.is_ist_session_active = lambda now_dt=None: (True, 6, "ACTIVE_SESSION")

        # Prior to iteration: price_history has 0 bars
        self.assertEqual(len(daemon.price_history["INFY.NS"]), 0)

        # Run single iteration: must auto-bootstrap bars 1..5 and evaluate bar 6
        with unittest.mock.patch("scripts.run_rp608_nse_shadow_daemon.datetime") as mock_dt:
            mock_dt.now.return_value = now_utc
            mock_dt.fromtimestamp = datetime.fromtimestamp
            mock_dt.fromisoformat = datetime.fromisoformat
            res = daemon.run_single_iteration()

        # History should now contain 5 bootstrapped bars + 1 live bar = 6 bars
        self.assertEqual(len(daemon.price_history["INFY.NS"]), 6)
        eval_rec = daemon.last_evaluations["INFY.NS"]
        self.assertEqual(eval_rec["history_length"], 6)
        self.assertNotEqual(eval_rec["decision_code"], "COLD_START_INSUFFICIENT_HISTORY")
        self.assertEqual(eval_rec["decision"], "RECOMMEND_LONG")
        self.assertEqual(eval_rec["history_provenance"]["historical_bootstrap_count"], 5)
        self.assertEqual(eval_rec["history_provenance"]["live_count"], 1)

    def test_22_restart_with_partially_persisted_history_per_symbol(self):
        """
        Verifies that recovery is evaluated strictly per-symbol:
        Symbol A (deficient, 2 bars) is queried and backfilled.
        Symbol B (already complete, 5 bars) is skipped with zero redundant queries.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = datetime(2026, 10, 27, 10, 45, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        adapter = MockKiteHistoricalAdapter(symbols=("TCS.NS", "INFY.NS"))
        queried_symbols = []

        def tracking_fetch(sym, f_utc, t_utc):
            queried_symbols.append(sym)
            return adapter.historical_data.get(sym, [])

        adapter.fetch_historical_15m_bars = tracking_fetch

        # 5 bars for TCS
        bars_tcs = []
        for idx, (h, m) in enumerate([(9, 15), (9, 30), (9, 45), (10, 0), (10, 15)], start=1):
            dt_ist = datetime(2026, 10, 27, h, m, 0)
            ts = int(((dt_ist - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
            bars_tcs.append(BarOHLCV(timestamp_utc_ms=ts, open=3000.0, high=3010.0, low=2990.0, close=3005.0, volume=100.0, source="kite_historical_same_day"))
        adapter.historical_data["TCS.NS"] = bars_tcs

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)
        daemon.symbols = ("TCS.NS", "INFY.NS")

        # TCS has 2 bars (deficient); INFY has 5 bars (complete)
        daemon.price_history["TCS.NS"] = bars_tcs[:2]
        daemon.price_history["INFY.NS"] = list(bars_tcs)

        res = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res["status"], "BOOTSTRAP_SUCCESS")
        # Only TCS should have been queried
        self.assertIn("TCS.NS", queried_symbols)
        self.assertNotIn("INFY.NS", queried_symbols)
        # TCS is now backfilled to 5 bars; INFY remains at 5 bars
        self.assertEqual(len(daemon.price_history["TCS.NS"]), 5)
        self.assertEqual(len(daemon.price_history["INFY.NS"]), 5)

    def test_23_partial_historical_api_failure_subset_isolation(self):
        """
        Verifies that an API failure for a subset of symbols does NOT prevent
        successful recovery of other valid symbols in the universe.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = datetime(2026, 10, 27, 10, 45, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        adapter = MockKiteHistoricalAdapter(symbols=("FAILING.NS", "WORKING.NS"))

        def selective_fetch(sym, f_utc, t_utc):
            if sym == "FAILING.NS":
                raise RuntimeError("500 Internal Server Error for FAILING.NS")
            # Build 5 bars for WORKING
            bars = []
            for idx, (h, m) in enumerate([(9, 15), (9, 30), (9, 45), (10, 0), (10, 15)], start=1):
                dt_ist = datetime(2026, 10, 27, h, m, 0)
                ts = int(((dt_ist - ist_offset).replace(tzinfo=timezone.utc)).timestamp() * 1000)
                bars.append(BarOHLCV(timestamp_utc_ms=ts, open=100.0, high=102.0, low=99.0, close=101.0, volume=100.0, source="kite_historical_same_day"))
            return bars

        adapter.fetch_historical_15m_bars = selective_fetch

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)
        daemon.symbols = ("FAILING.NS", "WORKING.NS")

        res = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res["status"], "BOOTSTRAP_SUCCESS")
        # WORKING.NS successfully recovered to 5 bars
        self.assertEqual(len(daemon.price_history["WORKING.NS"]), 5)
        # FAILING.NS remains at 0 bars without raising an unhandled exception
        self.assertEqual(len(daemon.price_history["FAILING.NS"]), 0)

    def test_24_repeated_bootstrap_unnecessary_request_throttling(self):
        """
        Verifies that repeated polling calls within the same bar interval
        do NOT issue redundant requests to the Kite historical API.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = datetime(2026, 10, 27, 10, 45, 0)
        now_utc = (now_ist - ist_offset).replace(tzinfo=timezone.utc)

        adapter = MockKiteHistoricalAdapter(symbols=("INFY.NS",))
        call_count = 0

        def counted_fetch(sym, f_utc, t_utc):
            nonlocal call_count
            call_count += 1
            return []

        adapter.fetch_historical_15m_bars = counted_fetch

        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, feed_adapter=adapter, single_bar_test=True)
        daemon.symbols = ("INFY.NS",)

        # Call #1: Queries adapter once and records throttle for bar 5
        daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(call_count, 1)

        # Call #2 (repeated invocation in same bar interval): Must return ALREADY_UP_TO_DATE without calling API
        res2 = daemon.bootstrap_same_day_history(now_utc)
        self.assertEqual(res2["status"], "ALREADY_UP_TO_DATE")
        self.assertEqual(call_count, 1)  # No additional API call made

    def test_25_quote_feasibility_gate_rejection_despite_warmup(self):
        """
        Scenario: Valid historical warm-up (5 bars) and strong momentum,
        but current live top-of-book quote is missing, zero, or crossed.
        Gate G4 MUST reject the trade as QUOTE_FEASIBILITY_FAILURE and NO_TRADE.
        """
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=True)
        sym = "TCS.NS"

        # Seed 5 valid history bars
        for idx in range(1, 6):
            p = 3000.0 + idx * 5.0
            daemon.price_history[sym].append(
                BarOHLCV(timestamp_utc_ms=idx * 900000, open=p-2, high=p+5, low=p-2, close=p, volume=1000.0, source="kite_historical_same_day")
            )

        # Bar 6: strong breakout
        b6 = BarOHLCV(timestamp_utc_ms=6 * 900000, open=3025.0, high=3150.0, low=3025.0, close=3140.0, volume=10000.0, source="kite_live")

        # Case A: Quote is None -> Rejected by Gate G4
        daemon.evaluate_symbol_bar(sym, bar_idx=6, latest_bar=b6, quote=None)
        e_none = daemon.last_evaluations[sym]
        self.assertEqual(e_none["decision"], "NO_TRADE")
        self.assertEqual(e_none["decision_code"], "QUOTE_FEASIBILITY_FAILURE")
        self.assertIn("Gate G4", e_none["explanation"])

        # Case B: Crossed Quote (Bid >= Ask) -> Rejected by Gate G4
        q_crossed = TopOfBookQuote(timestamp_utc_ms=6 * 900000, bid_price=3142.0, ask_price=3140.0, bid_qty=50, ask_qty=50)
        daemon.evaluate_symbol_bar(sym, bar_idx=6, latest_bar=b6, quote=q_crossed)
        e_crossed = daemon.last_evaluations[sym]
        self.assertEqual(e_crossed["decision"], "NO_TRADE")
        self.assertEqual(e_crossed["decision_code"], "QUOTE_FEASIBILITY_FAILURE")

        # Case C: Zero Quote (Bid = 0) -> Rejected by Gate G4
        q_zero = TopOfBookQuote(timestamp_utc_ms=6 * 900000, bid_price=0.0, ask_price=3140.0, bid_qty=0, ask_qty=50)
        daemon.evaluate_symbol_bar(sym, bar_idx=6, latest_bar=b6, quote=q_zero)
        e_zero = daemon.last_evaluations[sym]
        self.assertEqual(e_zero["decision"], "NO_TRADE")
        self.assertEqual(e_zero["decision_code"], "QUOTE_FEASIBILITY_FAILURE")

        # Case D: Stale Quote (Age > 60000ms) -> Rejected by Gate G4
        q_stale = TopOfBookQuote(timestamp_utc_ms=(6 * 900000) - 120000, bid_price=3138.0, ask_price=3140.0, bid_qty=50, ask_qty=50)
        daemon.evaluate_symbol_bar(sym, bar_idx=6, latest_bar=b6, quote=q_stale)
        e_stale = daemon.last_evaluations[sym]
        self.assertEqual(e_stale["decision"], "NO_TRADE")
        self.assertEqual(e_stale["decision_code"], "QUOTE_FEASIBILITY_FAILURE")
        self.assertIn("Stale", e_stale["explanation"])

    def test_26_weekend_and_holiday_session_transitions(self):
        """
        Verifies that weekends and official exchange holidays are recognized as closed,
        and never trigger an accidental session advance or trade evaluation.
        Also verifies pre-open restart, post-holiday session opening, and mid-session restart.
        """
        ist_offset = timedelta(hours=5, minutes=30)
        daemon = RP608NSEShadowDaemon(output_dir=self.test_dir, single_bar_test=False)

        # 1. Saturday (2026-10-31 10:30 IST)
        sat_ist = datetime(2026, 10, 31, 10, 30, 0)
        sat_utc = (sat_ist - ist_offset).replace(tzinfo=timezone.utc)
        is_active, b_idx, phase = daemon.is_ist_session_active(sat_utc)
        self.assertFalse(is_active)
        self.assertEqual(phase, "WEEKEND_CLOSED")
        res_sat = daemon.bootstrap_same_day_history(sat_utc)
        self.assertEqual(res_sat["status"], "WEEKEND_CLOSED")

        # 2. Republic Day Exchange Holiday (2026-01-26 10:30 IST)
        hol_ist = datetime(2026, 1, 26, 10, 30, 0)
        hol_utc = (hol_ist - ist_offset).replace(tzinfo=timezone.utc)
        is_active_h, b_idx_h, phase_h = daemon.is_ist_session_active(hol_utc)
        self.assertFalse(is_active_h)
        self.assertEqual(phase_h, "EXCHANGE_HOLIDAY")
        res_hol = daemon.bootstrap_same_day_history(hol_utc)
        self.assertEqual(res_hol["status"], "EXCHANGE_HOLIDAY")

        # 3. Verify session transition does NOT occur during holiday
        daemon.current_session_date = "2026-01-23"
        daemon.state.current_session_date = "2026-01-23"
        daemon.state.session_idx = 1
        with unittest.mock.patch("scripts.run_rp608_nse_shadow_daemon.datetime") as mock_dt:
            mock_dt.now.return_value = hol_utc
            mock_dt.fromtimestamp = datetime.fromtimestamp
            mock_dt.fromisoformat = datetime.fromisoformat
            res_iter = daemon.run_single_iteration()

        self.assertEqual(res_iter["status"], "EXCHANGE_HOLIDAY")
        self.assertEqual(daemon.state.session_idx, 1)  # Did not increment
        self.assertEqual(daemon.current_session_date, "2026-01-23")  # Did not overwrite

        # 4. Verify restart before 09:15 IST (Pre-Open) does not alter session state
        preopen_ist = datetime(2026, 1, 27, 8, 45, 0)
        preopen_utc = (preopen_ist - ist_offset).replace(tzinfo=timezone.utc)
        with unittest.mock.patch("scripts.run_rp608_nse_shadow_daemon.datetime") as mock_dt:
            mock_dt.now.return_value = preopen_utc
            mock_dt.fromtimestamp = datetime.fromtimestamp
            mock_dt.fromisoformat = datetime.fromisoformat
            res_pre = daemon.run_single_iteration()

        self.assertEqual(res_pre["status"], "PRE_OPEN")
        self.assertEqual(daemon.state.session_idx, 1)  # Preserved
        self.assertEqual(daemon.current_session_date, "2026-01-23")  # Untouched by closed heartbeat

        # 5. First session after holiday opens at 09:15 IST: transitions cleanly to 2026-01-27
        open_ist = datetime(2026, 1, 27, 9, 16, 0)
        open_utc = (open_ist - ist_offset).replace(tzinfo=timezone.utc)
        with unittest.mock.patch("scripts.run_rp608_nse_shadow_daemon.datetime") as mock_dt:
            mock_dt.now.return_value = open_utc
            mock_dt.fromtimestamp = datetime.fromtimestamp
            mock_dt.fromisoformat = datetime.fromisoformat
            res_open = daemon.run_single_iteration()

        self.assertEqual(daemon.state.session_idx, 2)  # Incremented cleanly
        self.assertEqual(daemon.current_session_date, "2026-01-27")

    def test_27_pending_evaluations_persistence_across_restarts(self):
        """
        Verifies that pending directional evaluations (Estimand B) and active trades (Estimand A)
        are durably persisted to disk and restored across daemon restarts.
        """
        daemon1 = RP608NSEShadowDaemon(output_dir=self.test_dir)
        sym = "RELIANCE.NS"

        # Register pending directional card for target bar 5
        daemon1.pending_directional_evals[5] = {
            sym: {
                "decision_price": 1000.0,
                "prediction": 1,
                "forecast_prob": 0.55,
                "card": {
                    "protocol_id": PROTOCOL_ID,
                    "estimand": "ESTIMAND_B",
                    "symbol": sym,
                    "session_idx": 1,
                    "bar_idx": 1,
                    "decision_timestamp_utc_ms": 1000,
                    "reference_price": 1000.0,
                    "prediction": 1,
                    "forecast_probability_up": 0.55,
                    "eligible": True,
                    "status": "PENDING_OUTCOME",
                    "forward_target_bar_idx": 5
                }
            }
        }
        # Register active trade
        daemon1.active_trades[sym] = {
            "trade_id": "trade_1",
            "symbol": sym,
            "entry_bar_idx": 1,
            "entry_price": 1000.0,
            "quantity": 100,
            "half_spread_bps": 1.5,
            "mfe_price": 1000.0,
            "mae_price": 1000.0
        }
        daemon1.save_pending_evaluations()

        # Verify durable JSON committed
        self.assertTrue(daemon1.pending_evaluations_path.exists())

        # Simulate daemon crash / restart: create new daemon instance
        daemon2 = RP608NSEShadowDaemon(output_dir=self.test_dir)
        self.assertIn(5, daemon2.pending_directional_evals)
        self.assertIn(sym, daemon2.pending_directional_evals[5])
        self.assertEqual(daemon2.pending_directional_evals[5][sym]["decision_price"], 1000.0)
        self.assertIn(sym, daemon2.active_trades)
        self.assertEqual(daemon2.active_trades[sym]["entry_price"], 1000.0)

        # Advance to Bar 5 on daemon2 and verify outcome resolution
        b5 = BarOHLCV(timestamp_utc_ms=5000, open=1015.0, high=1025.0, low=1010.0, close=1020.0, volume=1000.0)
        daemon2.evaluate_symbol_bar(sym, bar_idx=5, latest_bar=b5, quote=None)

        # Verify card resolved to ledger
        with open(daemon2.decision_cards_path, "r", encoding="utf-8") as f:
            cards = [json.loads(line) for line in f]
        resolved = [c for c in cards if c["bar_idx"] == 1 and c["status"] == "EVALUATED"]
        self.assertEqual(len(resolved), 1)
        self.assertEqual(resolved[0]["outcome_price"], 1020.0)
        self.assertEqual(resolved[0]["directional_label"], 1)


class MockKiteHistoricalAdapter:
    """Mock adapter for unit testing historical 15m candle recovery."""
    def __init__(self, symbols=NSE_119_SYMBOLS):
        self.symbols = symbols
        self.historical_data = {}
        self.should_raise = False

    def fetch_historical_15m_bars(self, symbol: str, from_utc: datetime, to_utc: datetime):
        if self.should_raise:
            raise RuntimeError("Simulated Kite historical API 500 Internal Error")
        return list(self.historical_data.get(symbol, []))

    def fetch_all_symbols(self):
        return {s: {"bars": [], "quote": None, "error": None} for s in self.symbols}


if __name__ == "__main__":
    unittest.main()
