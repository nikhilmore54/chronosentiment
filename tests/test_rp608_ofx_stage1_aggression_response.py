#!/usr/bin/env python3
"""Unit tests for OFX Stage 1 aggression→response helpers."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.observer.aggression_response import (
    TickPoint,
    aggregate_observations,
    observe_event,
    response_class,
)


def _pt(ms: int, mid: float, *, trade_like: bool, label: str, ltq: float = 10.0) -> TickPoint:
    return TickPoint(
        receipt_utc_ms=ms,
        ltp=mid,
        bid=mid - 0.05,
        ask=mid + 0.05,
        mid=mid,
        trade_like=trade_like,
        infer_label=label,
        infer_confidence="MEDIUM",
        ltq=ltq,
        volume=1000.0,
        bid_qty=100.0,
        ask_qty=100.0,
        update_class="TRADE_LIKE_VOLUME_AND_LT_CHANGE",
        symbol="TEST.NS",
        instrument_token=1,
    )


class TestStage1Observer(unittest.TestCase):
    def test_response_class(self):
        self.assertEqual(response_class(1, 10.0), "ADVANCES_WITH")
        self.assertEqual(response_class(1, -10.0), "ADVANCES_AGAINST")
        self.assertEqual(response_class(-1, -10.0), "ADVANCES_WITH")
        self.assertEqual(response_class(1, 0.5), "FLAT")

    def test_observe_buy_advances(self):
        points = [
            _pt(0, 100.0, trade_like=True, label="BUY_INITIATED_CANDIDATE"),
            _pt(1000, 100.10, trade_like=False, label="AMBIGUOUS"),
            _pt(5000, 100.20, trade_like=True, label="BUY_INITIATED_CANDIDATE"),
            _pt(15000, 100.30, trade_like=False, label="AMBIGUOUS"),
            _pt(30000, 100.40, trade_like=False, label="AMBIGUOUS"),
        ]
        obs = observe_event(points, 0)
        self.assertIsNotNone(obs)
        assert obs is not None
        self.assertEqual(obs["stage2_mechanism_label"], "NOT_ASSIGNED_STAGE_1")
        self.assertEqual(obs["price_response"]["5000"]["response_class"], "ADVANCES_WITH")
        agg = aggregate_observations([obs])
        self.assertEqual(agg["n_events"], 1)


if __name__ == "__main__":
    unittest.main()
