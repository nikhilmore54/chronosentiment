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
DATA_FILES = [
    'low_vol_BTCUSDT.json',
    'reversal_BTCUSDT.json'
]

REPORT_FILE = os.path.join(os.path.dirname(__file__), '../crypto_stage_f2_cross_regime_report.md')

VA_HIGH_THRESHOLD = 1.4638
LOW_PERSISTENCE_THRESHOLD = 0.1129

def analyze_regime(filename):
    filepath = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto', filename)
    with open(filepath, 'r') as f:
        raw_data = json.load(f)
        
    total_raw_count = len(raw_data)
    if total_raw_count == 0:
        return None
        
    first_timestamp = datetime.fromtimestamp(int(raw_data[0]['timestamp']))
    last_timestamp = datetime.fromtimestamp(int(raw_data[-1]['timestamp']))
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    data_rows = []
    
    closes = np.array([float(d['close']) for d in raw_data])
    highs = np.array([float(d['high']) for d in raw_data])
    lows = np.array([float(d['low']) for d in raw_data])
    
    eligible_t0 = 0
    complete_60_count = 0
    complete_300_count = 0
    excluded_incomplete_t0s = 0
    
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
        
        vol_last_30m = sum(c['volume'] for c in w[-30:])
        vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
        volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
        
        t0_price = float(raw_obs['close'])
        if t0_price <= 0:
            continue
            
        f_closes = closes[i+1 : i+301]
        f_highs = highs[i+1 : i+301]
        f_lows = lows[i+1 : i+301]
        
        ret_15 = (f_closes[14] - t0_price) / t0_price
        ret_30 = (f_closes[29] - t0_price) / t0_price
        ret_60 = (f_closes[59] - t0_price) / t0_price
        ret_120 = (f_closes[119] - t0_price) / t0_price
        ret_300 = (f_closes[299] - t0_price) / t0_price
        
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
    
    df['va_state'] = np.where(df['volume_acceleration_60m'] > VA_HIGH_THRESHOLD, 'HIGH', 'NORMAL')
    df['pers_state'] = np.where(df['persistence_240m'] <= LOW_PERSISTENCE_THRESHOLD, 'LOW', 'HIGH')
    df['cohort'] = df['va_state'] + " / " + df['pers_state']
    
    df['Persistent_Build'] = (df['ret_30'] > df['ret_15']) & \
                             (df['ret_60'] > df['ret_30']) & \
                             (df['ret_120'] > df['ret_60'])
                             
    df['Adverse_Excursion_Recovery'] = (df['time_to_mae_60'] < 30) & \
                                       (df['mae_60'] < 0) & \
                                       (df['max_recovery_after_mae_60'] > 0)
                                       
    up_count = (df['trend_dir_240m'] == 1).sum()
    down_count = (df['trend_dir_240m'] == -1).sum()
    zero_count = (df['trend_dir_240m'] == 0).sum()
    
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
    
    if n_pb == 0:
        failure_mode = "FATAL FAILURE. The Persistent Build shape entirely disappeared."
    elif overrep_ratio < 1.0:
        failure_mode = "FAILURE. The HIGH/LOW association reversed or vanished. HIGH/LOW is underrepresented."
    elif abs(p_aer_given_pb - historical_overlap) > 0.30:
        failure_mode = "STRUCTURAL DEGRADATION. Path shape recurred, but link to early adverse excursion weakened substantially."
    else:
        failure_mode = "REPLICATED. PB shape recurred, retained its strong overlap with AER, and HIGH/LOW remained strictly overrepresented."
        
    return {
        'name': filename.split('.')[0],
        'first_timestamp': first_timestamp,
        'last_timestamp': last_timestamp,
        'total_raw': total_raw_count,
        'eligible': eligible_t0,
        'complete_60': complete_60_count,
        'complete_300': complete_300_count,
        'excluded': excluded_incomplete_t0s,
        'up': up_count,
        'down': down_count,
        'zero': zero_count,
        'n_total': n_total,
        'n_pb': n_pb,
        'p_pb': n_pb / n_total if n_total > 0 else 0,
        'n_pb_aer': n_pb_and_aer,
        'p_aer_given_pb': p_aer_given_pb,
        'n_high_low': n_high_low,
        'p_high_low': p_high_low,
        'n_high_low_given_pb': n_high_low_given_pb,
        'p_high_low_given_pb': p_high_low_given_pb,
        'overrep_ratio': overrep_ratio,
        'failure_mode': failure_mode
    }

def run_stage_f2():
    print("Running F2 cross-regime tests...")
    results = []
    for f in DATA_FILES:
        res = analyze_regime(f)
        if res:
            results.append(res)
            
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage F2: Cross-Regime Replication\n\n")
        f.write("Testing frozen F1 hypothesis across totally different environmental regimes.\n\n")
        
        for r in results:
            f.write(f"## Regime: {r['name']}\n")
            f.write(f"- **Time Range**: {r['first_timestamp']} to {r['last_timestamp']}\n")
            f.write(f"- **Raw Observations**: {r['total_raw']}\n")
            f.write(f"- **Complete 300-bar N**: {r['complete_300']}\n\n")
            
            f.write("### Directional Audit\n")
            f.write(f"- UP Trend: {r['up']}\n")
            f.write(f"- DOWN Trend: {r['down']}\n")
            f.write(f"- ZERO Trend: {r['zero']}\n\n")
            
            f.write("### 1. Persistent Build Recurrence\n")
            f.write(f"- N = {r['n_pb']} ({r['p_pb']:.2%})\n\n")
            
            f.write("### 2. PB → AER Overlap\n")
            f.write(f"- N = {r['n_pb_aer']} occurrences\n")
            f.write(f"- P(AER | PB) = {r['p_aer_given_pb']:.2%} (Historical was 92.00%)\n\n")
            
            f.write("### 3. HIGH/LOW Representation Ratio\n")
            f.write(f"- Population Share: {r['p_high_low']:.2%}\n")
            f.write(f"- PB Share: {r['p_high_low_given_pb']:.2%}\n")
            f.write(f"- Overrepresentation: {r['overrep_ratio']:.2f}x\n\n")
            
            f.write("### 4. Failure Mode Assessment\n")
            f.write(f"**{r['failure_mode']}**\n\n")
            
    print(f"Stage F2 report written to {REPORT_FILE}")

if __name__ == "__main__":
    run_stage_f2()
