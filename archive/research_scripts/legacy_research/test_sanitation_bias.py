#!/usr/bin/env python3
"""
ChronoSentiment — Sanitation Bias Tester
Quantifies if the 'keep="last"' policy introduces a hidden signal bias compared to 'keep="first"'.
"""

import pandas as pd
from pathlib import Path
import subprocess

def run_test():
    csv_path = Path("archive/physics_divergence.csv")
    if not csv_path.exists():
        print("❌ Archive not found.")
        return

    print("🔍 Loading archive for bias testing...")
    cols = ["ts", "sym", "regime", "vol_bucket", "half_life", "legacy_exp", "gross", "noise", "micro_exp", "divergence"]
    df_raw = pd.read_csv(csv_path, names=cols)
    
    # Identify duplicate candidates
    dupes = df_raw.duplicated(subset=["ts", "sym"], keep=False)
    dupe_count = dupes.sum()
    
    if dupe_count == 0:
        print("✅ No duplicates found in archive. Causal lattice is already unique.")
        return

    print(f"📊 Found {dupe_count} duplicate rows (candidates for bias).")

    # Create two versions
    df_first = df_raw.drop_duplicates(subset=["ts", "sym"], keep="first")
    df_last = df_raw.drop_duplicates(subset=["ts", "sym"], keep="last")

    def get_metrics(df, label):
        # Primitive ablation logic to compare survival
        # (Simplified version of controlled_ablation_harness.py)
        model_b_thresh = 0.05
        df['suppress_b'] = df['divergence'] < model_b_thresh
        df['collapse'] = df['micro_exp'] < 0.001 # Primitive collapse proxy
        
        # Avoided Collapses (True Positives for suppression)
        av_coll = (df['suppress_b'] & df['collapse']).sum()
        false_supp = (df['suppress_b'] & ~df['collapse']).sum()
        
        return {
            "label": label,
            "count": len(df),
            "av_coll": av_coll,
            "false_supp": false_supp,
            "mean_div": df['divergence'].mean()
        }

    m_first = get_metrics(df_first, "Keep First (Live-Biased)")
    m_last = get_metrics(df_last, "Keep Last (Replay-Biased)")

    print("\n--- SANITATION BIAS RESULTS ---")
    print(f"Policy: {m_first['label']} | Ticks: {m_first['count']} | Mean Div: {m_first['mean_div']:.6f} | Avoided Coll: {m_first['av_coll']}")
    print(f"Policy: {m_last['label']} | Ticks: {m_last['count']} | Mean Div: {m_last['mean_div']:.6f} | Avoided Coll: {m_last['av_coll']}")
    
    div_delta = abs(m_first['mean_div'] - m_last['mean_div'])
    coll_delta = abs(m_first['av_coll'] - m_last['av_coll'])
    
    print(f"\nDelta Mean Divergence: {div_delta:.8f}")
    print(f"Delta Avoided Collapses: {coll_delta}")

    if div_delta < 0.0001 and coll_delta == 0:
        print("\n✅ VERDICT: Bias is NEGLIGIBLE. The lattice is causally stable.")
    else:
        print("\n⚠️ WARNING: Bias DETECTED. Sanitation policy matters.")

if __name__ == "__main__":
    run_test()
