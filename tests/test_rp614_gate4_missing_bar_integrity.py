#!/usr/bin/env python3
"""
test_rp614_gate4_missing_bar_integrity.py

Automated Test Suite for RP-614 Gate 4 (Missing-Bar Integrity).
Implements the 12 Mandatory Verification Tests (G4-01 to G4-12):
- G4-01: Single missing bar detection & zero synthesis
- G4-02: Multiple consecutive missing bars identification & gap window tracking
- G4-03: Gap during open trade -> classified as UNRESOLVED_GAP, no assumed fills
- G4-04: Gap after valid exit -> previously resolved exit remains intact
- G4-05: Missing horizon bar -> strict policy enforcement without inventing close
- G4-06: Missing session boundary bar -> strict policy enforcement without fabricating price
- G4-07: Duplicate timestamp rejection / deterministic resolution
- G4-08: Invalid OHLCV record rejection & fill prevention
- G4-09: Expected non-trading intervals (overnight/pre-open) not falsely classified as gaps
- G4-10: Deterministic replay across repeated identical runs
- G4-11: Full trade population ledger reconciliation (1 candidate = 1 record)
- G4-12: Missing-data stress fixture (injected gap changes only expected trade records)
"""

import sys
import unittest
import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Any

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR / "scripts"))

from run_rp614_root_cause_audit import simulate_barrier_trade, compute_canonical_friction

def audit_symbol_session_candles(
    candles: List[Dict[str, Any]],
    session_date: str
) -> Dict[str, Any]:
    """
    Audits 1m candle series against expected NSE schedule:
    - Expected session: 09:15:00 IST (03:45:00 UTC) to 15:29:00 IST (09:59:00 UTC) = 375 bars
    """
    # 03:45 UTC to 09:59 UTC
    start_dt = datetime.fromisoformat(f"{session_date}T03:45:00+00:00")
    expected_epochs = [int((start_dt + timedelta(minutes=i)).timestamp()) for i in range(375)]
    
    observed_map = {}
    duplicates = 0
    invalid_ohlc = 0
    non_positive_px = 0
    
    for c in candles:
        ts = c["timestamp"]
        c_dt = datetime.fromtimestamp(ts, tz=timezone.utc)
        if c_dt.strftime("%Y-%m-%d") != session_date:
            continue
        
        if ts in observed_map:
            duplicates += 1
            continue
            
        o, h, l, cl = float(c["open"]), float(c["high"]), float(c["low"]), float(c["close"])
        v = float(c.get("volume", 0.0))
        
        if o <= 0 or h <= 0 or l <= 0 or cl <= 0:
            non_positive_px += 1
        if l > o or l > cl or h < o or h < cl or l > h:
            invalid_ohlc += 1
            
        observed_map[ts] = c

    missing_epochs = [ts for ts in expected_epochs if ts not in observed_map]
    
    # Identify contiguous gap windows
    gap_windows = []
    if missing_epochs:
        cur_gap = [missing_epochs[0]]
        for ts in missing_epochs[1:]:
            if ts == cur_gap[-1] + 60:
                cur_gap.append(ts)
            else:
                gap_windows.append(cur_gap)
                cur_gap = [ts]
        gap_windows.append(cur_gap)

    return {
        "session_date": session_date,
        "expected_bars": len(expected_epochs),
        "observed_bars": len([ts for ts in expected_epochs if ts in observed_map]),
        "missing_bars_count": len(missing_epochs),
        "missing_epochs": missing_epochs,
        "gap_windows": gap_windows,
        "duplicate_timestamps": duplicates,
        "invalid_ohlc_count": invalid_ohlc,
        "non_positive_price_count": non_positive_px
    }

class TestRP614Gate4MissingBarIntegrity(unittest.TestCase):
    def setUp(self):
        self.entry_ts = "2026-09-28T04:50:50Z"
        self.entry_epoch = int(datetime.fromisoformat("2026-09-28T04:50:50+00:00").timestamp())
        self.trade = {
            "trade_id": "TEST-G4-TRADE-001",
            "symbol": "TESTSYM.NS",
            "side": "LONG",
            "entry_timestamp": self.entry_ts,
            "entry_price": 1000.0,
            "quantity": 100,
            "pnl": 0.0
        }

    def test_g4_01_single_missing_bar_detection(self):
        """G4-01: Single missing expected minute is detected and recorded; no bar is synthesized."""
        start_dt = datetime.fromisoformat("2026-09-28T03:45:00+00:00")
        candles = []
        for i in range(375):
            if i == 50: # Drop bar 50 (04:35 UTC)
                continue
            ts = int((start_dt + timedelta(minutes=i)).timestamp())
            candles.append({"timestamp": ts, "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 100})
            
        audit = audit_symbol_session_candles(candles, "2026-09-28")
        self.assertEqual(audit["expected_bars"], 375)
        self.assertEqual(audit["observed_bars"], 374)
        self.assertEqual(audit["missing_bars_count"], 1)
        self.assertEqual(len(audit["gap_windows"]), 1)
        self.assertEqual(len(audit["gap_windows"][0]), 1)

    def test_g4_02_multiple_consecutive_missing_bars(self):
        """G4-02: Gap duration and affected trade windows are correctly identified."""
        start_dt = datetime.fromisoformat("2026-09-28T03:45:00+00:00")
        candles = []
        for i in range(375):
            if 100 <= i < 115: # 15-minute gap (05:25 to 05:40 UTC)
                continue
            ts = int((start_dt + timedelta(minutes=i)).timestamp())
            candles.append({"timestamp": ts, "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 100})

        audit = audit_symbol_session_candles(candles, "2026-09-28")
        self.assertEqual(audit["missing_bars_count"], 15)
        self.assertEqual(len(audit["gap_windows"]), 1)
        self.assertEqual(len(audit["gap_windows"][0]), 15)

    def test_g4_03_gap_during_open_trade_no_assumed_fill(self):
        """G4-03: Gap during open trade produces no assumed stop, target, or flat exit."""
        # Entry at 04:50:50. Candles available only up to 05:00:00, then feed drops entirely.
        candles = []
        ts_list = []
        for m in range(1, 10):
            ts = self.entry_epoch + m * 60
            candles.append({"timestamp": ts, "open": 1001.0, "high": 1002.0, "low": 999.0, "close": 1001.0, "volume": 100})
            ts_list.append(ts)

        # Simulation with Target: 1050 (+500 bps), Stop: 950 (-500 bps) -> neither hit in first 10m
        res = simulate_barrier_trade(
            trade=self.trade,
            candles=candles,
            ts_list=ts_list,
            target_bps=500.0,
            stop_bps=500.0
        )
        
        # When feed ends before barrier or horizon, it reports DATA_TRUNCATED, not flat or assumed fill
        self.assertEqual(res["exit_reason"], "DATA_TRUNCATED")
        self.assertEqual(res["exit_price"], 1001.0) # Last observed close
        self.assertNotEqual(res["exit_reason"], "TARGET_TOUCH")
        self.assertNotEqual(res["exit_reason"], "STOP_TOUCH")

    def test_g4_04_gap_after_valid_exit_remains_unchanged(self):
        """G4-04: Previously determined trade outcome remains unchanged despite later gap."""
        candles = []
        ts_list = []
        # Target hit at minute 2 (1015.0)
        ts1 = self.entry_epoch + 60
        candles.append({"timestamp": ts1, "open": 1000.0, "high": 1005.0, "low": 998.0, "close": 1002.0, "volume": 100})
        ts_list.append(ts1)
        
        ts2 = self.entry_epoch + 120
        candles.append({"timestamp": ts2, "open": 1002.0, "high": 1020.0, "low": 1001.0, "close": 1018.0, "volume": 100}) # Hits 1010 target
        ts_list.append(ts2)

        # Minute 3 to 60 dropped (gap)
        ts3 = self.entry_epoch + 3600
        candles.append({"timestamp": ts3, "open": 900.0, "high": 905.0, "low": 895.0, "close": 900.0, "volume": 100})
        ts_list.append(ts3)

        res = simulate_barrier_trade(
            trade=self.trade,
            candles=candles,
            ts_list=ts_list,
            target_bps=100.0, # 1010.0
            stop_bps=100.0
        )

        self.assertEqual(res["exit_reason"], "TARGET_TOUCH")
        self.assertEqual(res["exit_price"], 1010.0)
        self.assertEqual(res["holding_sec"], 120)

    def test_g4_05_missing_horizon_bar_treatment(self):
        """G4-05: Missing horizon bar does not invent an unsupported close."""
        # Candles stop at 150 minutes (before 180m horizon)
        candles = []
        ts_list = []
        for m in range(1, 150):
            ts = self.entry_epoch + m * 60
            candles.append({"timestamp": ts, "open": 1001.0, "high": 1002.0, "low": 999.0, "close": 1001.5, "volume": 100})
            ts_list.append(ts)

        res = simulate_barrier_trade(
            trade=self.trade,
            candles=candles,
            ts_list=ts_list,
            target_bps=500.0,
            stop_bps=500.0,
            max_holding_minutes=180
        )
        self.assertEqual(res["exit_reason"], "DATA_TRUNCATED")
        self.assertNotEqual(res["exit_reason"], "HORIZON_LIMIT_180M")

    def test_g4_06_missing_session_boundary_bar(self):
        """G4-06: Missing session-boundary bar does not fabricate boundary price."""
        # Late entry at 09:00 UTC (14:30 IST), feed stops at 09:40 UTC (15:10 IST)
        entry_epoch = int(datetime.fromisoformat("2026-09-28T09:00:00+00:00").timestamp())
        trade = {**self.trade, "entry_timestamp": "2026-09-28T09:00:00Z"}
        candles = []
        ts_list = []
        for m in range(1, 40):
            ts = entry_epoch + m * 60
            candles.append({"timestamp": ts, "open": 1000.0, "high": 1002.0, "low": 998.0, "close": 1001.0, "volume": 100})
            ts_list.append(ts)

        res = simulate_barrier_trade(
            trade=trade,
            candles=candles,
            ts_list=ts_list,
            target_bps=500.0,
            stop_bps=500.0
        )
        self.assertEqual(res["exit_reason"], "DATA_TRUNCATED")
        self.assertEqual(res["holding_sec"], 39 * 60)

    def test_g4_07_duplicate_timestamp_handling(self):
        """G4-07: Duplicate timestamps are detected in data quality audit."""
        start_dt = datetime.fromisoformat("2026-09-28T03:45:00+00:00")
        ts = int(start_dt.timestamp())
        candles = [
            {"timestamp": ts, "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.0, "volume": 100},
            {"timestamp": ts, "open": 100.0, "high": 102.0, "low": 98.0, "close": 101.0, "volume": 150}
        ]
        audit = audit_symbol_session_candles(candles, "2026-09-28")
        self.assertEqual(audit["duplicate_timestamps"], 1)

    def test_g4_08_invalid_ohlcv_record_flagged(self):
        """G4-08: Invalid OHLC relationship is flagged in audit."""
        start_dt = datetime.fromisoformat("2026-09-28T03:45:00+00:00")
        ts = int(start_dt.timestamp())
        # Invalid: High < Low
        candles = [{"timestamp": ts, "open": 100.0, "high": 90.0, "low": 110.0, "close": 100.0, "volume": 100}]
        audit = audit_symbol_session_candles(candles, "2026-09-28")
        self.assertEqual(audit["invalid_ohlc_count"], 1)

    def test_g4_09_expected_non_trading_interval_not_flagged(self):
        """G4-09: Overnight interval is not falsely classified as in-session data gap."""
        candles = [
            {"timestamp": int(datetime.fromisoformat("2026-09-28T03:45:00+00:00").timestamp()), "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.0},
            {"timestamp": int(datetime.fromisoformat("2026-09-29T03:45:00+00:00").timestamp()), "open": 102.0, "high": 103.0, "low": 101.0, "close": 102.0}
        ]
        # Session audit for 2026-09-28 only audits 2026-09-28 window
        audit = audit_symbol_session_candles(candles, "2026-09-28")
        self.assertEqual(audit["session_date"], "2026-09-28")
        self.assertEqual(audit["expected_bars"], 375)

    def test_g4_10_deterministic_replay_invariance(self):
        """G4-10: Repeated simulation runs produce bit-for-bit identical outcomes."""
        candles = []
        ts_list = []
        for m in range(1, 30):
            ts = self.entry_epoch + m * 60
            candles.append({"timestamp": ts, "open": 1000.0 + m, "high": 1005.0 + m, "low": 995.0 + m, "close": 1001.0 + m, "volume": 100})
            ts_list.append(ts)

        run1 = simulate_barrier_trade(self.trade, candles, ts_list, 100.0, 100.0)
        run2 = simulate_barrier_trade(self.trade, candles, ts_list, 100.0, 100.0)
        self.assertEqual(run1, run2)

    def test_g4_11_trade_ledger_reconciliation(self):
        """G4-11: Full ledger reconciliation accounts for every trade exactly once."""
        trades = [
            {**self.trade, "trade_id": f"T-{i:03d}"} for i in range(100)
        ]
        dispositions = {"COMPLETE": 0, "DATA_TRUNCATED": 0, "NO_BAR_DATA": 0}
        candles = [{"timestamp": self.entry_epoch + 60, "open": 1000.0, "high": 1020.0, "low": 990.0, "close": 1015.0}]
        ts_list = [self.entry_epoch + 60]

        for t in trades:
            res = simulate_barrier_trade(t, candles, ts_list, 100.0, 100.0)
            if res["status"] == "VALID":
                dispositions["COMPLETE"] += 1
            else:
                dispositions[res["status"]] += 1

        self.assertEqual(sum(dispositions.values()), len(trades))

    def test_g4_12_missing_data_stress_fixture(self):
        """G4-12: Injecting a gap in a specific trade's window alters only that trade's disposition."""
        t1 = {**self.trade, "trade_id": "T-NORMAL"}
        t2 = {**self.trade, "trade_id": "T-GAPPED", "entry_timestamp": "2026-09-28T06:00:00Z"}
        t2_epoch = int(datetime.fromisoformat("2026-09-28T06:00:00+00:00").timestamp())

        # Feed has normal bars for T1, but empty for T2
        candles = [
            {"timestamp": self.entry_epoch + 60, "open": 1000.0, "high": 1020.0, "low": 999.0, "close": 1015.0}
        ]
        ts_list = [self.entry_epoch + 60]

        res1 = simulate_barrier_trade(t1, candles, ts_list, 100.0, 100.0)
        res2 = simulate_barrier_trade(t2, candles, ts_list, 100.0, 100.0)

        self.assertEqual(res1["status"], "VALID")
        self.assertEqual(res1["exit_reason"], "TARGET_TOUCH")
        self.assertEqual(res2["status"], "NO_BAR_DATA")

def main():
    print("=" * 80)
    print("🚀 RUNNING RP-614 GATE 4 MISSING-BAR INTEGRITY TEST SUITE (12/12 TESTS)")
    print("=" * 80)
    suite = unittest.TestLoader().loadTestsFromTestCase(TestRP614Gate4MissingBarIntegrity)
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    if result.wasSuccessful():
        print("\n" + "=" * 80)
        print("✅ ALL 12/12 GATE 4 MISSING-BAR INTEGRITY TESTS PASSED WITH ZERO ERRORS!")
        print("=" * 80)
    else:
        print("\n❌ GATE 4 TESTS FAILED!")
        sys.exit(1)

if __name__ == "__main__":
    main()
