#!/usr/bin/env python3
"""Phase 3 Experiment 3D: Outlier Event Ledger.

Ranks all A->B excursion entries by intensity (displacement from B centroid)
and extracts the Top 15 Extreme Excursions and 15 Median Excursions.
"""
import json
import sys
import numpy as np
import pandas as pd

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

entries = []
for q in ["Q1", "Q2"]:
    X = results[q]["X"]
    centroids = results[q]["centroids"]
    C_B = centroids[1]
    
    df = results[q]["full_df"]
    df["ecology"] = results[q]["labels"]
    
    for symbol in df["symbol"].unique():
        sym_mask = df["symbol"] == symbol
        sym_df = df[sym_mask].sort_values("date").reset_index(drop=False)
        idx_array = sym_df["index"].values
        
        for i in range(len(sym_df) - 1):
            row_t = sym_df.iloc[i]
            row_t1 = sym_df.iloc[i+1]
            
            # A -> B transition
            if row_t["ecology"] == 0 and row_t1["ecology"] == 1:
                idx_t1 = idx_array[i+1]
                coord_t1 = X[idx_t1]
                
                # Distance from B centroid
                dist_B = np.linalg.norm(coord_t1 - C_B)
                
                # Distance from global mean (which is 0 since we standardized, but we standardized per quarter.
                # So the global mean is exactly the origin in this space).
                dist_global = np.linalg.norm(coord_t1)
                
                entries.append({
                    "quarter": q,
                    "date": row_t1["date"],
                    "symbol": row_t1["symbol"],
                    "volatility": row_t1["realized_volatility"],
                    "range": row_t1["session_range_pct"],
                    "trend": row_t1["trend_strength"],
                    "return": row_t1["net_return_pct"],
                    "gap": row_t1["gap_pct"],
                    "dist_B_centroid": dist_B,
                    "dist_global_mean": dist_global
                })

entries_df = pd.DataFrame(entries)

# We want extreme B sessions. These are sessions with the HIGHEST distance from the global mean
# (they are the farthest out outliers in the excursion space). 
# We'll sort by dist_global_mean descending.
entries_df = entries_df.sort_values("dist_global_mean", ascending=False).reset_index(drop=True)

print(f"{'='*80}\n  PHASE 3: OUTLIER EVENT LEDGER (A -> B ENTRIES)\n{'='*80}")
print(f"Total A->B entries found: {len(entries_df)}\n")

print(f"{'#'*80}")
print(f"  LEDGER A: EXTREME EXCURSIONS (Top 15 farthest from global mean)")
print(f"{'#'*80}")
top_15 = entries_df.head(15)
print(f"{'Date':<12} | {'Sym':<9} | {'Dist(G)':>7} | {'Vol':>8} | {'Range%':>7} | {'Trend':>7} | {'Ret%':>7}")
print("-" * 80)
for _, row in top_15.iterrows():
    print(f"{row['date']:<12} | {row['symbol']:<9} | {row['dist_global_mean']:7.2f} | {row['volatility']:8.5f} | {row['range']:7.2f} | {row['trend']:7.2f} | {row['return']:+7.2f}")

print(f"\n\n{'#'*80}")
print(f"  LEDGER B: ORDINARY EXCURSIONS (15 closest to median distance)")
print(f"{'#'*80}")
median_dist = entries_df["dist_global_mean"].median()
entries_df["dist_from_median"] = np.abs(entries_df["dist_global_mean"] - median_dist)
median_15 = entries_df.sort_values("dist_from_median").head(15).sort_values("dist_global_mean", ascending=False)
print(f"Median global distance: {median_dist:.2f}\n")
print(f"{'Date':<12} | {'Sym':<9} | {'Dist(G)':>7} | {'Vol':>8} | {'Range%':>7} | {'Trend':>7} | {'Ret%':>7}")
print("-" * 80)
for _, row in median_15.iterrows():
    print(f"{row['date']:<12} | {row['symbol']:<9} | {row['dist_global_mean']:7.2f} | {row['volatility']:8.5f} | {row['range']:7.2f} | {row['trend']:7.2f} | {row['return']:+7.2f}")

print("\nDone.")
