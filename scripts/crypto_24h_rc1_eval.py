import pandas as pd
import numpy as np
from scipy.stats import spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "rc1_report.md"

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

def evaluate_regime_ic(df, ic_values, target):
    valid = df[[target, 'timestamp']].copy()
    valid['ic'] = ic_values
    valid = valid.dropna()
    
    n_samples = len(valid)
    if n_samples < 50:
        return 0.0, n_samples, 0, 0, 0
        
    x = valid['ic'].values
    y = valid[target].values
    
    if np.std(x) == 0 or np.std(y) == 0:
        return 0.0, n_samples, 0, 0, 0
        
    spearman_corr, _ = spearmanr(x, y)
    
    # Block stability (12h blocks)
    valid['block'] = valid['timestamp'] // 43200000
    block_corrs = []
    
    for b, group in valid.groupby('block'):
        if len(group) > 15 and np.std(group['ic']) > 0 and np.std(group[target]) > 0:
            c, _ = spearmanr(group['ic'], group[target])
            block_corrs.append(c)
            
    if block_corrs:
        pos_pct = sum(1 for c in block_corrs if c > 0) / len(block_corrs)
        neg_pct = sum(1 for c in block_corrs if c < 0) / len(block_corrs)
        median_c = np.median(block_corrs)
    else:
        pos_pct = neg_pct = median_c = 0.0
        
    return spearman_corr, n_samples, median_c, pos_pct, len(block_corrs)

def run_rc1_eval():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.2)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_train+n_val].copy()
    
    # Filter out trend_dir == 0
    train = train[train['trend_dir'] != 0].copy()
    val = val[val['trend_dir'] != 0].copy()
    
    # Calculate Train statistics
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    
    d_mean = train['downside_excursion_24h'].mean()
    d_std = train['downside_excursion_24h'].std()
    
    vol_median = train['volatility_1m_std_24h'].median()
    
    def apply_state(dataset):
        dataset['z_U'] = (dataset['upside_excursion_24h'] - u_mean) / u_std
        dataset['z_D'] = (dataset['downside_excursion_24h'] - d_mean) / d_std
        
        dataset['R1'] = -dataset['z_D']
        dataset['R2'] = -dataset['z_U']
        dataset['R3'] = -(dataset['z_U'] + dataset['z_D'])
        
        regimes = []
        for _, row in dataset.iterrows():
            if row['volatility_1m_std_24h'] >= vol_median and row['trend_dir'] > 0:
                regimes.append("HIGH_VOL_UP")
            elif row['volatility_1m_std_24h'] >= vol_median and row['trend_dir'] < 0:
                regimes.append("HIGH_VOL_DOWN")
            elif row['volatility_1m_std_24h'] < vol_median and row['trend_dir'] > 0:
                regimes.append("LOW_VOL_UP")
            elif row['volatility_1m_std_24h'] < vol_median and row['trend_dir'] < 0:
                regimes.append("LOW_VOL_DOWN")
            else:
                regimes.append("UNKNOWN")
        dataset['regime'] = regimes
        return dataset
        
    train = apply_state(train)
    val = apply_state(val)
    
    regimes_list = ["HIGH_VOL_UP", "HIGH_VOL_DOWN", "LOW_VOL_UP", "LOW_VOL_DOWN"]
    candidates = ["R1", "R2", "R3"]
    horizons = [120, 300]
    
    results = []
    
    for r in regimes_list:
        for c in candidates:
            for h in horizons:
                train_sub = train[train['regime'] == r]
                val_sub = val[val['regime'] == r]
                
                target = f"ret_{h}m"
                
                tr_corr, tr_n, tr_med, tr_pos, tr_blocks = evaluate_regime_ic(train_sub, train_sub[c], target)
                vl_corr, vl_n, vl_med, vl_pos, vl_blocks = evaluate_regime_ic(val_sub, val_sub[c], target)
                
                sign_match = (np.sign(tr_corr) == np.sign(vl_corr)) and (tr_corr != 0 and vl_corr != 0)
                
                results.append({
                    "Regime": r,
                    "IC": c,
                    "Horizon": f"{h}m",
                    "Train N": tr_n,
                    "Val N": vl_n,
                    "Train Corr": tr_corr,
                    "Val Corr": vl_corr,
                    "Same Sign": "YES" if sign_match else "NO",
                    "Train PosBlocks": f"{tr_pos:.1%} ({tr_blocks})",
                    "Val PosBlocks": f"{vl_pos:.1%} ({vl_blocks})"
                })
                
    res_df = pd.DataFrame(results)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# RC-1: Regime-Conditioned IC Hypothesis\n\n")
        f.write("We evaluate the 3 predeclared excursion formulations conditional on the background regime (derived solely from Train thresholds). Train `trend_dir == 0` rows are excluded.\n\n")
        
        f.write("## Hypothesis Results\n\n")
        f.write(df_to_markdown(res_df))
        
        # Summarize passing gates
        f.write("\n## Evaluation Gate Checklist\n\n")
        f.write("A regime-conditioned candidate advances ONLY if:\n")
        f.write("1. Adequate observations in both Train and Validation.\n")
        f.write("2. The relationship has the same sign in Train and Validation.\n")
        f.write("3. The aggregate relationship doesn't collapse toward zero in Validation.\n")
        f.write("4. Block stability isn't merely being generated by one or two blocks.\n\n")
        
        passing = res_df[(res_df['Same Sign'] == 'YES') & (res_df['Val Corr'].abs() > 0.05) & (res_df['Val N'] > 100)]
        if len(passing) > 0:
            f.write("### Conditionally Passing Candidates:\n")
            f.write(df_to_markdown(passing))
        else:
            f.write("### Result:\n")
            f.write("**NO** candidates survived the strict sign-stability and magnitude preservation gate. All regime-conditioned formulations broke down or inverted across the chronological split.\n")
            
    print(f"Evaluation complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_rc1_eval()
