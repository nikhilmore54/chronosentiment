import json
import statistics
import numpy as np
from pathlib import Path
from scipy.stats import spearmanr, pearsonr
from sklearn.preprocessing import StandardScaler

import warnings
warnings.filterwarnings('ignore')

out_dir = Path("phase1b2_75_artifacts")

signatures = []

dimensions = [
    "length", "mean_occ", "var_occ", "mean_over", "var_over", 
    "mean_acc", "var_acc", "mean_str", "var_str"
]

for event_dir in out_dir.iterdir():
    if not event_dir.is_dir(): continue
    for topo_dir in event_dir.iterdir():
        if not topo_dir.is_dir(): continue
        for cog_dir in topo_dir.iterdir():
            if not cog_dir.is_dir(): continue
            trace_path = cog_dir / "trace_v1.json"
            if not trace_path.exists(): continue
            
            with open(trace_path) as f:
                data = json.load(f)
                traces = data.get("traces", [])
                if traces:
                    occ = [t["occupancy"] for t in traces]
                    over = [t["overlap"] for t in traces]
                    acc = [t["acceptance_ratio"] for t in traces]
                    strc = [t["strictness_ratio"] for t in traces]
                    
                    sig = {
                        "substrate": event_dir.name,
                        "topology": topo_dir.name,
                        "cognition": cog_dir.name,
                        "length": len(traces),
                        "mean_occ": statistics.mean(occ),
                        "var_occ": statistics.pvariance(occ) if len(occ) > 1 else 0,
                        "mean_over": statistics.mean(over),
                        "var_over": statistics.pvariance(over) if len(over) > 1 else 0,
                        "mean_acc": statistics.mean(acc),
                        "var_acc": statistics.pvariance(acc) if len(acc) > 1 else 0,
                        "mean_str": statistics.mean(strc),
                        "var_str": statistics.pvariance(strc) if len(strc) > 1 else 0
                    }
                    signatures.append(sig)

# 1. Strict Intersection Filtering
registry = {}
for s in signatures:
    key = (s["substrate"], s["cognition"])
    if key not in registry:
        registry[key] = {}
    registry[key][s["topology"]] = s

valid_keys = [k for k, v in registry.items() if "tier1_5m" in v and "tier1_1m" in v]
N = len(valid_keys)

if N == 0:
    print("Error: No intersecting keys found.")
    exit(1)

data_5m = np.array([[registry[k]["tier1_5m"][d] for d in dimensions] for k in valid_keys])
data_1m = np.array([[registry[k]["tier1_1m"][d] for d in dimensions] for k in valid_keys])

scaler = StandardScaler()
data_5m = scaler.fit_transform(data_5m)
data_1m = scaler.fit_transform(data_1m)

def calc_dist_matrix(data):
    mat = np.zeros((N, N))
    for i in range(N):
        for j in range(N):
            mat[i, j] = np.linalg.norm(data[i] - data[j])
    return mat

dist_5m = calc_dist_matrix(data_5m)
dist_1m = calc_dist_matrix(data_1m)

# 3. Mantel Correlation (Global)
triu_indices = np.triu_indices(N, k=1)
vec_5m = dist_5m[triu_indices]
vec_1m = dist_1m[triu_indices]

mantel_corr, mantel_p = pearsonr(vec_5m, vec_1m)

# 4 & 5. Local Metrics
spearman_scores = []
top5_overlaps = []

for i in range(N):
    r, p = spearmanr(dist_5m[i], dist_1m[i])
    spearman_scores.append(r)
    
    nn_5m = set(np.argsort(dist_5m[i])[1:6])
    nn_1m = set(np.argsort(dist_1m[i])[1:6])
    top5_overlaps.append(len(nn_5m & nn_1m))

mean_spearman = np.mean(spearman_scores)
mean_overlap = np.mean(top5_overlaps)

# Expected random overlap for Top-K out of (N-1) items is K * (K / (N-1))
if N > 1:
    random_overlap = 5 * (5 / (N - 1))
else:
    random_overlap = 0

md = "# Phase 1B-2.75: Coordinate-System Audit\n\n"
md += f"**Intersection Filter:** {N} executions perfectly matched across `tier1_5m` and `tier1_1m` (identical substrate and cognition).\n\n"

md += "## Global Geometry Preservation\n"
md += f"- **Mantel Correlation (Pearson):** {mantel_corr:.4f} (p={mantel_p:.4e})\n\n"

md += "## Local Geometry Preservation\n"
md += f"- **Mean Spearman Rank Correlation:** {mean_spearman:.4f}\n"
md += f"- **Mean Top-5 Neighbour Overlap:** {mean_overlap:.2f} items\n"
md += f"- **Random Top-5 Overlap Baseline:** {random_overlap:.2f} items\n\n"

md += "## Interpretation\n"
if mean_spearman < 0.2 and mantel_corr < 0.2:
    md += "> **Result: DIFFERENT UNIVERSES.** The metrics indicate negligible geometric preservation. Topologies fundamentally rewrite the geometry space.\n"
elif mean_spearman > 0.7 and mantel_corr > 0.7:
    md += "> **Result: COORDINATE TRANSFORM.** High preservation detected. The topologies are distinct coordinate views of the same underlying geometry.\n"
else:
    md += "> **Result: MIXED/PARTIAL PRESERVATION.** The spaces retain some structure but are heavily deformed. Further investigation required.\n"

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1b2_75_coordinate_audit.md").write_text(md)
print(f"Stage 2.75 Audit Completed for N={N}. Mantel: {mantel_corr:.4f}, Spearman: {mean_spearman:.4f}")
