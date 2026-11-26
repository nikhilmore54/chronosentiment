#!/usr/bin/env python3
import json
import statistics
import sys
from pathlib import Path

def verify_reproducibility(ledger_path: Path):
    if not ledger_path.exists():
        print(f"❌ Ledger not found: {ledger_path}")
        sys.exit(1)

    print(f"🔍 Verifying Observability Determinism: {ledger_path}")
    
    history_ratios = []
    cycle_count = 0
    mismatches = 0

    with open(ledger_path, 'r') as f:
        for line in f:
            if not line.strip():
                continue
                
            row = json.loads(line)
            cycle = row['cycle']
            attempted = row.get('symbols_attempted', 0)
            returned = row.get('symbols_returned', 0)
            accepted = row.get('symbols_accepted', 0)
            
            recorded_observability = row.get('observability', {})
            recorded_regime = recorded_observability.get('regime_state')
            recorded_admissible = row.get('admissibility', {}).get('execution_admissible')
            
            if recorded_regime is None:
                continue # Skip cycles without observability block (e.g. old logs)
                
            cycle_count += 1
            
            strict_ratio = returned / attempted if attempted > 0 else 0.0
            acceptance_ratio = accepted / attempted if attempted > 0 else 0.0
            
            history_ratios.append({"strict": strict_ratio, "accept": acceptance_ratio})
            if len(history_ratios) > 5:
                history_ratios.pop(0)
                
            recovery_slope = history_ratios[-1]["accept"] - history_ratios[-2]["accept"] if len(history_ratios) > 1 else 0.0
            
            is_synchronized = (strict_ratio >= 0.9)
            is_degraded = (acceptance_ratio < 0.5)
            is_recovering = (recovery_slope > 0.1)
            
            if is_synchronized:
                derived_regime = "SYNCHRONIZED"
                new_entries_allowed = True
            elif is_degraded and not is_recovering:
                derived_regime = "DEGRADED_OBSERVABILITY"
                new_entries_allowed = False
            elif is_recovering:
                derived_regime = "TRANSITIONAL_RECOVERY"
                new_entries_allowed = False
            else:
                derived_regime = "FRAGMENTED_BUT_USABLE"
                new_entries_allowed = True
                
            derived_admissible = new_entries_allowed
            
            if derived_regime != recorded_regime or derived_admissible != recorded_admissible:
                print(f"❌ Cycle {cycle}: Mismatch!")
                print(f"   Recorded : Regime={recorded_regime}, Admissible={recorded_admissible}")
                print(f"   Derived  : Regime={derived_regime}, Admissible={derived_admissible}")
                mismatches += 1
            else:
                pass # Match

    if mismatches == 0:
        print(f"✅ PERFECT PARITY: All {cycle_count} regimes and admissibility states re-derived deterministically.")
        sys.exit(0)
    else:
        print(f"❌ {mismatches} mismatches found. Admissibility is not reproducible.")
        sys.exit(1)

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=str, default="state_archive/batches/batch_9904/runs/live/metadata/live_session_steps.jsonl")
    args = parser.parse_args()
    verify_reproducibility(Path(args.ledger))
