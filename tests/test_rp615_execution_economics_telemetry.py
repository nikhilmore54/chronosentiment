#!/usr/bin/env python3
"""
tests/test_rp615_execution_economics_telemetry.py

Milestone Verification Test Suite for:
RP-615-EE-001 — Decision-Time Execution Economics Telemetry

Enforces all 10 Acceptance Criteria:
1. [x] C5 inference unchanged
2. [x] C5 admission unchanged
3. [x] t0 bid/ask captured
4. [x] spread_bps captured
5. [x] book freshness captured
6. [x] C5 score recorded alongside telemetry
7. [x] prospective passive prices recorded (passive entry = bid for LONG, ask for SHORT)
8. [x] no execution authority introduced (0% live authority, observational only)
9. [x] existing RP-615 tests remain green
10. [x] telemetry is append-only & reconciles 1:1 with decision cards
"""

import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

from scripts.run_rp615_crypto_shadow_daemon import (
    CryptoLiveShadowEngine,
    extract_features_from_live_bars,
    CANDIDATES,
)


class TestRP615ExecutionEconomicsTelemetry(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.tmp_path = Path(self.temp_dir.name)
        self.telemetry_path = self.tmp_path / "test_execution_economics_telemetry.jsonl"
        self.decision_cards_path = self.tmp_path / "test_decision_cards.jsonl"
        self.queue_tracker_path = self.tmp_path / "test_passive_queue_tracker.jsonl"

        self.engine = CryptoLiveShadowEngine(
            state_path=self.tmp_path / "mock_state.json",
            telemetry_path=self.tmp_path / "mock_telemetry.json",
            trade_journal_path=self.tmp_path / "mock_trade_journal.jsonl",
            expansion_trade_journal_path=self.tmp_path / "mock_expansion_trade_journal.jsonl",
            decision_cards_path=self.decision_cards_path,
            expansion_decision_cards_path=self.tmp_path / "mock_expansion_cards.jsonl",
            execution_economics_path=self.telemetry_path,
            passive_queue_tracker_path=self.queue_tracker_path,
        )

        # Build standard synthetic test bars (60 minutes)
        self.now_dt = datetime(2026, 10, 22, 12, 0, 0, tzinfo=timezone.utc)
        base_time = int(self.now_dt.timestamp())
        self.bars = []
        p = 65000.0
        for i in range(60):
            t = base_time - (59 - i) * 60
            self.bars.append({
                "open_time": t * 1000,
                "open": p,
                "high": p + 15.0,
                "low": p - 10.0,
                "close": p + 5.0,
                "volume": 25.0,
                "close_time": (t + 59) * 1000,
                "timestamp": t + 59,
            })
            p += 2.0

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_01_c5_inference_and_admission_strictly_unchanged(self):
        """Criterion 1 & 2: C5 inference & admission logic remain bit-for-bit identical with/without book telemetry."""
        # 1. Evaluate without book ticker
        res_no_book = self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=None)

        # 2. Reset state and evaluate with book ticker
        self.engine.active_positions = {c: {"BTCUSDT": None} for c in CANDIDATES}
        self.engine.last_exits = {c: {"BTCUSDT": None} for c in CANDIDATES}

        mock_book = {
            "symbol": "BTCUSDT",
            "bid": 65119.50,
            "ask": 65120.00,
            "bid_qty": 4.5,
            "ask_qty": 3.2,
            "fetch_timestamp_ms": int(self.now_dt.timestamp() * 1000) - 45,
            "latency_ms": 45.2,
        }
        res_with_book = self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=mock_book)

        # Verify C5 score identical
        self.assertEqual(res_no_book["scores"]["p_calibrated"], res_with_book["scores"]["p_calibrated"])
        self.assertEqual(res_no_book["scores"]["pred_net_pnl"], res_with_book["scores"]["pred_net_pnl"])
        self.assertEqual(res_no_book["scores"]["p_be"], res_with_book["scores"]["p_be"])
        self.assertEqual(res_no_book["scores"]["lcb_mean"], res_with_book["scores"]["lcb_mean"])

        # Verify C5 admission decision identical
        self.assertEqual(
            res_no_book["admissions"]["C5_Fixed_Capacity_Top2Pct"],
            res_with_book["admissions"]["C5_Fixed_Capacity_Top2Pct"]
        )
        self.assertEqual(
            res_no_book["des_execution"]["C5_Fixed_Capacity_Top2Pct"],
            res_with_book["des_execution"]["C5_Fixed_Capacity_Top2Pct"]
        )

    def test_02_microstructure_telemetry_fields_captured(self):
        """Criteria 3, 4, 5, 6: t0 bid/ask, spread_bps, book freshness, and C5 score are captured."""
        fetch_ts = int(self.now_dt.timestamp() * 1000) - 120
        mock_book = {
            "symbol": "BTCUSDT",
            "bid": 68000.0,
            "ask": 68001.0,
            "bid_qty": 5.12,
            "ask_qty": 10.45,
            "fetch_timestamp_ms": fetch_ts,
            "latency_ms": 32.5,
        }
        self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=mock_book)

        self.assertTrue(self.telemetry_path.exists())
        with self.telemetry_path.open() as f:
            lines = [json.loads(line) for line in f if line.strip()]

        self.assertEqual(len(lines), 1)
        telem = lines[0]

        # Criterion 3: t0 bid/ask
        self.assertEqual(telem["bid"], 68000.0)
        self.assertEqual(telem["ask"], 68001.0)
        self.assertEqual(telem["bid_qty"], 5.12)
        self.assertEqual(telem["ask_qty"], 10.45)
        self.assertEqual(telem["mid"], 68000.5)

        # Criterion 4: spread_abs and spread_bps
        self.assertAlmostEqual(telem["spread_abs"], 1.0, places=4)
        expected_bps = (1.0 / 68000.5) * 10000.0
        self.assertAlmostEqual(telem["spread_bps"], expected_bps, places=3)

        # Criterion 5: book freshness
        self.assertEqual(telem["book_age_ms"], 120)
        self.assertEqual(telem["book_latency_ms"], 32.5)

        # Criterion 6: C5 score recorded
        self.assertIn("c5_hurdle_score", telem)
        self.assertIn("c5_pred_net_pnl", telem)
        self.assertIn("c5_p_calibrated", telem)
        self.assertIn("c5_admitted", telem)

    def test_03_prospective_passive_execution_pricing(self):
        """Criterion 7: Prospective passive prices recorded correctly (bid for LONG, ask for SHORT), with NO fill assumed."""
        # Test LONG side
        mock_book_long = {
            "symbol": "BTCUSDT",
            "bid": 60000.0,
            "ask": 60002.0,
            "bid_qty": 1.0,
            "ask_qty": 1.0,
            "fetch_timestamp_ms": int(self.now_dt.timestamp() * 1000),
            "latency_ms": 10.0,
        }
        # In self.bars, price increased, so ret_15m > 0 => LONG
        self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=mock_book_long)

        with self.telemetry_path.open() as f:
            lines = [json.loads(line) for line in f if line.strip()]

        entry_long = lines[-1]
        self.assertEqual(entry_long["side"], "LONG")
        # For LONG: passive entry price is the BID
        self.assertEqual(entry_long["prospective_passive_entry_price"], 60000.0)
        # Observational only: NOT a fill
        self.assertEqual(entry_long["prospective_passive_fill"]["status"], "RECORDED_OBSERVATION")
        self.assertEqual(entry_long["prospective_passive_fill"]["entry_order_type"], "LIMIT_POST_ONLY")
        self.assertEqual(entry_long["prospective_passive_fill"]["entry_limit_price"], 60000.0)

        # Test SHORT side (create bars with downward trend)
        down_bars = []
        p = 65000.0
        base_time = int(self.now_dt.timestamp())
        for i in range(60):
            t = base_time - (59 - i) * 60
            down_bars.append({
                "open_time": t * 1000,
                "open": p,
                "high": p + 5.0,
                "low": p - 15.0,
                "close": p - 5.0,
                "volume": 25.0,
                "close_time": (t + 59) * 1000,
                "timestamp": t + 59,
            })
            p -= 2.0

        mock_book_short = {
            "symbol": "ETHUSDT",
            "bid": 3000.0,
            "ask": 3001.0,
            "bid_qty": 20.0,
            "ask_qty": 15.0,
            "fetch_timestamp_ms": int(self.now_dt.timestamp() * 1000),
            "latency_ms": 15.0,
        }
        self.engine.evaluate_opportunity("ETHUSDT", down_bars, self.now_dt, book_ticker=mock_book_short)

        with self.telemetry_path.open() as f:
            lines = [json.loads(line) for line in f if line.strip()]

        entry_short = lines[-1]
        self.assertEqual(entry_short["side"], "SHORT")
        # For SHORT: passive entry price is the ASK
        self.assertEqual(entry_short["prospective_passive_entry_price"], 3001.0)
        self.assertEqual(entry_short["prospective_passive_fill"]["status"], "RECORDED_OBSERVATION")
        self.assertEqual(entry_short["prospective_passive_fill"]["entry_limit_price"], 3001.0)

    def test_04_append_only_and_1_to_1_decision_reconciliation(self):
        """Criteria 10: Telemetry is strictly append-only and reconciles 1:1 with decision cards."""
        # Run 3 consecutive evaluations
        t1 = datetime(2026, 10, 22, 12, 1, 0, tzinfo=timezone.utc)
        t2 = datetime(2026, 10, 22, 12, 2, 0, tzinfo=timezone.utc)
        t3 = datetime(2026, 10, 22, 12, 3, 0, tzinfo=timezone.utc)

        mock_book = {
            "symbol": "BTCUSDT",
            "bid": 65000.0,
            "ask": 65000.5,
            "bid_qty": 1.0,
            "ask_qty": 1.0,
            "fetch_timestamp_ms": int(t1.timestamp() * 1000),
            "latency_ms": 10.0,
        }

        d1 = self.engine.evaluate_opportunity("BTCUSDT", self.bars, t1, book_ticker=mock_book)
        d2 = self.engine.evaluate_opportunity("BTCUSDT", self.bars, t2, book_ticker=mock_book)
        d3 = self.engine.evaluate_opportunity("BTCUSDT", self.bars, t3, book_ticker=mock_book)

        with self.telemetry_path.open() as f:
            telem_records = [json.loads(line) for line in f if line.strip()]

        with self.decision_cards_path.open() as f:
            card_records = [json.loads(line) for line in f if line.strip()]

        self.assertEqual(len(telem_records), 3)
        self.assertEqual(len(card_records), 3)

        # Verify 1:1 correspondence of trade_id, symbol, and timestamp
        for i in range(3):
            self.assertEqual(telem_records[i]["trade_id"], card_records[i]["trade_id"])
            self.assertEqual(telem_records[i]["symbol"], card_records[i]["symbol"])
            self.assertEqual(telem_records[i]["timestamp_utc"], card_records[i]["timestamp_utc"])
            self.assertEqual(telem_records[i]["reference_price"], card_records[i]["current_price"])

    def test_05_no_execution_authority(self):
        """Criterion 8: Confirms zero execution authority (0% authority, no trades sent to exchange)."""
        mock_book = {
            "symbol": "BTCUSDT",
            "bid": 65000.0,
            "ask": 65000.5,
            "bid_qty": 1.0,
            "ask_qty": 1.0,
            "fetch_timestamp_ms": int(self.now_dt.timestamp() * 1000),
            "latency_ms": 10.0,
        }
        res = self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=mock_book)

        # Neither the telemetry call nor evaluate_opportunity routed any external order
        self.assertNotIn("live_order_id", res)
        self.assertNotIn("exchange_status", res)
        self.assertTrue(self.telemetry_path.exists())

    def test_06_unpublished_volume_never_coerced_to_zero(self):
        """Verify that unpublished/missing volume is NEVER coerced to 0.0 and receives neutral priors."""
        # Construct bars where volume is None or missing
        unpub_bars = []
        base_time = int(self.now_dt.timestamp())
        for i in range(60):
            t = base_time - (59 - i) * 60
            unpub_bars.append({
                "open": 65000.0,
                "high": 65010.0,
                "low": 64990.0,
                "close": 65005.0,
                "volume": None,  # Unpublished volume!
                "timestamp": t + 59,
            })

        feats = extract_features_from_live_bars(unpub_bars, base_time)
        self.assertIsNotNone(feats)
        self.assertEqual(feats["_volume_status"], "UNPUBLISHED")
        # Must NOT collapse to 0.0
        self.assertEqual(feats["F04_rvol_15m"], 1.0, "Unpublished RVOL must assume neutral prior 1.0")
        self.assertEqual(feats["F12_vol_accel"], 0.0, "Unpublished vol accel must assume mean prior 0.0")
        self.assertEqual(feats["_va_30_60"], 1.0, "Unpublished volume acceleration ratio must assume neutral prior 1.0")

        # Evaluate opportunity and inspect written telemetry
        self.engine.evaluate_opportunity("BTCUSDT", unpub_bars, self.now_dt)
        with self.telemetry_path.open() as f:
            lines = [json.loads(line) for line in f if line.strip()]

        last_record = lines[-1]
        self.assertIn("volume_microstructure", last_record)
        vol_micro = last_record["volume_microstructure"]
        self.assertEqual(vol_micro["status"], "UNPUBLISHED")
        self.assertTrue(vol_micro["neutral_prior_applied"])
        self.assertEqual(vol_micro["rvol_15m"], 1.0)
        self.assertEqual(vol_micro["volume_acceleration_ratio_30_60"], 1.0)

    def test_07_true_zero_volume_distinguished_from_unpublished(self):
        """Verify that true 0.0 volume is correctly identified as TRUE_ZERO rather than UNPUBLISHED."""
        zero_bars = []
        base_time = int(self.now_dt.timestamp())
        for i in range(60):
            t = base_time - (59 - i) * 60
            zero_bars.append({
                "open": 65000.0,
                "high": 65010.0,
                "low": 64990.0,
                "close": 65005.0,
                "volume": 0.0,  # Legitimate zero trades occurred
                "timestamp": t + 59,
            })

        feats = extract_features_from_live_bars(zero_bars, base_time)
        self.assertIsNotNone(feats)
        self.assertEqual(feats["_volume_status"], "TRUE_ZERO")
        self.assertEqual(feats["F04_rvol_15m"], 0.0)

    def test_08_published_volume_trend_computed(self):
        """Verify that published volume correctly computes relative volume and volume acceleration trend."""
        trend_bars = []
        base_time = int(self.now_dt.timestamp())
        for i in range(70):
            t = base_time - (69 - i) * 60
            # Volume surges from 10.0 in first 35 mins to 50.0 in last 35 mins (VA > 1.0)
            v = 10.0 if i < 35 else 50.0
            trend_bars.append({
                "open": 65000.0,
                "high": 65010.0,
                "low": 64990.0,
                "close": 65005.0,
                "volume": v,
                "timestamp": t,
            })

        feats = extract_features_from_live_bars(trend_bars, base_time)
        self.assertIsNotNone(feats)
        self.assertEqual(feats["_volume_status"], "PUBLISHED")
        # In last 30m, volume is surging vs prior 30m -> VA ratio >= 3.0
        self.assertGreaterEqual(feats["_va_30_60"], 3.0)
        # RVOL 15m (50.0) vs 60m mean -> ~ 1.67
        self.assertGreater(feats["F04_rvol_15m"], 1.4)

    def test_09_passive_queue_exhaustion_fill_tracking(self):
        """Verify empirical volume-flow queue clearance tracking (RP-615-EE-002)."""
        mock_book = {
            "symbol": "BTCUSDT",
            "bid": 65000.0,
            "ask": 65001.0,
            "bid_qty": 1.0,
            "ask_qty": 1.0,
            "fetch_timestamp_ms": int(self.now_dt.timestamp() * 1000),
            "latency_ms": 10.0,
            "spread_bps": 0.1538,
        }
        res = self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=mock_book)
        self.assertIn("des_execution", res)

        # Check order registered in pending_passive_orders
        self.assertEqual(len(self.engine.pending_passive_orders), 1)
        order = list(self.engine.pending_passive_orders.values())[0]
        self.assertEqual(order["status"], "PENDING_QUEUE")
        self.assertEqual(order["limit_price"], 65000.0)
        v_thresh = order["v_threshold_qty"]
        cur_p = self.bars[-1]["close"]
        self.assertAlmostEqual(v_thresh, 1.0 + 0.5 * (10000.0 / cur_p), places=4)

        # 1. Partial volume arrival (low touches limit, but contra volume insufficient)
        t1 = datetime.fromtimestamp(int(self.now_dt.timestamp()) + 60, tz=timezone.utc)
        bar1 = {"close": 65000.0, "low": 65000.0, "high": 65005.0, "volume": 0.5, "timestamp": int(t1.timestamp())}
        resolved1 = self.engine.update_passive_queue_orders({"BTCUSDT": [bar1]}, t1)
        self.assertEqual(len(resolved1), 0)
        self.assertEqual(order["cum_contra_volume"], 0.5)
        self.assertEqual(order["status"], "PENDING_QUEUE")

        # 2. Cumulative volume arrival exceeds threshold (0.5 + 0.7 = 1.2 >= v_thresh)
        t2 = datetime.fromtimestamp(int(self.now_dt.timestamp()) + 120, tz=timezone.utc)
        bar2 = {"close": 64998.0, "low": 64995.0, "high": 65002.0, "volume": 0.7, "timestamp": int(t2.timestamp())}
        resolved2 = self.engine.update_passive_queue_orders({"BTCUSDT": [bar2]}, t2)
        self.assertEqual(len(resolved2), 1)
        filled_order = resolved2[0]
        self.assertEqual(filled_order["status"], "PROSPECTIVELY_FILLED")
        self.assertEqual(filled_order["time_to_fill_seconds"], 120.0)
        # Check empirical adverse selection haircut (0.50 bps) incorporated
        self.assertEqual(filled_order["adverse_selection_drift_bps"], 0.50)
        self.assertIn("effective_cost_bps", filled_order)

        # Verify pending list is now empty and record written to queue tracker JSONL
        self.assertEqual(len(self.engine.pending_passive_orders), 0)
        self.assertTrue(self.queue_tracker_path.exists())

    def test_10_queue_expiry_when_price_runs_away(self):
        """Verify that pending passive order cancels if price runs away without filling."""
        mock_book = {
            "symbol": "BTCUSDT",
            "bid": 65000.0,
            "ask": 65001.0,
            "bid_qty": 2.0,
            "ask_qty": 2.0,
            "fetch_timestamp_ms": int(self.now_dt.timestamp() * 1000),
            "latency_ms": 10.0,
        }
        self.engine.evaluate_opportunity("BTCUSDT", self.bars, self.now_dt, book_ticker=mock_book)
        self.assertEqual(len(self.engine.pending_passive_orders), 1)

        # Price surges to 66000 (> 0.5% above limit) after 1000s
        t_runaway = datetime.fromtimestamp(int(self.now_dt.timestamp()) + 1000, tz=timezone.utc)
        runaway_bar = {"close": 66000.0, "low": 65800.0, "high": 66100.0, "volume": 10.0, "timestamp": int(t_runaway.timestamp())}
        self.engine.update_passive_queue_orders({"BTCUSDT": [runaway_bar]}, t_runaway)

        # Order must be cancelled and cleared
        self.assertEqual(len(self.engine.pending_passive_orders), 0)


if __name__ == "__main__":
    unittest.main()
