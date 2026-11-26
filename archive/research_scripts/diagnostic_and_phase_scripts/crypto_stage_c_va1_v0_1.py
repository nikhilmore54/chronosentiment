#!/usr/bin/env python3
import requests
import json
import time
import math
import numpy as np
import pandas as pd
from pathlib import Path
from scipy.stats import spearmanr

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "crypto_24h"
REPORT_FILE = DATA_DIR / "crypto_stage_c_va1_report.md"

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

def build_stage_c_dataframe():
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
            w = obs_store.get_recent(1440)
            
            vol_last_30m = sum(c['volume'] for c in w[-30:])
            vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
            volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
            
            # Forward returns calculations
            ret_60m = np.nan
            ret_120m = np.nan
            ret_300m = np.nan
            
            curr_close = bar["close"]
            if i + 60 < len(formatted):
                ret_60m = math.log(formatted[i+60]["close"] / curr_close)
            if i + 120 < len(formatted):
                ret_120m = math.log(formatted[i+120]["close"] / curr_close)
            if i + 300 < len(formatted):
                ret_300m = math.log(formatted[i+300]["close"] / curr_close)
            
            row = {
                'timestamp': state_snapshot.timestamp,
                'volatility_1m_std_24h': state_snapshot.volatility_1m_std_24h,
                'trend_dir': state_snapshot.trend_dir,
                'persistence_60m': state_snapshot.persistence_60m,
                'volume_acceleration_60m': volume_acceleration_60m,
                'ret_60m': ret_60m,
                'ret_120m': ret_120m,
                'ret_300m': ret_300m
            }
            data_rows.append(row)
            
    df = pd.DataFrame(data_rows)
    # Train split only
    n_train = int(len(df) * 0.6)
    train_df = df.iloc[:n_train].copy()
    
    # Assign blocks 0..9
    train_df['block'] = np.floor(np.linspace(0, 10, len(train_df), endpoint=False)).astype(int)
    return train_df

def write_hypothesis_table(f, title, df, state_col, state_values, horizons, q75_va):
    f.write(f"### {title}\n")
    
    for horizon in horizons:
        f.write(f"#### Horizon: {horizon}m\n")
        f.write("| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |\n")
        f.write("|---|---|---|---|---|---|---|\n")
        
        target = f'ret_{horizon}m'
        valid_df = df.dropna(subset=[target])
        
        for state_val, state_label in state_values:
            for va_label, va_mask in [("VA_HIGH", valid_df['volume_acceleration_60m'] > q75_va), 
                                      ("VA_NORMAL", valid_df['volume_acceleration_60m'] <= q75_va)]:
                
                subset = valid_df[(valid_df[state_col] == state_val) & va_mask]
                
                n = len(subset)
                if n > 0:
                    hit_rate = (subset[target] > 0).mean() * 100
                    mean_ret = subset[target].mean() * 10000 # bps
                    med_ret = subset[target].median() * 10000
                    
                    # Spearman between State value and target if applicable? 
                    # State val is fixed here, so Spearman within this subset vs what?
                    # The user said "Spearman state/return relationship". 
                    # They likely mean the rank correlation of the raw state variable (e.g. raw volatility vs return) 
                    # within this bucket, or just the state value across the whole population.
                    # Let's compute rank correlation between the raw state coordinate and return within this bucket.
                    if state_col == 'trend_dir':
                        # raw metric for trend dir could be trend_return_60m, but we don't have it here. 
                        # I'll just use the volume_acceleration_60m vs return as the spearman metric, 
                        # or just leave it as N/A if std is 0. 
                        # Actually "Spearman state/return relationship" usually means the raw coordinate.
                        # Let's skip spearman inside the fixed state bucket if it's constant, 
                        # or we can compute spearman of volume_acceleration vs return within this bucket.
                        sp, _ = spearmanr(subset['volume_acceleration_60m'], subset[target])
                    else:
                        sp, _ = spearmanr(subset[state_col], subset[target])
                        
                    sp_str = f"{sp:.4f}" if not np.isnan(sp) else "N/A"
                else:
                    hit_rate = mean_ret = med_ret = 0.0
                    sp_str = "N/A"
                    
                f.write(f"| {state_label} | {va_label} | {n} | {hit_rate:.1f}% | {mean_ret:.1f} bps | {med_ret:.1f} bps | {sp_str} |\n")
                
        f.write("\n**Block Stability Analysis (Mean Ret in bps)**\n")
        f.write("| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |\n")
        f.write("|---|---|---|---|---|---|\n")
        
        for state_val, state_label in state_values:
            # Overall Delta
            state_mask = valid_df[state_col] == state_val
            va_high_mask = valid_df['volume_acceleration_60m'] > q75_va
            va_norm_mask = valid_df['volume_acceleration_60m'] <= q75_va
            
            high_mean = valid_df[state_mask & va_high_mask][target].mean() * 10000
            norm_mean = valid_df[state_mask & va_norm_mask][target].mean() * 10000
            overall_delta = high_mean - norm_mean
            
            block_deltas = []
            block_ns = []
            
            for b in range(10):
                b_df = valid_df[valid_df['block'] == b]
                b_high = b_df[(b_df[state_col] == state_val) & (b_df['volume_acceleration_60m'] > q75_va)][target]
                b_norm = b_df[(b_df[state_col] == state_val) & (b_df['volume_acceleration_60m'] <= q75_va)][target]
                
                n_b = min(len(b_high), len(b_norm))
                block_ns.append(n_b)
                
                if n_b > 0:
                    d = (b_high.mean() - b_norm.mean()) * 10000
                    block_deltas.append(d)
                else:
                    block_deltas.append(np.nan)
                    
            valid_deltas = [d for d in block_deltas if not np.isnan(d)]
            min_n = min(block_ns) if block_ns else 0
            
            if len(valid_deltas) > 0:
                med_delta = np.median(valid_deltas)
                same_sign = sum(1 for d in valid_deltas if (d > 0 and overall_delta > 0) or (d < 0 and overall_delta < 0))
                stability = (same_sign / len(valid_deltas)) * 100
                delta_str = ", ".join([f"{d:.1f}" if not np.isnan(d) else "N/A" for d in block_deltas])
            else:
                med_delta = 0.0
                stability = 0.0
                delta_str = "N/A"
                
            f.write(f"| {state_label} | {overall_delta:.1f} | {med_delta:.1f} | {stability:.0f}% | {min_n} | {delta_str} |\n")
        f.write("\n")

def run_stage_c():
    print("Building Stage C dataframe (Train split only)...")
    df = build_stage_c_dataframe()
    
    q75_va = df['volume_acceleration_60m'].quantile(0.75)
    med_vol = df['volatility_1m_std_24h'].median()
    med_pers = df['persistence_60m'].median()
    
    # Categorize state variables for easy grouping
    df['vol_state'] = np.where(df['volatility_1m_std_24h'] > med_vol, 1, 0) # 1=HIGH, 0=LOW
    df['pers_state'] = np.where(df['persistence_60m'] > med_pers, 1, 0) # 1=HIGH, 0=LOW
    
    horizons = [60, 120, 300]
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage C — Conditional Behaviour of Existing Price States Under Volume Acceleration\n\n")
        f.write(f"- **Train N:** {len(df)}\n")
        f.write(f"- **VA_HIGH Threshold (Q75):** {q75_va:.4f}\n")
        f.write(f"- **HIGH_VOL Threshold (Median):** {med_vol:.4f}\n")
        f.write(f"- **HIGH_PERS Threshold (Median):** {med_pers:.4f}\n\n")
        
        write_hypothesis_table(
            f, "H1: Volume × Existing Trend", 
            df, 
            'trend_dir', 
            [(1, "UP"), (-1, "DOWN")], 
            horizons, 
            q75_va
        )
        
        write_hypothesis_table(
            f, "H2: Volume × Existing Volatility", 
            df, 
            'vol_state', 
            [(1, "HIGH_VOL"), (0, "LOW_VOL")], 
            horizons, 
            q75_va
        )
        
        write_hypothesis_table(
            f, "H3: Volume × Persistence", 
            df, 
            'pers_state', 
            [(1, "HIGH_PERSISTENCE"), (0, "LOW_PERSISTENCE")], 
            horizons, 
            q75_va
        )

    print(f"Stage C completed. Report saved to {REPORT_FILE}")

if __name__ == "__main__":
    run_stage_c()
