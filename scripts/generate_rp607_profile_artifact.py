#!/usr/bin/env python3
"""
scripts/generate_rp607_profile_artifact.py
Generates the authoritative RP-607-RANGE-001 stock profile artifact
for all 119 NSE symbols based on approved rolling historical dates.

Governance:
  - Strictly read-only computation over existing intraday_capture/yahoo_cache_1d.
  - Zero modification to strategy logic or thresholds.
  - 0.0% Order Authority.
"""

import json
from pathlib import Path
from collections import Counter
from scripts.run_rp608_nse_shadow_daemon import NSE_119_SYMBOLS
from scripts.run_rp607_range_compartmentalisation import compute_rolling_profiles

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
OUTPUT_PATH = WORKSPACE / "live_capture" / "shadow_validation" / "nse" / "rp607_stock_profiles.json"

def main():
    session_dates = ["2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01"]
    profiles_by_date = compute_rolling_profiles(NSE_119_SYMBOLS, session_dates)
    
    # As of the latest approved historical session date: 2026-10-01
    latest_date = "2026-10-01"
    raw_profiles = profiles_by_date[latest_date]
    
    clean_profiles = {}
    comp_counts = Counter()
    valid_count = 0
    quarantined_count = 0
    
    for symbol in sorted(NSE_119_SYMBOLS):
        p = raw_profiles.get(symbol, {})
        is_valid = p.get("valid", False)
        comp = p.get("compartment", "C_Quarantined (Insufficient Hist)")
        
        # Compartment short code
        if "C1" in comp:
            code = "C1"
        elif "C2" in comp:
            code = "C2"
        elif "C3" in comp:
            code = "C3"
        elif "C4" in comp:
            code = "C4"
        else:
            code = "QUARANTINED"
            
        comp_counts[comp] += 1
        if is_valid:
            valid_count += 1
        else:
            quarantined_count += 1
            
        clean_profiles[symbol] = {
            "symbol": symbol,
            "tradingsymbol": symbol.replace(".NS", ""),
            "compartment": comp,
            "compartment_code": code,
            "valid": is_valid,
            "quarantine_reason": None if is_valid else "Insufficient Hist (< 20 daily bars in cache)",
            "natr_20": round(float(p["natr_20"]), 1) if is_valid and p.get("natr_20") is not None else None,
            "dir_eff_20": round(float(p["dir_eff_20"]), 3) if is_valid and p.get("dir_eff_20") is not None else None,
            "ar1_60": round(float(p["ar1_60"]), 3) if is_valid and p.get("ar1_60") is not None else None,
            "cv_range": round(float(p["cv_range"]), 3) if is_valid and p.get("cv_range") is not None else None,
            "opp_ratio": round(float(p["opp_ratio"]), 1) if is_valid and p.get("opp_ratio") is not None else None,
            "h_20d": round(float(p["h_20d"]), 2) if is_valid and p.get("h_20d") is not None else None,
            "l_20d": round(float(p["l_20d"]), 2) if is_valid and p.get("l_20d") is not None else None,
            "profile_date": latest_date,
            "provenance": {
                "source": "yahoo_cache_1d",
                "protocol": "RP-607-RANGE-001",
                "calculated_as_of": "2026-09-30 23:59:59 UTC",
                "nature": "HISTORICAL_SNAPSHOT"
            }
        }
        
    artifact = {
        "schema_version": "1.0.0",
        "protocol_id": "RP-607-RANGE-001",
        "title": "RP-607 Historical Range-Based Stock Compartmentalisation Census",
        "profile_date": latest_date,
        "calculation_cutoff_utc": "2026-09-30T23:59:59Z",
        "nature": "HISTORICAL_DESCRIPTIVE_PROFILE",
        "census_total": len(NSE_119_SYMBOLS),
        "classified_count": valid_count,
        "quarantined_count": quarantined_count,
        "compartment_distribution": {
            "C1: High-Range Directional": comp_counts["C1: High-Range Directional"],
            "C2: High-Range Oscillator": comp_counts["C2: High-Range Oscillator"],
            "C3: Stable Moderate": comp_counts["C3: Stable Moderate"],
            "C4: Low-Range Friction-Bound": comp_counts["C4: Low-Range Friction-Bound"],
            "C_Quarantined (Insufficient Hist)": comp_counts["C_Quarantined (Insufficient Hist)"]
        },
        "profiles": clean_profiles
    }
    
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_PATH, "w") as f:
        json.dump(artifact, f, indent=2)
        
    print(f"Successfully generated {OUTPUT_PATH}")
    print(f"Census Total: {len(NSE_119_SYMBOLS)}")
    print(f"Classified: {valid_count} | Quarantined: {quarantined_count}")
    print(f"Distribution: {dict(comp_counts)}")

if __name__ == "__main__":
    main()
