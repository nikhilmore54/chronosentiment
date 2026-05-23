#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

def run_cross_topology_matrix():
    print("🔬 CROSS-TOPOLOGY CONVERGENCE MATRIX")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("=" * 85)
    
    windows = [5, 10, 20, 50, 100]
    topologies = {
        "topo_uniform_60": "synthetic_uniform_delay_steps",
        "topo_bimodal_180": "synthetic_bimodal_steps",
        "topo_wavefront": "synthetic_rolling_wave_steps",
        "topo_collapse_300": "synthetic_collapse_steps"
    }
    
    strategy = "rolling_window_momentum_v2_long"
    
    print(f"\n{'TOPOLOGY':<20} | {'WINDOW':<8} | {'RECOVERY TICKS':<15} | {'STATE OVERLAP':<15}")
    print("-" * 85)
    
    # Run the physics harnesses and parse immediately
    for topo_name, topo_file in topologies.items():
        ledger = f"state_archive/batches/batch_10001/runs/live/metadata/{topo_file}.jsonl"
        for w in windows:
            subprocess.run([
                "python3", "scripts/signal_physics_harness.py", 
                "--ledger", ledger, 
                "--strategy", strategy, 
                "--substrate", "10001", 
                "--symbol", "BTCUSDT",
                "--window-size", str(w)
            ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            
            physics_path = Path(f"state_archive/batches/batch_10001/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
            
            is_recovering = False
            recovery_ticks = 0
            completed_recoveries = []
            
            mci_overlaps = []
            
            if not physics_path.exists():
                print(f"{topo_name:<20} | {w:<8} | ERROR           | ERROR          ")
                continue
                
            with open(physics_path, 'r') as f:
                for line in f:
                    if not line.strip(): continue
                    row = json.loads(line)
                    trace = row.get("state_divergence_trace", {})
                    
                    if "memory_coherence_index" in trace:
                        mci = trace["memory_coherence_index"]
                        mci_overlaps.append(mci["state_overlap_ratio"])
                        
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
            
            if completed_recoveries:
                hl = round(sum(completed_recoveries)/len(completed_recoveries), 1)
            else:
                hl = -1.0 if is_recovering else 0.0
                
            hl_str = str(hl) if hl != -1.0 else "∞ (-1.0)"
            print(f"{topo_name:<20} | {w:<8} | {hl_str:<15} | {avg_overlap:<15}")
        print("-" * 85)

if __name__ == "__main__":
    run_cross_topology_matrix()
