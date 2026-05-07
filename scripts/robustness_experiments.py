import pandas as pd
import numpy as np
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"

def run_robustness_experiments():
    if not os.path.exists(ARCHIVE_PATH):
        print("Archive not found.")
        return

    cols = ["timestamp", "symbol", "regime", "vol_bucket", "half_life", 
            "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    df = df.sort_values(by=["symbol", "timestamp"]).copy()

    if len(df) < 50:
        print("Insufficient data for robustness testing.")
        return

    print("\n" + "="*80)
    print("🧪 PRE-PHASE 2D: ECOLOGICAL ROBUSTNESS & NULL BASELINES")
    print("="*80)

    # 1. Null Policy Baselines
    print("\n--- 1. Null Policy Baselines ---")
    
    baseline_exposure = df['divergence'].sum()
    
    # Null A: Random Suppression (Coin Flip)
    np.random.seed(42)
    random_sizes = np.random.choice([0.0, 1.0], size=len(df))
    random_exposure = (df['divergence'] * random_sizes).sum()
    
    # Null B: Naive Volatility Suppression (Suppress if HighVolatilityNoise)
    naive_sizes = np.where(df['regime'] == 'HighVolatilityNoise', 0.0, 1.0)
    naive_exposure = (df['divergence'] * naive_sizes).sum()

    # Null C: Fixed Threshold Suppression (Suppress if Divergence > 0.05)
    fixed_sizes = np.where(df['divergence'] > 0.05, 0.0, 1.0)
    fixed_exposure = (df['divergence'] * fixed_sizes).sum()

    print(f"Random Suppression Survival Gain       : {1.0 - (random_exposure / baseline_exposure):+.1%}")
    print(f"Naive Regime Suppression Survival Gain : {1.0 - (naive_exposure / baseline_exposure):+.1%}")
    print(f"Fixed Threshold Suppression Gain       : {1.0 - (fixed_exposure / baseline_exposure):+.1%}")
    
    # 2. Temporal Holdout Test (First 50% vs Last 50%)
    print("\n--- 2. Temporal Holdout Stationarity ---")
    
    midpoint = len(df) // 2
    first_half = df.iloc[:midpoint]
    second_half = df.iloc[midpoint:]
    
    print(f"Early Archive (N={len(first_half)}) Mean Divergence: {first_half['divergence'].mean():.4f}")
    print(f"Later Archive (N={len(second_half)}) Mean Divergence: {second_half['divergence'].mean():.4f}")
    
    # 3. Cross-Asset Stationarity
    print("\n--- 3. Cross-Asset Topology Variance ---")
    for sym, group in df.groupby('symbol'):
        print(f"  {sym} Mean Divergence : {group['divergence'].mean():.4f} | StdDev: {group['divergence'].std():.4f}")

if __name__ == "__main__":
    run_robustness_experiments()
