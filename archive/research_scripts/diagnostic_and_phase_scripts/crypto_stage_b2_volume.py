import os
import json
import numpy as np
import pandas as pd
from typing import List, Dict

CSV_PATH = os.path.join(os.path.dirname(__file__), '../datasets/crypto_behaviour_census_v0_1.csv')
DATA_PATH = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto/high_vol_BTCUSDT.json')
OUTPUT_PATH = os.path.join(os.path.dirname(__file__), '../datasets/crypto_behaviour_census_with_b2.csv')
REPORT_PATH = os.path.join(os.path.dirname(__file__), '../crypto_stage_b2_volume_report.md')

def run_stage_b2():
    print("Loading raw OHLCV and Stage A/B census...")
    df = pd.read_csv(CSV_PATH)
    
    with open(DATA_PATH, 'r') as f:
        raw_data = json.load(f)
        
    print(f"Loaded {len(raw_data)} raw observations and {len(df)} census records.")
    
    # We will build a DataFrame of the required volume metrics aligned by timestamp
    vol_data = []
    
    # Fast array access
    timestamps = []
    for r in raw_data:
        timestamps.append(int(r['timestamp']))
        
    volumes = np.array([float(r['volume']) for r in raw_data])
    
    print("Computing trailing volume descriptors...")
    for i in range(240, len(raw_data)):
        t0_timestamp = timestamps[i]
        
        vols_60 = volumes[i-59:i+1]
        vols_240 = volumes[i-239:i+1]
        
        vol_sum_60m = np.sum(vols_60)
        vol_sum_240m = np.sum(vols_240)
        
        vol_baseline = vol_sum_240m / 4.0
        rel_vol_60m = vol_sum_60m / vol_baseline if vol_baseline > 0 else 1.0
        
        vol_concentration_60m = np.max(vols_60) / vol_sum_60m if vol_sum_60m > 0 else 0.0
        
        vol_data.append({
            'timestamp': t0_timestamp,
            'volume_sum_60m': vol_sum_60m,
            'relative_volume_60m': rel_vol_60m,
            'volume_concentration_60m': vol_concentration_60m
        })
        
    vol_df = pd.DataFrame(vol_data)
    
    # Join with the existing census by exact timestamp
    print("Joining datasets...")
    original_n = len(df)
    
    merged = pd.merge(df, vol_df, on='timestamp', how='left')
    
    # Recalculate phenotypes (since Stage E was descriptive and didn't save to CSV)
    merged['Persistent_Build'] = (merged['ret_30'] > merged['ret_15']) & \
                                 (merged['ret_60'] > merged['ret_30']) & \
                                 (merged['ret_120'] > merged['ret_60'])
    merged['Early_Excursion_Decay'] = (merged['ret_60'] > 0) & \
                                      (merged['ret_120'] < merged['ret_60'] * 0.5)
    merged['Adverse_Excursion_Recovery'] = (merged['time_to_mae_60'] < 30) & \
                                           (merged['mae_60'] < 0) & \
                                           (merged['max_recovery_after_mae_60'] > 0)
    med_mfe = merged['mfe_60'].median()
    med_mae = merged['mae_60'].median()
    merged['Flat_Bound'] = (merged['mfe_60'] < med_mfe) & (merged['mae_60'] > med_mae)
    
    VA_HIGH_THRESHOLD = 1.4638
    LOW_PERSISTENCE_THRESHOLD = 0.1129
    merged['va_state'] = np.where(merged['volume_acceleration_60m'] > VA_HIGH_THRESHOLD, 'HIGH', 'NORMAL')
    merged['pers_state'] = np.where(merged['persistence_240m'] <= LOW_PERSISTENCE_THRESHOLD, 'LOW', 'HIGH')
    merged['cohort'] = merged['va_state'] + " / " + merged['pers_state']

    missing_count = merged['volume_sum_60m'].isna().sum()
    print(f"Missing count: {missing_count}")
    
    merged.to_csv(OUTPUT_PATH, index=False)
    
    print("Generating report...")
    with open(REPORT_PATH, 'w') as f:
        f.write("# Stage B2: Participation & Order-Flow Characterisation\n\n")
        
        f.write("## Data Audit\n")
        f.write(f"- **Frozen Census N**: {original_n}\n")
        f.write(f"- **Successfully Joined N**: {original_n - missing_count}\n")
        f.write(f"- **Missing/Invalid Volume N**: {missing_count}\n\n")
        f.write("*Note: `volume_acceleration_60m` is preserved exactly as evaluated in Stage A-D.*\n\n")
        
        vol_vars = ['volume_sum_60m', 'volume_acceleration_60m', 'relative_volume_60m', 'volume_concentration_60m']
        
        f.write("## Unconditional Volume Distributions\n")
        f.write("| Variable | P10 | P25 | P50 (Median) | P75 | P90 |\n")
        f.write("|---|---|---|---|---|---|\n")
        for v in vol_vars:
            p = merged[v].quantile([0.1, 0.25, 0.5, 0.75, 0.9])
            f.write(f"| {v} | {p[0.1]:.4f} | {p[0.25]:.4f} | {p[0.5]:.4f} | {p[0.75]:.4f} | {p[0.9]:.4f} |\n")
            
        f.write("\n## Volume by VA × Persistence Cohort (Medians)\n")
        f.write("| Cohort | N | volume_sum_60m | volume_acceleration_60m | relative_volume_60m | volume_concentration_60m |\n")
        f.write("|---|---|---|---|---|---|\n")
        
        cohorts = ['HIGH / LOW', 'HIGH / HIGH', 'NORMAL / LOW', 'NORMAL / HIGH']
        for c in cohorts:
            sub = merged[merged['cohort'] == c]
            if len(sub) > 0:
                f.write(f"| {c} | {len(sub)} | {sub['volume_sum_60m'].median():.4f} | {sub['volume_acceleration_60m'].median():.4f} | {sub['relative_volume_60m'].median():.4f} | {sub['volume_concentration_60m'].median():.4f} |\n")
                
        f.write("\n## Volume by Descriptive Phenotype (Medians)\n")
        f.write("Examining whether the identified 300-bar path shapes exhibit materially different participation structures.\n\n")
        f.write("| Phenotype | N | volume_sum_60m | volume_acceleration_60m | relative_volume_60m | volume_concentration_60m |\n")
        f.write("|---|---|---|---|---|---|\n")
        
        phenotypes = ['Persistent_Build', 'Early_Excursion_Decay', 'Adverse_Excursion_Recovery', 'Flat_Bound']
        for p in phenotypes:
            sub = merged[merged[p]]
            if len(sub) > 0:
                f.write(f"| {p} | {len(sub)} | {sub['volume_sum_60m'].median():.4f} | {sub['volume_acceleration_60m'].median():.4f} | {sub['relative_volume_60m'].median():.4f} | {sub['volume_concentration_60m'].median():.4f} |\n")

    print(f"Report written to {REPORT_PATH}")

if __name__ == "__main__":
    run_stage_b2()
