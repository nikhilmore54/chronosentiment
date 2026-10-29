#!/usr/bin/env python3
"""
tests/test_rp608_historical_replay.py

Unit test suite for RP-608 Phase 3 Standalone Offline Replay Harness.
Verifies causal execution physics, zero look-ahead bias, conservative resolution,
and statutory friction calculations.
"""

from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from scripts.replay_rp608_kite_history import (
    RP608HistoricalReplayHarness,
    ReplayBar,
    calc_indian_equity_friction
)


class TestRP608HistoricalReplay(unittest.TestCase):
    def setUp(self):
        self.test_dir = Path(tempfile.mkdtemp(prefix="test_rp608_replay_"))
        self.data_dir = self.test_dir / "data"
        self.output_dir = self.test_dir / "output"
        self.data_dir.mkdir(parents=True)
        self.output_dir.mkdir(parents=True)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_01_session_bar_reconciliation_25_bars(self):
        """Verifies session reconciliation accurately identifies 25-bar regular sessions."""
        sym = "TCS.NS"
        raw_bars = []
        # Construct 25 bars for 2026-07-06 from 09:15 to 15:15 IST starts
        # 09:15 IST = 03:45 UTC
        base_utc = int(datetime(2026, 7, 6, 3, 45, tzinfo=timezone.utc).timestamp())
        for i in range(25):
            raw_bars.append({
                "timestamp": base_utc + (i * 900),
                "open": 3000.0 + i,
                "high": 3005.0 + i,
                "low": 2995.0 + i,
                "close": 3002.0 + i,
                "volume": 1000.0
            })

        with open(self.data_dir / f"{sym}.json", "w", encoding="utf-8") as f:
            json.dump(raw_bars, f)

        harness = RP608HistoricalReplayHarness(
            data_dir=self.data_dir,
            output_dir=self.output_dir,
            symbols=[sym]
        )
        bars, audit = harness.load_and_validate_symbol_data(sym)
        self.assertEqual(len(bars), 25)
        self.assertEqual(audit["complete_25_bar_sessions"], 1)
        self.assertEqual(audit["incomplete_sessions"], 0)
        self.assertEqual(bars[0].bar_idx, 1)
        self.assertEqual(bars[-1].bar_idx, 25)

    def test_02_zero_lookahead_entry_at_next_bar_open(self):
        """Verifies signal generated at bar k close enters strictly at bar k+1 open with friction."""
        sym = "INFY.NS"
        raw_bars = []
        base_utc = int(datetime(2026, 7, 6, 3, 45, tzinfo=timezone.utc).timestamp())
        # First 4 bars flat, Bar 5 strong breakout -> signal at Bar 5 Close
        for i in range(5):
            raw_bars.append({
                "timestamp": base_utc + (i * 900),
                "open": 1000.0,
                "high": 1002.0 if i < 4 else 1100.0,
                "low": 998.0,
                "close": 1000.0 if i < 4 else 1095.0,
                "volume": 5000.0
            })
        # Bar 6: Open is 1100.0
        raw_bars.append({
            "timestamp": base_utc + (5 * 900),
            "open": 1100.0,
            "high": 1110.0,
            "low": 1090.0,
            "close": 1105.0,
            "volume": 5000.0
        })
        # Remaining bars to complete session
        for i in range(6, 25):
            raw_bars.append({
                "timestamp": base_utc + (i * 900),
                "open": 1105.0,
                "high": 1106.0,
                "low": 1104.0,
                "close": 1105.0,
                "volume": 1000.0
            })

        with open(self.data_dir / f"{sym}.json", "w", encoding="utf-8") as f:
            json.dump(raw_bars, f)

        harness = RP608HistoricalReplayHarness(
            data_dir=self.data_dir,
            output_dir=self.output_dir,
            symbols=[sym],
            half_spread_bps=2.5,
            slippage_bps=2.0
        )
        summary = harness.run_replay()
        self.assertGreater(summary["performance_metrics"]["total_trades"], 0)

        # Inspect recorded trade
        with open(harness.trades_path, "r", encoding="utf-8") as f:
            trades = [json.loads(line) for line in f]
        self.assertGreaterEqual(len(trades), 1)
        t = trades[0]
        # Entry bar MUST be Bar 6 (Bar 5 was the signal candle)
        self.assertEqual(t["entry_bar_idx"], 6)
        # Entry price must be based on Bar 6 Open (1100.0) + friction, NOT Bar 5 close (1095.0)
        expected_slip = 1100.0 * (4.5 / 10000.0)
        self.assertAlmostEqual(t["entry_price"], round(1100.0 + expected_slip, 2), delta=0.1)

    def test_03_conservative_simultaneous_breach_resolution(self):
        """Verifies when a candle breaches both target (+1.5%) and stop (-1.0%), resolves as STOP_LOSS."""
        sym = "RELIANCE.NS"
        raw_bars = []
        base_utc = int(datetime(2026, 7, 6, 3, 45, tzinfo=timezone.utc).timestamp())
        for i in range(5):
            raw_bars.append({
                "timestamp": base_utc + (i * 900),
                "open": 2000.0,
                "high": 2002.0 if i < 4 else 2200.0,
                "low": 1998.0,
                "close": 2000.0 if i < 4 else 2190.0,
                "volume": 5000.0
            })
        # Bar 6: Entry at 2200.0
        # Bar 7: Wild bar spanning 2150.0 (below stop ~2178) and 2250.0 (above target ~2233)
        raw_bars.append({
            "timestamp": base_utc + (5 * 900),
            "open": 2200.0,
            "high": 2205.0,
            "low": 2195.0,
            "close": 2200.0,
            "volume": 5000.0
        })
        raw_bars.append({
            "timestamp": base_utc + (6 * 900),
            "open": 2200.0,
            "high": 2260.0,  # breaches target +1.5% (2233)
            "low": 2150.0,   # breaches stop -1.0% (2178)
            "close": 2210.0,
            "volume": 5000.0
        })
        for i in range(7, 25):
            raw_bars.append({
                "timestamp": base_utc + (i * 900),
                "open": 2210.0,
                "high": 2211.0,
                "low": 2209.0,
                "close": 2210.0,
                "volume": 1000.0
            })

        with open(self.data_dir / f"{sym}.json", "w", encoding="utf-8") as f:
            json.dump(raw_bars, f)

        harness = RP608HistoricalReplayHarness(
            data_dir=self.data_dir,
            output_dir=self.output_dir,
            symbols=[sym]
        )
        summary = harness.run_replay()
        with open(harness.trades_path, "r", encoding="utf-8") as f:
            trades = [json.loads(line) for line in f]
        self.assertGreaterEqual(len(trades), 1)
        self.assertEqual(trades[0]["exit_reason"], "CONSERVATIVE_STOP_LOSS_SIMULTANEOUS")

    def test_04_evidence_classification_categories(self):
        """Verifies diagnostic classification into the 3 discrete categories."""
        harness = RP608HistoricalReplayHarness(self.data_dir, self.output_dir)
        # 1. Under-sampled
        c1 = harness.classify_evidence(trade_count=15, avg_net_return_bps=20.0, profit_factor=1.8)
        self.assertEqual(c1["category"], "INSUFFICIENT_EVIDENCE")

        # 2. Signals exist, but economics fail
        c2 = harness.classify_evidence(trade_count=100, avg_net_return_bps=-15.0, profit_factor=0.7)
        self.assertEqual(c2["category"], "SIGNALS_EXIST_BUT_ECONOMICS_FAIL")

        # 3. Promising out-of-sample evidence
        c3 = harness.classify_evidence(trade_count=100, avg_net_return_bps=25.0, profit_factor=1.45)
        self.assertEqual(c3["category"], "PROMISING_OUT_OF_SAMPLE_EVIDENCE")


if __name__ == "__main__":
    unittest.main()
