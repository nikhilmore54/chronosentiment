#!/usr/bin/env python3
import json
import math
from pathlib import Path

def compute_autocorrelation(series, lag=1):
    n = len(series)
    if n <= lag: return 0.0
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    if var < 1e-9: return None # Saturated
    cov = sum((series[i] - mean) * (series[i+lag] - mean) for i in range(n - lag))
    return cov / var

def compute_transition_entropy(series):
    if len(series) < 2: return 0.0
    from collections import Counter
    transitions = [round(series[i] - series[i-1], 3) for i in range(1, len(series))]
    counts = Counter(transitions)
    total = sum(counts.values())
    entropy = -sum((c/total) * math.log2(c/total) for c in counts.values() if c > 0)
    return entropy

def validate_horizon_sensitivity():
    print("🔬 INSTRUMENTATION VALIDATION: HORIZON SENSITIVITY")
    print("Substrate: BTCUSDT (batch_10001)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Window   : 50")
    print("=" * 115)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    
    target_modes = [
        "osc_P5_A25",     
        "osc_P100_A100",  
        "osc_P5_A100",    
        "collapse"
    ]
    
    # 72 hours is 4320 ticks (assuming 1 tick/minute).
    # We will test tau = 1440 (24h), 2880 (48h), 4320 (72h)
    horizons = [1440, 2880, 4320]
    
    print(f"{'TOPOLOGY':<15} | {'τ HORIZON':<10} | {'PEAK OCC.':<10} | {'MEAN OCC.':<10} | {'AUTOCORR(1)':<12} | {'TRANS. ENT.':<12} | {'HORIZON CLASS':<15}")
    print("-" * 115)
    
    for mode in target_modes:
        topo_file = f"synthetic_{mode}_steps"
        physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
        
        if not physics_path.exists():
            continue
            
        full_occupancy_series = []
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    occupancy = round(1.0 - mci["state_overlap_ratio"], 3)
                    full_occupancy_series.append(occupancy)
                    
        if not full_occupancy_series:
            continue
            
        for tau in horizons:
            series = full_occupancy_series[:tau]
            if not series: continue
            
            peak_occ = max(series)
            mean_occ = round(sum(series) / len(series), 3)
            ac1 = compute_autocorrelation(series, lag=1)
            t_entropy = round(compute_transition_entropy(series), 3)
            
            # Horizon-bounded observation class for this specific tau
            if peak_occ == 0.0:
                obs_class = f"C(τ={tau})-0"
            elif series[-1] > 0.0:
                obs_class = f"NC(τ={tau})"
            else:
                obs_class = f"C(τ={tau})"
                
            ac1_str = str(round(ac1, 3)) if ac1 is not None else "SATURATED"
            print(f"{mode:<15} | {tau:<10} | {peak_occ:<10} | {mean_occ:<10} | {ac1_str:<12} | {t_entropy:<12} | {obs_class:<15}")
            
        print("-" * 115)

if __name__ == "__main__":
    validate_horizon_sensitivity()
