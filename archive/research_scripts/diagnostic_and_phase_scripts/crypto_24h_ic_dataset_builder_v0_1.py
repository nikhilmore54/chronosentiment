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
    # Use EXACT start/end ms to perfectly align with previous v0 dataset
    # 1784310420000 is 1440m before the first emitted state 1784396820000
    # 1786903000000 is just after the last 300m forward label for the last emitted state
    start_ms = 1784310420000
    end_ms = 1786903000000
    
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
    
    out_file = DATA_DIR / f"ic_discovery_v0.1_{symbol}.csv"
    horizons = [15, 30, 60, 120, 300]
    
    print("Appending forward labels and exporting...")
    with open(out_file, "w", newline="") as f:
        writer = csv.writer(f)
        header = [
            "timestamp", 
            "volatility_1m_std_24h", 
            "upside_excursion_24h", 
            "downside_excursion_24h", 
            "trend_dir",
            "trend_return_15m",
            "trend_return_60m",
            "trend_return_240m",
            "volatility_std_60m",
            "volatility_std_240m",
            "volatility_ratio_60m_24h",
            "volatility_ratio_240m_24h",
            "persistence_60m",
            "persistence_240m"
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
                    state.trend_dir,
                    state.trend_return_15m,
                    state.trend_return_60m,
                    state.trend_return_240m,
                    state.volatility_std_60m,
                    state.volatility_std_240m,
                    state.volatility_ratio_60m_24h,
                    state.volatility_ratio_240m_24h,
                    state.persistence_60m,
                    state.persistence_240m
                ]
                current_px = formatted[idx]["close"]
                
                for h in horizons:
                    future_px = formatted[idx + h]["close"]
                    # Log return for targets to be consistent with features
                    ret = math.log(future_px / current_px)
                    row.append(ret)
                    
                writer.writerow(row)
                valid_rows += 1
                
    print(f"Dataset successfully exported to {out_file} with {valid_rows} labelled observations.")

if __name__ == "__main__":
    build_dataset()
