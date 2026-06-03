#!/usr/bin/env python3
"""Phase 3 Experiments 3B and 3C: Genesis and Decay Characterization.

Genesis (3B): Compare A->A vs A->B on all metrics.
Decay (3C): Compare B->B vs B->A on all metrics.
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

def print_comparison(name, x_stay, x_trans):
    n_s = len(x_stay)
    n_t = len(x_trans)
    if n_s < 2 or n_t < 2:
        return
    mean_s = x_stay.mean()
    mean_t = x_trans.mean()
    std_s = x_stay.std(ddof=1)
    std_t = x_trans.std(ddof=1)
    
    pooled_std = np.sqrt(((n_s - 1) * std_s**2 + (n_t - 1) * std_t**2) / (n_s + n_t - 2))
    d = (mean_t - mean_s) / pooled_std if pooled_std > 0 else 0
    t_stat, p_val = stats.ttest_ind(x_stay, x_trans, equal_var=False)
    
    print(f"    {name:<25}: STAY={mean_s:>8.4f} | TRANS={mean_t:>8.4f} | d={d:>+6.3f} | p={p_val:.3f}")

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

# Pre-compute distances
for q in ["Q1", "Q2"]:
    X = results[q]["X"]
    centroids = results[q]["centroids"]
    C_A = centroids[0]
    C_B = centroids[1]
    M = (C_A + C_B) / 2.0
    w = C_B - C_A
    w_unit = w / np.linalg.norm(w)
    
    dist_bound = np.abs((X - M) @ w_unit)
    dist_A = np.linalg.norm(X - C_A, axis=1)
    dist_B = np.linalg.norm(X - C_B, axis=1)
    
    df = results[q]["full_df"]
    df["ecology"] = results[q]["labels"]
    df["dist_boundary"] = dist_bound
    df["dist_A_centroid"] = dist_A
    df["dist_B_centroid"] = dist_B

pooled_df = pd.concat([results["Q1"]["full_df"], results["Q2"]["full_df"]], ignore_index=True)

print(f"{'='*70}\n  PHASE 3: GENESIS AND DECAY CHARACTERIZATION\n{'='*70}")

metrics_to_test = ["realized_volatility", "trend_strength", "gap_pct", "session_range_pct", "net_return_pct"]

for quarter, df in [("Q1", results["Q1"]["full_df"]), ("Q2", results["Q2"]["full_df"]), ("POOLED", pooled_df)]:
    print(f"\n\n{'#'*70}")
    print(f"  {quarter}")
    print(f"{'#'*70}")
    
    pairs = []
    for symbol in df["symbol"].unique():
        sym_df = df[df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        for i in range(len(sym_df) - 1):
            row_t = sym_df.iloc[i].to_dict()
            row_t["ecology_t1"] = sym_df.iloc[i+1]["ecology"]
            row_t["transition"] = 1 if row_t["ecology"] != row_t["ecology_t1"] else 0
            pairs.append(row_t)
            
    pair_df = pd.DataFrame(pairs)
    
    # ------------------------------------------------------------
    # GENESIS: A -> A vs A -> B
    # ------------------------------------------------------------
    print(f"\n  [GENESIS] Ecology A (Quiet)")
    ecoA_df = pair_df[pair_df["ecology"] == 0]
    n_stay_A = len(ecoA_df[ecoA_df["transition"] == 0])
    n_trans_A = len(ecoA_df[ecoA_df["transition"] == 1])
    print(f"    n(A->A) = {n_stay_A}, n(A->B) = {n_trans_A}")
    
    if n_stay_A >= 2 and n_trans_A >= 2:
        for m in metrics_to_test + ["dist_boundary", "dist_A_centroid"]:
            val_stay = ecoA_df[ecoA_df["transition"] == 0][m].values
            val_trans = ecoA_df[ecoA_df["transition"] == 1][m].values
            print_comparison(m, val_stay, val_trans)

    # ------------------------------------------------------------
    # DECAY: B -> B vs B -> A
    # ------------------------------------------------------------
    print(f"\n  [DECAY] Ecology B (Active)")
    ecoB_df = pair_df[pair_df["ecology"] == 1]
    n_stay_B = len(ecoB_df[ecoB_df["transition"] == 0]) # B->B
    n_trans_B = len(ecoB_df[ecoB_df["transition"] == 1]) # B->A
    print(f"    n(B->B) = {n_stay_B}, n(B->A) = {n_trans_B}")
    
    if n_stay_B >= 2 and n_trans_B >= 2:
        for m in metrics_to_test + ["dist_boundary", "dist_B_centroid"]:
            val_stay = ecoB_df[ecoB_df["transition"] == 0][m].values
            val_trans = ecoB_df[ecoB_df["transition"] == 1][m].values
            print_comparison(m, val_stay, val_trans)

print("\n\nDone.")
