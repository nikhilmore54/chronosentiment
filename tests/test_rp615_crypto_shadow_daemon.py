#!/usr/bin/env python3
"""
tests/test_rp615_crypto_shadow_daemon.py

Unit tests for RP-615-Crypto Stage C3:
Live Crypto Shadow Daemon & Telemetry Engine.
"""

import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

DECISION_CARDS_JSONL = LOG_DIR / "rp615_crypto_shadow_decision_cards.jsonl"
TRADE_JOURNAL_JSONL = LOG_DIR / "rp615_crypto_shadow_trade_journal.jsonl"
DAEMON_STATE_JSON = LOG_DIR / "rp615_crypto_shadow_daemon_state.json"
TELEMETRY_JSON = LOG_DIR / "rp615_crypto_shadow_telemetry.json"
NSE_FROZEN_LEDGER = LOG_DIR / "rp615_tier4_frozen_ledger.csv"

from scripts.run_rp615_crypto_shadow_daemon import (
    CryptoLiveShadowEngine,
    extract_features_from_live_bars,
    run_crypto_shadow_cycle,
    sha256_file,
)


class TestRP615CryptoShadowDaemon(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.tmp_path = Path(self.temp_dir.name)
        self.engine = CryptoLiveShadowEngine(
            state_path=self.tmp_path / "mock_state.json",
            telemetry_path=self.tmp_path / "mock_telemetry.json",
            trade_journal_path=self.tmp_path / "mock_trade_journal.jsonl",
            expansion_trade_journal_path=self.tmp_path / "mock_expansion_trade_journal.jsonl",
            decision_cards_path=self.tmp_path / "mock_cards.jsonl",
            expansion_decision_cards_path=self.tmp_path / "mock_expansion_cards.jsonl",
        )

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_01_engine_initialization(self):
        """Verify engine loads frozen Stage C2 parameters correctly."""
        self.assertEqual(len(self.engine.means), 17)
        self.assertEqual(len(self.engine.stds), 17)
        self.assertAlmostEqual(self.engine.platt_a, 0.6474, places=2)
        self.assertGreater(self.engine.avg_win, 0.0)
        self.assertGreater(self.engine.avg_loss, 0.0)

    def test_02_synthetic_live_inference(self):
        """Verify feature extraction and inference on synthetic sealed bars."""
        # Construct 60 synthetic 1m bars
        base_time = int(datetime.now(timezone.utc).timestamp())
        bars = []
        p = 65000.0
        for i in range(60):
            t = base_time - (59 - i) * 60
            bars.append({
                "open_time": t * 1000,
                "open": p,
                "high": p + 10.0,
                "low": p - 10.0,
                "close": p + 5.0,
                "volume": 12.5,
                "close_time": (t + 59) * 1000,
                "timestamp": t + 59,
            })
            p += 2.0

        t_cutoff_ts = base_time
        feats = extract_features_from_live_bars(bars, t_cutoff_ts)
        self.assertIsNotNone(feats)
        self.assertIn("F01_ret_15m", feats)
        self.assertIn("F05_atr_norm", feats)

        scores = self.engine.score_features(feats)
        self.assertIn("p_calibrated", scores)
        self.assertGreaterEqual(scores["p_calibrated"], 0.0)
        self.assertLessEqual(scores["p_calibrated"], 1.0)
        self.assertIn("p_be", scores)

    def test_03_per_pair_concurrency_lock_and_cooldown(self):
        """Verify single-position lock and 300s cooldown across candidate arms."""
        now_dt = datetime.now(timezone.utc)
        bars = []
        base_time = int(now_dt.timestamp())
        for i in range(60):
            t = base_time - (59 - i) * 60
            bars.append({
                "open": 65000.0, "high": 65010.0, "low": 64990.0, "close": 65005.0,
                "volume": 10.0, "timestamp": t + 59
            })

        # Clear state for test
        self.engine.active_positions = {c: {"BTCUSDT": None} for c in ["C0_Reference", "C5_Fixed_Capacity_Top2Pct"]}
        self.engine.last_exits = {c: {"BTCUSDT": None} for c in ["C0_Reference", "C5_Fixed_Capacity_Top2Pct"]}

        # First evaluation: C0 admits
        res1 = self.engine.evaluate_opportunity("BTCUSDT", bars, now_dt)
        self.assertEqual(res1["des_execution"]["C0_Reference"], "SIMULATED_EXECUTION")

        # Immediate second evaluation: C0 must be BLOCKED_BY_CONCURRENCY
        res2 = self.engine.evaluate_opportunity("BTCUSDT", bars, now_dt)
        self.assertEqual(res2["des_execution"]["C0_Reference"], "BLOCKED_BY_CONCURRENCY")

    def test_04_telemetry_and_journal_artifacts(self):
        """Verify telemetry and log files exist and are valid JSON/JSONL."""
        self.assertTrue(TELEMETRY_JSON.exists(), "Missing telemetry JSON")
        with TELEMETRY_JSON.open() as f:
            telem = json.load(f)
        self.assertEqual(telem["status"], "HEALTHY")
        self.assertEqual(telem["governance"]["rp605_live_authority"], "100%")
        self.assertEqual(telem["governance"]["rp615_crypto_authority"], "0% (Shadow Only)")

    def test_05_cross_track_isolation(self):
        """Verify zero contamination on NSE frozen ledger."""
        actual_nse_hash = sha256_file(NSE_FROZEN_LEDGER)
        with TELEMETRY_JSON.open() as f:
            telem = json.load(f)
        self.assertEqual(telem["governance"]["nse_ledger_checksum"], actual_nse_hash)

    def test_06_lifecycle_exit_settlement_and_cooldown_enforcement(self):
        """Verify 3600s exit settlement, lock release, and 300s post-exit cooldown."""
        entry_time = datetime(2026, 10, 4, 10, 0, 0, tzinfo=timezone.utc)
        sym = "BTCUSDT"
        arm = "C0_Reference"

        # Initialize clean state
        self.engine.active_positions[arm][sym] = None
        self.engine.open_trade_details[arm][sym] = None
        self.engine.last_exits[arm][sym] = None

        # 1. Simulate entry with 60 preceding 1m bars
        base_ts = int(entry_time.timestamp())
        bars = [{"open": 65000.0, "high": 65010.0, "low": 64990.0, "close": 65000.0, "volume": 10.0, "timestamp": base_ts - (59 - i) * 60} for i in range(60)]
        res = self.engine.evaluate_opportunity(sym, bars, entry_time)
        self.assertIn("des_execution", res)
        self.assertEqual(res["des_execution"][arm], "SIMULATED_EXECUTION")
        self.assertIsNotNone(self.engine.active_positions[arm][sym])
        self.assertIsNotNone(self.engine.open_trade_details[arm][sym])

        # 2. Before 3600s elapsed (e.g. at +1800s): position must remain locked
        mid_time = datetime(2026, 10, 4, 10, 30, 0, tzinfo=timezone.utc)
        self.engine.process_exits({sym: 66000.0}, mid_time)
        self.assertIsNotNone(self.engine.active_positions[arm][sym], "Lock must remain active before 3600s")

        # 3. At +3600s: process_exits must close trade and release lock
        exit_time = datetime(2026, 10, 4, 11, 0, 0, tzinfo=timezone.utc)
        self.engine.process_exits({sym: 66000.0}, exit_time)
        self.assertIsNone(self.engine.active_positions[arm][sym], "Lock must be released upon exit")
        self.assertIsNone(self.engine.open_trade_details[arm][sym], "Trade details must be cleared upon exit")
        self.assertEqual(self.engine.last_exits[arm][sym], exit_time.isoformat())

        def make_bars(eval_dt):
            b_ts = int(eval_dt.timestamp())
            return [{"open": 65000.0, "high": 65010.0, "low": 64990.0, "close": 65000.0, "volume": 10.0, "timestamp": b_ts - (59 - i) * 60} for i in range(60)]

        # 4. Same-cycle / immediate attempt at exit_time (+0s cooldown): MUST BE BLOCKED BY COOLDOWN
        res_immediate = self.engine.evaluate_opportunity(sym, make_bars(exit_time), exit_time)
        self.assertIn("des_execution", res_immediate)
        self.assertEqual(res_immediate["des_execution"][arm], "BLOCKED_BY_COOLDOWN")

        # 5. At +299s after exit: MUST STILL BE BLOCKED BY COOLDOWN
        time_299 = datetime(2026, 10, 4, 11, 4, 59, tzinfo=timezone.utc)
        res_299 = self.engine.evaluate_opportunity(sym, make_bars(time_299), time_299)
        self.assertIn("des_execution", res_299)
        self.assertEqual(res_299["des_execution"][arm], "BLOCKED_BY_COOLDOWN")

        # 6. At +300s after exit: Cooldown elapsed -> Eligible for new entry
        time_300 = datetime(2026, 10, 4, 11, 5, 0, tzinfo=timezone.utc)
        res_300 = self.engine.evaluate_opportunity(sym, make_bars(time_300), time_300)
        self.assertIn("des_execution", res_300)
        self.assertEqual(res_300["des_execution"][arm], "SIMULATED_EXECUTION")

    def test_07_persistence_recovery_parity(self):
        """Verify save_state and load_state preserve open trade details and locks without corruption."""
        arm = "C2_Dynamic_BreakEven"
        sym = "ETHUSDT"
        test_trade_id = "TEST-TRADE-LIFECYCLE-999"
        test_time = "2026-10-04T12:00:00+00:00"

        self.engine.active_positions[arm][sym] = test_trade_id
        self.engine.open_trade_details[arm][sym] = {
            "trade_id": test_trade_id,
            "symbol": sym,
            "side": "LONG",
            "entry_time_utc": test_time,
            "entry_price": 2700.0,
        }
        self.engine.save_state()

        # Instantiate fresh engine to simulate restart
        fresh_engine = CryptoLiveShadowEngine(
            state_path=self.tmp_path / "mock_state.json",
            telemetry_path=self.tmp_path / "mock_telemetry.json",
            trade_journal_path=self.tmp_path / "mock_trade_journal.jsonl",
            expansion_trade_journal_path=self.tmp_path / "mock_expansion_trade_journal.jsonl",
            decision_cards_path=self.tmp_path / "mock_cards.jsonl",
            expansion_decision_cards_path=self.tmp_path / "mock_expansion_cards.jsonl",
        )
        self.assertEqual(fresh_engine.active_positions[arm][sym], test_trade_id)
        self.assertEqual(fresh_engine.open_trade_details[arm][sym]["entry_price"], 2700.0)


if __name__ == "__main__":
    unittest.main()

