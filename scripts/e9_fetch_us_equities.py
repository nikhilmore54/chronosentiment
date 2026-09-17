#!/usr/bin/env python3
import requests
import json
import time
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "e9" / "us_equities"

def fetch_yahoo_1m(symbol, range_str="5d"):
    url = f"https://query2.finance.yahoo.com/v8/finance/chart/{symbol}?range={range_str}&interval=1m"
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
    }
    print(f"Fetching {symbol} 1m data from Yahoo...")
    resp = requests.get(url, headers=headers)
    if resp.status_code != 200:
        print(f"Error fetching {symbol}: {resp.status_code} {resp.text}")
        return None
        
    data = resp.json()
    if not data or "chart" not in data or not data["chart"]["result"]:
        print(f"No data for {symbol}")
        return None
        
    result = data["chart"]["result"][0]
    timestamps = result.get("timestamp", [])
    quote = result["indicators"]["quote"][0]
    
    opens = quote.get("open", [])
    highs = quote.get("high", [])
    lows = quote.get("low", [])
    closes = quote.get("close", [])
    volumes = quote.get("volume", [])
    
    formatted = []
    for i in range(len(timestamps)):
        if closes[i] is not None:
            formatted.append({
                "timestamp": timestamps[i],
                "open": opens[i],
                "high": highs[i],
                "low": lows[i],
                "close": closes[i],
                "volume": volumes[i]
            })
            
    return formatted

def main():
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    symbols = ["SPY", "QQQ", "NVDA", "AAPL", "MSFT"]
    
    for sym in symbols:
        bars = fetch_yahoo_1m(sym, "5d")
        if bars:
            out_file = DATA_DIR / f"{sym}.json"
            with open(out_file, "w") as f:
                json.dump(bars, f)
            print(f"Saved {len(bars)} bars to {out_file}")
        time.sleep(1)

if __name__ == "__main__":
    main()
