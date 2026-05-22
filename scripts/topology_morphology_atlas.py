#!/usr/bin/env python3
import json
import math
from pathlib import Path

def compute_autocorrelation(series, lag=1):
    n = len(series)
    if n <= lag: return 0.0
    
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    if var < 1e-9: return "SATURATED" if mean > 0.01 else 0.0
    
    cov = sum((series[i] - mean) * (series[i+lag] - mean) for i in range(n - lag))
    return round(cov / var, 3)

def compute_transition_entropy(series):
    if len(series) < 2: return 0.0
    from collections import Counter
    # Measure Shannon entropy of occupancy first-order differences
    transitions = [round(series[i] - series[i-1], 3) for i in range(1, len(series))]
    counts = Counter(transitions)
    total = sum(counts.values())
    entropy = -sum((c/total) * math.log2(c/total) for c in counts.values() if c > 0)
    return round(entropy, 3)

def map_topology_morphology():
    print("🔬 TOPOLOGY MORPHOLOGY ATLAS")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Window   : 50")
    print("=" * 125)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    
    target_modes = [
        "osc_P5_A25",     
        "osc_P100_A100",  
        "osc_P50_A100",   
        "osc_P5_A100",    
        "collapse",
        "uniform_delay",
        "bimodal"
    ]
    
    print(f"{'TOPOLOGY':<18} | {'PEAK OCC.':<10} | {'MEAN OCC.':<10} | {'AUTOCORR(1)':<12} | {'AUTOCORR(10)':<12} | {'TRANS. ENTROPY':<15} | {'OBSERVED CLASS':<15}")
    print("-" * 120)
    
    for mode in target_modes:
        topo_file = f"synthetic_{mode}_steps"
        physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
        
        if not physics_path.exists():
            print(f"{mode:<18} | ERROR: Missing physics ledger")
            continue
            
        occupancy_series = []
        
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    occupancy = round(1.0 - mci["state_overlap_ratio"], 3)
                    occupancy_series.append(occupancy)
                    
        if not occupancy_series:
            print(f"{mode:<18} | ERROR: Empty occupancy series")
            continue
            
        peak_occupancy = max(occupancy_series)
        mean_occupancy = round(sum(occupancy_series) / len(occupancy_series), 3)
        
        ac_1 = compute_autocorrelation(occupancy_series, lag=1)
        ac_10 = compute_autocorrelation(occupancy_series, lag=10)
        t_entropy = compute_transition_entropy(occupancy_series)
        
        # Horizon-bounded observation class
        if peak_occupancy == 0.0:
            obs_class = "Cτ-0"
        elif occupancy_series[-1] > 0.0:
            obs_class = "NCτ"
        else:
            obs_class = "Cτ"
            
        ac1_str = str(ac_1)
        ac10_str = str(ac_10)
        print(f"{mode:<18} | {peak_occupancy:<10} | {mean_occupancy:<10} | {ac1_str:<12} | {ac10_str:<12} | {t_entropy:<15} | {obs_class:<15}")
        
    print("=" * 120)

if __name__ == "__main__":
    map_topology_morphology()
