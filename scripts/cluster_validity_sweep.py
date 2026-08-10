"""Cluster validity sweep for Q1 session catalog.
Generates:
- archive/research_outputs/cluster_validity_report.md (table of scores and embedded plot)
- archive/research_outputs/cluster_validity_plot.png (k vs Silhouette, Davies‑Bouldin, Calinski‑Harabasz)
"""

import json
from pathlib import Path
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from sklearn.preprocessing import StandardScaler
from sklearn.cluster import AgglomerativeClustering
from sklearn.metrics import silhouette_score, davies_bouldin_score, calinski_harabasz_score

# Paths
CATALOG_PATH = Path("phase1/analysis/coordinate_audit/session_catalog_q1.json")
MD_OUT = Path("archive/research_outputs/cluster_validity_report.md")
PNG_OUT = Path("archive/research_outputs/cluster_validity_plot.png")

# Load catalog
catalog = json.loads(CATALOG_PATH.read_text())

# Build DataFrame
records = []
for entry in catalog:
    records.append({
        "date": entry["date"],
        "symbol": entry["symbol"],
        "realized_volatility": entry.get("realized_volatility"),
        "trend_strength": entry.get("trend_strength"),
        "gap_pct": entry.get("gap_pct"),
        "session_range_pct": entry.get("session_range_pct"),
        "net_return_pct": entry.get("net_return_pct"),
    })

df = pd.DataFrame(records)

# Metrics to use
metrics = ["realized_volatility", "trend_strength", "gap_pct", "session_range_pct", "net_return_pct"]
# Drop any rows with missing values (e.g., first session gaps are null)
df_clean = df.dropna(subset=metrics)
X = df_clean[metrics].astype(float).values

# Standardize
scaler = StandardScaler()
X_std = scaler.fit_transform(X)

# Sweep k from 2 to 10
k_range = range(2, 11)
results = []
for k in k_range:
    clustering = AgglomerativeClustering(n_clusters=k)
    labels = clustering.fit_predict(X_std)
    # Compute metrics; silhouette requires at least 2 clusters and less than n_samples
    sil = silhouette_score(X_std, labels)
    db = davies_bouldin_score(X_std, labels)
    ch = calinski_harabasz_score(X_std, labels)
    results.append({"k": k, "silhouette": sil, "davies_bouldin": db, "calinski_harabasz": ch})

# Plot the three scores
plt.figure(figsize=(8, 6))
ks = [r["k"] for r in results]
plt.plot(ks, [r["silhouette"] for r in results], marker='o', label='Silhouette')
plt.plot(ks, [r["davies_bouldin"] for r in results], marker='s', label='Davies‑Bouldin')
plt.plot(ks, [r["calinski_harabasz"] for r in results], marker='^', label='Calinski‑Harabasz')
plt.xlabel('Number of clusters (k)')
plt.title('Cluster validity metrics vs k')
plt.legend()
plt.grid(True, linestyle='--', alpha=0.5)
plt.tight_layout()
plt.savefig(PNG_OUT, dpi=150)
plt.close()
print(f"Plot saved to {PNG_OUT}")

# Markdown report
md_lines = []
md_lines.append("# Q1 Cluster Validity Sweep\n")
md_lines.append(f"Data source: `{CATALOG_PATH.name}` – **{len(df_clean)} sessions used** (rows with missing metrics dropped).\n")
md_lines.append("## Scores table\n")
md_lines.append("| k | Silhouette | Davies‑Bouldin | Calinski‑Harabasz |\n|---|---|---|---|\n")
for r in results:
    md_lines.append(f"| {r['k']} | {r['silhouette']:.4f} | {r['davies_bouldin']:.4f} | {r['calinski_harabasz']:.2f} |\n")
md_lines.append("\n## Score plot\n")
md_lines.append(f"![Cluster validity plot]({PNG_OUT.name})\n")

MD_OUT.write_text("\n".join(md_lines))
print(f"Markdown report written to {MD_OUT}")
