import json
import os
import shutil
import subprocess
import sys
import hashlib

# Sandbox-safe temp dir
TMP_DIR = "core/artifacts/.tmp_rejection_tests"
CANONICAL_MANIFEST_PATH = "core/artifacts/phase2e_q/2026_nvda_sync_oscillatory_fragmentation/tier1_5m/manifest.json"
CANONICAL_DIR = os.path.dirname(CANONICAL_MANIFEST_PATH)

def hash_dict(d):
    """Returns SHA-256 hash of a dictionary with sorted keys."""
    h = hashlib.sha256()
    h.update(json.dumps(d, sort_keys=True, separators=(',', ':')).encode('utf-8'))
    return h.hexdigest()

def run_verifier(manifest_path):
    try:
        res = subprocess.run(["python3", "scripts/verify_manifest_v1.py", manifest_path], capture_output=True, text=True)
        return json.loads(res.stdout)
    except Exception as e:
        print(f"Error running verifier: {e}")
        return None

def run_certifier(manifest_path):
    try:
        res = subprocess.run(["python3", "scripts/certify_equivalence_v1.py", manifest_path], capture_output=True, text=True)
        return json.loads(res.stdout)
    except subprocess.CalledProcessError as e:
        return json.loads(e.output)
    except Exception as e:
        print(f"Error running certifier: {e}")
        return None

def setup_test_env():
    if os.path.exists(TMP_DIR):
        shutil.rmtree(TMP_DIR)
    shutil.copytree(CANONICAL_DIR, TMP_DIR)
    return os.path.join(TMP_DIR, "manifest.json")

def print_result(name, passed, msg):
    status = "✅ PASS" if passed else "❌ FAIL"
    print(f"[{status}] {name}: {msg}")

def test_1_manifest_tampering_unsigned():
    test_manifest = setup_test_env()
    
    with open(test_manifest, 'r') as f:
        data = json.load(f)
        
    # Mutate baseline_a without updating manifest_id
    data["geometry_outputs"]["baseline_a"] = 99
    
    with open(test_manifest, 'w') as f:
        json.dump(data, f, indent=4)
        
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res["status"] == "ATTESTATION_FAILED":
        for failure in res["failures"]:
            if failure["error"] == "HASH_MISMATCH" and failure["component"] == "manifest_id":
                passed = True
                msg = "Correctly rejected tampered manifest_id."
                break
    
    if not passed:
        msg = f"Failed to reject unsigned tampering properly. Res: {res}"
        
    print_result("Test 1: Manifest Tampering (Unsigned)", passed, msg)
    return passed

def test_2_manifest_tampering_signed():
    test_manifest = setup_test_env()
    
    with open(test_manifest, 'r') as f:
        data = json.load(f)
        
    # Mutate baseline_a
    data["geometry_outputs"]["baseline_a"] = 99
    
    # Recalculate manifest_id (sign it)
    content_for_id = json.loads(json.dumps(data))
    if "manifest_id" in content_for_id:
        del content_for_id["manifest_id"]
    if "provenance" in content_for_id and "generated_at" in content_for_id["provenance"]:
        del content_for_id["provenance"]["generated_at"]
        
    data["manifest_id"] = hash_dict(content_for_id)
    
    with open(test_manifest, 'w') as f:
        json.dump(data, f, indent=4)
        
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res["status"] == "ATTESTATION_FAILED":
        for failure in res["failures"]:
            if failure["error"] == "GEOMETRY_MISMATCH" and failure["component"] == "baseline_a":
                passed = True
                msg = "Correctly rejected GEOMETRY_MISMATCH on signed tamper."
                break
                
    if not passed:
        msg = f"Failed to reject signed tampering properly. Res: {res}"
        
    print_result("Test 2: Manifest Tampering (Signed)", passed, msg)
    return passed

def test_3_trace_file_corruption():
    test_manifest = setup_test_env()
    
    # Corrupt the rolling_50 trace
    trace_path = os.path.join(TMP_DIR, "rolling_50", "trace_summary.json")
    with open(trace_path, 'r') as f:
        trace_data = json.load(f)
        
    trace_data["persistence"] = 99
    
    with open(trace_path, 'w') as f:
        json.dump(trace_data, f)
        
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res["status"] == "ATTESTATION_FAILED":
        for failure in res["failures"]:
            if failure["error"] == "HASH_MISMATCH" and "rolling_50/trace_summary.json" in failure["component"]:
                passed = True
                msg = "Correctly rejected trace file corruption (HASH_MISMATCH)."
                break
                
    if not passed:
        msg = f"Failed to reject trace file corruption. Res: {res}"
        
    print_result("Test 3: Trace File Corruption", passed, msg)
    return passed

def test_4_missing_artifact():
    test_manifest = setup_test_env()
    
    # Delete the event_reset trace
    trace_path = os.path.join(TMP_DIR, "event_reset", "trace_summary.json")
    os.remove(trace_path)
    
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res["status"] == "ATTESTATION_FAILED":
        for failure in res["failures"]:
            if failure["error"] == "MISSING_ARTIFACT" and "event_reset/trace_summary.json" in failure["details"]:
                passed = True
                msg = "Correctly rejected missing artifact."
                break
                
    if not passed:
        msg = f"Failed to reject missing artifact. Res: {res}"
        
    print_result("Test 4: Missing Artifact Extraction", passed, msg)
    return passed

def test_5_substrate_mutability_rejection():
    test_manifest = setup_test_env()
    
    with open(test_manifest, 'r') as f:
        data = json.load(f)
        
    # Mutate expected chronology_hash
    # Update manifest ID so it passes that check first
    data["replay_identity"]["chronology_hash"] = "invalid_hash_xyz"
    
    content_for_id = json.loads(json.dumps(data))
    if "manifest_id" in content_for_id:
        del content_for_id["manifest_id"]
    if "provenance" in content_for_id and "generated_at" in content_for_id["provenance"]:
        del content_for_id["provenance"]["generated_at"]
    data["manifest_id"] = hash_dict(content_for_id)
    
    with open(test_manifest, 'w') as f:
        json.dump(data, f, indent=4)
        
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res["status"] == "ATTESTATION_FAILED":
        for failure in res["failures"]:
            if failure["error"] == "HASH_MISMATCH" and failure["component"] == "chronology_hash":
                passed = True
                msg = "Correctly rejected invalid substrate hash."
                break
                
    if not passed:
        msg = f"Failed to reject invalid substrate hash. Res: {res}"
        
    print_result("Test 5: Substrate Mutability Rejection", passed, msg)
    return passed

def test_6_equivalence_certifier_rejection():
    test_manifest = setup_test_env()
    
    # We want certify_equivalence_v1.py to fail.
    # To do this, we can give it an invalid manifest_id in the original manifest.
    # The certifier will run the replay, generate a new manifest, and compare the IDs.
    # It should fail with EQUIVALENCE_MISMATCH.
    with open(test_manifest, 'r') as f:
        data = json.load(f)
        
    data["manifest_id"] = "fake_original_id_123"
    
    with open(test_manifest, 'w') as f:
        json.dump(data, f, indent=4)
        
    res = run_certifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res["status"] == "EQUIVALENCE_FAILED":
        for failure in res["failures"]:
            if failure["error"] == "EQUIVALENCE_MISMATCH":
                passed = True
                msg = "Correctly rejected equivalence due to mismatched outputs."
                break
                
    if not passed:
        msg = f"Failed to reject certifier mismatch. Res: {res}"
        
    print_result("Test 6: Equivalence Certifier Rejection", passed, msg)
    return passed

if __name__ == "__main__":
    print("--- Running Rejection Integrity Suite ---")
    
    results = [
        test_1_manifest_tampering_unsigned(),
        test_2_manifest_tampering_signed(),
        test_3_trace_file_corruption(),
        test_4_missing_artifact(),
        test_5_substrate_mutability_rejection(),
        test_6_equivalence_certifier_rejection()
    ]
    
    if os.path.exists(TMP_DIR):
        shutil.rmtree(TMP_DIR)
        
    if all(results):
        print("\n✅ All rejection tests passed. Rejection integrity confirmed.")
        sys.exit(0)
    else:
        print("\n❌ Some tests failed. Rejection integrity compromised.")
        sys.exit(1)
