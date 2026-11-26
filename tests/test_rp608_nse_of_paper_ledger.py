"""NSE OF PAPER ledger (RP-608-NSE-OF-PAPER-TRADE-001) — 0% authority."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

from scripts.rp608_nse_of_paper_ledger import (
    PROTOCOL_ID,
    NseOfPaperLedger,
    size_nse_of_paper_from_entry_signal,
)


def _quote(bid_px=100.0, ask_px=100.2, bid_qty=200.0, ask_qty=40.0, half=2.5):
    return SimpleNamespace(
        bid_price=bid_px,
        ask_price=ask_px,
        bid_qty=bid_qty,
        ask_qty=ask_qty,
        half_spread_bps=half,
    )


def _friction(**kwargs):
    buy = kwargs["buy_price"] * kwargs["quantity"]
    sell = kwargs["sell_price"] * kwargs["quantity"]
    total_fric = 0.01 * (buy + sell)  # stub
    return {
        "total_friction": total_fric,
        "effective_friction_bps": 10.0,
    }


class TestNseOfPaperLedger(unittest.TestCase):
    def test_open_on_recommend_long_with_of_support(self):
        with tempfile.TemporaryDirectory() as td:
            ledger = NseOfPaperLedger(Path(td), friction_fn=_friction)
            q = _quote(bid_qty=500, ask_qty=50)
            ev = {
                "decision": "RECOMMEND_LONG",
                "bar_idx": 6,
                "session_idx": 1,
                "reference_price": 100.0,
                "order_flow_supported": True,
                "order_flow_long_supported": True,
                "order_flow_side": "LONG",
                "order_flow_imbalance": 0.8,
                "order_flow_source": "kite_l1",
            }
            rec = ledger.try_open_from_eval(
                symbol="GAIL.NS", eval_record=ev, quote=q, now_ms=1_700_000_000_000
            )
            self.assertIsNotNone(rec)
            self.assertTrue(rec["active"])
            self.assertEqual(rec["protocol_id"], PROTOCOL_ID)
            self.assertEqual(rec["live_order_authority"], 0.0)
            self.assertEqual(rec["direction"], "LONG")
            self.assertEqual(ledger.summary()["n_open"], 1)
            # one-per-bar
            again = ledger.try_open_from_eval(
                symbol="GAIL.NS", eval_record=ev, quote=q, now_ms=1_700_000_000_100
            )
            self.assertTrue(again and again.get("of_trade_id") == rec["of_trade_id"])

    def test_no_open_without_recommend_or_of(self):
        with tempfile.TemporaryDirectory() as td:
            ledger = NseOfPaperLedger(Path(td), friction_fn=_friction)
            q = _quote()
            self.assertIsNone(
                ledger.try_open_from_eval(
                    symbol="GAIL.NS",
                    eval_record={
                        "decision": "NO_TRADE",
                        "bar_idx": 6,
                        "order_flow_supported": True,
                        "order_flow_long_supported": True,
                        "order_flow_imbalance": 0.9,
                        "reference_price": 100.0,
                    },
                    quote=q,
                    now_ms=1_700_000_000_000,
                )
            )
            # Ask-heavy book → no LONG OF
            q2 = _quote(bid_qty=10, ask_qty=200)
            self.assertIsNone(
                ledger.try_open_from_eval(
                    symbol="GAIL.NS",
                    eval_record={
                        "decision": "RECOMMEND_LONG",
                        "bar_idx": 6,
                        "order_flow_supported": False,
                        "order_flow_long_supported": False,
                        "order_flow_imbalance": -0.9,
                        "reference_price": 100.0,
                    },
                    quote=q2,
                    now_ms=1_700_000_000_000,
                )
            )

    def test_reverse_of_exit_after_min_hold_and_persist(self):
        with tempfile.TemporaryDirectory() as td:
            ledger = NseOfPaperLedger(Path(td), friction_fn=_friction)
            t0 = 1_700_000_000_000
            q_long = _quote(bid_qty=400, ask_qty=40)
            ev = {
                "decision": "RECOMMEND_LONG",
                "bar_idx": 7,
                "reference_price": 100.0,
                "order_flow_supported": True,
                "order_flow_long_supported": True,
                "order_flow_side": "LONG",
                "order_flow_imbalance": 0.82,
                "order_flow_source": "kite_l1",
            }
            opened = ledger.try_open_from_eval(
                symbol="SBIN.NS", eval_record=ev, quote=q_long, now_ms=t0
            )
            self.assertTrue(opened and opened["active"])
            q_short = _quote(bid_qty=20, ask_qty=300)
            # Immediate reverse — min hold suppresses
            still = ledger.evaluate_open_tick(
                symbol="SBIN.NS", quote=q_short, now_ms=t0 + 1_000, bar_idx=7
            )
            self.assertTrue(still and still.get("active"))
            self.assertIn("min_hold", str(still.get("close_suppressed_reason") or ""))
            # Past min hold — arm persist
            armed = ledger.evaluate_open_tick(
                symbol="SBIN.NS", quote=q_short, now_ms=t0 + 300_000, bar_idx=8
            )
            self.assertTrue(armed and armed.get("active"))
            # Persist 30s
            gone = ledger.evaluate_open_tick(
                symbol="SBIN.NS", quote=q_short, now_ms=t0 + 300_000 + 30_000, bar_idx=8
            )
            self.assertIsNone(gone)
            self.assertEqual(ledger.summary()["n_open"], 0)
            summ = ledger.summary()["of_outcomes_summary"]
            self.assertEqual(summ["n_trades"], 1)
            recent = ledger.load_recent_outcomes(5)
            self.assertEqual(recent[-1]["exit_reason"], "OPPOSITE_SIDE_ORDER_FLOW")
            self.assertEqual(recent[-1]["execution_mode"], "PAPER")
            self.assertEqual(recent[-1]["live_order_authority"], 0.0)

    def test_session_square_off(self):
        with tempfile.TemporaryDirectory() as td:
            ledger = NseOfPaperLedger(Path(td), friction_fn=_friction)
            t0 = 1_700_000_000_000
            q = _quote(bid_qty=300, ask_qty=50)
            ledger.try_open_from_eval(
                symbol="ITC.NS",
                eval_record={
                    "decision": "RECOMMEND_LONG",
                    "bar_idx": 10,
                    "reference_price": 100.0,
                    "order_flow_supported": True,
                    "order_flow_long_supported": True,
                    "order_flow_imbalance": 0.7,
                },
                quote=q,
                now_ms=t0,
            )
            n = ledger.square_off_all(
                quotes_by_symbol={"ITC.NS": q},
                now_ms=t0 + 60_000,
                bar_idx=23,
                reason="SESSION_SQUARE_OFF",
            )
            self.assertEqual(n, 1)
            self.assertEqual(ledger.summary()["n_open"], 0)
            self.assertEqual(
                ledger.load_recent_outcomes(1)[-1]["exit_reason"], "SESSION_SQUARE_OFF"
            )

    def test_thin_opposite_suppresses_open(self):
        with tempfile.TemporaryDirectory() as td:
            ledger = NseOfPaperLedger(Path(td), friction_fn=_friction)
            # Extreme imb via hollow ask
            q = _quote(bid_qty=500, ask_qty=0.01)
            rec = ledger.try_open_from_eval(
                symbol="RELIANCE.NS",
                eval_record={
                    "decision": "RECOMMEND_LONG",
                    "bar_idx": 5,
                    "reference_price": 100.0,
                    "order_flow_supported": True,
                    "order_flow_long_supported": True,
                    "order_flow_imbalance": 0.99,
                },
                quote=q,
                now_ms=1_700_000_000_000,
            )
            self.assertTrue(rec is None or not (rec or {}).get("active"))
            supp = ledger.summary().get("last_suppressed") or {}
            self.assertTrue(supp.get("reason"))

    def test_sizing_participation(self):
        size = size_nse_of_paper_from_entry_signal(
            entry_price=100.0,
            quote=_quote(bid_qty=200, ask_qty=50),
        )
        self.assertEqual(size["entry_signal_qty"], 200.0)
        self.assertGreaterEqual(size["simulated_quantity"], 1.0)
        self.assertIn(size["sizing_source"], {
            "participation", "clamped_min_notional", "clamped_max_notional", "fallback"
        })


if __name__ == "__main__":
    unittest.main()
