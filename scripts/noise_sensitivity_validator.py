#!/usr/bin/env python3
import json
import math
import subprocess
from pathlib import Path
import sys
sys.path.append(str(Path(__file__).parent))
from synthetic_fragmentation_injector import inject_topology

def compute_autocorrelation(series, lag=1):
    n = len(series)
    if n <= lag: return 0.0
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    if var < 1e-9: return None
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

def validate_noise_sensitivity():
    print("🔬 INSTRUMENTATION VALIDATION: NOISE SENSITIVITY")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Window   : 50")
    print("Target   : osc_P50_A50")
    print("=" * 115)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    window = 50
    base_ledger = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/live_session_steps.jsonl")
    
    # We will test noise amplitudes N=0%, N=5%, N=10%, N=20%
    noises = [0, 5, 10, 20]
    
    print(f"{'TOPOLOGY':<18} | {'PEAK OCC.':<10} | {'MEAN OCC.':<10} | {'AUTOCORR(1)':<12} | {'TRANS. ENT.':<12} | {'SENSITIVITY (Δ ENT)':<20}")
    print("-" * 115)
    
    baseline_ent = None
    
    for n in noises:
        mode = f"osc_P50_A50_N{n}"
        
        # 1. Inject topology with specific noise
        inject_topology(base_ledger, mode)
        topo_file = f"synthetic_{mode}_steps"
        ledger = f"state_archive/batches/batch_{batch_id}/runs/live/metadata/{topo_file}.jsonl"
        
        # 2. Run physics harness
        subprocess.run([
            "python3", "scripts/signal_physics_harness.py", 
            "--ledger", ledger, 
            "--strategy", strategy, 
            "--substrate", str(batch_id), 
            "--symbol", "BTCUSDT",
            "--window-size", str(window)
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        # 3. Read ledger
        physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
        
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
        
        peak_occ = round(max(occupancy_series), 3)
        mean_occ = round(sum(occupancy_series) / len(occupancy_series), 3)
        ac1 = compute_autocorrelation(occupancy_series, lag=1)
        t_entropy = round(compute_transition_entropy(occupancy_series), 3)
        
        ac1_str = str(round(ac1, 3)) if ac1 is not None else "SATURATED"
        
        if n == 0:
            baseline_ent = t_entropy
            delta_ent_str = "BASELINE"
        else:
            delta_ent = round(t_entropy - baseline_ent, 3)
            delta_ent_str = f"{delta_ent:+} ({(delta_ent/baseline_ent)*100:+.1f}%)" if baseline_ent else "N/A"
            
        print(f"{mode:<18} | {peak_occ:<10} | {mean_occ:<10} | {ac1_str:<12} | {t_entropy:<12} | {delta_ent_str:<20}")
        
    print("=" * 115)

if __name__ == "__main__":
    validate_noise_sensitivity()
