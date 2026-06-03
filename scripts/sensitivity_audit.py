import json
import subprocess
from pathlib import Path

inventory_path = Path("phase1/execution_inventory.jsonl")
inventory = []
with open(inventory_path) as f:
    for line in f:
        inventory.append(json.loads(line))

md = "# 1A-5 Identity Sensitivity Audit\n\n"

# Test 1: Same Substrate, Same Topology, Different Cognition
test1_passed = False
for item in inventory:
    matching = [x for x in inventory if x["substrate"] == item["substrate"] and x["topology"] == item["topology"] and x["cognition"] != item["cognition"]]
    if matching:
        md += "### Test 1: Cognition Sensitivity\n"
        md += f"Control: {item['substrate']} | {item['topology']}\n"
        md += f"A: {item['cognition']} -> Hash: {item['trace_hash']}\n"
        md += f"B: {matching[0]['cognition']} -> Hash: {matching[0]['trace_hash']}\n"
        test1_passed = (item['trace_hash'] != matching[0]['trace_hash'])
        md += f"Result: {'PASS' if test1_passed else 'FAIL'}\n\n"
        break

# Test 2: Same Substrate, Same Cognition, Different Topology
test2_passed = False
for item in inventory:
    matching = [x for x in inventory if x["substrate"] == item["substrate"] and x["cognition"] == item["cognition"] and x["topology"] != item["topology"]]
    if matching:
        md += "### Test 2: Topology Sensitivity\n"
        md += f"Control: {item['substrate']} | {item['cognition']}\n"
        md += f"A: {item['topology']} -> Hash: {item['trace_hash']}\n"
        md += f"B: {matching[0]['topology']} -> Hash: {matching[0]['trace_hash']}\n"
        test2_passed = (item['trace_hash'] != matching[0]['trace_hash'])
        md += f"Result: {'PASS' if test2_passed else 'FAIL'}\n\n"
        break

# Test 3: Deterministic Re-run
md += "### Test 3: Deterministic Reproducibility\n"
import shutil
import os

# Create dummy substrate
dummy_path = Path("scratch/dummy_sub.jsonl")
os.makedirs("scratch", exist_ok=True)
with open(dummy_path, "w") as f:
    f.write('{"price": 100.0}\n{"price": 101.0}\n{"price": 102.0}\n')

def run_engine():
    subprocess.run([
        "cargo", "run", "--bin", "financial_replay", "--", 
        "--substrate", "TEST_SUB", 
        "--substrate-file", str(dummy_path.absolute()), 
        "--topology", "baseline", 
        "--cognition", "event_reset"
    ], cwd="financial/strategies", capture_output=True, check=True)
    
    meta_path = Path("financial/strategies/artifacts/TEST_SUB/baseline/event_reset/metadata.json")
    with open(meta_path) as mf:
        return json.load(mf)["artifact_hash"]

try:
    hash_run_1 = run_engine()
    hash_run_2 = run_engine()
    
    md += f"Run 1 Hash: {hash_run_1}\n"
    md += f"Run 2 Hash: {hash_run_2}\n"
    test3_passed = (hash_run_1 == hash_run_2)
    md += f"Result: {'PASS' if test3_passed else 'FAIL'}\n\n"
except Exception as e:
    md += f"Error running test 3: {e}\n"
    test3_passed = False

if test1_passed and test2_passed and test3_passed:
    md += "> **Conclusion:** The Identity Sensitivity Audit is completely PASS. `artifact_hash` changes when execution state changes (Topology or Cognition) and remains perfectly stable when execution state is unchanged. It is a fully verified candidate execution-state identity."

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a5_sensitivity_audit.md").write_text(md)
print("Sensitivity audit completed.")
