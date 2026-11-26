import json
import math
from pathlib import Path
import statistics

with open("phase1/collision_families.json") as f:
    families = json.load(f)

def pearson(x, y):
    if len(x) < 2: return 0.0
    mean_x, mean_y = statistics.mean(x), statistics.mean(y)
    num = sum((a - mean_x) * (b - mean_y) for a, b in zip(x, y))
    den_x = sum((a - mean_x)**2 for a in x)
    den_y = sum((b - mean_y)**2 for b in y)
    if den_x == 0 or den_y == 0:
        return 1.0 if x == y else 0.0
    return num / math.sqrt(den_x * den_y)

def euclid(x, y):
    return math.sqrt(sum((a - b)**2 for a, b in zip(x, y)))

md = "# 1A-2G Cognition-Variant Distance Metrics\n\n"
md += "| Hash | Substrate | Topology | Ticks | Corr(Occ) | Corr(Over) | Corr(Acc) | Corr(Str) | Euclid(Total) |\n"
md += "| ---- | --------- | -------- | ----- | --------- | ---------- | --------- | --------- | ------------- |\n"

for rh, execs in families.items():
    # Group by (substrate, topology, tick_count)
    groups = {}
    for ex in execs:
        key = (ex["substrate"], ex["topology"], ex["tick_count"])
        if key not in groups: groups[key] = []
        groups[key].append(ex)
        
    for (sub, top, ticks), group in groups.items():
        if len(group) == 2 and group[0]["cognition"] != group[1]["cognition"] and ticks > 0:
            with open(group[0]["trace_path"]) as f: t1 = json.load(f).get("traces", [])
            with open(group[1]["trace_path"]) as f: t2 = json.load(f).get("traces", [])
            
            if len(t1) != ticks or len(t2) != ticks:
                continue
                
            occ1 = [t["occupancy"] for t in t1]; occ2 = [t["occupancy"] for t in t2]
            over1 = [t["overlap"] for t in t1]; over2 = [t["overlap"] for t in t2]
            acc1 = [t["acceptance_ratio"] for t in t1]; acc2 = [t["acceptance_ratio"] for t in t2]
            str1 = [t["strictness_ratio"] for t in t1]; str2 = [t["strictness_ratio"] for t in t2]
            
            c_occ = pearson(occ1, occ2)
            c_over = pearson(over1, over2)
            c_acc = pearson(acc1, acc2)
            c_str = pearson(str1, str2)
            
            e_total = math.sqrt(
                euclid(occ1, occ2)**2 + 
                euclid(over1, over2)**2 + 
                euclid(acc1, acc2)**2 + 
                euclid(str1, str2)**2
            )
            
            md += f"| `{rh[:8]}` | {sub[:15]}... | {top} | {ticks} | {c_occ:.3f} | {c_over:.3f} | {c_acc:.3f} | {c_str:.3f} | {e_total:.2f} |\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1a2g_distances.md").write_text(md)
print("Distances computed.")
