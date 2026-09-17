import pandas as pd
import numpy as np
from scipy.stats import pearsonr, spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_BTCUSDT.csv"

def analyze_feature(df, feature, horizon):
    target = f"ret_{horizon}m"
    
    # Drop NaNs
    valid = df[[feature, target, 'timestamp']].dropna()
    if len(valid) < 100:
        return None
        
    x = valid[feature].values
    y = valid[target].values
    
    # Exclude zero-variance features
    if np.std(x) == 0 or np.std(y) == 0:
        return None
        
    # Correlations
    pearson_corr, p_p = pearsonr(x, y)
    spearman_corr, p_s = spearmanr(x, y)
    
    # Hit rate: Does the sign of the feature predict the sign of the return?
    # Note: For upside/downside excursion and volatility, they are always positive.
    # We will center them around their median for directional prediction tests.
    x_centered = x - np.median(x)
    hits = np.sum(np.sign(x_centered) == np.sign(y))
    hit_rate = hits / len(x) if len(x) > 0 else 0
    
    # Quantile Monotonicity (5 bins)
    # Add small noise to x to prevent non-unique bin edges if many exact zeros exist
    x_noise = x + np.random.normal(0, 1e-8, len(x))
    try:
        valid['q'] = pd.qcut(x_noise, q=5, labels=False)
        quantiles = valid.groupby('q')[target].mean().values
        # Simple check: does it strictly increase or decrease?
        q_diff = np.diff(quantiles)
        mono_up = np.all(q_diff > 0)
        mono_down = np.all(q_diff < 0)
        monotonicity = "Up" if mono_up else "Down" if mono_down else "Mixed"
    except Exception:
        quantiles = [0] * 5
        monotonicity = "Error"
        
    # Time-block stability (12h blocks)
    # 12h = 43200000 ms
    valid['block'] = valid['timestamp'] // 43200000
    block_corrs = []
    for b, group in valid.groupby('block'):
        if len(group) > 30 and np.std(group[feature]) > 0 and np.std(group[target]) > 0:
            c, _ = spearmanr(group[feature], group[target])
            block_corrs.append(c)
            
    if block_corrs:
        pos_blocks = sum(1 for c in block_corrs if c > 0)
        block_stability = pos_blocks / len(block_corrs)
    else:
        block_stability = 0
        
    return {
        "feature": feature,
        "horizon": horizon,
        "pearson": pearson_corr,
        "spearman": spearman_corr,
        "hit_rate": hit_rate,
        "monotonicity": monotonicity,
        "q1_ret": quantiles[0] if len(quantiles)>0 else 0,
        "q5_ret": quantiles[-1] if len(quantiles)>0 else 0,
        "block_stability": block_stability,
        "n_blocks": len(block_corrs)
    }

def run_discovery():
    print(f"Loading dataset from {DATA_FILE}...")
    df = pd.read_csv(DATA_FILE)
    
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.2)
    n_test = n_total - n_train - n_val
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_train+n_val].copy()
    test = df.iloc[n_train+n_val:].copy()
    
    print(f"Split: Train {len(train)} | Validate {len(val)} | Holdout {len(test)}")
    
    features = [
        "volatility_1m_std_24h", 
        "upside_excursion_24h", 
        "downside_excursion_24h", 
        "trend_dir"
    ]
    horizons = [15, 30, 60, 120, 300]
    
    results = []
    
    for f in features:
        print(f"Analyzing {f}...")
        for h in horizons:
            res = analyze_feature(train, f, h)
            if res:
                results.append(res)
                
    # Format as markdown table
    print("\n# Train Set Baseline IC Research\n")
    
    headers = ["Feature", "Horizon", "Spearman", "Pearson", "HitRate", "Mono", "Q1 Mean Ret", "Q5 Mean Ret", "Pos Blocks %"]
    print("| " + " | ".join(headers) + " |")
    print("|" + "|".join(["---" for _ in headers]) + "|")
    
    for r in results:
        row = [
            r['feature'],
            f"{r['horizon']}m",
            f"{r['spearman']:.4f}",
            f"{r['pearson']:.4f}",
            f"{r['hit_rate']:.1%}",
            r['monotonicity'],
            f"{r['q1_ret']:.2%}",
            f"{r['q5_ret']:.2%}",
            f"{r['block_stability']:.1%} ({r['n_blocks']} blocks)"
        ]
        print("| " + " | ".join(row) + " |")

if __name__ == "__main__":
    run_discovery()
