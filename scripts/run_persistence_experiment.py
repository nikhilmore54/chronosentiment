#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

def run_persistence_experiment():
    print("🔬 PERSISTENCE SCAR CONVERGENCE GEOMETRY EXPERIMENT")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Topology : topo_wavefront")
    print("Strategy : rolling_window_momentum_v2_long")
    print("=" * 70)
    
    windows = [5, 10, 20, 50, 100, 250, 500]
    results = []
    
    ledger = "state_archive/batches/batch_10001/runs/live/metadata/synthetic_rolling_wave_steps.jsonl"
    strategy = "rolling_window_momentum_v2_long"
    
    for w in windows:
        print(f"\n⚡ Running physics harness with WINDOW_SIZE = {w}...")
        subprocess.run([
            "python3", "scripts/signal_physics_harness.py", 
            "--ledger", ledger, 
            "--strategy", strategy, 
            "--substrate", "10001", 
            "--symbol", "BTCUSDT",
            "--window-size", str(w)
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        # Parse the physics ledger to get the geometry
        physics_path = Path(f"state_archive/batches/batch_10001/runs/live/metadata/physics_ledger_{strategy}_synthetic_rolling_wave_steps.jsonl")
        
        is_recovering = False
        recovery_ticks = 0
        completed_recoveries = []
        
        mci_overlaps = []
        mci_distances = []
        
        if not physics_path.exists():
            print(f"❌ Failed to find output for window {w}")
            continue
            
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    mci_overlaps.append(mci["state_overlap_ratio"])
                    mci_distances.append(mci["window_distance"])
                    
                    is_fragmented = mci["state_overlap_ratio"] < 1.0
                    overlap_perfect = mci["state_overlap_ratio"] == 1.0
                    
                    if is_fragmented and not is_recovering:
                        is_recovering = True
                        recovery_ticks = 0
                    elif is_recovering:
                        if overlap_perfect:
                            completed_recoveries.append(recovery_ticks)
                            is_recovering = False
                        else:
                            recovery_ticks += 1
                            
        avg_overlap = round(sum(mci_overlaps)/len(mci_overlaps), 2) if mci_overlaps else 1.0
        avg_distance = round(sum(mci_distances)/len(mci_distances), 2) if mci_distances else 0.0
        
        if completed_recoveries:
            hl = round(sum(completed_recoveries)/len(completed_recoveries), 1)
        else:
            hl = -1.0 if is_recovering else 0.0
            
        results.append({
            "window": w,
            "avg_overlap": avg_overlap,
            "avg_distance": avg_distance,
            "recovery_ticks": hl,
            "scar_state": "Permanent (Non-Convergent)" if hl == -1.0 else "Recoverable (Convergent)"
        })
        
    print("\n" + "=" * 80)
    print(f"{'WINDOW':<10} | {'RECOVERY TICKS':<15} | {'STATE OVERLAP':<15} | {'SCAR STATE':<30}")
    print("-" * 80)
    for r in results:
        hl_str = str(r['recovery_ticks']) if r['recovery_ticks'] != -1.0 else "∞ (-1.0)"
        print(f"{r['window']:<10} | {hl_str:<15} | {r['avg_overlap']:<15} | {r['scar_state']:<30}")
    print("=" * 80)

if __name__ == "__main__":
    run_persistence_experiment()
