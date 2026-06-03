import json
from pathlib import Path

with open("phase1/collision_families.json") as f:
    families = json.load(f)

md = "# 1A-2D Cross-Corpus Invariance Audit\n\n"
md += "| Replay Hash | Family Size | Topologies | Cognitions | Substrates |\n"
md += "| ----------- | ----------- | ---------- | ---------- | ---------- |\n"

multi_cog_count = 0
multi_top_count = 0
multi_sub_count = 0
total_families = len(families)

for rh, execs in sorted(families.items(), key=lambda x: len(x[1]), reverse=True):
    tops = set(ex["topology"] for ex in execs)
    cogs = set(ex["cognition"] for ex in execs)
    subs = set(ex["substrate"] for ex in execs)
    
    if len(cogs) > 1: multi_cog_count += 1
    if len(tops) > 1: multi_top_count += 1
    if len(subs) > 1: multi_sub_count += 1
    
    md += f"| `{rh[:8]}...` | {len(execs)} | {len(tops)} | {len(cogs)} | {len(subs)} |\n"

md += f"\n### Summary\n"
md += f"- **Total Collision Families**: {total_families}\n"
md += f"- **Families spanning multiple Cognitions**: {multi_cog_count}/{total_families} ({(multi_cog_count/total_families)*100:.2f}%)\n"
md += f"- **Families spanning multiple Topologies**: {multi_top_count}/{total_families} ({(multi_top_count/total_families)*100:.2f}%)\n"
md += f"- **Families spanning multiple Substrates**: {multi_sub_count}/{total_families} ({(multi_sub_count/total_families)*100:.2f}%)\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a2d_invariance_audit.md").write_text(md)
print("Invariance audit generated.")
