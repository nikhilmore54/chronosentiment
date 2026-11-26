import json
import matplotlib.pyplot as plt
from pathlib import Path
import os
import argparse

def plot_observatory(batch_id: int):
    archive_dir = Path(f"state_archive/batches/batch_{batch_id:03d}/runs/live/metadata")
    snapshots_file = archive_dir / "propagation_snapshots.jsonl"
    out_dir = Path("observatory_plots")
    out_dir.mkdir(exist_ok=True)
    
    if not snapshots_file.exists():
        print(f"No snapshots found at {snapshots_file}")
        return
        
    snapshots = []
    with open(snapshots_file, "r") as f:
        for line in f:
            if line.strip():
                snapshots.append(json.loads(line))
                
    if not snapshots:
        print("No snapshots to plot.")
        return
        
    print(f"Generating plots for {len(snapshots)} propagation snapshots...")
    
    # 1. Fragmentation Decay Curve
    plt.figure(figsize=(10, 6))
    for i, snap in enumerate(snapshots):
        curve = snap.get("recovery_curve", [])
        if curve:
            times = [pt["t"] for pt in curve]
            syncs = [pt["sync"] * 100 for pt in curve] # percentage
            
            # Use phase as label if it's the first time we see it, else just color
            label = f"Bar {i+1} ({snap.get('phase', 'UNKNOWN')})"
            plt.plot(times, syncs, marker='o', label=label, linewidth=2, alpha=0.8)
            
    plt.title("Provider Fragmentation Decay Curves (sync_ratio over time)")
    plt.xlabel("Seconds after Anchor Visibility (t)")
    plt.ylabel("Cohort Synchronization Ratio (%)")
    plt.grid(True, linestyle="--", alpha=0.6)
    plt.legend()
    plt.tight_layout()
    plt.savefig(out_dir / f"decay_curves_batch_{batch_id:03d}.png", dpi=200)
    plt.close()
    
    # 2. Entropy and Lag scatter
    plt.figure(figsize=(10, 6))
    lags = [s.get("anchor_lag_ms", 0) for s in snapshots]
    entropies = [s.get("entropy_peak", 0) for s in snapshots]
    
    plt.scatter(lags, entropies, c='crimson', s=100, alpha=0.7, edgecolors='black')
    plt.title("Provider Lag vs Synchronization Entropy")
    plt.xlabel("Anchor Provider Lag (ms)")
    plt.ylabel("Fragmentation Entropy (Peak)")
    plt.grid(True, linestyle="--", alpha=0.6)
    
    for i, snap in enumerate(snapshots):
        plt.annotate(f"B{i+1}", (lags[i], entropies[i]), xytext=(5, 5), textcoords='offset points')
        
    plt.tight_layout()
    plt.savefig(out_dir / f"lag_vs_entropy_batch_{batch_id:03d}.png", dpi=200)
    plt.close()
    
    print(f"✅ Plots saved to {out_dir}/")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch-id", type=int, default=3)
    args = parser.parse_args()
    
    plot_observatory(args.batch_id)
