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

def compute_transition_entropy(series, rounding_decimals):
    if len(series) < 2: return 0.0
    from collections import Counter
    # Binning implicitly via rounding
    transitions = [round(series[i] - series[i-1], rounding_decimals) for i in range(1, len(series))]
    counts = Counter(transitions)
    total = sum(counts.values())
    entropy = -sum((c/total) * math.log2(c/total) for c in counts.values() if c > 0)
    return entropy

def validate_implementation_sensitivity():
    print("🔬 INSTRUMENTATION VALIDATION: IMPLEMENTATION SENSITIVITY")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Window   : 50")
    print("=" * 140)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    
    target_modes = [
        "osc_P100_A100",  
        "osc_P50_A100",   
        "osc_P5_A100",    
        "collapse"
    ]
    
    print(f"{'TOPOLOGY':<16} | {'AC(L=1)':<10} | {'AC(L=2)':<10} | {'AC(L=5)':<10} | {'AC(L=10)':<10} | {'AC(L=20)':<10} | {'ENT(d=1)':<12} | {'ENT(d=2)':<12} | {'ENT(d=3)':<12}")
    print("-" * 140)
    
    for mode in target_modes:
        topo_file = f"synthetic_{mode}_steps"
        physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
        
        if not physics_path.exists():
            continue
            
        occupancy_series = []
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    # Base occupancy extracted accurately
                    occupancy_series.append(1.0 - mci["state_overlap_ratio"])
                    
        if not occupancy_series: continue
        
        # Calculate lag spectrum
        acs = []
        for l in [1, 2, 5, 10, 20]:
            ac = compute_autocorrelation(occupancy_series, lag=l)
            ac_str = str(round(ac, 3)) if ac is not None else "SAT"
            acs.append(ac_str)
            
        # Calculate entropy binning spectrum
        ents = []
        for d in [1, 2, 3]:
            ent = compute_transition_entropy(occupancy_series, rounding_decimals=d)
            ents.append(str(round(ent, 3)))
            
        print(f"{mode:<16} | {acs[0]:<10} | {acs[1]:<10} | {acs[2]:<10} | {acs[3]:<10} | {acs[4]:<10} | {ents[0]:<12} | {ents[1]:<12} | {ents[2]:<12}")
        
    print("=" * 140)

if __name__ == "__main__":
    validate_implementation_sensitivity()
