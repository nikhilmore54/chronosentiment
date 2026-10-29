#!/usr/bin/env python3
"""
tests/test_rp608_timing_experiment.py

Unit Test Suite for RP-608-TIMING-EXP-001 Horizon-Aware Timing Engine.
Verifies all five required replay validation gates:
  1. Timestamp alignment and horizon eligibility at every boundary.
  2. Synthetic outcome fixtures (target, stop, both-crossed conservative, timeout, square-off, data failure).
  3. Hand-worked statutory friction reconciliation (8 components + spread + slippage).
  4. Determinism and reproducibility (identical inputs -> identical hashes).
  5. Decoupled estimands and non-censoring independence.
"""

import hashlib
import json
import unittest
from datetime import datetime, timezone

from scripts.replay_rp608_nse_timing import (
    get_bar_interval_info,
    is_horizon_eligible,
    evaluate_lifecycle_trade,
    evaluate_directional_prediction,
    construct_replay_row,
    compute_aggregate_metrics
)
from scripts.run_rp608_nse_shadow_daemon import calc_indian_equity_friction


class TestRP608TimingExperiment(unittest.TestCase):

    def test_01_timestamp_alignment_and_horizon_eligibility(self):
        """Verifies exact 15m candle close timestamps and horizon-aware runway eligibility."""
        session_date = "2026-10-26"

        # Bar 1: [09:15, 09:30 IST), closes at 09:30:00 IST -> 345 min available
        b1 = get_bar_interval_info(1, session_date)
        self.assertEqual(b1["interval_span_ist"], "[09:15, 09:30 IST)")
        self.assertEqual(b1["available_minutes_to_squareoff"], 345)
        self.assertTrue(is_horizon_eligible(1, 60)[0])
        self.assertTrue(is_horizon_eligible(1, 7)[0])

        # Bar 19: [13:45, 14:00 IST), closes at 14:00:00 IST -> 75 min available
        b19 = get_bar_interval_info(19, session_date)
        self.assertEqual(b19["interval_span_ist"], "[13:45, 14:00 IST)")
        self.assertEqual(b19["available_minutes_to_squareoff"], 75)
        self.assertTrue(is_horizon_eligible(19, 60)[0])

        # Bar 20: [14:00, 14:15 IST), closes at 14:15:00 IST -> 60 min available (exact boundary for 60m)
        b20 = get_bar_interval_info(20, session_date)
        self.assertEqual(b20["available_minutes_to_squareoff"], 60)
        self.assertTrue(is_horizon_eligible(20, 60)[0])
        self.assertTrue(is_horizon_eligible(20, 30)[0])

        # Bar 21: [14:15, 14:30 IST), closes at 14:30:00 IST -> 45 min available
        b21 = get_bar_interval_info(21, session_date)
        self.assertEqual(b21["available_minutes_to_squareoff"], 45)
        self.assertFalse(is_horizon_eligible(21, 60)[0])  # 60m requires 60m > 45m -> Excluded
        self.assertTrue(is_horizon_eligible(21, 30)[0])   # 30m eligible
        self.assertTrue(is_horizon_eligible(21, 15)[0])   # 15m eligible
        self.assertTrue(is_horizon_eligible(21, 7)[0])    # 7m eligible

        # Bar 22: [14:30, 14:45 IST), closes at 14:45:00 IST -> 30 min available (exact boundary for 30m)
        b22 = get_bar_interval_info(22, session_date)
        self.assertEqual(b22["available_minutes_to_squareoff"], 30)
        self.assertFalse(is_horizon_eligible(22, 60)[0])
        self.assertTrue(is_horizon_eligible(22, 30)[0])
        self.assertTrue(is_horizon_eligible(22, 15)[0])

        # Bar 23: [14:45, 15:00 IST), closes at 15:00:00 IST -> 15 min available (exact boundary for 15m)
        b23 = get_bar_interval_info(23, session_date)
        self.assertEqual(b23["available_minutes_to_squareoff"], 15)
        self.assertFalse(is_horizon_eligible(23, 60)[0])
        self.assertFalse(is_horizon_eligible(23, 30)[0])
        self.assertTrue(is_horizon_eligible(23, 15)[0])
        self.assertTrue(is_horizon_eligible(23, 7)[0])

        # Bar 24: [15:00, 15:15 IST), closes at 15:15:00 IST -> 0 min available (mandatory square-off)
        b24 = get_bar_interval_info(24, session_date)
        self.assertEqual(b24["available_minutes_to_squareoff"], 0)
        self.assertFalse(is_horizon_eligible(24, 7)[0])
        self.assertFalse(is_horizon_eligible(24, 15)[0])

    def test_02_synthetic_outcome_fixtures(self):
        """Tests target, stop, conservative ambiguity, timeout, square-off, and data failures."""
        session_date = "2026-10-26"
        entry_p = 1000.0
        # Entry at 10:00 IST (UTC 04:30)
        t_entry = int(datetime(2026, 10, 26, 4, 30, tzinfo=timezone.utc).timestamp() * 1000)

        # 1. Target Hit Only: Bid rises to 1016.0 (+1.6% > +1.5% target)
        q_target = [{"timestamp_utc_ms": t_entry + 60000, "bid_price": 1016.0, "ask_price": 1017.0}]
        res_tp = evaluate_lifecycle_trade(entry_p, t_entry, 30, session_date, q_target)
        self.assertEqual(res_tp["status"], "TARGET_HIT")
        self.assertEqual(res_tp["exit_price"], 1015.0)  # Filled at target price
        # Target gross is +150 bps, friction is ~17.35 bps -> net is ~132.65 bps
        self.assertGreater(res_tp["net_return_bps"], 130.0)
        self.assertLess(res_tp["net_return_bps"], 135.0)

        # 2. Stop Loss Hit Only: Bid drops to 989.0 (-1.1% < -1.0% stop)
        q_stop = [{"timestamp_utc_ms": t_entry + 60000, "bid_price": 989.0, "ask_price": 990.0}]
        res_sl = evaluate_lifecycle_trade(entry_p, t_entry, 30, session_date, q_stop)
        self.assertEqual(res_sl["status"], "STOP_LOSS_HIT")
        self.assertEqual(res_sl["exit_price"], 990.0)  # Filled at stop price
        # Stop turnover is ₹199,000; friction recomputed dynamically to ~17.17 bps (not target's 17.35 bps)
        self.assertAlmostEqual(res_sl["statutory_friction_bps"], 17.17, places=1)
        self.assertAlmostEqual(res_sl["net_return_bps"], -117.17, places=1)

        # 3. Both Crossed in Same Interval: Conservative resolution resolves as STOP_LOSS_HIT
        q_both = [{"timestamp_utc_ms": t_entry + 60000, "bid_price": 980.0, "ask_price": 1020.0}]
        res_both = evaluate_lifecycle_trade(entry_p, t_entry, 30, session_date, q_both)
        self.assertEqual(res_both["status"], "STOP_LOSS_HIT")

        # 4. Max Holding Expired: 15m horizon elapses at 10:15 without hitting target/stop
        q_timeout = [{"timestamp_utc_ms": t_entry + 901000, "bid_price": 1005.0, "ask_price": 1006.0}]
        res_timeout = evaluate_lifecycle_trade(entry_p, t_entry, 15, session_date, q_timeout)
        self.assertEqual(res_timeout["status"], "MAX_HOLDING_EXPIRED")
        self.assertEqual(res_timeout["exit_price"], 1005.0)

        # 5. Mandatory Square-Off: Trade at 14:45 with 60m horizon reaches 15:15 IST
        t_entry_late = int(datetime(2026, 10, 26, 9, 15, tzinfo=timezone.utc).timestamp() * 1000)  # 14:45 IST
        t_sqoff = int(datetime(2026, 10, 26, 9, 45, tzinfo=timezone.utc).timestamp() * 1000)      # 15:15 IST
        q_sqoff = [{"timestamp_utc_ms": t_sqoff, "bid_price": 1002.0, "ask_price": 1003.0}]
        res_sqoff = evaluate_lifecycle_trade(entry_p, t_entry_late, 60, session_date, q_sqoff)
        self.assertEqual(res_sqoff["status"], "MANDATORY_INTRADAY_SQUARE_OFF")
        self.assertEqual(res_sqoff["exit_price"], 1002.0)

        # 6. Data Quality Failure: No quotes provided
        res_missing = evaluate_lifecycle_trade(entry_p, t_entry, 15, session_date, [])
        self.assertEqual(res_missing["status"], "DATA_QUALITY_UNRESOLVED")
        self.assertIsNone(res_missing["exit_price"])

    def test_03_hand_worked_statutory_friction_reconciliation(self):
        """Verifies exact 8-component friction calculation against hand-worked arithmetic."""
        # Buy ₹100,000 (100 shares @ ₹1000), Sell ₹101,500 (100 shares @ ₹1015)
        # Total turnover: ₹201,500
        buy_p = 1000.0
        sell_p = 1015.0
        qty = 100.0
        spread_half_bps = 2.5

        fric = calc_indian_equity_friction(buy_p, sell_p, qty, half_spread_bps=spread_half_bps)

        # 1. Brokerage: min(20, 100000 * 0.0003 = 30) = 20 buy + 20 sell = 40.0
        self.assertEqual(fric["brokerage"], 40.0)
        # 2. STT: 101500 * 0.00025 = 25.375
        self.assertEqual(fric["stt"], 25.375)
        # 3. Exchange: 201500 * 0.0000297 = 5.9846
        self.assertAlmostEqual(fric["exchange_charges"], 5.9846, places=3)
        # 4. SEBI: 201500 * 0.000001 = 0.2015
        self.assertAlmostEqual(fric["sebi_charges"], 0.2015, places=3)
        # 5. GST: 0.18 * (40 + 5.98455 + 0.2015) = 8.3135
        self.assertAlmostEqual(fric["gst"], 8.3135, places=3)
        # 6. Stamp duty: 100000 * 0.00003 = 3.0
        self.assertEqual(fric["stamp_duty"], 3.0)
        # 7. Spread: 201500 * (2.5 / 10000) = 50.375
        self.assertEqual(fric["spread_cost"], 50.375)
        # 8. Slippage: 201500 * (2.0 / 10000) = 40.30
        self.assertEqual(fric["slippage_cost"], 40.3)

        expected_total = 40.0 + 25.375 + 5.98455 + 0.2015 + 8.3135 + 3.0 + 50.375 + 40.3
        self.assertAlmostEqual(fric["total_friction"], expected_total, places=2)
        # Effective friction bps: (total / 100000) * 10000 ~ 17.35 bps
        self.assertGreater(fric["effective_friction_bps"], 15.0)
        self.assertLess(fric["effective_friction_bps"], 20.0)

    def test_04_determinism_and_reproducibility(self):
        """Verifies that identical input data produces identical SHA-256 JSON output."""
        session_date = "2026-10-26"
        t_entry = int(datetime(2026, 10, 26, 5, 0, tzinfo=timezone.utc).timestamp() * 1000)
        quotes = [
            {"timestamp_utc_ms": t_entry + 60000, "bid_price": 1005.0, "ask_price": 1006.0},
            {"timestamp_utc_ms": t_entry + 120000, "bid_price": 1016.0, "ask_price": 1017.0}
        ]

        row1 = construct_replay_row(
            bar_idx=5,
            session_date_iso=session_date,
            symbol="TCS.NS",
            decision_price=1000.0,
            traj15=0.082,
            history_length=6,
            horizon_minutes=30,
            subsequent_quotes=quotes
        )

        row2 = construct_replay_row(
            bar_idx=5,
            session_date_iso=session_date,
            symbol="TCS.NS",
            decision_price=1000.0,
            traj15=0.082,
            history_length=6,
            horizon_minutes=30,
            subsequent_quotes=quotes
        )

        h1 = hashlib.sha256(json.dumps(row1, sort_keys=True).encode("utf-8")).hexdigest()
        h2 = hashlib.sha256(json.dumps(row2, sort_keys=True).encode("utf-8")).hexdigest()
        self.assertEqual(h1, h2)
        self.assertEqual(row1["estimand_a"]["status"], "TARGET_HIT")

    def test_05_decoupled_estimands_and_non_censoring_independence(self):
        """Verifies that Estimand A resolves cleanly even when Estimand B is right-censored."""
        session_date = "2026-10-26"
        # Bar 21 close: 14:30 IST (09:00 UTC)
        t_bar21 = int(datetime(2026, 10, 26, 9, 0, tzinfo=timezone.utc).timestamp() * 1000)

        # Quotes: Target hit at 14:50 IST (09:20 UTC, 20m later)
        quotes = [
            {"timestamp_utc_ms": t_bar21 + 1200000, "bid_price": 1016.0, "ask_price": 1017.0}
        ]

        # For a 30m lifecycle trade, target is hit at +20m
        est_a = evaluate_lifecycle_trade(1000.0, t_bar21, 30, session_date, quotes)
        self.assertEqual(est_a["status"], "TARGET_HIT")
        self.assertEqual(est_a["exit_price"], 1015.0)

        # For 60m directional prediction, 14:30 + 60m = 15:30:00 (exact close)
        # But if bar 22 (14:45 IST, 09:15 UTC): 14:45 + 60m = 15:45 > 15:30 -> RIGHT_CENSORED
        t_bar22 = int(datetime(2026, 10, 26, 9, 15, tzinfo=timezone.utc).timestamp() * 1000)
        est_b_censored = evaluate_directional_prediction(1000.0, t_bar22, 60, session_date, quotes)
        self.assertEqual(est_b_censored["status"], "RIGHT_CENSORED_INCOMPLETE_HORIZON")
        self.assertIsNone(est_b_censored["forward_return_bps"])

        # Replay row at Bar 22 with 30m horizon:
        # Estimand A enters and hits target, while Estimand B (60m) is right-censored
        row = construct_replay_row(
            bar_idx=22,
            session_date_iso=session_date,
            symbol="INFY.NS",
            decision_price=1000.0,
            traj15=0.075,
            history_length=10,
            horizon_minutes=30,
            subsequent_quotes=[
                {"timestamp_utc_ms": t_bar22 + 600000, "bid_price": 1016.0, "ask_price": 1017.0},
                {"timestamp_utc_ms": t_bar22 + 1800000, "bid_price": 1018.0, "ask_price": 1019.0}
            ]
        )
        # Invariant: Estimand A trade is NOT discarded; resolves as TARGET_HIT
        self.assertTrue(row["entry_eligible"])
        self.assertEqual(row["estimand_a"]["status"], "TARGET_HIT")
        self.assertEqual(row["estimand_b"]["status"], "EVALUATED")  # 30m from 14:45 is 15:15 <= 15:30


if __name__ == "__main__":
    unittest.main()
