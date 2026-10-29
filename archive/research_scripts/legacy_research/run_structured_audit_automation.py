#!/usr/bin/env python3
"""
Structured Paper-Trading Audit Script
Automates Phase 1-5 of the Verification Drill.
"""

import time
import json
import os
from pathlib import Path
from datetime import datetime

STATE_PATH = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL/analysis/real_live/governor_state.json")

def set_governor(multiplier: float, gate_open: bool, reason: str):
    data = {
        "multiplier": multiplier,
        "gate_open": gate_open,
        "reason": reason,
        "ts": int(time.time())
    }
    # Atomic write
    tmp = STATE_PATH.with_suffix(".tmp")
    with open(tmp, "w") as f:
        json.dump(data, f)
    tmp.replace(STATE_PATH)
    print(f"[{datetime.now().strftime('%H:%M:%S')}] GOVERNOR -> mult={multiplier:.2f} gate={gate_open} reason={reason}", flush=True)

def heartbeat_sleep(seconds: float, multiplier: float, gate_open: bool, reason: str):
    start = time.time()
    while time.time() - start < seconds:
        set_governor(multiplier, gate_open, reason)
        time.sleep(2)

def main():
    print("--- STARTING STRUCTURED AUDIT DRILL ---", flush=True)
    
    # Phase 1: Baseline (5 mins for verification)
    heartbeat_sleep(300, 1.0, True, "PHASE_1_BASELINE")
    
    # Phase 2: Mild Friction
    heartbeat_sleep(300, 0.65, True, "PHASE_2_MILD_FRICTION")
    
    # Phase 3: Hostile (Flash)
    heartbeat_sleep(120, 0.0, False, "PHASE_3_HOSTILE_FLASH")
    
    # Phase 4: Recovery
    heartbeat_sleep(60, 0.40, True, "PHASE_4_RECOVERY_A")
    heartbeat_sleep(60, 0.70, True, "PHASE_4_RECOVERY_B")
    heartbeat_sleep(120, 1.00, True, "PHASE_4_RECOVERY_C")
    
    # Phase 5: Oscillation
    print("--- STARTING OSCILLATION TEST ---", flush=True)
    for i in range(4):
        heartbeat_sleep(15, 0.65, True, f"PHASE_5_OSC_ON_{i}")
        heartbeat_sleep(15, 1.0, True, f"PHASE_5_OSC_OFF_{i}")

    print("--- AUDIT DRILL COMPLETE ---", flush=True)

if __name__ == "__main__":
    main()
