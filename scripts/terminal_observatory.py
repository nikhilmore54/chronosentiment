import time
import json
import os
from pathlib import Path
import argparse

def render_dashboard(batch_id: int, run_label: str = "live"):
    archive_dir = Path(f"state_archive/batches/batch_{batch_id:03d}/runs/{run_label}/metadata")
    snapshots_file = archive_dir / "propagation_snapshots.jsonl"
    
    try:
        while True:
            os.system('clear' if os.name == 'posix' else 'cls')
            print("=" * 65)
            print(" 🔭 CHRONOSENTIMENT — TEMPORAL PROPAGATION OBSERVATORY")
            print("=" * 65)
            
            if not snapshots_file.exists():
                print(f"\n  [Waiting for traces in batch_{batch_id:03d}...]")
            else:
                with open(snapshots_file, "r") as f:
                    lines = [line.strip() for line in f if line.strip()]
                
                if not lines:
                    print(f"\n  [Waiting for traces in batch_{batch_id:03d}...]")
                else:
                    latest = json.loads(lines[-1])
                    print(f"  Target TS       : {latest.get('target_ts')} | Phase: {latest.get('phase')}")
                    print(f"  Session ID      : {latest.get('session_id')}")
                    print("-" * 65)
                    
                    lag = latest.get('anchor_lag_ms')
                    sync = latest.get('cohort_sync_ratio_t0', 0)
                    entropy = latest.get('entropy_peak', 0)
                    state = latest.get('provider_state', 'UNKNOWN')
                    
                    print(f"  Anchor Lag      : {lag} ms")
                    print(f"  Init Sync (t=0) : {sync:.2%}")
                    print(f"  Entropy Peak    : {entropy}")
                    print(f"  Prop State      : {state}")
                    
                    half_life = latest.get('cohort_sync_half_life_ms')
                    if half_life:
                        print(f"  Sync Half-Life  : {half_life} ms")
                    
                    print("-" * 65)
                    print("  Fragmentation Decay Curve (Convergence Kinetics):")
                    curve = latest.get("recovery_curve", [])
                    if not curve:
                        print("  [No decay curve recorded]")
                    else:
                        for pt in curve:
                            sync_val = pt.get("sync", 0)
                            bar_len = int(sync_val * 40)
                            bar = "█" * bar_len + "░" * (40 - bar_len)
                            print(f"    t={pt.get('t', 0):<3}s | {sync_val:>6.2%} | {bar}")
            
            print("=" * 65)
            print("  Press Ctrl+C to exit. Refreshing every 5s...")
            time.sleep(5)
            
    except KeyboardInterrupt:
        print("\nExiting Observatory Dashboard.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Live Terminal Observatory")
    parser.add_argument("--batch-id", type=int, default=3)
    parser.add_argument("--run-label", default="live")
    args = parser.parse_args()
    
    render_dashboard(args.batch_id, args.run_label)
