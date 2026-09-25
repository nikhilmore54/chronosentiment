import os
import pandas as pd
import numpy as np

CSV_PATH = os.path.join(os.path.dirname(__file__), '../datasets/crypto_behaviour_census_v0_1.csv')
REPORT_PATH = os.path.join(os.path.dirname(__file__), '../crypto_stage_e_phenotypes_report.md')

def run_stage_e():
    print("Loading census dataset...")
    df = pd.read_csv(CSV_PATH)
    
    # We only want the 2581 observations with complete 300-bar windows
    # Wait, the CSV already has 2582 rows, which are exactly those with 300 bars.
    print(f"Loaded {len(df)} trajectories.")
    
    # 1. Persistent Build
    df['Persistent_Build'] = (df['ret_30'] > df['ret_15']) & \
                             (df['ret_60'] > df['ret_30']) & \
                             (df['ret_120'] > df['ret_60'])
                             
    # 2. Early Excursion & Decay
    df['Early_Excursion_Decay'] = (df['ret_60'] > 0) & \
                                  (df['ret_120'] < df['ret_60'] * 0.5)
                                  
    # 3. Adverse Excursion & Recovery
    df['Adverse_Excursion_Recovery'] = (df['time_to_mae_60'] < 30) & \
                                       (df['mae_60'] < 0) & \
                                       (df['max_recovery_after_mae_60'] > 0)
                                       
    # 4. Flat / Bound
    med_mfe = df['mfe_60'].median()
    med_mae = df['mae_60'].median()
    df['Flat_Bound'] = (df['mfe_60'] < med_mfe) & (df['mae_60'] > med_mae)
    
    phenotypes = ['Persistent_Build', 'Early_Excursion_Decay', 'Adverse_Excursion_Recovery', 'Flat_Bound']
    
    # VA / Persist state (from Stage B logic)
    VA_HIGH_THRESHOLD = 1.4638
    LOW_PERSISTENCE_THRESHOLD = 0.1129
    df['va_state'] = np.where(df['volume_acceleration_60m'] > VA_HIGH_THRESHOLD, 'HIGH', 'NORMAL')
    df['pers_state'] = np.where(df['persistence_240m'] <= LOW_PERSISTENCE_THRESHOLD, 'LOW', 'HIGH')
    df['cohort'] = df['va_state'] + " / " + df['pers_state']
    
    with open(REPORT_PATH, 'w') as f:
        f.write("# Stage E: Behavioural Phenotype Extraction\n\n")
        
        f.write("## Phenotype Definitions (Descriptive)\n")
        f.write("- **Persistent Build**: `ret_30 > ret_15 AND ret_60 > ret_30 AND ret_120 > ret_60`\n")
        f.write("- **Early Excursion & Decay**: `ret_60 > 0 AND ret_120 < ret_60 * 0.5`\n")
        f.write("- **Adverse Excursion & Recovery**: `time_to_mae_60 < 30 AND mae_60 < 0 AND max_recovery_after_mae_60 > 0`\n")
        f.write(f"- **Flat / Bound**: `mfe_60 < {med_mfe:.4f} AND mae_60 > {med_mae:.4f}` (empirical medians)\n\n")
        
        f.write("## Unconditional Frequency\n")
        f.write("| Phenotype | N | % of Population |\n")
        f.write("|---|---|---|\n")
        for p in phenotypes:
            n = df[p].sum()
            f.write(f"| {p} | {n} | {n/len(df):.2%} |\n")
            
        f.write("\n## Overlap Matrix\n")
        f.write("| Phenotype | Persistent_Build | Early_Excursion_Decay | Adverse_Excursion_Recovery | Flat_Bound |\n")
        f.write("|---|---|---|---|---|\n")
        for p1 in phenotypes:
            row = [p1]
            for p2 in phenotypes:
                overlap = (df[p1] & df[p2]).sum()
                row.append(str(overlap))
            f.write("| " + " | ".join(row) + " |\n")
            
        f.write("\n## Phenotype Profiles (Medians)\n")
        for p in phenotypes:
            sub = df[df[p]]
            if len(sub) == 0:
                continue
            f.write(f"### {p} (N={len(sub)})\n")
            
            # VAxPersist distribution
            f.write("**VA × Persistence Distribution:**\n")
            vc = sub['cohort'].value_counts()
            for c, count in vc.items():
                f.write(f"- {c}: {count} ({count/len(sub):.1%})\n")
                
            f.write("\n**Returns:**\n")
            f.write("| 15m | 30m | 60m | 120m | 300m |\n")
            f.write("|---|---|---|---|---|\n")
            f.write(f"| {sub['ret_15'].median():.4f} | {sub['ret_30'].median():.4f} | {sub['ret_60'].median():.4f} | {sub['ret_120'].median():.4f} | {sub['ret_300'].median():.4f} |\n")
            
            f.write("\n**Excursion & Path:**\n")
            f.write("| MFE | MAE | Time-to-MFE | Time-to-MAE | Recovery (after MAE) | Giveback (after MFE) |\n")
            f.write("|---|---|---|---|---|---|\n")
            f.write(f"| {sub['mfe_60'].median():.4f} | {sub['mae_60'].median():.4f} | {sub['time_to_mfe_60'].median():.1f} | {sub['time_to_mae_60'].median():.1f} | {sub['max_recovery_after_mae_60'].median():.4f} | {sub['max_giveback_after_mfe_60'].median():.4f} |\n\n")

    print(f"Report written to {REPORT_PATH}")

if __name__ == "__main__":
    run_stage_e()
