import pandas as pd
import numpy as np
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "transition_discovery_v0_1_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "transition_matrix_train_v0_1.md"

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

def run_matrix():
    df = pd.read_csv(DATA_FILE)
    n_train = int(len(df) * 0.6)
    train = df.iloc[:n_train].copy()
    
    # 1. Define Categories based on Train statistics
    vol_median = train['volatility_1m_std_24h'].median()
    
    def get_vol_transition(row):
        past_high = row['past_volatility_24h'] > vol_median
        curr_high = row['volatility_1m_std_24h'] > vol_median
        if not past_high and curr_high: return "LOW -> HIGH"
        if past_high and not curr_high: return "HIGH -> LOW"
        if past_high and curr_high: return "MAINTAIN_HIGH"
        return "MAINTAIN_LOW"
        
    def get_dir_transition(row):
        past_dir = row['past_trend_dir']
        curr_dir = row['trend_dir']
        if past_dir == 1 and curr_dir == -1: return "UP -> DOWN"
        if past_dir == -1 and curr_dir == 1: return "DOWN -> UP"
        if past_dir == 1 and curr_dir == 1: return "MAINTAIN_UP"
        if past_dir == -1 and curr_dir == -1: return "MAINTAIN_DOWN"
        return "OTHER"
        
    train['vol_trans'] = train.apply(get_vol_transition, axis=1)
    train['dir_trans'] = train.apply(get_dir_transition, axis=1)
    
    train['block'] = train['timestamp'] // 43200000
    
    horizons = [60, 120, 300]
    targets = ["reversal", "vol_expansion", "excursion_exceeded"]
    
    results = []
    
    for target_base in targets:
        for h in horizons:
            target_col = f"{target_base}_{h}m"
            baseline_mean = train[target_col].mean()
            
            # Analyze Volatility Transitions
            for trans_val in ["LOW -> HIGH", "HIGH -> LOW", "MAINTAIN_HIGH", "MAINTAIN_LOW"]:
                subset = train[train['vol_trans'] == trans_val]
                if len(subset) == 0: continue
                
                target_mean = subset[target_col].mean()
                diff = target_mean - baseline_mean
                
                block_means = subset.groupby('block')[target_col].mean()
                if diff > 0:
                    stable_blocks = (block_means > baseline_mean).mean()
                else:
                    stable_blocks = (block_means < baseline_mean).mean()
                    
                results.append({
                    "Target": f"{target_base.capitalize()}",
                    "Horizon": f"{h}m",
                    "Transition_Type": "Volatility",
                    "Transition": trans_val,
                    "N": len(subset),
                    "Baseline_%": f"{baseline_mean*100:.1f}%",
                    "Cond_%": f"{target_mean*100:.1f}%",
                    "Diff": f"{diff*100:+.1f}%",
                    "Block_Stability": f"{stable_blocks*100:.1f}%"
                })
                
            # Analyze Directional Transitions
            for trans_val in ["UP -> DOWN", "DOWN -> UP", "MAINTAIN_UP", "MAINTAIN_DOWN"]:
                subset = train[train['dir_trans'] == trans_val]
                if len(subset) == 0: continue
                
                target_mean = subset[target_col].mean()
                diff = target_mean - baseline_mean
                
                block_means = subset.groupby('block')[target_col].mean()
                if diff > 0:
                    stable_blocks = (block_means > baseline_mean).mean()
                else:
                    stable_blocks = (block_means < baseline_mean).mean()
                    
                results.append({
                    "Target": f"{target_base.capitalize()}",
                    "Horizon": f"{h}m",
                    "Transition_Type": "Direction",
                    "Transition": trans_val,
                    "N": len(subset),
                    "Baseline_%": f"{baseline_mean*100:.1f}%",
                    "Cond_%": f"{target_mean*100:.1f}%",
                    "Diff": f"{diff*100:+.1f}%",
                    "Block_Stability": f"{stable_blocks*100:.1f}%"
                })
                
    res_df = pd.DataFrame(results)
    
    with open(REPORT_FILE, "w") as f:
        f.write("# Crypto 24H State Transition Research v0.1\n")
        f.write("## Stage A: Predeclared Transition/Outcome Matrix (Train Split)\n\n")
        f.write("This matrix evaluates structural deviations from the baseline probability for each behavioural target. ")
        f.write("`Block_Stability` represents the percentage of 12h chronological blocks where the conditional mean correctly deviated from the baseline mean in the direction of the aggregate deviation.\n\n")
        
        for t in ["Reversal", "Vol_expansion", "Excursion_exceeded"]:
            f.write(f"### Target: {t}\n")
            f.write(df_to_markdown(res_df[res_df['Target'] == t].drop(columns=['Target'])))
            f.write("\n")
            
    print(f"Matrix generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_matrix()
