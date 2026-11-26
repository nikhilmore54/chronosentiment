import json
import time
import os

BRIDGE_PATH = "analysis/real_live/governor_state.json"
TMP_PATH = BRIDGE_PATH + ".tmp"

def write_state(multiplier, gate_open, reason):
    data = {
        "multiplier": multiplier,
        "gate_open": gate_open,
        "reason": reason,
        "ts": int(time.time())
    }
    with open(TMP_PATH, "w") as f:
        json.dump(data, f)
    os.replace(TMP_PATH, BRIDGE_PATH)
    # print(f"[{time.strftime('%H:%M:%S')}] Heartbeat: mult={multiplier}, gate={gate_open}")

def run_audit():
    print("🚀 Starting HEARTBEAT Governor Audit Protocol")
    
    steps = [
        (1.0, True, "NOMINAL (Baseline)"),
        (0.65, True, "THROTTLE (Mild Friction)"),
        (0.0, False, "HALT (Hostile/Flash)"),
        (0.40, True, "RECOVERY_STEP_1"),
        (0.70, True, "RECOVERY_STEP_2"),
        (1.0, True, "NOMINAL (Audit Complete)")
    ]
    
    for mult, gate, reason in steps:
        print(f"[{time.strftime('%H:%M:%S')}] Entering Phase: {reason}")
        # Run each phase for 120 seconds with a 2s heartbeat
        for _ in range(60): 
            write_state(mult, gate, reason)
            time.sleep(2)
            
    print("✅ Audit Protocol Finished")

if __name__ == "__main__":
    run_audit()
