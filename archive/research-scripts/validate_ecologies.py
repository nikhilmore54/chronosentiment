# validate_ecologies.py
"""Main pipeline for ecological‑structure validation (Phase 1B).
The script follows the plan documented in
implementation_plan_ecology_validation.md.
It produces four artifacts in the project root:
- cluster_stability_report.md
- cluster_stability_plot.png
- null_model_comparison.json
- ecology_certification.json
"""
import json
import os
from pathlib import Path
import numpy as np
import matplotlib.pyplot as plt

from ecology_utils import (
    load_session_catalog,
    standardize,
    ward_clustering,
    bootstrap_projection,
    perturbation_ari,
    permutation_null_metrics,
    compute_real_metrics,
    empirical_pvalue,
)

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
import argparse

parser = argparse.ArgumentParser(description="Ecology validation (Phase 1B)")
parser.add_argument("--catalog", type=str,
    default=str(Path(__file__).parent / "phase1/analysis/coordinate_audit/session_catalog_q1.json"),
    help="Path to the session catalog JSON (default: Q1 catalog)")
parser.add_argument("--output-dir", type=str,
    default=str(Path(__file__).parent),
    help="Directory where validation artifacts will be written (default: script directory)")
parser.add_argument("--k-range", type=str, default="2-10",
    help="Range of k values to test, expressed as 'min-max' (default: 2-10)")

args = parser.parse_args()
CATALOG_PATH = Path(args.catalog)
OUTPUT_DIR = Path(args.output_dir)
# Parse k_range like "2-10"
try:
    k_min, k_max = map(int, args.k_range.split("-"))
    K_RANGE = range(k_min, k_max + 1)
except Exception:
    raise ValueError("Invalid --k-range format; expected 'min-max'.")

BOOTSTRAP_REPEATS = 30
PERTURB_SIGMAS = [0.005, 0.01, 0.02, 0.05]
NULL_REPEATS = 30

def main():
    # ---------------------------------------------------
    # Ensure output directory exists
    # ---------------------------------------------------
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # ---------------------------------------------------
    # Load and standardize data
    # ---------------------------------------------------
    df = load_session_catalog(str(CATALOG_PATH))
    X_df, means, stds = standardize(df)
    X = X_df.values

    # Containers for results
    results = {}
    null_collect = {}

    for k in K_RANGE:
        # ---------------------------------------------------
        # Reference clustering
        # ---------------------------------------------------
        ref_labels = ward_clustering(X, n_clusters=k)

        # ---------------------------------------------------
        # Real‑data metrics
        # ---------------------------------------------------
        real_metrics = compute_real_metrics(X, ref_labels)

        # ---------------------------------------------------
        # Bootstrap stability (projected ARI)
        # ---------------------------------------------------
        bootstrap_aris = bootstrap_projection(
            X, ref_labels, n_clusters=k, n_bootstrap=BOOTSTRAP_REPEATS
        )
        boot_mean = float(np.mean(bootstrap_aris))
        boot_std = float(np.std(bootstrap_aris))

        # ---------------------------------------------------
        # Perturbation robustness (ARI vs. noisy data)
        # ---------------------------------------------------
        pert_aris = {}
        for sigma in PERTURB_SIGMAS:
            ari = perturbation_ari(
                X, ref_labels, n_clusters=k, sigma=sigma, means=means, stds=stds
            )
            pert_aris[sigma] = float(ari)

        # ---------------------------------------------------
        # Null model (feature‑wise permutation)
        # ---------------------------------------------------
        null_metrics = permutation_null_metrics(X, n_clusters=k, n_null=NULL_REPEATS)
        # Store null distributions for later plotting / p‑value calculation
        null_collect[k] = null_metrics

        # ---------------------------------------------------
        # Empirical p‑values for real metrics vs. null
        # ---------------------------------------------------
        p_sil = empirical_pvalue(real_metrics["silhouette"], null_metrics["silhouette"])
        p_db = empirical_pvalue(real_metrics["db"], null_metrics["db"])
        p_ch = empirical_pvalue(real_metrics["ch"], null_metrics["ch"])

        # ---------------------------------------------------
        # Assemble per‑k result dictionary
        # ---------------------------------------------------
        results[k] = {
            "reference_labels": ref_labels.tolist(),
            "real_metrics": real_metrics,
            "bootstrap_ari": {
                "mean": boot_mean,
                "std": boot_std,
                "values": bootstrap_aris,
            },
            "perturbation_ari": {str(s): v for s, v in pert_aris.items()},
            "null_pvalues": {"silhouette": p_sil, "db": p_db, "ch": p_ch},
        }

    # ---------------------------------------------------
    # Write artifacts
    # ---------------------------------------------------
    write_report(results, null_collect)
    write_null_json(null_collect)
    write_certification(results)

def write_report(results, null_collect):
    """Generate a human‑readable markdown report summarising the evidence.
    The report purposefully avoids any naming or interpretation of clusters.
    """
    lines = []
    lines.append("# Ecological Structure Validation Report")
    lines.append("\nThis report presents evidence that the Q1 session‑metric space contains non‑random cluster structure. No regime names or interpretations are provided at this stage.\n")
    lines.append("## Summary Table")
    lines.append("| k | Silhouette | Silhouette p | Bootstrap ARI (mean±std) | Perturbation ARI σ=0.02 |\n|---|------------|--------------|--------------------------|------------------------|")
    for k, info in results.items():
        sil = info["real_metrics"]["silhouette"]
        p_sil = info["null_pvalues"]["silhouette"]
        boot = info["bootstrap_ari"]
        pert_ari_0_02 = info["perturbation_ari"].get("0.02", "N/A")
        lines.append(
            f"| {k} | {sil:.3f} | {p_sil:.3f} | {boot['mean']:.3f}±{boot['std']:.3f} | {pert_ari_0_02:.3f} |"
        )
    lines.append("\n## Detailed Evidence per k")
    for k, info in results.items():
        lines.append(f"### k = {k}\n")
        lines.append("**Real‑data metrics**\n")
        for metric, val in info["real_metrics"].items():
            lines.append(f"- {metric}: {val:.4f}\n")
        lines.append("**Bootstrap stability (ARI)**\n")
        lines.append(
            f"- Mean ARI: {info['bootstrap_ari']['mean']:.4f}\n- Std ARI: {info['bootstrap_ari']['std']:.4f}\n"
        )
        lines.append("**Perturbation robustness (ARI vs. noisy data)**\n")
        for sigma_str, ari_val in info["perturbation_ari"].items():
            lines.append(f"- σ={sigma_str}: ARI = {ari_val:.4f}\n")
        lines.append("**Null‑model comparison (empirical p‑values)**\n")
        for metric, pval in info["null_pvalues"].items():
            lines.append(f"- {metric} p‑value: {pval:.4f}\n")
        lines.append("---\n")
    # Write to file
    report_path = OUTPUT_DIR / "cluster_stability_report.md"
    with open(report_path, "w") as f:
        f.write("\n".join(lines))

    # ---------------------------------------------------
    # Plotting (k vs. silhouette, bootstrap ARI, perturbation ARI)
    # ---------------------------------------------------
    ks = list(results.keys())
    sil_means = [results[k]["real_metrics"]["silhouette"] for k in ks]
    sil_null_means = [np.mean(null_collect[k]["silhouette"]) for k in ks]
    boot_means = [results[k]["bootstrap_ari"]["mean"] for k in ks]
    pert_ari_sigma02 = [results[k]["perturbation_ari"].get("0.02", np.nan) for k in ks]

    plt.figure(figsize=(10, 6))
    plt.plot(ks, sil_means, "o-", label="Silhouette (real)")
    plt.plot(ks, sil_null_means, "x--", label="Silhouette (null mean)")
    plt.plot(ks, boot_means, "s-", label="Bootstrap ARI (mean)")
    plt.plot(ks, pert_ari_sigma02, "d-", label="Perturbation ARI σ=0.02")
    plt.xlabel("Number of clusters k")
    plt.ylabel("Metric value")
    plt.title("Ecological validation metrics across k")
    plt.legend()
    plt.grid(True, linestyle="--", alpha=0.5)
    plot_path = OUTPUT_DIR / "cluster_stability_plot.png"
    plt.tight_layout()
    plt.savefig(plot_path, dpi=150)
    plt.close()

def write_null_json(null_collect):
    """Save raw null‑model metric arrays for reproducibility."""
    out_path = OUTPUT_DIR / "null_model_comparison.json"
    # Convert numpy arrays/lists to plain python lists
    serialisable = {
        str(k): {
            metric: values for metric, values in metrics.items()
        }
        for k, metrics in null_collect.items()
    }
    with open(out_path, "w") as f:
        json.dump(serialisable, f, indent=2)

def write_certification(results):
    """Create a concise evidence‑only JSON artifact.
    The JSON purpose is to be consumed by downstream phases; it does **not** embed
    hard thresholds or a final decision.
    """
    # Choose the k with the highest silhouette that also has p < 0.05 for silhouette.
    # This heuristic is only for convenience; users may pick another k.
    best_k = None
    best_sil = -np.inf
    for k, info in results.items():
        sil = info["real_metrics"]["silhouette"]
        p = info["null_pvalues"]["silhouette"]
        if p < 0.05 and sil > best_sil:
            best_sil = sil
            best_k = k
    if best_k is None:
        best_k = min(results.keys())  # fallback

    cert = {
        "k": best_k,
        "silhouette": results[best_k]["real_metrics"]["silhouette"],
        "silhouette_null_p": results[best_k]["null_pvalues"]["silhouette"],
        "bootstrap_ari_mean": results[best_k]["bootstrap_ari"]["mean"],
        "bootstrap_ari_std": results[best_k]["bootstrap_ari"]["std"],
        "perturbation_ari_sigma_0.02": results[best_k]["perturbation_ari"].get("0.02"),
        "notes": "Evidence artifact – no hard certification decision applied."
    }
    out_path = OUTPUT_DIR / "ecology_certification.json"
    with open(out_path, "w") as f:
        json.dump(cert, f, indent=2)

if __name__ == "__main__":
    main()
