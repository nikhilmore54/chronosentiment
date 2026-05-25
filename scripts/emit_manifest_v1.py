import hashlib
import json
import os
import sys
import subprocess
from datetime import datetime, timezone

MANIFEST_VERSION = "1.0.0"
GENERATOR_VERSION = "emitter_v1.0.0"

def get_git_commit():
    try:
        return subprocess.check_output(['git', 'rev-parse', 'HEAD'], stderr=subprocess.DEVNULL).decode('utf-8').strip()
    except Exception:
        return "unknown"

def hash_file(filepath):
    """Returns SHA-256 hash of a file."""
    h = hashlib.sha256()
    if not os.path.exists(filepath):
        return None
    with open(filepath, 'rb') as f:
        while chunk := f.read(8192):
            h.update(chunk)
    return h.hexdigest()

def hash_dict(d):
    """Returns SHA-256 hash of a dictionary (keys sorted)."""
    h = hashlib.sha256()
    h.update(json.dumps(d, sort_keys=True).encode('utf-8'))
    return h.hexdigest()

def emit_manifest(
    replay_id, 
    replay_class, 
    authority_type, 
    session_ontology, 
    start_ts, 
    end_ts, 
    shift_offset, 
    symbols, 
    timeframe, 
    descriptors, 
    substrate_file, 
    topology, 
    cognition_a, 
    cognition_b,
    artifact_dir
):
    
    # 1. Hashes
    chronology_hash = hash_file(substrate_file)
    if not chronology_hash:
        raise FileNotFoundError(f"Substrate file not found: {substrate_file}")

    trace_a_path = os.path.join(artifact_dir, cognition_a, "trace_summary.json")
    trace_b_path = os.path.join(artifact_dir, cognition_b, "trace_summary.json")
    
    trace_a_hash = hash_file(trace_a_path)
    trace_b_hash = hash_file(trace_b_path)
    
    if not trace_a_hash or not trace_b_hash:
        raise FileNotFoundError("Observability traces not found. Has the replay been executed?")

    # Read geometry outputs directly from traces to avoid mismatch
    with open(trace_a_path) as f:
        baseline_a = json.load(f).get("persistence")
    with open(trace_b_path) as f:
        baseline_b = json.load(f).get("persistence")

    topology_hash = hashlib.sha256(topology.encode('utf-8')).hexdigest()
    cognition_a_hash = hashlib.sha256(cognition_a.encode('utf-8')).hexdigest()
    cognition_b_hash = hashlib.sha256(cognition_b.encode('utf-8')).hexdigest()

    schema_def = {
        "version": MANIFEST_VERSION,
        "fields": ["manifest_id", "replay_identity", "replay_class", "session_ontology", "extraction_metadata", "instrument_metadata", "replay_descriptor", "geometry_outputs", "artifact_fingerprints", "determinism_metadata", "provenance"]
    }
    schema_hash = hash_dict(schema_def)

    # 2. Build the structural payload
    manifest = {
        "replay_identity": {
            "replay_id": replay_id,
            "chronology_hash": chronology_hash
        },
        "replay_class": {
            "replay_class": replay_class,
            "authority_type": authority_type
        },
        "session_ontology": session_ontology,
        "extraction_metadata": {
            "start_ts": start_ts,
            "end_ts": end_ts,
            "shift_offset": shift_offset
        },
        "instrument_metadata": {
            "symbols": symbols,
            "timeframe": timeframe
        },
        "replay_descriptor": descriptors,
        "geometry_outputs": {
            "baseline_a": baseline_a,
            "baseline_b": baseline_b
        },
        "artifact_fingerprints": {
            f"tier1_5m/{cognition_a}/trace_summary.json": trace_a_hash,
            f"tier1_5m/{cognition_b}/trace_summary.json": trace_b_hash
        },
        "determinism_metadata": {
            "topology_hash": topology_hash,
            "cognitions": {
                cognition_a: cognition_a_hash,
                cognition_b: cognition_b_hash
            }
        },
        "provenance": {
            "commit_hash": get_git_commit(),
            "manifest_version": MANIFEST_VERSION,
            "schema_hash": schema_hash,
            "generator_version": GENERATOR_VERSION,
            "generated_at": datetime.now(timezone.utc).isoformat()
        }
    }

    # 3. Content-Addressable Identity
    # The manifest_id is the hash of the strictly sorted manifest (excluding the generated_at and manifest_id itself)
    content_for_id = json.loads(json.dumps(manifest))
    del content_for_id['provenance']['generated_at']
    manifest_id = hash_dict(content_for_id)
    
    # Prepend manifest_id
    final_manifest = {"manifest_id": manifest_id}
    final_manifest.update(manifest)

    # 4. Serialize
    out_path = os.path.join(artifact_dir, "manifest.json")
    with open(out_path, 'w') as f:
        json.dump(final_manifest, f, indent=4, sort_keys=True)
        
    print(f"✅ Generated Manifest {manifest_id} at {out_path}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 emit_manifest_v1.py <config.json>")
        sys.exit(1)
        
    config_path = sys.argv[1]
    with open(config_path) as f:
        config = json.load(f)
        
    emit_manifest(**config)
