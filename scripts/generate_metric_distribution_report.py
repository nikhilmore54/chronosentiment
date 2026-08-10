"""Generate a metric distribution report for Q1 session catalog.

Outputs:
- archive/research_outputs/metric_distribution_report.md (markdown report)
- PNG histograms for each metric (saved in the same directory)
"""

import json
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from pathlib import Path
# import scipy.stats.skew replaced with pandas Series.skew

METRICS = [
    "realized_volatility",
    "trend_strength",
    "gap_pct",
    "session_range_pct",
    "net_return_pct",
]

def load_catalog(path: Path) -> pd.DataFrame:
    data = json.loads(path.read_text())
    df = pd.DataFrame(data)
    # Coerce numeric columns
    for col in METRICS:
        df[col] = pd.to_numeric(df[col], errors="coerce")
    return df

def compute_stats(series: pd.Series):
    series = series.dropna()
    return {
        "count": int(series.count()),
        "min": float(series.min()),
        "max": float(series.max()),
        "mean": float(series.mean()),
        "median": float(series.median()),
        "std": float(series.std()),
        "skew": float(series.skew()),
        "p5": float(series.quantile(0.05)),
        "p25": float(series.quantile(0.25)),
        "p75": float(series.quantile(0.75)),
        "p95": float(series.quantile(0.95)),
        "outliers": int(((series < series.quantile(0.05)) | (series > series.quantile(0.95))).sum()),
    }

def plot_histogram(series: pd.Series, metric: str, out_dir: Path):
    plt.figure(figsize=(6, 4))
    series = series.dropna()
    plt.hist(series, bins=30, color="#4A90E2", edgecolor="black", alpha=0.7)
    plt.title(f"Histogram of {metric}")
    plt.xlabel(metric)
    plt.ylabel("Count")
    plt.grid(True, linestyle="--", alpha=0.5)
    out_path = out_dir / f"hist_{metric}.png"
    plt.tight_layout()
    plt.savefig(out_path, dpi=150)
    plt.close()
    return out_path.name

def main():
    catalog_path = Path("phase1/analysis/coordinate_audit/session_catalog_q1.json")
    out_md = Path("archive/research_outputs/metric_distribution_report.md")
    out_dir = out_md.parent
    df = load_catalog(catalog_path)

    md_parts = ["# Q1 Metric Distribution Report\n"]
    md_parts.append(f"Generated from `{catalog_path.name}` – **{len(df)} sessions**.\n")

    for metric in METRICS:
        series = df[metric]
        stats = compute_stats(series)
        img_name = plot_histogram(series, metric, out_dir)
        md_parts.append(f"## {metric.replace('_', ' ').title()}\n")
        md_parts.append(f"![{metric} histogram]({img_name})\n")
        md_parts.append("| Statistic | Value |\n|---|---|\n")
        for key, val in stats.items():
            md_parts.append(f"| {key} | {val:.4f} |\n")
        md_parts.append("\n---\n")

    out_md.write_text("\n".join(md_parts))
    print(f"Report written to {out_md}")

if __name__ == "__main__":
    main()
