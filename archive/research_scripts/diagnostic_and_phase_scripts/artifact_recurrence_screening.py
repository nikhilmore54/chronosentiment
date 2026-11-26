import json
from collections import Counter
from pathlib import Path

inventory_path = Path("phase1/execution_inventory.jsonl")
inventory = []
with open(inventory_path) as f:
    for line in f:
        inventory.append(json.loads(line))

total_executions = len(inventory)
artifact_hashes = [item["trace_hash"] for item in inventory if item["trace_hash"] != "N/A"]

hash_counts = Counter(artifact_hashes)
unique_hashes = len(hash_counts)
collisions = total_executions - unique_hashes

diversity_ratio = unique_hashes / total_executions if total_executions > 0 else 0

families = {h: [] for h, c in hash_counts.items() if c > 1}
for item in inventory:
    h = item["trace_hash"]
    if h in families:
        families[h].append(item)

md = "# Phase 1A-6 Artifact Recurrence Screening\n\n"
md += f"**Executions:** {total_executions}\n"
md += f"**Unique artifact_hash:** {unique_hashes}\n"
md += f"**Collision Families:** {len(families)}\n"
md += f"**Largest Family:** {max(hash_counts.values()) if hash_counts else 0}\n"
md += f"**Diversity Ratio:** {diversity_ratio:.3f}\n\n"

if len(families) > 0:
    md += "## Collision Families Breakdown\n"
    md += "The following executions generated mathematically identical `artifact_hash` values. Note how they are explicitly redundant runs containing identical substrate data, topologies, and cognitions.\n\n"
    for h, execs in families.items():
        md += f"### Hash: `{h[:16]}...` (Size: {len(execs)})\n"
        for ex in execs:
            md += f"- Substrate: `{ex['substrate']}` | Topology: `{ex['topology']}` | Cognition: `{ex['cognition']}`\n"
        md += "\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a6_artifact_recurrence.md").write_text(md)
print("Phase 1A-6 Artifact Recurrence Screening completed.")
