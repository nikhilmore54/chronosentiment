import pandas as pd
import numpy as np
from scipy.stats import pearsonr, spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_5m" / "ic_discovery_5m_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_5m" / "baseline_discovery_report.md"

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
        return 0, 0
    df = df.copy()
    # 12h blocks = 43200000 ms
    df['block'] = df['timestamp'] // 43200000
    block_corrs = []
    
    for b, group in df.groupby('block'):
        # using 10 observations minimum for 5m block since max is 144 (12h * 12)
        if len(group) > 10 and np.std(group[feature_col]) > 0 and np.std(group[target_col]) > 0:
            c, _ = spearmanr(group[feature_col], group[target_col])
            block_corrs.append(c)
            
    pos_pct = sum(1 for c in block_corrs if c > 0) / len(block_corrs) if block_corrs else 0
    
    return len(block_corrs), pos_pct

def evaluate_ic(df, mask, ic_values, target):
    valid = df[mask].copy()
    valid = valid[[target, 'timestamp']].copy()
    valid['ic'] = ic_values[mask]
    valid = valid.dropna()
    
    n_samples = len(valid)
    n_blocks, pos_pct = get_block_stats(valid, 'ic', target)
    
    if n_samples < 30 or np.std(valid['ic']) == 0 or np.std(valid[target]) == 0:
        return None, n_samples, n_blocks, 0
        
    spearman_corr, _ = spearmanr(valid['ic'], valid[target])
    
    return spearman_corr, n_samples, n_blocks, pos_pct

def run_evaluation():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.8)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_val].copy()
    
    # 1. Derive statistics from Train
    t_mean = train['trend_dir'].mean()
    t_std = train['trend_dir'].std()
    
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    
    d_mean = train['downside_excursion_24h'].mean()
    d_std = train['downside_excursion_24h'].std()
    
    train_vol_sorted = np.sort(train['volatility_5m_std_24h'].dropna().values)
    def get_vol_rank(val):
        idx = np.searchsorted(train_vol_sorted, val)
        return idx / len(train_vol_sorted)
        
    results = []
    
    for split_name, target_df in [("Train", train), ("Validation", val)]:
        target_df['z_T'] = (target_df['trend_dir'] - t_mean) / t_std
        target_df['z_U'] = (target_df['upside_excursion_24h'] - u_mean) / u_std
        target_df['z_D'] = (target_df['downside_excursion_24h'] - d_mean) / d_std
        target_df['g_V'] = target_df['volatility_5m_std_24h'].apply(get_vol_rank)
        
        ic_c1 = -(target_df['z_T'] + target_df['z_U'] + target_df['z_D'])
        ic_c2 = -(target_df['z_U'] + target_df['z_D'])
        ic_c3 = -(0.5 * target_df['z_U'] + 1.0 * target_df['z_D'])
        ic_c4 = ic_c1 * target_df['g_V']
        ic_c5 = target_df['z_D'] - target_df['z_U']
        
        candidates = [
            ("C1", ic_c1),
            ("C2", ic_c2),
            ("C3", ic_c3),
            ("C4", ic_c4),
            ("C5", ic_c5)
        ]
        
        horizons = [120, 300]
        mask = pd.Series(True, index=target_df.index)
        
        for name, ic_vals in candidates:
            for h in horizons:
                target_col = f"ret_{h}m"
                corr, n_samp, n_blks, pos_pct = evaluate_ic(target_df, mask, ic_vals, target_col)
                if corr is not None:
                    results.append({
                        "Split": split_name,
                        "IC": name,
                        "Horizon": f"{h}m",
                        "Spearman": f"{corr:.4f}",
                        "N": n_samp,
                        "Blocks": n_blks,
                        "PosBlocks": f"{pos_pct*100:.1f}%"
                    })
                    
    res_df = pd.DataFrame(results)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Crypto 5m Research: Stage A Baseline IC Discovery\n\n")
        f.write("Evaluation of C1-C5 on the native 5m substrate (288-bar state warmup). Horizons are 120m (24 bars) and 300m (60 bars).\n\n")
        
        f.write(df_to_markdown(res_df))
        f.write("\n")
        
    print(f"5m Baseline Discovery complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_evaluation()
