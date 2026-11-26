#!/usr/bin/env python3
import requests
import json
import time
import math
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
REPORT_FILE = DATA_DIR / "crypto_state_characterisation_v0_1.md"

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

def build_state_dataframe():
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
        
        if state_snapshot:
            w = obs_store.get_recent(1440)
            recent_60 = w[-60:]
            
            # Feature 1: Volume Acceleration 60m
            vol_last_30m = sum(c['volume'] for c in w[-30:])
            vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
            volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
            
            # Feature 2: Price-Volume Alignment 60m
            vols_60 = np.array([c['volume'] for c in recent_60])
            rets_60 = np.array([(c['close'] - c['open'])/c['open'] if c['open'] > 0 else 0 for c in recent_60])
            if np.std(vols_60) > 0 and np.std(rets_60) > 0:
                price_volume_alignment_60m = np.corrcoef(vols_60, rets_60)[0, 1]
            else:
                price_volume_alignment_60m = 0.0
                
            # Feature 3: Wick Asymmetry 60m
            upper_wicks = [c['high'] - max(c['open'], c['close']) for c in recent_60]
            lower_wicks = [min(c['open'], c['close']) - c['low'] for c in recent_60]
            total_upper = sum(upper_wicks)
            total_lower = sum(lower_wicks)
            wick_asymmetry_60m = total_upper / (total_upper + total_lower) if (total_upper + total_lower) > 0 else 0.5
            
            row = {
                'timestamp': state_snapshot.timestamp,
                'volatility_1m_std_24h': state_snapshot.volatility_1m_std_24h,
                'trend_dir': state_snapshot.trend_dir,
                'persistence_60m': state_snapshot.persistence_60m,
                'persistence_240m': state_snapshot.persistence_240m,
                'volume_acceleration_60m': volume_acceleration_60m,
                'price_volume_alignment_60m': price_volume_alignment_60m,
                'wick_asymmetry_60m': wick_asymmetry_60m
            }
            data_rows.append(row)
            
    df = pd.DataFrame(data_rows)
    # Train split only
    n_train = int(len(df) * 0.6)
    return df.iloc[:n_train].copy()

def duration_metrics(series):
    if not series.any():
        return 0, 0, 0
    group = (series != series.shift()).cumsum()
    lengths = series.groupby(group).sum()
    valid_lengths = lengths[lengths > 0]
    if len(valid_lengths) == 0:
        return 0, 0, 0
    return len(valid_lengths), valid_lengths.mean(), valid_lengths.median()

def run_characterisation():
    print("Building state dataframe (Train split only)...")
    train_df = build_state_dataframe()
    
    features = ['volume_acceleration_60m', 'price_volume_alignment_60m', 'wick_asymmetry_60m']
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Crypto Expanded State v0.1: Stage B Characterisation\n")
        f.write("Evaluation of the 3 surviving distinct dimensions on the Train split.\n\n")
        
        f.write("## 1. Distribution / Regime Structure\n")
        f.write("| Feature | Q10 | Q25 | Q50 | Q75 | Q90 | % > Q90 | % < Q10 |\n")
        f.write("|---|---|---|---|---|---|---|---|\n")
        for col in features:
            q = train_df[col].quantile([0.1, 0.25, 0.5, 0.75, 0.9])
            gt_90 = (train_df[col] > q[0.9]).mean() * 100
            lt_10 = (train_df[col] < q[0.1]).mean() * 100
            f.write(f"| `{col}` | {q[0.1]:.4f} | {q[0.25]:.4f} | {q[0.5]:.4f} | {q[0.75]:.4f} | {q[0.9]:.4f} | {gt_90:.1f}% | {lt_10:.1f}% |\n")
            
        f.write("\n## 2. Temporal Behaviour\n")
        f.write("| Feature | ACF (1m) | ACF (15m) | ACF (60m) | ACF (240m) | > Median Episodes | > Median Duration (Mean) | > Median Duration (Med) | < Median Episodes | < Median Duration (Mean) | < Median Duration (Med) |\n")
        f.write("|---|---|---|---|---|---|---|---|---|---|---|\n")
        for col in features:
            ac1 = train_df[col].autocorr(lag=1)
            ac15 = train_df[col].autocorr(lag=15)
            ac60 = train_df[col].autocorr(lag=60)
            ac240 = train_df[col].autocorr(lag=240)
            
            med = train_df[col].median()
            ep_above, mean_above, med_above = duration_metrics(train_df[col] > med)
            ep_below, mean_below, med_below = duration_metrics(train_df[col] < med)
            
            f.write(f"| `{col}` | {ac1:.4f} | {ac15:.4f} | {ac60:.4f} | {ac240:.4f} | {ep_above} | {mean_above:.1f} | {med_above:.1f} | {ep_below} | {mean_below:.1f} | {med_below:.1f} |\n")
            
        f.write("\n## 3. Interaction with Existing State\n")
        
        vol_med = train_df['volatility_1m_std_24h'].median()
        is_high_vol = train_df['volatility_1m_std_24h'] > vol_med
        
        f.write("### Volume Acceleration x Volatility x Trend\n")
        f.write("| Volatility | Trend Dir | Median Volume Acceleration | N |\n")
        f.write("|---|---|---|---|\n")
        for v_state, v_mask in [("HIGH", is_high_vol), ("LOW", ~is_high_vol)]:
            for t_state, t_val in [("UP", 1), ("DOWN", -1)]:
                subset = train_df[v_mask & (train_df['trend_dir'] == t_val)]
                med_val = subset['volume_acceleration_60m'].median() if len(subset) > 0 else 0
                f.write(f"| {v_state} | {t_state} | {med_val:.4f} | {len(subset)} |\n")
                
        pers_med = train_df['persistence_60m'].median()
        is_high_pers = train_df['persistence_60m'] > pers_med
        
        f.write("\n### Wick Asymmetry x Trend Dir x Persistence\n")
        f.write("| Trend Dir | Persistence | Median Wick Asymmetry | N |\n")
        f.write("|---|---|---|---|\n")
        for t_state, t_val in [("UP", 1), ("DOWN", -1)]:
            for p_state, p_mask in [("HIGH", is_high_pers), ("LOW", ~is_high_pers)]:
                subset = train_df[(train_df['trend_dir'] == t_val) & p_mask]
                med_val = subset['wick_asymmetry_60m'].median() if len(subset) > 0 else 0
                f.write(f"| {t_state} | {p_state} | {med_val:.4f} | {len(subset)} |\n")
                
        f.write("\n## 4. Cross-Dimensional Events\n")
        f.write("Examining the temporal clustering of predefined composite states (using Q75/Q25 bounds).\n\n")
        
        q75_accel = train_df['volume_acceleration_60m'].quantile(0.75)
        q75_pers = train_df['persistence_60m'].quantile(0.75)
        q25_pers = train_df['persistence_60m'].quantile(0.25)
        q75_vol = train_df['volatility_1m_std_24h'].quantile(0.75)
        q75_wick = train_df['wick_asymmetry_60m'].quantile(0.75)
        q25_wick = train_df['wick_asymmetry_60m'].quantile(0.25)
        
        events = [
            ("High Vol Accel (>Q75) + Strong Trend (>Q75 Pers)", (train_df['volume_acceleration_60m'] > q75_accel) & (train_df['persistence_60m'] > q75_pers)),
            ("High Vol Accel (>Q75) + Weak Trend (<Q25 Pers)", (train_df['volume_acceleration_60m'] > q75_accel) & (train_df['persistence_60m'] < q25_pers)),
            ("High Vol Accel (>Q75) + High Volatility (>Q75)", (train_df['volume_acceleration_60m'] > q75_accel) & (train_df['volatility_1m_std_24h'] > q75_vol)),
            ("High Wick Asym (>Q75) + Strong Trend (>Q75 Pers)", (train_df['wick_asymmetry_60m'] > q75_wick) & (train_df['persistence_60m'] > q75_pers)),
            ("High Wick Asym (>Q75) + Weak Trend (<Q25 Pers)", (train_df['wick_asymmetry_60m'] > q75_wick) & (train_df['persistence_60m'] < q25_pers)),
        ]
        
        f.write("| Composite Event | Episodes | Event Freq (N) | Expected N (Independent) | Mean Duration (mins) | Median Duration (mins) |\n")
        f.write("|---|---|---|---|---|---|\n")
        
        total_n = len(train_df)
        for name, mask in events:
            n_events = mask.sum()
            expected_n = total_n * 0.25 * 0.25 
            episodes, mean_dur, med_dur = duration_metrics(mask)
            f.write(f"| {name} | {episodes} | {n_events} | {expected_n:.1f} | {mean_dur:.1f} | {med_dur:.1f} |\n")

    print(f"Characterisation complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_characterisation()
