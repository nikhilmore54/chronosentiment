import json
import glob
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse

def get_market_phase(ts: int) -> str:
    """Classify the target_ts into a market phase (IST)."""
    IST = timezone(timedelta(hours=5, minutes=30))
    dt = datetime.fromtimestamp(ts, tz=timezone.utc).astimezone(IST)
    
    t = dt.time()
    if t.hour == 9 and 15 <= t.minute <= 30:
        return "OPEN_TRANSITION"
    elif t.hour == 15 and 0 <= t.minute <= 30:
        return "CLOSE_TRANSITION"
    elif t.hour == 9 and t.minute < 15:
        return "PRE_OPEN_AUCTION"
    else:
        return "MIDDAY_CONTINUOUS"

def generate_snapshots(batch_id: int, run_label: str = "live"):
    archive_dir = Path(f"state_archive/batches/batch_{batch_id:03d}/runs/{run_label}/metadata")
    steps_log = archive_dir / "live_session_steps.jsonl"
    out_file = archive_dir / "propagation_snapshots.jsonl"
    
    if not steps_log.exists():
        print(f"No steps log found at {steps_log}")
        return

    snapshots = []
    
    with open(steps_log, "r") as f:
        for line in f:
            if not line.strip():
                continue
            try:
                step = json.loads(line)
            except json.JSONDecodeError:
                continue
                
            telemetry = step.get("propagation_telemetry")
            if not telemetry:
                continue
                
            # Derive the regime signature
            session_id = step.get("session_id", "unknown")
            target_ts = step.get("target_ts")
            if not target_ts:
                continue
                
            phase = get_market_phase(target_ts)
            
            snapshot = {
                "session_id": session_id,
                "target_ts": target_ts,
                "phase": phase,
                "anchor_lag_ms": telemetry.get("provider_lag_ms"),
                "cohort_sync_ratio_t0": telemetry.get("cohort_sync_ratio"),
                "cohort_sync_half_life_ms": telemetry.get("cohort_sync_half_life_ms"),
                "entropy_peak": telemetry.get("fragmentation_entropy"),
                "provider_state": telemetry.get("propagation_state"),
                "exchange_to_observer_latency_ms": telemetry.get("exchange_to_observer_latency_ms"),
                "recovery_curve": telemetry.get("fragmentation_decay_curve", [])
            }
            snapshots.append(snapshot)

    with open(out_file, "w") as f:
        for s in snapshots:
            f.write(json.dumps(s) + "\n")
            
    print(f"✅ Generated {len(snapshots)} Propagation Regime Snapshots -> {out_file}")
    
    # Print a quick summary of the latest
    if snapshots:
        latest = snapshots[-1]
        print("\nLatest Snapshot Signature:")
        print(json.dumps(latest, indent=2))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate Propagation Regime Snapshots")
    parser.add_argument("--batch-id", type=int, default=3)
    parser.add_argument("--run-label", default="live")
    args = parser.parse_args()
    
    generate_snapshots(args.batch_id, args.run_label)
