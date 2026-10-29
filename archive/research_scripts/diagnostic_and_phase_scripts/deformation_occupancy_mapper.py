#!/usr/bin/env python3
import json
from pathlib import Path

def map_deformation_occupancy():
    print("🔬 DEFORMATION OCCUPANCY MAPPING")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Strategy : rolling_window_momentum_v2_long")
    print("Window   : 50")
    print("=" * 110)
    
    batch_id = 10001
    strategy = "rolling_window_momentum_v2_long"
    
    target_modes = [
        "osc_P5_A25",     # Invariant regime
        "osc_P100_A100",  # Convergent regime (high amp, long period)
        "osc_P50_A100",   # Non-convergent regime (high amp, med period)
        "osc_P5_A100",    # Non-convergent regime (high amp, short period)
    ]
    
    print(f"{'TOPOLOGY':<15} | {'PEAK OCCUPANCY':<15} | {'SATURATION DURATION':<20} | {'MEAN OCCUPANCY':<15} | {'BEHAVIOR':<20}")
    print("-" * 110)
    
    for mode in target_modes:
        topo_file = f"synthetic_{mode}_steps"
        physics_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/physics_ledger_{strategy}_{topo_file}.jsonl")
        
        if not physics_path.exists():
            print(f"{mode:<15} | ERROR: Missing physics ledger")
            continue
            
        occupancy_series = []
        
        with open(physics_path, 'r') as f:
            for line in f:
                if not line.strip(): continue
                row = json.loads(line)
                trace = row.get("state_divergence_trace", {})
                
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    # Deformation occupancy is the fraction of the window that is contaminated
                    # Occupancy = 1.0 - state_overlap_ratio
                    occupancy = round(1.0 - mci["state_overlap_ratio"], 2)
                    occupancy_series.append(occupancy)
                    
        if not occupancy_series:
            print(f"{mode:<15} | ERROR: Empty occupancy series")
            continue
            
        peak_occupancy = max(occupancy_series)
        mean_occupancy = round(sum(occupancy_series) / len(occupancy_series), 2)
        
        # Calculate saturation duration (how many ticks stay at peak_occupancy)
        saturation_ticks = sum(1 for o in occupancy_series if o == peak_occupancy)
        
        if peak_occupancy == 0.0:
            behavior = "Invariant"
        elif occupancy_series[-1] > 0.0:
            behavior = "Non-Convergent"
        else:
            behavior = "Convergent"
            
        print(f"{mode:<15} | {peak_occupancy:<15} | {saturation_ticks:<20} | {mean_occupancy:<15} | {behavior:<20}")
        
        # Save exact series to artifact for deep analysis if needed
        out_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/occupancy_{mode}.json")
        with open(out_path, 'w') as f:
            json.dump({
                "mode": mode,
                "peak_occupancy": peak_occupancy,
                "mean_occupancy": mean_occupancy,
                "saturation_ticks": saturation_ticks,
                "occupancy_series": occupancy_series
            }, f)
            
    print("=" * 110)

if __name__ == "__main__":
    map_deformation_occupancy()
