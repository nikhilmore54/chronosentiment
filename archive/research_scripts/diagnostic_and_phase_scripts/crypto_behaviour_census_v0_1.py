import os
import json
import numpy as np
import pandas as pd
from typing import List, Dict

import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

DATA_PATH = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto/high_vol_BTCUSDT.json')
OUTPUT_CSV = os.path.join(os.path.dirname(__file__), '../datasets/crypto_behaviour_census_v0_1.csv')
REPORT_FILE = os.path.join(os.path.dirname(__file__), '../crypto_behaviour_census_v0_1_report.md')

VA_HIGH_THRESHOLD = 1.4638
LOW_PERSISTENCE_THRESHOLD = 0.1129

def load_data() -> List[Dict]:
    with open(DATA_PATH, 'r') as f:
        return json.load(f)

def construct_census(raw_data):
    print(f"Constructing census from {len(raw_data)} observations.")
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    data_rows = []
    
    # We need a fast lookup for future bars
    closes = np.array([float(d['close']) for d in raw_data])
    highs = np.array([float(d['high']) for d in raw_data])
    lows = np.array([float(d['low']) for d in raw_data])
    
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
        
        # Need 1440m history and 300m future
        if state_snapshot and i + 300 < len(raw_data):
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
            
            mfe_pct = (np.max(c_highs_60) - t0_price) / t0_price
            mae_pct = (np.min(c_lows_60) - t0_price) / t0_price
            time_to_mfe = int(np.argmax(c_highs_60) + 1)
            time_to_mae = int(np.argmin(c_lows_60) + 1)
            
            # Path behaviour
            max_recovery_after_mae = (np.max(f_highs[time_to_mae:60]) - np.min(c_lows_60)) / t0_price if time_to_mae < 60 else 0.0
            max_giveback_after_mfe = (np.max(c_highs_60) - np.min(f_lows[time_to_mfe:60])) / t0_price if time_to_mfe < 60 else 0.0
            final_return_rel_mfe = ret_60 - mfe_pct
            final_return_rel_mae = ret_60 - mae_pct
            crossed_pos_1pct = int(mfe_pct >= 0.01)
            crossed_neg_1pct = int(mae_pct <= -0.01)
            crossed_pos_2pct = int(mfe_pct >= 0.02)
            crossed_neg_2pct = int(mae_pct <= -0.02)
            
            # Exit behaviour (Categorical Target/Stop over 60 bars)
            # Long bias evaluation
            target_hit = np.where((c_highs_60 - t0_price) / t0_price >= 0.05)[0]
            stop_hit = np.where((c_lows_60 - t0_price) / t0_price <= -0.05)[0]
            
            t_target = target_hit[0] if len(target_hit) > 0 else 999
            t_stop = stop_hit[0] if len(stop_hit) > 0 else 999
            
            if t_target == 999 and t_stop == 999:
                exit_reason = "TimeStop"
            elif t_target < t_stop:
                exit_reason = "Target"
            else:
                exit_reason = "Stop"
                
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
                'mfe_60': mfe_pct,
                'mae_60': mae_pct,
                'time_to_mfe_60': time_to_mfe,
                'time_to_mae_60': time_to_mae,
                'max_recovery_after_mae_60': max_recovery_after_mae,
                'max_giveback_after_mfe_60': max_giveback_after_mfe,
                'final_return_rel_mfe_60': final_return_rel_mfe,
                'final_return_rel_mae_60': final_return_rel_mae,
                'crossed_pos_1pct_60': crossed_pos_1pct,
                'crossed_neg_1pct_60': crossed_neg_1pct,
                'crossed_pos_2pct_60': crossed_pos_2pct,
                'crossed_neg_2pct_60': crossed_neg_2pct,
                'exit_reason_60': exit_reason
            }
            data_rows.append(row)
            
    df = pd.DataFrame(data_rows)
    os.makedirs(os.path.dirname(OUTPUT_CSV), exist_ok=True)
    df.to_csv(OUTPUT_CSV, index=False)
    print(f"Saved {len(df)} records to {OUTPUT_CSV}")
    return df

def generate_report(df, total_raw_count):
    print("Generating report...")
    with open(REPORT_FILE, "w") as f:
        f.write("# Crypto Behaviour Census v0.1: Results\n\n")
        
        # Audit Table
        f.write("## Data Audit\n")
        
        # Calculate exactly how many would have 60-bar vs 300-bar
        # Our dataframe explicitly dropped everything without 300 bars. 
        # But we know total_raw_count, and warmup is 1440.
        eligible_t0 = total_raw_count - 1440
        complete_60_count = total_raw_count - 1440 - 60 if (total_raw_count - 1440) > 60 else 0
        complete_300_count = total_raw_count - 1440 - 300 if (total_raw_count - 1440) > 300 else 0
        
        up_count = (df['trend_dir_240m'] == 1).sum()
        down_count = (df['trend_dir_240m'] == -1).sum()
        zero_count = (df['trend_dir_240m'] == 0).sum()
        
        f.write("| Metric | Count |\n")
        f.write("|---|---|\n")
        f.write(f"| Raw observations | {total_raw_count} |\n")
        f.write(f"| Eligible T0 (after 1440m warmup) | {eligible_t0} |\n")
        f.write(f"| Complete 60-bar forward window | {complete_60_count} |\n")
        f.write(f"| Complete 300-bar forward window | {complete_300_count} (Census N) |\n")
        f.write(f"| UP trend (240m) | {up_count} |\n")
        f.write(f"| DOWN trend (240m) | {down_count} |\n")
        f.write(f"| ZERO trend (240m) | {zero_count} |\n\n")
        
        # Stage A Unconditional
        f.write("## Stage A — Future Behaviour Census (Unconditional)\n")
        f.write("Raw behavioral distributions across all valid T0.\n\n")
        
        med_ret_15 = df['ret_15'].median()
        med_ret_30 = df['ret_30'].median()
        med_ret_60 = df['ret_60'].median()
        med_ret_120 = df['ret_120'].median()
        med_ret_300 = df['ret_300'].median()
        med_mfe = df['mfe_60'].median()
        med_mae = df['mae_60'].median()
        med_tmfe = df['time_to_mfe_60'].median()
        med_tmae = df['time_to_mae_60'].median()
        med_recovery = df['max_recovery_after_mae_60'].median()
        med_giveback = df['max_giveback_after_mfe_60'].median()
        pos_1_pct = df['crossed_pos_1pct_60'].mean()
        neg_1_pct = df['crossed_neg_1pct_60'].mean()
        pos_2_pct = df['crossed_pos_2pct_60'].mean()
        neg_2_pct = df['crossed_neg_2pct_60'].mean()
        
        vc = df['exit_reason_60'].value_counts(normalize=True)
        tgt = vc.get('Target', 0.0)
        stp = vc.get('Stop', 0.0)
        ts = vc.get('TimeStop', 0.0)
        
        f.write("### Terminal & Excursion Returns (60-bar constraints except ret)\n")
        f.write("| 15m Ret | 30m Ret | 60m Ret | 120m Ret | 300m Ret | MFE | MAE | Time-to-MFE | Time-to-MAE |\n")
        f.write("|---|---|---|---|---|---|---|---|---|\n")
        f.write(f"| {med_ret_15:.4f} | {med_ret_30:.4f} | {med_ret_60:.4f} | {med_ret_120:.4f} | {med_ret_300:.4f} | {med_mfe:.4f} | {med_mae:.4f} | {med_tmfe:.1f} | {med_tmae:.1f} |\n\n")
        
        f.write("### Path & Exit Behaviours\n")
        f.write("| Max Recovery (after MAE) | Max Giveback (after MFE) | Crossed +1% | Crossed -1% | Crossed +2% | Crossed -2% | Target (+5%) | Stop (-5%) | TimeStop |\n")
        f.write("|---|---|---|---|---|---|---|---|---|\n")
        f.write(f"| {med_recovery:.4f} | {med_giveback:.4f} | {pos_1_pct:.2%} | {neg_1_pct:.2%} | {pos_2_pct:.2%} | {neg_2_pct:.2%} | {tgt:.2%} | {stp:.2%} | {ts:.2%} |\n\n")
        
        # Stage B
        f.write("## Stage B — State Conditioning (60-bar)\n")
        f.write("Partitioned by predefined state thresholds:\n")
        f.write(f"- VA_HIGH > {VA_HIGH_THRESHOLD}\n")
        f.write(f"- LOW_PERSISTENCE <= {LOW_PERSISTENCE_THRESHOLD}\n\n")
        
        df['va_state'] = np.where(df['volume_acceleration_60m'] > VA_HIGH_THRESHOLD, 'HIGH', 'NORMAL')
        df['pers_state'] = np.where(df['persistence_240m'] <= LOW_PERSISTENCE_THRESHOLD, 'LOW', 'HIGH')
        
        f.write("| VA State | Persistence State | N | Median 60m Ret | Hit Rate | Median MFE | Median MAE | % crossed +1% | % crossed -1% |\n")
        f.write("|---|---|---|---|---|---|---|---|---|\n")
        for va in ['HIGH', 'NORMAL']:
            for pers in ['LOW', 'HIGH']:
                subset = df[(df['va_state'] == va) & (df['pers_state'] == pers)]
                if len(subset) > 0:
                    med_ret = subset['ret_60'].median()
                    hr = (subset['ret_60'] > 0).mean()
                    med_mfe = subset['mfe_60'].median()
                    med_mae = subset['mae_60'].median()
                    pct_pos1 = subset['crossed_pos_1pct_60'].mean()
                    pct_neg1 = subset['crossed_neg_1pct_60'].mean()
                    f.write(f"| {va} | {pers} | {len(subset)} | {med_ret:.4f} | {hr:.2%} | {med_mfe:.4f} | {med_mae:.4f} | {pct_pos1:.2%} | {pct_neg1:.2%} |\n")
                    
        # Stage C
        f.write("\n## Stage C — Directional Decomposition\n")
        f.write("Evaluating the 60-bar outcome across trailing trend components.\n\n")
        
        f.write("| Trend | VA State | Persistence State | N | Median 60m Ret | Hit Rate | Median MFE | Median MAE |\n")
        f.write("|---|---|---|---|---|---|---|---|\n")
        for trend in [1, -1, 0]:
            trend_str = "UP" if trend == 1 else ("DOWN" if trend == -1 else "ZERO")
            for va in ['HIGH', 'NORMAL']:
                for pers in ['LOW', 'HIGH']:
                    subset = df[(df['trend_dir_240m'] == trend) & (df['va_state'] == va) & (df['pers_state'] == pers)]
                    if len(subset) > 0:
                        med_ret = subset['ret_60'].median()
                        hr = (subset['ret_60'] > 0).mean()
                        med_mfe = subset['mfe_60'].median()
                        med_mae = subset['mae_60'].median()
                        f.write(f"| {trend_str} | {va} | {pers} | {len(subset)} | {med_ret:.4f} | {hr:.2%} | {med_mfe:.4f} | {med_mae:.4f} |\n")
        
        if down_count == 0:
            f.write("\n*Note: The current dataset contains exclusively UP trend observations (240m trailing).* \n")

        # Stage D
        f.write("\n## Stage D — Temporal Decomposition (300-bar)\n")
        f.write("Evolution of terminal returns across forward horizons (15m, 30m, 60m, 120m, 300m).\n\n")
        
        f.write("| VA State | Persistence State | Trend | 15m Med | 30m Med | 60m Med | 120m Med | 300m Med |\n")
        f.write("|---|---|---|---|---|---|---|---|\n")
        for va in ['HIGH', 'NORMAL']:
            for pers in ['LOW', 'HIGH']:
                for trend in [1, -1, 0]:
                    trend_str = "UP" if trend == 1 else ("DOWN" if trend == -1 else "ZERO")
                    subset = df[(df['va_state'] == va) & (df['pers_state'] == pers) & (df['trend_dir_240m'] == trend)]
                    if len(subset) > 0:
                        m15 = subset['ret_15'].median()
                        m30 = subset['ret_30'].median()
                        m60 = subset['ret_60'].median()
                        m120 = subset['ret_120'].median()
                        m300 = subset['ret_300'].median()
                        f.write(f"| {va} | {pers} | {trend_str} | {m15:.4f} | {m30:.4f} | {m60:.4f} | {m120:.4f} | {m300:.4f} |\n")
                        
        # Exit behavior info
        f.write("\n## Exit Behaviour Distribution\n")
        f.write("| VA State | Persistence State | Target (5%) | Stop (-5%) | TimeStop (60m) |\n")
        f.write("|---|---|---|---|---|\n")
        for va in ['HIGH', 'NORMAL']:
            for pers in ['LOW', 'HIGH']:
                subset = df[(df['va_state'] == va) & (df['pers_state'] == pers)]
                if len(subset) > 0:
                    vc = subset['exit_reason_60'].value_counts(normalize=True)
                    tgt = vc.get('Target', 0.0)
                    stp = vc.get('Stop', 0.0)
                    ts = vc.get('TimeStop', 0.0)
                    f.write(f"| {va} | {pers} | {tgt:.2%} | {stp:.2%} | {ts:.2%} |\n")

    print(f"Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    raw_data = load_data()
    df = construct_census(raw_data)
    generate_report(df, len(raw_data))
