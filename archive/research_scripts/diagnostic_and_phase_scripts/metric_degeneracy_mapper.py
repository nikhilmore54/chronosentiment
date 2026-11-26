#!/usr/bin/env python3
import json
import math
import subprocess
from pathlib import Path
import sys
sys.path.append(str(Path(__file__).parent))
from synthetic_fragmentation_injector import inject_topology

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
    
    # AC
    if var < 1e-9:
        ac1 = None
        ac10 = None
    else:
        ac1 = sum((series[i] - mean) * (series[i+1] - mean) for i in range(n - 1)) / var
        if n > 10:
            ac10 = sum((series[i] - mean) * (series[i+10] - mean) for i in range(n - 10)) / var
        else:
            ac10 = None
            
    return {"var": var, "ac1": ac1, "ac10": ac10, "ent": ent}

def map_metric_degeneracy():
    print("🔬 METROLOGY VALIDATION: METRIC DEGENERACY BOUNDARIES")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Sweep    : osc_P50 Amplitude 0 to 100")
    print("=" * 135)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    window = 50
    base_ledger = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/live_session_steps.jsonl")
    
    print(f"{'AMPLITUDE':<10} | {'MEAN OCC.':<10} | {'VARIANCE':<12} | {'AC(L=1)':<12} | {'AC(L=10)':<12} | {'ENTROPY':<12} | {'AC VALID?':<12} | {'STATE SENSITIVITY':<20}")
    print("-" * 135)
    
    for a in range(0, 105, 10):
        mode = f"osc_P50_A{a}"
        
        inject_topology(base_ledger, mode)
        topo_file = f"synthetic_{mode}_steps"
        ledger = f"state_archive/batches/batch_{batch_id}/runs/live/metadata/{topo_file}.jsonl"
        
        subprocess.run([
            "python3", "scripts/signal_physics_harness.py", 
            "--ledger", ledger, 
            "--strategy", strategy, 
            "--substrate", str(batch_id), 
            "--symbol", "BTCUSDT",
            "--window-size", str(window)
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
        
        mean_occ = round(sum(series) / len(series), 3)
        metrics = compute_metrics(series)
        var_str = f"{metrics['var']:.2e}"
        ac1_str = str(round(metrics['ac1'], 3)) if metrics['ac1'] is not None else "DEGENERATE"
        ac10_str = str(round(metrics['ac10'], 3)) if metrics['ac10'] is not None else "DEGENERATE"
        ent_str = str(round(metrics['ent'], 3))
        
        ac_valid = "NO" if metrics['ac1'] is None else "YES"
        
        # State Sensitivity Classification
        if metrics['var'] < 1e-9:
            sensitivity = "SATURATION DEGENERACY"
        elif metrics['ent'] < 0.1:
            sensitivity = "LOW VARIANCE COMPRESSION"
        else:
            sensitivity = "INSTRUMENT ACTIVE"
            
        print(f"{a:<10} | {mean_occ:<10} | {var_str:<12} | {ac1_str:<12} | {ac10_str:<12} | {ent_str:<12} | {ac_valid:<12} | {sensitivity:<20}")
        
    print("=" * 135)

if __name__ == "__main__":
    map_metric_degeneracy()
