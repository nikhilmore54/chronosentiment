import json
import yfinance as yf
import pandas as pd
import sys
from datetime import datetime, timedelta

def generate_real_validation_data(symbols, interval, days):
    period = f"{days}d"
    
    # Fetch data for all symbols
    dfs = {}
    for sym in symbols:
        df = yf.download(sym, period=period, interval=interval, auto_adjust=True, progress=False)
        if not df.empty:
            if isinstance(df.columns, pd.MultiIndex):
                df.columns = df.columns.get_level_values(0)
            dfs[sym] = df
            
    if not dfs:
        print("No data fetched", file=sys.stderr)
        return

    # Find common timestamps
    common_ts = None
    for sym, df in dfs.items():
        ts = set(df.index)
        if common_ts is None:
            common_ts = ts
        else:
            common_ts = common_ts.intersection(ts)
            
    sorted_ts = sorted(list(common_ts))
    
    for ts in sorted_ts:
        batch = []
        for sym in symbols:
            row = dfs[sym].loc[ts]
            batch.append({
                "symbol": sym,
                "timestamp": int(ts.timestamp()),
                "open": float(row["Open"]),
                "high": float(row["High"]),
                "low": float(row["Low"]),
                "close": float(row["Close"]),
                "volume": float(row.get("Volume", 0))
            })
        print(json.dumps(batch))

if __name__ == "__main__":
    symbols = ["BTC-USD", "ETH-USD", "SOL-USD", "RELIANCE.NS", "TCS.NS", "HDFCBANK.NS"]
    generate_real_validation_data(symbols, "5m", 10)
