"""Order-flow mark + reverse-flow exit (not time-based)."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.rp608_order_flow_mark import (
    ORDER_FLOW_EXIT_LABEL,
    ORDER_FLOW_LABEL,
    classify_order_flow_support,
    inspect_order_flow_book,
    is_order_flow_exit_signal,
    l1_from_nse_quote,
)
from scripts.rp608_trade_plan_overlay import (
    CONFIG,
    OF_DEPTH_QUALITY_PROTOCOL_ID,
    OF_ENTRY_QUALITY_PROTOCOL_ID,
    OF_PAPER_PROTOCOL_ID,
    TradePlanOverlayManager,
    calculate_trade_plan,
    compute_of_depth_quality,
    compute_of_paper_cost_breakdown,
    of_entry_signal_qty,
    of_exit_signal_qty,
    size_of_paper_from_entry_signal,
)


class TestOrderFlowExitDriven(unittest.TestCase):
    def setUp(self):
        # Existing suite expects immediate paper open; entry persist covered separately.
        self._prev_entry_persist = CONFIG.get("of_paper_entry_persist_ms")
        CONFIG["of_paper_entry_persist_ms"] = 0

    def tearDown(self):
        CONFIG["of_paper_entry_persist_ms"] = self._prev_entry_persist

    def test_exit_only_on_opposite_confirm(self):
        long_l1 = {"bid_qty": 90, "ask_qty": 10, "source": "t"}
        short_l1 = {"bid_qty": 10, "ask_qty": 90, "source": "t"}
        neutral = {"bid_qty": 50, "ask_qty": 50, "source": "t"}

        self.assertTrue(classify_order_flow_support("LONG", long_l1)["order_flow_supported"])
        # Neutral does NOT exit LONG OF
        self.assertFalse(is_order_flow_exit_signal("LONG", neutral)["exit"])
        # Same-side stronger does NOT exit
        self.assertFalse(is_order_flow_exit_signal("LONG", long_l1)["exit"])
        # Opposite confirm exits
        ex = is_order_flow_exit_signal("LONG", short_l1)
        self.assertTrue(ex["exit"])
        self.assertEqual(ex["order_flow_exit_label"], ORDER_FLOW_EXIT_LABEL)
        self.assertEqual(ex["opposite_side"], "SHORT")

    def test_overlay_of_survives_without_60m_and_exits_on_reverse(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            card = {
                "bar_idx": 100,
                "decision_event_idx": 100,
                "timestamp_utc": "2026-11-01T00:00:00Z",
                "close_time_ms": 1793500000000,
                "reference_price": 100.0,
                "atr_14": 1.0,
                "policy_b_action": "LONG",
                "order_flow_supported": True,
                "order_flow_label": ORDER_FLOW_LABEL,
                "order_flow_imbalance": 0.8,
                "order_flow_source": "test",
                "execution_quote": {
                    "best_bid_price": 99.9,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 90,
                    "best_ask_qty": 10,
                    "exchange_event_time_ms": 1793500000000,
                },
            }
            plan = mgr.process_decision_card(card)
            self.assertIsNotNone(plan)
            of = mgr.state.get("active_order_flow_recommendation")
            self.assertTrue(of and of.get("active"))
            self.assertEqual(of["direction"], "LONG")
            self.assertNotIn("active_until_ms", of)  # not time-gated
            self.assertIsNotNone(of.get("entry_candle"))
            self.assertEqual(of["entry_candle"].get("bar_idx"), 100)
            self.assertEqual(of["entry_candle"].get("decision_event_idx"), 100)

            # Neutral book: still active (past 60m plan wall — OF must remain)
            still = mgr.evaluate_order_flow_tick(
                {"best_bid_price": 100.0, "best_ask_price": 100.1, "best_bid_qty": 50, "best_ask_qty": 50},
                now_ms=1793500000000 + 3_600_000 + 1,
                persist=True,
            )
            self.assertIsNotNone(still)
            self.assertTrue(still.get("active"))

            # Reverse flow after min hold + opposite persist (anti-churn)
            arm_ms = 1793500000000 + 3_600_000 + 2
            armed = mgr.evaluate_order_flow_tick(
                {"best_bid_price": 100.0, "best_ask_price": 100.1, "best_bid_qty": 10, "best_ask_qty": 90},
                now_ms=arm_ms,
                persist=True,
                candle_ctx={"bar_idx": 104, "decision_event_idx": 999, "close_time_ms": arm_ms},
            )
            self.assertIsNotNone(armed)
            self.assertTrue(armed.get("active"))
            exit_ms = arm_ms + 30_000  # opposite OF must persist 30s
            gone = mgr.evaluate_order_flow_tick(
                {"best_bid_price": 100.0, "best_ask_price": 100.1, "best_bid_qty": 10, "best_ask_qty": 90},
                now_ms=exit_ms,
                persist=True,
                candle_ctx={"bar_idx": 104, "decision_event_idx": 999, "close_time_ms": exit_ms},
            )
            self.assertIsNone(gone)
            self.assertIsNone(mgr.state.get("active_order_flow_recommendation"))
            closed = mgr.state.get("last_closed_order_flow_recommendation")
            self.assertEqual(closed.get("exit_reason"), "OPPOSITE_SIDE_ORDER_FLOW")
            self.assertEqual((closed.get("exit_candle") or {}).get("bar_idx"), 104)
            self.assertEqual((closed.get("exit_candle") or {}).get("decision_event_idx"), 999)
            # PAPER OF P&L ledger (independent of 60m plan)
            self.assertIsNotNone(closed.get("exit_price"))
            self.assertIsNotNone(closed.get("net_pnl_usdt"))
            self.assertEqual(closed.get("execution_mode"), "PAPER")
            self.assertEqual(closed.get("live_order_authority"), 0.0)
            self.assertEqual(closed.get("protocol_id"), OF_PAPER_PROTOCOL_ID)
            self.assertIn("signal_exit_ms", closed)
            self.assertIn("paper_close_ms", closed)
            self.assertEqual(closed.get("paper_close_lag_ms"), 0)
            self.assertIn("spread_cost_usdt", closed)
            self.assertIn("fees_usdt", closed)
            self.assertIn("slippage_usdt", closed)
            self.assertEqual(mgr.state["of_outcomes_summary"]["n_trades"], 1)
            recent = mgr.load_recent_of_outcomes(5)
            self.assertEqual(len(recent), 1)
            self.assertEqual(recent[0]["exit_reason"], "OPPOSITE_SIDE_ORDER_FLOW")
            self.assertIn("net_return_bps", recent[0])
            self.assertEqual(recent[0]["execution_mode"], "PAPER")
            self.assertEqual(recent[0]["live_order_authority"], 0.0)


class TestOfPaperTradeObservability(unittest.TestCase):
    def setUp(self):
        self._prev_entry_persist = CONFIG.get("of_paper_entry_persist_ms")
        CONFIG["of_paper_entry_persist_ms"] = 0

    def tearDown(self):
        CONFIG["of_paper_entry_persist_ms"] = self._prev_entry_persist

    def _open_long(self, mgr: TradePlanOverlayManager) -> dict:
        card = {
            "bar_idx": 100,
            "decision_event_idx": 100,
            "timestamp_utc": "2026-11-01T00:00:00Z",
            "close_time_ms": 1793500000000,
            "reference_price": 100.0,
            "atr_14": 1.0,
            "policy_b_action": "LONG",
            "order_flow_supported": True,
            "order_flow_label": ORDER_FLOW_LABEL,
            "order_flow_imbalance": 0.8,
            "order_flow_source": "test",
            "execution_quote": {
                "best_bid_price": 99.9,
                "best_ask_price": 100.1,
                "best_bid_qty": 90,
                "best_ask_qty": 10,
                "exchange_event_time_ms": 1793500000000,
            },
        }
        mgr.process_decision_card(card)
        of = mgr.state.get("active_order_flow_recommendation")
        assert of and of.get("active")
        return of

    def test_open_paper_trade_identity_and_no_plan_geometry(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of = self._open_long(mgr)
            self.assertTrue(str(of["of_trade_id"]).startswith("OF-PAPER-"))
            self.assertEqual(of["execution_mode"], "PAPER")
            self.assertEqual(of["live_order_authority"], 0.0)
            self.assertEqual(of["protocol_id"], OF_PAPER_PROTOCOL_ID)
            self.assertTrue(of["independent_of_trade_plan_ttl"])
            self.assertIsNone(of.get("stop_loss_price"))
            self.assertIsNone(of.get("profit_target_price"))
            self.assertIsNone(of.get("expiry_time_ms"))
            self.assertTrue((Path(td) / "of_paper_trades_open.json").exists())
            self.assertTrue((Path(td) / "of_paper_trade_opens.jsonl").exists())
            snap = mgr.build_of_shadow_trade_snapshot(
                {
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.2,
                    "best_bid_qty": 80,
                    "best_ask_qty": 20,
                },
                now_ms=1793500000000 + 60_000,
            )
            self.assertIsNotNone(snap)
            self.assertTrue(snap["open"])
            self.assertEqual(snap["of_trade_id"], of["of_trade_id"])
            self.assertEqual(snap["live_order_authority"], 0.0)
            self.assertIn("unrealized_net_pnl_usdt", snap)
            self.assertIn("spread_cost_usdt", snap)
            self.assertIn("fees_usdt", snap)

    def test_same_side_refresh_does_not_duplicate_id(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of1 = self._open_long(mgr)
            tid = of1["of_trade_id"]
            mgr._activate_order_flow_recommendation(
                direction="LONG",
                of={"order_flow_imbalance": 0.7, "order_flow_source": "test"},
                plan_id="P2",
                entry_price=100.2,
                now_ms=1793500000000 + 900_000,
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.2,
                    "best_bid_qty": 85,
                    "best_ask_qty": 15,
                },
            )
            of2 = mgr.state["active_order_flow_recommendation"]
            self.assertEqual(of2["of_trade_id"], tid)

    def test_plan_tp_does_not_close_of_paper_trade(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            self._open_long(mgr)
            tid = mgr.state["active_order_flow_recommendation"]["of_trade_id"]
            mgr.state["active_plan"] = {
                "state": "OPEN",
                "plan_id": "PLAN-X",
                "direction": "LONG",
                "entry_price": 100.1,
                "stop_loss_price": 99.0,
                "profit_target_price": 100.05,
                "simulated_quantity": 10,
                "simulated_notional_usdt": 1000.0,
                "entry_time_ms": 1793500000000,
                "expiry_time_ms": 1793500000000 + 3_600_000,
            }
            # Hit TP on plan; OF must remain (neutral book — no reverse OF)
            mgr.evaluate_market_tick(
                {
                    "best_bid_price": 100.2,
                    "best_ask_price": 100.3,
                    "best_bid_qty": 50,
                    "best_ask_qty": 50,
                },
                now_ms=1793500000000 + 10_000,
            )
            of = mgr.state.get("active_order_flow_recommendation")
            self.assertTrue(of and of.get("active"))
            self.assertEqual(of["of_trade_id"], tid)

    def test_min_hold_suppresses_instant_reverse_exit(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of = self._open_long(mgr)
            tid = of["of_trade_id"]
            t0 = int(of["activated_ms"])
            # Immediate reverse OF — must NOT close (min hold 5m)
            still = mgr.evaluate_order_flow_tick(
                {
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 10,
                    "best_ask_qty": 90,
                },
                now_ms=t0 + 1_000,
                persist=True,
            )
            self.assertIsNotNone(still)
            self.assertTrue(still.get("active"))
            self.assertEqual(still.get("of_trade_id"), tid)
            self.assertIn("close_suppressed_reason", still)
            self.assertEqual(mgr.state["of_outcomes_summary"]["n_trades"], 0)
            # Still suppressed just under 5m hold
            still2 = mgr.evaluate_order_flow_tick(
                {
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 10,
                    "best_ask_qty": 90,
                },
                now_ms=t0 + 299_000,
                persist=True,
            )
            self.assertTrue(still2 and still2.get("active"))
            self.assertIn("min_hold", str(still2.get("close_suppressed_reason") or ""))

    def test_one_paper_trade_per_15m_bar(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of1 = self._open_long(mgr)
            # Force-close past min hold (5m) + persist (30s)
            t0 = int(of1["activated_ms"])
            mgr.state["active_order_flow_recommendation"]["opposite_exit_seen_ms"] = t0 + 300_000
            mgr._clear_order_flow_recommendation(
                reason="OPPOSITE_SIDE_ORDER_FLOW",
                exit_meta={
                    "exit": True,
                    "opposite_side": "SHORT",
                    "order_flow_imbalance": -0.8,
                },
                now_ms=t0 + 300_000 + 30_000,
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 10,
                    "best_ask_qty": 90,
                },
            )
            self.assertIsNone(mgr.state.get("active_order_flow_recommendation"))
            # Same bar_idx 100 — refuse re-open
            again = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of={"order_flow_imbalance": 0.8, "order_flow_source": "test"},
                plan_id="P9",
                entry_price=100.2,
                now_ms=t0 + 70_000,
                candle_mark={"bar_idx": 100, "decision_event_idx": 101},
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.2,
                    "best_bid_qty": 90,
                    "best_ask_qty": 10,
                },
            )
            self.assertTrue(again is None or not (again or {}).get("active"))

    def test_of_outcome_logging_is_idempotent_on_trade_id(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            row = {
                "of_trade_id": "OF-PAPER-TEST-1",
                "net_pnl_usdt": -1.0,
                "gross_pnl_usdt": 0.2,
                "friction_usdt": 1.2,
                "fees_usdt": 1.2,
                "spread_cost_usdt": 0.0,
                "slippage_usdt": 0.0,
                "execution_mode": "PAPER",
                "live_order_authority": 0.0,
            }
            self.assertTrue(mgr._log_of_outcome(row))
            self.assertFalse(mgr._log_of_outcome(row))
            self.assertFalse(mgr._log_of_outcome(dict(row)))
            lines = (Path(td) / "of_trade_outcomes.jsonl").read_text().strip().splitlines()
            self.assertEqual(len(lines), 1)

    def test_reconcile_drops_ghost_open_already_in_ledger(self):
        """OPEN must not mirror a closed of_trade_id (stats mismatch)."""
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of = self._open_long(mgr)
            tid = of["of_trade_id"]
            entry_qty = float(of["simulated_quantity"])
            # Simulate dual-writer: outcome booked but active left open
            self.assertTrue(
                mgr._log_of_outcome(
                    {
                        "of_trade_id": tid,
                        "direction": "LONG",
                        "entry_price": of["entry_price"],
                        "exit_price": 101.0,
                        "simulated_quantity": entry_qty,
                        "simulated_notional_usdt": of["simulated_notional_usdt"],
                        "net_pnl_usdt": 0.05,
                        "net_return_bps": 0.5,
                        "gross_pnl_usdt": 1.0,
                        "friction_usdt": 0.95,
                        "fees_usdt": 0.9,
                        "spread_cost_usdt": 0.05,
                        "slippage_usdt": 0.0,
                        "execution_mode": "PAPER",
                        "live_order_authority": 0.0,
                    }
                )
            )
            self.assertTrue(mgr.state["active_order_flow_recommendation"]["active"])
            self.assertTrue(mgr.reconcile_of_paper_already_closed(persist=True))
            self.assertIsNone(mgr.state.get("active_order_flow_recommendation"))
            # last_closed must hydrate from ledger (not empty RECONCILED stub)
            last = mgr.state.get("last_closed_order_flow_recommendation") or {}
            self.assertEqual(last.get("of_trade_id"), tid)
            self.assertEqual(last.get("exit_price"), 101.0)
            self.assertEqual(last.get("net_pnl_usdt"), 0.05)
            snap = mgr.build_of_shadow_trade_snapshot(
                {"best_bid_price": 100.0, "best_ask_price": 100.1, "best_bid_qty": 10, "best_ask_qty": 90}
            )
            self.assertTrue(snap is None or not snap.get("open"))
            # Resurrection from disk open file must also fail
            mgr.state["active_order_flow_recommendation"] = dict(of)
            mgr._write_of_open_state_file(of)
            mgr2 = TradePlanOverlayManager(base_dir=Path(td))
            self.assertIsNone(mgr2.state.get("active_order_flow_recommendation"))
            recent = mgr2.load_recent_of_outcomes(5)
            self.assertEqual(len(recent), 1)
            self.assertEqual(recent[0]["of_trade_id"], tid)

    def test_paper_open_rejects_thin_opposite_and_micro_signal(self):
        """PAPER open quality: hollow/micro books do not book a paper trade."""
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            # Extreme imb via empty ask — thin_opposite_diagnostic true
            thin = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of={"order_flow_imbalance": 0.99, "order_flow_source": "test"},
                plan_id="THIN",
                entry_price=100.0,
                now_ms=1793500000000,
                candle_mark={"bar_idx": 200, "decision_event_idx": 200},
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 50.0,
                    "best_ask_qty": 0.01,  # thin opposite → reject
                },
            )
            self.assertTrue(thin is None or not (thin or {}).get("active"))
            supp = mgr.state.get("last_of_paper_open_suppressed") or {}
            self.assertTrue(supp.get("reason"))
            # Micro signal qty below 0.25 floor
            micro = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of={"order_flow_imbalance": 0.6, "order_flow_source": "test"},
                plan_id="MICRO",
                entry_price=100.0,
                now_ms=1793500001000,
                candle_mark={"bar_idx": 201, "decision_event_idx": 201},
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 0.05,  # < 0.25 min signal
                    "best_ask_qty": 0.02,
                },
            )
            self.assertTrue(micro is None or not (micro or {}).get("active"))
            self.assertIn("entry_signal_qty", str((mgr.state.get("last_of_paper_open_suppressed") or {}).get("reason")))

    def test_restart_reconstructs_open_without_new_id(self):
        with tempfile.TemporaryDirectory() as td:
            base = Path(td)
            mgr = TradePlanOverlayManager(base_dir=base)
            of = self._open_long(mgr)
            tid = of["of_trade_id"]
            # Wipe overlay state active but keep open journal — reconstruct from disk
            mgr.state["active_order_flow_recommendation"] = None
            mgr._save_state()
            mgr2 = TradePlanOverlayManager(base_dir=base)
            restored = mgr2.state.get("active_order_flow_recommendation")
            self.assertTrue(restored and restored.get("active"))
            self.assertEqual(restored["of_trade_id"], tid)
            self.assertTrue(restored.get("reconstructed_from_open_journal"))
            snap = mgr2.build_of_shadow_trade_snapshot(
                {"best_bid_price": 99.8, "best_ask_price": 100.0, "best_bid_qty": 70, "best_ask_qty": 30}
            )
            self.assertEqual(snap["of_trade_id"], tid)
            self.assertEqual(snap["execution_mode"], "PAPER")

    def test_depth_quality_telemetry_on_open_and_exit_no_entry_gate(self):
        """Depth metrics journaled on quality PAPER opens; thin diagnostic remains telemetry."""
        thin = compute_of_depth_quality(
            direction="LONG",
            quote={
                "best_bid_price": 78530.0,
                "best_ask_price": 78530.1,
                "best_bid_qty": 0.6206,
                "best_ask_qty": 0.0001,
            },
            imbalance=1.0,
            phase="entry",
        )
        self.assertEqual(thin["protocol_id"], OF_DEPTH_QUALITY_PROTOCOL_ID)
        self.assertFalse(thin["gates_entry"])  # depth struct itself never gates OF mark
        self.assertFalse(thin["gates_exit"])
        self.assertTrue(thin["thin_opposite_diagnostic"])
        self.assertGreater(thin["combined_touch_notional_usdt"], 0.0)
        self.assertAlmostEqual(thin["same_side_touch_qty"], 0.6206)
        self.assertLess(thin["opposing_side_touch_notional_usdt"], thin["same_side_touch_notional_usdt"])

        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            # Healthy book (not thin / above signal floor) so PAPER opens and stamps depth
            card = {
                "bar_idx": 200,
                "decision_event_idx": 200,
                "timestamp_utc": "2026-11-01T00:00:00Z",
                "close_time_ms": 1793500000000,
                "reference_price": 100.0,
                "atr_14": 1.0,
                "policy_b_action": "LONG",
                "order_flow_supported": True,
                "order_flow_label": ORDER_FLOW_LABEL,
                "order_flow_imbalance": 0.8,
                "order_flow_source": "test",
                "execution_quote": {
                    "best_bid_price": 99.9,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 90.0,
                    "best_ask_qty": 10.0,
                    "exchange_event_time_ms": 1793500000000,
                },
            }
            mgr.process_decision_card(card)
            of = mgr.state.get("active_order_flow_recommendation")
            self.assertTrue(of and of.get("active"))
            dq = of.get("depth_quality") or {}
            self.assertEqual(dq.get("protocol_id"), OF_DEPTH_QUALITY_PROTOCOL_ID)
            self.assertEqual(dq.get("primary_depth_metric"), "combined_touch_notional_usdt")
            self.assertIn("combined_touch_notional_usdt", dq)
            self.assertFalse(dq.get("thin_opposite_diagnostic"))
            snap = mgr.build_of_shadow_trade_snapshot(
                {
                    "best_bid_price": 100.2,
                    "best_ask_price": 100.3,
                    "best_bid_qty": 1.0,
                    "best_ask_qty": 1.0,
                },
                now_ms=1793500000000 + 60_000,
            )
            self.assertIsNotNone(snap.get("unrealized_gross_pnl_usdt"))
            self.assertIsNotNone(snap.get("unrealized_net_pnl_usdt"))
            # Gross can be positive while net is fee-negative
            self.assertGreater(float(snap["unrealized_gross_pnl_usdt"]), float(snap["unrealized_net_pnl_usdt"]))

            t0 = int(of["activated_ms"])
            mgr.state["active_order_flow_recommendation"]["opposite_exit_seen_ms"] = t0 + 300_000
            mgr._clear_order_flow_recommendation(
                reason="OPPOSITE_SIDE_ORDER_FLOW",
                exit_meta={"exit": True, "opposite_side": "SHORT", "order_flow_imbalance": -0.9},
                now_ms=t0 + 300_000 + 30_000,
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 0.1,
                    "best_ask_qty": 5.0,
                },
            )
            recent = mgr.load_recent_of_outcomes(1)
            self.assertIn("depth_quality_entry", recent[0])
            self.assertIn("depth_quality_exit", recent[0])
            self.assertEqual(recent[0].get("depth_quality_protocol_id"), OF_DEPTH_QUALITY_PROTOCOL_ID)

    def test_entry_signal_pct_sizing_and_exit_qty_lock(self):
        """PAPER qty = 5–15% of entry-side L1; exit must reuse that qty (not exit depth)."""
        # LONG signal = bidQty 90 → 10% = 9; at px 100 → notional 900 → clamp max 500 → qty 5
        long_size = size_of_paper_from_entry_signal(
            direction="LONG",
            entry_price=100.0,
            quote={"best_bid_qty": 90, "best_ask_qty": 10},
        )
        self.assertEqual(long_size["entry_signal_qty"], 90.0)
        self.assertEqual(long_size["participation_pct"], 0.10)
        self.assertEqual(long_size["sizing_source"], "clamped_max_notional")
        self.assertAlmostEqual(long_size["simulated_notional_usdt"], 500.0)
        self.assertAlmostEqual(long_size["simulated_quantity"], 5.0)

        # SMALL book: bidQty 2 → 10% = 0.2 → notional 20 → clamp min 25
        small = size_of_paper_from_entry_signal(
            direction="LONG",
            entry_price=100.0,
            quote={"best_bid_qty": 2, "best_ask_qty": 1},
        )
        self.assertEqual(small["sizing_source"], "clamped_min_notional")
        self.assertAlmostEqual(small["simulated_notional_usdt"], 25.0)

        # Mid-range: bidQty 4 → 10% = 0.4 → notional 40 (inside [25,500])
        mid = size_of_paper_from_entry_signal(
            direction="LONG",
            entry_price=100.0,
            quote={"best_bid_qty": 4, "best_ask_qty": 1},
        )
        self.assertEqual(mid["sizing_source"], "entry_signal_pct")
        self.assertAlmostEqual(mid["simulated_quantity"], 0.4)
        self.assertAlmostEqual(mid["simulated_notional_usdt"], 40.0)

        # Participation hard clamp to [5%, 15%]
        over = size_of_paper_from_entry_signal(
            direction="SHORT",
            entry_price=100.0,
            quote={"best_bid_qty": 1, "best_ask_qty": 5},
            participation_pct=0.50,
        )
        self.assertEqual(over["participation_pct"], 0.15)
        under = size_of_paper_from_entry_signal(
            direction="SHORT",
            entry_price=100.0,
            quote={"best_bid_qty": 1, "best_ask_qty": 5},
            participation_pct=0.01,
        )
        self.assertEqual(under["participation_pct"], 0.05)

        # Exit P&L must use locked entry qty even if notional/exit depth disagree
        locked = 0.4
        pnl = compute_of_paper_cost_breakdown(
            direction="LONG",
            entry_price=100.0,
            exit_price=101.0,
            notional=1000.0,  # stale fixed notional must NOT win
            entry_bid=99.9,
            entry_ask=100.1,
            exit_bid=100.9,
            exit_ask=101.1,
            quantity=locked,
        )
        self.assertAlmostEqual(pnl["simulated_quantity"], locked)
        self.assertAlmostEqual(pnl["simulated_notional_usdt"], 40.0)

        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of = self._open_long(mgr)
            # bidQty 90 @ ~100.1 ask → 10% raw then max-notional clamp
            entry_qty = float(of["simulated_quantity"])
            self.assertGreater(entry_qty, 0.0)
            self.assertEqual(of["participation_pct"], 0.10)
            self.assertEqual(of["entry_signal_qty"], 90.0)
            self.assertNotAlmostEqual(of["simulated_notional_usdt"], 1000.0)
            t0 = int(of["activated_ms"])
            mgr.state["active_order_flow_recommendation"]["opposite_exit_seen_ms"] = t0 + 300_000
            mgr._clear_order_flow_recommendation(
                reason="OPPOSITE_SIDE_ORDER_FLOW",
                exit_meta={
                    "exit": True,
                    "opposite_side": "SHORT",
                    "order_flow_imbalance": -0.8,
                },
                now_ms=t0 + 300_000 + 30_000,
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    # Exit book depth deliberately different — must NOT resize
                    "best_bid_qty": 1,
                    "best_ask_qty": 999,
                },
            )
            closed = mgr.state.get("last_closed_order_flow_recommendation")
            self.assertAlmostEqual(float(closed["simulated_quantity"]), entry_qty)
            self.assertAlmostEqual(float(closed["exit_quantity"]), entry_qty)
            self.assertAlmostEqual(float(closed["entry_paper_quantity"]), entry_qty)
            self.assertTrue(closed.get("qty_matched_entry_to_exit"))
            # Exit-book signal differs from entry signal — must not resize paper qty
            self.assertEqual(float(of["entry_signal_qty"]), 90.0)
            self.assertAlmostEqual(float(closed["exit_signal_qty"]), 999.0)
            self.assertNotAlmostEqual(float(closed["exit_signal_qty"]), float(of["entry_signal_qty"]))
            self.assertEqual((of.get("entry_candle") or {}).get("signal_qty"), 90.0)
            self.assertEqual((of.get("entry_candle") or {}).get("paper_quantity"), entry_qty)
            recent = mgr.load_recent_of_outcomes(1)
            self.assertAlmostEqual(float(recent[0]["simulated_quantity"]), entry_qty)
            self.assertAlmostEqual(float(recent[0]["exit_quantity"]), entry_qty)
            self.assertTrue(recent[0].get("qty_matched_entry_to_exit"))
            self.assertEqual(of_entry_signal_qty("LONG", {"best_bid_qty": 90, "best_ask_qty": 10}), 90.0)
            self.assertEqual(of_exit_signal_qty("LONG", {"best_bid_qty": 1, "best_ask_qty": 999}), 999.0)

    def test_inspect_order_flow_book_marks_both_sides(self):
        long_l1 = {"bid_qty": 90, "ask_qty": 10, "bid_price": 100.0, "ask_price": 100.1}
        insp = inspect_order_flow_book(long_l1, primary_side="LONG")
        self.assertTrue(insp["order_flow_inspection"])
        self.assertTrue(insp["order_flow_supported"])
        self.assertTrue(insp["order_flow_long_supported"])
        self.assertFalse(insp["order_flow_short_supported"])
        self.assertEqual(insp["order_flow_label"], ORDER_FLOW_LABEL)
        self.assertEqual(insp["order_flow_side"], "LONG")

        short_primary = inspect_order_flow_book(long_l1, primary_side="SHORT")
        self.assertFalse(short_primary["order_flow_supported"])
        self.assertTrue(short_primary["order_flow_long_supported"])

    def test_l1_from_nse_quote_object(self):
        class Q:
            bid_qty = 80.0
            ask_qty = 20.0
            bid_price = 10.0
            ask_price = 10.1

        l1 = l1_from_nse_quote(Q(), source="kite_l1")
        self.assertEqual(l1["bid_qty"], 80.0)
        self.assertEqual(l1["ask_qty"], 20.0)
        self.assertEqual(l1["source"], "kite_l1")
        self.assertTrue(
            classify_order_flow_support("LONG", l1)["order_flow_supported"]
        )

    def test_scrub_on_load_does_not_drop_booked_open_and_restores(self):
        """Legacy scrub_on_load must not kill booked PAPER; restore heals journal orphans."""
        with tempfile.TemporaryDirectory() as td:
            base = Path(td)
            mgr = TradePlanOverlayManager(base_dir=base)
            of = self._open_long(mgr)
            tid = of["of_trade_id"]
            # Simulate legacy scrub (entry quality misapplied post-book)
            mgr.state["last_of_paper_open_suppressed"] = {
                "reason": "scrub_on_load_thin_opposite_diagnostic",
                "of_trade_id": tid,
                "direction": "LONG",
                "entry_signal_qty": of.get("entry_signal_qty"),
                "depth_quality": of.get("depth_quality"),
            }
            mgr.state["active_order_flow_recommendation"] = None
            mgr._write_of_open_state_file(None)
            mgr._save_state()
            # Restart path: reconstruct empty, then restore from opens journal
            mgr2 = TradePlanOverlayManager(base_dir=base)
            restored = mgr2.state.get("active_order_flow_recommendation")
            self.assertTrue(restored and restored.get("active"))
            self.assertEqual(restored.get("of_trade_id"), tid)
            self.assertTrue(restored.get("restored_after_scrub_on_load"))
            self.assertIsNone(mgr2.state.get("last_of_paper_open_suppressed"))
            # Thin diagnostic on open must not scrub again on a third load
            mgr2.state["active_order_flow_recommendation"]["depth_quality"] = {
                "thin_opposite_diagnostic": True,
                "same_side_touch_notional_usdt": 20_000.0,
            }
            mgr2._save_state()
            mgr3 = TradePlanOverlayManager(base_dir=base)
            still = mgr3.state.get("active_order_flow_recommendation")
            self.assertTrue(still and still.get("active"))
            self.assertEqual(still.get("of_trade_id"), tid)

    def test_exit_candle_does_not_reuse_entry_bar_idx(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            of = self._open_long(mgr)
            t0 = int(of["activated_ms"])
            mgr.state["active_order_flow_recommendation"]["opposite_exit_seen_ms"] = t0 + 300_000
            mgr._clear_order_flow_recommendation(
                reason="OPPOSITE_SIDE_ORDER_FLOW",
                exit_meta={"exit": True, "opposite_side": "SHORT", "order_flow_imbalance": -0.9},
                now_ms=t0 + 300_000 + 30_000,
                candle_mark={
                    "bar_idx": 105,
                    "decision_event_idx": 777,
                    "mark_time_ms": t0 + 300_000 + 30_000,
                },
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 10,
                    "best_ask_qty": 90,
                },
            )
            closed = mgr.state.get("last_closed_order_flow_recommendation") or {}
            xc = closed.get("exit_candle") or {}
            self.assertEqual(xc.get("bar_idx"), 105)
            self.assertEqual(xc.get("decision_event_idx"), 777)
            self.assertNotEqual(xc.get("bar_idx"), (of.get("entry_candle") or {}).get("bar_idx"))


class TestOfPaperEntryQualityPersist(unittest.TestCase):
    """Entry OF persist + quality gate (does not change ±0.30 classify)."""

    def setUp(self):
        self._prev_persist = CONFIG.get("of_paper_entry_persist_ms")
        self._prev_gate = None
        import os

        self._os = os
        self._prev_gate = os.environ.get("RP608_OF_PAPER_QUALITY_GATE")
        os.environ["RP608_OF_PAPER_QUALITY_GATE"] = "1"
        CONFIG["of_paper_entry_persist_ms"] = 30_000

    def tearDown(self):
        CONFIG["of_paper_entry_persist_ms"] = self._prev_persist
        if self._prev_gate is None:
            self._os.environ.pop("RP608_OF_PAPER_QUALITY_GATE", None)
        else:
            self._os.environ["RP608_OF_PAPER_QUALITY_GATE"] = self._prev_gate

    def _healthy_quote(self):
        return {
            "best_bid_price": 99.9,
            "best_ask_price": 100.1,
            "best_bid_qty": 90.0,
            "best_ask_qty": 10.0,
        }

    def test_entry_persist_blocks_then_allows(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            t0 = 1793500000000
            of_meta = {
                "order_flow_imbalance": 0.8,
                "order_flow_source": "test",
            }
            first = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of=of_meta,
                plan_id="P1",
                entry_price=100.0,
                now_ms=t0,
                candle_mark={"bar_idx": 100},
                quote=self._healthy_quote(),
            )
            self.assertTrue(first is None or not (first or {}).get("active"))
            supp = mgr.state.get("last_of_paper_open_suppressed") or {}
            self.assertIn("entry_persist", str(supp.get("reason") or ""))
            q = supp.get("of_entry_quality") or {}
            self.assertEqual(q.get("protocol_id"), OF_ENTRY_QUALITY_PROTOCOL_ID)
            self.assertFalse(q.get("persist_ok"))
            self.assertTrue(q.get("would_have_opened") or supp.get("would_have_opened"))
            arm = mgr.state.get("of_entry_persist_arm") or {}
            self.assertEqual(arm.get("direction"), "LONG")

            # Same-side OF continues — still short of 30s
            mid = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of=of_meta,
                plan_id="P1",
                entry_price=100.0,
                now_ms=t0 + 10_000,
                candle_mark={"bar_idx": 100},
                quote=self._healthy_quote(),
            )
            self.assertTrue(mid is None or not (mid or {}).get("active"))

            opened = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of=of_meta,
                plan_id="P1",
                entry_price=100.0,
                now_ms=t0 + 30_000,
                candle_mark={"bar_idx": 100},
                quote=self._healthy_quote(),
            )
            self.assertTrue(opened and opened.get("active"))
            self.assertTrue((opened.get("of_entry_quality") or {}).get("persist_ok"))
            self.assertIsNone(mgr.state.get("of_entry_persist_arm"))

    def test_entry_persist_clears_when_of_drops(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            t0 = 1793500000000
            mgr._activate_order_flow_recommendation(
                direction="LONG",
                of={"order_flow_imbalance": 0.8, "order_flow_source": "test"},
                plan_id="P2",
                entry_price=100.0,
                now_ms=t0,
                candle_mark={"bar_idx": 101},
                quote=self._healthy_quote(),
            )
            self.assertIsNotNone(mgr.state.get("of_entry_persist_arm"))
            # Neutral L1 via continue path clears arm
            gone = mgr.continue_of_paper_entry_persist(
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 50.0,
                    "best_ask_qty": 50.0,
                },
                now_ms=t0 + 5_000,
                persist=True,
            )
            self.assertIsNone(gone)
            self.assertIsNone(mgr.state.get("of_entry_persist_arm"))

    def test_depth_reject_stamps_quality(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = TradePlanOverlayManager(base_dir=Path(td))
            CONFIG["of_paper_entry_persist_ms"] = 0
            thin = mgr._activate_order_flow_recommendation(
                direction="LONG",
                of={"order_flow_imbalance": 0.99, "order_flow_source": "test"},
                plan_id="THIN2",
                entry_price=100.0,
                now_ms=1793500000000,
                candle_mark={"bar_idx": 202},
                quote={
                    "best_bid_price": 100.0,
                    "best_ask_price": 100.1,
                    "best_bid_qty": 50.0,
                    "best_ask_qty": 0.01,
                },
            )
            self.assertTrue(thin is None or not (thin or {}).get("active"))
            q = (mgr.state.get("last_of_paper_open_suppressed") or {}).get(
                "of_entry_quality"
            ) or {}
            self.assertTrue(q.get("persist_ok"))
            self.assertFalse(q.get("depth_ok"))
            self.assertIn("thin_opposite", str(q.get("reason") or ""))

    def test_imbalance_threshold_unchanged(self):
        from scripts.rp608_order_flow_mark import ORDER_FLOW_IMBALANCE_THRESHOLD

        self.assertAlmostEqual(ORDER_FLOW_IMBALANCE_THRESHOLD, 0.30)
        long_ok = classify_order_flow_support(
            "LONG", {"bid_qty": 65, "ask_qty": 35, "source": "t"}
        )
        # (65-35)/(65+35)=0.30 exactly → supported at threshold
        self.assertTrue(long_ok["order_flow_supported"])
        below = classify_order_flow_support(
            "LONG", {"bid_qty": 64, "ask_qty": 36, "source": "t"}
        )
        self.assertFalse(below["order_flow_supported"])


if __name__ == "__main__":
    unittest.main()
