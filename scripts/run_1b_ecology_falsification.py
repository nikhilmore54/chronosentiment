import json
from pathlib import Path
import math
import statistics

sigs_path = Path("phase1/geometry_signatures.json")
with open(sigs_path) as f:
    signatures = json.load(f)

# The 9 dimensions
dimensions = [
    "length", "mean_occ", "var_occ", "mean_over", "var_over", 
    "mean_acc", "var_acc", "mean_str", "var_str"
]

# Standard scale (z-score) to prevent length from dominating Euclidean distance
scaled_sigs = []
for d in dimensions:
    vals = [s[d] for s in signatures]
    mean = statistics.mean(vals)
    stdev = statistics.pstdev(vals) if statistics.pstdev(vals) > 0 else 1.0
    for s in signatures:
        s[f"{d}_z"] = (s[d] - mean) / stdev

# Compute Pairwise Distances
for s1 in signatures:
    distances = []
    for s2 in signatures:
        if s1 == s2: continue
        dist = math.sqrt(sum((s1[f"{d}_z"] - s2[f"{d}_z"])**2 for d in dimensions))
        distances.append({"hash": s2["hash"], "dist": dist, "topology": s2["topology"], "cognition": s2["cognition"]})
    
    distances.sort(key=lambda x: x["dist"])
    s1["nn"] = distances[:3]

# Audit 1: Nearest Neighbour Structure
d1 = [s["nn"][0]["dist"] for s in signatures]
d2 = [s["nn"][1]["dist"] for s in signatures]
d3 = [s["nn"][2]["dist"] for s in signatures]

# Audit 2: Topology Dominance
topo_matches = sum(1 for s in signatures if s["topology"] == s["nn"][0]["topology"])
topo_rate = topo_matches / len(signatures)

# Audit 3: Cognition Separation
cog_matches = sum(1 for s in signatures if s["cognition"] == s["nn"][0]["cognition"])
cog_rate = cog_matches / len(signatures)

md = "# Phase 1B: Ecology Falsification Audit\n\n"

md += "### Audit 1: Nearest-Neighbour Structure\n"
md += "Measuring the distribution of Euclidean distances (Z-score standardized) to the 1st, 2nd, and 3rd nearest neighbours.\n\n"
md += f"- **1st NN Distance:** Mean: {statistics.mean(d1):.3f} | StdDev: {statistics.pstdev(d1):.3f} | Max: {max(d1):.3f}\n"
md += f"- **2nd NN Distance:** Mean: {statistics.mean(d2):.3f} | StdDev: {statistics.pstdev(d2):.3f} | Max: {max(d2):.3f}\n"
md += f"- **3rd NN Distance:** Mean: {statistics.mean(d3):.3f} | StdDev: {statistics.pstdev(d3):.3f} | Max: {max(d3):.3f}\n\n"

# Check for separation gap
gap_detected = statistics.mean(d1) < (statistics.mean(d3) * 0.5)
if gap_detected:
    md += "> **Result:** A significant separation gap exists between immediate neighbours and further neighbours. The space is structurally clumpy (ecological) rather than continuous.\n\n"
else:
    md += "> **Result:** Distances scale smoothly. The space appears continuous rather than ecological.\n\n"

md += "### Audit 2: Topology Dominance\n"
md += "Testing if the geometry space is essentially just a proxy for predefined topologies.\n\n"
md += f"- **Topology Match Rate (1st NN):** {topo_rate*100:.2f}%\n\n"
if topo_rate > 0.90:
    md += "> **Result:** Topology strongly dominates the geometry space. What looks like an ecology is highly likely to just be a topology cluster.\n\n"
else:
    md += "> **Result:** Topology does NOT strictly dominate. Geometries cross topology boundaries.\n\n"

md += "### Audit 3: Cognition Separation\n"
md += "Testing if cognition leaves a measurable footprint on the geometry signature.\n\n"
md += f"- **Cognition Match Rate (1st NN):** {cog_rate*100:.2f}%\n\n"
if cog_rate > 0.80:
    md += "> **Result:** Cognition leaves a strong, measurable footprint that causes executions with identical cognitions to cluster together.\n\n"
else:
    md += "> **Result:** Cognition does NOT strongly dictate geometric clustering.\n\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1b_ecology_falsification.md").write_text(md)
print("Ecology Falsification Audit completed.")
