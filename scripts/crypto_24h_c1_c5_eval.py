import argparse
import pandas as pd
import numpy as np
from scipy.stats import pearsonr, spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_BTCUSDT.csv"

def evaluate_ic(df, ic_name, ic_values, horizon):
    target = f"ret_{horizon}m"
    valid = df[[target, 'timestamp']].copy()
    valid['ic'] = ic_values
    valid = valid.dropna()
    
    if len(valid) < 100:
        return None
        
    x = valid['ic'].values
    y = valid[target].values
    
    if np.std(x) == 0 or np.std(y) == 0:
        return None
        
    pearson_corr, _ = pearsonr(x, y)
    spearman_corr, _ = spearmanr(x, y)
    
    # Hit rate
    x_centered = x - np.median(x)
    hits = np.sum(np.sign(x_centered) == np.sign(y))
    hit_rate = hits / len(x)
    
    # Quantile Monotonicity
    x_noise = x + np.random.normal(0, 1e-8, len(x))
    try:
        valid['q'] = pd.qcut(x_noise, q=5, labels=False)
        quantiles = valid.groupby('q')[target].mean().values
        q_diff = np.diff(quantiles)
        mono_up = np.all(q_diff > 0)
        mono_down = np.all(q_diff < 0)
        monotonicity = "Up" if mono_up else "Down" if mono_down else "Mixed"
    except Exception:
        quantiles = [0]*5
        monotonicity = "Error"
    
    # Block stability (12h blocks = 43200000 ms)
    valid['block'] = valid['timestamp'] // 43200000
    block_corrs = []
    
    for b, group in valid.groupby('block'):
        if len(group) > 30 and np.std(group['ic']) > 0 and np.std(group[target]) > 0:
            c, _ = spearmanr(group['ic'], group[target])
            block_corrs.append(c)
            
    if block_corrs:
        pos_pct = sum(1 for c in block_corrs if c > 0) / len(block_corrs)
        neg_pct = sum(1 for c in block_corrs if c < 0) / len(block_corrs)
        median_c = np.median(block_corrs)
        p25 = np.percentile(block_corrs, 25)
        p75 = np.percentile(block_corrs, 75)
        worst = np.min(block_corrs)
        best = np.max(block_corrs)
    else:
        pos_pct = neg_pct = median_c = p25 = p75 = worst = best = 0
        
    return {
        "ic_name": ic_name,
        "horizon": f"{horizon}m",
        "spearman": spearman_corr,
        "pearson": pearson_corr,
        "hit_rate": hit_rate,
        "monotonicity": monotonicity,
        "q1_ret": quantiles[0] if len(quantiles)>0 else 0,
        "q5_ret": quantiles[-1] if len(quantiles)>0 else 0,
        "median_block": median_c,
        "pos_pct": pos_pct,
        "neg_pct": neg_pct,
        "p25": p25,
        "p75": p75,
        "worst": worst,
        "best": best,
        "n_blocks": len(block_corrs)
    }

def run_evaluation(split="train"):
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.2)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_train+n_val].copy()
    holdout = df.iloc[n_train+n_val:].copy()
    
    # Determine the target dataset
    if split == "train":
        target_df = train
    elif split == "validate":
        target_df = val
    elif split == "holdout":
        target_df = holdout
    else:
        raise ValueError("Invalid split")
        
    print(f"Running evaluation on '{split}' split ({len(target_df)} rows)...")
    
    # ----------------------------------------------------
    # CRITICAL: Always use Train statistics for normalization
    # ----------------------------------------------------
    t_mean = train['trend_dir'].mean()
    t_std = train['trend_dir'].std()
    
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    
    d_mean = train['downside_excursion_24h'].mean()
    d_std = train['downside_excursion_24h'].std()
    
    # For volatility rank gating, we evaluate the quantile in the train set.
    # We can approximate this by sorting train volatility.
    train_vol_sorted = np.sort(train['volatility_1m_std_24h'].dropna().values)
    
    def get_vol_rank(val):
        # find the percentage of train values less than 'val'
        idx = np.searchsorted(train_vol_sorted, val)
        return idx / len(train_vol_sorted)
        
    target_df['z_T'] = (target_df['trend_dir'] - t_mean) / t_std
    target_df['z_U'] = (target_df['upside_excursion_24h'] - u_mean) / u_std
    target_df['z_D'] = (target_df['downside_excursion_24h'] - d_mean) / d_std
    target_df['g_V'] = target_df['volatility_1m_std_24h'].apply(get_vol_rank)
    
    # Define IC candidates using the EXACT same formulas
    ic_c1 = -(target_df['z_T'] + target_df['z_U'] + target_df['z_D'])
    ic_c2 = -(target_df['z_U'] + target_df['z_D'])
    ic_c3 = -(0.5 * target_df['z_U'] + 1.0 * target_df['z_D'])
    ic_c4 = ic_c1 * target_df['g_V']
    ic_c5 = target_df['z_D'] - target_df['z_U']
    
    candidates = [
        ("C1 (Dir Stretch)", ic_c1),
        ("C2 (Excursion MR)", ic_c2),
        ("C3 (Downside Asym)", ic_c3),
        ("C4 (Vol Gated C1)", ic_c4),
        ("C5 (Excursion Spread)", ic_c5)
    ]
    
    horizons = [120, 300]
    results = []
    
    for name, ic_vals in candidates:
        for h in horizons:
            res = evaluate_ic(target_df, name, ic_vals, h)
            if res:
                results.append(res)
                
    # Generate Report
    report_file = WORKSPACE / "datasets" / "crypto_24h" / f"c1_c5_{split}_report.md"
    
    with open(report_file, "w") as f:
        f.write(f"# Predeclared Candidate IC Evaluation ({split.capitalize()} Set)\n\n")
        
        # Primary Table
        f.write("## 1. Aggregate Telemetry\n\n")
        f.write("| IC Candidate | Horizon | Spearman | Pearson | HitRate | Mono | Q1 Ret | Q5 Ret |\n")
        f.write("|---|---|---|---|---|---|---|---|\n")
        for r in results:
            f.write(f"| {r['ic_name']} | {r['horizon']} | {r['spearman']:.4f} | {r['pearson']:.4f} | {r['hit_rate']:.1%} | {r['monotonicity']} | {r['q1_ret']:.2%} | {r['q5_ret']:.2%} |\n")
            
        f.write("\n## 2. Block Stability (12h Chronological Blocks)\n\n")
        f.write("| IC Candidate | Horizon | Median | Pos % | Neg % | P25 | P75 | Worst | Best | N |\n")
        f.write("|---|---|---|---|---|---|---|---|---|---|\n")
        for r in results:
            f.write(f"| {r['ic_name']} | {r['horizon']} | {r['median_block']:.4f} | {r['pos_pct']:.1%} | {r['neg_pct']:.1%} | {r['p25']:.4f} | {r['p75']:.4f} | {r['worst']:.4f} | {r['best']:.4f} | {r['n_blocks']} |\n")

    print(f"Evaluation complete. Report generated at {report_file}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--split", choices=["train", "validate", "holdout"], default="train")
    args = parser.parse_args()
    
    run_evaluation(args.split)
