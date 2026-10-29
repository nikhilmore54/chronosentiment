import json
from pathlib import Path
import math
import statistics
from collections import defaultdict

sigs_path = Path("phase1/geometry_signatures.json")
with open(sigs_path) as f:
    signatures = json.load(f)

# Group by topology
by_topology = defaultdict(list)
for s in signatures:
    by_topology[s["topology"]].append(s)

dimensions = [
    "length", "mean_occ", "var_occ", "mean_over", "var_over", 
    "mean_acc", "var_acc", "mean_str", "var_str"
]

md = "# Phase 1B-2: Within-Topology Audits\n\n"
md += "Testing whether meaningful geometric structure and cognition footprints survive when topology is held strictly constant.\n\n"

for topo, sigs in by_topology.items():
    if len(sigs) < 4:
        continue # Skip topologies without enough executions for 3rd NN
        
    md += f"## Topology: `{topo}` (Executions: {len(sigs)})\n\n"
    
    # Standard scale within topology
    scaled_sigs = []
    for d in dimensions:
        vals = [s[d] for s in sigs]
        mean = statistics.mean(vals)
        stdev = statistics.pstdev(vals) if statistics.pstdev(vals) > 0 else 1.0
        for s in sigs:
            s[f"{d}_z"] = (s[d] - mean) / stdev if stdev > 0 else 0.0

    # Compute Pairwise Distances
    for s1 in sigs:
        distances = []
        for s2 in sigs:
            if s1 == s2: continue
            dist = math.sqrt(sum((s1[f"{d}_z"] - s2[f"{d}_z"])**2 for d in dimensions))
            distances.append({"hash": s2["hash"], "dist": dist, "cognition": s2["cognition"]})
        
        distances.sort(key=lambda x: x["dist"])
        s1["nn"] = distances[:3]

    # Audit 1B-2A: Nearest Neighbour Structure
    d1 = [s["nn"][0]["dist"] for s in sigs]
    d2 = [s["nn"][1]["dist"] for s in sigs]
    d3 = [s["nn"][2]["dist"] for s in sigs]
    
    mean_d1 = statistics.mean(d1)
    mean_d3 = statistics.mean(d3)
    gap_detected = mean_d1 < (mean_d3 * 0.5)

    md += "### Phase 1B-2A: Ecology Audit (Structure)\n"
    md += f"- **1st NN Distance (Mean):** {mean_d1:.3f}\n"
    md += f"- **3rd NN Distance (Mean):** {mean_d3:.3f}\n"
    md += f"- **Gap Detected:** {'Yes (Structured/Clumpy)' if gap_detected else 'No (Smooth/Continuous)'}\n\n"

    # Audit 1B-2B: Cognition Separation
    cog_matches = sum(1 for s in sigs if s["cognition"] == s["nn"][0]["cognition"])
    cog_rate = cog_matches / len(sigs)

    md += "### Phase 1B-2B: Cognition Audit (Footprint)\n"
    md += f"- **Cognition Match Rate (1st NN):** {cog_rate*100:.2f}%\n"
    
    if cog_rate > 0.80:
        md += "- **Result:** Cognition defines a strong geometry layer within this topology.\n\n"
    elif cog_rate > 0.50:
        md += "- **Result:** Cognition has a measurable but non-dominant footprint.\n\n"
    else:
        md += "- **Result:** Cognition footprint collapses; geometry is driven by something else.\n\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1b2_within_topology_audit.md").write_text(md)
print("Within-Topology Audits completed.")
