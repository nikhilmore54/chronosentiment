"""OFX volume/aggressor-direction mark — parallel to baseline L1 OF."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow import (
    ExecutedTrade,
    SideClass,
    aggregate_executed_flow,
    classify_ofx_volume_support,
    inspect_ofx_volume_book,
    is_ofx_volume_exit_signal,
)
from research.rp608_ofx.executed_flow.aggregate import SIDE_DIRECT
from scripts.rp608_ofx_crypto_volume_flow_mark import (
    build_ofx_volume_flow_snapshot,
    refresh_ofx_volume_flow_mark,
)
from scripts.rp608_order_flow_mark import (
    ORDER_FLOW_IMBALANCE_THRESHOLD,
    classify_order_flow_support,
)


def _t(qty, aggressor, ts=1000, seq="1"):
    return ExecutedTrade(
        instrument_id="BTCUSDT",
        timestamp_ms=ts,
        price=100.0,
        quantity=qty,
        side=SideClass(aggressor=aggressor, classification=SIDE_DIRECT, evidence="t"),
        source="test",
        sequence_id=seq,
    )


class TestOfxVolumeMark(unittest.TestCase):
    def test_long_supported_on_buy_dominant_volume(self):
        r = aggregate_executed_flow(
            [_t(9.0, "BUY", seq="a"), _t(1.0, "SELL", seq="b")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        mark = classify_ofx_volume_support("LONG", r)
        self.assertTrue(mark["ofx_volume_supported"])
        self.assertGreaterEqual(mark["ofx_volume_imbalance"], 0.30)
        self.assertTrue(mark["baseline_of_inert"])
        self.assertEqual(mark["construct"], "EXECUTED_AGGRESSOR_VOLUME")

    def test_short_supported_on_sell_dominant_volume(self):
        r = aggregate_executed_flow(
            [_t(1.0, "BUY", seq="a"), _t(9.0, "SELL", seq="b")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        self.assertTrue(classify_ofx_volume_support("SHORT", r)["ofx_volume_supported"])
        self.assertFalse(classify_ofx_volume_support("LONG", r)["ofx_volume_supported"])

    def test_opposite_volume_exit(self):
        r = aggregate_executed_flow(
            [_t(1.0, "BUY", seq="a"), _t(9.0, "SELL", seq="b")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        ex = is_ofx_volume_exit_signal("LONG", r)
        self.assertTrue(ex["exit"])
        self.assertEqual(ex["opposite_side"], "SHORT")

    def test_does_not_use_l1_book(self):
        # Baseline L1 would be bid-heavy; volume mark uses trades only
        l1 = classify_order_flow_support(
            "LONG", {"bid_qty": 90, "ask_qty": 10, "source": "l1"}
        )
        self.assertTrue(l1["order_flow_supported"])
        r = aggregate_executed_flow(
            [_t(1.0, "BUY", seq="a"), _t(9.0, "SELL", seq="b")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        vol = classify_ofx_volume_support("LONG", r)
        self.assertFalse(vol["ofx_volume_supported"])
        self.assertEqual(ORDER_FLOW_IMBALANCE_THRESHOLD, 0.30)  # baseline untouched


class TestSnapshotFromJournal(unittest.TestCase):
    def test_refresh_writes_snapshot(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            journal = root / "aggtrades.jsonl"
            rows = [
                {
                    "symbol": "BTCUSDT",
                    "agg_trade_id": 1,
                    "price": 100.0,
                    "quantity": 8.0,
                    "is_buyer_maker": False,
                    "aggressor": "BUY",
                    "trade_time_ms": 1_700_000_000_000,
                    "raw_ref": {"s": "BTCUSDT", "a": 1, "q": "8", "m": False, "T": 1_700_000_000_000},
                },
                {
                    "symbol": "BTCUSDT",
                    "agg_trade_id": 2,
                    "price": 100.0,
                    "quantity": 1.0,
                    "is_buyer_maker": True,
                    "aggressor": "SELL",
                    "trade_time_ms": 1_700_000_000_500,
                    "raw_ref": {"s": "BTCUSDT", "a": 2, "q": "1", "m": True, "T": 1_700_000_000_500},
                },
            ]
            journal.write_text("\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8")
            snap = refresh_ofx_volume_flow_mark(
                root, symbol="BTCUSDT", now_ms=1_700_000_030_000, window_ms=60_000
            )
            self.assertEqual(snap["dominant_aggressor"], "BUY_AGGRESSOR")
            self.assertTrue(snap["baseline_of_inert"])
            out = root / "ofx_volume_flow_snapshot.json"
            self.assertTrue(out.exists())
            disk = json.loads(out.read_text(encoding="utf-8"))
            self.assertEqual(disk["construct"], "EXECUTED_AGGRESSOR_VOLUME")


if __name__ == "__main__":
    unittest.main()
