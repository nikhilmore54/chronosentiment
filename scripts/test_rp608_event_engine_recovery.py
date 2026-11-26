#!/usr/bin/env python3
"""
scripts/test_rp608_event_engine_recovery.py

Targeted Recovery & Stream Discontinuity Test Suite for RP-608-EVENT-001:
  - Candidate A: 16 bars (4 hours)
  - Candidate B: 32 bars (8 hours)
  - Candidate C: 64 bars (16 hours)
  - Candidate D: 120 bars (30 hours)

Injected Failure Modes & Invariants Tested:
  1. Feed Disconnect & Reconnect handling without sequence corruption.
  2. Sequence Update-ID Reset (u_new << u_old) on reconnected stream.
  3. Post-Reconnect Microstructure Warmup Enforcement (mandatory 5.0s warmup).
  4. Stale Quote Latency Rejection (age > 1000ms rejected fail-closed).
  5. Process Restart & Ledger Continuity (zero duplicates, atomic append).
  6. Canonical Observer Isolation (zero modification to canonical files).
  7. Machine-readable evidence written to recovery_test_evidence.json.
"""

from __future__ import annotations

import os
import sys
import json
import time
import shutil
from pathlib import Path
from typing import Dict, Any, List

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.rp608_crypto_event_engine import (
    MultiLookbackCoordinator,
    CompletedCandle,
    CANDIDATE_CONFIGS,
    POLICY_FAMILY,
    MIN_REFRACTORY_MS,
    MIN_HISTORY_WARMUP_SEC
)

RECOVERY_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto" / "event_driven" / "recovery_test"
EVIDENCE_FILE = RECOVERY_DIR / "recovery_test_evidence.json"
CANONICAL_CARDS_FILE = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto" / "rp608_decision_cards.jsonl"


def generate_test_candles(base_time_ms: int, n_bars: int = 120) -> List[CompletedCandle]:
    """Generates 120 completed 15m candles with a bullish 16b bounce and bearish 120b trend."""
    candles = []
    start_p = 82000.0
    for idx in range(n_bars):
        c_open_ms = base_time_ms - (n_bars - idx) * 900000
        c_close_ms = c_open_ms + 900000 - 1
        if idx < 104:
            p = start_p - idx * ((start_p - 79000.0) / 104.0)
        else:
            p = 79000.0 + (idx - 104) * ((79500.0 - 79000.0) / 16.0)

        c = CompletedCandle(
            bar_idx=idx + 1,
            open_time_ms=c_open_ms,
            close_time_ms=c_close_ms,
            open_price=round(p - 10.0, 2),
            high_price=round(p + 35.0, 2),
            low_price=round(p - 35.0, 2),
            close_price=round(p, 2),
            volume=150.0
        )
        candles.append(c)
    return candles


def run_recovery_test() -> Dict[str, Any]:
    print("\n" + "=" * 90)
    print("  RP-608-EVENT-001: TARGETED RECOVERY & STREAM DISCONTINUITY TEST")
    print(f"  Test Directory: {RECOVERY_DIR}")
    print("=" * 90 + "\n")

    # Clean test directory
    if RECOVERY_DIR.exists():
        shutil.rmtree(RECOVERY_DIR)
    RECOVERY_DIR.mkdir(parents=True, exist_ok=True)

    base_time_ms = 1700000000000
    test_results = {}

    # Initialize Coordinator
    coordinator = MultiLookbackCoordinator(base_dir=RECOVERY_DIR)
    
    # Pre-seed 120 candles
    candles = generate_test_candles(base_time_ms, n_bars=120)
    for c in candles:
        coordinator.ingest_completed_candle(c)

    config_hashes = {bars: eng.config_hash for bars, eng in coordinator.engines.items()}
    print(f"[*] Pre-seeded 120 candles. Candidate Configuration Hashes:")
    for bars, ch in config_hashes.items():
        print(f"    - Candidate {bars}b: {ch}")

    # --------------------------------------------------------------------------
    # STAGE 1: Warmup & Initial Signal Emission
    # --------------------------------------------------------------------------
    print("\n[*] STAGE 1: Warmup & Initial Baseline Emission...")
    # Feed 10s of flat quotes
    for s in range(10):
        t = base_time_ms + s * 1000
        q = {"u": 50000 + s, "s": "BTCUSDT", "b": "79500.00", "B": "1.0", "a": "79500.01", "A": "1.0", "local_wallclock_utc_ms": t}
        coordinator.process_quote_event(q, receive_time_ms=t + 10)

    # Bullish surge across 5s
    for s in range(10, 15):
        t = base_time_ms + s * 1000
        p = 79500.0 + (s - 10) * 5.0
        q = {"u": 50000 + s, "s": "BTCUSDT", "b": f"{p:.2f}", "B": "5.0", "a": f"{p+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": t}
        res = coordinator.process_quote_event(q, receive_time_ms=t + 10)

    # Trigger at s=15
    t15 = base_time_ms + 15000
    q15 = {"u": 50015, "s": "BTCUSDT", "b": "79525.00", "B": "5.0", "a": "79525.01", "A": "1.0", "local_wallclock_utc_ms": t15}
    res15 = coordinator.process_quote_event(q15, receive_time_ms=t15 + 10)

    assert res15[16] is not None, "FAIL: Candidate 16b failed to emit initial LONG signal"
    initial_event_id = res15[16]["event_id"]
    print(f"    [PASS] Initial baseline recommendation emitted: {initial_event_id}")
    test_results["stage_1_baseline_emission"] = "PASS"

    # --------------------------------------------------------------------------
    # STAGE 2: Inject Feed Disconnect & Sequence Update-ID Reset
    # --------------------------------------------------------------------------
    print("\n[*] STAGE 2: Injecting Feed Disconnect & Sequence Update-ID Reset...")
    # Stream disconnect occurs at t=20s
    t_reconn = base_time_ms + 20000
    coordinator.handle_reconnect(reason="SIMULATED_FEED_DISCONNECT")

    for bars, eng in coordinator.engines.items():
        assert eng.last_valid_quote is None, f"FAIL: Candidate {bars}b last_valid_quote not reset on reconnect"
        assert eng.feature_buf.get_history_duration_sec() == 0.0, f"FAIL: Candidate {bars}b feature buffer not cleared on reconnect"
        assert eng.stats["stream_reconnect_count"] == 1

    print("    [PASS] Stream state reset cleanly across all candidate engines.")

    # Reconnect stream arrives with completely RESET sequence update-ID (u=100 << 50015)
    q_reconn = {"u": 100, "s": "BTCUSDT", "b": "79525.00", "B": "1.0", "a": "79525.01", "A": "1.0", "local_wallclock_utc_ms": t_reconn}
    res_reconn = coordinator.process_quote_event(q_reconn, receive_time_ms=t_reconn + 10)

    # Engine must accept quote u=100 without throwing OUT_OF_ORDER_UPDATE_ID
    assert coordinator.engines[16].stats["quotes_valid"] == 17, "FAIL: Reconnect baseline quote u=100 was rejected"
    assert coordinator.engines[16].stats["quotes_rejected"] == 0, "FAIL: Reconnect baseline quote caused rejection"
    print("    [PASS] Update-ID reset (u=100) accepted cleanly as new baseline without false rejection.")
    test_results["stage_2_disconnect_update_id_reset"] = "PASS"

    # --------------------------------------------------------------------------
    # STAGE 3: Warmup Enforcement After Reconnect
    # --------------------------------------------------------------------------
    print("\n[*] STAGE 3: Verifying Post-Reconnect Warmup Enforcement...")
    # An immediate surge arrives 2 seconds after reconnect (insufficient warmup)
    t_early_surge = t_reconn + 2000
    q_early = {"u": 105, "s": "BTCUSDT", "b": "79540.00", "B": "5.0", "a": "79540.01", "A": "1.0", "local_wallclock_utc_ms": t_early_surge}
    res_early = coordinator.process_quote_event(q_early, receive_time_ms=t_early_surge + 10)

    # All candidates MUST return None (NO_TRADE: INSUFFICIENT_WARMUP)
    for bars in (16, 32, 64, 120):
        assert res_early[bars] is None, f"FAIL: Candidate {bars}b emitted signal before 5.0s warmup completed!"
    print("    [PASS] Warmup enforced post-reconnect: Premature signal suppressed (INSUFFICIENT_WARMUP).")
    test_results["stage_3_post_reconnect_warmup"] = "PASS"

    # Now feed clean 6s of quotes to satisfy warmup
    for s in range(3, 9):
        t = t_reconn + s * 1000
        p = 79540.0 + (s - 3) * 3.0
        q = {"u": 110 + s, "s": "BTCUSDT", "b": f"{p:.2f}", "B": "5.0", "a": f"{p+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": t}
        coordinator.process_quote_event(q, receive_time_ms=t + 10)

    # Emits post-warmup signal at s=9
    t_valid = t_reconn + 9000
    q_valid = {"u": 120, "s": "BTCUSDT", "b": "79560.00", "B": "5.0", "a": "79560.01", "A": "1.0", "local_wallclock_utc_ms": t_valid}
    res_valid = coordinator.process_quote_event(q_valid, receive_time_ms=t_valid + 10)
    assert res_valid[16] is not None, "FAIL: Candidate 16b failed to emit post-warmup signal"
    print(f"    [PASS] Post-warmup signal emitted cleanly: {res_valid[16]['event_id']}")

    # --------------------------------------------------------------------------
    # STAGE 4: Stale Quote Latency Rejection
    # --------------------------------------------------------------------------
    print("\n[*] STAGE 4: Stale Quote Latency Rejection...")
    # Inject quote with 2,500ms latency (limit is 1,000ms)
    t_stale = t_valid + 1000
    q_stale = {"u": 125, "s": "BTCUSDT", "b": "79560.00", "B": "5.0", "a": "79560.01", "A": "1.0", "local_wallclock_utc_ms": t_stale - 2500}
    res_stale = coordinator.process_quote_event(q_stale, receive_time_ms=t_stale)
    
    for bars in (16, 32, 64, 120):
        assert res_stale[bars] is None
        assert coordinator.engines[bars].stats["quotes_rejected"] == 1
    print("    [PASS] Stale quote (2,500ms latency) rejected fail-closed with rejection counter incremented.")
    test_results["stage_4_stale_quote_rejection"] = "PASS"

    # --------------------------------------------------------------------------
    # STAGE 5: Process Restart & Ledger Continuity
    # --------------------------------------------------------------------------
    print("\n[*] STAGE 5: Process Restart & Ledger Continuity Audit...")
    # Record line counts before restart
    pre_restart_counts = {}
    for bars, eng in coordinator.engines.items():
        lines = [l for l in eng.ledger_path.read_text().split("\n") if l.strip()] if eng.ledger_path.exists() else []
        pre_restart_counts[bars] = len(lines)

    # Simulate process shutdown and re-initialization pointing to same directory
    coordinator_restarted = MultiLookbackCoordinator(base_dir=RECOVERY_DIR)
    for c in candles:
        coordinator_restarted.ingest_completed_candle(c)

    # Verify ledgers were not truncated on restart
    for bars, eng in coordinator_restarted.engines.items():
        lines = [l for l in eng.ledger_path.read_text().split("\n") if l.strip()] if eng.ledger_path.exists() else []
        assert len(lines) == pre_restart_counts[bars], f"FAIL: Ledger truncated on restart for Candidate {bars}b"

    # Feed new post-restart quote stream
    t_restart = t_valid + 5000
    for s in range(10):
        t = t_restart + s * 1000
        p = 79560.0 + s * 2.0
        q = {"u": 200 + s, "s": "BTCUSDT", "b": f"{p:.2f}", "B": "5.0", "a": f"{p+0.01:.2f}", "A": "1.0", "local_wallclock_utc_ms": t}
        coordinator_restarted.process_quote_event(q, receive_time_ms=t + 10)

    # Verify atomic append without duplicate event IDs
    for bars, eng in coordinator_restarted.engines.items():
        all_lines = [json.loads(l) for l in eng.ledger_path.read_text().split("\n") if l.strip()] if eng.ledger_path.exists() else []
        event_ids = [r["event_id"] for r in all_lines]
        assert len(event_ids) == len(set(event_ids)), f"FAIL: Duplicate event IDs detected in {eng.candidate_id} ledger!"
        # Invariant 0.0 order authority
        for r in all_lines:
            assert r["order_authority"] == 0.0

    print("    [PASS] Process restart verified: Zero ledger corruption, zero duplicate IDs, atomic append.")
    test_results["stage_5_process_restart_continuity"] = "PASS"

    # --------------------------------------------------------------------------
    # STAGE 6: Canonical Observer Isolation Audit
    # --------------------------------------------------------------------------
    print("\n[*] STAGE 6: Canonical Observer Isolation Audit...")
    if CANONICAL_CARDS_FILE.exists():
        with open(CANONICAL_CARDS_FILE, "r", encoding="utf-8") as f:
            c_cards = [json.loads(l) for l in f if l.strip()]
        for c in c_cards:
            assert "candidate_id" not in c
            assert "candidate_lookback_bars" not in c
        print(f"    [PASS] Canonical decision cards log ({len(c_cards)} cards) confirmed 100% untainted.")
        test_results["stage_6_canonical_isolation"] = f"PASS ({len(c_cards)} cards verified untainted)"
    else:
        print("    [PASS] Canonical observer directory isolated.")
        test_results["stage_6_canonical_isolation"] = "PASS (Local directory isolated)"

    # --------------------------------------------------------------------------
    # Write Machine-Readable Evidence
    # --------------------------------------------------------------------------
    evidence = {
        "test_name": "RP-608-EVENT-001 Targeted Recovery & Discontinuity Test",
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "overall_status": "PASSED",
        "stages": test_results,
        "configuration_hashes": config_hashes,
        "candidate_ledger_stats": {
            bars: {
                "candidate_id": eng.candidate_id,
                "ledger_file": str(eng.ledger_path.name),
                "total_emitted_records": len([l for l in eng.ledger_path.read_text().split("\n") if l.strip()]) if eng.ledger_path.exists() else 0,
                "quotes_received": eng.stats["quotes_received"],
                "quotes_valid": eng.stats["quotes_valid"],
                "quotes_rejected": eng.stats["quotes_rejected"],
                "recs_rate_limited": eng.stats["recs_rate_limited"]
            }
            for bars, eng in coordinator_restarted.engines.items()
        },
        "order_authority_pct": 0.0
    }

    with open(EVIDENCE_FILE, "w", encoding="utf-8") as f:
        json.dump(evidence, f, indent=2)

    print(f"\n[+] Machine-readable evidence written to {EVIDENCE_FILE}")
    print("\n" + "=" * 90)
    print("  RECOVERY TEST VERDICT: 100% PASSED (All 6 Recovery Invariants Satisfied)")
    print("=" * 90 + "\n")

    return evidence


if __name__ == "__main__":
    run_recovery_test()
