"""Generate ecological clustering for Q1 session catalog.
Outputs:
- archive/research_outputs/ecology_cluster_report.json (cluster assignments and metadata)
- archive/research_outputs/ecology_pca_plot.png (2D PCA scatter colored by cluster)
- archive/research_outputs/ecology_cluster_report.md (markdown summary)
"""

import json
from pathlib import Path
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from sklearn.preprocessing import StandardScaler
from sklearn.decomposition import PCA
from sklearn.cluster import AgglomerativeClustering

# Paths
CATALOG_PATH = Path("phase1/analysis/coordinate_audit/session_catalog_q1.json")
JSON_OUT = Path("archive/research_outputs/ecology_cluster_report.json")
MD_OUT = Path("archive/research_outputs/ecology_cluster_report.md")
PNG_OUT = Path("archive/research_outputs/ecology_pca_plot.png")

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

# Metrics matrix
metrics = ["realized_volatility", "trend_strength", "gap_pct", "session_range_pct", "net_return_pct"]
# Drop rows that have any missing metric values (e.g., gap_pct may be null for the first session)
df_clean = df.dropna(subset=metrics)
X = df_clean[metrics].astype(float).values

# Standardize (z‑score)
scaler = StandardScaler()
X_std = scaler.fit_transform(X)

# PCA to 2 dimensions for visualization
pca = PCA(n_components=2, random_state=0)
X_pca = pca.fit_transform(X_std)

df_clean["pca1"] = X_pca[:, 0]
df_clean["pca2"] = X_pca[:, 1]

# Hierarchical clustering – choose 4 clusters (reasonable for Q1 ecology)
clust = AgglomerativeClustering(n_clusters=4)
labels = clust.fit_predict(X_std)

df_clean["cluster"] = labels

# Export JSON report (list of dicts)
report = df_clean.to_dict(orient="records")
JSON_OUT.write_text(json.dumps(report, indent=2))
print(f"JSON report written to {JSON_OUT}")

# Scatter plot
plt.figure(figsize=(8, 6))
for c in sorted(df_clean["cluster"].unique()):
    subset = df_clean[df_clean["cluster"] == c]
    plt.scatter(subset["pca1"], subset["pca2"], label=f"Cluster {c}", alpha=0.7)
plt.title("PCA of Q1 Sessions – Ecological Clusters")
plt.xlabel("PC 1")
plt.ylabel("PC 2")
plt.legend()
plt.grid(True, ls="--", alpha=0.5)
plt.tight_layout()
plt.savefig(PNG_OUT, dpi=150)
plt.close()
print(f"PCA plot saved to {PNG_OUT}")

# Markdown summary
md_lines = []
md_lines.append("# Q1 Ecological Clustering Report\n")
md_lines.append(f"Generated from `{CATALOG_PATH.name}` – **{len(df_clean)} sessions**.\n")
md_lines.append(f"**Number of clusters:** {len(df_clean['cluster'].unique())}\n")
md_lines.append(f"![PCA plot]({PNG_OUT.name})\n")

md_lines.append("## Cluster Highlights\n")
for c in sorted(df_clean['cluster'].unique()):
    subset = df_clean[df_clean['cluster'] == c]
    # Pick the session with highest realized volatility as representative
    top_vol = subset.loc[subset['realized_volatility'].idxmax()]
    md_lines.append(f"### Cluster {c}\n")
    md_lines.append(f"* Sessions: {len(subset)}\n")
    md_lines.append(f"* Representative (highest volatility): {top_vol['date']} {top_vol['symbol']}\n")
    md_lines.append("| Metric | Value |\n|---|---|\n")
    for m in metrics:
        md_lines.append(f"| {m} | {top_vol[m]:.4f} |\n")
    md_lines.append("\n")

MD_OUT.write_text("\n".join(md_lines))
print(f"Markdown report written to {MD_OUT}")

if __name__ == "__main__":
    # script entry point – all work is done on import
    pass
