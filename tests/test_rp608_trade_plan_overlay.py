#!/usr/bin/env python3
"""
tests/test_rp608_trade_plan_overlay.py

Deterministic Unit Tests for RP-608 Trade-Plan Overlay v1 (Gate B).
Fully Isolated: Uses temporary directories to prevent ledger contamination.
Validates:
  1. Long levels calculation & rounding
  2. Short levels calculation & rounding
  3. Gross vs Net Reward/Risk & friction arithmetic
  4. Stop-loss trigger evaluation
  5. Profit-target trigger evaluation
  6. Ambiguous intra-observation tie-breaking
  7. 60-minute holding horizon time-exit
  8. Concurrency: 1 plan max & signal suppression
  9. Strictly 0.0% Order Authority invariant
"""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import tempfile
import time
import unittest
from unittest import mock

from scripts.rp608_trade_plan_overlay import (
    calculate_trade_plan,
    round_tick,
    TradePlanOverlayManager,
    CONFIG
)


class TestTradePlanOverlay(unittest.TestCase):

    def setUp(self):
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.test_overlay_dir = Path(self.tmp_dir.name)

        self.sample_card_short = {
            "bar_idx": 21,
            "timestamp_utc": "2026-10-26T00:44:59.999Z",
            "close_time_ms": 1792975499999,
            "reference_price": 80170.00,
            "atr_14": 264.43,
            "policy_b_action": "SHORT",
            "policy_c_action": "SHORT",
            "execution_quote": {
                "best_bid_price": 80170.00,
                "best_ask_price": 80170.01,
                "exchange_event_time_ms": 1792975500000
            }
        }

        self.sample_card_long = {
            "bar_idx": 20,
            "timestamp_utc": "2026-10-26T00:29:59.999Z",
            "close_time_ms": 1792974599999,
            "reference_price": 80272.85,
            "atr_14": 248.12,
            "policy_b_action": "LONG",
            "policy_c_action": "NO_TRADE",
            "execution_quote": {
                "best_bid_price": 80272.85,
                "best_ask_price": 80272.86,
                "exchange_event_time_ms": 1792974600000
            }
        }

    def tearDown(self):
        self.tmp_dir.cleanup()

    def test_01_short_trade_plan_levels(self):
        """Validates short entry at bid, SL at E + 1.0*ATR, TP at E - 1.5*ATR."""
        plan = calculate_trade_plan(self.sample_card_short)
        self.assertIsNotNone(plan)
        self.assertEqual(plan["direction"], "SHORT")
        self.assertEqual(plan["entry_price"], 80170.00)
        
        self.assertEqual(plan["stop_loss_price"], 80434.43)
        expected_tp = round_tick(80170.00 - (1.5 * 264.43))
        self.assertEqual(plan["profit_target_price"], expected_tp)
        self.assertEqual(plan["gross_rr_ratio"], 1.50)

    def test_02_long_trade_plan_levels(self):
        """Validates long entry at ask, SL at E - 1.0*ATR, TP at E + 1.5*ATR."""
        plan = calculate_trade_plan(self.sample_card_long)
        self.assertIsNotNone(plan)
        self.assertEqual(plan["direction"], "LONG")
        self.assertEqual(plan["entry_price"], 80272.86)
        
        expected_sl = round_tick(80272.86 - 248.12)
        expected_tp = round_tick(80272.86 + (1.5 * 248.12))
        self.assertEqual(plan["stop_loss_price"], expected_sl)
        self.assertEqual(plan["profit_target_price"], expected_tp)
        self.assertEqual(plan["gross_rr_ratio"], 1.50)

    def test_03_friction_and_net_reward_risk(self):
        """Verifies 12.0 bps round-trip friction and net R:R degradation."""
        plan = calculate_trade_plan(self.sample_card_short)
        self.assertIsNotNone(plan)
        self.assertEqual(plan["round_trip_friction_bps"], 12.0)
        self.assertGreater(plan["est_friction_usdt"], 0.0)
        
        self.assertLess(plan["net_target_profit_usdt"], plan["gross_reward_usdt"])
        self.assertGreater(plan["net_stop_loss_usdt"], plan["gross_risk_usdt"])
        self.assertLess(plan["net_rr_ratio"], 1.50)
        self.assertGreater(plan["net_rr_ratio"], 0.50)

    def test_04_target_hit_resolution(self):
        """Verifies simulated exit when profit target is hit."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        mgr.state["active_plan"] = plan
        mgr._save_state()

        quote_tp = {"best_bid_price": 79769.99, "best_ask_price": 79770.00}
        resolved = mgr.evaluate_market_tick(quote_tp)
        
        self.assertIsNotNone(resolved)
        self.assertEqual(resolved["state"], "CLOSED_TP")
        self.assertGreater(resolved["net_pnl_usdt"], 0.0)
        self.assertIsNone(mgr.state["active_plan"])

    def test_05_stop_loss_hit_resolution(self):
        """Verifies simulated exit when stop loss is hit."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        mgr.state["active_plan"] = plan
        mgr._save_state()

        quote_sl = {"best_bid_price": 80439.99, "best_ask_price": 80440.00}
        resolved = mgr.evaluate_market_tick(quote_sl)
        
        self.assertIsNotNone(resolved)
        self.assertEqual(resolved["state"], "CLOSED_SL")
        self.assertLess(resolved["net_pnl_usdt"], 0.0)
        self.assertIsNone(mgr.state["active_plan"])

    def test_06_ambiguous_crossing_tie_break(self):
        """Verifies ambiguous crossing resolves conservatively to CLOSED_SL."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        mgr.state["active_plan"] = plan
        mgr._save_state()

        quote_ambig = {"best_bid_price": 79700.00, "best_ask_price": 80500.00}
        resolved = mgr.evaluate_market_tick(quote_ambig)
        
        self.assertIsNotNone(resolved)
        self.assertEqual(resolved["state"], "CLOSED_SL")
        self.assertEqual(resolved["exit_reason"], "AMBIGUOUS_STOP_TARGET")

    def test_07_60_minute_time_exit(self):
        """Verifies time exit triggers when 3600 seconds elapse."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        plan["expiry_time_ms"] = int((time.time() - 10) * 1000)
        mgr.state["active_plan"] = plan
        mgr._save_state()

        quote_normal = {"best_bid_price": 80160.00, "best_ask_price": 80160.01}
        resolved = mgr.evaluate_market_tick(quote_normal)
        
        self.assertIsNotNone(resolved)
        self.assertEqual(resolved["state"], "TIME_EXIT")
        self.assertEqual(resolved["exit_reason"], "MAX_HORIZON_60M_EXPIRED")

    def test_08_concurrency_single_plan_limit(self):
        """Verifies concurrency limit of 1 open plan and signal suppression."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        mgr.state["active_plan"] = None
        mgr.state["suppressed_signals_count"] = 0

        plan1 = mgr.process_decision_card(self.sample_card_short)
        self.assertIsNotNone(plan1)
        self.assertEqual(plan1["state"], "OPEN")

        plan2 = mgr.process_decision_card(self.sample_card_long)
        self.assertEqual(plan2["plan_id"], plan1["plan_id"])
        self.assertEqual(mgr.state["suppressed_signals_count"], 1)

    def test_09_zero_order_authority_invariant(self):
        """Guarantees order authority is strictly 0.0%."""
        self.assertEqual(CONFIG["order_authority_pct"], 0.0)
        plan = calculate_trade_plan(self.sample_card_short)
        self.assertIn("0.0%", plan["order_authority"])

    def test_10_expired_open_plan_case(self):
        """Tests that sync_lifecycle immediately transitions an expired open plan to TIME_EXIT."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        # Set expiry in the past
        plan["expiry_time_ms"] = 1700000000000
        mgr.state["active_plan"] = plan
        mgr._save_state()

        # Current quote at expiry time — disable live REST so test stays deterministic
        q = {"best_bid_price": 80100.00, "best_ask_price": 80100.01}
        with mock.patch("scripts.rp608_trade_plan_overlay.fetch_live_btcusdt_book", return_value=None):
            status = mgr.sync_lifecycle(current_quote=q, now_ms=1700000001000)

        self.assertFalse(status["is_active"])
        self.assertEqual(status["state"], "TIME_EXIT")
        self.assertIsNone(mgr.state["active_plan"])
        self.assertIsNotNone(mgr.state.get("last_closed_plan"))
        self.assertEqual(mgr.state["last_closed_plan"]["state"], "TIME_EXIT")

    def test_12_short_target_hit_via_live_book_not_stale_card(self):
        """
        Regression: plan must close on live ask <= TP even when the decision-card
        execution quote is still near entry (the dashboard bug that left TP open).
        """
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        self.assertIsNotNone(plan)
        tp = plan["profit_target_price"]
        mgr.state["active_plan"] = plan
        mgr._save_state()

        stale_card_quote = {
            "best_bid_price": 80170.00,
            "best_ask_price": 80170.01,
        }  # near entry — would NOT hit TP alone
        live_past_tp = {
            "best_bid_price": tp - 50.0,
            "best_ask_price": tp - 49.0,
            "quote_source": "binance_rest_bookTicker",
        }
        with mock.patch(
            "scripts.rp608_trade_plan_overlay.fetch_live_btcusdt_book",
            return_value=live_past_tp,
        ):
            status = mgr.sync_lifecycle(current_quote=stale_card_quote, now_ms=int(time.time() * 1000))

        self.assertFalse(status["is_active"])
        self.assertEqual(status["state"], "CLOSED_TP")
        self.assertEqual(status["display_plan"]["exit_reason"], "PROFIT_TARGET_HIT")
        self.assertIsNone(mgr.state["active_plan"])
        self.assertGreater(status["display_plan"]["net_pnl_usdt"], 0.0)

    def test_13_hit_then_reverse_inside_15m_still_closed(self):
        """
        Target touch must latch immediately; a reverse before the next 15m close
        must not leave the plan OPEN or unwind the TP fill.
        """
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        tp = plan["profit_target_price"]
        mgr.state["active_plan"] = plan
        mgr._save_state()
        t0 = int(time.time() * 1000)

        # Mid-bar: ask prints through TP
        hit = mgr.evaluate_market_tick(
            {
                "best_bid_price": tp - 20.0,
                "best_ask_price": tp - 19.0,
                "quote_source": "binance_ws_bookTicker",
            },
            reload_state=False,
            now_ms=t0 + 30_000,
        )
        self.assertEqual(hit["state"], "CLOSED_TP")
        self.assertIsNone(mgr.state["active_plan"])
        closed_id = hit["plan_id"]

        # Same 15m window: price reverses violently above entry — plan stays closed
        again = mgr.evaluate_market_tick(
            {
                "best_bid_price": 80500.0,
                "best_ask_price": 80500.1,
                "quote_source": "binance_ws_bookTicker",
            },
            reload_state=False,
            now_ms=t0 + 120_000,
        )
        self.assertIsNone(again)
        self.assertIsNone(mgr.state["active_plan"])
        self.assertEqual(mgr.state["last_closed_plan"]["plan_id"], closed_id)
        self.assertEqual(mgr.state["last_closed_plan"]["state"], "CLOSED_TP")

    def test_14_forming_kline_path_through_closes_before_bar_close(self):
        """Forming 15m low/high path-through closes even if L1 ask has already bounced."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan = calculate_trade_plan(self.sample_card_short)
        tp = plan["profit_target_price"]
        mgr.state["active_plan"] = plan
        mgr._save_state()

        # L1 ask already reversed above TP, but forming candle low printed through TP
        resolved = mgr.evaluate_market_tick(
            {
                "best_bid_price": plan["entry_price"] - 10.0,
                "best_ask_price": plan["entry_price"] - 9.0,  # not through TP
                "low_price": tp - 5.0,  # path-through
                "high_price": plan["entry_price"] + 5.0,
                "quote_source": "binance_ws_forming_kline_15m",
            },
            reload_state=False,
        )
        self.assertIsNotNone(resolved)
        self.assertEqual(resolved["state"], "CLOSED_TP")
        self.assertEqual(resolved["exit_reason"], "PROFIT_TARGET_HIT")

    def test_11_executable_side_mark_to_market_valuation(self):
        """Verifies executable exit side quote and net P&L after friction."""
        mgr = TradePlanOverlayManager(base_dir=self.test_overlay_dir)
        plan_long = calculate_trade_plan(self.sample_card_long)
        # Entry price: 80272.86, qty: 0.012458
        entry_p = plan_long["entry_price"]
        qty = plan_long["simulated_quantity"]

        # Market quote: bid = 80400.00, ask = 80400.01
        # For LONG, executable exit is bid = 80400.00
        quote_market = {"best_bid_price": 80400.00, "best_ask_price": 80400.01}
        val = mgr.compute_mtm_valuation(plan_long, quote_market, now_ms=plan_long["entry_time_ms"] + 60000)

        self.assertEqual(val["entry_price"], entry_p)
        self.assertEqual(val["current_bid"], 80400.00)
        self.assertEqual(val["current_ask"], 80400.01)
        self.assertEqual(val["executable_exit_price"], 80400.00)
        
        expected_gross = (80400.00 - entry_p) * qty
        expected_friction = (1000.0 + (80400.00 * qty)) * (6.0 / 10000.0)
        expected_net = expected_gross - expected_friction
        self.assertAlmostEqual(val["unrealized_gross_pnl_usdt"], expected_gross, places=2)
        self.assertAlmostEqual(val["unrealized_net_pnl_usdt"], expected_net, places=2)
        self.assertFalse(val["is_expired"])
        self.assertGreater(val["time_remaining_seconds"], 0)


if __name__ == "__main__":
    unittest.main()

