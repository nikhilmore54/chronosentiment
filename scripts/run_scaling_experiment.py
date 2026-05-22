#!/usr/bin/env python3
import subprocess
import json
import time

EXPERIMENT = {
    "experiment_id": "us_scaling_001",
    "batch_id": 950,
    "steps": [
        {"cycles": 5, "workers": 2,  "symbol_limit": 50},
        {"cycles": 5, "workers": 5,  "symbol_limit": 50},
        {"cycles": 5, "workers": 5,  "symbol_limit": 100},
        {"cycles": 5, "workers": 5,  "symbol_limit": 250},
        {"cycles": 5, "workers": 5,  "symbol_limit": 500},
        {"cycles": 5, "workers": 10, "symbol_limit": 500}
    ]
}

def main():
    print(f"🧪 Starting deterministic experiment: {EXPERIMENT['experiment_id']}")
    print(f"   Target batch: {EXPERIMENT['batch_id']}")
    print(f"   Steps to run: {len(EXPERIMENT['steps'])}\n")

    for idx, step in enumerate(EXPERIMENT["steps"]):
        print("=" * 60)
        print(f"🔬 EXPERIMENT STEP {idx + 1}/{len(EXPERIMENT['steps'])}")
        print(f"   Config: {json.dumps(step)}")
        print("=" * 60)
        
        cmd = [
            "python3", "scripts/run_live_session.py",
            "--batch-id", str(EXPERIMENT["batch_id"]),
            "--cycles", str(step["cycles"]),
            "--bar-sec", "60",
            "--provider-lag-sec", "15",
            "--max-workers", str(step["workers"]),
            "--symbol-limit", str(step["symbol_limit"])
        ]
        
        start_time = time.time()
        try:
            subprocess.run(cmd, check=True)
            elapsed = time.time() - start_time
            print(f"\n✅ Step {idx + 1} completed in {elapsed:.1f}s.\n")
        except subprocess.CalledProcessError as e:
            print(f"\n❌ Step {idx + 1} failed: {e}\n")
            break
        except KeyboardInterrupt:
            print("\n🛑 Experiment aborted by user.")
            break

if __name__ == "__main__":
    main()
