#!/usr/bin/env python3
import json
import math
from pathlib import Path

def compute_autocorrelation(series, lag=1):
    n = len(series)
    if n <= lag: return 0.0
    
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    if var == 0: return 1.0 if mean > 0 else 0.0
    
    cov = sum((series[i] - mean) * (series[i+lag] - mean) for i in range(n - lag))
    return round(cov / var, 3)

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
    
    print(f"{'TOPOLOGY':<18} | {'PEAK OCC.':<12} | {'MEAN OCC.':<12} | {'AUTOCORR (L=1)':<15} | {'AUTOCORR (L=10)':<16} | {'OBSERVED CLASS':<15}")
    print("-" * 125)
    
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
        
        # Horizon-bounded observation class
        if peak_occupancy == 0.0:
            obs_class = "Cτ-0"
        elif occupancy_series[-1] > 0.0:
            obs_class = "NCτ"
        else:
            obs_class = "Cτ"
            
        print(f"{mode:<18} | {peak_occupancy:<12} | {mean_occupancy:<12} | {ac_1:<15} | {ac_10:<16} | {obs_class:<15}")
        
    print("=" * 125)

if __name__ == "__main__":
    map_topology_morphology()
