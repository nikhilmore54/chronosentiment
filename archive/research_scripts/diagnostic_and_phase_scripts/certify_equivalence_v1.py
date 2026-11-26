import json
import os
import sys
import subprocess
import tempfile
import shutil
import hashlib

def run_command(cmd, cwd=None):
    try:
        result = subprocess.run(cmd, cwd=cwd, check=True, capture_output=True, text=True)
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Command failed: {' '.join(cmd)}\nError: {e.stderr}", file=sys.stderr)
        raise

def certify_equivalence(original_manifest_path):
    if not os.path.exists(original_manifest_path):
        return {
            "status": "EQUIVALENCE_FAILED",
            "failures": [{"error": "MISSING_ORIGINAL_MANIFEST", "details": f"Manifest not found at {original_manifest_path}"}]
        }

    with open(original_manifest_path, 'r') as f:
        manifest = json.load(f)

    replay_id = manifest.get("replay_identity", {}).get("replay_id")
    if not replay_id:
        return {"status": "EQUIVALENCE_FAILED", "failures": [{"error": "MALFORMED_MANIFEST", "details": "Missing replay_id"}]}

    # 1. Path Resolution
    project_root = "core"
    symbols = manifest.get("instrument_metadata", {}).get("symbols", [])
    start_ts = manifest.get("extraction_metadata", {}).get("start_ts")
    timeframe = manifest.get("instrument_metadata", {}).get("timeframe", "5m")
    tf_str = "5m" if "5m" in timeframe else "1m"
    
    clean_replay_id = replay_id
    if clean_replay_id.endswith(f"_{tf_str}"):
        clean_replay_id = clean_replay_id[:-len(f"_{tf_str}")]

    sym = symbols[0].lower() if symbols else "unknown"
    substrate_dir = os.path.abspath(os.path.join(project_root, "chronology", "historical", f"{clean_replay_id}_{tf_str}"))
    substrate_file = os.path.abspath(os.path.join(substrate_dir, f"{sym}_{start_ts}.jsonl"))

    if not os.path.exists(substrate_file):
        return {"status": "EQUIVALENCE_FAILED", "failures": [{"error": "MISSING_SUBSTRATE", "details": f"Cannot find {substrate_file}"}]}

    determinism = manifest.get("determinism_metadata", {})
    cognitions = determinism.get("cognitions", {})
    if "rolling_50" not in cognitions or "event_reset" not in cognitions:
        return {"status": "EQUIVALENCE_FAILED", "failures": [{"error": "MALFORMED_MANIFEST", "details": "Manifest must contain rolling_50 and event_reset cognitions."}]}
    
    cognition_a = "rolling_50"
    cognition_b = "event_reset"
    
    # We'll assume topology name is "osc_50_1.0" or passed if we can't extract it easily
    # Right now, topologies aren't stored as raw strings in the manifest (only hashes).
    # We default to osc_50_1.0. A future iteration might require storing raw topology name in the manifest.
    topology = "osc_50_1.0"

    # 2. Ephemeral Execution
    temp_base = os.path.join(project_root, "artifacts", ".tmp_equivalence")
    os.makedirs(temp_base, exist_ok=True)
    temp_dir = tempfile.mkdtemp(prefix=f"cert_{replay_id}_", dir=temp_base)
    
    try:
        # Run replay A
        cmd_a = [
            "cargo", "run", "--release", "--bin", "trace_replay", "--",
            "--substrate", "tier1_1m",
            "--substrate-file", substrate_file,
            "--topology", topology,
            "--cognition", cognition_a
        ]
        run_command(cmd_a, cwd=project_root)
        
        # Copy trace A to temp
        trace_a_src = os.path.join(project_root, "artifacts", "tier1_1m", topology, cognition_a)
        trace_a_dest = os.path.join(temp_dir, cognition_a)
        shutil.copytree(trace_a_src, trace_a_dest)

        # Run replay B
        cmd_b = [
            "cargo", "run", "--release", "--bin", "trace_replay", "--",
            "--substrate", "tier1_1m",
            "--substrate-file", substrate_file,
            "--topology", topology,
            "--cognition", cognition_b
        ]
        run_command(cmd_b, cwd=project_root)
        
        # Copy trace B to temp
        trace_b_src = os.path.join(project_root, "artifacts", "tier1_1m", topology, cognition_b)
        trace_b_dest = os.path.join(temp_dir, cognition_b)
        shutil.copytree(trace_b_src, trace_b_dest)

        # 3. Temporary Manifest Emission
        config_path = os.path.join(temp_dir, "config.json")
        config = {
            "replay_id": replay_id,
            "replay_class": manifest.get("replay_class", {}).get("replay_class"),
            "authority_type": manifest.get("replay_class", {}).get("authority_type"),
            "session_ontology": manifest.get("session_ontology"),
            "start_ts": start_ts,
            "end_ts": manifest.get("extraction_metadata", {}).get("end_ts"),
            "shift_offset": manifest.get("extraction_metadata", {}).get("shift_offset"),
            "symbols": symbols,
            "timeframe": timeframe,
            "descriptors": manifest.get("replay_descriptor", []),
            "substrate_file": substrate_file,
            "topology": topology,
            "cognition_a": cognition_a,
            "cognition_b": cognition_b,
            "artifact_dir": temp_dir,
            "commit_hash": manifest.get("provenance", {}).get("commit_hash")
        }
        
        with open(config_path, "w") as f:
            json.dump(config, f)

        run_command(["python3", "scripts/emit_manifest_v1.py", config_path])
        
        temp_manifest_path = os.path.join(temp_dir, "manifest.json")
        if not os.path.exists(temp_manifest_path):
             raise Exception("Emitter failed to create temporary manifest.")

        with open(temp_manifest_path, 'r') as f:
            temp_manifest = json.load(f)

        original_id = manifest.get("manifest_id")
        replayed_id = temp_manifest.get("manifest_id")

        # 4. Equivalence Verification
        result = {
            "original_manifest_id": original_id,
            "replayed_manifest_id": replayed_id,
            "attestations": [],
            "failures": []
        }

        if original_id == replayed_id:
            result["attestations"].append("EQUIVALENT_MANIFEST_ID")
        else:
            result["failures"].append({"error": "EQUIVALENCE_MISMATCH", "details": "Manifest IDs diverged."})
            
        # Verify using verify script on temp manifest just to be thorough
        try:
            verify_output = run_command(["python3", "scripts/verify_manifest_v1.py", temp_manifest_path])
            result["attestations"].append("TEMP_MANIFEST_PASSED_VERIFICATION")
        except subprocess.CalledProcessError as e:
            result["failures"].append({"error": "VERIFICATION_FAILED", "details": e.output})

        if len(result["failures"]) == 0:
            result["status"] = "EQUIVALENCE_CERTIFIED"
        else:
            result["status"] = "EQUIVALENCE_FAILED"

        return result

    finally:
        shutil.rmtree(temp_dir)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 certify_equivalence_v1.py <manifest.json>")
        sys.exit(1)
        
    res = certify_equivalence(sys.argv[1])
    print(json.dumps(res, indent=4))
    if res["status"] != "EQUIVALENCE_CERTIFIED":
        sys.exit(1)
