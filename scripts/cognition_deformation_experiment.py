#!/usr/bin/env python3
import json
import math
import subprocess
from pathlib import Path
import sys
sys.path.append(str(Path(__file__).parent))
from synthetic_fragmentation_injector import inject_topology

def compute_frozen_metrics(series):
    n = len(series)
    if n < 2:
        return {"mean": 0.0, "var": 0.0, "ac1": None, "ent": 0.0}
        
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    
    from collections import Counter
    transitions = [round(series[i] - series[i-1], 2) for i in range(1, n)]
    counts = Counter(transitions)
    total = sum(counts.values())
    ent = -sum((c/total) * math.log2(c/total) for c in counts.values() if c > 0)
    
    if var < 1e-9:
        ac1 = None
    else:
        ac1 = sum((series[i] - mean) * (series[i+1] - mean) for i in range(n - 1)) / var
            
    return {"mean": mean, "var": var, "ac1": ac1, "ent": ent}

def run_cognition_deformation_experiment():
    print("🔬 COGNITION DEFORMATION EXPERIMENT (METROLOGY FROZEN)")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Objective: Compare Cognition Geometry (W=50 vs W=100) under Bounded Topologies")
    print("=" * 115)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    base_ledger = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/live_session_steps.jsonl")
    
    # Selected bounded topologies
    topologies = ["osc_P100_A100", "osc_P50_A100", "osc_P5_A100"]
    windows = [50, 100]
    
    print(f"{'TOPOLOGY':<16} | {'COGNITION (W)':<15} | {'MEAN OCC.':<10} | {'AC(L=1)':<12} | {'TRANS. ENT.':<12} | {'METRIC STATUS':<15}")
    print("-" * 115)
    
    for mode in topologies:
        inject_topology(base_ledger, mode)
        topo_file = f"synthetic_{mode}_steps"
        ledger = f"state_archive/batches/batch_{batch_id}/runs/live/metadata/{topo_file}.jsonl"
        
        for w in windows:
            subprocess.run([
                "python3", "scripts/signal_physics_harness.py", 
                "--ledger", ledger, 
                "--strategy", strategy, 
                "--substrate", str(batch_id), 
                "--symbol", "BTCUSDT",
                "--window-size", str(w)
            ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            
            physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
            
            series = []
            if physics_path.exists():
                with open(physics_path, 'r') as f:
                    for line in f:
                        if not line.strip(): continue
                        row = json.loads(line)
                        trace = row.get("state_divergence_trace", {})
                        if "memory_coherence_index" in trace:
                            series.append(1.0 - trace["memory_coherence_index"]["state_overlap_ratio"])
                            
            if not series: continue
            
            metrics = compute_frozen_metrics(series)
            
            mean_str = f"{metrics['mean']:.3f}"
            ac1_str = str(round(metrics['ac1'], 3)) if metrics['ac1'] is not None else "DEGENERATE"
            ent_str = f"{metrics['ent']:.3f}"
            
            status = "VALID" if metrics['var'] > 1e-9 else "SATURATED"
            
            print(f"{mode:<16} | {'W=' + str(w):<15} | {mean_str:<10} | {ac1_str:<12} | {ent_str:<12} | {status:<15}")
            
    print("=" * 115)

if __name__ == "__main__":
    run_cognition_deformation_experiment()
