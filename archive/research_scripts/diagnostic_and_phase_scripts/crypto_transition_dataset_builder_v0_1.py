#!/usr/bin/env python3
import requests
import json
import time
import math
import csv
from pathlib import Path
from datetime import datetime

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

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
        time.sleep(0.1)
        
    return all_klines

def build_dataset():
    start_ms = 1784310420000
    end_ms = 1786903000000
    
    symbol = "BTCUSDT"
    klines = fetch_binance_klines(symbol, "1m", start_ms, end_ms)
    
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
        
    print(f"Fetched {len(formatted)} 1m observations for {symbol}.")
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
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
            states.append((i, state_snapshot))
            
    print(f"Generated {len(states)} valid state snapshots.")
    
    out_file = DATA_DIR / f"transition_discovery_v0_1_{symbol}.csv"
    horizons = [60, 120, 300]
    DELTA = 300
    
    print("Computing transitions and behavioural targets...")
    with open(out_file, "w", newline="") as f:
        writer = csv.writer(f)
        header = [
            "timestamp", 
            "volatility_1m_std_24h", 
            "upside_excursion_24h", 
            "downside_excursion_24h", 
            "trend_dir",
            "past_volatility_24h",
            "past_trend_dir",
            "delta_volatility",
            "delta_upside",
            "delta_downside"
        ]
        
        for h in horizons:
            header.extend([
                f"reversal_{h}m",
                f"vol_expansion_{h}m",
                f"excursion_exceeded_{h}m"
            ])
            
        writer.writerow(header)
        
        valid_rows = 0
        
        for k in range(DELTA, len(states)):
            idx_current, current_state = states[k]
            idx_past, past_state = states[k - DELTA]
            
            # Ensure continuity of indices just to be safe (delta should be exactly 300 bars)
            if idx_current - idx_past != DELTA:
                continue
                
            if idx_current + max(horizons) < len(formatted):
                delta_volatility = current_state.volatility_1m_std_24h - past_state.volatility_1m_std_24h
                delta_upside = current_state.upside_excursion_24h - past_state.upside_excursion_24h
                delta_downside = current_state.downside_excursion_24h - past_state.downside_excursion_24h
                
                row = [
                    current_state.timestamp,
                    current_state.volatility_1m_std_24h,
                    current_state.upside_excursion_24h,
                    current_state.downside_excursion_24h,
                    current_state.trend_dir,
                    past_state.volatility_1m_std_24h,
                    past_state.trend_dir,
                    delta_volatility,
                    delta_upside,
                    delta_downside
                ]
                
                start_px = formatted[idx_current]["close"]
                current_extreme = max(abs(current_state.upside_excursion_24h), abs(current_state.downside_excursion_24h))
                current_direction = current_state.trend_dir
                
                for h in horizons:
                    future_path = formatted[idx_current : idx_current + h + 1]
                    end_px = future_path[-1]["close"]
                    
                    # 1. Direction Reversal
                    future_direction = 1 if end_px > start_px else (-1 if end_px < start_px else 0)
                    reversal = 1 if (future_direction == -current_direction and future_direction != 0 and current_direction != 0) else 0
                    
                    # 2. Volatility Expansion
                    closes = [b["close"] for b in future_path]
                    rets = [math.log(closes[i] / closes[i-1]) for i in range(1, len(closes))]
                    if len(rets) > 0:
                        mean_ret = sum(rets) / len(rets)
                        var = sum((x - mean_ret)**2 for x in rets) / len(rets)
                        future_vol = math.sqrt(var)
                    else:
                        future_vol = 0
                    
                    vol_expansion = 1 if future_vol > current_state.volatility_1m_std_24h else 0
                    
                    # 3. Excursion Stability
                    highs = [b["high"] for b in future_path]
                    lows = [b["low"] for b in future_path]
                    future_upside = (max(highs) - start_px) / start_px if start_px > 0 else 0
                    future_downside = (min(lows) - start_px) / start_px if start_px > 0 else 0
                    future_extreme = max(abs(future_upside), abs(future_downside))
                    
                    excursion_exceeded = 1 if future_extreme > current_extreme else 0
                    
                    row.extend([reversal, vol_expansion, excursion_exceeded])
                    
                writer.writerow(row)
                valid_rows += 1
                
    print(f"Dataset successfully exported to {out_file} with {valid_rows} transition observations.")

if __name__ == "__main__":
    build_dataset()
