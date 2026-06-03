#!/usr/bin/env python3
"""Phase 2B Duration Control Test.

Test: vol(t+1) ~ vol(t) + ecology + run_length
To determine if run_length adds dynamical memory information beyond current volatility.
"""
import json
import sys
import numpy as np
import pandas as pd
import statsmodels.api as sm

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

# Process both quarters
for q in ["Q1", "Q2"]:
    df = results[q]["full_df"]
    df["ecology"] = results[q]["labels"]
    
    run_lengths = []
    for symbol in df["symbol"].unique():
        sym_mask = df["symbol"] == symbol
        ecologies = df.loc[sym_mask, "ecology"].values
        rl = np.zeros(len(ecologies), dtype=int)
        
        current_eco = ecologies[0]
        current_len = 1
        rl[0] = 1
        for i in range(1, len(ecologies)):
            if ecologies[i] == current_eco:
                current_len += 1
            else:
                current_eco = ecologies[i]
                current_len = 1
            rl[i] = current_len
        df.loc[sym_mask, "run_length"] = rl

pooled_df = pd.concat([results["Q1"]["full_df"], results["Q2"]["full_df"]], ignore_index=True)

pairs = []
for symbol in pooled_df["symbol"].unique():
    sym_df = pooled_df[pooled_df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
    for i in range(len(sym_df) - 1):
        row_t = sym_df.iloc[i]
        row_t1 = sym_df.iloc[i+1]
        pairs.append({
            "vol_t": row_t["realized_volatility"],
            "ecology_t": row_t["ecology"],
            "run_length_t": row_t["run_length"],
            "vol_t1": row_t1["realized_volatility"],
        })
pair_df = pd.DataFrame(pairs)

# We want to test this within Ecology A specifically, since that's where the duration effect was observed
ecoA_pairs = pair_df[pair_df["ecology_t"] == 0].copy()

# Regression 1: Overall pooled
print("\n=== Pooled Regression (All States) ===")
X = pair_df[["vol_t", "ecology_t", "run_length_t"]]
X = sm.add_constant(X)
y = pair_df["vol_t1"]
model = sm.OLS(y, X).fit()
print(model.summary())

# Regression 2: Within Ecology A only
print("\n=== Ecology A Regression (Quiet State) ===")
# Here we just need vol(t) and run_length(t)
X_A = ecoA_pairs[["vol_t", "run_length_t"]]
X_A = sm.add_constant(X_A)
y_A = ecoA_pairs["vol_t1"]
model_A = sm.OLS(y_A, X_A).fit()
print(model_A.summary())

# Test 3: Log-transform run_length in Ecology A, since effects often decay logarithmically
print("\n=== Ecology A Regression (Log Run Length) ===")
ecoA_pairs["log_run_length"] = np.log(ecoA_pairs["run_length_t"])
X_A_log = ecoA_pairs[["vol_t", "log_run_length"]]
X_A_log = sm.add_constant(X_A_log)
model_A_log = sm.OLS(y_A, X_A_log).fit()
print(model_A_log.summary())
