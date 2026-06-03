import json
from pathlib import Path
import math
import statistics
import random
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

md = "# Phase 1B-2C: Label Permutation Audit\n\n"
md += "Testing whether cognition alignment in nearest neighbours is a genuine geometric footprint or just a random sampling artifact.\n\n"

for topo, sigs in by_topology.items():
    if len(sigs) < 4:
        continue
        
    md += f"## Topology: `{topo}`\n"
    
    # Standard scale
    for d in dimensions:
        vals = [s[d] for s in sigs]
        mean = statistics.mean(vals)
        stdev = statistics.pstdev(vals) if statistics.pstdev(vals) > 0 else 1.0
        for s in sigs:
            s[f"{d}_z"] = (s[d] - mean) / stdev if stdev > 0 else 0.0

    # True NN Distances and Match
    true_matches = 0
    nn_indices = []
    
    for i, s1 in enumerate(sigs):
        distances = []
        for j, s2 in enumerate(sigs):
            if i == j: continue
            dist = math.sqrt(sum((s1[f"{d}_z"] - s2[f"{d}_z"])**2 for d in dimensions))
            distances.append((dist, j))
        
        distances.sort(key=lambda x: x[0])
        nn_idx = distances[0][1]
        nn_indices.append(nn_idx)
        
        if s1["cognition"] == sigs[nn_idx]["cognition"]:
            true_matches += 1
            
    true_rate = true_matches / len(sigs)
    
    # Permutation Test
    original_cogs = [s["cognition"] for s in sigs]
    perm_rates = []
    
    iterations = 1000
    for _ in range(iterations):
        shuffled = original_cogs.copy()
        random.shuffle(shuffled)
        
        p_match = 0
        for i in range(len(sigs)):
            if shuffled[i] == shuffled[nn_indices[i]]:
                p_match += 1
        perm_rates.append(p_match / len(sigs))
        
    mean_perm = statistics.mean(perm_rates)
    p_value = sum(1 for r in perm_rates if r >= true_rate) / iterations
    
    md += f"- **Observed NN Cognition Match:** {true_rate*100:.2f}%\n"
    md += f"- **Random Permutation Baseline:** {mean_perm*100:.2f}%\n"
    md += f"- **p-value (1000 iterations):** {p_value:.4f}\n\n"
    
    if p_value < 0.05:
        md += "> **Result:** STATISTICALLY SIGNIFICANT. The cognition footprint is genuinely encoded in the geometry.\n\n"
    else:
        md += "> **Result:** NOT SIGNIFICANT. The cognition alignment may be a random composition artifact.\n\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1b2c_permutation_audit.md").write_text(md)
print("Label Permutation Audit completed.")
