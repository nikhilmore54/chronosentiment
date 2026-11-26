#!/usr/bin/env python3
"""
Phase A: TRL Summary Extractor & Causal Observability Dashboard.
Parses live_session_steps.jsonl to summarize chronology confidence,
fragmentation, and synchronization transitions for rapid diagnosis.
"""

import argparse
import json
import sys
from pathlib import Path
from statistics import mean

def main():
    parser = argparse.ArgumentParser(description="Extract TRL metrics and transitions from steps log")
    parser.add_argument("--batch-id", type=int, required=True)
    parser.add_argument("--run-label", default="live")
    args = parser.parse_args()

    archive_dir = Path("state_archive/batches") / f"batch_{args.batch_id:03d}" / "runs" / args.run_label
    steps_log = archive_dir / "metadata" / "live_session_steps.jsonl"
    out_summary = archive_dir / "metadata" / "trl_summary.json"

    if not steps_log.exists():
        print(f"❌ Steps log not found at {steps_log}", file=sys.stderr)
        sys.exit(1)

    steps = []
    with open(steps_log) as f:
        for line in f:
            if line.strip():
                try:
                    steps.append(json.loads(line))
                except Exception as e:
                    print(f"⚠️ Error parsing step line: {e}", file=sys.stderr)

    if not steps:
        print("❌ No steps recorded in log file.", file=sys.stderr)
        sys.exit(1)

    # ── Session isolation ────────────────────────────────────────────────
    # Steps log is append-only; filter to most recent session_id to avoid
    # contamination from prior runs in the same archive.
    all_sessions = sorted({s.get("session_id") for s in steps if s.get("session_id")})
    active_session = all_sessions[-1] if all_sessions else None
    if active_session:
        steps = [s for s in steps if s.get("session_id") == active_session]
        legacy_count = sum(1 for s in steps if not s.get("session_id"))
        if legacy_count:
            print(f"  ℹ️  Filtered {legacy_count} legacy step(s) without session_id")
    else:
        active_session = "<legacy-no-session-id>"

    committed = [s for s in steps if s.get("barrier_committed")]
    stalled = [s for s in steps if not s.get("barrier_committed")]

    confidences = [s.get("barrier_confidence", 0.0) for s in steps]
    fragmentations = [s.get("fragmentation_ratio", 0.0) for s in steps]
    health_scores = [s.get("api_health_score", 0.0) for s in steps]
    integrities = [s.get("chronology_integrity", "UNKNOWN") for s in steps]
    failures = [s.get("failure_type") for s in stalled if s.get("failure_type")]

    # Calculate metrics
    peak_part = max((s.get("participating_symbols", 0) for s in steps), default=0)
    avg_conf = mean(confidences) if confidences else 0.0
    avg_frag = mean(fragmentations) if fragmentations else 0.0
    avg_health = mean(health_scores) if health_scores else 0.0

    print("=" * 80)
    print(f"  TEMPORAL RELIABILITY LAYER (TRL) OBSERVED STATE TRANSITIONS")
    print(f"  Batch {args.batch_id:03d} | Run: {args.run_label} | Session: {active_session}")
    print(f"  Total Cycles (this session): {len(steps)}")
    print("=" * 80)
    
    print(f"\n  1. Cohort-Wide Synchronicity Aggregates:")
    print(f"     - Peak Participating Symbols : {peak_part} symbol(s)")
    print(f"     - Average Chronology Conf.   : {avg_conf:.2%}")
    print(f"     - Average Fragmentation Ratio: {avg_frag:.2%}")
    print(f"     - Average API Health Score   : {avg_health:.2%}")

    print(f"\n  2. Chronology Integrity Transition Sequence:")
    transition_str = " → ".join(integrities)
    print(f"     {transition_str}")

    print(f"\n  3. Cycle-by-Cycle Epistemic Matrix:")
    print(f"  {'Cycle':^5} | {'TS':^12} | {'Committed':^9} | {'Confidence':^10} | {'Frag Ratio':^10} | {'Integrity':^12} | {'Triangulation':^22}")
    print("  " + "─" * 90)
    for s in steps:
        ts_val = str(s.get("ts", "STALL"))
        comm_str = "YES" if s.get("barrier_committed") else "NO"
        conf_str = f"{s.get('barrier_confidence', 0.0):.2%}"
        frag_str = f"{s.get('fragmentation_ratio', 0.0):.2%}"
        integ = s.get("chronology_integrity", "UNKNOWN")
        tri = s.get("provider_triangulation")
        if tri:
            tri_str = tri.get("outcome", f"probed={tri.get('symbols_probed',0)} adv={tri.get('advanced','?')}")
        else:
            tri_str = "—"
        print(f"  {s['cycle']:^5} | {ts_val:^12} | {comm_str:^9} | {conf_str:^10} | {frag_str:^10} | {integ:^12} | {tri_str:^22}")

    if stalled:
        print(f"\n  4. Divergence & Causal Stalls Detected:")
        for s in stalled:
            reason = s.get("skip_reason", "UNKNOWN_REASON")
            fail_type = s.get("failure_type", "UNKNOWN_PATHOLOGY")
            target = s.get("target_ts", "NONE")
            print(f"     - Cycle {s['cycle']}: {reason} | Pathology: {fail_type} | Target TS: {target}")
    else:
        print(f"\n  4. Divergence & Causal Stalls Detected:")
        print("     - Zero chronology stalls or provider divergence events recorded. Pure sequence stability.")

    # Export structured summary
    summary_data = {
        "schema_version": 1,
        "batch_id": args.batch_id,
        "run_label": args.run_label,
        "session_id": active_session,
        "total_cycles": len(steps),
        "committed_barriers": len(committed),
        "stalled_barriers": len(stalled),
        "peak_participating_symbols": peak_part,
        "average_chronology_confidence": avg_conf,
        "average_fragmentation_ratio": avg_frag,
        "average_api_health_score": avg_health,
        "chronology_integrity_sequence": integrities,
        "causal_stalls": [
            {
                "cycle": s["cycle"],
                "reason": s.get("skip_reason"),
                "failure_type": s.get("failure_type"),
                "target_ts": s.get("target_ts"),
                "provider_triangulation": s.get("provider_triangulation"),
            } for s in stalled
        ]
    }

    out_summary.parent.mkdir(parents=True, exist_ok=True)
    with open(out_summary, "w") as f:
        json.dump(summary_data, f, indent=4)
    
    print("\n" + "=" * 80)
    print(f"✅ TRL Causal Observability summary saved to {out_summary}")
    print("=" * 80)

if __name__ == "__main__":
    main()
