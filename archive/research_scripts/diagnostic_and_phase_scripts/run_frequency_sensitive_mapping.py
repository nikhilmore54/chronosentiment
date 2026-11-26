#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path
sys.path.append(str(Path(__file__).parent))
from synthetic_fragmentation_injector import inject_topology

def run_frequency_sensitive_mapping():
    print("🔬 FREQUENCY-SENSITIVE CONVERGENCE MAPPING")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Target   : W=50 (Non-Convergent under default Wavefront)")
    print("=" * 85)
    
    batch_id = 10001
    base_ledger = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/live_session_steps.jsonl")
    
    # We will vary Period (cadence) and Amplitude (severity)
    periods = [5, 10, 20, 50, 100]
    amplitudes = [25, 50, 75, 100]
    
    strategy = "rolling_window_momentum_v2_long"
    window = 50
    
    print(f"\n{'TOPOLOGY (osc_P_A)':<20} | {'WINDOW':<8} | {'RECOVERY TICKS':<15} | {'STATE OVERLAP':<15}")
    print("-" * 85)
    
    for p in periods:
        for a in amplitudes:
            mode = f"osc_P{p}_A{a}"
            # 1. Inject topology
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
            
            # 3. Parse physics ledger
            physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
            
            is_recovering = False
            recovery_ticks = 0
            completed_recoveries = []
            
            mci_overlaps = []
            
            if not physics_path.exists():
                print(f"{mode:<20} | {window:<8} | ERROR           | ERROR          ")
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
            print(f"{mode:<20} | {window:<8} | {hl_str:<15} | {avg_overlap:<15}")
            
    print("-" * 85)

if __name__ == "__main__":
    run_frequency_sensitive_mapping()
