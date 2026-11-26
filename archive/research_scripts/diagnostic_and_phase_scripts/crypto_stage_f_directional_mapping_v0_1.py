#!/usr/bin/env python3
import requests
import json
import time
import math
import numpy as np
import pandas as pd
from pathlib import Path

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "crypto_24h"
REPORT_FILE = DATA_DIR / "crypto_stage_f_directional_report.md"

FROZEN_VA_Q75 = 1.4638
FROZEN_PERS_MEDIAN = 0.1129

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

def evaluate_subset(f, title, df_subset):
    f.write(f"### {title}\n")
    
    va_high = df_subset[df_subset['volume_acceleration_60m'] > FROZEN_VA_Q75]
    va_norm = df_subset[df_subset['volume_acceleration_60m'] <= FROZEN_VA_Q75]
    
    n_high = len(va_high)
    n_norm = len(va_norm)
    
    f.write("| Metric | VA_HIGH | VA_NORMAL | Δ (High - Norm) |\n")
    f.write("|---|---|---|---|\n")
    f.write(f"| N | {n_high} | {n_norm} | - |\n")
    
    if n_high > 0 and n_norm > 0:
        mean_h = va_high['ret_300m'].mean() * 10000
        mean_n = va_norm['ret_300m'].mean() * 10000
        med_h = va_high['ret_300m'].median() * 10000
        med_n = va_norm['ret_300m'].median() * 10000
        hit_h = (va_high['ret_300m'] > 0).mean() * 100
        hit_n = (va_norm['ret_300m'] > 0).mean() * 100
        
        diff = mean_h - mean_n
        
        f.write(f"| Mean Ret | {mean_h:.1f} bps | {mean_n:.1f} bps | **{diff:.1f} bps** |\n")
        f.write(f"| Med Ret | {med_h:.1f} bps | {med_n:.1f} bps | {med_h - med_n:.1f} bps |\n")
        f.write(f"| Hit Rate | {hit_h:.1f}% | {hit_n:.1f}% | {hit_h - hit_n:.1f}% |\n\n")
        
        # Block Stability
        block_deltas = []
        block_ns = []
        
        for b in range(10):
            b_df = df_subset[df_subset['block'] == b]
            b_high = b_df[b_df['volume_acceleration_60m'] > FROZEN_VA_Q75]['ret_300m']
            b_norm = b_df[b_df['volume_acceleration_60m'] <= FROZEN_VA_Q75]['ret_300m']
            
            n_b_h = len(b_high)
            n_b_n = len(b_norm)
            block_ns.append((n_b_h, n_b_n))
            
            if n_b_h > 0 and n_b_n > 0:
                d = (b_high.mean() - b_norm.mean()) * 10000
                block_deltas.append(d)
            else:
                block_deltas.append(np.nan)
                
        valid_deltas = [d for d in block_deltas if not np.isnan(d)]
        if valid_deltas:
            med_delta = np.median(valid_deltas)
            same_sign = sum(1 for d in valid_deltas if (d > 0 and diff > 0) or (d < 0 and diff < 0))
            stability = (same_sign / len(valid_deltas)) * 100
            
            delta_str = ", ".join([f"{d:.1f}" if not np.isnan(d) else "N/A" for d in block_deltas])
            n_str = ", ".join([f"({h}/{n})" for h, n in block_ns])
            min_h = min([h for h, n in block_ns])
            min_n = min([n for h, n in block_ns])
            
            f.write("**Block Stability Analysis**\n")
            f.write(f"- **Overall Δ Sign:** {'Positive' if diff > 0 else 'Negative'}\n")
            f.write(f"- **Median Block Δ:** {med_delta:.1f} bps\n")
            f.write(f"- **Sign Stability:** {stability:.0f}%\n")
            f.write(f"- **Min Block N (High/Norm):** {min_h} / {min_n}\n")
            f.write(f"- **10 Block Δ Sequence:** {delta_str}\n")
            f.write(f"- **10 Block N Sequence (High/Norm):** {n_str}\n\n")
    else:
        f.write("\nInsufficient data for comparison.\n\n")

def run_stage_f():
    print("Fetching Prospective Data (Aug 17, 2026 - Present)...")
    
    prospective_start_ms = 1786924800000 
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
            bar["timestamp"], bar["open"], bar["high"], bar["low"], bar["close"], bar["volume"]
        )
        if not added:
            continue
            
        state_snapshot = state_engine.update(obs_store)
        
        if state_snapshot and bar["timestamp"] >= prospective_start_ms:
            w = obs_store.get_recent(1440)
            
            vol_last_30m = sum(c['volume'] for c in w[-30:])
            vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
            va_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
            
            if i + 300 < len(formatted):
                ret_300m = math.log(formatted[i+300]["close"] / bar["close"])
                row = {
                    'timestamp': state_snapshot.timestamp,
                    'persistence_60m': state_snapshot.persistence_60m,
                    'trend_dir': state_snapshot.trend_dir,
                    'volume_acceleration_60m': va_60m,
                    'ret_300m': ret_300m,
                }
                data_rows.append(row)
                
    df = pd.DataFrame(data_rows)
    df_low_pers = df[df['persistence_60m'] <= FROZEN_PERS_MEDIAN].copy()
    
    # Assign chronological blocks to the LOW_PERSISTENCE population
    df_low_pers['block'] = np.floor(np.linspace(0, 10, len(df_low_pers), endpoint=False)).astype(int)
    
    df_up = df_low_pers[df_low_pers['trend_dir'] == 1].copy()
    df_down = df_low_pers[df_low_pers['trend_dir'] == -1].copy()
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage F — Directional Mapping of H-VA2b\n\n")
        f.write(f"- **Base State:** LOW_PERSISTENCE (<= {FROZEN_PERS_MEDIAN})\n")
        f.write(f"- **Horizon:** 300m\n")
        f.write(f"- **Total N:** {len(df_low_pers)}\n")
        f.write(f"- **UP Trend N:** {len(df_up)}\n")
        f.write(f"- **DOWN Trend N:** {len(df_down)}\n\n")
        
        evaluate_subset(f, "LOW_PERSISTENCE + UP Trend", df_up)
        evaluate_subset(f, "LOW_PERSISTENCE + DOWN Trend", df_down)

    print(f"Stage F directional mapping complete. Report saved to {REPORT_FILE}")

if __name__ == "__main__":
    run_stage_f()
