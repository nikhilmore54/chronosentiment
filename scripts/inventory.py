import os
import json
from pathlib import Path

base_dir = Path("infrastructure/core/artifacts")

results = []
substrates = set()
topologies = set()
cognitions = set()

for hash_file in base_dir.rglob("replay_hash.txt"):
    parts = hash_file.parts
    cognition = parts[-2]
    topology = parts[-3]
    substrate = parts[-4]
    
    replay_hash = hash_file.read_text().strip()
    
    trace_file = hash_file.parent / "trace_v1.json"
    tick_count = "N/A"
    if trace_file.exists():
        with open(trace_file, 'r') as f:
            for i in range(15):
                line = f.readline()
                if '"total_ticks"' in line:
                    tick_count = line.split(':')[1].strip().strip(',')
                    break
                    
    substrates.add(substrate)
    topologies.add(topology)
    cognitions.add(cognition)
    
    results.append({
        "substrate": substrate,
        "topology": topology,
        "cognition": cognition,
        "hash": replay_hash[:8] + "...",
        "trace": str(trace_file),
        "ticks": tick_count
    })

results.sort(key=lambda x: (x["substrate"], x["topology"], x["cognition"]))

md = "# Phase 1A-0 Data Sufficiency Audit Inventory\n\n"
md += f"- **Total Replay Executions**: {len(results)}\n"
md += f"- **Distinct Substrates/Events**: {len(substrates)}\n"
md += f"- **Distinct Topologies**: {len(topologies)}\n"
md += f"- **Distinct Cognitions**: {len(cognitions)}\n\n"

md += "| Substrate | Topology | Cognition | Replay Hash | Trace Path | Tick Count |\n"
md += "| --------- | -------- | --------- | ----------- | ---------- | ---------- |\n"

for r in results:
    md += f"| {r['substrate']} | {r['topology']} | {r['cognition']} | {r['hash']} | `{r['trace']}` | {r['ticks']} |\n"

out_path = "/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1A0_inventory.md"
with open(out_path, "w") as f:
    f.write(md)

print(f"Inventory completed. Found {len(results)} executions.")
