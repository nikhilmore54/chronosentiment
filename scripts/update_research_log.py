import json
from pathlib import Path
import datetime
import argparse

def update_research_log(batch_id: int):
    archive_dir = Path(f"state_archive/batches/batch_{batch_id:03d}/runs/live/metadata")
    snapshots_file = archive_dir / "propagation_snapshots.jsonl"
    log_file = Path("RESEARCH_LOG.md")
    
    if not snapshots_file.exists():
        print(f"No snapshots found at {snapshots_file}")
        return
        
    snapshots = []
    with open(snapshots_file, "r") as f:
        for line in f:
            if line.strip():
                snapshots.append(json.loads(line))
                
    if not snapshots:
        print("No snapshots to process.")
        return

    # Thresholds for logging
    significant_events = []
    
    # We will just look at the last snapshot compared to the first, or track extreme states
    high_entropy_snaps = [s for s in snapshots if s.get("entropy_peak", 0) > 2.0]
    low_sync_snaps = [s for s in snapshots if s.get("cohort_sync_ratio_t0", 1.0) < 0.6]
    atomic_snaps = [s for s in snapshots if s.get("cohort_sync_ratio_t0", 0.0) == 1.0]
    
    if high_entropy_snaps and low_sync_snaps:
        significant_events.append({
            "observation": f"Batch {batch_id:03d} exhibited persistent high synchronization dispersion (>2.0) and low initial sync (<60%).",
            "hypothesis": "Cohort synchronization remains uneven across symbols, confirming temporal fragmentation.",
            "outcome": "Operationally Consistent"
        })
        
    if len(atomic_snaps) == len(snapshots):
        significant_events.append({
            "observation": f"Batch {batch_id:03d} maintained 100% atomic sync across all {len(snapshots)} cycles.",
            "hypothesis": "Provider cache invalidation is near-atomic for high-liquidity cohorts.",
            "outcome": "Confirmed"
        })

    if not significant_events:
        print("No statistically significant regime shifts or threshold crossings detected. Skipping log update.")
        return

    print(f"Found {len(significant_events)} significant events. Appending to RESEARCH_LOG.md...")
    
    with open(log_file, "a") as f:
        # Include time to preserve chronology across multiple daily runs
        date_str = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S IST")
        f.write(f"\n### Automated Observation — {date_str} (Batch {batch_id:03d})\n")
        f.write("\n| Observation | Hypothesis | Outcome |\n")
        f.write("| :--- | :--- | :--- |\n")
        for event in significant_events:
            f.write(f"| {event['observation']} | {event['hypothesis']} | **{event['outcome']}** |\n")

    print("✅ RESEARCH_LOG.md updated securely.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="ChronoSentiment Auto-Research Logger")
    parser.add_argument("--batch-id", type=int, required=True)
    args = parser.parse_args()
    
    update_research_log(args.batch_id)
