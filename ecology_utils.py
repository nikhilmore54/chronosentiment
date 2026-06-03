# ecology_utils.py
"""Utility functions for ecological structure validation.
This module provides helpers for loading data, standardizing metrics,
performing Ward hierarchical clustering, bootstrap stability projection,
perturbation robustness, and feature‑wise permutation null generation.
All functions are pure and have no side‑effects beyond returning values.
"""
import json
import numpy as np
import pandas as pd
from pathlib import Path
from sklearn.preprocessing import StandardScaler
from sklearn.cluster import AgglomerativeClustering
from sklearn.metrics import silhouette_score, davies_bouldin_score, calinski_harabasz_score, adjusted_rand_score
from scipy.optimize import linear_sum_assignment


def load_session_catalog(catalog_path: str) -> pd.DataFrame:
    """Load the session catalog JSON and return a DataFrame with the five metrics.
    Rows with any missing metric are dropped.
    """
    with open(catalog_path, "r") as f:
        data = json.load(f)
    df = pd.DataFrame(data)
    metrics = [
        "realized_volatility",
        "trend_strength",
        "gap_pct",
        "session_range_pct",
        "net_return_pct",
    ]
    df = df[metrics]
    df = df.dropna()
    return df


def standardize(df: pd.DataFrame) -> tuple[pd.DataFrame, np.ndarray, np.ndarray]:
    """Z‑score the DataFrame.
    Returns the scaled DataFrame, the means and stds (as arrays) for later noise addition.
    """
    scaler = StandardScaler()
    scaled = scaler.fit_transform(df.values)
    scaled_df = pd.DataFrame(scaled, columns=df.columns, index=df.index)
    return scaled_df, scaler.mean_, scaler.scale_


def ward_clustering(X: np.ndarray, n_clusters: int) -> np.ndarray:
    """Run Ward hierarchical clustering on the data matrix X.
    Returns a 1‑D array of cluster labels (0‑based).
    """
    model = AgglomerativeClustering(n_clusters=n_clusters, linkage="ward")
    labels = model.fit_predict(X)
    return labels


def bootstrap_projection(
    X: np.ndarray,
    original_labels: np.ndarray,
    n_clusters: int,
    n_bootstrap: int = 30,
) -> list[float]:
    """Perform bootstrap stability assessment.
    For each bootstrap sample we:
    1. Sample rows *with replacement*.
    2. Cluster the bootstrap sample.
    3. Project the bootstrap labels back onto the original index set by majority vote.
    4. Compute ARI between the projected labels and the original labels *only on the overlapping rows*.
    Returns a list of ARI values (one per bootstrap run).
    """
    n = X.shape[0]
    ari_scores = []
    rng = np.random.default_rng()
    for _ in range(n_bootstrap):
        # Sample indices with replacement
        boot_idx = rng.integers(0, n, size=n)
        X_boot = X[boot_idx]
        boot_labels = ward_clustering(X_boot, n_clusters)
        # Map bootstrap labels back to original observations
        # Build a dict: original index -> list of bootstrap labels assigned to its copies
        label_dict = {i: [] for i in range(n)}
        for b_idx, orig_i in enumerate(boot_idx):
            label_dict[orig_i].append(boot_labels[b_idx])
        # For each original observation present in the bootstrap, take majority vote
        projected = []
        present = []
        for i in range(n):
            votes = label_dict[i]
            if votes:  # observation appears at least once
                # majority vote (break ties by smallest label)
                counts = np.bincount(votes)
                maj = int(np.argmax(counts))
                projected.append(maj)
                present.append(i)
        # Compute ARI on the overlapping subset
        ari = adjusted_rand_score(original_labels[present], np.array(projected))
        ari_scores.append(ari)
    return ari_scores


def perturbation_ari(
    X: np.ndarray,
    original_labels: np.ndarray,
    n_clusters: int,
    sigma: float,
    means: np.ndarray,
    stds: np.ndarray,
) -> float:
    """Add Gaussian noise (relative to feature std) and compute ARI against original labels.
    Returns a single ARI value.
    """
    rng = np.random.default_rng()
    noise = rng.normal(loc=0.0, scale=sigma, size=X.shape) * stds  # scale per feature
    X_noisy = X + noise
    noisy_labels = ward_clustering(X_noisy, n_clusters)
    ari = adjusted_rand_score(original_labels, noisy_labels)
    return ari


def permutation_null_metrics(
    X: np.ndarray,
    n_clusters: int,
    n_null: int = 30,
) -> dict[str, list[float]]:
    """Generate null distributions by permuting each feature independently.
    Returns a dict with keys 'silhouette', 'db', 'ch' mapping to lists of metric values.
    """
    n, dim = X.shape
    rng = np.random.default_rng()
    results = {"silhouette": [], "db": [], "ch": []}
    for _ in range(n_null):
        X_perm = np.copy(X)
        for d in range(dim):
            rng.shuffle(X_perm[:, d])
        labels = ward_clustering(X_perm, n_clusters)
        # Silhouette requires at least 2 clusters and less than n samples per cluster
        sil = silhouette_score(X_perm, labels) if n_clusters > 1 else np.nan
        db = davies_bouldin_score(X_perm, labels)
        ch = calinski_harabasz_score(X_perm, labels)
        results["silhouette"].append(sil)
        results["db"].append(db)
        results["ch"].append(ch)
    return results


def compute_real_metrics(X: np.ndarray, labels: np.ndarray) -> dict[str, float]:
    """Compute silhouette, DB, and CH on the original data/labels.
    Returns a dict with the three metric values.
    """
    sil = silhouette_score(X, labels) if len(np.unique(labels)) > 1 else np.nan
    db = davies_bouldin_score(X, labels)
    ch = calinski_harabasz_score(X, labels)
    return {"silhouette": sil, "db": db, "ch": ch}


def empirical_pvalue(real_value: float, null_values: list[float]) -> float:
    """Two‑sided empirical p‑value: proportion of null values >= real value.
    Adds 1 to numerator and denominator for a conservative estimate.
    """
    greater_eq = sum(1 for v in null_values if v >= real_value)
    p = (1 + greater_eq) / (1 + len(null_values))
    return p

# Helper for aligning labels when comparing clusterings of different subsets
def align_labels(reference: np.ndarray, target: np.ndarray) -> np.ndarray:
    """Align `target` labels to `reference` using the Hungarian algorithm.
    Returns a new label array with the same set of integers as `reference`.
    """
    # Build contingency matrix
    n_ref = len(np.unique(reference))
    n_tar = len(np.unique(target))
    contingency = np.zeros((n_ref, n_tar), dtype=int)
    for i in range(len(reference)):
        contingency[reference[i], target[i]] += 1
    row_ind, col_ind = linear_sum_assignment(-contingency)
    mapping = {col: row for row, col in zip(row_ind, col_ind)}
    aligned = np.vectorize(mapping.get)(target)
    return aligned

if __name__ == "__main__":
    # Simple sanity test when run directly
    catalog_path = Path(__file__).parent / "session_catalog_q1.json"
    df = load_session_catalog(str(catalog_path))
    X, means, stds = standardize(df)
    labels = ward_clustering(X.values, n_clusters=4)
    print("Loaded", len(df), "sessions, first clustering ARI with itself = 1.0")
