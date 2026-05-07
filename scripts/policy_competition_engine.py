import pandas as pd
import numpy as np
import json
import os

ARCHIVE_PATH = "archive/physics_divergence.csv"

# ---------------------------------------------------------
# POLICY FAMILIES
# ---------------------------------------------------------
class Organism:
    def __init__(self, name):
        self.name = name
    
    def evaluate(self, asset, state, risk_score):
        return "MAINTAIN", 1.0

class DefensiveOrganism(Organism):
    def evaluate(self, asset, state, risk_score):
        if risk_score > 0.40 or state == "COMPRESSING":
            return "LIQUIDATE", 0.0
        return "MAINTAIN", 1.0

class ElasticOrganism(Organism):
    def evaluate(self, asset, state, risk_score):
        if state == "COLLAPSED" and risk_score > 0.70:
            return "REDUCE", 0.5
        return "MAINTAIN", 1.0

class FragilityAwareOrganism(Organism):
    def evaluate(self, asset, state, risk_score):
        if asset == "BTC-USD" and risk_score > 0.45:
            return "REDUCE", 0.3
        elif asset == "ETH-USD" and (state == "COMPRESSING" or risk_score > 0.60):
            return "LIQUIDATE", 0.0
        elif asset == "SOL-USD" and risk_score > 0.85:
            return "REDUCE", 0.5
        return "MAINTAIN", 1.0

def run_policy_competition():
    if not os.path.exists(ARCHIVE_PATH):
        print("Archive not found.")
        return

    cols = ["timestamp", "symbol", "regime", "vol_bucket", "half_life", 
            "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"]
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
    df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
    df = df.sort_values(by=["symbol", "timestamp"]).copy()

    if df.empty:
        return

    COLLAPSE_THRESHOLD = 0.06
    COMPRESSING_THRESHOLD = 0.04

    # Calculate basic physics
    df['hostility_accel'] = df.groupby('symbol')['divergence'].diff(periods=2).fillna(0)
    shifted_micro = df.groupby('symbol')['micro_exp'].shift(2)
    df['envelope_decay'] = np.where(shifted_micro > 0, -df.groupby('symbol')['micro_exp'].diff(periods=2) / shifted_micro, 0)
    df['noise_ratio'] = np.where(df['gross_move'] > 0, df['noise_floor'] / df['gross_move'], 0)
    df['compression_vel'] = df.groupby('symbol')['noise_ratio'].diff(periods=2).fillna(0)

    # Organisms
    organisms = [
        DefensiveOrganism("Defensive (High Suppression)"),
        ElasticOrganism("Elastic (High Tolerance)"),
        FragilityAwareOrganism("Fragility-Aware (Asset Specific)")
    ]

    metrics = {org.name: {'avoided_collapse': 0, 'false_abort': 0, 'survivability_gain_pct': 0.0, 'total_exposure': 0.0} for org in organisms}
    baseline_exposure = 0.0

    print("\n" + "="*80)
    print("🧬 PHASE 2C.2: MULTI-AGENT POLICY COMPETITION")
    print("="*80)

    # Evaluate each tick
    for _, row in df.iterrows():
        sym = row['symbol']
        div = row['divergence']
        
        # Risk Math
        h_accel = np.clip(row['hostility_accel'] * 10, -1, 1)
        e_decay = np.clip(row['envelope_decay'], -1, 1)
        c_vel = np.clip(row['compression_vel'] * 5, -1, 1)
        
        raw_risk = (0.5 * h_accel) + (0.3 * e_decay) + (0.2 * c_vel)
        risk_score = np.clip((raw_risk + 0.5), 0.0, 1.0)
        
        state = "COLLAPSED" if div >= COLLAPSE_THRESHOLD else "COMPRESSING" if div >= COMPRESSING_THRESHOLD else "EXPANSIVE"
        
        baseline_exposure += div

        for org in organisms:
            action, size = org.evaluate(sym, state, risk_score)
            
            # Exposure
            metrics[org.name]['total_exposure'] += (div * size)
            
            # Just rough proxy logic for false aborts / true positives in this pass
            if action in ["LIQUIDATE", "REDUCE"] and state != "COLLAPSED" and risk_score < 0.6:
                metrics[org.name]['false_abort'] += 1
            if action in ["LIQUIDATE", "REDUCE"] and state == "COLLAPSED":
                metrics[org.name]['avoided_collapse'] += 1

    # Print Results
    print(f"\nBaseline Hostility Exposure: {baseline_exposure:.4f}")
    for org in organisms:
        m = metrics[org.name]
        m['survivability_gain_pct'] = 1.0 - (m['total_exposure'] / max(baseline_exposure, 0.0001))
        
        print(f"\n[{org.name}]")
        print(f"  Survival Gain (Exposure Reduction) : {m['survivability_gain_pct']:+.1%}")
        print(f"  Catastrophic Ticks Avoided         : {m['avoided_collapse']}")
        print(f"  Opportunity Suppression (False Pos): {m['false_abort']}")

if __name__ == "__main__":
    run_policy_competition()
