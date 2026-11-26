import json
from collections import defaultdict
from pathlib import Path

inventory = Path("phase1/execution_inventory.jsonl")

# 1A-2A Collision Family Extraction
families = defaultdict(list)
with open(inventory) as f:
    for line in f:
        data = json.loads(line)
        families[data["replay_hash"]].append(data)

collision_families = {k: v for k, v in families.items() if len(v) > 1}

Path("phase1/collision_families.json").write_text(json.dumps(collision_families, indent=2))

# 1A-2B Composition Audit
md = "# 1A-2B Collision Family Composition Audit\n\n"
md += f"Total Collision Families: {len(collision_families)}\n\n"

for rh, members in sorted(collision_families.items(), key=lambda x: len(x[1]), reverse=True):
    substrates = set(m["substrate"] for m in members)
    topologies = set(m["topology"] for m in members)
    cognitions = set(m["cognition"] for m in members)
    
    md += f"### Family `{rh[:8]}...` ({len(members)} executions)\n"
    md += f"- **Substrates**: {', '.join(sorted(substrates))}\n"
    md += f"- **Topologies**: {', '.join(sorted(topologies))}\n"
    md += f"- **Cognitions**: {', '.join(sorted(cognitions))}\n\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a2b_composition_audit.md").write_text(md)
print(f"Extracted {len(collision_families)} collision families.")
