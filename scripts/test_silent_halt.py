import subprocess
import time
import json
import os
import signal

def run_silent_halt_drill():
    # Use absolute paths relative to script location
    script_dir = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.dirname(script_dir)
    gov_path = os.path.join(root_dir, "analysis/real_live/governor_state.json")
    core_dir = os.path.join(root_dir, "core")
    
    os.makedirs(os.path.dirname(gov_path), exist_ok=True)
    
    # 1. Initialize Baseline (Gate Open)
    print("🟢 Initializing Governor: GATE_OPEN")
    with open(gov_path, 'w') as f:
        json.dump({"multiplier": 1.0, "gate_open": True, "ts": int(time.time())}, f)
    
    print(f"🚀 Starting Live Engine in {core_dir} (Silent Stdin)...")
    # Use pipe for stdin to keep it open but silent
    process = subprocess.Popen(
        ["cargo", "run", "--release", "--example", "live_engine"],
        cwd=core_dir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )
    
    # Wait for engine to boot and initialize
    print("⏳ Waiting for engine initialization (5s)...")
    time.sleep(5)
    
    # 2. Trigger HALT
    print("🛑 Triggering HALT (gate_open=False, mult=0.0)...")
    with open(gov_path, 'w') as f:
        json.dump({"multiplier": 0.0, "gate_open": False, "ts": int(time.time())}, f)
    
    # Observe for 5 seconds to capture multiple heartbeats
    print("⏳ Observing silent rejection heartbeats (5s)...")
    time.sleep(5)
    
    print("⌛ Drill Complete. Capturing logs...")
    # Send SIGTERM gracefully
    process.terminate()
    
    try:
        stdout, _ = process.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        stdout, _ = process.communicate()
    
    print("\n--- ENGINE LOGS (AUDIT EVIDENCE) ---")
    relevant_logs = []
    found_halt = False
    for line in stdout.splitlines():
        if "[SAFETY]" in line or "[GATE_REJECT]" in line:
            relevant_logs.append(line)
            if "[SAFETY] HALT enforced" in line:
                found_halt = True
    
    if not relevant_logs:
        print("❌ ERROR: No safety or rejection logs found!")
        print("Full output for debugging:")
        print(stdout)
    else:
        for line in relevant_logs:
            print(line)
    
    print("-------------------------------------")
    
    # Validation
    if found_halt and any("[GATE_REJECT]" in l for l in relevant_logs):
        print("✅ PASS: Silent HALT enforced and heartbeats observed.")
    else:
        print("❌ FAIL: Missing critical safety or heartbeat logs.")

if __name__ == "__main__":
    run_silent_halt_drill()
