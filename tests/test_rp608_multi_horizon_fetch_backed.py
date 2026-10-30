"""Multi-Horizon Books must prefer RP-608 fetch artifacts over demo journals."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock
import sys

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts import rp608_multi_horizon_status as mh


class TestMultiHorizonFetchBacked(unittest.TestCase):
    def test_demo_observation_filter(self):
        self.assertTrue(mh._is_demo_observation({"thesis": "pi_demo_swing"}))
        self.assertTrue(mh._is_demo_observation({"thesis": "demo multi-day swing"}))
        self.assertTrue(mh._is_demo_observation({"plan_id": "observe_build_001"}))
        self.assertFalse(mh._is_demo_observation({"thesis": "rp608_nse_active_trade", "symbol": "INFY.NS"}))

    def test_h0_nse_from_pending_active_trades(self):
        with tempfile.TemporaryDirectory() as td:
            nse = Path(td)
            pending = {
                "active_trades": {
                    "INFY.NS": {
                        "symbol": "INFY.NS",
                        "side": "LONG",
                        "entry_price": 1175.0,
                        "stop_price": 1163.25,
                        "target_price": 1192.62,
                        "entry_bar_idx": 10,
                        "status": "OPEN",
                    }
                }
            }
            (nse / "rp608_nse_pending_evaluations.json").write_text(json.dumps(pending), encoding="utf-8")
            with mock.patch.object(mh, "NSE_DIR", nse):
                recs = mh._rp608_nse_h0_recs({"INFY.NS": 1175.0})
            self.assertEqual(len(recs), 1)
            self.assertEqual(recs[0]["horizon_class"], "INTRADAY")
            self.assertEqual(recs[0]["source"], "RP-608-NSE-FETCH")
            self.assertEqual(recs[0]["entry_price"], 1175.0)
            self.assertEqual(recs[0]["mark_price"], 1175.0)

    def test_crypto_h0_from_overlay(self):
        with tempfile.TemporaryDirectory() as td:
            ov = Path(td) / "overlay_state.json"
            ov.write_text(
                json.dumps(
                    {
                        "active_plan": {
                            "plan_id": "PLAN-1-SHORT-1",
                            "direction": "SHORT",
                            "state": "OPEN",
                            "entry_price": 77561.0,
                            "stop_loss_price": 77796.35,
                            "profit_target_price": 77207.97,
                            "last_mark_price": 77657.18,
                            "atr_14": 235.0,
                            "signal_cadence": "tick_book_ticker",
                        }
                    }
                ),
                encoding="utf-8",
            )
            with mock.patch.object(mh, "CRYPTO_OVERLAY_STATE", ov):
                recs = mh._rp608_crypto_h0_recs()
            self.assertEqual(len(recs), 1)
            self.assertEqual(recs[0]["side"], "SHORT")
            self.assertEqual(recs[0]["source"], "RP-608-CRYPTO-FETCH")
            self.assertEqual(recs[0]["mark_price"], 77657.18)


if __name__ == "__main__":
    unittest.main()
