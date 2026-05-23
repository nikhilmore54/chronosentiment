#!/usr/bin/env python3
import json
import math
from pathlib import Path

def compute_metrics(series):
    n = len(series)
    if n < 2:
        return {"var": 0.0, "ac1": None, "ac10": None, "ent": 0.0}
        
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    
    # Entropy (d=2 binning)
    from collections import Counter
    transitions = [round(series[i] - series[i-1], 2) for i in range(1, n)]
    counts = Counter(transitions)
    total = sum(counts.values())
    ent = -sum((c/total) * math.log2(c/total) for c in counts.values() if c > 0)
    
    if var < 1e-9:
        ac1 = None
        ac10 = None
    else:
        ac1 = sum((series[i] - mean) * (series[i+1] - mean) for i in range(n - 1)) / var
        if n > 10:
            ac10 = sum((series[i] - mean) * (series[i+10] - mean) for i in range(n - 10)) / var
        else:
            ac10 = None
            
    return {"mean": mean, "var": var, "ac1": ac1, "ac10": ac10, "ent": ent}

def apply_representation(overlap_series, variant):
    represented = []
    for overlap in overlap_series:
        occ = 1.0 - overlap
        if variant == "linear":
            represented.append(occ)
        elif variant == "squared":
            represented.append(occ ** 2)
        elif variant == "sqrt":
            represented.append(math.sqrt(occ))
        elif variant == "binary_thresh":
            represented.append(1.0 if occ >= 0.5 else 0.0)
    return represented

def map_representation_sensitivity():
    print("🔬 METROLOGY VALIDATION: REPRESENTATION SENSITIVITY MAPPING")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("=" * 135)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    
    target_modes = [
        "osc_P100_A100",  
        "osc_P50_A100",   
        "osc_P5_A100"
    ]
    
    variants = ["linear", "squared", "sqrt", "binary_thresh"]
    
    print(f"{'TOPOLOGY':<16} | {'REPRESENTATION':<15} | {'MEAN OCC.':<10} | {'VARIANCE':<12} | {'AC(L=1)':<12} | {'AC(L=10)':<12} | {'ENTROPY':<12}")
    print("-" * 135)
    
    for mode in target_modes:
        topo_file = f"synthetic_{mode}_steps"
        physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
        
        if not physics_path.exists():
            continue
            
        overlap_series = []
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                if "memory_coherence_index" in trace:
                    overlap_series.append(trace["memory_coherence_index"]["state_overlap_ratio"])
                    
        if not overlap_series: continue
        
        for variant in variants:
            occ_series = apply_representation(overlap_series, variant)
            metrics = compute_metrics(occ_series)
            
            mean_str = f"{metrics['mean']:.3f}"
            var_str = f"{metrics['var']:.2e}"
            ac1_str = str(round(metrics['ac1'], 3)) if metrics['ac1'] is not None else "DEGENERATE"
            ac10_str = str(round(metrics['ac10'], 3)) if metrics['ac10'] is not None else "DEGENERATE"
            ent_str = str(round(metrics['ent'], 3))
            
            print(f"{mode:<16} | {variant:<15} | {mean_str:<10} | {var_str:<12} | {ac1_str:<12} | {ac10_str:<12} | {ent_str:<12}")
            
        print("-" * 135)
        
if __name__ == "__main__":
    map_representation_sensitivity()
