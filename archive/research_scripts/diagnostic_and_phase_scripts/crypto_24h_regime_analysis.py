import pandas as pd
import numpy as np
from scipy.stats import spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "regime_analysis_report.md"

def compare_distributions(train, val):
    features = [
        "volatility_1m_std_24h", 
        "upside_excursion_24h", 
        "downside_excursion_24h", 
        "trend_dir"
    ]
    
    stats = []
    for f in features:
        train_mean = train[f].mean()
        train_std = train[f].std()
        val_mean = val[f].mean()
        val_std = val[f].std()
        
        # KS-test equivalent simplified metrics (shift in mean and std)
        mean_shift = (val_mean - train_mean) / train_std if train_std != 0 else 0
        
        stats.append({
            "Feature": f,
            "Train Mean": train_mean,
            "Train Std": train_std,
            "Val Mean": val_mean,
            "Val Std": val_std,
            "Mean Shift (Z)": mean_shift
        })
        
    return pd.DataFrame(stats)

def regime_conditional_correlations(train, val, feature_to_test, horizon=300):
    target = f"ret_{horizon}m"
    
    # We will test the strongest train feature: downside_excursion_24h against regimes of volatility and trend
    vol_median_train = train['volatility_1m_std_24h'].median()
    
    results = []
    
    for split_name, df in [("Train", train), ("Validation", val)]:
        valid = df[[feature_to_test, target, 'volatility_1m_std_24h', 'trend_dir', 'timestamp']].dropna()
        if len(valid) < 100:
            continue
            
        # Regimes based on Train thresholds to prevent look-ahead bias
        # 1. High vs Low Volatility
        high_vol_mask = valid['volatility_1m_std_24h'] >= vol_median_train
        low_vol_mask = valid['volatility_1m_std_24h'] < vol_median_train
        
        high_vol_corr, _ = spearmanr(valid[high_vol_mask][feature_to_test], valid[high_vol_mask][target]) if sum(high_vol_mask) > 30 else (0,0)
        low_vol_corr, _ = spearmanr(valid[low_vol_mask][feature_to_test], valid[low_vol_mask][target]) if sum(low_vol_mask) > 30 else (0,0)
        
        # 2. Up Trend vs Down Trend (Ignore 0 for binary split)
        up_trend_mask = valid['trend_dir'] > 0
        down_trend_mask = valid['trend_dir'] < 0
        
        up_trend_corr, _ = spearmanr(valid[up_trend_mask][feature_to_test], valid[up_trend_mask][target]) if sum(up_trend_mask) > 30 else (0,0)
        down_trend_corr, _ = spearmanr(valid[down_trend_mask][feature_to_test], valid[down_trend_mask][target]) if sum(down_trend_mask) > 30 else (0,0)
        
        results.append({
            "Split": split_name,
            "Target": f"{feature_to_test} -> {target}",
            "High Vol Corr": high_vol_corr,
            "Low Vol Corr": low_vol_corr,
            "Up Trend Corr": up_trend_corr,
            "Down Trend Corr": down_trend_corr,
            "N": len(valid)
        })
        
    return pd.DataFrame(results)

def df_to_markdown(df):
    if len(df) == 0:
        return ""
    headers = list(df.columns)
    markdown = "| " + " | ".join(headers) + " |\n"
    markdown += "|---" * len(headers) + "|\n"
    for _, row in df.iterrows():
        row_strs = [f"{x:.4f}" if isinstance(x, float) else str(x) for x in row]
        markdown += "| " + " | ".join(row_strs) + " |\n"
    return markdown

def run_analysis():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.2)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_train+n_val].copy()
    
    dist_df = compare_distributions(train, val)
    
    cond_downside = regime_conditional_correlations(train, val, "downside_excursion_24h", 300)
    cond_upside = regime_conditional_correlations(train, val, "upside_excursion_24h", 300)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Regime and Distribution Analysis: Train vs Validation\n\n")
        f.write("## 1. Feature Distribution Shift\n\n")
        f.write(df_to_markdown(dist_df))
        f.write("\n\n")
        
        f.write("## 2. Regime-Conditional Correlations (Spearman at 300m)\n\n")
        f.write("We evaluate the raw correlation of the strongest Train variables against forward returns, conditioned on the trailing regime.\n\n")
        
        f.write("### A. Downside Excursion\n\n")
        f.write(df_to_markdown(cond_downside))
        f.write("\n\n")
        
        f.write("### B. Upside Excursion\n\n")
        f.write(df_to_markdown(cond_upside))
        f.write("\n\n")
        
    print(f"Analysis complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_analysis()
