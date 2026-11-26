#!/usr/bin/env python3
"""
Automated Provider Chronology Observatory Scheduler.
This runs continuously and triggers parallel traces at specific market regimes:
- Pre-Open (09:05 IST)
- Open (09:15 IST)
- Morning Continuous (10:30 IST)
- Midday Lull (13:00 IST)
- Pre-Close (15:10 IST)
- Closing Window (15:30 IST)
- Post-Close (15:45 IST)
"""

import time
import subprocess
import sys
from datetime import datetime, timedelta, timezone

IST = timezone(timedelta(hours=5, minutes=30))

# Defined daily capture times (Hour, Minute) - Regime Synchronization Sampling
CAPTURE_TIMES = [
    (9, 5),   # Pre-Open (Auction Initialization)
    (9, 15),  # Open Transition (Highest Observability Stress)
    (10, 30), # Morning Continuous (Baseline Propagation)
    (13, 0),  # Midday Lull (Low-Liquidity Diffusion)
    (15, 10), # Pre-Close (Pre-Close Reconciliation Window)
    (15, 30), # Closing Window (Closing Synchronization Stress)
    (15, 45)  # Post-Close (Closure Reconciliation)
]

def run_command(cmd: list[str]) -> bool:
    print(f"\n🚀 Running: {' '.join(cmd)}")
    try:
        process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()
        process.wait()
        return process.returncode == 0
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def run_observatory_pass():
    print(f"\n[{datetime.now(IST).strftime('%Y-%m-%d %H:%M:%S')}] 🔭 TRIGGERING OBSERVATORY CAPTURE")
    
    # Run traces concurrently
    # We will use subprocess directly so they can run in parallel
    print("Initiating traces for Broad Market (003) and Banking (910)...")
    
    cmd_003 = ["python3", "scripts/run_live_session.py", "--batch-id", "003", "--cycles", "4", "--live-only", "--retry-profile", "continuous_market", "--temporal-observatory"]
    cmd_910 = ["python3", "scripts/run_live_session.py", "--batch-id", "910", "--cycles", "4", "--live-only", "--retry-profile", "continuous_market", "--temporal-observatory"]
    
    p1 = subprocess.Popen(cmd_003, stdout=subprocess.PIPE, text=True)
    p2 = subprocess.Popen(cmd_910, stdout=subprocess.PIPE, text=True)
    
    p1.wait()
    p2.wait()
    
    print("Traces completed. Generating snapshots...")
    run_command(["python3", "scripts/generate_propagation_snapshots.py", "--batch-id", "003"])
    run_command(["python3", "scripts/generate_propagation_snapshots.py", "--batch-id", "910"])
    
    print("Updating Research Log...")
    run_command(["python3", "scripts/update_research_log.py", "--batch-id", "003"])
    run_command(["python3", "scripts/update_research_log.py", "--batch-id", "910"])
    
    print("Capture sequence completed.")

def get_next_capture_time() -> datetime:
    now = datetime.now(IST)
    
    # Find next time today
    for hour, minute in CAPTURE_TIMES:
        candidate = now.replace(hour=hour, minute=minute, second=0, microsecond=0)
        if candidate > now:
            return candidate
            
    # If no times left today, return first time tomorrow
    tomorrow = now + timedelta(days=1)
    return tomorrow.replace(hour=CAPTURE_TIMES[0][0], minute=CAPTURE_TIMES[0][1], second=0, microsecond=0)

def main():
    print("=" * 60)
    print(" 🔭 PROVIDER CHRONOLOGY OBSERVATORY DAEMON")
    print("=" * 60)
    print("Will capture provider synchronization at:")
    for h, m in CAPTURE_TIMES:
        print(f" - {h:02d}:{m:02d} IST")
    print("=" * 60)

    while True:
        target = get_next_capture_time()
        now = datetime.now(IST)
        sleep_sec = (target - now).total_seconds()
        
        print(f"\nNext capture scheduled for: {target.strftime('%Y-%m-%d %H:%M:%S')} IST")
        
        # Sleep loop
        t_end = time.time() + sleep_sec
        while time.time() < t_end:
            remaining = t_end - time.time()
            if remaining <= 0:
                break
            sleep_chunk = min(60.0, remaining)
            time.sleep(sleep_chunk)
            
        run_observatory_pass()

if __name__ == "__main__":
    main()
