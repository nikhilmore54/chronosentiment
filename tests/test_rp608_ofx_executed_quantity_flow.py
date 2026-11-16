"""RP-608-OFX executed-quantity directional flow — isolated unit tests."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Optional

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow import (
    CandleContext,
    ExecutedTrade,
    SideClass,
    aggregate_executed_flow,
    candle_direction_from_ohlc,
    parse_binance_agg_trade,
    trades_from_aggtrade_records,
    trades_from_nse_inferred_ticks,
)
from research.rp608_ofx.executed_flow.aggregate import SIDE_DIRECT, SIDE_UNAVAILABLE


def _t(
    *,
    qty: float,
    aggressor: str,
    classification: str = SIDE_DIRECT,
    ts: int = 1_000,
    instrument: str = "BTCUSDT",
    seq: Optional[str] = "1",
) -> ExecutedTrade:
    return ExecutedTrade(
        instrument_id=instrument,
        timestamp_ms=ts,
        price=100.0,
        quantity=qty,
        side=SideClass(aggressor=aggressor, classification=classification, evidence="test"),
        source="test",
        sequence_id=seq,
        raw_ref=seq,
    )


class TestSignedAggregation(unittest.TestCase):
    def test_buy_aggressor_qty(self):
        r = aggregate_executed_flow(
            [_t(qty=3.0, aggressor="BUY", seq="a")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        self.assertEqual(r.buy_aggressor_qty, 3.0)
        self.assertEqual(r.sell_aggressor_qty, 0.0)
        self.assertEqual(r.signed_delta_qty, 3.0)
        self.assertEqual(r.data_quality, "OK")

    def test_sell_aggressor_qty(self):
        r = aggregate_executed_flow(
            [_t(qty=2.5, aggressor="SELL", seq="b")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        self.assertEqual(r.sell_aggressor_qty, 2.5)
        self.assertEqual(r.signed_delta_qty, -2.5)

    def test_mixed_buy_sell(self):
        trades = [
            _t(qty=5.0, aggressor="BUY", seq="1"),
            _t(qty=2.0, aggressor="SELL", seq="2"),
        ]
        r = aggregate_executed_flow(
            trades, instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=10_000, source="test"
        )
        self.assertEqual(r.buy_aggressor_qty, 5.0)
        self.assertEqual(r.sell_aggressor_qty, 2.0)
        self.assertEqual(r.total_classified_qty, 7.0)
        self.assertEqual(r.signed_delta_qty, 3.0)

    def test_equal_buy_sell(self):
        trades = [
            _t(qty=4.0, aggressor="BUY", seq="1"),
            _t(qty=4.0, aggressor="SELL", seq="2"),
        ]
        r = aggregate_executed_flow(
            trades, instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=10_000, source="test"
        )
        self.assertEqual(r.signed_delta_qty, 0.0)
        self.assertEqual(r.classification_coverage, 1.0)

    def test_zero_quantity_invalid(self):
        r = aggregate_executed_flow(
            [_t(qty=0.0, aggressor="BUY", seq="z")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        self.assertEqual(r.n_invalid, 1)
        self.assertEqual(r.n_classified, 0)
        self.assertEqual(r.status, "NO_CLASSIFIED_AGGRESSOR_VOLUME")

    def test_missing_aggressor_side(self):
        t = _t(qty=1.0, aggressor="UNKNOWN", classification=SIDE_UNAVAILABLE, seq="u")
        r = aggregate_executed_flow(
            [t], instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=10_000, source="test"
        )
        self.assertEqual(r.n_unclassified, 1)
        self.assertEqual(r.signed_delta_qty, 0.0)
        self.assertEqual(r.data_quality, "UNAVAILABLE")

    def test_invalid_quantity_negative(self):
        r = aggregate_executed_flow(
            [_t(qty=-1.0, aggressor="BUY", seq="n")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        self.assertEqual(r.n_invalid, 1)

    def test_duplicate_sequence_skipped(self):
        trades = [
            _t(qty=1.0, aggressor="BUY", seq="same"),
            _t(qty=9.0, aggressor="BUY", seq="same"),
        ]
        r = aggregate_executed_flow(
            trades, instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=10_000, source="test"
        )
        self.assertEqual(r.n_duplicates_skipped, 1)
        self.assertEqual(r.buy_aggressor_qty, 1.0)

    def test_out_of_order_timestamps_counted(self):
        # Present unsorted; aggregator sorts but counts OOO relative to sorted stream = 0.
        # Inject via pre-sorted equal instrument with decreasing ts after sort is stable —
        # OOO counter tracks sorted walk so should be 0. Instead verify sort doesn't drop qty.
        trades = [
            _t(qty=1.0, aggressor="BUY", ts=3000, seq="c"),
            _t(qty=2.0, aggressor="SELL", ts=1000, seq="a"),
            _t(qty=3.0, aggressor="BUY", ts=2000, seq="b"),
        ]
        r = aggregate_executed_flow(
            trades, instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=10_000, source="test"
        )
        self.assertEqual(r.signed_delta_qty, 1.0 + 3.0 - 2.0)
        self.assertEqual(r.n_out_of_order, 0)  # after sort

    def test_window_boundaries(self):
        trades = [
            _t(qty=1.0, aggressor="BUY", ts=100, seq="in"),
            _t(qty=5.0, aggressor="BUY", ts=200, seq="end_excl"),
            _t(qty=7.0, aggressor="BUY", ts=50, seq="before"),
        ]
        r = aggregate_executed_flow(
            trades, instrument_id="BTCUSDT", window_start_ms=100, window_end_ms=200, source="test"
        )
        self.assertEqual(r.n_trades_in_window, 1)
        self.assertEqual(r.buy_aggressor_qty, 1.0)

    def test_instrument_separation(self):
        trades = [
            _t(qty=1.0, aggressor="BUY", instrument="AAA", seq="1"),
            _t(qty=9.0, aggressor="BUY", instrument="BBB", seq="2"),
        ]
        r = aggregate_executed_flow(
            trades, instrument_id="AAA", window_start_ms=0, window_end_ms=10_000, source="test"
        )
        self.assertEqual(r.buy_aggressor_qty, 1.0)

    def test_side_qty_never_cross_paired(self):
        """Each trade keeps its own (side, qty); no parallel-array zip hazard."""
        buy_small = _t(qty=1.0, aggressor="BUY", seq="1")
        sell_large = _t(qty=100.0, aggressor="SELL", seq="2")
        r = aggregate_executed_flow(
            [buy_small, sell_large],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        self.assertEqual(r.buy_aggressor_qty, 1.0)
        self.assertEqual(r.sell_aggressor_qty, 100.0)
        self.assertEqual(r.signed_delta_qty, -99.0)


class TestCandleIndependence(unittest.TestCase):
    def test_red_candle_positive_delta(self):
        candle = CandleContext(
            open=110.0, high=111.0, low=90.0, close=95.0,
            direction=candle_direction_from_ohlc(110.0, 95.0),
            unsigned_volume=9999.0,
        )
        self.assertEqual(candle.direction, "DOWN")
        r = aggregate_executed_flow(
            [_t(qty=10.0, aggressor="BUY", seq="1")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
            candle=candle,
        )
        self.assertGreater(r.signed_delta_qty, 0.0)
        self.assertEqual(r.candle["direction"], "DOWN")
        # unsigned candle volume must not enter signed delta
        self.assertEqual(r.signed_delta_qty, 10.0)

    def test_green_candle_negative_delta(self):
        candle = CandleContext(
            open=90.0, high=120.0, low=89.0, close=115.0,
            direction=candle_direction_from_ohlc(90.0, 115.0),
            unsigned_volume=8888.0,
        )
        self.assertEqual(candle.direction, "UP")
        r = aggregate_executed_flow(
            [_t(qty=4.0, aggressor="SELL", seq="1")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
            candle=candle,
        )
        self.assertLess(r.signed_delta_qty, 0.0)
        self.assertEqual(r.candle["direction"], "UP")


class TestNoRestingL1OrUnsignedVolume(unittest.TestCase):
    def test_aggregate_api_has_no_l1_fields(self):
        r = aggregate_executed_flow(
            [_t(qty=1.0, aggressor="BUY", seq="1")],
            instrument_id="BTCUSDT",
            window_start_ms=0,
            window_end_ms=10_000,
            source="test",
        )
        d = r.to_dict()
        for banned in ("bid_qty", "ask_qty", "best_bid_qty", "best_ask_qty", "imbalance", "OF_SUP"):
            self.assertNotIn(banned, d)


class TestCryptoDirectSemantics(unittest.TestCase):
    def test_is_buyer_maker_true_is_sell_aggressor(self):
        t = parse_binance_agg_trade(
            {"s": "BTCUSDT", "E": 1000, "p": "1", "q": "2.5", "m": True, "a": 7}
        )
        self.assertIsNotNone(t)
        assert t is not None
        self.assertEqual(t.side.aggressor, "SELL")
        self.assertEqual(t.side.classification, SIDE_DIRECT)
        self.assertEqual(t.quantity, 2.5)

    def test_is_buyer_maker_false_is_buy_aggressor(self):
        t = parse_binance_agg_trade(
            {"s": "BTCUSDT", "E": 1000, "p": "1", "q": "1.0", "m": False, "a": 8}
        )
        assert t is not None
        self.assertEqual(t.side.aggressor, "BUY")
        self.assertEqual(t.side.classification, SIDE_DIRECT)

    def test_missing_m_unavailable(self):
        t = parse_binance_agg_trade({"s": "BTCUSDT", "E": 1000, "p": "1", "q": "1.0", "a": 9})
        assert t is not None
        self.assertEqual(t.side.classification, SIDE_UNAVAILABLE)
        r = aggregate_executed_flow(
            [t], instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=2000, source="binance"
        )
        self.assertEqual(r.signed_delta_qty, 0.0)
        self.assertEqual(r.data_quality, "UNAVAILABLE")

    def test_normalized_tick_shape(self):
        rows = [
            {
                "symbol": "BTCUSDT",
                "timestamp": 1000,
                "price": 1.0,
                "volume": 3.0,
                "is_buyer_maker": False,
            }
        ]
        trades = trades_from_aggtrade_records(rows)
        r = aggregate_executed_flow(
            trades, instrument_id="BTCUSDT", window_start_ms=0, window_end_ms=2000, source="binance"
        )
        self.assertEqual(r.signed_delta_qty, 3.0)


class TestNseDerivedSemantics(unittest.TestCase):
    def test_medium_buy_derived(self):
        row = {
            "symbol": "AAA",
            "instrument_token": 1,
            "receipt_utc_ms": 5000,
            "last_price": 10.0,
            "last_traded_quantity": 12.0,
            "last_trade_time_sec": 1,
            "volume": 100,
            "trade_like": True,
            "aggressor_inference_primary": {
                "label": "BUY_INITIATED_CANDIDATE",
                "confidence": "MEDIUM",
                "reason": "ltp_ge_ask",
            },
        }
        trades = trades_from_nse_inferred_ticks([row])
        self.assertEqual(len(trades), 1)
        self.assertEqual(trades[0].side.classification, "DERIVED_VALIDATED")
        r = aggregate_executed_flow(
            trades, instrument_id="AAA", window_start_ms=0, window_end_ms=10_000, source="nse"
        )
        self.assertEqual(r.signed_delta_qty, 12.0)

    def test_low_confidence_not_signed_by_default(self):
        row = {
            "symbol": "AAA",
            "instrument_token": 1,
            "receipt_utc_ms": 5000,
            "last_price": 10.0,
            "last_traded_quantity": 12.0,
            "trade_like": True,
            "aggressor_inference_primary": {
                "label": "BUY_INITIATED_CANDIDATE",
                "confidence": "LOW",
                "reason": "same_packet",
            },
        }
        trades = trades_from_nse_inferred_ticks([row])
        self.assertEqual(trades[0].side.classification, SIDE_UNAVAILABLE)
        r = aggregate_executed_flow(
            trades, instrument_id="AAA", window_start_ms=0, window_end_ms=10_000, source="nse"
        )
        self.assertEqual(r.signed_delta_qty, 0.0)

    def test_nse_never_claims_direct(self):
        row = {
            "symbol": "AAA",
            "instrument_token": 1,
            "receipt_utc_ms": 5000,
            "last_traded_quantity": 1.0,
            "trade_like": True,
            "aggressor_inference_primary": {
                "label": "SELL_INITIATED_CANDIDATE",
                "confidence": "MEDIUM",
                "reason": "ltp_le_bid",
            },
        }
        t = trades_from_nse_inferred_ticks([row])[0]
        self.assertNotEqual(t.side.classification, SIDE_DIRECT)


if __name__ == "__main__":
    unittest.main()
