import pandas as pd
import numpy as np
import json
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"
PROFILES_PATH = "scripts/ecology_profiles.json"

def calculate_collapse_risk():
    if not os.path.exists(ARCHIVE_PATH):
        print(f"Archive not found at {ARCHIVE_PATH}.")
        return

    # Load ecology profiles
    with open(PROFILES_PATH, 'r') as f:
        profiles = json.load(f)

    cols = [
        "timestamp", "symbol", "regime", "vol_bucket", "half_life", 
        "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"
    ]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    df["timestamp"] = pd.to_datetime(df["timestamp"], unit="s")
    
    if df.empty:
        print("Not enough data to model collapse.")
        return

    print("\n" + "="*80)
    print("⚠️ PHASE 2C: COLLAPSE FORECASTER & EARLY-WARNING SYSTEM")
    print("="*80)

    # Process each asset
    latest_risks = []
    
    for sym, group in df.groupby("symbol"):
        group = group.sort_values(by="timestamp").copy()
        
        # Calculate trailing indicators
        # 1. Hostility Acceleration (change in divergence over last 3 ticks)
        group['div_roc'] = group['divergence'].diff(periods=2)
        
        # 2. Survivable Envelope Decay (rate of shrinking micro_exp)
        group['micro_exp_decay'] = -group['micro_exp'].diff(periods=2) / group['micro_exp'].shift(2)
        
        # 3. Compression Velocity (noise floor expanding faster than gross move)
        group['noise_ratio'] = group['noise_floor'] / group['gross_move']
        group['compression_vel'] = group['noise_ratio'].diff(periods=2)

        # Drop NaNs from rolling calc
        valid = group.dropna().copy()
        if valid.empty:
            continue
            
        latest = valid.iloc[-1]
        
        # Dynamic weights
        w1, w2, w3 = 0.5, 0.3, 0.2
        
        # Normalize the metrics roughly for the score
        hostility_accel = np.clip(latest['div_roc'] * 10, -1, 1)
        envelope_decay = np.clip(latest['micro_exp_decay'], -1, 1)
        compression_vel = np.clip(latest['compression_vel'] * 5, -1, 1)
        
        # Calculate raw risk score
        risk_score = (w1 * hostility_accel) + (w2 * envelope_decay) + (w3 * compression_vel)
        # Shift to 0-1 range roughly
        risk_score = np.clip((risk_score + 0.5), 0.0, 1.0)
        
        # Apply ecology profile adjustments
        ecology = profiles.get(sym, {})
        base_risk = ecology.get("collapse_risk_base", 0.1)
        
        # Final Ecology-Aware Risk
        final_risk = (risk_score * 0.7) + (base_risk * 0.3)
        
        status = "CRITICAL" if final_risk > 0.7 else "ELEVATED" if final_risk > 0.4 else "STABLE"
        
        latest_risks.append({
            'Symbol': sym,
            'Ecology': ecology.get('type', 'unknown'),
            'Divergence': f"{latest['divergence']:.3f}",
            'Hostility Accel': f"{hostility_accel:+.2f}",
            'Envelope Decay': f"{envelope_decay:+.2f}",
            'Collapse Risk': final_risk,
            'Status': status,
            'Policy': ecology.get('policy', 'none')
        })

    # Print Report
    risk_df = pd.DataFrame(latest_risks).sort_values(by='Collapse Risk', ascending=False)
    
    for _, row in risk_df.iterrows():
        print(f"\n[{row['Status']}] {row['Symbol']} - {row['Ecology'].upper().replace('_', ' ')}")
        print(f"  Collapse Risk Score : {row['Collapse Risk']:.1%}")
        print(f"  Current Divergence  : {row['Divergence']}")
        print(f"  Hostility Accel     : {row['Hostility Accel']}")
        print(f"  Envelope Decay      : {row['Envelope Decay']}")
        print(f"  Recommended Policy  : {row['Policy'].upper().replace('_', ' ')}")

if __name__ == "__main__":
    calculate_collapse_risk()
