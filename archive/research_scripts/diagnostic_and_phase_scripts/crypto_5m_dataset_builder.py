#!/usr/bin/env python3
import requests
import time
import csv
from pathlib import Path
from datetime import datetime

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_5m.observation_store import ObservationStore
from apps.crypto_5m.rolling_state_engine import RollingStateEngine

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "crypto_5m"
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
        time.sleep(0.1)
        
    return all_klines

def build_dataset():
    # Exact timestamps from the original 1m discovery run
    start_ms = 1784310420000
    end_ms = 1786903000000
    
    symbol = "BTCUSDT"
    klines = fetch_binance_klines(symbol, "5m", start_ms, end_ms)
    
    formatted = []
    for d in klines:
        formatted.append({
            "timestamp": int(d[0]/1000) * 1000,
            "open": float(d[1]),
            "high": float(d[2]),
            "low": float(d[3]),
            "close": float(d[4]),
            "volume": float(d[5])
        })
        
    print(f"Fetched {len(formatted)} 5m observations for {symbol}.")
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    states = []
    
    print("Computing continuous rolling state (with 288-bar / 24h warm-up)...")
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
            states.append((i, state_snapshot))
            
    print(f"Generated {len(states)} valid state snapshots.")
    
    out_file = DATA_DIR / f"ic_discovery_5m_{symbol}.csv"
    horizons_bars = [3, 6, 12, 24, 60]  # 15m, 30m, 60m, 120m, 300m
    horizon_names = [15, 30, 60, 120, 300]
    
    print("Appending forward labels and exporting...")
    with open(out_file, "w", newline="") as f:
        writer = csv.writer(f)
        header = [
            "timestamp", 
            "volatility_5m_std_24h", 
            "upside_excursion_24h", 
            "downside_excursion_24h", 
            "trend_dir"
        ]
        for h in horizon_names:
            header.append(f"ret_{h}m")
        writer.writerow(header)
        
        valid_rows = 0
        for idx, state in states:
            if idx + max(horizons_bars) < len(formatted):
                row = [
                    state.timestamp,
                    state.volatility_5m_std_24h,
                    state.upside_excursion_24h,
                    state.downside_excursion_24h,
                    state.trend_dir
                ]
                current_px = formatted[idx]["close"]
                
                for h_bars in horizons_bars:
                    future_px = formatted[idx + h_bars]["close"]
                    ret = (future_px - current_px) / current_px
                    row.append(ret)
                    
                writer.writerow(row)
                valid_rows += 1
                
    print(f"Dataset successfully exported to {out_file} with {valid_rows} labelled observations.")

if __name__ == "__main__":
    build_dataset()
