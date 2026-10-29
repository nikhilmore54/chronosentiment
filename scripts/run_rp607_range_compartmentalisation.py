#!/usr/bin/env python3
"""
RP-607-RANGE-001: Historical Range-Based Stock Compartmentalisation Runner
Integrity & Reproducibility Audit Engine

Implements Option C Decoupled Architecture:
  - Layer 1: Macro Stock Movement Profile (5 Dimensions: Range Magnitude, Directional Efficiency,
             Reversion Tendency, Range Stability, Cost-Adjusted Opportunity).
             Recomputed strictly rolling day-by-day prior to each trading session.
  - Layer 2: Situational Range Position (Position within trailing 20-day High-Low Envelope).
             Reconciles Canonical Deciles (<=0.15, 0.15-0.35, 0.35-0.65, 0.65-0.85, >=0.85)
             and Continuous 3-Zone Partition (<=0.20, 0.20-0.80, >=0.80).

Governance:
  - adapters/chronosentiment remains 100% FROZEN (13/13 discipline invariants).
  - Strictly 0.0% Live Order Authority (Shadow Evaluation Only).
  - No threshold tuning against closed RP-605 holdout; descriptive attribution only.
"""

import os
import sys
import glob
import json
from pathlib import Path
import pandas as pd
import numpy as np

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
CACHE_1D = WORKSPACE / "intraday_capture" / "yahoo_cache_1d"
CACHE_1M = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"

def load_data():
    trade_files = sorted(glob.glob(str(LOG_DIR / "rp611_nse_*_trades.csv")))
    trades_df = pd.concat([pd.read_csv(f) for f in trade_files], ignore_index=True)
    trades_df["entry_dt"] = pd.to_datetime(trades_df["entry_timestamp"])
    trades_df["exit_dt"] = pd.to_datetime(trades_df["exit_timestamp"])
    trades_df["session_date"] = trades_df["entry_dt"].dt.date
    trades_df["notional"] = trades_df["entry_price"] * trades_df["quantity"]
    trades_df["gross_bps"] = (trades_df["gross_pnl"] / trades_df["notional"]) * 10000.0
    trades_df["net_bps"] = (trades_df["net_pnl"] / trades_df["notional"]) * 10000.0
    return trades_df

def compute_rolling_profiles(symbols, session_dates):
    """
    Computes rolling Layer 1 profiles strictly prior to each trading session date.
    Session Cutoffs:
      - 2026-09-28: up to 2026-09-25 23:59:59 UTC
      - 2026-09-29: up to 2026-09-28 23:59:59 UTC
      - 2026-09-30: up to 2026-09-29 23:59:59 UTC
      - 2026-10-01: up to 2026-09-30 23:59:59 UTC
    """
    session_cutoffs = {
        session_dates[0]: "2026-09-25 23:59:59+00:00",
        session_dates[1]: "2026-09-28 23:59:59+00:00",
        session_dates[2]: "2026-09-29 23:59:59+00:00",
        session_dates[3]: "2026-09-30 23:59:59+00:00",
    }

    rolling_profiles = {}
    for sess, cutoff in session_cutoffs.items():
        prof_map = {}
        for s in symbols:
            p_1d = CACHE_1D / f"{s}.json"
            df_hist = None
            if p_1d.exists():
                with open(p_1d) as f:
                    d = json.load(f)
                    df_d = pd.DataFrame(d)
                    df_d["dt"] = pd.to_datetime(df_d["timestamp"], unit="s", utc=True)
                    df_d = df_d.sort_values("dt").reset_index(drop=True)
                    df_hist = df_d[df_d["dt"] <= cutoff]

            # Require minimum 20 daily bars for valid profile
            if df_hist is None or len(df_hist) < 20:
                prof_map[s] = {
                    "compartment": "C_Quarantined (Insufficient Hist)",
                    "valid": False,
                    "h_20d": np.nan,
                    "l_20d": np.nan,
                    "natr_20": np.nan,
                    "dir_eff_20": np.nan,
                    "ar1_60": np.nan,
                    "opp_ratio": np.nan
                }
                continue

            c = df_hist["close"].values
            h = df_hist["high"].values
            l = df_hist["low"].values

            # Exact True Range calculation
            tr = np.maximum(h[1:] - l[1:], np.maximum(np.abs(h[1:] - c[:-1]), np.abs(l[1:] - c[:-1])))
            tr_20 = tr[-20:]

            # 1. Range Magnitude (NATR_20 in bps)
            natr_20 = (np.mean(tr_20) / c[-1]) * 10000.0

            # 2. Directional Efficiency: 20 intervals net displacement / cumulative TR
            disp_20 = abs(c[-1] - c[-21])
            path_20 = np.sum(tr_20)
            dir_eff_20 = disp_20 / (path_20 + 1e-6)

            # 3. Autoregressive Reversion Tendency (AR1 over up to 60 days)
            rets = np.diff(c) / c[:-1]
            rets_60 = rets[-60:] if len(rets) >= 60 else rets
            ar1 = np.corrcoef(rets_60[:-1], rets_60[1:])[0, 1] if len(rets_60) > 5 and np.std(rets_60) > 1e-6 else -0.10
            if np.isnan(ar1):
                ar1 = -0.10

            # 4. Range Stability (CV of True Range)
            cv_range = np.std(tr_20) / (np.mean(tr_20) + 1e-6)

            # 5. Opportunity Ratio
            opp_ratio = natr_20 / 9.00

            # 20-day high and low envelope
            h_20d = np.max(h[-20:])
            l_20d = np.min(l[-20:])

            # Formalized Precedence Hierarchy:
            # 1. Low-Range Friction-Bound
            if natr_20 < 160.0 or opp_ratio < 18.0:
                comp = "C4: Low-Range Friction-Bound"
            # 2. High-Range Directional
            elif natr_20 >= 220.0 and dir_eff_20 >= 0.14:
                comp = "C1: High-Range Directional"
            # 3. High-Range Oscillator
            elif natr_20 >= 220.0 and ar1 <= -0.12:
                comp = "C2: High-Range Oscillator"
            # 4. Stable Moderate Movers
            else:
                comp = "C3: Stable Moderate"

            prof_map[s] = {
                "compartment": comp,
                "valid": True,
                "natr_20": natr_20,
                "dir_eff_20": dir_eff_20,
                "ar1_60": ar1,
                "cv_range": cv_range,
                "opp_ratio": opp_ratio,
                "h_20d": h_20d,
                "l_20d": l_20d
            }
        rolling_profiles[sess] = prof_map

    return rolling_profiles

def main():
    print("=" * 100)
    print("RP-607-RANGE-001: INTEGRITY & REPRODUCIBILITY AUDIT ENGINE")
    print("Reconciled Dual-Layer Stock Compartmentalisation (Rolling Day-by-Day Profiles)")
    print("=" * 100)

    trades_df = load_data()
    symbols = sorted(trades_df["symbol"].unique())
    session_dates = sorted(trades_df["session_date"].unique())
    print(f"Loaded {len(trades_df)} completed trades across {len(symbols)} symbols in {len(session_dates)} sessions.")

    rolling_profiles = compute_rolling_profiles(symbols, session_dates)

    # Audit Stability across sessions
    transitions = 0
    for s in symbols:
        comps = [rolling_profiles[sess][s]["compartment"] for sess in session_dates]
        if len(set(comps)) > 1:
            transitions += 1
    print(f"Rolling Profile Stability: {transitions} / {len(symbols)} symbols changed compartment across sessions.")

    records = []
    for _, row in trades_df.iterrows():
        sess = row["session_date"]
        s = row["symbol"]
        prof = rolling_profiles[sess][s]
        px = row["entry_price"]

        if not prof["valid"]:
            comp = "C_Quarantined (Insufficient Hist)"
            zone_canonical = "Unclassified"
            zone_cont = "Unclassified"
            pos = np.nan
        else:
            comp = prof["compartment"]
            h_20d = prof["h_20d"]
            l_20d = prof["l_20d"]
            pos = (px - l_20d) / ((h_20d - l_20d) + 1e-6)
            pos = np.clip(pos, 0.0, 1.0)

            # Canonical Specification with Transition Bands
            if pos <= 0.15:
                zone_canonical = "Z_Lower (<=0.15)"
            elif pos < 0.35:
                zone_canonical = "Z_Trans_L (0.15-0.35)"
            elif pos <= 0.65:
                zone_canonical = "Z_Mid (0.35-0.65)"
            elif pos < 0.85:
                zone_canonical = "Z_Trans_U (0.65-0.85)"
            else:
                zone_canonical = "Z_Upper (>=0.85)"

            # Continuous 3-Zone Partition
            if pos <= 0.20:
                zone_cont = "Z_Lower (<=0.20)"
            elif pos >= 0.80:
                zone_cont = "Z_Upper (>=0.80)"
            else:
                zone_cont = "Z_Mid (0.20-0.80)"

        rec = dict(row)
        rec["compartment"] = comp
        rec["zone_canonical"] = zone_canonical
        rec["zone_cont"] = zone_cont
        rec["pos_20d"] = pos
        records.append(rec)

    df_all = pd.DataFrame(records)

    print("\n=== TABLE 1: RECONCILED COMPARTMENT SUMMARY ===")
    cs = df_all.groupby("compartment").agg(
        symbols=("symbol", "nunique"),
        trades=("trade_id", "count"),
        gross_pnl=("gross_pnl", "sum"),
        friction=("friction_inr", "sum"),
        net_pnl=("net_pnl", "sum"),
        gross_bps=("gross_bps", "mean"),
        net_bps=("net_bps", "mean"),
        gross_wr=("gross_pnl", lambda x: (x > 0).mean() * 100),
        net_wr=("net_pnl", lambda x: (x > 0).mean() * 100)
    ).reset_index()
    print(cs.to_string(index=False))

    print("\n=== TABLE 2: RECONCILED 4x3 CANONICAL MATRIX ===")
    m_can = df_all[df_all["compartment"] != "C_Quarantined (Insufficient Hist)"].groupby(["compartment", "zone_canonical"]).agg(
        symbols=("symbol", "nunique"),
        trades=("trade_id", "count"),
        gross_pnl=("gross_pnl", "sum"),
        net_pnl=("net_pnl", "sum"),
        gross_bps=("gross_bps", "mean"),
        net_bps=("net_bps", "mean"),
        net_wr=("net_pnl", lambda x: (x > 0).mean() * 100)
    ).reset_index()
    print(m_can.to_string(index=False))

    print("\n=== TABLE 3: CONTINUOUS 3-ZONE PARTITION MATRIX ===")
    m_cont = df_all[df_all["compartment"] != "C_Quarantined (Insufficient Hist)"].groupby(["compartment", "zone_cont"]).agg(
        symbols=("symbol", "nunique"),
        trades=("trade_id", "count"),
        gross_pnl=("gross_pnl", "sum"),
        net_pnl=("net_pnl", "sum"),
        gross_bps=("gross_bps", "mean"),
        net_bps=("net_bps", "mean"),
        net_wr=("net_pnl", lambda x: (x > 0).mean() * 100)
    ).reset_index()
    print(m_cont.to_string(index=False))

    print("\n=== SESSION CONCENTRATION (CANONICAL MATRIX) ===")
    sess_ct = pd.crosstab(
        [df_all["compartment"], df_all["zone_canonical"]], 
        df_all["session_date"]
    )
    print(sess_ct.to_string())

if __name__ == "__main__":
    main()
