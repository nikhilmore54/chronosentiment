import json
from pathlib import Path

families_path = Path("phase1/collision_families.json")
with open(families_path) as f:
    families = json.load(f)

# Find the 11-member family (ebc55689...)
btc_family = None
for rh, execs in families.items():
    if len(execs) == 11 and execs[0]["substrate"] == "BTCUSDT":
        btc_family = execs
        break

md = "# Deep Dive: BTCUSDT Family `ebc55689`\n\n"
md += "| Execution (Trace Hash) | Topology | Cognition |\n"
md += "| ---------------------- | -------- | --------- |\n"

for ex in sorted(btc_family, key=lambda x: (x["topology"], x["cognition"])):
    md += f"| `{ex['trace_hash'][:8]}...` | {ex['topology']} | {ex['cognition']} |\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/btcusdt_deepdive.md").write_text(md)
print("BTCUSDT table generated.")
