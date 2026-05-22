#!/usr/bin/env python3
import json
import math
from pathlib import Path
import statistics

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

def compute_pearson_correlation(x, y):
    n = len(x)
    if n < 2: return 0.0
    mean_x = sum(x) / n
    mean_y = sum(y) / n
    num = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
    den_x = sum((x[i] - mean_x) ** 2 for i in range(n))
    den_y = sum((y[i] - mean_y) ** 2 for i in range(n))
    if den_x == 0 or den_y == 0: return 0.0
    return num / math.sqrt(den_x * den_y)

def map_cross_axis_covariance():
    print("🔬 CROSS-AXIS COVARIANCE MAPPING")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Window   : 50")
    print("=" * 90)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    
    # We will load all available topologies generated so far
    base_dir = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata")
    ledger_files = list(base_dir.glob(f"physics_ledger_{strategy}_synthetic_*_steps.jsonl"))
    
    metrics = {
        "Peak_Occ": [],
        "Mean_Occ": [],
        "Autocorr_L1": [],
        "Autocorr_L10": [],
        "Trans_Entropy": []
    }
    
    for physics_path in ledger_files:
        occupancy_series = []
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    occupancy_series.append(1.0 - mci["state_overlap_ratio"])
                    
        if not occupancy_series: continue
        
        ac1 = compute_autocorrelation(occupancy_series, lag=1)
        ac10 = compute_autocorrelation(occupancy_series, lag=10)
        
        # Exclude degenerate cases (saturated variance) from covariance calculation
        # because they distort correlation geometry.
        if ac1 is None or ac10 is None:
            continue
            
        peak_occ = max(occupancy_series)
        mean_occ = sum(occupancy_series) / len(occupancy_series)
        t_ent = compute_transition_entropy(occupancy_series)
        
        metrics["Peak_Occ"].append(peak_occ)
        metrics["Mean_Occ"].append(mean_occ)
        metrics["Autocorr_L1"].append(ac1)
        metrics["Autocorr_L10"].append(ac10)
        metrics["Trans_Entropy"].append(t_ent)
        
    n_samples = len(metrics["Peak_Occ"])
    print(f"Computed over {n_samples} unsaturated topology realizations.")
    print("-" * 90)
    
    keys = list(metrics.keys())
    print(f"{'':<15} | " + " | ".join(f"{k:<12}" for k in keys))
    print("-" * 90)
    
    for k1 in keys:
        row = []
        for k2 in keys:
            corr = compute_pearson_correlation(metrics[k1], metrics[k2])
            row.append(f"{round(corr, 3):<12}")
        print(f"{k1:<15} | " + " | ".join(row))
        
    print("=" * 90)

if __name__ == "__main__":
    map_cross_axis_covariance()
