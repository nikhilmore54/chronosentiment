#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.research.rp608_nse_of_paper_production_recon_one_trade import (
    independent_zerodha_intraday_charges,
    reconcile_trade,
)
from scripts.rp608_nse_equity_costs import (
    BROKERAGE_PCT,
    BROKERAGE_RATE,
    EXCHANGE_PCT,
    EXCHANGE_TURNOVER_RATE,
    SEBI_PCT,
    SEBI_TURNOVER_RATE,
    STT_RATE_SELL_LEG,
    STT_SELL_PCT,
    calc_indian_equity_friction,
    calc_indian_equity_friction_from_orders,
    calc_nse_of_paper_accounting,
    executed_orders_from_ledger_trade,
)


class TestNseEquityCosts(unittest.TestCase):
    def test_gst_not_on_turnover(self):
        fr = calc_indian_equity_friction(
            1000, 1010, 100, half_spread_bps=0, include_spread_crossing=False, slippage_bps_per_leg=0
        )
        taxable = fr["brokerage"] + fr["exchange_charges"] + fr["sebi_charges"]
        self.assertAlmostEqual(fr["gst"], 0.18 * taxable, places=4)

    def test_percent_to_decimal_conversion(self):
        self.assertAlmostEqual(BROKERAGE_RATE, BROKERAGE_PCT / 100.0)
        self.assertAlmostEqual(STT_RATE_SELL_LEG, STT_SELL_PCT / 100.0)
        self.assertAlmostEqual(EXCHANGE_TURNOVER_RATE, EXCHANGE_PCT / 100.0)
        self.assertAlmostEqual(SEBI_TURNOVER_RATE, SEBI_PCT / 100.0)
        self.assertAlmostEqual(SEBI_TURNOVER_RATE, 1e-6)

    def test_executed_orders_unequal_brokerage(self):
        fr = calc_indian_equity_friction_from_orders(
            [
                {"side": "BUY", "turnover": 100_000.0},
                {"side": "BUY", "turnover": 10_000.0},
                {"side": "SELL", "turnover": 110_000.0},
            ],
            slippage_bps_per_leg=0.0,
        )
        self.assertAlmostEqual(fr["brokerage"], 43.0, places=4)
        self.assertEqual(fr["n_buy_orders"], 2)
        self.assertEqual(fr["n_sell_orders"], 1)

    def test_rejects_equal_split_without_turnovers(self):
        with self.assertRaises(ValueError):
            calc_indian_equity_friction(100, 100, 10, n_buy_orders=2)

    def test_long_vs_short_side_assignment(self):
        """SHORT: STT on entry (sell), stamp on exit (buy)."""
        long_acct = calc_nse_of_paper_accounting(
            entry_price=100.0, exit_price=101.0, quantity=10, direction="LONG"
        )
        short_acct = calc_nse_of_paper_accounting(
            entry_price=100.0, exit_price=99.0, quantity=10, direction="SHORT"
        )
        # LONG buy 1000 / sell 1010
        self.assertAlmostEqual(long_acct["stt"], 1010.0 * 0.00025, places=4)
        self.assertAlmostEqual(long_acct["stamp_duty"], 1000.0 * 0.00003, places=4)
        self.assertEqual(long_acct["touch_gross_pnl_inr"], 10.0)

        # SHORT sell 1000 entry / buy 990 exit
        self.assertAlmostEqual(short_acct["stt"], 1000.0 * 0.00025, places=4)
        self.assertAlmostEqual(short_acct["stamp_duty"], 990.0 * 0.00003, places=4)
        self.assertEqual(short_acct["touch_gross_pnl_inr"], 10.0)

        long_sides = [o["side"] for o in long_acct["executed_orders"]]
        short_sides = [o["side"] for o in short_acct["executed_orders"]]
        self.assertEqual(long_sides, ["BUY", "SELL"])
        self.assertEqual(short_sides, ["SELL", "BUY"])

    def test_tatasteel_paper_trade_unit_anchor(self):
        entry, exit_px, qty = 175.63, 175.94, 14.0
        acct = calc_nse_of_paper_accounting(
            entry_price=entry,
            exit_price=exit_px,
            quantity=qty,
            direction="LONG",
            executed_orders=[
                {"side": "BUY", "turnover": entry * qty},
                {"side": "SELL", "turnover": exit_px * qty},
            ],
        )
        buy_to, sell_to = entry * qty, exit_px * qty
        brok = min(buy_to * 0.0003, 20) + min(sell_to * 0.0003, 20)
        stt = sell_to * 0.00025
        exch = (buy_to + sell_to) * 0.0000307
        sebi = (buy_to + sell_to) * 0.000001
        stamp = buy_to * 0.00003
        gst = 0.18 * (brok + exch + sebi)
        statutory = brok + stt + exch + sebi + stamp + gst
        gross = (exit_px - entry) * qty
        self.assertAlmostEqual(acct["touch_gross_pnl_inr"], round(gross, 2))
        self.assertAlmostEqual(acct["statutory_overhead_inr"], round(statutory, 2))
        self.assertAlmostEqual(acct["stt"], round(stt, 4))
        self.assertAlmostEqual(acct["stamp_duty"], round(stamp, 4))
        self.assertLess(acct["statutory_overhead_inr"], 3.87)

    def test_production_recon_tatasteel_from_ledger_fixture(self):
        """
        Final acceptance gate: one real ledger trade reconstructed from
        executed-order list, matched to independent broker schedule.
        """
        fixture = ROOT / "tests" / "fixtures" / "nse_of_paper_tatasteel_production_trade.json"
        trade = json.loads(fixture.read_text(encoding="utf-8"))
        legs = executed_orders_from_ledger_trade(trade)
        self.assertEqual([o["side"] for o in legs], ["BUY", "SELL"])
        self.assertAlmostEqual(legs[0]["turnover"], 175.63 * 14.0, places=4)
        self.assertAlmostEqual(legs[1]["turnover"], 175.94 * 14.0, places=4)
        # Must be price×qty per execution — not combined/n
        self.assertNotAlmostEqual(legs[0]["turnover"], legs[1]["turnover"], places=2)

        report = reconcile_trade(trade)
        self.assertTrue(report["pass"], report)
        self.assertFalse(report["equal_split_used"])
        self.assertEqual(report["disposition"], "PRODUCTION_RECON_PASS")
        self.assertEqual(report["historical_v1_bulk_totals"], "PROVISIONAL")
        self.assertAlmostEqual(
            report["paper_accounting_v1"]["touch_gross_pnl_inr"], 4.34, places=2
        )
        self.assertAlmostEqual(
            report["paper_accounting_v1"]["statutory_overhead_inr"], 2.62, places=2
        )
        self.assertFalse(report["paper_accounting_v1"]["spread_crossing_charged"])

    def test_multi_fill_reconstruction_no_equal_split(self):
        """Partial fills: brokerage capped per actual order turnover."""
        trade = {
            "of_trade_id": "SYNTH-MULTI-FILL-LONG",
            "direction": "LONG",
            "entry_price": 100.0,
            "exit_price": 110.0,
            "exit_quantity": 1100.0,
            "fills": [
                {"side": "BUY", "turnover": 100_000.0},
                {"side": "BUY", "turnover": 10_000.0},
                {"side": "SELL", "turnover": 110_000.0},
            ],
        }
        legs = executed_orders_from_ledger_trade(trade)
        self.assertEqual(len(legs), 3)
        # Equal-split of buy 110k across 2 would be 55k each — must not happen
        buy_turnovers = sorted(o["turnover"] for o in legs if o["side"] == "BUY")
        self.assertEqual(buy_turnovers, [10_000.0, 100_000.0])

        shared = calc_indian_equity_friction_from_orders(legs, slippage_bps_per_leg=0.0)
        independent = independent_zerodha_intraday_charges(legs)
        self.assertAlmostEqual(shared["brokerage"], 43.0, places=4)
        self.assertAlmostEqual(independent["brokerage"], 43.0, places=4)
        self.assertAlmostEqual(shared["stt"], 110_000.0 * 0.00025, places=4)
        self.assertAlmostEqual(shared["stamp_duty"], 110_000.0 * 0.00003, places=4)
        taxable = (
            independent["brokerage"]
            + independent["exchange_charges"]
            + independent["sebi_charges"]
        )
        self.assertAlmostEqual(independent["gst"], round(0.18 * taxable, 4), places=4)


if __name__ == "__main__":
    unittest.main()
