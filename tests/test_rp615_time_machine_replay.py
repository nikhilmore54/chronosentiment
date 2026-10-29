#!/usr/bin/env python3
"""
tests/test_rp615_time_machine_replay.py

Unit tests for RP-615 Time-Machine Replay & Fault-Injection Engine.
Verifies all 5 Operational Safeguards:
1. Model Digest Mismatch -> Hard Stop No-Start.
2. C1 Write-Ahead Timestamp Violation -> Quarantine & Flag.
3. C2 Telemetry Read Leakage -> Immediate Abort.
4. C4 Concurrency Anomaly -> Quarantine Context Anomaly.
5. Delayed Settlement Data -> Graceful PENDING_SETTLEMENT state.
"""

import unittest
from pathlib import Path
from scripts.run_rp615_time_machine_replay import (
    TimeMachineShadowHarness,
    ModelDigestMismatchException,
    C2ReadViolationException,
    FROZEN_MODEL_HASH,
)

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
REPLAY_REPORT_MD = LOG_DIR / "rp615_time_machine_replay_report.md"


class TestRP615TimeMachineReplay(unittest.TestCase):

    def setUp(self):
        self.assertTrue(REPLAY_REPORT_MD.exists(), f"Missing {REPLAY_REPORT_MD}")

    def test_01_model_digest_mismatch_halt(self):
        """Verify hard stop on corrupted model hash."""
        with self.assertRaises(ModelDigestMismatchException):
            _ = TimeMachineShadowHarness(inject_fault="FAULT_MODEL_DIGEST_MISMATCH")

    def test_02_c1_write_ahead_quarantine(self):
        """Verify late C1 timestamps are quarantined."""
        harness = TimeMachineShadowHarness(inject_fault="FAULT_C1_WRITE_AHEAD_LATE")
        meta = {"trade_id": "TEST-01", "session_date": "2026-10-05", "symbol": "TCS.NS"}
        res = harness.process_incoming_opportunity(100.0, 100.0, 0.45, 10.0, meta)
        self.assertEqual(res["status"], "QUARANTINED")
        self.assertEqual(len(harness.quarantine_log), 1)

    def test_03_c2_read_leakage_abort(self):
        """Verify reading C2 telemetry during C1 inference triggers abort."""
        harness = TimeMachineShadowHarness(inject_fault="FAULT_C2_READ_LEAKAGE")
        meta = {"trade_id": "TEST-02", "session_date": "2026-10-05", "symbol": "INFY.NS"}
        with self.assertRaises(C2ReadViolationException):
            _ = harness.process_incoming_opportunity(100.0, 100.1, 0.45, 10.0, meta)

    def test_04_c4_concurrency_anomaly_quarantine(self):
        """Verify future exit timestamps in C4 context trigger quarantine."""
        harness = TimeMachineShadowHarness(inject_fault="FAULT_C4_CONCURRENCY_ANOMALY")
        harness.c4_resolved_exits.append({"trade_id": "FUTURE-01", "epoch_exit": 500.0, "is_win": 1})
        meta = {"trade_id": "TEST-03", "session_date": "2026-10-05", "symbol": "RELIANCE.NS"}
        res = harness.process_incoming_opportunity(100.0, 100.1, 0.45, 10.0, meta)
        self.assertEqual(res["status"], "QUARANTINED")

    def test_05_delayed_settlement_pending_state(self):
        """Verify missing settlement data marks PENDING_SETTLEMENT without crashing."""
        harness = TimeMachineShadowHarness(inject_fault="FAULT_DELAYED_SETTLEMENT")
        meta = {"trade_id": "TEST-04", "session_date": "2026-10-05", "symbol": "SBIN.NS"}
        harness.process_exit_settlement(150.0, meta, None, None, None)
        self.assertEqual(harness.c3_store[0]["status"], "PENDING_SETTLEMENT")


if __name__ == "__main__":
    unittest.main()
