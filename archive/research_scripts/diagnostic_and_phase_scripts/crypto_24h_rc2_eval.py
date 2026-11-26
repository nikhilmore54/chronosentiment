import pandas as pd
import numpy as np
from scipy.stats import spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "rc2_report.md"

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

def get_block_details(df, target):
    valid = df[[target, 'ic', 'timestamp']].dropna()
    valid['block'] = valid['timestamp'] // 43200000
    
    blocks = []
    for b, group in valid.groupby('block'):
        n = len(group)
        if n > 15 and np.std(group['ic']) > 0 and np.std(group[target]) > 0:
            c, _ = spearmanr(group['ic'], group[target])
            blocks.append({
                "Block_ID": b,
                "N": n,
                "Spearman": c,
                "Upside_Mean": -group['ic'].mean(), # Since IC = -z_U
                "Ret_Mean": group[target].mean()
            })
    return pd.DataFrame(blocks)

def get_quintile_curve(df, target):
    valid = df[[target, 'ic']].dropna().copy()
    if len(valid) == 0:
        return {}
    
    x_noise = valid['ic'] + np.random.normal(0, 1e-8, len(valid))
    try:
        valid['q'] = pd.qcut(x_noise, q=5, labels=["Q1", "Q2", "Q3", "Q4", "Q5"])
        means = valid.groupby('q')[target].mean().to_dict()
    except Exception:
        means = {"Q1": 0, "Q2": 0, "Q3": 0, "Q4": 0, "Q5": 0}
    return means

def run_rc2():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.2)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_train+n_val].copy()
    
    train = train[train['trend_dir'] != 0].copy()
    val = val[val['trend_dir'] != 0].copy()
    
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    vol_median = train['volatility_1m_std_24h'].median()
    
    def prep(dataset):
        dataset['z_U'] = (dataset['upside_excursion_24h'] - u_mean) / u_std
        dataset['ic'] = -dataset['z_U']
        mask = (dataset['volatility_1m_std_24h'] < vol_median) & (dataset['trend_dir'] < 0)
        return dataset[mask].copy()
        
    t_sub = prep(train)
    v_sub = prep(val)
    
    horizons = [120, 300]
    
    with open(REPORT_FILE, "w") as f:
        f.write("# RC-2: LOW_VOL_DOWN / R2 Diagnostic Stability Test\n\n")
        f.write("We isolate the `LOW_VOL_DOWN` regime and the `R2` formulation (`-z_upside_excursion`) to analyze whether its positive rank association is broadly distributed or concentrated in a few anomalous chronological blocks.\n\n")
        
        for h in horizons:
            target = f"ret_{h}m"
            f.write(f"## Horizon: {h}m\n\n")
            
            # 1. Quintile curves
            t_curve = get_quintile_curve(t_sub, target)
            v_curve = get_quintile_curve(v_sub, target)
            
            f.write("### 1. Conditional Quintile Curves (R2 vs Mean Return)\n\n")
            f.write("| Split | Q1 (Low R2/High Excursion) | Q2 | Q3 | Q4 | Q5 (High R2/Low Excursion) |\n")
            f.write("|---|---|---|---|---|---|\n")
            
            def fmt(c): return f"{c*100:.3f}%"
            
            t_row = [fmt(t_curve.get(q,0)) for q in ["Q1", "Q2", "Q3", "Q4", "Q5"]]
            v_row = [fmt(v_curve.get(q,0)) for q in ["Q1", "Q2", "Q3", "Q4", "Q5"]]
            
            f.write(f"| Train | {' | '.join(t_row)} |\n")
            f.write(f"| Validation | {' | '.join(v_row)} |\n\n")
            
            # 2. Block details for Validation
            v_blocks = get_block_details(v_sub, target)
            
            f.write("### 2. Validation Block Breakdown\n\n")
            if len(v_blocks) > 0:
                pos_blocks = sum(v_blocks['Spearman'] > 0)
                tot_blocks = len(v_blocks)
                f.write(f"**Validation Positive Blocks:** {pos_blocks} / {tot_blocks} ({pos_blocks/tot_blocks:.1%})\n\n")
                f.write(df_to_markdown(v_blocks))
            else:
                f.write("No valid validation blocks.\n")
            f.write("\n")
            
if __name__ == "__main__":
    run_rc2()
