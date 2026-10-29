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
        for row_index, line in enumerate(f_in):
            if not line.strip(): continue
            row = json.loads(line)
            if not row.get("admissibility"): continue
            
            attempted = row.get("symbols_attempted", 100)
            if attempted == 0: attempted = 100
            
            # Ensure "freshness" and "observability" exist
            if "freshness" not in row: row["freshness"] = {}
            if "observability" not in row: row["observability"] = {}
            
            # Simulated cycle if not present
            cycle = row.get("cycle", int(row["barrier_ts"] / 60) % 100)
            
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
                wave_prog = ((cycle % 10) + 1) / 10.0
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
            elif mode.startswith("osc_"):
                import math
                import random
                parts = mode.split("_")
                period = int(parts[1][1:])
                amplitude = int(parts[2][1:]) / 100.0
                noise = 0.0
                if len(parts) > 3 and parts[3].startswith("N"):
                    noise = int(parts[3][1:]) / 100.0
                
                # A cosine wave from 1.0 down to (1.0 - amplitude)
                wave = (math.cos(2 * math.pi * (row_index / period)) + 1) / 2 # 0.0 to 1.0
                
                # Inject bounded stochastic noise
                if noise > 0.0:
                    wave_noise = random.uniform(-noise, noise)
                    wave = max(0.0, min(1.0, wave + wave_noise))
                    
                wave_prog = (1.0 - amplitude) + (amplitude * wave)
                strict = wave_prog
                accept = min(1.0, wave_prog + 0.3)
                lag_stddev = 45.0
                median_lag = int(90 * amplitude)
            elif mode == "plateau_low":
                wave_prog = 0.2
                strict = wave_prog
                accept = wave_prog
                lag_stddev = 10.0
                median_lag = 300
            elif mode == "impulse_shock":
                wave_prog = 0.0 if 2000 < row_index < 2010 else 1.0
                strict = wave_prog
                accept = wave_prog
                lag_stddev = 10.0 if wave_prog == 0.0 else 0.0
                median_lag = 300 if wave_prog == 0.0 else 0
            elif mode == "drift_field":
                wave_prog = max(0.1, 1.0 - (row_index / 4320.0))
                strict = wave_prog
                accept = wave_prog
                lag_stddev = 10.0
                median_lag = int((1.0 - wave_prog) * 100)
            elif mode == "fragmented_regime":
                wave_prog = 1.0 if (row_index // 10) % 2 == 0 else 0.1
                strict = wave_prog
                accept = wave_prog
                lag_stddev = 10.0
                median_lag = 300 if wave_prog == 0.1 else 0
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
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch", type=int, default=10000)
    args = parser.parse_args()
    
    base = Path(f"state_archive/batches/batch_{args.batch}/runs/live/metadata/live_session_steps.jsonl")
    for mode in ["uniform_delay", "bimodal", "rolling_wave", "anticipatory", "collapse"]:
        inject_topology(base, mode)
