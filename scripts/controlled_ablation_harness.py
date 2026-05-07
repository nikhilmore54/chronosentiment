import pandas as pd
import numpy as np
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"

def run_ablation_harness():
    if not os.path.exists(ARCHIVE_PATH):
        print("Archive not found.")
        return

    cols = ["timestamp", "symbol", "regime", "vol_bucket", "half_life", 
            "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    df = df.sort_values(by=["symbol", "timestamp"]).copy()

    if len(df) < 50:
        print("Insufficient data for ablation.")
        return

    print("\n" + "="*80)
    print("🔬 PHASE 2B.6: CONTROLLED STATISTICAL ABLATION HARNESS")
    print("="*80)

    # Calculate Derivatives
    df['hostility_accel'] = df.groupby('symbol')['divergence'].diff(periods=2).fillna(0)
    df['noise_ratio'] = np.where(df['gross_move'] > 0, df['noise_floor'] / df['gross_move'], 0)
    df['compression_vel'] = df.groupby('symbol')['noise_ratio'].diff(periods=2).fillna(0)

    # Determine "Actual Topology Collapse" (Future Target)
    # Target: Will divergence exceed 0.06 in the next 3 ticks?
    COLLAPSE_THRESHOLD = 0.06
    df['is_collapsed'] = df['divergence'] >= COLLAPSE_THRESHOLD
    df['future_collapse'] = df.groupby('symbol')['is_collapsed'].shift(-1) | \
                            df.groupby('symbol')['is_collapsed'].shift(-2) | \
                            df.groupby('symbol')['is_collapsed'].shift(-3)

    # Drop NaNs from shift
    valid_df = df.dropna(subset=['future_collapse']).copy()
    valid_df['future_collapse'] = valid_df['future_collapse'].astype(bool)

    baseline_exposure = valid_df['divergence'].sum()
    
    # --- MODEL DEFINITIONS ---
    
    # Model A: Raw Divergence No-Action (The Environment)
    exposure_A = baseline_exposure
    
    # Model B: Static Divergence Thresholding (The Permanent Null)
    # Suppress if divergence > 0.05
    size_B = np.where(valid_df['divergence'] > 0.05, 0.0, 1.0)
    exposure_B = (valid_df['divergence'] * size_B).sum()
    survival_gain_B = 1.0 - (exposure_B / baseline_exposure)
    
    # Model C: Derivatives Lift
    # Suppress if divergence > 0.05 OR (hostility accelerating sharply)
    size_C = np.where((valid_df['divergence'] > 0.05) | (valid_df['hostility_accel'] > 0.02), 0.0, 1.0)
    exposure_C = (valid_df['divergence'] * size_C).sum()
    survival_gain_C = 1.0 - (exposure_C / baseline_exposure)

    # --- RESULTS ---
    print(f"\nTotal Ticks Evaluated: {len(valid_df)}")
    print(f"Total Hostility Exposure (Model A): {baseline_exposure:.4f}\n")

    print("--- 1. SURVIVABILITY LIFT ---")
    print(f"Model B (Thresholding) Survival Gain : {survival_gain_B:+.1%}")
    print(f"Model C (Derivatives)  Survival Gain : {survival_gain_C:+.1%}")
    
    incremental_lift_C = survival_gain_C - survival_gain_B
    print(f"Incremental Lift (Model C over B)    : {incremental_lift_C:+.1%}")

    # --- 2. REGIME SEGMENTATION ---
    print("\n--- 2. REGIME SEGMENTATION (MODEL B vs MODEL C) ---")
    
    # Split by High vs Low Hostility (based on noise_ratio)
    median_noise = valid_df['noise_ratio'].median()
    low_hostility = valid_df[valid_df['noise_ratio'] <= median_noise]
    high_hostility = valid_df[valid_df['noise_ratio'] > median_noise]

    def measure_regime(segment, name):
        if segment.empty: return
        base_exp = segment['divergence'].sum()
        
        s_B = np.where(segment['divergence'] > 0.05, 0.0, 1.0)
        s_C = np.where((segment['divergence'] > 0.05) | (segment['hostility_accel'] > 0.02), 0.0, 1.0)
        
        gain_B = 1.0 - ((segment['divergence'] * s_B).sum() / base_exp)
        gain_C = 1.0 - ((segment['divergence'] * s_C).sum() / base_exp)
        
        print(f"  {name} Regime (N={len(segment)}):")
        print(f"    Model B Gain: {gain_B:+.1%} | Model C Gain: {gain_C:+.1%} | Lift: {gain_C - gain_B:+.1%}")

    measure_regime(low_hostility, "Low Hostility (Clean)")
    measure_regime(high_hostility, "High Hostility (Choppy)")

    # --- 3. PREDICTIVE PRECISION ---
    print("\n--- 3. PREDICTIVE PRECISION ---")
    # Did Model C suppress transmissible ticks that Model B would have captured?
    # False Positive = Suppressed (size=0) BUT future_collapse == False
    
    fp_B = len(valid_df[(size_B == 0.0) & (valid_df['future_collapse'] == False)])
    fp_C = len(valid_df[(size_C == 0.0) & (valid_df['future_collapse'] == False)])
    
    tp_B = len(valid_df[(size_B == 0.0) & (valid_df['future_collapse'] == True)])
    tp_C = len(valid_df[(size_C == 0.0) & (valid_df['future_collapse'] == True)])

    print(f"Model B - True Collapses Avoided: {tp_B} | False Suppressions: {fp_B}")
    print(f"Model C - True Collapses Avoided: {tp_C} | False Suppressions: {fp_C}")
    print(f"Model C vs B: Avoided {tp_C - tp_B} more collapses, but suffered {fp_C - fp_B} more false suppressions.")

if __name__ == "__main__":
    run_ablation_harness()
