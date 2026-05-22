import pandas as pd
import numpy as np
import json
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"
PROFILES_PATH = "scripts/ecology_profiles.json"

def run_counterfactual_replay():
    if not os.path.exists(ARCHIVE_PATH) or not os.path.exists(PROFILES_PATH):
        print("Required files (archive or ecology profiles) not found.")
        return

    with open(PROFILES_PATH, 'r') as f:
        profiles = json.load(f)

    cols = [
        "timestamp", "symbol", "regime", "vol_bucket", "half_life", 
        "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"
    ]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    
    if df.empty:
        print("Archive is empty.")
        return

    # To calculate derivatives, we need to sort by symbol and time
    df = df.sort_values(by=["symbol", "timestamp"]).copy()

    # Define threshold for "Actual Topology Collapse" (e.g., Divergence >= 0.06)
    # This is what we are trying to predict
    COLLAPSE_THRESHOLD = 0.06
    
    print("\n" + "="*80)
    print("🔬 STAGE 2: OFFLINE COUNTERFACTUAL REPLAY ENGINE")
    print("="*80)

    # 1. Compute Derivatives & Forecasts iteratively
    # We simulate sequential reception of ticks
    df['hostility_accel'] = df.groupby('symbol')['divergence'].diff(periods=2).fillna(0)
    
    # micro_exp decay (protect against div by 0)
    shifted_micro = df.groupby('symbol')['micro_exp'].shift(2)
    df['envelope_decay'] = np.where(shifted_micro > 0, 
                                    -df.groupby('symbol')['micro_exp'].diff(periods=2) / shifted_micro, 
                                    0)
    
    # compression velocity
    df['noise_ratio'] = np.where(df['gross_move'] > 0, df['noise_floor'] / df['gross_move'], 0)
    df['compression_vel'] = df.groupby('symbol')['noise_ratio'].diff(periods=2).fillna(0)

    # 2. Compute Risk Score & Shadow Policy
    w1, w2, w3 = 0.5, 0.3, 0.2
    
    # Arrays to hold results
    shadow_actions = []
    shadow_sizings = []
    risk_scores = []
    actual_states = []

    for _, row in df.iterrows():
        sym = row['symbol']
        ecology = profiles.get(sym, {})
        base_risk = ecology.get("collapse_risk_base", 0.1)
        ecology_type = ecology.get("type", "unknown")
        
        # Raw metrics
        h_accel = np.clip(row['hostility_accel'] * 10, -1, 1)
        e_decay = np.clip(row['envelope_decay'], -1, 1)
        c_vel = np.clip(row['compression_vel'] * 5, -1, 1)
        
        # Risk Math
        raw_risk = (w1 * h_accel) + (w2 * e_decay) + (w3 * c_vel)
        risk = np.clip((raw_risk + 0.5), 0.0, 1.0)
        final_risk = (risk * 0.7) + (base_risk * 0.3)
        risk_scores.append(final_risk)
        
        # Determine actual state (hindsight)
        state = "COLLAPSED" if row['divergence'] >= COLLAPSE_THRESHOLD else "COMPRESSING" if row['divergence'] >= 0.04 else "EXPANSIVE"
        actual_states.append(state)
        
        # Policy Engine
        action = "MAINTAIN"
        size = 1.0
        
        if ecology_type == "fragile_explosive":
            if final_risk > 0.45:
                action, size = "REDUCE_70", 0.3
            elif state == "COMPRESSING":
                action, size = "REDUCE_50", 0.5
        elif ecology_type == "collapse_trap":
            if state == "COMPRESSING" or final_risk > 0.60:
                action, size = "LIQUIDATE", 0.0
        elif ecology_type == "elastic_resilient":
            if final_risk > 0.85:
                action, size = "REDUCE_50", 0.5

        shadow_actions.append(action)
        shadow_sizings.append(size)

    df['collapse_risk'] = risk_scores
    df['actual_state'] = actual_states
    df['shadow_action'] = shadow_actions
    df['shadow_sizing'] = shadow_sizings

    # 3. Calculate Counterfactual Metrics
    
    # A. Forecast Accuracy
    # Did High Risk (>0.45) precede an actual COLLAPSED state within the next 3 ticks?
    df['future_collapsed'] = df.groupby('symbol')['actual_state'].shift(-1).isin(['COLLAPSED']) | \
                             df.groupby('symbol')['actual_state'].shift(-2).isin(['COLLAPSED']) | \
                             df.groupby('symbol')['actual_state'].shift(-3).isin(['COLLAPSED'])
    
    high_risk_flags = df[df['collapse_risk'] > 0.45]
    if len(high_risk_flags) > 0:
        forecast_accuracy = high_risk_flags['future_collapsed'].mean()
    else:
        forecast_accuracy = 0.0

    # B. Catastrophic Avoidance
    # How many times was 'LIQUIDATE' or 'REDUCE_70' called right before or during COLLAPSED?
    avoided_catastrophes = len(df[
        (df['actual_state'] == 'COLLAPSED') & 
        (df['shadow_action'].isin(['LIQUIDATE', 'REDUCE_70']))
    ])

    # C. Opportunity Suppression Cost
    # How many times did we LIQUIDATE or REDUCE_70, but the next 3 ticks remained EXPANSIVE?
    df['future_expansive'] = df.groupby('symbol')['actual_state'].shift(-1).isin(['EXPANSIVE']) & \
                             df.groupby('symbol')['actual_state'].shift(-2).isin(['EXPANSIVE'])
    
    missed_opportunities = len(df[
        (df['shadow_action'].isin(['LIQUIDATE', 'REDUCE_70'])) & 
        (df['future_expansive'] == True)
    ])
    
    # D. Survival Gain (proxy)
    # Sum of divergence (hostility) we AVOIDED interacting with. 
    # High divergence = bad. Avoiding high divergence = Survival Gain.
    baseline_hostility_exposure = df['divergence'].sum()
    shadow_hostility_exposure = (df['divergence'] * df['shadow_sizing']).sum()
    survival_gain_pct = 1.0 - (shadow_hostility_exposure / max(baseline_hostility_exposure, 0.0001))

    print("\n[METRICS OVERVIEW]")
    print(f"Total Ticks Analyzed            : {len(df)}")
    print(f"Forecast Accuracy (True Pos)    : {forecast_accuracy:.1%}")
    print(f"Catastrophic Ticks Avoided      : {avoided_catastrophes} ticks")
    print(f"Opportunity Suppression Cost    : {missed_opportunities} ticks (False Positives)")
    print(f"Estimated Survival Gain (Risk)  : {survival_gain_pct:+.1%} hostility exposure reduced")
    
    print("\n[SAMPLE SHADOW COUNTERFACTUAL LEDGER]")
    sample = df[['symbol', 'divergence', 'actual_state', 'collapse_risk', 'shadow_action']].tail(10)
    print(sample.to_string(index=False))

if __name__ == "__main__":
    run_counterfactual_replay()
