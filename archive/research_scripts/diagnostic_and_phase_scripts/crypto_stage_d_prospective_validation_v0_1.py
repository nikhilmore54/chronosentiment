#!/usr/bin/env python3
import requests
import json
import time
import math
import numpy as np
import pandas as pd
from pathlib import Path
from scipy.stats import spearmanr
from datetime import datetime

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "crypto_24h"
REPORT_FILE = DATA_DIR / "crypto_stage_d_prospective_report.md"

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

def get_ci(mean1, mean2, var1, var2, n1, n2):
    if n1 < 2 or n2 < 2:
        return np.nan, np.nan
    se = math.sqrt(var1/n1 + var2/n2)
    diff = mean1 - mean2
    return diff - 1.96*se, diff + 1.96*se

def run_stage_d():
    print("Fetching Prospective Data (Aug 17, 2026 - Present)...")
    
    # 2026-08-17 00:00:00 UTC
    prospective_start_ms = 1786924800000 
    # Fetch from 24h prior to ensure ObservationStore is warmed up when prospective_start_ms hits
    fetch_start_ms = prospective_start_ms - (1440 * 60 * 1000)
    end_ms = int(time.time() * 1000)
    
    symbol = "BTCUSDT"
    klines = fetch_binance_klines(symbol, "1m", fetch_start_ms, end_ms)
    
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
        
        # Only record states that belong to the prospective window
        if state_snapshot and bar["timestamp"] >= prospective_start_ms:
            w = obs_store.get_recent(1440)
            
            vol_last_30m = sum(c['volume'] for c in w[-30:])
            vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
            volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
            
            ret_120m = np.nan
            ret_300m = np.nan
            
            curr_close = bar["close"]
            
            # Explicitly enforce forward horizon limits
            # The observation must have a complete forward horizon to be evaluated
            has_120 = (i + 120 < len(formatted))
            has_300 = (i + 300 < len(formatted))
            
            if has_120:
                ret_120m = math.log(formatted[i+120]["close"] / curr_close)
            if has_300:
                ret_300m = math.log(formatted[i+300]["close"] / curr_close)
            
            row = {
                'timestamp': state_snapshot.timestamp,
                'persistence_60m': state_snapshot.persistence_60m,
                'volume_acceleration_60m': volume_acceleration_60m,
                'ret_120m': ret_120m,
                'ret_300m': ret_300m,
                'has_120m': has_120,
                'has_300m': has_300
            }
            data_rows.append(row)
            
    df = pd.DataFrame(data_rows)
    
    # Exclusions
    total_obs = len(df)
    df_120 = df[df['has_120m'] == True].copy()
    df_300 = df[df['has_300m'] == True].copy()
    excluded_120 = total_obs - len(df_120)
    excluded_300 = total_obs - len(df_300)
    
    # Frozen Thresholds
    FROZEN_VA_Q75 = 1.4638
    FROZEN_PERS_MEDIAN = 0.1129
    
    # Assign blocks 0..9
    df_120['block'] = np.floor(np.linspace(0, 10, len(df_120), endpoint=False)).astype(int)
    df_300['block'] = np.floor(np.linspace(0, 10, len(df_300), endpoint=False)).astype(int)
    
    first_ts = datetime.utcfromtimestamp(df['timestamp'].min() / 1000).strftime('%Y-%m-%d %H:%M:%S UTC')
    last_ts = datetime.utcfromtimestamp(df['timestamp'].max() / 1000).strftime('%Y-%m-%d %H:%M:%S UTC')
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage D — Prospective Validation of H-VA2\n\n")
        f.write("## Provenance Record\n")
        f.write(f"- **First Prospective Timestamp:** {first_ts}\n")
        f.write(f"- **Last Available Timestamp:** {last_ts}\n")
        f.write(f"- **Total Complete Observations in Window:** {total_obs}\n")
        f.write(f"- **Excluded (Incomplete 120m Horizon):** {excluded_120}\n")
        f.write(f"- **Excluded (Incomplete 300m Horizon):** {excluded_300}\n")
        f.write(f"- **FROZEN VA_HIGH Threshold:** {FROZEN_VA_Q75}\n")
        f.write(f"- **FROZEN Persistence Threshold:** {FROZEN_PERS_MEDIAN}\n\n")
        
        def evaluate_hypothesis(f, hyp_name, df_target, pers_mask, target, expect_dir):
            va_high = df_target[pers_mask & (df_target['volume_acceleration_60m'] > FROZEN_VA_Q75)]
            va_norm = df_target[pers_mask & (df_target['volume_acceleration_60m'] <= FROZEN_VA_Q75)]
            
            n_high = len(va_high)
            n_norm = len(va_norm)
            
            f.write(f"### {hyp_name}\n")
            f.write("| Metric | VA_HIGH | VA_NORMAL | Δ (High - Norm) |\n")
            f.write("|---|---|---|---|\n")
            
            if n_high > 0 and n_norm > 0:
                mean_high = va_high[target].mean() * 10000
                mean_norm = va_norm[target].mean() * 10000
                med_high = va_high[target].median() * 10000
                med_norm = va_norm[target].median() * 10000
                hit_high = (va_high[target] > 0).mean() * 100
                hit_norm = (va_norm[target] > 0).mean() * 100
                
                var_high = va_high[target].var() * (10000**2)
                var_norm = va_norm[target].var() * (10000**2)
                
                diff = mean_high - mean_norm
                ci_low, ci_high = get_ci(mean_high, mean_norm, var_high, var_norm, n_high, n_norm)
                
                f.write(f"| N | {n_high} | {n_norm} | - |\n")
                f.write(f"| Mean Ret | {mean_high:.1f} bps | {mean_norm:.1f} bps | **{diff:.1f} bps** |\n")
                f.write(f"| Med Ret | {med_high:.1f} bps | {med_norm:.1f} bps | {med_high - med_norm:.1f} bps |\n")
                f.write(f"| Hit Rate | {hit_high:.1f}% | {hit_norm:.1f}% | {hit_high - hit_norm:.1f}% |\n")
                f.write(f"| 95% CI of Δ | - | - | [{ci_low:.1f}, {ci_high:.1f}] bps |\n\n")
                
                # Block Stability
                block_deltas = []
                for b in range(10):
                    b_df = df_target[df_target['block'] == b]
                    b_high = b_df[pers_mask & (b_df['volume_acceleration_60m'] > FROZEN_VA_Q75)][target]
                    b_norm = b_df[pers_mask & (b_df['volume_acceleration_60m'] <= FROZEN_VA_Q75)][target]
                    
                    if len(b_high) > 0 and len(b_norm) > 0:
                        d = (b_high.mean() - b_norm.mean()) * 10000
                        block_deltas.append(d)
                    else:
                        block_deltas.append(np.nan)
                        
                valid_deltas = [d for d in block_deltas if not np.isnan(d)]
                if valid_deltas:
                    med_delta = np.median(valid_deltas)
                    
                    if expect_dir == "LOWER":
                        same_sign = sum(1 for d in valid_deltas if d < 0)
                    else:
                        same_sign = sum(1 for d in valid_deltas if d > 0)
                        
                    stability = (same_sign / len(valid_deltas)) * 100
                    delta_str = ", ".join([f"{d:.1f}" if not np.isnan(d) else "N/A" for d in block_deltas])
                    
                    f.write("**Block Stability Analysis**\n")
                    f.write(f"- **Expected Sign:** {'Negative' if expect_dir == 'LOWER' else 'Positive'}\n")
                    f.write(f"- **Median Block Δ:** {med_delta:.1f} bps\n")
                    f.write(f"- **Sign Stability:** {stability:.0f}%\n")
                    f.write(f"- **10 Block Δ Sequence:** {delta_str}\n\n")
                
        # Run H-VA2a
        evaluate_hypothesis(
            f, 
            "H-VA2a: High Persistence (120m) -> Expected LOWER Return", 
            df_120, 
            df_120['persistence_60m'] > FROZEN_PERS_MEDIAN, 
            'ret_120m',
            "LOWER"
        )
        
        # Run H-VA2b
        evaluate_hypothesis(
            f, 
            "H-VA2b: Low Persistence (300m) -> Expected HIGHER Return", 
            df_300, 
            df_300['persistence_60m'] <= FROZEN_PERS_MEDIAN, 
            'ret_300m',
            "HIGHER"
        )

    print(f"Stage D validation completed. Report saved to {REPORT_FILE}")

if __name__ == "__main__":
    run_stage_d()
