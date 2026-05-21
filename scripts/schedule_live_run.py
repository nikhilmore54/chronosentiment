#!/usr/bin/env python3
"""
Automated Scheduler for APAC Open Live Validation Pass.
Calculates delay until 05:20 AM IST, sleeps, then executes:
  1. Live Session Ingest (batch_903, 6 cycles)
  2. Substrate Freeze
  3. Deterministic Replay
  4. Cryptographic Replay Equivalence Certification
  5. TRL Summary extraction
"""

import time
import subprocess
import sys
import json
from pathlib import Path
from datetime import datetime, timedelta, timezone

TARGET_HOUR = 5
TARGET_MINUTE = 20

def get_seconds_to_target() -> tuple[float, str]:
    # yfinance / local clock operates on Indian Standard Time (UTC+5:30)
    IST = timezone(timedelta(hours=5, minutes=30))
    now = datetime.now(IST)
    
    # Target time on same day
    target = now.replace(hour=TARGET_HOUR, minute=TARGET_MINUTE, second=0, microsecond=0)
    
    if target <= now:
        # If it's already past target hour, schedule for next day (shouldn't happen tonight)
        target += timedelta(days=1)
        
    delta = target - now
    return delta.total_seconds(), target.strftime("%Y-%m-%d %H:%M:%S")

def run_command(cmd: list[str]) -> bool:
    print(f"\n🚀 Running: {' '.join(cmd)}")
    try:
        # Run process and stream output to console in real-time
        process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()
        process.wait()
        success = process.returncode == 0
        if success:
            print(f"✅ Success: {' '.join(cmd)}")
        else:
            print(f"❌ Failed (exit code {process.returncode}): {' '.join(cmd)}")
        return success
    except Exception as e:
        print(f"❌ Error executing command: {e}")
        return False

def main():
    print("=" * 80)
    print("  CHRONOSENTIMENT AUTOMATED SCHEDULER — APAC OPEN PASS")
    print("=" * 80)
    
    IST = timezone(timedelta(hours=5, minutes=30))
    sec_to_wait, target_time_str = get_seconds_to_target()
    
    print(f"  Current Time : {datetime.now(IST).strftime('%Y-%m-%d %H:%M:%S')} IST")
    print(f"  Target Time  : {target_time_str} IST (05:20 AM)")
    print(f"  Sleep Delay  : {sec_to_wait:.1f} seconds ({sec_to_wait / 60:.1f} minutes)")
    print("=" * 80)
    
    # Sleep Loop with periodic updates (every 10 minutes)
    t_start = time.time()
    t_end = t_start + sec_to_wait
    
    while time.time() < t_end:
        remaining = t_end - time.time()
        if remaining <= 0:
            break
        print(f"  [WAITING] {remaining/60:.1f} minutes remaining until APAC Open (Target: 05:20 AM IST)...")
        sys.stdout.flush()
        # Sleep up to 10 minutes or remaining time
        sleep_chunk = min(600.0, remaining)
        time.sleep(sleep_chunk)
        
    print("\n⏰ 05:20 AM IST REACHED! COMMENCING APAC OPEN LIVE PASS...")
    sys.stdout.flush()
    
    # Save sealed automation metadata
    metadata_dir = Path("state_archive/batches/batch_903/runs/live/metadata")
    metadata_dir.mkdir(parents=True, exist_ok=True)
    
    automation_metadata = {
        "schema_version": 1,
        "scheduled_target": target_time_str,
        "actual_start": datetime.now(IST).isoformat(),
        "batch_id": 903,
        "cycles": 6,
    }
    with open(metadata_dir / "apac_scheduler.json", "w") as f:
        json.dump(automation_metadata, f, indent=4)
    print(f"💾 Persisted automation metadata to {metadata_dir / 'apac_scheduler.json'}")
    
    # ── Phase 1: Live Ingest Soak (6 cycles) ──
    live_cmd = [
        "python3", "scripts/run_live_session.py",
        "--batch-id", "903",
        "--cycles", "6",
        "--live-only",
        "--quorum-ratio", "0.25",
        "--provider-lag-sec", "8"
    ]
    live_ok = run_command(live_cmd)
    
    # ── Phase 2: Substrate Freeze ──
    # Run freeze regardless of live exit code to capture baseline
    freeze_cmd = [
        "python3", "scripts/freeze_cohort_candles.py",
        "--batch-id", "903",
        "--max-workers", "15"
    ]
    freeze_ok = run_command(freeze_cmd)
    
    # ── Phase 3: Deterministic Replay ──
    replay_cmd = [
        "python3", "scripts/run_nse_cohort.py",
        "--batch-id", "903",
        "--from-frozen",
        "--fresh",
        "--run-label", "replay_equiv"
    ]
    replay_ok = run_command(replay_cmd)
    
    # ── Phase 4: Replay Equivalence Certification ──
    cert_cmd = [
        "python3", "scripts/compare_replay_equivalence.py",
        "--batch-id", "903",
        "--live-label", "live",
        "--replay-label", "replay_equiv"
    ]
    cert_ok = run_command(cert_cmd)
    
    # ── Phase 5: TRL Summary Extraction ──
    trl_cmd = [
        "python3", "scripts/extract_trl_summary.py",
        "--batch-id", "903",
        "--run-label", "live"
    ]
    trl_ok = run_command(trl_cmd)

    # ── Persist machine-queryable exit codes ──
    # Allows post-hoc audit to distinguish phase failures without parsing stdout.
    # exit_code 0 = success, 1 = failure (mirrors Unix convention).
    pipeline_results = {
        "schema_version": 1,
        "completed_at": datetime.now(IST).isoformat(),
        "batch_id": 903,
        "run_label": "live",
        "phases": {
            "live_ingest":   {"exit_code": 0 if live_ok   else 1, "success": live_ok},
            "freeze":        {"exit_code": 0 if freeze_ok else 1, "success": freeze_ok},
            "replay":        {"exit_code": 0 if replay_ok else 1, "success": replay_ok},
            "certification": {"exit_code": 0 if cert_ok   else 1, "success": cert_ok},
            "trl_summary":   {"exit_code": 0 if trl_ok    else 1, "success": trl_ok},
        },
        "all_succeeded": all([live_ok, freeze_ok, replay_ok, cert_ok, trl_ok]),
    }
    with open(metadata_dir / "pipeline_results.json", "w") as f:
        json.dump(pipeline_results, f, indent=4)
    print(f"\n💾 Pipeline exit codes persisted to {metadata_dir / 'pipeline_results.json'}")

    print("\n" + "=" * 80)
    print("  APAC OPEN AUTOMATION RESULTS SUMMARY")
    print("=" * 80)
    print(f"  1. Live Ingestion Ingest : {'SUCCESS ✅' if live_ok   else 'FAILED ❌'}")
    print(f"  2. Substrate Freeze      : {'SUCCESS ✅' if freeze_ok else 'FAILED ❌'}")
    print(f"  3. Deterministic Replay  : {'SUCCESS ✅' if replay_ok else 'FAILED ❌'}")
    print(f"  4. Replay Certification  : {'SUCCESS ✅' if cert_ok   else 'FAILED ❌'}")
    print(f"  5. TRL Summary Extraction: {'SUCCESS ✅' if trl_ok    else 'FAILED ❌'}")
    print("=" * 80)
    print("  Verification complete. Logs stored and certified in state_archive.")
    print("=" * 80)

if __name__ == "__main__":
    main()
