"""NSE OFX volume paper + muhurat session helpers."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from datetime import date, datetime
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_nse_muhurat_session import (  # noqa: E402
    completed_bars_since_open,
    muhurat_window_for,
    resolve_session_window,
    session_phase,
)
from scripts.rp608_ofx_nse_volume_flow_mark import (  # noqa: E402
    OfxNseVolumeFlowBuffer,
    build_ofx_nse_volume_flow_snapshot,
)
from scripts.rp608_ofx_nse_volume_paper_trades import OfxNseVolumePaperManager  # noqa: E402


def _vol_snap(*, buy: float, sell: float, symbol: str = "RELIANCE.NS", now: int = 1_700_000_060_000) -> dict:
    tot = buy + sell
    signed = buy - sell
    return {
        "symbol": symbol,
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
        "buffer_rows_in_window": 10,
        "phase": "test",
    }


def _quote(bid=1000.0, ask=1000.5, bid_qty=500.0, ask_qty=400.0):
    return {
        "best_bid_price": bid,
        "best_ask_price": ask,
        "best_bid_qty": bid_qty,
        "best_ask_qty": ask_qty,
        "bid_price": bid,
        "ask_price": ask,
        "bid_qty": bid_qty,
        "ask_qty": ask_qty,
    }


class TestMuhuratSession(unittest.TestCase):
    def test_muhurat_2026_11_08_overrides_sunday_holiday(self):
        # Sunday Diwali muhurat — must not be WEEKEND_CLOSED
        now = datetime(2026, 11, 8, 14, 25, 0)  # IST wall
        win, reason = resolve_session_window(now, exchange_holidays={date(2026, 11, 8)})
        self.assertEqual(reason, "")
        self.assertIsNotNone(win)
        assert win is not None
        self.assertEqual(win.kind, "MUHURAT")
        active, bar, phase = session_phase(now, win)
        self.assertTrue(active)
        self.assertEqual(phase, "ACTIVE_MUHURAT_SESSION")
        self.assertEqual(bar, 1)
        self.assertEqual(completed_bars_since_open(now, win), 0)

        mid = datetime(2026, 11, 8, 14, 40, 0)
        self.assertEqual(completed_bars_since_open(mid, win), 1)

    def test_pre_muhurat_still_closed(self):
        now = datetime(2026, 11, 8, 10, 0, 0)
        win = muhurat_window_for(date(2026, 11, 8))
        assert win is not None
        active, bar, phase = session_phase(now, win)
        self.assertFalse(active)
        self.assertEqual(phase, "PRE_OPEN_MUHURAT")


def _nse_surge_ok(*, entry_volume=90.0, side="BUY"):
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


class TestOfxNseVolumePaper(unittest.TestCase):
    def test_open_on_recommend_long_with_volume_support(self):
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as td:
            mgr = OfxNseVolumePaperManager(Path(td))
            with patch.object(mgr, "_entry_surge_gate", return_value=_nse_surge_ok()):
                opened = mgr.try_open_from_eval(
                    symbol="RELIANCE.NS",
                    eval_record={"decision": "RECOMMEND_LONG", "bar_idx": 1, "reference_price": 1000.5},
                    volume_snapshot=_vol_snap(buy=90.0, sell=10.0),
                    quote=_quote(),
                    now_ms=1_000_000,
                )
            self.assertIsNotNone(opened)
            assert opened is not None
            self.assertTrue(opened["active"])
            self.assertEqual(opened["direction"], "LONG")
            self.assertEqual(opened["construct"], "DERIVED_VALIDATED_AGGRESSOR_VOLUME")
            self.assertTrue(opened["baseline_of_inert"])
            self.assertEqual(opened["entry_aggressor_qty"], 90.0)

            # No surge → no open another symbol
            with patch.object(
                mgr,
                "_entry_surge_gate",
                return_value={"passed": False, "reason": "surge_below_multiple"},
            ):
                none = mgr.try_open_from_eval(
                    symbol="TCS.NS",
                    eval_record={"decision": "RECOMMEND_LONG", "bar_idx": 1},
                    volume_snapshot=_vol_snap(buy=90.0, sell=10.0, symbol="TCS.NS"),
                    quote=_quote(),
                    now_ms=1_000_100,
                )
            self.assertIsNone(none)

            self.assertEqual(opened.get("entry_aggressor_side"), "BUY")
            self.assertEqual(opened.get("exit_match_aggressor_side"), "SELL")

            # Window sell ~4.4% off entry buy 90 → inside ±10% band
            t_arm = 1_000_000 + 300_000 + 1
            with patch.object(mgr, "_sum_opposite_aggressor_since_entry", return_value=0.0):
                armed = mgr.evaluate_open_tick(
                    symbol="RELIANCE.NS",
                    volume_snapshot=_vol_snap(buy=10.0, sell=86.0, now=t_arm),
                    quote=_quote(bid=999.0, ask=999.5),
                    now_ms=t_arm,
                )
                self.assertTrue(armed and armed.get("active"))
                t_close = t_arm + 30_000
                closed = mgr.evaluate_open_tick(
                    symbol="RELIANCE.NS",
                    volume_snapshot=_vol_snap(buy=10.0, sell=86.0, now=t_close),
                    quote=_quote(bid=998.0, ask=998.5),
                    now_ms=t_close,
                )
            self.assertIsNotNone(closed)
            assert closed is not None
            self.assertEqual(closed["exit_reason"], "OPPOSING_VOLUME_MATCH")
            self.assertIn("net_pnl_inr", closed)
            summ = mgr.summary()
            self.assertEqual(summ["n_trades"], 1)
            self.assertEqual(summ["n_open"], 0)

    def test_buffer_builds_derived_snapshot(self):
        buf = OfxNseVolumeFlowBuffer()
        now = 1_800_000_000_000
        for i in range(5):
            buf.ingest_row(
                {
                    "symbol": "INFY.NS",
                    "trade_like": True,
                    "receipt_utc_ms": now - 10_000 + i,
                    "last_price": 1500.0,
                    "last_traded_quantity": 100.0,
                    "aggressor_inference_primary": {
                        "label": "BUY_INITIATED_CANDIDATE",
                        "confidence": "MEDIUM",
                        "reason": "test",
                    },
                    "instrument_token": 1,
                    "last_trade_time_sec": 1,
                    "volume": float(i + 1),
                }
            )
        snap = build_ofx_nse_volume_flow_snapshot(
            symbol="INFY.NS", buffer=buf, now_ms=now, window_ms=60_000
        )
        self.assertEqual(snap["side_classification_mode"], "DERIVED_VALIDATED")
        self.assertEqual(snap["symbol"], "INFY.NS")
        self.assertGreater(snap["buy_aggressor_qty"], 0)
        self.assertTrue(snap["baseline_of_inert"])

    def test_open_short_on_recommend_short_with_volume_support(self):
        """NSE OFX mirrors crypto: SHORT decision + 3× surge on sell tick."""
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as td:
            mgr = OfxNseVolumePaperManager(Path(td))
            with patch.object(
                mgr,
                "_entry_surge_gate",
                return_value=_nse_surge_ok(entry_volume=90.0, side="SELL"),
            ):
                opened = mgr.try_open_from_eval(
                    symbol="TCS.NS",
                    eval_record={
                        "decision": "RECOMMEND_SHORT",
                        "bar_idx": 3,
                        "reference_price": 1000.0,
                    },
                    volume_snapshot=_vol_snap(buy=10.0, sell=90.0, symbol="TCS.NS"),
                    quote=_quote(),
                    now_ms=2_000_000,
                )
            self.assertIsNotNone(opened)
            assert opened is not None
            self.assertEqual(opened["direction"], "SHORT")
            self.assertEqual(opened["entry_aggressor_side"], "SELL")
            self.assertEqual(opened["exit_match_aggressor_side"], "BUY")
            self.assertEqual(opened["entry_aggressor_qty"], 90.0)
            self.assertEqual(opened["ofx_match_dev_min"], 0.0)
            self.assertEqual(opened["ofx_match_dev_max"], 0.10)

            t_arm = 2_000_000 + 300_000 + 1
            with patch.object(mgr, "_sum_opposite_aggressor_since_entry", return_value=0.0):
                armed = mgr.evaluate_open_tick(
                    symbol="TCS.NS",
                    volume_snapshot=_vol_snap(buy=86.0, sell=10.0, symbol="TCS.NS", now=t_arm),
                    quote=_quote(bid=1001.0, ask=1001.5),
                    now_ms=t_arm,
                )
                self.assertTrue(armed and armed.get("active"))
                closed = mgr.evaluate_open_tick(
                    symbol="TCS.NS",
                    volume_snapshot=_vol_snap(
                        buy=86.0, sell=10.0, symbol="TCS.NS", now=t_arm + 30_000
                    ),
                    quote=_quote(bid=1002.0, ask=1002.5),
                    now_ms=t_arm + 30_000,
                )
            self.assertIsNotNone(closed)
            assert closed is not None
            self.assertEqual(closed["exit_reason"], "OPPOSING_VOLUME_MATCH")
            self.assertEqual(closed.get("exit_match_qty_source"), "window_opposing_aggressor")

    def test_legacy_open_restamped_to_pm10_band(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            # Simulate pre-±10% open still stamped ±6% / 2–8%
            legacy = {
                "open": True,
                "n_open": 1,
                "trades": [
                    {
                        "active": True,
                        "symbol": "INFY.NS",
                        "direction": "LONG",
                        "ofx_trade_id": "LEGACY-1",
                        "entry_price": 1000.0,
                        "entry_aggressor_qty": 100.0,
                        "buy_aggressor_qty": 100.0,
                        "sell_aggressor_qty": 10.0,
                        "simulated_quantity": 1.0,
                        "entry_paper_quantity": 1.0,
                        "simulated_notional_inr": 1000.0,
                        "activated_ms": 1_000_000,
                        "ofx_match_dev_min": 0.0,
                        "ofx_match_dev_max": 0.06,
                        "exit_rule": "per_symbol_opposing_volume_match_pm6pct",
                    }
                ],
            }
            (root / "ofx_nse_volume_paper_open.json").write_text(
                json.dumps(legacy), encoding="utf-8"
            )
            mgr = OfxNseVolumePaperManager(root)
            opened = mgr.open_snapshot()[0]
            self.assertEqual(opened["ofx_match_dev_min"], 0.0)
            self.assertEqual(opened["ofx_match_dev_max"], 0.10)
            self.assertIn("pm10pct", opened["exit_rule"])

    def test_opposing_qty_is_per_symbol_only(self):
        """RELIANCE open must not close on TCS opposing volume."""
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as td:
            mgr = OfxNseVolumePaperManager(Path(td))
            with patch.object(mgr, "_entry_surge_gate", return_value=_nse_surge_ok()):
                opened = mgr.try_open_from_eval(
                    symbol="RELIANCE.NS",
                    eval_record={"decision": "RECOMMEND_LONG", "bar_idx": 2, "reference_price": 1000.5},
                    volume_snapshot=_vol_snap(buy=90.0, sell=10.0, symbol="RELIANCE.NS"),
                    quote=_quote(),
                    now_ms=3_000_000,
                )
            self.assertIsNotNone(opened)
            assert opened is not None
            self.assertEqual(opened["match_symbol"], "RELIANCE.NS")
            self.assertEqual(opened["ofx_match_scope"], "PER_SYMBOL_ONLY")

            t_arm = 3_000_000 + 300_000 + 1
            # Simulate: sum for RELIANCE is tiny; even if TCS has huge sell flow elsewhere
            with patch.object(mgr, "_sum_opposite_aggressor_since_entry", return_value=2.0) as mocked:
                still = mgr.evaluate_open_tick(
                    symbol="RELIANCE.NS",
                    volume_snapshot=_vol_snap(buy=10.0, sell=90.0, symbol="RELIANCE.NS", now=t_arm),
                    quote=_quote(bid=999.0, ask=999.5),
                    now_ms=t_arm,
                )
                self.assertTrue(still and still.get("active"))
                # Must request opposing qty with RELIANCE key only
                self.assertEqual(mocked.call_args.kwargs.get("symbol"), "RELIANCE.NS")

            # Wrong-symbol snapshot must not drive exit
            blocked = mgr.evaluate_open_tick(
                symbol="RELIANCE.NS",
                volume_snapshot=_vol_snap(buy=10.0, sell=90.0, symbol="TCS.NS", now=t_arm),
                quote=_quote(),
                now_ms=t_arm,
            )
            self.assertTrue(blocked and blocked.get("active"))
            self.assertIn("symbol_mismatch", str(blocked.get("close_suppressed_reason") or ""))


if __name__ == "__main__":
    unittest.main()
