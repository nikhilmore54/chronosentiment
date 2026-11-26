import json
import statistics
from pathlib import Path

families_path = Path("phase1/collision_families.json")
if not families_path.exists():
    print("No families")
    exit(1)

with open(families_path) as f:
    families = json.load(f)

summary = {}

for rh, executions in families.items():
    tick_counts = []
    mean_occupancies = []
    mean_overlaps = []
    mean_acceptances = []
    mean_strictnesses = []
    
    for ex in executions:
        trace_path = Path(ex["trace_path"])
        if not trace_path.exists():
            continue
            
        with open(trace_path) as tf:
            try:
                trace_data = json.load(tf)
            except json.JSONDecodeError:
                continue
            
        ticks = trace_data.get("traces", [])
        tick_counts.append(len(ticks))
        
        if len(ticks) > 0:
            occ = [t["occupancy"] for t in ticks]
            ov = [t["overlap"] for t in ticks]
            acc = [t["acceptance_ratio"] for t in ticks]
            strc = [t["strictness_ratio"] for t in ticks]
            
            mean_occupancies.append(statistics.mean(occ))
            mean_overlaps.append(statistics.mean(ov))
            mean_acceptances.append(statistics.mean(acc))
            mean_strictnesses.append(statistics.mean(strc))
            
    if not tick_counts:
        continue
        
    def var(lst):
        if len(lst) < 2: return 0.0
        return statistics.variance(lst)
        
    summary[rh] = {
        "execution_count": len(executions),
        "tick_count_variance": var(tick_counts),
        "occupancy_mean_variance": var(mean_occupancies),
        "overlap_mean_variance": var(mean_overlaps),
        "acceptance_mean_variance": var(mean_acceptances),
        "strictness_mean_variance": var(mean_strictnesses),
    }

Path("phase1/family_summary.json").write_text(json.dumps(summary, indent=2))

md = "# 1A-2C Family Characterization Summary\n\n"
md += "| Family | Execs | Tick Count Var | Occupancy Var | Overlap Var | Accept Var | Strict Var |\n"
md += "| ------ | ----- | -------------- | ------------- | ----------- | ---------- | ---------- |\n"

for rh, s in sorted(summary.items(), key=lambda x: x[1]["execution_count"], reverse=True):
    md += f"| `{rh[:8]}...` | {s['execution_count']} | {s['tick_count_variance']:.2f} | {s['occupancy_mean_variance']:.6f} | {s['overlap_mean_variance']:.6f} | {s['acceptance_mean_variance']:.6f} | {s['strictness_mean_variance']:.6f} |\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a2c_characterization.md").write_text(md)
print("Characterization complete.")
