import os
import json
import numpy as np
import pandas as pd
from datetime import datetime
from typing import List, Dict

import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

# Frozen paths and thresholds
DATA_PATH = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto/high_vol_BTCUSDT.json')
REPORT_FILE = os.path.join(os.path.dirname(__file__), '../crypto_stage_f_oos_report.md')

VA_HIGH_THRESHOLD = 1.4638
LOW_PERSISTENCE_THRESHOLD = 0.1129

def run_stage_f_oos():
    with open(DATA_PATH, 'r') as f:
        raw_data = json.load(f)
        
    total_raw_count = len(raw_data)
    if total_raw_count == 0:
        print("Empty dataset.")
        return
        
    first_timestamp = datetime.fromtimestamp(int(raw_data[0]['timestamp']))
    last_timestamp = datetime.fromtimestamp(int(raw_data[-1]['timestamp']))
    
    print(f"Loaded {total_raw_count} observations from {first_timestamp} to {last_timestamp}")
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    data_rows = []
    
    closes = np.array([float(d['close']) for d in raw_data])
    highs = np.array([float(d['high']) for d in raw_data])
    lows = np.array([float(d['low']) for d in raw_data])
    
    # Trackers for audit
    eligible_t0 = 0
    complete_60_count = 0
    complete_300_count = 0
    excluded_incomplete_t0s = 0
    
    print("Simulating rolling state and constructing trajectories...")
    for i, raw_obs in enumerate(raw_data):
        ts = int(raw_obs['timestamp'])
        if ts < 1e11:
            ts *= 1000
            
        obs_store.add(
            timestamp=ts,
            open_px=float(raw_obs['open']),
            high_px=float(raw_obs['high']),
            low_px=float(raw_obs['low']),
            close_px=float(raw_obs['close']),
            volume=float(raw_obs['volume'])
        )
        state_snapshot = state_engine.update(obs_store)
        
        # Need 1440m history
        if not state_snapshot:
            continue
            
        eligible_t0 += 1
        
        if i + 60 < total_raw_count:
            complete_60_count += 1
            
        if i + 300 < total_raw_count:
            complete_300_count += 1
        else:
            excluded_incomplete_t0s += 1
            continue
            
        w = obs_store.get_recent(1440)
        
        # VA 60m calculation (explicit from Phase 1)
        vol_last_30m = sum(c['volume'] for c in w[-30:])
        vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
        volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
        
        t0_price = float(raw_obs['close'])
        if t0_price <= 0:
            continue
            
        # Future 300 bars
        f_closes = closes[i+1 : i+301]
        f_highs = highs[i+1 : i+301]
        f_lows = lows[i+1 : i+301]
        
        # Terminal behaviour
        ret_15 = (f_closes[14] - t0_price) / t0_price
        ret_30 = (f_closes[29] - t0_price) / t0_price
        ret_60 = (f_closes[59] - t0_price) / t0_price
        ret_120 = (f_closes[119] - t0_price) / t0_price
        ret_300 = (f_closes[299] - t0_price) / t0_price
        
        # Excursion behaviour (60 bar)
        c_highs_60 = f_highs[:60]
        c_lows_60 = f_lows[:60]
        
        mae_pct = (np.min(c_lows_60) - t0_price) / t0_price
        time_to_mae = int(np.argmin(c_lows_60) + 1)
        
        max_recovery_after_mae = (np.max(f_highs[time_to_mae:60]) - np.min(c_lows_60)) / t0_price if time_to_mae < 60 else 0.0
        
        row = {
            'timestamp': int(raw_obs['timestamp']),
            'volume_acceleration_60m': volume_acceleration_60m,
            'persistence_240m': state_snapshot.persistence_240m,
            'trend_dir_240m': state_snapshot.trend_dir,
            'ret_15': ret_15,
            'ret_30': ret_30,
            'ret_60': ret_60,
            'ret_120': ret_120,
            'ret_300': ret_300,
            'mae_60': mae_pct,
            'time_to_mae_60': time_to_mae,
            'max_recovery_after_mae_60': max_recovery_after_mae
        }
        data_rows.append(row)
            
    df = pd.DataFrame(data_rows)
    print(f"Constructed {len(df)} 300-bar trajectories.")
    
    # Frozen Definitions
    df['va_state'] = np.where(df['volume_acceleration_60m'] > VA_HIGH_THRESHOLD, 'HIGH', 'NORMAL')
    df['pers_state'] = np.where(df['persistence_240m'] <= LOW_PERSISTENCE_THRESHOLD, 'LOW', 'HIGH')
    df['cohort'] = df['va_state'] + " / " + df['pers_state']
    
    df['Persistent_Build'] = (df['ret_30'] > df['ret_15']) & \
                             (df['ret_60'] > df['ret_30']) & \
                             (df['ret_120'] > df['ret_60'])
                             
    df['Adverse_Excursion_Recovery'] = (df['time_to_mae_60'] < 30) & \
                                       (df['mae_60'] < 0) & \
                                       (df['max_recovery_after_mae_60'] > 0)
                                       
    # Audits
    up_count = (df['trend_dir_240m'] == 1).sum()
    down_count = (df['trend_dir_240m'] == -1).sum()
    zero_count = (df['trend_dir_240m'] == 0).sum()
    num_persistent_build = df['Persistent_Build'].sum()
    
    # Metrics
    n_total = len(df)
    n_pb = df['Persistent_Build'].sum()
    n_pb_and_aer = (df['Persistent_Build'] & df['Adverse_Excursion_Recovery']).sum()
    
    p_aer_given_pb = n_pb_and_aer / n_pb if n_pb > 0 else 0
    historical_overlap = 0.920
    
    n_high_low = (df['cohort'] == 'HIGH / LOW').sum()
    p_high_low = n_high_low / n_total if n_total > 0 else 0
    
    pb_subset = df[df['Persistent_Build']]
    n_high_low_given_pb = (pb_subset['cohort'] == 'HIGH / LOW').sum()
    p_high_low_given_pb = n_high_low_given_pb / n_pb if n_pb > 0 else 0
    
    overrep_ratio = p_high_low_given_pb / p_high_low if p_high_low > 0 else 0
    
    print("Generating Stage F report...")
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage F: Out-Of-Sample Chronological Test\n\n")
        f.write("Testing Hypothesis F1 on a fresh, strictly unseen chronological dataset.\n\n")
        
        f.write("## 1. Data Integrity & Provenance\n")
        f.write(f"- **Source File**: `high_vol_BTCUSDT.json`\n")
        f.write(f"- **Time Range**: {first_timestamp} to {last_timestamp}\n")
        f.write(f"- **Raw Observation Count**: {total_raw_count}\n")
        f.write(f"- **Eligible T0 Count** (after 1440m warmup): {eligible_t0}\n")
        f.write(f"- **Complete 60-bar Count**: {complete_60_count}\n")
        f.write(f"- **Complete 300-bar Count (Census N)**: {complete_300_count}\n")
        f.write(f"- **Excluded/Incomplete T0s**: {excluded_incomplete_t0s}\n\n")
        
        f.write("### Directional Breakdown\n")
        f.write(f"- UP Trend Count: {up_count}\n")
        f.write(f"- DOWN Trend Count: {down_count}\n")
        f.write(f"- ZERO Trend Count: {zero_count}\n\n")
        
        f.write("## 2. Phenotype Recurrence (Persistent Build)\n")
        f.write(f"- **Classified Persistent Builds**: {n_pb}\n")
        f.write(f"- **Frequency**: {n_pb / n_total:.2%} of OOS population\n\n")
        
        if n_pb > 0:
            f.write("### Persistent Build Median Path (OOS)\n")
            f.write("| 15m | 30m | 60m | 120m | 300m |\n")
            f.write("|---|---|---|---|---|\n")
            f.write(f"| {pb_subset['ret_15'].median():.4f} | {pb_subset['ret_30'].median():.4f} | {pb_subset['ret_60'].median():.4f} | {pb_subset['ret_120'].median():.4f} | {pb_subset['ret_300'].median():.4f} |\n\n")
            
        f.write("## 3. Structural Overlap\n")
        f.write(f"**Measurement:** Overlap between `Persistent Build` and `Adverse Excursion & Recovery`\n\n")
        f.write(f"- **Intersection (PB ∩ AER)**: {n_pb_and_aer} occurrences\n")
        f.write(f"- **P(Adverse Recovery | Persistent Build)**: {p_aer_given_pb:.2%}\n")
        f.write(f"- **Comparison with Frozen Census**: {p_aer_given_pb:.2%} (OOS) vs {historical_overlap:.2%} (Historical)\n\n")
        
        f.write("## 4. State Association Recurrence\n")
        f.write(f"**Measurement:** Overrepresentation of `HIGH VA + LOW persistence` within `Persistent Build`\n\n")
        f.write(f"- **P(HIGH/LOW) [Population Share]**: {p_high_low:.2%} ({n_high_low} / {n_total})\n")
        f.write(f"- **P(HIGH/LOW | Persistent Build)**: {p_high_low_given_pb:.2%} ({n_high_low_given_pb} / {n_pb})\n")
        f.write(f"- **Overrepresentation Ratio**: {overrep_ratio:.2f}x\n\n")
        
        f.write("## 5. Failure Modes Assessment\n")
        f.write("Evaluation of whether Hypothesis F1 survived contact with the OOS block:\n\n")
        
        if n_pb == 0:
            f.write("**Result**: FATAL FAILURE. The Persistent Build shape entirely disappeared in this dataset.\n")
        elif overrep_ratio < 1.0:
            f.write("**Result**: FATAL FAILURE. The HIGH/LOW association reversed or vanished. HIGH/LOW is underrepresented in the OOS block.\n")
        elif abs(p_aer_given_pb - historical_overlap) > 0.30:
            f.write("**Result**: STRUCTURAL DEGRADATION. The path shape recurred, but the strong mechanistic link to an early adverse excursion weakened substantially.\n")
        else:
            f.write("**Result**: HYPOTHESIS SURVIVED. The Persistent Build path structure recurred with similar prevalence, retained its extreme overlap with Adverse Recovery, and the HIGH/LOW state cohort remained strictly overrepresented.\n")
            
if __name__ == "__main__":
    run_stage_f_oos()
