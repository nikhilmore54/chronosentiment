import pandas as pd
import numpy as np
from scipy.stats import pearsonr, spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_v0.1_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "expanded_discovery_report.md"

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
                row_strs.append(f"{x:.4f}" if abs(x) > 0.01 else f"{x:.5f}")
            else:
                row_strs.append(str(x))
        markdown += "| " + " | ".join(row_strs) + " |\n"
    return markdown

def evaluate_feature(df, feature_name, horizon):
    target = f"ret_{horizon}m"
    valid = df[[target, feature_name, 'timestamp']].copy()
    valid = valid.dropna()
    
    if len(valid) < 100:
        return None
        
    x = valid[feature_name].values
    y = valid[target].values
    
    if np.std(x) == 0 or np.std(y) == 0:
        return None
        
    pearson_corr, _ = pearsonr(x, y)
    spearman_corr, _ = spearmanr(x, y)
    
    # Hit rate
    x_centered = x - np.median(x)
    hits = np.sum(np.sign(x_centered) == np.sign(y))
    hit_rate = hits / len(x)
    
    # Quantile Monotonicity (Q1 vs Q5)
    x_noise = x + np.random.normal(0, 1e-8, len(x))
    try:
        valid['q'] = pd.qcut(x_noise, q=5, labels=False)
        quantiles = valid.groupby('q')[target].mean().values
        q1_ret = quantiles[0]
        q5_ret = quantiles[-1]
    except Exception:
        q1_ret, q5_ret = 0, 0
    
    # Block stability (12h blocks = 43200000 ms)
    valid['block'] = valid['timestamp'] // 43200000
    block_corrs = []
    
    for b, group in valid.groupby('block'):
        if len(group) > 30 and np.std(group[feature_name]) > 0 and np.std(group[target]) > 0:
            c, _ = spearmanr(group[feature_name], group[target])
            block_corrs.append(c)
            
    if block_corrs:
        pos_pct = sum(1 for c in block_corrs if c > 0) / len(block_corrs)
    else:
        pos_pct = 0
        
    return {
        "Feature": feature_name,
        "Horizon": f"{horizon}m",
        "Spearman": spearman_corr,
        "Pearson": pearson_corr,
        "HitRate": hit_rate,
        "Q1_Ret": q1_ret,
        "Q5_Ret": q5_ret,
        "PosBlocks": pos_pct,
        "N": len(x)
    }

def run_expanded_discovery():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    
    train = df.iloc[:n_train].copy()
    
    features = [
        # Existing 4
        "volatility_1m_std_24h", 
        "upside_excursion_24h", 
        "downside_excursion_24h", 
        "trend_dir",
        
        # Short-term directional state
        "trend_return_15m",
        "trend_return_60m",
        "trend_return_240m",
        
        # Volatility dynamics
        "volatility_std_60m",
        "volatility_std_240m",
        "volatility_ratio_60m_24h",
        "volatility_ratio_240m_24h",
        
        # Persistence
        "persistence_60m",
        "persistence_240m"
    ]
    
    horizons = [15, 30, 60, 120, 300]
    
    results = []
    
    for feat in features:
        for h in horizons:
            res = evaluate_feature(train, feat, h)
            if res:
                results.append(res)
                
    res_df = pd.DataFrame(results)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# State Expansion v0.1: Baseline Feature Discovery (Train Only)\n\n")
        f.write("Evaluation of all 12 expanded state variables on the Train block to measure unconditional linear Edge.\n\n")
        
        def fmt(x, kind):
            if kind == "pct": return f"{x*100:.1f}%"
            if kind == "bp": return f"{x*10000:.1f}bps"
            if kind == "corr": return f"{x:.4f}"
            return str(x)
        
        # Format the dataframe nicely
        formatted_df = res_df.copy()
        formatted_df['Spearman'] = formatted_df['Spearman'].apply(lambda x: fmt(x, 'corr'))
        formatted_df['Pearson'] = formatted_df['Pearson'].apply(lambda x: fmt(x, 'corr'))
        formatted_df['HitRate'] = formatted_df['HitRate'].apply(lambda x: fmt(x, 'pct'))
        formatted_df['Q1_Ret'] = formatted_df['Q1_Ret'].apply(lambda x: fmt(x, 'bp'))
        formatted_df['Q5_Ret'] = formatted_df['Q5_Ret'].apply(lambda x: fmt(x, 'bp'))
        formatted_df['PosBlocks'] = formatted_df['PosBlocks'].apply(lambda x: fmt(x, 'pct'))
        
        f.write(df_to_markdown(formatted_df))
        f.write("\n")
        
    print(f"Discovery complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_expanded_discovery()
