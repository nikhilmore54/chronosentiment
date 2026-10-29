#!/usr/bin/env python3
import json
import hashlib
from pathlib import Path

def build_survivability_surface():
    print("🔬 ECOLOGICAL SURVIVABILITY SURFACE COMPILER")
    
    atlas_path = Path("archive/datasets/ECOLOGY_ATLAS_v1.json")
    if not atlas_path.exists():
        print("❌ archive/datasets/ECOLOGY_ATLAS_v1.json not found.")
        return
        
    with open(atlas_path, 'r') as f:
        atlas = json.load(f)
        
    metadata_dir = Path("state_archive/batches/batch_10000/runs/live/metadata")
    
    strategies = {}
    
    for record in atlas.get("experiments", []):
        strat_id = record["strategy_id"]
        topo_id = record["topology_id"]
        
        if strat_id not in strategies:
            strategies[strat_id] = {"strategy_id": strat_id, "topologies": []}
            
        # Parse economic ledger
        # Filename example: economic_ledger_momentum_2tick_v1_synthetic_wavefront_steps.jsonl
        # Let's glob to find the right one because the filename varies based on topology ID vs ledger name.
        econ_file = None
        for f in metadata_dir.glob(f"economic_ledger_{strat_id}_*.jsonl"):
            # Check if this matches the topo_id (we map topo_id to expected keywords in filename)
            match = False
            if topo_id == "topo_uniform_60" and "uniform" in f.name: match = True
            elif topo_id == "topo_bimodal_180" and "bimodal" in f.name: match = True
            elif topo_id == "topo_wavefront" and "rolling_wave" in f.name: match = True
            elif topo_id == "topo_anticipatory" and "anticipatory" in f.name: match = True
            elif topo_id == "topo_collapse_300" and "collapse" in f.name: match = True
            
            if match:
                econ_file = f
                break
                
        if not econ_file:
            print(f"⚠️ Economic ledger not found for {strat_id} | {topo_id}")
            continue
            
        econ_tape = []
        with open(econ_file, 'r') as f:
            for line in f:
                if line.strip():
                    econ_tape.append(json.loads(line))
                    
        total_canon = sum(r["canonical_pnl_ticks"] for r in econ_tape)
        total_frag = sum(r["fragmented_pnl_ticks"] for r in econ_tape)
        total_intents = len(econ_tape)
        blocked_intents = sum(1 for r in econ_tape if r["action"] == "BLOCKED")
        
        # Calculate execution_non_realization_rate
        non_realized_rate = round(blocked_intents / total_intents, 4) if total_intents > 0 else 0.0
        
        # Build 4 Planes
        topology_plane = {
            "TSI": record["topology_severity_index"],
            "acceptance_ratio": round(1.0 - non_realized_rate, 2),  # Proxy for data admission rate for actionable intent
            "lag_stddev": "proxy_mapped", # We don't have exact lag stddev tracked in the atlas yet, using proxy
            "recovery_half_life": record["memory_coherence"]["recovery_half_life_ticks"]
        }
        
        state_plane = {
            "MCI_overlap": record["memory_coherence"]["avg_state_overlap"],
            "cognitive_divergences": record["cognitive_divergences"],
            "replay_stable": record["ecology_properties"]["replay_stable"]
        }
        
        economic_plane = {
            "canonical_pnl": round(total_canon, 2),
            "fragmented_pnl": round(total_frag, 2),
            "economic_divergence": round(total_canon - total_frag, 2),
            "execution_non_realization_rate": non_realized_rate
        }
        
        chronology_plane = {
            "ecology_fingerprint_hash": record["ecology_fingerprint_hash"],
            "economic_ledger_hash": econ_tape[0]["replay_hash"] if econ_tape else "empty",
            "state_reconstruction_integrity": "quarantined" if "adaptive" in strat_id else "canonical"
        }
        
        # Deterministic Surface Hashing
        surface_record = {
            "topology_id": topo_id,
            "topology_plane": topology_plane,
            "state_plane": state_plane,
            "economic_plane": economic_plane,
            "chronology_plane": chronology_plane,
            "replay_hash": record["replay_hash"]
        }
        
        # Hash the record itself to create the deterministic surface_hash
        record_str = json.dumps(surface_record, sort_keys=True).encode('utf-8')
        surface_hash = hashlib.sha256(record_str).hexdigest()[:16]
        surface_record["surface_hash"] = surface_hash
        
        strategies[strat_id]["topologies"].append(surface_record)
        
    surface_artifact = {
        "surface_id": "batch_10000_surface_v1",
        "substrate_id": "batch_10000",
        "strategies": list(strategies.values())
    }
    
    out_file = Path("archive/datasets/SURVIVABILITY_SURFACE_ARTIFACT_v1.json")
    with open(out_file, 'w') as f:
        json.dump(surface_artifact, f, indent=2)
        
    print(f"💾 Compiled Deterministic Surface Artifact to {out_file.name}")
    print("   Total Strategies Mapped: ", len(strategies))

if __name__ == "__main__":
    build_survivability_surface()
