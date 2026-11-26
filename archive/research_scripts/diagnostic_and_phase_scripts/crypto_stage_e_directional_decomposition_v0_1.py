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
REPORT_FILE = DATA_DIR / "crypto_stage_e_directional_report.md"

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

def get_trim_metrics(series):
    abs_series = series.abs()
    p99 = abs_series.quantile(0.99)
    # Remove top 1% absolute returns
    mask = abs_series <= p99
    trimmed_series = series[mask]
    n_removed = len(series) - len(trimmed_series)
    
    raw_mean = series.mean() * 10000
    trimmed_mean = trimmed_series.mean() * 10000
    diff = raw_mean - trimmed_mean
    return n_removed, raw_mean, trimmed_mean, diff

def run_stage_e():
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
                    'volume_acceleration_60m': va_60m,
                    'ret_300m': ret_300m,
                }
                data_rows.append(row)
                
    df = pd.DataFrame(data_rows)
    df_low_pers = df[df['persistence_60m'] <= FROZEN_PERS_MEDIAN].copy()
    
    va_high = df_low_pers[df_low_pers['volume_acceleration_60m'] > FROZEN_VA_Q75]['ret_300m']
    va_norm = df_low_pers[df_low_pers['volume_acceleration_60m'] <= FROZEN_VA_Q75]['ret_300m']
    
    def calc_metrics(series):
        if len(series) == 0:
            return {}
        return {
            'N': len(series),
            'hit_rate': (series > 0).mean() * 100,
            'neg_rate': (series < 0).mean() * 100,
            'long_mean': series[series > 0].mean() * 10000 if len(series[series > 0]) > 0 else 0.0,
            'short_mean': series[series < 0].mean() * 10000 if len(series[series < 0]) > 0 else 0.0,
            'expected_signed': series.mean() * 10000,
            'mean_abs': series.abs().mean() * 10000,
            'med_abs': series.abs().median() * 10000,
            'q10': series.quantile(0.1) * 10000,
            'q25': series.quantile(0.25) * 10000,
            'q50': series.quantile(0.5) * 10000,
            'q75': series.quantile(0.75) * 10000,
            'q90': series.quantile(0.9) * 10000
        }
        
    m_high = calc_metrics(va_high)
    m_norm = calc_metrics(va_norm)
    
    trim_high = get_trim_metrics(va_high)
    trim_norm = get_trim_metrics(va_norm)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage E — Directional Decomposition of H-VA2b\n\n")
        f.write(f"- **State:** LOW_PERSISTENCE (<= {FROZEN_PERS_MEDIAN})\n")
        f.write(f"- **Horizon:** 300m\n")
        f.write(f"- **Prospective Dataset N:** {len(df_low_pers)}\n\n")
        
        f.write("## 1. Signed Directional Behaviour\n")
        f.write("| Metric | VA_HIGH | VA_NORMAL | Δ |\n")
        f.write("|---|---|---|---|\n")
        f.write(f"| N | {m_high['N']} | {m_norm['N']} | - |\n")
        f.write(f"| Positive-Return Freq (Hit Rate) | {m_high['hit_rate']:.1f}% | {m_norm['hit_rate']:.1f}% | {m_high['hit_rate'] - m_norm['hit_rate']:.1f}% |\n")
        f.write(f"| Negative-Return Freq | {m_high['neg_rate']:.1f}% | {m_norm['neg_rate']:.1f}% | {m_high['neg_rate'] - m_norm['neg_rate']:.1f}% |\n")
        f.write(f"| Expected Signed Value (Mean) | {m_high['expected_signed']:.1f} bps | {m_norm['expected_signed']:.1f} bps | **{m_high['expected_signed'] - m_norm['expected_signed']:.1f} bps** |\n")
        f.write(f"| LONG-Direction Expected Return | {m_high['long_mean']:.1f} bps | {m_norm['long_mean']:.1f} bps | {m_high['long_mean'] - m_norm['long_mean']:.1f} bps |\n")
        f.write(f"| SHORT-Direction Expected Loss | {m_high['short_mean']:.1f} bps | {m_norm['short_mean']:.1f} bps | {m_high['short_mean'] - m_norm['short_mean']:.1f} bps |\n\n")

        f.write("## 2. Magnitude (Absolute Expansion) Behaviour\n")
        f.write("| Metric | VA_HIGH | VA_NORMAL | Δ |\n")
        f.write("|---|---|---|---|\n")
        f.write(f"| Mean Absolute Return | {m_high['mean_abs']:.1f} bps | {m_norm['mean_abs']:.1f} bps | {m_high['mean_abs'] - m_norm['mean_abs']:.1f} bps |\n")
        f.write(f"| Median Absolute Return | {m_high['med_abs']:.1f} bps | {m_norm['med_abs']:.1f} bps | {m_high['med_abs'] - m_norm['med_abs']:.1f} bps |\n\n")

        f.write("## 3. Signed Return Distribution\n")
        f.write("| Quantile | VA_HIGH | VA_NORMAL | Δ |\n")
        f.write("|---|---|---|---|\n")
        for q in ['q10', 'q25', 'q50', 'q75', 'q90']:
            label = q.upper()
            f.write(f"| {label} | {m_high[q]:.1f} bps | {m_norm[q]:.1f} bps | {m_high[q] - m_norm[q]:.1f} bps |\n")
            
        f.write("\n## 4. 1% Absolute-Return Tail Sensitivity\n")
        f.write("Removing the largest 1% of absolute returns from each respective subgroup to test extreme tail reliance.\n\n")
        f.write("| Metric | VA_HIGH | VA_NORMAL | Δ |\n")
        f.write("|---|---|---|---|\n")
        f.write(f"| N Removed | {trim_high[0]} | {trim_norm[0]} | - |\n")
        f.write(f"| Raw Mean Return | {trim_high[1]:.1f} bps | {trim_norm[1]:.1f} bps | {trim_high[1] - trim_norm[1]:.1f} bps |\n")
        f.write(f"| Trimmed Mean Return | {trim_high[2]:.1f} bps | {trim_norm[2]:.1f} bps | {trim_high[2] - trim_norm[2]:.1f} bps |\n")
        f.write(f"| Loss to Trimming (Raw - Trim) | {trim_high[3]:.1f} bps | {trim_norm[3]:.1f} bps | - |\n")

    print(f"Stage E directional decomposition complete. Report saved to {REPORT_FILE}")

if __name__ == "__main__":
    run_stage_e()
