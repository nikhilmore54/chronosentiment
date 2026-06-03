"""Generate a markdown summary for the Q1 session catalog.

Reads `phase1/analysis/coordinate_audit/session_catalog_q1.json` and produces
`session_catalog_q1_summary.md` with ranked tables for the same metrics as the
pilot summary.
"""

import json
import pandas as pd
from pathlib import Path

def load_catalog(path: Path) -> pd.DataFrame:
    data = json.loads(path.read_text())
    return pd.DataFrame(data)

def top_bottom(df: pd.DataFrame, column: str, n: int = 5):
    return df.nlargest(n, column), df.nsmallest(n, column)

def format_table(df: pd.DataFrame, columns: list) -> str:
    header = "| " + " | ".join(columns) + " |"
    sep = "|" + "---|" * len(columns)
    rows = []
    for _, row in df.iterrows():
        rows.append("| " + " | ".join(str(row[col]) for col in columns) + " |")
    return "\n".join([header, sep] + rows)

def main():
    catalog_path = Path("phase1/analysis/coordinate_audit/session_catalog_q1.json")
    out_path = Path("session_catalog_q1_summary.md")
    df = load_catalog(catalog_path)
    # Ensure numeric columns are floats
    for col in ["gap_pct", "realized_volatility", "trend_strength",
                "session_range_pct", "net_return_pct"]:
        df[col] = pd.to_numeric(df[col], errors="coerce")
    parts = []
    parts.append("# Q1 Session Catalog Summary\n")
    parts.append(f"Generated from `{catalog_path.name}` – **{len(df)} sessions**.\n")
    # Volatility
    top_vol, bot_vol = top_bottom(df, "realized_volatility")
    parts.append("## Realized Volatility – Top 5 Sessions")
    parts.append(format_table(top_vol, ["date", "symbol", "realized_volatility"]))
    parts.append("\n## Realized Volatility – Bottom 5 Sessions")
    parts.append(format_table(bot_vol, ["date", "symbol", "realized_volatility"]))
    # Trend
    top_trend, bot_trend = top_bottom(df, "trend_strength")
    parts.append("\n## Trend Strength – Top 5 Sessions")
    parts.append(format_table(top_trend, ["date", "symbol", "trend_strength"]))
    parts.append("\n## Trend Strength – Bottom 5 Sessions")
    parts.append(format_table(bot_trend, ["date", "symbol", "trend_strength"]))
    # Gap
    gap_df = df.dropna(subset=["gap_pct"])
    top_gap = gap_df.nlargest(5, "gap_pct")
    bot_gap = gap_df.nsmallest(5, "gap_pct")
    parts.append("\n## Gap Percentage – Largest Positive Gaps (Top 5)")
    parts.append(format_table(top_gap, ["date", "symbol", "gap_pct"]))
    parts.append("\n## Gap Percentage – Largest Negative Gaps (Bottom 5)")
    parts.append(format_table(bot_gap, ["date", "symbol", "gap_pct"]))
    # Additional useful metrics
    parts.append("\n## Session Range Percentage – Top 5 Sessions")
    parts.append(format_table(df.nlargest(5, "session_range_pct"),
                               ["date", "symbol", "session_range_pct"]))
    parts.append("\n## Net Return Percentage – Top 5 Sessions")
    parts.append(format_table(df.nlargest(5, "net_return_pct"),
                               ["date", "symbol", "net_return_pct"]))
    out_path.write_text("\n".join(parts))
    print(f"Summary written to {out_path}")

if __name__ == "__main__":
    main()
