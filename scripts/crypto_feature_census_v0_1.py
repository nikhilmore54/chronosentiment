#!/usr/bin/env python3
import requests
import json
import time
import math
import csv
import numpy as np
import pandas as pd
from pathlib import Path
from datetime import datetime
from scipy.stats import spearmanr

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "crypto_24h"
REPORT_FILE = DATA_DIR / "crypto_feature_census_report.md"

def fetch_binance_klines(symbol, interval, start_time_ms, end_time_ms):
    url = "https://api.binance.com/api/v3/klines"
    all_klines = []
    current_start = start_time_ms
    
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
            break
            
        data = resp.json()
        if not data:
            break
            
        all_klines.extend(data)
        current_start = data[-1][0] + 1
        time.sleep(0.1)
        
    return all_klines

def build_census():
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
        
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    data_rows = []
    
    for bar in formatted:
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
        
        # Only compute if the base state engine is warmed up (1440 candles)
        if state_snapshot:
            w = obs_store.get_recent(1440)
            recent_60 = w[-60:]
            
            # Volume Topology
            vol_60m = sum(c['volume'] for c in recent_60)
            vol_24h = sum(c['volume'] for c in w)
            rvol_60m_24h = vol_60m / (vol_24h / 24) if vol_24h > 0 else 1.0
            
            vol_last_30m = sum(c['volume'] for c in w[-30:])
            vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
            volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
            
            vols_60 = np.array([c['volume'] for c in recent_60])
            rets_60 = np.array([(c['close'] - c['open'])/c['open'] if c['open'] > 0 else 0 for c in recent_60])
            if np.std(vols_60) > 0 and np.std(rets_60) > 0:
                price_volume_alignment_60m = np.corrcoef(vols_60, rets_60)[0, 1]
            else:
                price_volume_alignment_60m = 0.0
                
            # Candle Morphology
            ranges_60 = [(c['high'] - c['low'])/c['open'] if c['open'] > 0 else 0 for c in recent_60]
            avg_range_pct_60m = sum(ranges_60) / 60
            
            upper_wicks = [c['high'] - max(c['open'], c['close']) for c in recent_60]
            lower_wicks = [min(c['open'], c['close']) - c['low'] for c in recent_60]
            total_upper = sum(upper_wicks)
            total_lower = sum(lower_wicks)
            wick_asymmetry_60m = total_upper / (total_upper + total_lower) if (total_upper + total_lower) > 0 else 0.5
            
            highest = max(c['high'] for c in recent_60)
            lowest = min(c['low'] for c in recent_60)
            close_px = recent_60[-1]['close']
            close_location_60m = (close_px - lowest) / (highest - lowest) if highest > lowest else 0.5
            
            net_ret = abs(recent_60[-1]['close'] - recent_60[0]['open']) / recent_60[0]['open'] if recent_60[0]['open'] > 0 else 0
            trend_strength_60m = net_ret / avg_range_pct_60m if avg_range_pct_60m > 0 else 0
            
            row = {
                # 13 Base features
                'volatility_1m_std_24h': state_snapshot.volatility_1m_std_24h,
                'upside_excursion_24h': state_snapshot.upside_excursion_24h,
                'downside_excursion_24h': state_snapshot.downside_excursion_24h,
                'trend_dir': state_snapshot.trend_dir,
                'trend_return_15m': state_snapshot.trend_return_15m,
                'trend_return_60m': state_snapshot.trend_return_60m,
                'trend_return_240m': state_snapshot.trend_return_240m,
                'volatility_std_60m': state_snapshot.volatility_std_60m,
                'volatility_std_240m': state_snapshot.volatility_std_240m,
                'volatility_ratio_60m_24h': state_snapshot.volatility_ratio_60m_24h,
                'volatility_ratio_240m_24h': state_snapshot.volatility_ratio_240m_24h,
                'persistence_60m': state_snapshot.persistence_60m,
                'persistence_240m': state_snapshot.persistence_240m,
                
                # 7 New features
                'rvol_60m_24h': rvol_60m_24h,
                'volume_acceleration_60m': volume_acceleration_60m,
                'price_volume_alignment_60m': price_volume_alignment_60m,
                'avg_range_pct_60m': avg_range_pct_60m,
                'wick_asymmetry_60m': wick_asymmetry_60m,
                'close_location_60m': close_location_60m,
                'trend_strength_60m': trend_strength_60m
            }
            data_rows.append(row)
            
    df = pd.DataFrame(data_rows)
    
    # Restrict to Train split (first 60%)
    n_train = int(len(df) * 0.6)
    train_df = df.iloc[:n_train].copy()
    
    base_features = [
        'volatility_1m_std_24h', 'upside_excursion_24h', 'downside_excursion_24h', 'trend_dir',
        'trend_return_15m', 'trend_return_60m', 'trend_return_240m', 'volatility_std_60m',
        'volatility_std_240m', 'volatility_ratio_60m_24h', 'volatility_ratio_240m_24h',
        'persistence_60m', 'persistence_240m'
    ]
    
    new_features = [
        'rvol_60m_24h', 'volume_acceleration_60m', 'price_volume_alignment_60m',
        'avg_range_pct_60m', 'wick_asymmetry_60m', 'close_location_60m', 'trend_strength_60m'
    ]
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Crypto Feature Census v0.1\n")
        f.write("## Stage A: Orthogonality Audit (Train Split)\n\n")
        
        f.write("### 1. Descriptive Statistics of New Dimensions\n")
        f.write("| Feature | Mean | Std Dev | Min | Max | Missing Rate |\n")
        f.write("|---|---|---|---|---|---|\n")
        for col in new_features:
            mean = train_df[col].mean()
            std = train_df[col].std()
            cmin = train_df[col].min()
            cmax = train_df[col].max()
            missing = train_df[col].isna().mean()
            f.write(f"| `{col}` | {mean:.4f} | {std:.4f} | {cmin:.4f} | {cmax:.4f} | {missing*100:.1f}% |\n")
        
        f.write("\n### 2. Cross-Correlation Among New Features (Pearson)\n")
        f.write("| Feature | " + " | ".join([f"`{c}`" for c in new_features]) + " |\n")
        f.write("|---" * (len(new_features)+1) + "|\n")
        for col1 in new_features:
            row_str = f"| `{col1}` |"
            for col2 in new_features:
                corr = train_df[col1].corr(train_df[col2])
                row_str += f" {corr:.2f} |"
            f.write(row_str + "\n")
            
        f.write("\n### 3. Pearson Correlation vs Existing 13-field State\n")
        f.write("| New Feature | " + " | ".join([f"`{c}`" for c in base_features]) + " |\n")
        f.write("|---" * (len(base_features)+1) + "|\n")
        for col1 in new_features:
            row_str = f"| `{col1}` |"
            for col2 in base_features:
                corr = train_df[col1].corr(train_df[col2])
                row_str += f" {corr:.2f} |"
            f.write(row_str + "\n")
            
        f.write("\n### 4. Spearman (Rank) Correlation vs Existing 13-field State\n")
        f.write("| New Feature | " + " | ".join([f"`{c}`" for c in base_features]) + " |\n")
        f.write("|---" * (len(base_features)+1) + "|\n")
        for col1 in new_features:
            row_str = f"| `{col1}` |"
            for col2 in base_features:
                corr, _ = spearmanr(train_df[col1], train_df[col2])
                row_str += f" {corr:.2f} |"
            f.write(row_str + "\n")
            
    print(f"Feature census complete. Report at {REPORT_FILE}")

if __name__ == "__main__":
    build_census()
