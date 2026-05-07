import pandas as pd
import numpy as np
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"

def analyze_divergence():
    if not os.path.exists(ARCHIVE_PATH):
        print(f"Archive not found at {ARCHIVE_PATH}. Let the engine run longer.")
        return

    cols = [
        "timestamp", "symbol", "regime", "vol_bucket", "half_life", 
        "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"
    ]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    
    # Filter out warmup/initialization rows without valid regimes
    # (e.g. earlier legacy rows that lacked the full coordinate system)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    
    if df.empty:
        print("Not enough structured physics data yet. Keep accumulating.")
        return

    df["timestamp"] = pd.to_datetime(df["timestamp"], unit="s")
    
    print("\n" + "="*80)
    print("🔭 PHASE 2B.5: EXECUTION PHYSICS OBSERVATORY & RECURRENCE ANALYSIS")
    print("="*80)
    
    print("\n--- 1. Symbol Divergence Profiles (Topology Stability) ---")
    sym_group = df.groupby("symbol").agg(
        samples=("divergence", "count"),
        mean_div=("divergence", "mean"),
        std_div=("divergence", "std"),
        mean_noise=("noise_floor", "mean"),
        mean_gross=("gross_move", "mean")
    ).round(6)
    print(sym_group)
    
    print("\n--- 2. Regime-Conditioned Summaries (Environmental Coherence) ---")
    regime_group = df.groupby("regime").agg(
        samples=("divergence", "count"),
        mean_div=("divergence", "mean"),
        std_div=("divergence", "std"),
        mean_micro_exp=("micro_exp", "mean")
    ).round(6)
    print(regime_group)
    
    print("\n--- 3. Compressibility Footprint (Divergence vs Microstructure Hostility) ---")
    # Identify high noise vs low noise environments and their divergence
    noise_median = df["noise_floor"].median()
    df["noise_state"] = np.where(df["noise_floor"] > noise_median, "High Hostility", "Low Hostility")
    comp_group = df.groupby(["symbol", "noise_state"]).agg(
        mean_div=("divergence", "mean"),
        mean_micro_exp=("micro_exp", "mean")
    ).round(6)
    print(comp_group)

if __name__ == "__main__":
    analyze_divergence()
