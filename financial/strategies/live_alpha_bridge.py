import yfinance as yf
import pandas as pd
import time
import requests
import os
from fastapi import FastAPI
from typing import List

# --- ChronoSentiment Live Alpha Bridge ---

app = FastAPI(title="ChronoSentiment Recommendation API")

# Storage for live executable opportunities
recommendations = []

def load_tickers(file_path="tickers.txt"):
    if not os.path.exists(file_path):
        return ["RELIANCE.NS", "TCS.NS", "INFY.NS", "HDFCBANK.NS", "ICICIBANK.NS"]
    with open(file_path, "r") as f:
        return [line.strip() for line in f if line.strip()]

def fetch_live_data(tickers):
    """
    Fetches the last 50 minutes of 1m data for the specified tickers.
    """
    data = {}
    for ticker in tickers:
        try:
            print(f"Fetching {ticker}...")
            df = yf.download(ticker, period="1d", interval="1m", progress=False)
            if not df.empty:
                # Keep last 50 candles for feature building
                data[ticker] = df.tail(50)
        except Exception as e:
            print(f"Error fetching {ticker}: {e}")
    return data

def apply_guardrails(df):
    """
    Institutional production safeguards.
    """
    if df.empty or len(df) < 5:
        return False
        
    last_close = df["Close"].iloc[-1]
    last_vol = df["Volume"].iloc[-1]
    
    # 1. Volume Filter (Basic Liquidity)
    if last_vol < 1000:
        return False
        
    # 2. Spread Proxy (Friction Check)
    spread = (df["High"] - df["Low"]).iloc[-1]
    if spread / last_close > 0.005: # 50bps spread gate
        return False
        
    return True

@app.get("/recommendations")
def get_recommendations():
    return recommendations

@app.get("/health")
def health():
    return {"status": "online", "engine": "ChronoSentiment 2.0 (Phase C.1.6b)"}

# Internal loop logic would go here - for now, we've provided the structural skeleton
# as requested in the Deployment Path.

if __name__ == "__main__":
    import uvicorn
    # In production, this would be a separate process from the fetcher loop
    uvicorn.run(app, host="0.0.0.0", port=8000)
