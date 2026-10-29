#!/usr/bin/env python3
"""Phase 3 Experiment 3A: Boundary Analysis.

Exploits the most replicated geometric object (the separation boundary) to see
if transitions occur near the boundary, uniformly, or deep inside ecologies.
"""
import json
import sys
import numpy as np
import pandas as pd
from scipy import stats

sys.path.insert(0, ".")
from ecology_utils import load_session_catalog, standardize, ward_clustering

def load_full(path):
    with open(path) as f:
        data = json.load(f)
    return pd.DataFrame(data)

def align_labels_by_centroid(X, labels, ref_centroids):
    from scipy.optimize import linear_sum_assignment
    centroids = np.array([X[labels == i].mean(axis=0) for i in range(len(np.unique(labels)))])
    n = len(ref_centroids)
    cost = np.zeros((n, n))
    for i in range(n):
        for j in range(n):
            cost[i, j] = np.linalg.norm(ref_centroids[i] - centroids[j])
    row_ind, col_ind = linear_sum_assignment(cost)
    mapping = {col: row for row, col in zip(row_ind, col_ind)}
    return np.array([mapping[l] for l in labels])

results = {}
for quarter, path in [("Q1", "phase1/analysis/coordinate_audit/session_catalog_q1.json"),
                       ("Q2", "phase1/analysis/coordinate_audit/session_catalog_q2.json")]:
    full_df = load_full(path)
    metric_df = load_session_catalog(path)
    metrics = ["realized_volatility", "trend_strength", "gap_pct", "session_range_pct", "net_return_pct"]
    full_df = full_df.dropna(subset=metrics).reset_index(drop=True)
    
    X_df, means, stds = standardize(metric_df)
    X = X_df.values
    labels = ward_clustering(X, n_clusters=2)
    centroids = np.array([X[labels == i].mean(axis=0) for i in range(2)])
    
    results[quarter] = {
        "full_df": full_df,
        "X": X,
        "labels": labels,
        "centroids": centroids,
    }

results["Q2"]["labels"] = align_labels_by_centroid(
    results["Q2"]["X"], results["Q2"]["labels"], results["Q1"]["centroids"]
)
results["Q2"]["centroids"] = np.array([
    results["Q2"]["X"][results["Q2"]["labels"] == i].mean(axis=0) for i in range(2)
])

# Compute signed and absolute distance to boundary
for q in ["Q1", "Q2"]:
    X = results[q]["X"]
    centroids = results[q]["centroids"]
    
    # We define Ecology A as 0, Ecology B as 1
    C_A = centroids[0]
    C_B = centroids[1]
    
    # Midpoint of the two centroids
    M = (C_A + C_B) / 2.0
    
    # Separation vector pointing from A to B
    w = C_B - C_A
    w_norm = np.linalg.norm(w)
    w_unit = w / w_norm
    
    # Signed distance to the hyperplane passing through M orthogonal to w
    # Positive means it's on the B side of the midpoint, negative means A side
    signed_dist = (X - M) @ w_unit
    abs_dist = np.abs(signed_dist)
    
    df = results[q]["full_df"]
    df["ecology"] = results[q]["labels"]
    df["signed_dist"] = signed_dist
    df["dist_to_boundary"] = abs_dist
    
    # Let's verify classification consistency with the geometric boundary
    # In Ward, cells are defined by distance to centroid. Because it's spherical, 
    # it perfectly matches the bisecting hyperplane.
    geom_labels = (signed_dist > 0).astype(int)
    mismatches = np.sum(geom_labels != results[q]["labels"])
    if mismatches > 0:
        print(f"[{q}] Warning: {mismatches} sessions mismatched between Ward labels and geometric hyperplane.")

# Pooling the data
pooled_df = pd.concat([results["Q1"]["full_df"], results["Q2"]["full_df"]], ignore_index=True)

print(f"{'='*70}\n  PHASE 3: BOUNDARY ANALYSIS\n{'='*70}")

for quarter, df in [("Q1", results["Q1"]["full_df"]), ("Q2", results["Q2"]["full_df"]), ("POOLED", pooled_df)]:
    print(f"\n\n{'#'*70}")
    print(f"  {quarter}")
    print(f"{'#'*70}")
    
    pairs = []
    for symbol in df["symbol"].unique():
        sym_df = df[df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        for i in range(len(sym_df) - 1):
            row_t = sym_df.iloc[i]
            row_t1 = sym_df.iloc[i+1]
            pairs.append({
                "ecology_t": row_t["ecology"],
                "dist_t": row_t["dist_to_boundary"],
                "signed_dist_t": row_t["signed_dist"],
                "ecology_t1": row_t1["ecology"],
                "transition": 1 if row_t["ecology"] != row_t1["ecology"] else 0
            })
    pair_df = pd.DataFrame(pairs)
    
    for eco, label, transition_name in [(0, "A (Quiet)", "A -> B"), (1, "B (Active)", "B -> A")]:
        eco_df = pair_df[pair_df["ecology_t"] == eco]
        n_total = len(eco_df)
        n_trans = eco_df["transition"].sum()
        p_trans = n_trans / n_total if n_total > 0 else 0
        
        print(f"\n  [Ecology {label}] Overall P({transition_name}) = {p_trans:.3f} ({n_trans}/{n_total})")
        
        if n_trans > 0 and n_total > n_trans:
            # Does distance correlate with transition? (Expect negative: closer = more likely)
            r, p = stats.pointbiserialr(eco_df["transition"], eco_df["dist_t"])
            print(f"    Point-biserial correlation (dist vs transition): r = {r:+.3f} (p = {p:.3f})")
            
            mean_dist_stay = eco_df[eco_df["transition"] == 0]["dist_t"].mean()
            mean_dist_trans = eco_df[eco_df["transition"] == 1]["dist_t"].mean()
            
            # Cohen's d between the two distributions
            d_stay = eco_df[eco_df["transition"] == 0]["dist_t"].values
            d_trans = eco_df[eco_df["transition"] == 1]["dist_t"].values
            pooled_std = np.sqrt(((len(d_stay)-1)*np.var(d_stay, ddof=1) + (len(d_trans)-1)*np.var(d_trans, ddof=1)) / (len(d_stay)+len(d_trans)-2))
            cohens_d = (mean_dist_trans - mean_dist_stay) / pooled_std
            
            print(f"    Mean distance to boundary:")
            print(f"      Sessions that STAYED      : {mean_dist_stay:.3f}")
            print(f"      Sessions that TRANSITIONED: {mean_dist_trans:.3f}")
            print(f"      Effect Size (Cohen's d)   : {cohens_d:+.3f}")
            
            # Quintile / Tercile analysis
            if n_total >= 30:
                print(f"    Transition probability by distance tier:")
                # Use terciles for Ecology B (smaller n) and quintiles for A
                num_bins = 5 if eco == 0 else 3
                eco_df = eco_df.copy()
                eco_df["dist_tier"] = pd.qcut(eco_df["dist_t"], q=num_bins, labels=False)
                
                tier_stats = []
                for tier in range(num_bins):
                    tier_data = eco_df[eco_df["dist_tier"] == tier]
                    n_tier = len(tier_data)
                    n_trans_tier = tier_data["transition"].sum()
                    p_tier = n_trans_tier / n_tier if n_tier > 0 else 0
                    tier_mean_dist = tier_data["dist_t"].mean()
                    tier_stats.append((tier, n_tier, n_trans_tier, p_tier, tier_mean_dist))
                
                for tier, n, nt, pt, md in tier_stats:
                    if tier == 0:
                        tier_desc = "Closest to boundary"
                    elif tier == num_bins - 1:
                        tier_desc = "Deepest in ecology"
                    else:
                        tier_desc = "Intermediate"
                    print(f"      Tier {tier} ({tier_desc:<19}): P(Trans) = {pt:.3f} ({nt:2d}/{n:2d}) | Mean Dist = {md:.3f}")

print("\n\nDone.")
