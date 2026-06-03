"""Audit trend strength for selected sessions.

Loads the canonical 1‑minute CSV for each specified date/symbol,
computes:
  - net_return_pct
  - average intraday range pct
  - trend_strength (net_return_pct / avg_range_pct)

Prints a concise table for manual inspection.
"""

import pandas as pd
from pathlib import Path

# Import metric helpers from session_metrics
from session_metrics import (
    load_canonical,
    net_return_pct,
    avg_range_pct,
    trend_strength,
)

SESSIONS = [
    ("2025-01-02", "NIFTY"),
    ("2025-01-06", "NIFTY"),
    ("2025-01-07", "NIFTY"),
]

def audit():
    rows = []
    for date_str, symbol in SESSIONS:
        csv_path = Path("historical_capture/batch") / date_str / "canonical" / f"{symbol}_1m.csv"
        if not csv_path.is_file():
            print(f"[WARN] Missing CSV for {date_str} {symbol}: {csv_path}")
            continue
        df = load_canonical(csv_path)
        ohlc = {
            "open": float(df.iloc[0]["open"]),
            "close": float(df.iloc[-1]["close"]),
        }
        net_ret = net_return_pct(ohlc["open"], ohlc["close"])
        avg_rng = avg_range_pct(df, ohlc["open"])
        tr_strength = trend_strength(df, ohlc["open"], ohlc["close"])
        rows.append({
            "date": date_str,
            "symbol": symbol,
            "net_return_pct": net_ret,
            "avg_range_pct": avg_rng,
            "trend_strength": tr_strength,
        })
    # Print markdown table
    if rows:
        cols = ["date", "symbol", "net_return_pct", "avg_range_pct", "trend_strength"]
        header = "| " + " | ".join(cols) + " |"
        sep = "|" + "---|" * len(cols)
        print(header)
        print(sep)
        for r in rows:
            print("| " + " | ".join(str(r[col]) for col in cols) + " |")

if __name__ == "__main__":
    audit()
