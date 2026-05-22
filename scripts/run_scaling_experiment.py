#!/usr/bin/env python3
"""
Controlled scaling experiment runner.
Executes a predefined schedule of cohorts to measure scalability curves mechanically.
"""

import json
import subprocess
from pathlib import Path

def main():
    experiment_def = {
        "experiment_id": "us_scaling_001",
        "base_batch_id": 950,
        "steps": [
            {"step_id": 1, "cycles": 5, "workers": 2,  "symbol_limit": 50},
            {"step_id": 2, "cycles": 5, "workers": 5,  "symbol_limit": 50},
            {"step_id": 3, "cycles": 5, "workers": 5,  "symbol_limit": 100},
            {"step_id": 4, "cycles": 5, "workers": 5,  "symbol_limit": 250},
            {"step_id": 5, "cycles": 5, "workers": 5,  "symbol_limit": 500},
            {"step_id": 6, "cycles": 5, "workers": 10, "symbol_limit": 500}
        ]
    }

    base_cohort_path = Path(f"cohorts/batch_{experiment_def['base_batch_id']:03d}.txt")
    if not base_cohort_path.exists():
        print(f"Base cohort {base_cohort_path} not found.")
        return

    base_symbols = [line.strip() for line in base_cohort_path.read_text().splitlines() if line.strip()]

    output_file = Path(f"state_archive/experiments/{experiment_def['experiment_id']}.jsonl")
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    print(f"🚀 Starting Experiment {experiment_def['experiment_id']}")

    for step in experiment_def["steps"]:
        exp_batch_id = 9900 + step["step_id"]
        
        # 1. Create sub-cohort
        sub_symbols = base_symbols[:step["symbol_limit"]]
        sub_cohort_path = Path(f"cohorts/batch_{exp_batch_id:03d}.txt")
        sub_cohort_path.write_text("\n".join(sub_symbols) + "\n")
        
        print(f"\n--- STEP {step['step_id']}: {step['symbol_limit']} symbols, {step['workers']} workers ---")
        
        # 2. Freeze the baseline (using step workers)
        freeze_cmd = [
            "python3", "scripts/freeze_cohort_candles.py",
            "--batch-id", str(exp_batch_id),
            "--max-workers", str(step["workers"])
        ]
        print(f"   📥 Freezing baseline: {' '.join(freeze_cmd)}")
        subprocess.run(freeze_cmd, check=True)
        
        # 3. Run Live Session
        live_cmd = [
            "python3", "scripts/run_live_session.py",
            "--batch-id", str(exp_batch_id),
            "--cycles", str(step["cycles"]),
            "--bar-sec", "60",
            "--provider-lag-sec", "15",
            "--max-workers", str(step["workers"])
        ]
        print(f"   🚀 Running live soak: {' '.join(live_cmd)}")
        subprocess.run(live_cmd, check=True)
        
        # 4. Collect results
        ledger_path = Path(f"state_archive/batches/batch_{exp_batch_id:03d}/runs/live/metadata/live_session_steps.jsonl")
        if ledger_path.exists():
            records = [json.loads(line) for line in ledger_path.read_text().splitlines() if line.strip()]
            for rec in records:
                rec["experiment_id"] = experiment_def["experiment_id"]
                rec["step_id"] = step["step_id"]
                rec["workers"] = step["workers"]
                
                with open(output_file, "a") as f:
                    f.write(json.dumps(rec) + "\n")
        
        print(f"✅ Step {step['step_id']} collected.")

    print(f"\n🎉 Experiment {experiment_def['experiment_id']} completed. Results in {output_file}")

if __name__ == "__main__":
    main()
