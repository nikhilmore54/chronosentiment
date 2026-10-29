"""Generate a markdown summary of the session catalog.

Reads `phase1/analysis/coordinate_audit/session_catalog.json` and produces
`archive/research_outputs/session_catalog_summary.md` with ranked tables for various metrics.
"""

import json
import pandas as pd
from pathlib import Path

def load_catalog(path: Path) -> pd.DataFrame:
    data = json.loads(path.read_text())
    return pd.DataFrame(data)

def rank_series(series: pd.Series, ascending: bool = False) -> pd.Series:
    # Return ranking positions (1 = best) based on sort order
    return series.rank(method="first", ascending=ascending).astype(int)

def top_bottom(df: pd.DataFrame, column: str, n: int = 5):
    top = df.nlargest(n, column)
    bottom = df.nsmallest(n, column)
    return top, bottom

def format_table(df: pd.DataFrame, columns: list) -> str:
    # Use markdown table format
    header = "| " + " | ".join(columns) + " |"
    sep = "|" + "---|" * len(columns)
    rows = []
    for _, row in df.iterrows():
        rows.append("| " + " | ".join(str(row[col]) for col in columns) + " |")
    return "\n".join([header, sep] + rows)

def main():
    catalog_path = Path("phase1/analysis/coordinate_audit/session_catalog.json")
    out_path = Path("archive/research_outputs/session_catalog_summary.md")
    df = load_catalog(catalog_path)

    # Ensure numeric columns are proper floats (some may be None)
    numeric_cols = ["gap_pct", "realized_volatility", "trend_strength",
                    "session_range_pct", "net_return_pct"]
    for col in numeric_cols:
        df[col] = pd.to_numeric(df[col], errors="coerce")

    # Prepare markdown content
    md_parts = []
    md_parts.append("# Session Catalog Summary\n")
    md_parts.append("Generated from `session_catalog.json` – **22 sessions**.\n")

    # Top/Bottom Volatility
    top_vol, bot_vol = top_bottom(df, "realized_volatility")
    md_parts.append("## Realized Volatility – Top 5 Sessions")
    md_parts.append(format_table(top_vol, ["date", "symbol", "realized_volatility"]))
    md_parts.append("\n## Realized Volatility – Bottom 5 Sessions")
    md_parts.append(format_table(bot_vol, ["date", "symbol", "realized_volatility"]))

    # Top/Bottom Trend Strength
    top_trend, bot_trend = top_bottom(df, "trend_strength")
    md_parts.append("\n## Trend Strength – Top 5 Sessions")
    md_parts.append(format_table(top_trend, ["date", "symbol", "trend_strength"]))
    md_parts.append("\n## Trend Strength – Bottom 5 Sessions")
    md_parts.append(format_table(bot_trend, ["date", "symbol", "trend_strength"]))

    # Largest positive/negative gaps (ignore nulls)
    gap_df = df.dropna(subset=["gap_pct"])
    top_gap = gap_df.nlargest(5, "gap_pct")
    bot_gap = gap_df.nsmallest(5, "gap_pct")
    md_parts.append("\n## Gap Percentage – Largest Positive Gaps (Top 5)")
    md_parts.append(format_table(top_gap, ["date", "symbol", "gap_pct"]))
    md_parts.append("\n## Gap Percentage – Largest Negative Gaps (Bottom 5)")
    md_parts.append(format_table(bot_gap, ["date", "symbol", "gap_pct"]))

    # Additional useful metrics (range and net return)
    md_parts.append("\n## Session Range Percentage – Top 5 Sessions")
    md_parts.append(format_table(df.nlargest(5, "session_range_pct"),
                                ["date", "symbol", "session_range_pct"]))
    md_parts.append("\n## Net Return Percentage – Top 5 Sessions")
    md_parts.append(format_table(df.nlargest(5, "net_return_pct"),
                                ["date", "symbol", "net_return_pct"]))

    out_path.write_text("\n".join(md_parts))
    print(f"Summary written to {out_path}")

if __name__ == "__main__":
    main()
