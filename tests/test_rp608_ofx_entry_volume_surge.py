"""OFX entry: tick volume ≥ 3× mean of preceding 20 completed ticks."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow import ExecutedTrade, SideClass
from research.rp608_ofx.executed_flow.aggregate import SIDE_DIRECT
from research.rp608_ofx.executed_flow.volume_mark import (
    evaluate_ofx_entry_volume_surge,
    ofx_entry_surge_from_trades,
)


def _t(qty, aggressor, ts, seq):
    return ExecutedTrade(
        instrument_id="BTCUSDT",
        timestamp_ms=ts,
        price=100.0,
        quantity=qty,
        side=SideClass(aggressor=aggressor, classification=SIDE_DIRECT, evidence="t"),
        source="test",
        sequence_id=str(seq),
    )


class TestOfxEntryVolumeSurge(unittest.TestCase):
    def test_pass_when_entry_at_least_3x_avg(self):
        gate = evaluate_ofx_entry_volume_surge(
            entry_volume=30.0,
            preceding_tick_volumes=[10.0] * 20,
        )
        self.assertTrue(gate["passed"])
        self.assertEqual(gate["avg_preceding_volume"], 10.0)
        self.assertEqual(gate["surge_threshold"], 30.0)
        self.assertTrue(gate["order_flow_signals_excluded"])

    def test_fail_below_3x(self):
        gate = evaluate_ofx_entry_volume_surge(
            entry_volume=29.0,
            preceding_tick_volumes=[10.0] * 20,
        )
        self.assertFalse(gate["passed"])
        self.assertEqual(gate["reason"], "surge_below_multiple")

    def test_insufficient_lookback(self):
        gate = evaluate_ofx_entry_volume_surge(
            entry_volume=100.0,
            preceding_tick_volumes=[1.0] * 5,
        )
        self.assertFalse(gate["passed"])
        self.assertEqual(gate["reason"], "insufficient_preceding_ticks")

    def test_from_trades_long_buy_surge(self):
        trades = [_t(1.0, "SELL", 1000 + i, i) for i in range(20)]
        trades.append(_t(3.0, "BUY", 2000, 99))
        gate = ofx_entry_surge_from_trades(trades, direction="LONG")
        self.assertTrue(gate["passed"])
        self.assertEqual(gate["entry_aggressor_side"], "BUY")
        self.assertEqual(gate["entry_volume"], 3.0)

    def test_from_trades_ignores_wrong_side_as_entry(self):
        # Last tick is SELL — LONG needs BUY entry tick further back with 20 prior
        trades = [_t(1.0, "BUY", 1000 + i, i) for i in range(20)]
        trades.append(_t(3.0, "BUY", 2000, 98))
        trades.append(_t(9.0, "SELL", 2001, 99))  # not used as LONG entry
        gate = ofx_entry_surge_from_trades(trades, direction="LONG")
        self.assertTrue(gate["passed"])
        self.assertEqual(gate["entry_tick_timestamp_ms"], 2000)
        self.assertEqual(gate["entry_volume"], 3.0)


if __name__ == "__main__":
    unittest.main()
