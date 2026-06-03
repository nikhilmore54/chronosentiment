import json
from pathlib import Path

with open("phase1/collision_families.json") as f:
    families = json.load(f)

md = "# 1A-2F Corpus-Wide Length Audit\n\n"

lengths = set()
collisions_across_length = 0
total_families = len(families)

for rh, execs in families.items():
    family_lengths = set(ex["tick_count"] for ex in execs)
    lengths.update(family_lengths)
    if len(family_lengths) > 1:
        collisions_across_length += 1

md += f"- **Total Collision Families**: {total_families}\n"
md += f"- **Families spanning multiple lengths**: {collisions_across_length}\n"
md += f"- **Distinct lengths in collision families**: {sorted(list(lengths))}\n\n"

if collisions_across_length == 0:
    md += "> **Conclusion**: 100% of all collision families require EXACT length equivalence. There is not a single instance of a replay identity collision between traces of different lengths. Length is a hard constraint on the macroscopic identity.\n"
else:
    md += f"> **Conclusion**: {collisions_across_length} families span multiple lengths. Length is NOT a strict invariant for the replay identity.\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a2f_length_audit.md").write_text(md)
print("Length audit completed.")
