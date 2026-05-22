import pandas as pd
import numpy as np
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"

def classify_execution_state(row, thresholds):
    # Phase A: Expansive (High Survivability, Low Divergence)
    # Phase B: Compressing (Moderate Survivability, Rising Divergence)
    # Phase C: Collapsed (Negligible Survivability, Spiking Divergence)
    
    div = row['divergence']
    
    if div >= thresholds['collapse']:
        return 'COLLAPSED'
    elif div >= thresholds['compressing']:
        return 'COMPRESSING'
    else:
        return 'EXPANSIVE'

def build_transition_model():
    if not os.path.exists(ARCHIVE_PATH):
        print(f"Archive not found at {ARCHIVE_PATH}.")
        return

    cols = [
        "timestamp", "symbol", "regime", "vol_bucket", "half_life", 
        "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"
    ]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    df["timestamp"] = pd.to_datetime(df["timestamp"], unit="s")
    df = df.sort_values(by=["symbol", "timestamp"])

    if df.empty:
        print("Not enough data to model transitions.")
        return

    # Define dynamic thresholds based on the environment's own geometry
    # E.g., Top 20% of divergence is COLLAPSED, next 30% is COMPRESSING, bottom 50% is EXPANSIVE
    div_75 = df['divergence'].quantile(0.75)
    div_50 = df['divergence'].quantile(0.50)
    
    # If the environment is completely un-distorted (all 0.0), provide fallback thresholds
    if div_75 == 0.0:
        div_75 = 0.20
        div_50 = 0.08
        
    thresholds = {'collapse': div_75, 'compressing': div_50}

    df['exec_state'] = df.apply(lambda r: classify_execution_state(r, thresholds), axis=1)

    print("\n" + "="*80)
    print("🕸️ PHASE 2C: EXECUTION TOPOLOGY TRANSITION MODEL")
    print("="*80)
    print(f"Thresholds -> Collapsed: >={thresholds['collapse']:.3f} | Compressing: >={thresholds['compressing']:.3f}\n")

    # 1. State Distribution
    print("--- 1. Current State Distribution ---")
    print(df['exec_state'].value_counts(normalize=True).round(3))
    
    # 2. Transition Probability Matrix
    print("\n--- 2. Transition Probability Matrix ---")
    transitions = []
    for sym, group in df.groupby('symbol'):
        group['next_state'] = group['exec_state'].shift(-1)
        valid_transitions = group.dropna(subset=['next_state'])
        for _, row in valid_transitions.iterrows():
            transitions.append({
                'symbol': sym,
                'from_state': row['exec_state'],
                'to_state': row['next_state']
            })
            
    if transitions:
        trans_df = pd.DataFrame(transitions)
        
        # Global Transition Matrix
        global_tm = pd.crosstab(trans_df['from_state'], trans_df['to_state'], normalize='index').round(3)
        print("\nGlobal Transition Probabilities:")
        print(global_tm)
        
        # Asset Specific Transition Matrices
        print("\nAsset-Specific Collapse Probabilities (P(Collapsed | State)):")
        for sym in trans_df['symbol'].unique():
            sym_tm = pd.crosstab(
                trans_df[trans_df['symbol'] == sym]['from_state'], 
                trans_df[trans_df['symbol'] == sym]['to_state'], 
                normalize='index'
            )
            if 'COLLAPSED' in sym_tm.columns:
                print(f"  {sym}:")
                for state in sym_tm.index:
                    prob = sym_tm.loc[state, 'COLLAPSED']
                    print(f"    {state} -> COLLAPSED: {prob:.1%}")
    
    # 3. Persistence Duration
    print("\n--- 3. State Persistence (Average sequential observations) ---")
    for sym, group in df.groupby('symbol'):
        # Calculate sequential run lengths of the same state
        state_runs = (group['exec_state'] != group['exec_state'].shift(1)).cumsum().rename('run_id')
        run_lengths = group.groupby(['exec_state', state_runs]).size().reset_index(name='duration')
        avg_durations = run_lengths.groupby('exec_state')['duration'].mean().round(2)
        print(f"\n{sym}:")
        print(avg_durations.to_string())

if __name__ == "__main__":
    build_transition_model()
