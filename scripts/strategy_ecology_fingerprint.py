#!/usr/bin/env python3
import json
import hashlib
from pathlib import Path
import sys
import subprocess

def compute_fingerprint(physics_ledger_path: Path, synthetic_ledger_path: Path, topology_id: str, strategy_id: str):
    if not physics_ledger_path.exists() or not synthetic_ledger_path.exists():
        print(f"❌ Missing ledger for {topology_id} - {strategy_id}")
        return None
        
    execution_tape = []
    diverged_count = 0
    mci_overlaps = []
    mci_distances = []
    
    # Repair ecology tracking
    repair_induced_divergences = 0
    canonical_intent_matches = 0
    fragmented_divergences = 0
    
    # State tracking for Memory Recovery Half-Life
    is_recovering = False
    recovery_ticks = 0
    completed_recoveries = []
    
    with open(physics_ledger_path, 'r') as f:
        for line in f:
            if line.strip():
                row = json.loads(line)
                execution_tape.append(row)
                trace = row.get("state_divergence_trace", {})
                
                intent_live = trace.get("intent_live")
                intent_frag = trace.get("intent_fragmented")
                intent_rep = trace.get("intent_repaired")
                
                if trace.get("divergence_reason"):
                    diverged_count += 1
                    
                if intent_live and intent_frag and intent_rep:
                    frag_wrong = (intent_frag != intent_live)
                    rep_wrong = (intent_rep != intent_live)
                    
                    if frag_wrong: fragmented_divergences += 1
                    
                    if frag_wrong and not rep_wrong:
                        canonical_intent_matches += 1
                    elif not frag_wrong and rep_wrong:
                        repair_induced_divergences += 1
                        
                if "memory_coherence_index" in trace:
                    mci = trace["memory_coherence_index"]
                    mci_overlaps.append(mci["state_overlap_ratio"])
                    mci_distances.append(mci["window_distance"])
                    
                    # Memory Recovery Half-Life Logic
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
                    
    regimes = []
    lags = []
    with open(synthetic_ledger_path, 'r') as f:
        for line in f:
            if line.strip():
                try:
                    row = json.loads(line)
                    regimes.append(row["observability"]["regime_state"])
                    lags.append(row["freshness"]["median_symbol_lag_sec"])
                except Exception:
                    pass
                    
    generated = len(execution_tape)
    executed = sum(1 for r in execution_tape if r['action'].startswith('EXEC'))
    blocked = sum(1 for r in execution_tape if r['action'] == 'BLOCKED')
    held = sum(1 for r in execution_tape if r['action'] == 'HELD')
    
    # We only care about execution rate of intents that were NOT HELD.
    actionable_intents = executed + blocked
    execution_rate = round(executed / actionable_intents, 2) if actionable_intents > 0 else 0
    
    regime_counts = {}
    for r in regimes:
        regime_counts[r] = regime_counts.get(r, 0) + 1
        
    total_regimes = len(regimes) or 1
    regime_exposure = {k: round(v / total_regimes, 2) for k, v in regime_counts.items()}
    
    if lags:
        median_lag = sorted(lags)[len(lags)//2]
    else:
        median_lag = 0
        
    tsi_map = {
        "topo_uniform_60": 0.2,
        "topo_wavefront": 0.4,
        "topo_bimodal_180": 0.6,
        "topo_anticipatory": 0.7,
        "topo_collapse_300": 1.0
    }
    
    # Basic fingerprint structure
    fingerprint = {
        "strategy_id": strategy_id,
        "topology_id": topology_id,
        "intent_sequence_stable": (diverged_count == 0),
        "cognitive_divergences": diverged_count,
        "topology_severity_index": tsi_map.get(topology_id, 0.0),
        "execution_stats": {
            "generated": generated,
            "executed": executed,
            "blocked": blocked,
            "execution_rate": execution_rate
        },
        "memory_coherence": {
            "avg_window_distance": round(sum(mci_distances)/len(mci_distances), 2) if mci_distances else 0.0,
            "avg_state_overlap": round(sum(mci_overlaps)/len(mci_overlaps), 2) if mci_overlaps else 1.0,
            "recovery_half_life_ticks": round(sum(completed_recoveries)/len(completed_recoveries), 1) if completed_recoveries else (-1.0 if is_recovering else 0.0)
        },
        "repair_ecology": {
            "repair_induced_divergences": repair_induced_divergences,
            "canonical_intent_matches": canonical_intent_matches,
            "raw_fragmented_divergences": fragmented_divergences
        } if "adaptive" in strategy_id else {},
        "regime_exposure": regime_exposure,
        "lag_profile": {
            "median_lag": median_lag
        },
        "ecology_properties": {
            "sync_sensitivity": round(regime_exposure.get("SYNCHRONIZED", 0.0), 2),
            "fragmentation_tolerance": execution_rate,
            "collapse_resilience": 1.0 if (topology_id == "topo_collapse_300" and blocked > 0 and executed == 0) else 0.0,
            "replay_stable": True
        }
    }
    
    # Get replay artifact hash from the last item in the execution tape or calculate it
    tape_str = json.dumps(execution_tape, sort_keys=True).encode('utf-8')
    fingerprint["replay_hash"] = hashlib.sha256(tape_str).hexdigest()[:16]
    
    # Generate the master ecology fingerprint hash
    hash_str = json.dumps(fingerprint, sort_keys=True).encode('utf-8')
    fingerprint["ecology_fingerprint_hash"] = hashlib.sha256(hash_str).hexdigest()[:16]
    
    return fingerprint

def run_mapper(batch_id: int = 10000):
    print("🔬 ECOLOGY MAPPING ENGINE v1")
    
    topologies = {
        "topo_uniform_60": "synthetic_uniform_delay_steps",
        "topo_bimodal_180": "synthetic_bimodal_steps",
        "topo_wavefront": "synthetic_rolling_wave_steps",
        "topo_anticipatory": "synthetic_anticipatory_steps",
        "topo_collapse_300": "synthetic_collapse_steps"
    }
    
    metadata_dir = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata")
    
    registry_file = Path("ECOLOGY_ATLAS_v1.json")
    if registry_file.exists():
        with open(registry_file, 'r') as f:
            registry = json.load(f)
    else:
        registry = {}
        
    registry["version"] = "v1.1"
    registry["experiments"] = registry.get("experiments", [])
    
    strategies = [
        "momentum_2tick_v1", 
        "mean_reversion_2tick_v1", 
        "mean_reversion_2tick_v2_adaptive",
        "rolling_window_momentum_v1",
        "rolling_window_momentum_v2_long",
        "rolling_window_momentum_v3_adaptive"
    ]
    
    for strategy_id in strategies:
        for topo_id, file_stem in topologies.items():
            physics_path = metadata_dir / f"physics_ledger_{strategy_id}_{file_stem}.jsonl"
            synthetic_path = metadata_dir / f"{file_stem}.jsonl"
            
            if not physics_path.exists():
                print(f"⚡ Running physics harness for {topo_id} | {strategy_id}...")
                symbol_arg = "BTCUSDT" if batch_id == 10001 else "AAPL"
                subprocess.run(["python3", "scripts/signal_physics_harness.py", "--ledger", str(synthetic_path), "--strategy", strategy_id, "--substrate", str(batch_id), "--symbol", symbol_arg], check=True)
                
            fp = compute_fingerprint(physics_path, synthetic_path, topo_id, strategy_id)
            if fp:
                # Upsert into registry
                existing_idx = next((i for i, exp in enumerate(registry["experiments"]) if exp["topology_id"] == topo_id and exp["strategy_id"] == fp["strategy_id"]), None)
                if existing_idx is not None:
                    registry["experiments"][existing_idx] = fp
                else:
                    registry["experiments"].append(fp)
                    
                print(f"\n[{strategy_id} | {topo_id}] -> Ecology Hash: {fp['ecology_fingerprint_hash']}")
                print(json.dumps(fp, indent=2))
            
    with open(registry_file, 'w') as f:
        json.dump(registry, f, indent=2)
    print(f"\n💾 Saved {len(registry['experiments'])} fingerprints to ECOLOGY_ATLAS_v1.json")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch", type=int, default=10000)
    args = parser.parse_args()
    run_mapper(args.batch)
