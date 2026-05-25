import json
import os
import shutil
import subprocess
import sys
import time
import concurrent.futures

TMP_DIR = "core/artifacts/.tmp_robustness_tests"
CANONICAL_MANIFEST_PATH = "core/artifacts/phase2e_q/2026_nvda_sync_oscillatory_fragmentation/tier1_5m/manifest.json"
CANONICAL_DIR = os.path.dirname(CANONICAL_MANIFEST_PATH)

def run_verifier(manifest_path):
    try:
        res = subprocess.run(["python3", "scripts/verify_manifest_v1.py", manifest_path], capture_output=True, text=True)
        return json.loads(res.stdout)
    except subprocess.CalledProcessError as e:
        try:
            return json.loads(e.output)
        except:
            return {"status": "ERROR", "message": e.output}

def run_certifier(manifest_path):
    try:
        res = subprocess.run(["python3", "scripts/certify_equivalence_v1.py", manifest_path], capture_output=True, text=True)
        return json.loads(res.stdout)
    except subprocess.CalledProcessError as e:
        try:
            return json.loads(e.output)
        except:
            return {"status": "ERROR", "message": e.output}

def setup_test_env():
    if os.path.exists(TMP_DIR):
        shutil.rmtree(TMP_DIR)
    shutil.copytree(CANONICAL_DIR, TMP_DIR)
    return os.path.join(TMP_DIR, "manifest.json")

def print_result(name, passed, msg):
    status = "✅ PASS" if passed else "❌ FAIL"
    print(f"[{status}] {name}: {msg}")

def test_1_interrupted_replay_recovery():
    test_manifest = setup_test_env()
    
    # Simulate partial execution by truncating one of the trace_summary files 
    # to simulate a process killed halfway through writing.
    trace_path = os.path.join(TMP_DIR, "rolling_50", "trace_summary.json")
    with open(trace_path, 'r') as f:
        data_str = f.read()
        
    # Truncate halfway
    partial_str = data_str[:len(data_str)//2]
    
    with open(trace_path, 'w') as f:
        f.write(partial_str)
        
    # Verifier should fail gracefully with a JSON decode error or HASH_MISMATCH
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    if res and res.get("status") == "ATTESTATION_FAILED":
        for failure in res.get("failures", []):
            if failure.get("error") == "JSON_DECODE_ERROR" or failure.get("error") == "HASH_MISMATCH":
                passed = True
                msg = "Correctly rejected partially written/interrupted trace."
                break
                
    if not passed:
        msg = f"Failed to reject partial trace. Res: {res}"
        
    print_result("Test 1: Interrupted Replay Recovery", passed, msg)
    return passed

def test_2_parallel_replay_determinism():
    test_manifest = setup_test_env()
    
    # Run the certifier 4 times concurrently
    num_runs = 4
    results = []
    
    print("    Running 4 parallel equivalence certifications...")
    
    with concurrent.futures.ThreadPoolExecutor(max_workers=num_runs) as executor:
        futures = [executor.submit(run_certifier, test_manifest) for _ in range(num_runs)]
        for future in concurrent.futures.as_completed(futures):
            results.append(future.result())
            
    passed = True
    msg = ""
    first_replayed_id = None
    
    for i, res in enumerate(results):
        if not res or res.get("status") != "EQUIVALENCE_CERTIFIED":
            passed = False
            msg = f"Run {i} failed equivalence certification: {res}"
            break
            
        replayed_id = res.get("replayed_manifest_id")
        if first_replayed_id is None:
            first_replayed_id = replayed_id
        elif replayed_id != first_replayed_id:
            passed = False
            msg = f"Race condition detected! Divergent manifest_ids: {first_replayed_id} vs {replayed_id}"
            break
            
    if passed:
        msg = "All 4 parallel certifier runs achieved exact determinism without race conditions."
        
    print_result("Test 2: Parallel Replay Determinism", passed, msg)
    return passed

def test_3_serialization_perturbation():
    test_manifest = setup_test_env()
    
    with open(test_manifest, 'r') as f:
        data = json.load(f)
        
    # We will perturb the manifest by dumping it with a completely different layout
    # (no sorting, random key order by nature of dict iteration, large indent).
    with open(test_manifest, 'w') as f:
        json.dump(data, f, indent=8, sort_keys=False, separators=(', ', ': '))
        
    # Add some raw whitespace padding to the end of the file
    with open(test_manifest, 'a') as f:
        f.write("\n\n\n    \n")
        
    res = run_verifier(test_manifest)
    
    passed = False
    msg = ""
    
    if res and res.get("status") == "ATTESTATION_PASSED":
        passed = True
        msg = "Verifier correctly canonicalized and verified heavily perturbed serialization format."
    else:
        msg = f"Verifier failed to handle serialization perturbation. Res: {res}"
        
    print_result("Test 3: Serialization Perturbation", passed, msg)
    return passed

if __name__ == "__main__":
    print("--- Running Robustness Engineering Suite ---")
    
    results = [
        test_1_interrupted_replay_recovery(),
        test_2_parallel_replay_determinism(),
        test_3_serialization_perturbation()
    ]
    
    if os.path.exists(TMP_DIR):
        shutil.rmtree(TMP_DIR)
        
    if all(results):
        print("\n✅ All robustness tests passed. System is environmentally stable.")
        sys.exit(0)
    else:
        print("\n❌ Some tests failed. System contains environmental fragility.")
        sys.exit(1)
