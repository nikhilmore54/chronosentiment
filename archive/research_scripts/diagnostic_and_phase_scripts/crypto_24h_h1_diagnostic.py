import pandas as pd
import numpy as np
from scipy.stats import spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_v0.1_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "h1_diagnostic_report.md"

def df_to_markdown(df):
    if len(df) == 0:
        return ""
    headers = list(df.columns)
    markdown = "| " + " | ".join(headers) + " |\n"
    markdown += "|---" * len(headers) + "|\n"
    for _, row in df.iterrows():
        row_strs = []
        for x in row:
            if isinstance(x, float):
                row_strs.append(f"{x:.4f}")
            else:
                row_strs.append(str(x))
        markdown += "| " + " | ".join(row_strs) + " |\n"
    return markdown

def run_h1_diagnostics():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.8)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_val].copy()
    
    # Derive thresholds from Train
    vol_median = train['volatility_1m_std_24h'].median()
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    
    # Isolate H1 on Validation
    val_mask = (val['volatility_1m_std_24h'] < vol_median) & (val['trend_return_60m'] > 0)
    h1_val = val[val_mask].copy()
    
    h1_val['z_U'] = (h1_val['upside_excursion_24h'] - u_mean) / u_std
    h1_val['R1'] = -h1_val['z_U']
    
    valid = h1_val[['ret_300m', 'R1', 'timestamp']].dropna()
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage C.1: H1 Diagnostic Checks (Validation Only)\n\n")
        f.write("Hypothesis: `LOW_VOL_UP / R1 / 300m`\n\n")
        
        # 1. Q1-Q5 Monotonicity
        f.write("## 1. Quintile Ordering & 2. Score-bin Occupancy\n")
        valid['Q'] = pd.qcut(valid['R1'] + np.random.normal(0, 1e-8, len(valid)), 5, labels=['Q1 (Low R1)', 'Q2', 'Q3', 'Q4', 'Q5 (High R1)'])
        q_stats = valid.groupby('Q')['ret_300m'].agg(['count', 'mean', 'median']).reset_index()
        q_stats.columns = ['Quintile', 'N', 'Mean_Ret', 'Median_Ret']
        q_stats['Mean_Ret'] = q_stats['Mean_Ret'].apply(lambda x: f"{x*10000:.1f}bps")
        q_stats['Median_Ret'] = q_stats['Median_Ret'].apply(lambda x: f"{x*10000:.1f}bps")
        f.write(df_to_markdown(q_stats))
        f.write("\n")
        
        # 3. Block-level distribution
        f.write("## 3. Block-level Distribution\n")
        valid['block'] = valid['timestamp'] // 43200000
        block_res = []
        for b, grp in valid.groupby('block'):
            if len(grp) > 5 and np.std(grp['R1']) > 0 and np.std(grp['ret_300m']) > 0:
                c, _ = spearmanr(grp['R1'], grp['ret_300m'])
                block_res.append({"Block_ID": b, "N": len(grp), "Spearman": c})
        blk_df = pd.DataFrame(block_res)
        f.write(df_to_markdown(blk_df))
        f.write("\n")
        
        # 4. Conditional return & 5. Sign symmetry
        f.write("## 4. Conditional Return & 5. Sign Symmetry\n")
        median_r1 = valid['R1'].median()
        high_r1 = valid[valid['R1'] > median_r1]
        low_r1 = valid[valid['R1'] <= median_r1]
        
        cond_df = pd.DataFrame([
            {"Subset": "High R1 (> Median)", "N": len(high_r1), "Mean_Ret": f"{high_r1['ret_300m'].mean()*10000:.1f}bps", "Median_Ret": f"{high_r1['ret_300m'].median()*10000:.1f}bps"},
            {"Subset": "Low R1 (<= Median)", "N": len(low_r1), "Mean_Ret": f"{low_r1['ret_300m'].mean()*10000:.1f}bps", "Median_Ret": f"{low_r1['ret_300m'].median()*10000:.1f}bps"}
        ])
        f.write(df_to_markdown(cond_df))
        f.write("\n")
        
        # 6. Outlier/Influence Check
        f.write("## 6. Outlier/Influence Check\n")
        orig_corr, _ = spearmanr(valid['R1'], valid['ret_300m'])
        
        # Remove top/bottom 1% of returns
        p1 = valid['ret_300m'].quantile(0.01)
        p99 = valid['ret_300m'].quantile(0.99)
        trimmed = valid[(valid['ret_300m'] > p1) & (valid['ret_300m'] < p99)]
        trim_corr, _ = spearmanr(trimmed['R1'], trimmed['ret_300m'])
        
        out_df = pd.DataFrame([
            {"Sample": "Full Validation", "N": len(valid), "Spearman": orig_corr},
            {"Sample": "Trimmed (1% Tails Removed)", "N": len(trimmed), "Spearman": trim_corr}
        ])
        f.write(df_to_markdown(out_df))
        f.write("\n")

    print(f"H1 Diagnostics complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_h1_diagnostics()
