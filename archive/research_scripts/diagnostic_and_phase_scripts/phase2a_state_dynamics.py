#!/usr/bin/env python3
"""Phase 2A State Dynamics Analysis.

Experiments:
1. Attractor vs Excursion Dynamics (Run-length → Transition Probability)
2A. State → Next-Day Intensity (Volatility & Range)
2B. State Persistence → Future Intensity
3. Descriptive Survival Analysis
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
    """Align labels to reference centroids."""
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

def print_categorical_comparison(name, x_A, x_B):
    """Print effect sizes and p-values for a categorical comparison."""
    n_A = len(x_A)
    n_B = len(x_B)
    if n_A < 2 or n_B < 2:
        return
    mean_A = x_A.mean()
    mean_B = x_B.mean()
    std_A = x_A.std(ddof=1)
    std_B = x_B.std(ddof=1)
    
    pooled_std = np.sqrt(((n_A - 1) * std_A**2 + (n_B - 1) * std_B**2) / (n_A + n_B - 2))
    d = (mean_A - mean_B) / pooled_std if pooled_std > 0 else 0
    t_stat, p_val = stats.ttest_ind(x_A, x_B, equal_var=False)
    
    print(f"    {name}:")
    print(f"      n     : A={n_A}, B={n_B}")
    print(f"      mean  : A={mean_A:.5f}, B={mean_B:.5f}")
    print(f"      std   : A={std_A:.5f}, B={std_B:.5f}")
    print(f"      Cohen d: {d:+.4f}")
    print(f"      p-val  : {p_val:.4f}")

# ============================================================
# Load both quarters
# ============================================================
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

# Align Q2 to Q1
results["Q2"]["labels"] = align_labels_by_centroid(
    results["Q2"]["X"], results["Q2"]["labels"], results["Q1"]["centroids"]
)
results["Q2"]["centroids"] = np.array([
    results["Q2"]["X"][results["Q2"]["labels"] == i].mean(axis=0) for i in range(2)
])

# Separation vectors
q1_sep = results["Q1"]["centroids"][0] - results["Q1"]["centroids"][1]
q2_sep = results["Q2"]["centroids"][0] - results["Q2"]["centroids"][1]
avg_sep = (q1_sep / np.linalg.norm(q1_sep) + q2_sep / np.linalg.norm(q2_sep)) / 2
avg_sep = avg_sep / np.linalg.norm(avg_sep)

# Attach eco_position and run lengths
for q in ["Q1", "Q2"]:
    r = results[q]
    df = r["full_df"]
    df["ecology"] = r["labels"]
    df["eco_position"] = r["X"] @ avg_sep
    
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

print(f"{'='*70}\n  PHASE 2A: STATE DYNAMICS\n{'='*70}")

# Pool the data for robust statistics (optional, doing per-quarter and pooled)
pooled_df = pd.concat([results["Q1"]["full_df"], results["Q2"]["full_df"]], ignore_index=True)

for quarter, df in [("Q1", results["Q1"]["full_df"]), ("Q2", results["Q2"]["full_df"]), ("POOLED", pooled_df)]:
    print(f"\n\n{'#'*70}")
    print(f"  {quarter}")
    print(f"{'#'*70}")
    
    # Extract consecutive pairs (t, t+1)
    pairs = []
    for symbol in df["symbol"].unique():
        sym_df = df[df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        for i in range(len(sym_df) - 1):
            row_t = sym_df.iloc[i]
            row_t1 = sym_df.iloc[i+1]
            pairs.append({
                "ecology_t": row_t["ecology"],
                "run_length_t": row_t["run_length"],
                "eco_position_t": row_t["eco_position"],
                "vol_t1": row_t1["realized_volatility"],
                "range_t1": row_t1["session_range_pct"],
                "ecology_t1": row_t1["ecology"]
            })
    pair_df = pd.DataFrame(pairs)
    
    # ============================================================
    # EXPERIMENT 1: Run-length → Transition Probability
    # ============================================================
    print(f"\n  [EXPERIMENT 1] Attractor vs Excursion Dynamics")
    for eco, eco_name in [(0, "A (Quiet)"), (1, "B (Active)")]:
        print(f"\n    Ecology {eco_name}: P(Stay | run_length = k)")
        eco_pairs = pair_df[pair_df["ecology_t"] == eco]
        for k in [1, 2, 3]:
            if k == 3:
                k_pairs = eco_pairs[eco_pairs["run_length_t"] >= k]
                k_label = "3+"
            else:
                k_pairs = eco_pairs[eco_pairs["run_length_t"] == k]
                k_label = str(k)
            
            n_total = len(k_pairs)
            if n_total > 0:
                n_stay = sum(k_pairs["ecology_t1"] == eco)
                p_stay = n_stay / n_total
                print(f"      run_length = {k_label}: {p_stay:.3f} (stayed {n_stay}/{n_total})")

    # ============================================================
    # EXPERIMENT 2A: State → Next-Day Intensity
    # ============================================================
    print(f"\n  [EXPERIMENT 2A] Categorical State → Next-Day Intensity")
    
    ecoA_pairs = pair_df[pair_df["ecology_t"] == 0]
    ecoB_pairs = pair_df[pair_df["ecology_t"] == 1]
    
    print_categorical_comparison(
        "Volatility(t+1)", 
        ecoA_pairs["vol_t1"].values, 
        ecoB_pairs["vol_t1"].values
    )
    print_categorical_comparison(
        "Range(t+1)", 
        ecoA_pairs["range_t1"].values, 
        ecoB_pairs["range_t1"].values
    )
    
    print(f"\n  [EXPERIMENT 2A] Continuous eco_position(t) → Next-Day Intensity")
    rho_vol, p_vol = stats.spearmanr(pair_df["eco_position_t"], pair_df["vol_t1"])
    rho_rng, p_rng = stats.spearmanr(pair_df["eco_position_t"], pair_df["range_t1"])
    print(f"    eco_position(t) vs Volatility(t+1): Spearman ρ = {rho_vol:+.4f} (p = {p_vol:.4f})")
    print(f"    eco_position(t) vs Range(t+1):      Spearman ρ = {rho_rng:+.4f} (p = {p_rng:.4f})")

    # ============================================================
    # EXPERIMENT 2B: State Persistence → Future Intensity
    # ============================================================
    print(f"\n  [EXPERIMENT 2B] Run-length(t) → Next-Day Intensity")
    for eco, eco_name in [(0, "A"), (1, "B")]:
        print(f"\n    Ecology {eco_name}:")
        eco_pairs = pair_df[pair_df["ecology_t"] == eco]
        
        # We will compare run_length = 1 vs run_length >= 2
        rl1 = eco_pairs[eco_pairs["run_length_t"] == 1]
        rl2_plus = eco_pairs[eco_pairs["run_length_t"] >= 2]
        
        print_categorical_comparison(
            "Volatility(t+1) [run=1 vs run>=2]", 
            rl1["vol_t1"].values, 
            rl2_plus["vol_t1"].values
        )
        print_categorical_comparison(
            "Range(t+1) [run=1 vs run>=2]", 
            rl1["range_t1"].values, 
            rl2_plus["range_t1"].values
        )
        
        if len(eco_pairs) > 5:
            rho_vol, p_vol = stats.spearmanr(eco_pairs["run_length_t"], eco_pairs["vol_t1"])
            rho_rng, p_rng = stats.spearmanr(eco_pairs["run_length_t"], eco_pairs["range_t1"])
            print(f"    Spearman ρ (run_length vs Vol): {rho_vol:+.4f} (p = {p_vol:.4f})")
            print(f"    Spearman ρ (run_length vs Range): {rho_rng:+.4f} (p = {p_rng:.4f})")

    # ============================================================
    # EXPERIMENT 3: Descriptive Survival Analysis
    # ============================================================
    print(f"\n  [EXPERIMENT 3] Descriptive Survival Analysis")
    all_runs = {"A": [], "B": []}
    
    for symbol in df["symbol"].unique():
        sym_df = df[df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        ecologies = sym_df["ecology"].values
        if len(ecologies) == 0: continue
        
        current_label = ecologies[0]
        run_length = 1
        for i in range(1, len(ecologies)):
            if ecologies[i] == current_label:
                run_length += 1
            else:
                label_name = "A" if current_label == 0 else "B"
                all_runs[label_name].append(run_length)
                current_label = ecologies[i]
                run_length = 1
        # The last run is technically censored, but we'll include it for descriptive shape
        label_name = "A" if current_label == 0 else "B"
        all_runs[label_name].append(run_length)
        
    for name in ["A", "B"]:
        runs = np.array(all_runs[name])
        if len(runs) == 0: continue
        max_run = runs.max()
        
        # KM survival function S(t) = P(T > t)
        # Empirical: count of runs > t / total runs
        t_vals = np.arange(1, max_run + 2)
        s_vals = [np.sum(runs >= t) / len(runs) for t in t_vals]
        
        print(f"    Ecology {name} (n={len(runs)} runs):")
        for t, s in zip(t_vals, s_vals):
            if t <= 5 or s == 0:
                print(f"      S({t}) = P(length >= {t}) = {s:.3f}")
            if s == 0:
                break
