import json
import numpy as np
from pathlib import Path
from collections import defaultdict
import hdbscan
from sklearn.cluster import DBSCAN, AgglomerativeClustering
from sklearn.preprocessing import StandardScaler
import warnings

warnings.filterwarnings('ignore')

sigs_path = Path("phase1/geometry_signatures.json")
with open(sigs_path) as f:
    signatures = json.load(f)

by_topology = defaultdict(list)
for s in signatures:
    by_topology[s["topology"]].append(s)

dimensions = [
    "length", "mean_occ", "var_occ", "mean_over", "var_over", 
    "mean_acc", "var_acc", "mean_str", "var_str"
]

def jaccard(c1, c2):
    s1, s2 = set(c1), set(c2)
    if not s1 or not s2: return 0.0
    return len(s1 & s2) / len(s1 | s2)

md = "# Phase 1B-3 Stage 1: Blind Discovery\n\n"
md += "Running HDBSCAN, DBSCAN, and Agglomerative clustering on the unmasked geometric signatures. Enforcing consensus across >= 2 families with Jaccard > 0.7.\n\n"

consensus_ecologies = defaultdict(list)

for topo in ["tier1_5m", "tier1_1m"]:
    sigs = by_topology[topo]
    if len(sigs) < 10: continue
    
    md += f"## Topology: `{topo}`\n"
    
    # Extract Real Data
    data = np.array([[s[d] for d in dimensions] for s in sigs])
    scaler = StandardScaler()
    data_scaled = scaler.fit_transform(data)
    
    # 1. HDBSCAN
    clusterer_h = hdbscan.HDBSCAN(min_cluster_size=4)
    labels_h = clusterer_h.fit_predict(data_scaled)
    clusters_h = [np.where(labels_h == c)[0].tolist() for c in set(labels_h) if c != -1]
    
    # 2. DBSCAN
    clusterer_d = DBSCAN(eps=1.5, min_samples=4)
    labels_d = clusterer_d.fit_predict(data_scaled)
    clusters_d = [np.where(labels_d == c)[0].tolist() for c in set(labels_d) if c != -1]
    
    # 3. Agglomerative
    clusterer_a = AgglomerativeClustering(n_clusters=None, distance_threshold=4.0)
    labels_a = clusterer_a.fit_predict(data_scaled)
    clusters_a = [np.where(labels_a == c)[0].tolist() for c in set(labels_a) if np.sum(labels_a == c) >= 4]
    
    md += f"- **HDBSCAN Discovered:** {len(clusters_h)} candidates\n"
    md += f"- **DBSCAN Discovered:** {len(clusters_d)} candidates\n"
    md += f"- **Agglomerative Discovered:** {len(clusters_a)} candidates\n"
    
    # Consensus Filtering
    all_clusters = [(c, 'H') for c in clusters_h] + [(c, 'D') for c in clusters_d] + [(c, 'A') for c in clusters_a]
    matched_indices = set()
    final_clusters = []
    
    for i, (c1, fam1) in enumerate(all_clusters):
        if i in matched_indices: continue
        
        matches = [c1]
        for j, (c2, fam2) in enumerate(all_clusters):
            if i != j and fam1 != fam2 and j not in matched_indices:
                if jaccard(c1, c2) > 0.7:
                    matches.append(c2)
                    matched_indices.add(j)
                    
        if len(matches) >= 2:
            # Union of the matching clusters to form the core ecology
            core_members = set()
            for m in matches: core_members.update(m)
            final_clusters.append(list(core_members))
            matched_indices.add(i)
            
    md += f"- **Consensus Ecologies (Regions):** {len(final_clusters)}\n\n"
    
    for idx, c in enumerate(final_clusters):
        md += f"### Region `{topo}_R{idx}` (Size: {len(c)})\n"
        member_hashes = [sigs[i]["hash"] for i in c]
        md += f"Members: {', '.join(member_hashes)}\n\n"
        
        # Save to global dictionary
        consensus_ecologies[f"{topo}_R{idx}"] = [sigs[i] for i in c]

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1b3_stage1_blind_discovery.md").write_text(md)

with open("phase1/discovered_ecologies.json", "w") as f:
    json.dump(consensus_ecologies, f, indent=2)

print("Stage 1 Blind Discovery completed.")
