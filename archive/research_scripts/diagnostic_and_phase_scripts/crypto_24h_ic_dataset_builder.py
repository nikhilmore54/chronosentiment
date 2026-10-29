#!/usr/bin/env python3
import requests
import json
import time
import math
import csv
from pathlib import Path
from datetime import datetime, timedelta

from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "crypto_24h"
DATA_DIR.mkdir(parents=True, exist_ok=True)

def fetch_binance_klines(symbol, interval, start_time_ms, end_time_ms):
    url = "https://api.binance.com/api/v3/klines"
    all_klines = []
    current_start = start_time_ms
    
    print(f"Fetching {symbol} {interval} from {datetime.fromtimestamp(start_time_ms/1000)} to {datetime.fromtimestamp(end_time_ms/1000)}")
    
    while current_start < end_time_ms:
        params = {
            "symbol": symbol,
            "interval": interval,
            "startTime": current_start,
            "endTime": end_time_ms,
            "limit": 1000
        }
        resp = requests.get(url, params=params)
        if resp.status_code != 200:
            print(f"Error fetching data: {resp.text}")
            break
            
        data = resp.json()
        if not data:
            break
            
        all_klines.extend(data)
        current_start = data[-1][0] + 1
        time.sleep(0.1) # Rate limit protection
        
    return all_klines

def build_dataset():
    # E9 used the last 30 days (roughly ~Aug 17 to Sep 17).
    # To strictly avoid E9 data, we will fetch the 30 days prior to that.
    # ~July 18 to ~Aug 17
    end_dt = datetime.now() - timedelta(days=32)
    start_dt = end_dt - timedelta(days=30)
    
    end_ms = int(end_dt.timestamp() * 1000)
    start_ms = int(start_dt.timestamp() * 1000)
    
    symbol = "BTCUSDT"
    klines = fetch_binance_klines(symbol, "1m", start_ms, end_ms)
    
    formatted = []
    for d in klines:
        formatted.append({
            "timestamp": int(d[0]/1000) * 1000, # ms timestamp
            "open": float(d[1]),
            "high": float(d[2]),
            "low": float(d[3]),
            "close": float(d[4]),
            "volume": float(d[5])
        })
        
    print(f"Fetched {len(formatted)} 1m observations for {symbol}.")
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    # Store states with their corresponding t_index so we can look up future prices
    states = []
    
    print("Computing continuous rolling state (with 24h warm-up)...")
    for i, bar in enumerate(formatted):
        added = obs_store.add(
            bar["timestamp"], 
            bar["open"], 
            bar["high"], 
            bar["low"], 
            bar["close"], 
            bar["volume"]
        )
        if not added:
            continue
            
        state_snapshot = state_engine.update(obs_store)
        if state_snapshot:
            # We save the index 'i' to find the close price at i+H
            states.append((i, state_snapshot))
            
    print(f"Generated {len(states)} valid state snapshots.")
    
    out_file = DATA_DIR / f"ic_discovery_{symbol}.csv"
    horizons = [15, 30, 60, 120, 300]
    
    print("Appending forward labels and exporting...")
    with open(out_file, "w", newline="") as f:
        writer = csv.writer(f)
        header = [
            "timestamp", 
            "volatility_1m_std_24h", 
            "upside_excursion_24h", 
            "downside_excursion_24h", 
            "trend_dir"
        ]
        for h in horizons:
            header.append(f"ret_{h}m")
        writer.writerow(header)
        
        valid_rows = 0
        for idx, state in states:
            # Check if all horizons are available
            if idx + max(horizons) < len(formatted):
                row = [
                    state.timestamp,
                    state.volatility_1m_std_24h,
                    state.upside_excursion_24h,
                    state.downside_excursion_24h,
                    state.trend_dir
                ]
                current_px = formatted[idx]["close"]
                
                for h in horizons:
                    future_px = formatted[idx + h]["close"]
                    ret = (future_px - current_px) / current_px
                    row.append(ret)
                    
                writer.writerow(row)
                valid_rows += 1
                
    print(f"Dataset successfully exported to {out_file} with {valid_rows} labelled observations.")

if __name__ == "__main__":
    build_dataset()
