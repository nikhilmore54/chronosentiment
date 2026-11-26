#!/usr/bin/env python3
"""Phase 2 targeted tests — mechanism validation.

Test 1: ecological_position(t) → net_return(t+1)
Test 2: gap_pct(t) → P(transition)
Test 3: GMM(k=2) vs Ward — separation vector comparison
"""
import json
import sys
import numpy as np
import pandas as pd
from scipy import stats

sys.path.insert(0, ".")
from ecology_utils import load_session_catalog, standardize, ward_clustering

METRICS = ["realized_volatility", "trend_strength", "gap_pct", "session_range_pct", "net_return_pct"]

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

# ============================================================
# Load both quarters
# ============================================================
results = {}

for quarter, path in [("Q1", "phase1/analysis/coordinate_audit/session_catalog_q1.json"),
                       ("Q2", "phase1/analysis/coordinate_audit/session_catalog_q2.json")]:
    full_df = load_full(path)
    metric_df = load_session_catalog(path)
    X_df, means, stds = standardize(metric_df)
    X = X_df.values
    labels = ward_clustering(X, n_clusters=2)
    
    # Store for cross-quarter alignment
    centroids = np.array([X[labels == i].mean(axis=0) for i in range(2)])
    
    results[quarter] = {
        "full_df": full_df,
        "metric_df": metric_df,
        "X": X,
        "labels": labels,
        "centroids": centroids,
        "means": means,
        "stds": stds,
    }

# Align Q2 to Q1
results["Q2"]["labels"] = align_labels_by_centroid(
    results["Q2"]["X"], results["Q2"]["labels"], results["Q1"]["centroids"]
)
results["Q2"]["centroids"] = np.array([
    results["Q2"]["X"][results["Q2"]["labels"] == i].mean(axis=0) for i in range(2)
])

# Compute separation vectors
q1_sep = results["Q1"]["centroids"][0] - results["Q1"]["centroids"][1]
q2_sep = results["Q2"]["centroids"][0] - results["Q2"]["centroids"][1]

# Average separation vector (for projection)
avg_sep = (q1_sep / np.linalg.norm(q1_sep) + q2_sep / np.linalg.norm(q2_sep)) / 2
avg_sep = avg_sep / np.linalg.norm(avg_sep)  # normalize

print(f"Q1 separation vector: {np.round(q1_sep, 3).tolist()}")
print(f"Q2 separation vector: {np.round(q2_sep, 3).tolist()}")
print(f"Averaged unit vector:  {np.round(avg_sep, 3).tolist()}")

# ============================================================
# TEST 1: ecological_position(t) → net_return(t+1)
# ============================================================
print(f"\n{'='*70}")
print(f"  TEST 1: ECOLOGICAL POSITION → NEXT-SESSION NET RETURN")
print(f"{'='*70}")

for quarter in ["Q1", "Q2"]:
    r = results[quarter]
    full_df = r["full_df"].dropna(subset=METRICS).reset_index(drop=True)
    X = r["X"]
    
    # Compute ecological position (projection onto separation vector)
    eco_pos = X @ avg_sep
    
    full_df = full_df.copy()
    full_df["eco_position"] = eco_pos
    
    for symbol in full_df["symbol"].unique():
        sym_df = full_df[full_df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        
        # Pair: eco_position(t) with net_return(t+1)
        t_pos = sym_df["eco_position"].values[:-1]
        t1_return = sym_df["net_return_pct"].values[1:]
        
        # Pearson correlation
        if len(t_pos) > 5:
            r_val, p_val = stats.pearsonr(t_pos, t1_return)
            # Spearman (rank) correlation
            rho, p_rho = stats.spearmanr(t_pos, t1_return)
            
            # Conditional expectation: split by sign of eco_position
            low = t1_return[t_pos < np.median(t_pos)]
            high = t1_return[t_pos >= np.median(t_pos)]
            
            print(f"\n  {quarter} {symbol} (n={len(t_pos)}):")
            print(f"    Pearson r  = {r_val:+.4f} (p = {p_val:.4f})")
            print(f"    Spearman ρ = {rho:+.4f} (p = {p_rho:.4f})")
            print(f"    E[return | low eco]  = {low.mean():.4f} (n={len(low)})")
            print(f"    E[return | high eco] = {high.mean():.4f} (n={len(high)})")
            print(f"    Difference           = {high.mean() - low.mean():+.4f}")

    # Also pooled across symbols
    print(f"\n  {quarter} POOLED:")
    all_pairs = []
    for symbol in full_df["symbol"].unique():
        sym_df = full_df[full_df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        for i in range(len(sym_df) - 1):
            all_pairs.append((sym_df.iloc[i]["eco_position"], sym_df.iloc[i+1]["net_return_pct"]))
    
    all_pairs = np.array(all_pairs)
    r_val, p_val = stats.pearsonr(all_pairs[:, 0], all_pairs[:, 1])
    rho, p_rho = stats.spearmanr(all_pairs[:, 0], all_pairs[:, 1])
    
    med = np.median(all_pairs[:, 0])
    low_ret = all_pairs[all_pairs[:, 0] < med, 1]
    high_ret = all_pairs[all_pairs[:, 0] >= med, 1]
    
    print(f"    Pearson r  = {r_val:+.4f} (p = {p_val:.4f})")
    print(f"    Spearman ρ = {rho:+.4f} (p = {p_rho:.4f})")
    print(f"    E[return | low eco]  = {low_ret.mean():.4f} ± {low_ret.std():.4f}")
    print(f"    E[return | high eco] = {high_ret.mean():.4f} ± {high_ret.std():.4f}")
    print(f"    Difference           = {high_ret.mean() - low_ret.mean():+.4f}")
    
    # Also test: does eco_position(t) predict DIRECTION of return(t+1)?
    # (not magnitude, just sign — is it more likely to be up/down?)
    # Use: does higher eco_position predict higher absolute return?
    abs_ret = np.abs(all_pairs[:, 1])
    r_abs, p_abs = stats.pearsonr(all_pairs[:, 0], abs_ret)
    print(f"    Pearson r (eco → |return|) = {r_abs:+.4f} (p = {p_abs:.4f})")

# ============================================================
# TEST 2: gap_pct → transition probability
# ============================================================
print(f"\n{'='*70}")
print(f"  TEST 2: GAP_PCT → TRANSITION PROBABILITY")
print(f"{'='*70}")

for quarter in ["Q1", "Q2"]:
    r = results[quarter]
    full_df = r["full_df"].dropna(subset=METRICS).reset_index(drop=True)
    labels = r["labels"]
    full_df["ecology"] = labels
    
    for symbol in full_df["symbol"].unique():
        sym_df = full_df[full_df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        
        gaps = []
        transitions = []
        for i in range(len(sym_df) - 1):
            gap = sym_df.iloc[i+1]["gap_pct"]  # gap entering session t+1
            curr_eco = sym_df.iloc[i]["ecology"]
            next_eco = sym_df.iloc[i+1]["ecology"]
            did_transition = 1 if curr_eco != next_eco else 0
            if not np.isnan(gap):
                gaps.append(gap)
                transitions.append(did_transition)
        
        gaps = np.array(gaps)
        transitions = np.array(transitions)
        
        if len(gaps) > 5:
            # Point-biserial correlation (gap continuous, transition binary)
            r_val, p_val = stats.pointbiserialr(transitions, gaps)
            
            # Also: does |gap| predict transition?
            r_abs, p_abs = stats.pointbiserialr(transitions, np.abs(gaps))
            
            # Conditional: mean |gap| for stay vs transition
            stay_gap = np.abs(gaps[transitions == 0])
            trans_gap = np.abs(gaps[transitions == 1])
            
            print(f"\n  {quarter} {symbol} (n={len(gaps)}):")
            print(f"    gap → transition: r = {r_val:+.4f} (p = {p_val:.4f})")
            print(f"    |gap| → transition: r = {r_abs:+.4f} (p = {p_abs:.4f})")
            print(f"    Mean |gap| when STAY:       {stay_gap.mean():.4f} (n={len(stay_gap)})")
            print(f"    Mean |gap| when TRANSITION: {trans_gap.mean():.4f} (n={len(trans_gap)})")
    
    # Pooled
    all_gaps = []
    all_trans = []
    for symbol in full_df["symbol"].unique():
        sym_df = full_df[full_df["symbol"] == symbol].sort_values("date").reset_index(drop=True)
        for i in range(len(sym_df) - 1):
            gap = sym_df.iloc[i+1]["gap_pct"]
            curr_eco = sym_df.iloc[i]["ecology"]
            next_eco = sym_df.iloc[i+1]["ecology"]
            did_transition = 1 if curr_eco != next_eco else 0
            if not np.isnan(gap):
                all_gaps.append(gap)
                all_trans.append(did_transition)
    
    all_gaps = np.array(all_gaps)
    all_trans = np.array(all_trans)
    r_val, p_val = stats.pointbiserialr(all_trans, all_gaps)
    r_abs, p_abs = stats.pointbiserialr(all_trans, np.abs(all_gaps))
    
    stay_gap = np.abs(all_gaps[all_trans == 0])
    trans_gap = np.abs(all_gaps[all_trans == 1])
    
    print(f"\n  {quarter} POOLED (n={len(all_gaps)}):")
    print(f"    gap → transition: r = {r_val:+.4f} (p = {p_val:.4f})")
    print(f"    |gap| → transition: r = {r_abs:+.4f} (p = {p_abs:.4f})")
    print(f"    Mean |gap| when STAY:       {stay_gap.mean():.4f}")
    print(f"    Mean |gap| when TRANSITION: {trans_gap.mean():.4f}")

# ============================================================
# TEST 3: GMM vs Ward — separation vector comparison
# ============================================================
print(f"\n{'='*70}")
print(f"  TEST 3: GMM (k=2) vs WARD — SEPARATION VECTOR COMPARISON")
print(f"{'='*70}")

from sklearn.mixture import GaussianMixture

for quarter in ["Q1", "Q2"]:
    r = results[quarter]
    X = r["X"]
    ward_labels = r["labels"]
    ward_centroids = r["centroids"]
    
    # Fit GMM
    gmm = GaussianMixture(n_components=2, random_state=42, covariance_type="full")
    gmm_labels = gmm.fit_predict(X)
    gmm_centroids = gmm.means_
    
    # Align GMM labels to Ward
    gmm_labels_aligned = align_labels_by_centroid(X, gmm_labels, ward_centroids)
    gmm_centroids_aligned = np.array([X[gmm_labels_aligned == i].mean(axis=0) for i in range(2)])
    
    # Separation vectors
    ward_sep = ward_centroids[0] - ward_centroids[1]
    gmm_sep = gmm_centroids_aligned[0] - gmm_centroids_aligned[1]
    
    cos_sim = np.dot(ward_sep, gmm_sep) / (np.linalg.norm(ward_sep) * np.linalg.norm(gmm_sep))
    
    # ARI between Ward and GMM
    from sklearn.metrics import adjusted_rand_score
    ari = adjusted_rand_score(ward_labels, gmm_labels_aligned)
    
    # GMM cluster sizes
    unique, counts = np.unique(gmm_labels_aligned, return_counts=True)
    
    # GMM covariance structure
    covs = gmm.covariances_
    # Diagonal variances for each component
    diag_vars = [np.diag(covs[i]).tolist() for i in range(2)]
    
    print(f"\n  {quarter}:")
    print(f"    Ward cluster sizes: {dict(zip(*np.unique(ward_labels, return_counts=True)))}")
    print(f"    GMM  cluster sizes: {dict(zip(unique.tolist(), counts.tolist()))}")
    print(f"    ARI (Ward vs GMM): {ari:.4f}")
    print(f"    Separation vector cosine (Ward vs GMM): {cos_sim:.4f}")
    print(f"    Ward separation: {np.round(ward_sep, 3).tolist()}")
    print(f"    GMM  separation: {np.round(gmm_sep, 3).tolist()}")
    print(f"    GMM covariance diagonal (cluster 0): {[round(v, 3) for v in diag_vars[0]]}")
    print(f"    GMM covariance diagonal (cluster 1): {[round(v, 3) for v in diag_vars[1]]}")
    
    # Variance ratio for GMM
    var_ratio_A = np.diag(covs[0]).sum()
    var_ratio_B = np.diag(covs[1]).sum()
    print(f"    GMM total variance: cluster 0 = {var_ratio_A:.3f}, cluster 1 = {var_ratio_B:.3f}")
    print(f"    Variance ratio (B/A): {var_ratio_B / var_ratio_A:.3f}")

# Cross-quarter GMM separation vector comparison
print(f"\n  Cross-quarter GMM comparison:")
# Recompute GMM for both quarters
gmm_seps = {}
for quarter in ["Q1", "Q2"]:
    X = results[quarter]["X"]
    gmm = GaussianMixture(n_components=2, random_state=42, covariance_type="full")
    gmm_labels = gmm.fit_predict(X)
    gmm_labels_aligned = align_labels_by_centroid(X, gmm_labels, results["Q1"]["centroids"])
    gmm_centroids_aligned = np.array([X[gmm_labels_aligned == i].mean(axis=0) for i in range(2)])
    gmm_seps[quarter] = gmm_centroids_aligned[0] - gmm_centroids_aligned[1]

gmm_cross_cos = np.dot(gmm_seps["Q1"], gmm_seps["Q2"]) / (
    np.linalg.norm(gmm_seps["Q1"]) * np.linalg.norm(gmm_seps["Q2"])
)
print(f"    Q1 GMM sep: {np.round(gmm_seps['Q1'], 3).tolist()}")
print(f"    Q2 GMM sep: {np.round(gmm_seps['Q2'], 3).tolist()}")
print(f"    Cosine similarity (Q1 GMM ↔ Q2 GMM): {gmm_cross_cos:.4f}")

ward_cross_cos = np.dot(q1_sep, q2_sep) / (np.linalg.norm(q1_sep) * np.linalg.norm(q2_sep))
print(f"    Cosine similarity (Q1 Ward ↔ Q2 Ward): {ward_cross_cos:.4f}")

print(f"\n{'='*70}")
print(f"  SUMMARY")
print(f"{'='*70}")
