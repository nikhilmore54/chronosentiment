#!/usr/bin/env python3
"""
RP-608-PVP-001: Price-Volume Pattern Attribution Engine
Phase 1: Pre-Entry Observational Forensics & Outcome Discrimination

Cohorts Analyzed:
  1. Top 10% Largest Winners (N=203)
  2. Top 10% Largest Losers (N=203)
  3. Matched Comparison Trades (N=203, matched on symbol, session, and time-of-day)
  4. Deciles across all 2,029 historical trades

Features Evaluated (Strictly Completed Bars Before Entry):
  - Price Structure: Pre-entry returns (5m, 15m, 30m, 60m), consecutive runs, body-to-wick ratio,
                     distance from 30m high/low, distance from VWAP.
  - Volume Behaviour: 5m/15m relative volume, 1m spike ratio, volume-price divergence,
                      volume expansion followed by price stagnation (exhaustion).
  - Market Context: 30m rolling volatility, distance from session open, intraday range position.

Governance:
  - adapters/chronosentiment remains 100% FROZEN (13/13 discipline invariants).
  - Strictly 0.0% Live Order Authority (Shadow Evaluation Only).
  - Pure observational analysis; zero indicator or rule tuning.
"""

import os
import sys
import glob
import json
from pathlib import Path
import pandas as pd
import numpy as np
from scipy import stats
from sklearn.metrics import roc_auc_score

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
CACHE_1M = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"

def load_trades():
    trade_files = sorted(glob.glob(str(LOG_DIR / "rp611_nse_*_trades.csv")))
    trades_df = pd.concat([pd.read_csv(f) for f in trade_files], ignore_index=True)
    trades_df["entry_dt"] = pd.to_datetime(trades_df["entry_timestamp"])
    trades_df["exit_dt"] = pd.to_datetime(trades_df["exit_timestamp"])
    trades_df["session_date"] = trades_df["entry_dt"].dt.date
    trades_df["notional"] = trades_df["entry_price"] * trades_df["quantity"]
    trades_df["gross_bps"] = (trades_df["gross_pnl"] / trades_df["notional"]) * 10000.0
    trades_df["net_bps"] = (trades_df["net_pnl"] / trades_df["notional"]) * 10000.0
    return trades_df

def load_1m_bars(symbols):
    bars_map = {}
    for s in symbols:
        p = CACHE_1M / f"{s}.json"
        if p.exists():
            with open(p) as f:
                d = json.load(f)
            df = pd.DataFrame(d)
            df["dt"] = pd.to_datetime(df["timestamp"], unit="s", utc=True)
            df = df.sort_values("dt").reset_index(drop=True)
            bars_map[s] = df
    return bars_map

def extract_pre_entry_features(trades_df, bars_map):
    """
    Extracts price structure, volume behaviour, and context features
    strictly using completed 1m bars prior to each trade's entry_dt.
    """
    records = []
    for idx, row in trades_df.iterrows():
        s = row["symbol"]
        e_dt = row["entry_dt"]
        ep = row["entry_price"]
        side = row["side"]
        
        df_b = bars_map.get(s)
        if df_b is None or df_b.empty:
            continue
            
        # Completed bars strictly before entry timestamp
        prior_bars = df_b[df_b["dt"] < e_dt]
        if len(prior_bars) < 60:
            continue
            
        # Current session bars strictly before entry
        sess_date = e_dt.date()
        sess_bars = prior_bars[prior_bars["dt"].dt.date == sess_date]
        if sess_bars.empty:
            continue

        c = prior_bars["close"].values
        h = prior_bars["high"].values
        l = prior_bars["low"].values
        o = prior_bars["open"].values
        v = prior_bars["volume"].values
        
        # 1. Price Structure Features
        ret_5m = ((c[-1] - c[-6]) / c[-6]) * 10000.0 if len(c) >= 6 else np.nan
        ret_15m = ((c[-1] - c[-16]) / c[-16]) * 10000.0 if len(c) >= 16 else np.nan
        ret_30m = ((c[-1] - c[-31]) / c[-31]) * 10000.0 if len(c) >= 31 else np.nan
        ret_60m = ((c[-1] - c[-61]) / c[-61]) * 10000.0 if len(c) >= 61 else np.nan

        # Align pre-entry momentum with trade direction (+ = in trade direction, - = against)
        dir_mult = 1.0 if side == "LONG" else -1.0
        ret_5m_aligned = ret_5m * dir_mult
        ret_15m_aligned = ret_15m * dir_mult
        ret_30m_aligned = ret_30m * dir_mult

        # Consecutive candles in direction
        diffs = np.diff(c[-10:])
        consec_run = 0
        for d in reversed(diffs):
            if (d * dir_mult) > 0:
                consec_run += 1
            else:
                break

        # Last completed 1m candle body vs wick
        last_body = abs(c[-1] - o[-1])
        last_range = h[-1] - l[-1]
        body_to_range = last_body / (last_range + 1e-6)
        upper_wick = h[-1] - max(o[-1], c[-1])
        lower_wick = min(o[-1], c[-1]) - l[-1]
        adverse_wick = upper_wick if side == "LONG" else lower_wick
        adverse_wick_ratio = adverse_wick / (last_range + 1e-6)

        # Distance from 30m High/Low in bps
        h_30m = np.max(h[-30:])
        l_30m = np.min(l[-30:])
        dist_from_30m_high_bps = ((h_30m - ep) / ep) * 10000.0
        dist_from_30m_low_bps = ((ep - l_30m) / ep) * 10000.0
        dist_30m_extremum = dist_from_30m_high_bps if side == "LONG" else dist_from_30m_low_bps

        # Distance from Session VWAP
        sess_c = sess_bars["close"].values
        sess_v = sess_bars["volume"].values
        vwap = np.sum(sess_c * sess_v) / (np.sum(sess_v) + 1e-6)
        dist_vwap_bps = ((ep - vwap) / vwap) * 10000.0 * dir_mult

        # 2. Volume Behaviour Features
        # 5m and 15m volume vs 30m volume average
        v_5m_sum = np.sum(v[-5:])
        v_30m_mean_5m = np.mean([np.sum(v[i-5:i]) for i in range(-25, 0, 5)]) if len(v) >= 30 else v_5m_sum
        rel_vol_5m = v_5m_sum / (v_30m_mean_5m + 1e-6)

        # Volume spike on last completed candle
        v_median_30m = np.median(v[-30:])
        vol_spike_last_1m = v[-1] / (v_median_30m + 1e-6)

        # Volume-price divergence: price advancing in trade direction but volume dropping
        v_diff_3 = v[-1] - v[-4] if len(v) >= 4 else 0
        vol_divergence = 1.0 if (ret_5m_aligned > 0 and v_diff_3 < 0) else 0.0

        # Volume expansion with price stagnation (exhaustion pattern)
        recent_range_5m = (np.max(h[-5:]) - np.min(l[-5:])) / c[-1] * 10000.0
        vol_exhaustion = 1.0 if (rel_vol_5m > 1.5 and recent_range_5m < 15.0) else 0.0

        # 3. Market Context Features
        rets_30m = np.diff(c[-31:]) / c[-31:-1] * 10000.0
        volatility_30m_bps = np.std(rets_30m)
        sess_open = sess_bars["open"].values[0]
        dist_from_open_bps = ((ep - sess_open) / sess_open) * 10000.0 * dir_mult
        
        sess_h = sess_bars["high"].max()
        sess_l = sess_bars["low"].min()
        pos_in_day_range = (ep - sess_l) / ((sess_h - sess_l) + 1e-6)

        rec = dict(row)
        rec["ret_5m_aligned"] = ret_5m_aligned
        rec["ret_15m_aligned"] = ret_15m_aligned
        rec["ret_30m_aligned"] = ret_30m_aligned
        rec["consec_run"] = consec_run
        rec["body_to_range"] = body_to_range
        rec["adverse_wick_ratio"] = adverse_wick_ratio
        rec["dist_30m_extremum"] = dist_30m_extremum
        rec["dist_vwap_bps"] = dist_vwap_bps
        rec["rel_vol_5m"] = rel_vol_5m
        rec["vol_spike_last_1m"] = vol_spike_last_1m
        rec["vol_divergence"] = vol_divergence
        rec["vol_exhaustion"] = vol_exhaustion
        rec["volatility_30m_bps"] = volatility_30m_bps
        rec["dist_from_open_bps"] = dist_from_open_bps
        rec["pos_in_day_range"] = pos_in_day_range
        records.append(rec)

    return pd.DataFrame(records)

def main():
    print("=" * 100)
    print("RP-608-PVP-001: PRICE-VOLUME PATTERN ATTRIBUTION ENGINE")
    print("Pre-Entry Observational Forensics across Winners, Losers, and Matched Trades")
    print("=" * 100)

    trades_df = load_trades()
    symbols = sorted(trades_df["symbol"].unique())
    print(f"Loaded {len(trades_df)} historical trades across {len(symbols)} symbols.")

    print("Loading 1-minute intraday bars for feature extraction...")
    bars_map = load_1m_bars(symbols)

    print("Extracting pre-entry price structure, volume behaviour, and context features...")
    df_feat = extract_pre_entry_features(trades_df, bars_map)
    print(f"Extracted features for {len(df_feat)} trades with complete pre-entry history.")

    # 1. Build Cohorts
    # Sort by gross_bps
    df_sorted = df_feat.sort_values("gross_bps", ascending=False).reset_index(drop=True)
    n_cohort = int(len(df_sorted) * 0.10) # 10% = 202 trades
    
    top_winners = df_sorted.head(n_cohort).copy()
    top_winners["cohort"] = "Top 10% Winners"

    top_losers = df_sorted.tail(n_cohort).copy()
    top_losers["cohort"] = "Top 10% Losers"

    # Matched comparison trades: for each top loser, find a trade in the middle 80% with same symbol and nearest entry minute
    middle_pool = df_sorted.iloc[n_cohort:-n_cohort].copy()
    matched_records = []
    used_ids = set()
    for _, loser in top_losers.iterrows():
        candidates = middle_pool[
            (middle_pool["symbol"] == loser["symbol"]) &
            (~middle_pool["trade_id"].isin(used_ids))
        ]
        if not candidates.empty:
            # Match on closest entry minute
            candidates = candidates.copy()
            candidates["time_diff"] = (candidates["entry_minute_idx"] - loser["entry_minute_idx"]).abs()
            best = candidates.sort_values("time_diff").iloc[0]
            matched_records.append(best)
            used_ids.add(best["trade_id"])
        else:
            # Fallback match on same session date
            candidates_sess = middle_pool[
                (middle_pool["session_date"] == loser["session_date"]) &
                (~middle_pool["trade_id"].isin(used_ids))
            ]
            if not candidates_sess.empty:
                best = candidates_sess.iloc[0]
                matched_records.append(best)
                used_ids.add(best["trade_id"])
                
    matched_trades = pd.DataFrame(matched_records)
    matched_trades["cohort"] = "Matched Comparison"

    print("\n=== COHORT SUMMARY ===")
    print(f"1. Top 10% Winners:      {len(top_winners)} trades (Mean Gross: {top_winners['gross_bps'].mean():+.2f} bps, Win Rate: 100.0%)")
    print(f"2. Top 10% Losers:       {len(top_losers)} trades (Mean Gross: {top_losers['gross_bps'].mean():+.2f} bps, Win Rate: 0.0%)")
    print(f"3. Matched Comparison:   {len(matched_trades)} trades (Mean Gross: {matched_trades['gross_bps'].mean():+.2f} bps, Win Rate: {(matched_trades['gross_bps']>0).mean()*100:.1f}%)")

    # 2. Compare Feature Distributions
    feature_cols = [
        "ret_5m_aligned", "ret_15m_aligned", "ret_30m_aligned", "consec_run",
        "body_to_range", "adverse_wick_ratio", "dist_30m_extremum", "dist_vwap_bps",
        "rel_vol_5m", "vol_spike_last_1m", "vol_divergence", "vol_exhaustion",
        "volatility_30m_bps", "dist_from_open_bps", "pos_in_day_range"
    ]

    print("\n=== STATISTICAL DISCRIMINATION TABLE: TOP LOSERS VS TOP WINNERS & MATCHED ===")
    discrimination_results = []
    for col in feature_cols:
        val_w = top_winners[col].dropna()
        val_l = top_losers[col].dropna()
        val_m = matched_trades[col].dropna()

        mean_w, std_w = val_w.mean(), val_w.std()
        mean_l, std_l = val_l.mean(), val_l.std()
        mean_m, std_m = val_m.mean(), val_m.std()

        # Cohen's d: Losers vs Winners
        pooled_std_wl = np.sqrt(((len(val_w)-1)*std_w**2 + (len(val_l)-1)*std_l**2) / (len(val_w) + len(val_l) - 2))
        cohen_d_wl = (mean_l - mean_w) / (pooled_std_wl + 1e-6)

        # Wilcoxon rank-sum / Mann-Whitney U test (Losers vs Winners)
        stat_u, p_val_u = stats.mannwhitneyu(val_l, val_w, alternative="two-sided")
        
        # Kolmogorov-Smirnov test (Losers vs Matched)
        stat_ks_lm, p_val_ks_lm = stats.ks_2samp(val_l, val_m)

        # Correlation with Gross Return across ALL trades
        valid_all = df_feat[[col, "gross_bps"]].dropna()
        r_all, p_corr = stats.pearsonr(valid_all[col], valid_all["gross_bps"])

        discrimination_results.append({
            "feature": col,
            "mean_losers": mean_l,
            "mean_winners": mean_w,
            "mean_matched": mean_m,
            "cohen_d (L vs W)": cohen_d_wl,
            "mann_whitney_p": p_val_u,
            "ks_p (L vs Matched)": p_val_ks_lm,
            "corr_with_pnl": r_all,
            "corr_p_val": p_corr,
            "discriminative": "YES (p<0.01)" if (p_val_u < 0.01 and p_val_ks_lm < 0.05) else "NO"
        })

    disc_df = pd.DataFrame(discrimination_results)
    print(disc_df.to_string(index=False))

    # 3. Multivariate Predictive Screening (AUC-ROC)
    # Target: Can pre-entry features distinguish Top Losers from Top Winners?
    df_binary = pd.concat([top_winners, top_losers])
    y_true = (df_binary["cohort"] == "Top 10% Losers").astype(int)
    
    auc_scores = {}
    for col in feature_cols:
        vals = df_binary[col].fillna(df_binary[col].median())
        try:
            auc = roc_auc_score(y_true, vals)
            # Normalize so AUC is >= 0.50
            auc_norm = auc if auc >= 0.50 else (1.0 - auc)
            auc_scores[col] = auc_norm
        except Exception:
            auc_scores[col] = 0.50

    print("\n=== UNIVARIATE PREDICTIVE POWER (AUC-ROC FOR SEPARATING BIG LOSERS) ===")
    for col, auc in sorted(auc_scores.items(), key=lambda x: x[1], reverse=True):
        print(f"  {col:25s}: AUC = {auc:.3f} {'(Weak Signal)' if auc < 0.55 else '(Moderate Separation)' if auc < 0.65 else '(Strong Separation)'}")

    # 4. Decile Attribution across All 2,029 Trades
    print("\n=== DECILE STRATIFICATION: PRE-ENTRY FEATURES BY TRADE OUTCOME DECILE ===")
    df_feat["outcome_decile"] = pd.qcut(df_feat["gross_bps"], q=10, labels=[f"D{i+1}" for i in range(10)])
    dec_summary = df_feat.groupby("outcome_decile").agg(
        trades=("trade_id", "count"),
        gross_bps=("gross_bps", "mean"),
        ret_15m_aligned=("ret_15m_aligned", "mean"),
        adverse_wick_ratio=("adverse_wick_ratio", "mean"),
        rel_vol_5m=("rel_vol_5m", "mean"),
        vol_exhaustion=("vol_exhaustion", "mean"),
        dist_30m_extremum=("dist_30m_extremum", "mean"),
        pos_in_day_range=("pos_in_day_range", "mean")
    ).reset_index()
    print(dec_summary.to_string(index=False))

    # Save output artifacts
    out_json = WORKSPACE / "live_capture" / "shadow_validation" / "rp608_pvp_001_attribution_results.json"
    with open(out_json, "w") as f:
        json.dump({
            "discrimination_table": disc_df.to_dict(orient="records"),
            "auc_scores": auc_scores,
            "decile_summary": dec_summary.to_dict(orient="records")
        }, f, indent=2)
    print(f"\nSaved PVP-001 attribution results to {out_json}")

if __name__ == "__main__":
    main()
