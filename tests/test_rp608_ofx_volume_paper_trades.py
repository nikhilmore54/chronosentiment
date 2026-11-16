"""OFX volume paper book — opposing-volume match exit (−10%..+10%)."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow.volume_mark import is_ofx_opposing_volume_match
from scripts.rp608_ofx_volume_paper_trades import OfxVolumePaperManager


def _vol_snap(*, buy: float, sell: float, now: int = 1_700_000_060_000) -> dict:
    tot = buy + sell
    signed = buy - sell
    return {
        "symbol": "BTCUSDT",
        "window_start_ms": now - 60_000,
        "window_end_ms": now + 1,
        "window_ms": 60_000,
        "buy_aggressor_qty": buy,
        "sell_aggressor_qty": sell,
        "total_classified_qty": tot,
        "signed_delta_qty": signed,
        "classification_coverage": 1.0,
        "data_quality": "OK",
        "status": "OK",
        "journal_path": "test",
        "inspection": {
            "ofx_volume_imbalance": signed / tot if tot else None,
        },
    }


def _quote(bid=100.0, ask=100.1):
    return {
        "best_bid_price": bid,
        "best_ask_price": ask,
        "best_bid_qty": 1.0,
        "best_ask_qty": 1.0,
        "quote_source": "test",
    }


class TestOpposingVolumeMatch(unittest.TestCase):
    def test_match_within_pm10(self):
        m = is_ofx_opposing_volume_match(
            entry_aggressor_qty=1.0, opposite_aggressor_qty=0.95
        )
        self.assertTrue(m["match"])
        self.assertAlmostEqual(m["match_deviation"], 0.05, places=5)

    def test_exact_match_allowed(self):
        m = is_ofx_opposing_volume_match(
            entry_aggressor_qty=1.0, opposite_aggressor_qty=1.0
        )
        self.assertTrue(m["match"])
        self.assertAlmostEqual(m["match_deviation"], 0.0, places=5)

    def test_edge_plus_10pct(self):
        m = is_ofx_opposing_volume_match(
            entry_aggressor_qty=1.0, opposite_aggressor_qty=1.10
        )
        self.assertTrue(m["match"])

    def test_former_pm6_edge_still_matches(self):
        m = is_ofx_opposing_volume_match(
            entry_aggressor_qty=1.0, opposite_aggressor_qty=1.06
        )
        self.assertTrue(m["match"])

    def test_not_enough_opposing(self):
        m = is_ofx_opposing_volume_match(
            entry_aggressor_qty=1.0, opposite_aggressor_qty=0.50
        )
        self.assertFalse(m["match"])

    def test_overshoot_past_10pct_does_not_match(self):
        m = is_ofx_opposing_volume_match(
            entry_aggressor_qty=1.0, opposite_aggressor_qty=1.11
        )
        self.assertFalse(m["match"])
        self.assertTrue(m["match_overshoot"])
        self.assertEqual(m["reason"], "overshoot_outside_pm10_band")


def _surge_ok(*, direction="SHORT", entry_volume=9.0):
    side = "SELL" if direction == "SHORT" else "BUY"
    return {
        "passed": True,
        "ofx_entry_surge_label": "OFX_ENTRY_VOLUME_SURGE",
        "entry_volume": entry_volume,
        "entry_aggressor_side": side,
        "entry_tick_timestamp_ms": 1_000_000,
        "avg_preceding_volume": entry_volume / 3.0,
        "surge_threshold": entry_volume,
        "surge_ratio": 3.0,
        "surge_multiple": 3.0,
        "lookback_required": 20,
        "preceding_tick_count": 20,
        "order_flow_signals_excluded": True,
        "baseline_of_inert": True,
        "reason": "surge_pass",
    }


class TestOfxVolumePaper(unittest.TestCase):
    def test_open_on_volume_support_and_close_on_opposing_match(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = OfxVolumePaperManager(Path(td), notional_usdt=100.0)
            with patch.object(mgr, "_entry_surge_gate", return_value=_surge_ok()):
                opened = mgr.try_open_from_decision(
                    direction="SHORT",
                    volume_snapshot=_vol_snap(buy=1.0, sell=9.0, now=1_000_000),
                    quote=_quote(),
                    plan_id="P1",
                    bar_idx=10,
                    now_ms=1_000_000,
                )
            self.assertIsNotNone(opened)
            assert opened is not None
            self.assertEqual(opened["direction"], "SHORT")
            self.assertEqual(opened["entry_aggressor_qty"], 9.0)
            self.assertEqual(opened["exit_match_aggressor_side"], "BUY")
            self.assertTrue(opened["baseline_of_inert"])
            self.assertTrue(opened["ofx_entry_surge"]["passed"])

            with patch.object(mgr, "_sum_opposite_aggressor_since_entry", return_value=50.0):
                still = mgr.evaluate_tick(
                    volume_snapshot=_vol_snap(buy=50.0, sell=1.0, now=1_001_000),
                    quote=_quote(bid=99.9, ask=100.0),
                    now_ms=1_001_000,
                )
                self.assertTrue(still and still.get("active"))
                self.assertEqual(still.get("ofx_match_reason"), "overshoot_outside_pm10_band")

            # Window buy ~4.4% off entry sell 9.0 → inside ±10%
            t_arm = 1_000_000 + 300_000 + 1
            with patch.object(mgr, "_sum_opposite_aggressor_since_entry", return_value=0.0):
                armed = mgr.evaluate_tick(
                    volume_snapshot=_vol_snap(buy=8.6, sell=1.0, now=t_arm),
                    quote=_quote(bid=99.5, ask=99.6),
                    now_ms=t_arm,
                )
                self.assertTrue(armed and armed.get("active"))
                self.assertIsNotNone(armed.get("opposite_exit_seen_ms"))

                t_close = t_arm + 30_000
                closed = mgr.evaluate_tick(
                    volume_snapshot=_vol_snap(buy=8.6, sell=1.0, now=t_close),
                    quote=_quote(bid=99.0, ask=99.1),
                    now_ms=t_close,
                )
            self.assertIsNotNone(closed)
            assert closed is not None
            self.assertFalse(closed.get("active", True))
            self.assertEqual(closed["exit_reason"], "OPPOSING_VOLUME_MATCH")
            self.assertIn("exit_match_deviation", closed)
            lines = (Path(td) / "ofx_volume_paper_outcomes.jsonl").read_text().strip().splitlines()
            self.assertEqual(len(lines), 1)

    def test_no_open_without_surge(self):
        with tempfile.TemporaryDirectory() as td:
            mgr = OfxVolumePaperManager(Path(td))
            with patch.object(
                mgr,
                "_entry_surge_gate",
                return_value={"passed": False, "reason": "surge_below_multiple"},
            ):
                opened = mgr.try_open_from_decision(
                    direction="LONG",
                    volume_snapshot=_vol_snap(buy=9.0, sell=1.0),
                    quote=_quote(),
                    bar_idx=1,
                    now_ms=2_000_000,
                )
            self.assertTrue(opened is None or not (opened or {}).get("active"))


if __name__ == "__main__":
    unittest.main()
