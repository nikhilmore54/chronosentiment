import pandas as pd
import numpy as np
from scipy.stats import spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_v0.1_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "expanded_rh_validation_report.md"

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

def get_block_stats(df, feature_col, target_col):
    if len(df) == 0:
        return 0, 0, 0, 0
    df = df.copy()
    df['block'] = df['timestamp'] // 43200000
    block_counts = df.groupby('block').size()
    
    n_blocks = len(block_counts)
    min_obs = block_counts.min() if n_blocks > 0 else 0
    med_obs = block_counts.median() if n_blocks > 0 else 0
    
    block_corrs = []
    for b, group in df.groupby('block'):
        if len(group) > 30 and np.std(group[feature_col]) > 0 and np.std(group[target_col]) > 0:
            c, _ = spearmanr(group[feature_col], group[target_col])
            block_corrs.append(c)
            
    pos_pct = sum(1 for c in block_corrs if c > 0) / len(block_corrs) if block_corrs else 0
    
    return n_blocks, min_obs, med_obs, pos_pct

def evaluate_hypothesis(df, mask, ic_values, target):
    valid = df[mask].copy()
    valid = valid[[target, 'timestamp']].copy()
    valid['ic'] = ic_values[mask]
    valid = valid.dropna()
    
    n_samples = len(valid)
    n_blocks, min_obs, med_obs, pos_pct = get_block_stats(valid, 'ic', target)
    
    if n_samples < 30 or np.std(valid['ic']) == 0 or np.std(valid[target]) == 0:
        return None, n_samples, n_blocks, min_obs, med_obs, 0
        
    spearman_corr, _ = spearmanr(valid['ic'], valid[target])
    
    return spearman_corr, n_samples, n_blocks, min_obs, med_obs, pos_pct

def run_validation():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.8)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_val].copy()
    
    # 1. Derive thresholds strictly from Train
    vol_median = train['volatility_1m_std_24h'].median()
    pers_median = train['persistence_240m'].median()
    
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    d_mean = train['downside_excursion_24h'].mean()
    d_std = train['downside_excursion_24h'].std()
    
    # 2. Apply transformations to Train and Validation
    for split in [train, val]:
        split['z_U'] = (split['upside_excursion_24h'] - u_mean) / u_std
        split['z_D'] = (split['downside_excursion_24h'] - d_mean) / d_std
        split['R1'] = -split['z_U']
        split['R2'] = -split['z_D']
        
    # 3. Define Hypotheses
    # H1: LOW_VOL_UP / R1 / 300m
    # Regime: volatility_1m_std_24h < Train median and trend_return_60m > 0
    h1_mask_train = (train['volatility_1m_std_24h'] < vol_median) & (train['trend_return_60m'] > 0)
    h1_mask_val = (val['volatility_1m_std_24h'] < vol_median) & (val['trend_return_60m'] > 0)
    
    # H2: LOW_VOL_EXPANDING / R1 / 300m
    # Regime: low 24h volatility + volatility_ratio_240m_24h >= 1
    h2_mask_train = (train['volatility_1m_std_24h'] < vol_median) & (train['volatility_ratio_240m_24h'] >= 1)
    h2_mask_val = (val['volatility_1m_std_24h'] < vol_median) & (val['volatility_ratio_240m_24h'] >= 1)
    
    # H3: CHOP_DOWN / R2 / 300m
    # Regime: persistence_240m < Train median + trend_return_60m < 0
    h3_mask_train = (train['persistence_240m'] < pers_median) & (train['trend_return_60m'] < 0)
    h3_mask_val = (val['persistence_240m'] < pers_median) & (val['trend_return_60m'] < 0)
    
    results = []
    
    def add_result(h_name, split_name, df_split, mask, ic_col, target):
        corr, n_samp, n_blks, min_obs, med_obs, pos_pct = evaluate_hypothesis(df_split, mask, df_split[ic_col], target)
        if corr is not None:
            results.append({
                "Hypothesis": h_name,
                "Split": split_name,
                "Spearman": f"{corr:.4f}",
                "N": n_samp,
                "Blocks": n_blks,
                "PosBlocks": f"{pos_pct*100:.1f}%",
                "Min Obs/Blk": int(min_obs),
                "Med Obs/Blk": int(med_obs)
            })
            
    add_result("H1 (LOW_VOL_UP / R1)", "Train", train, h1_mask_train, 'R1', 'ret_300m')
    add_result("H1 (LOW_VOL_UP / R1)", "Validation", val, h1_mask_val, 'R1', 'ret_300m')
    
    add_result("H2 (LOW_VOL_EXPANDING / R1)", "Train", train, h2_mask_train, 'R1', 'ret_300m')
    add_result("H2 (LOW_VOL_EXPANDING / R1)", "Validation", val, h2_mask_val, 'R1', 'ret_300m')
    
    add_result("H3 (CHOP_DOWN / R2)", "Train", train, h3_mask_train, 'R2', 'ret_300m')
    add_result("H3 (CHOP_DOWN / R2)", "Validation", val, h3_mask_val, 'R2', 'ret_300m')

    res_df = pd.DataFrame(results)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Stage C: Validation of Frozen Expanded Hypotheses\n\n")
        f.write("Evaluation of H1, H2, and H3 on the strict chronological 20% Validation split, using only thresholds and normalizations derived from the Train split.\n\n")
        
        f.write("## H1: LOW_VOL_UP / R1 / 300m\n")
        f.write(df_to_markdown(res_df[res_df['Hypothesis'].str.startswith('H1')].drop(columns=['Hypothesis'])))
        f.write("\n")
        
        f.write("## H2: LOW_VOL_EXPANDING / R1 / 300m\n")
        f.write(df_to_markdown(res_df[res_df['Hypothesis'].str.startswith('H2')].drop(columns=['Hypothesis'])))
        f.write("\n")
        
        f.write("## H3: CHOP_DOWN / R2 / 300m\n")
        f.write(df_to_markdown(res_df[res_df['Hypothesis'].str.startswith('H3')].drop(columns=['Hypothesis'])))
        f.write("\n")
        
    print(f"Validation Evaluation complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_validation()
