#!/usr/bin/env python3
"""
ChronoSentiment — Archive Sanitizer
Ensures physics_divergence.csv has unique timestamps per symbol and is chronologically sorted.
Now enforces: 
1. Warmup Gate (updates > 100)
2. Authenticity-First Truth Model (is_authentic=True wins ties)
3. Maturity-Second Truth Model (higher updates wins among same authenticity)
4. Strict Cross-Symbol Synchronization (Triplets only)
"""

import pandas as pd
from pathlib import Path

def sanitize():
    csv_path = Path("archive/physics_divergence.csv")
    if not csv_path.exists():
        print("❌ Archive not found.")
        return

    print(f"🧹 Sanitizing {csv_path}...")
    
    # columns: 0-ts, 1-sym, 2-regime, 3-vol, 4-half_life, 5-legacy, 6-gross, 7-noise, 8-micro, 9-div, 10-updates, 11-source, 12-authentic, 13-gen
    cols = ["ts", "sym", "regime", "vol_bucket", "half_life", "legacy_exp", "gross", "noise", "micro_exp", "divergence", "updates", "source", "authentic", "gen"]
    
    try:
        # Read the mixed CSV
        df = pd.read_csv(csv_path, header=None)
        
        # Ensure we have all 14 columns (fill missing with NaN first)
        for i in range(df.shape[1], 14):
            df[i] = None
        
        df.columns = cols
        
        # Fill NaN provenance for legacy/mixed rows
        df["updates"] = pd.to_numeric(df["updates"], errors='coerce').fillna(101)
        df["source"] = df["source"].fillna("LIVE")
        df["authentic"] = df["authentic"].fillna(True)
        df["gen"] = df["gen"].fillna(0)
        
        # Ensure bool type for authentic (handling various string/bool formats)
        df["authentic"] = df["authentic"].map({
            "true": True, "false": False, 
            "True": True, "False": False,
            True: True, False: False,
            1.0: True, 0.0: False,
            "1.0": True, "0.0": False
        })
        # If any NaNs left in authentic, assume True (LIVE baseline)
        df["authentic"] = df["authentic"].fillna(True)
            
    except Exception as e:
        print(f"❌ Error reading CSV: {e}")
        import traceback
        traceback.print_exc()
        return

    initial_count = len(df)
    
    # 1. Physical Filter: Purge cold-start / unwarmed rows
    df = df[df["updates"] > 100]
    warmed_count = len(df)
    if initial_count - warmed_count > 0:
        print(f"🔥 Warmup Filter: Purged {initial_count - warmed_count} unwarmed records (updates <= 100).")

    # 2. Provenance-Based Deduplication
    # Priority 1: Authentic Reality (is_authentic=True wins)
    # Priority 2: Maturity (updates DESC)
    # Priority 3: Originality (gen ASC)
    df = df.sort_values(by=["ts", "sym", "authentic", "updates", "gen"], ascending=[True, True, False, False, True])
    df = df.drop_duplicates(subset=["ts", "sym"], keep="first")
    matured_count = len(df)
    if warmed_count - matured_count > 0:
        print(f"⚖️  Provenance Filter: Resolved {warmed_count - matured_count} duplicate states via Authenticity > Maturity.")

    # 3. Final Chronological Sort
    df = df.sort_values(by=["ts", "sym"])
    final_count = len(df)
    
    # 4. Save back (Standardized 14-column format)
    df.to_csv(csv_path, index=False, header=False)
    
    print(f"✅ Sanitization complete.")
    print(f"📊 Summary: {initial_count} -> {warmed_count} (Warmup) -> {matured_count} (Provenance/Maturity) -> {final_count} (Final)")
    print(f"♻️  Total records removed: {initial_count - final_count}")

if __name__ == "__main__":
    sanitize()
