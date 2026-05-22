#!/usr/bin/env python3
import json
import sys
from pathlib import Path

def compute_admissibility(strict_ratio, acceptance_ratio, recovery_slope):
    is_synchronized = (strict_ratio >= 0.9)
    is_degraded = (acceptance_ratio < 0.5)
    is_recovering = (recovery_slope > 0.1)
    
    if is_synchronized:
        regime = "SYNCHRONIZED"
        new_entries = True
    elif is_degraded and not is_recovering:
        regime = "DEGRADED_OBSERVABILITY"
        new_entries = False
    elif is_recovering:
        regime = "TRANSITIONAL_RECOVERY"
        new_entries = False
    else:
        regime = "FRAGMENTED_BUT_USABLE"
        new_entries = True
        
    return {
        "execution_admissible": new_entries,
        "admissibility_reason": regime,
        "new_entries_allowed": new_entries,
        "exits_allowed": True,
        "observability_schema_version": "v1.0",
        "classification_policy_version": "v1.0"
    }

def inject_topology(ledger_path: Path, mode: str):
    if not ledger_path.exists():
        print(f"❌ Ledger not found: {ledger_path}")
        return
        
    out_path = ledger_path.parent / f"synthetic_{mode}_steps.jsonl"
    print(f"🔬 Injecting {mode.ljust(15)} Topology -> {out_path.name}")
    
    history_accept = []
    
    with open(ledger_path, 'r') as f_in, open(out_path, 'w') as f_out:
        for line in f_in:
            if not line.strip(): continue
            row = json.loads(line)
            if not row.get("admissibility"): continue
            
            attempted = row.get("symbols_attempted", 100)
            if attempted == 0: attempted = 100
            
            # Base metrics
            strict = 1.0
            accept = 1.0
            lag_stddev = 0.0
            median_lag = 0
            
            if mode == "uniform_delay":
                strict = 0.0
                accept = 1.0
                lag_stddev = 0.0
                median_lag = 60
            elif mode == "bimodal":
                strict = 0.5
                accept = 1.0
                lag_stddev = 90.0
                median_lag = 90
            elif mode == "rolling_wave":
                # Simulated cascade over time - using cycle as wave progression
                wave_prog = ((row["cycle"] % 10) + 1) / 10.0
                strict = wave_prog
                accept = min(1.0, wave_prog + 0.3)
                lag_stddev = 45.0
            elif mode == "anticipatory":
                strict = 0.7
                accept = 1.0
                lag_stddev = 30.0
                median_lag = -60
            elif mode == "collapse":
                strict = 0.0
                accept = 0.0
                lag_stddev = 10.0
                median_lag = 300
            else:
                pass # Baseline
                
            history_accept.append(accept)
            if len(history_accept) > 5: history_accept.pop(0)
            recovery_slope = history_accept[-1] - history_accept[-2] if len(history_accept) > 1 else 0.0
            
            # Perturb raw counts
            row["symbols_returned"] = int(attempted * strict)
            row["symbols_accepted"] = int(attempted * accept)
            row["freshness"]["lag_stddev"] = lag_stddev
            row["freshness"]["median_symbol_lag_sec"] = median_lag
            
            # Recompute deterministic admissibility
            row["observability"]["strict_ratio"] = strict
            row["observability"]["acceptance_ratio"] = accept
            row["observability"]["recovery_slope"] = recovery_slope
            
            row["admissibility"] = compute_admissibility(strict, accept, recovery_slope)
            row["observability"]["regime_state"] = row["admissibility"]["admissibility_reason"]
            
            f_out.write(json.dumps(row) + "\n")
            
if __name__ == "__main__":
    base = Path("state_archive/batches/batch_10000/runs/live/metadata/live_session_steps.jsonl")
    for mode in ["uniform_delay", "bimodal", "rolling_wave", "anticipatory", "collapse"]:
        inject_topology(base, mode)
