import pandas as pd
import numpy as np
from scipy.stats import spearmanr
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "ic_discovery_v0.1_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "expanded_rh_report.md"

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

def get_block_stats(df):
    if len(df) == 0:
        return 0, 0, 0
    df = df.copy()
    df['block'] = df['timestamp'] // 43200000
    block_counts = df.groupby('block').size()
    
    n_blocks = len(block_counts)
    min_obs = block_counts.min() if n_blocks > 0 else 0
    med_obs = block_counts.median() if n_blocks > 0 else 0
    
    return n_blocks, min_obs, med_obs

def evaluate_regime_ic(df, ic_values, target):
    valid = df[[target, 'timestamp']].copy()
    valid['ic'] = ic_values
    valid = valid.dropna()
    
    n_samples = len(valid)
    n_blocks, min_obs, med_obs = get_block_stats(valid)
    
    if n_blocks < 6:
        return None, n_samples, n_blocks, min_obs, med_obs
        
    x = valid['ic'].values
    y = valid[target].values
    
    if np.std(x) == 0 or np.std(y) == 0:
        return None, n_samples, n_blocks, min_obs, med_obs
        
    spearman_corr, _ = spearmanr(x, y)
    
    return spearman_corr, n_samples, n_blocks, min_obs, med_obs

def run_rh_eval():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    
    train = df.iloc[:n_train].copy()
    
    # Calculate global Train statistics for z-scores
    u_mean = train['upside_excursion_24h'].mean()
    u_std = train['upside_excursion_24h'].std()
    
    d_mean = train['downside_excursion_24h'].mean()
    d_std = train['downside_excursion_24h'].std()
    
    # Calculate state medians for regime thresholds
    vol_median = train['volatility_1m_std_24h'].median()
    pers_median = train['persistence_240m'].median()
    
    # Pre-calculate R1, R2, R3
    train['z_U'] = (train['upside_excursion_24h'] - u_mean) / u_std
    train['z_D'] = (train['downside_excursion_24h'] - d_mean) / d_std
    
    train['R1'] = -train['z_U']
    train['R2'] = -train['z_D']
    train['R3'] = -(train['z_U'] + train['z_D'])
    
    # Filter 0 observations for the directional split
    train_dir = train[train['trend_return_60m'] != 0].copy()
    
    results = []
    
    # Define hypotheses evaluator
    def eval_hypothesis(name, regime_col, df_filtered):
        total_N = len(train) # reference total N for percentage
        
        for regime_val in df_filtered[regime_col].unique():
            subset = df_filtered[df_filtered[regime_col] == regime_val]
            
            for c in ["R1", "R2", "R3"]:
                for h in [120, 300]:
                    target = f"ret_{h}m"
                    corr, n_samp, n_blks, min_obs, med_obs = evaluate_regime_ic(subset, subset[c], target)
                    
                    if corr is None:
                        corr_str = "INSUFFICIENT_BLOCK_COVERAGE"
                    else:
                        corr_str = f"{corr:.4f}"
                        
                    results.append({
                        "Hypothesis": name,
                        "Regime": regime_val,
                        "IC": c,
                        "Horizon": f"{h}m",
                        "N": n_samp,
                        "N %": f"{n_samp/total_N:.1%}",
                        "Blocks": n_blks,
                        "Min Obs/Blk": int(min_obs),
                        "Med Obs/Blk": int(med_obs),
                        "Spearman": corr_str
                    })
    
    # ----------------------------------------------------
    # RH-1: Volatility State x Short-Term Direction
    # ----------------------------------------------------
    rh1_regimes = []
    for _, row in train_dir.iterrows():
        vol_str = "HIGH_VOL" if row['volatility_1m_std_24h'] >= vol_median else "LOW_VOL"
        dir_str = "UP" if row['trend_return_60m'] > 0 else "DOWN"
        rh1_regimes.append(f"{vol_str}_{dir_str}")
    train_dir['RH1_Regime'] = rh1_regimes
    
    eval_hypothesis("RH-1", "RH1_Regime", train_dir)
    
    # ----------------------------------------------------
    # RH-2: Volatility Level x Volatility Dynamics
    # ----------------------------------------------------
    train_dyn = train.dropna(subset=['volatility_ratio_240m_24h']).copy()
    rh2_regimes = []
    for _, row in train_dyn.iterrows():
        vol_str = "HIGH_VOL" if row['volatility_1m_std_24h'] >= vol_median else "LOW_VOL"
        dyn_str = "EXPANDING" if row['volatility_ratio_240m_24h'] >= 1 else "CONTRACTING"
        rh2_regimes.append(f"{vol_str}_{dyn_str}")
    train_dyn['RH2_Regime'] = rh2_regimes
    
    eval_hypothesis("RH-2", "RH2_Regime", train_dyn)
    
    # ----------------------------------------------------
    # RH-3: Path Efficiency x Short-Term Direction
    # ----------------------------------------------------
    train_pers = train_dir.dropna(subset=['persistence_240m']).copy()
    rh3_regimes = []
    for _, row in train_pers.iterrows():
        pers_str = "TREND" if row['persistence_240m'] >= pers_median else "CHOP"
        dir_str = "UP" if row['trend_return_60m'] > 0 else "DOWN"
        rh3_regimes.append(f"{pers_str}_{dir_str}")
    train_pers['RH3_Regime'] = rh3_regimes
    
    eval_hypothesis("RH-3", "RH3_Regime", train_pers)
    
    res_df = pd.DataFrame(results)
    
    # Order by hypothesis
    with open(REPORT_FILE, "w") as f:
        f.write("# Expanded Regime Matrix v0.1: Train Discovery\n\n")
        
        f.write("## RH-1: Volatility State × Short-Term Direction\n")
        f.write(df_to_markdown(res_df[res_df['Hypothesis'] == 'RH-1'].drop(columns=['Hypothesis'])))
        f.write("\n")
        
        f.write("## RH-2: Volatility Level × Volatility Dynamics\n")
        f.write(df_to_markdown(res_df[res_df['Hypothesis'] == 'RH-2'].drop(columns=['Hypothesis'])))
        f.write("\n")
        
        f.write("## RH-3: Path Efficiency × Short-Term Direction\n")
        f.write(df_to_markdown(res_df[res_df['Hypothesis'] == 'RH-3'].drop(columns=['Hypothesis'])))
        f.write("\n")
        
    print(f"RH Evaluation complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_rh_eval()
