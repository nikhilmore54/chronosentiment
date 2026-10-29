#!/usr/bin/env python3
"""
scripts/research/study_historical_tick_levels.py

Empirical Multi-Level Tick Study across:
- Level 1: Sub-Second Micro-Ticks (Trade-by-trade order flow & queue absorption)
- Level 2: Intraday Bar-Ticks (1m to 300m excursion trajectories & volume acceleration)
- Level 3: Session-Horizon Ticks (1 to 20 sessions multi-day outcomes)

Purpose:
Learn ground-truth empirical dynamics from historical data at various tick levels
to inform execution modeling, fill feasibility, and volume-trend conditioning.
"""

from __future__ import annotations

import glob
import json
import math
from pathlib import Path
from typing import Any, Dict, List, Tuple

import numpy as np
import pandas as pd

WORKSPACE = Path(__file__).resolve().parent.parent.parent
ARCHIVE_BTC_TICKS = WORKSPACE / "archive" / "historical_research" / "chronology" / "live_capture" / "btcusdt_1780044475.jsonl"
ARCHIVE_SPY_TICKS = WORKSPACE / "archive" / "historical_research" / "chronology" / "historical" / "spy_capture" / "spy_1779283800000.jsonl"
ARCHIVE_DAX_TICKS = WORKSPACE / "archive" / "historical_research" / "chronology" / "historical" / "dax_capture" / "gdaxi_1779346800000.jsonl"

CRYPTO_CENSUS_B2 = WORKSPACE / "datasets" / "crypto_behaviour_census_with_b2.csv"
P4_INTRADAY_FLAT = WORKSPACE / "datasets" / "p4_intraday_flat.csv"
RECOMMENDATIONS_DIR = WORKSPACE / "datasets" / "recommendation" / "historical"


# =====================================================================
# Level 1: Sub-Second Micro-Ticks Analysis
# =====================================================================
def analyze_level1_micro_ticks(tick_path: Path) -> Dict[str, Any]:
    print(f"\n[Level 1] Analyzing Micro-Ticks from: {tick_path.name}")
    if not tick_path.exists():
        return {"status": "FILE_NOT_FOUND"}

    records = []
    with tick_path.open("r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                try:
                    records.append(json.loads(line))
                except json.JSONDecodeError:
                    continue

    df = pd.DataFrame(records)
    n_ticks = len(df)
    if n_ticks == 0:
        return {"status": "NO_RECORDS"}

    df["timestamp"] = pd.to_numeric(df["timestamp"])
    df["price"] = pd.to_numeric(df["price"])
    df["volume"] = pd.to_numeric(df["volume"])
    df["is_buyer_maker"] = df["is_buyer_maker"].astype(bool)

    # Sort strictly by timestamp
    df = df.sort_values("timestamp").reset_index(drop=True)

    # Inter-arrival time (ms)
    df["dt_ms"] = df["timestamp"].diff().fillna(0)
    burst_pct_sub10ms = (df["dt_ms"] <= 10).mean() * 100.0
    burst_pct_sub50ms = (df["dt_ms"] <= 50).mean() * 100.0

    # Aggression split: is_buyer_maker True = buyer was maker => aggressive taker SELL hit the bid
    # False = seller was maker => aggressive taker BUY lifted the ask
    taker_sells = df["is_buyer_maker"].sum()
    taker_buys = n_ticks - taker_sells
    sell_aggression_pct = (taker_sells / n_ticks) * 100.0

    # Volume distribution per trade tick
    vol_p10, vol_p25, vol_p50, vol_p75, vol_p90, vol_p99 = df["volume"].quantile([0.1, 0.25, 0.5, 0.75, 0.9, 0.99])
    trade_notional = df["price"] * df["volume"]
    notional_p50 = trade_notional.median()
    notional_p90 = trade_notional.quantile(0.90)

    # Consecutive trade runs at the exact same price level (Order queue absorption)
    same_price_runs = []
    run_vol = 0.0
    run_len = 0
    cur_p = df.iloc[0]["price"]

    for i in range(len(df)):
        p = df.iloc[i]["price"]
        v = df.iloc[i]["volume"]
        if p == cur_p:
            run_len += 1
            run_vol += v
        else:
            same_price_runs.append((run_len, run_vol, cur_p))
            cur_p = p
            run_len = 1
            run_vol = v
    same_price_runs.append((run_len, run_vol, cur_p))

    run_lens = [r[0] for r in same_price_runs]
    run_vols = [r[1] for r in same_price_runs]

    # Price tick changes between trades
    df["dp"] = df["price"].diff().fillna(0.0)
    zero_ticks_pct = (df["dp"] == 0.0).mean() * 100.0
    min_tick_abs = 0.01
    single_tick_pct = (df["dp"].abs() == min_tick_abs).mean() * 100.0
    multi_tick_pct = (df["dp"].abs() > min_tick_abs).mean() * 100.0

    # Adverse drift: When an aggressive taker sell hits the bid, what is the forward price drift over 10, 50, 100 ticks?
    fwd_drift_10 = (df["price"].shift(-10) - df["price"]) / df["price"] * 10000.0  # in bps
    fwd_drift_50 = (df["price"].shift(-50) - df["price"]) / df["price"] * 10000.0
    
    # Conditional on aggressive taker sell (buyer maker = True)
    adverse_drift_sell_10 = fwd_drift_10[df["is_buyer_maker"]].median()
    adverse_drift_sell_50 = fwd_drift_50[df["is_buyer_maker"]].median()

    # Conditional on aggressive taker buy (buyer maker = False)
    fwd_drift_buy_10 = fwd_drift_10[~df["is_buyer_maker"]].median()
    fwd_drift_buy_50 = fwd_drift_50[~df["is_buyer_maker"]].median()

    results = {
        "n_ticks": n_ticks,
        "time_span_seconds": round((df['timestamp'].iloc[-1] - df['timestamp'].iloc[0]) / 1000.0, 1),
        "dt_ms_median": float(df["dt_ms"].median()),
        "burst_pct_sub10ms": round(burst_pct_sub10ms, 2),
        "burst_pct_sub50ms": round(burst_pct_sub50ms, 2),
        "taker_sells_pct": round(sell_aggression_pct, 2),
        "taker_buys_pct": round(100.0 - sell_aggression_pct, 2),
        "vol_coin_median": float(round(vol_p50, 6)),
        "vol_coin_p90": float(round(vol_p90, 6)),
        "notional_usd_median": float(round(notional_p50, 2)),
        "notional_usd_p90": float(round(notional_p90, 2)),
        "same_price_run_len_median": int(np.median(run_lens)),
        "same_price_run_len_p90": int(np.percentile(run_lens, 90)),
        "same_price_run_vol_median": float(round(np.median(run_vols), 4)),
        "same_price_run_vol_p90": float(round(np.percentile(run_vols, 90), 4)),
        "price_step_zero_pct": round(zero_ticks_pct, 2),
        "price_step_single_tick_pct": round(single_tick_pct, 2),
        "price_step_multi_tick_pct": round(multi_tick_pct, 2),
        "post_taker_sell_drift_10ticks_bps": float(round(adverse_drift_sell_10, 4)),
        "post_taker_sell_drift_50ticks_bps": float(round(adverse_drift_sell_50, 4)),
        "post_taker_buy_drift_10ticks_bps": float(round(fwd_drift_buy_10, 4)),
        "post_taker_buy_drift_50ticks_bps": float(round(fwd_drift_buy_50, 4)),
    }
    return results


# =====================================================================
# Level 2: Intraday Bar-Ticks & Volume Acceleration Trajectories
# =====================================================================
def analyze_level2_intraday_trajectories(census_path: Path, p4_path: Path) -> Dict[str, Any]:
    print(f"\n[Level 2] Analyzing Intraday Trajectories from: {census_path.name}")
    if not census_path.exists():
        return {"status": "FILE_NOT_FOUND"}

    df = pd.read_csv(census_path)
    n_obs = len(df)

    # 1. Volume Acceleration (VA_60m) Quartiles & Regimes
    # VA = vol_last_30m / vol_prev_30m
    q25_va = df["volume_acceleration_60m"].quantile(0.25)
    q50_va = df["volume_acceleration_60m"].quantile(0.50)
    q75_va = df["volume_acceleration_60m"].quantile(0.75)

    def assign_va_regime(va: float) -> str:
        if math.isnan(va) or va is None:
            return "UNPUBLISHED/MISSING"
        if va < 0.85:
            return "DECAYING (<0.85)"
        elif va <= 1.15:
            return "STABLE (0.85-1.15)"
        elif va <= 1.46:
            return "EXPANDING (1.15-1.46)"
        else:
            return "HIGH_SURGE (>1.46)"

    df["va_regime"] = df["volume_acceleration_60m"].apply(assign_va_regime)

    # Calculate metrics grouped by volume acceleration regime
    regime_stats = {}
    for regime, group in df.groupby("va_regime"):
        n_g = len(group)
        pb_rate = group["Persistent_Build"].mean() * 100.0 if "Persistent_Build" in group else 0.0
        decay_rate = group["Early_Excursion_Decay"].mean() * 100.0 if "Early_Excursion_Decay" in group else 0.0
        aer_rate = group["Adverse_Excursion_Recovery"].mean() * 100.0 if "Adverse_Excursion_Recovery" in group else 0.0

        mfe_60_mean = group["mfe_60"].mean() * 100.0
        mae_60_mean = group["mae_60"].mean() * 100.0
        ret_60_mean = group["ret_60"].mean() * 100.0
        ret_300_mean = group["ret_300"].mean() * 100.0
        
        # Asymmetry ratio: MFE / |MAE|
        asym_ratio = mfe_60_mean / max(abs(mae_60_mean), 1e-4)

        regime_stats[regime] = {
            "N": n_g,
            "pct_of_sample": round((n_g / n_obs) * 100.0, 1),
            "persistent_build_pct": round(pb_rate, 2),
            "early_excursion_decay_pct": round(decay_rate, 2),
            "adverse_recovery_pct": round(aer_rate, 2),
            "mfe_60m_pct": round(mfe_60_mean, 3),
            "mae_60m_pct": round(mae_60_mean, 3),
            "asymmetry_mfe_mae_ratio": round(asym_ratio, 2),
            "ret_60m_pct": round(ret_60_mean, 3),
            "ret_300m_pct": round(ret_300_mean, 3),
            "time_to_mfe_min": round(group["time_to_mfe_60"].median(), 1),
            "time_to_mae_min": round(group["time_to_mae_60"].median(), 1),
        }

    # 2. Multi-Horizon Resolution Study from p4_intraday_flat.csv
    horizon_stats = {}
    if p4_path.exists():
        df_p4 = pd.read_csv(p4_path)
        for h, grp in df_p4.groupby("horizon"):
            n_h = len(grp)
            target_hit_pct = grp["target_attained"].mean() * 100.0
            risk_hit_pct = grp["risk_attained"].mean() * 100.0
            avg_mfe = grp["mfe"].mean() * 100.0
            avg_mae = grp["mae"].mean() * 100.0

            # Target hit rate split by volume regime
            vol_split = {}
            for v_reg, v_grp in grp.groupby("volume_regime"):
                vol_split[v_reg] = {
                    "N": len(v_grp),
                    "target_hit_pct": round(v_grp["target_attained"].mean() * 100.0, 1),
                    "risk_hit_pct": round(v_grp["risk_attained"].mean() * 100.0, 1),
                    "mean_intraday_return_pct": round(v_grp["intraday_return"].mean() * 100.0, 3),
                }

            horizon_stats[h] = {
                "N": n_h,
                "n_bars": int(grp["n_bars"].iloc[0]),
                "target_hit_pct": round(target_hit_pct, 2),
                "risk_hit_pct": round(risk_hit_pct, 2),
                "avg_mfe_pct": round(avg_mfe, 3),
                "avg_mae_pct": round(avg_mae, 3),
                "volume_regime_split": vol_split,
            }

    return {
        "n_census_obs": n_obs,
        "va_quantiles": {"P25": round(q25_va, 4), "P50": round(q50_va, 4), "P75": round(q75_va, 4)},
        "va_regimes": regime_stats,
        "horizon_evaluations": horizon_stats,
    }


# =====================================================================
# Level 3: Session / Multi-Day Horizon Ticks Analysis
# =====================================================================
def analyze_level3_session_horizons(rec_dir: Path) -> Dict[str, Any]:
    print(f"\n[Level 3] Analyzing Session Ticks from: {rec_dir}")
    if not rec_dir.exists():
        return {"status": "DIR_NOT_FOUND"}

    jsonl_files = list(rec_dir.glob("*.jsonl"))
    if not jsonl_files:
        return {"status": "NO_FILES"}

    all_traces = []
    for f in jsonl_files:
        with f.open("r", encoding="utf-8") as fh:
            for line in fh:
                if line.strip():
                    try:
                        all_traces.append(json.loads(line))
                    except json.JSONDecodeError:
                        continue

    df = pd.DataFrame(all_traces)
    n_sessions = len(df)
    if n_sessions == 0:
        return {"status": "NO_SESSIONS"}

    # Relative volume 20-day trend
    df["relative_volume_20"] = pd.to_numeric(df.get("relative_volume_20", 1.0))
    df["outcome"] = df.get("outcome", "UNKNOWN")
    df["sessions_to_outcome"] = pd.to_numeric(df.get("sessions_to_outcome", 0))

    # Split into RVOL categories: Low (<0.8), Normal (0.8 - 1.2), High (>1.2)
    def rvol_cat(r: float) -> str:
        if math.isnan(r) or r is None or r <= 0:
            return "UNPUBLISHED/MISSING"
        if r < 0.8:
            return "LOW_RVOL (<0.8)"
        elif r <= 1.2:
            return "NORMAL_RVOL (0.8-1.2)"
        else:
            return "SURGE_RVOL (>1.2)"

    df["rvol_category"] = df["relative_volume_20"].apply(rvol_cat)

    outcome_by_rvol = {}
    for cat, grp in df.groupby("rvol_category"):
        n_c = len(grp)
        target_first = (grp["outcome"] == "TARGET_BEFORE_RISK").mean() * 100.0
        risk_first = (grp["outcome"] == "RISK_BEFORE_TARGET").mean() * 100.0
        horizon_reached = (grp["outcome"] == "HORIZON_REACHED").mean() * 100.0
        med_sessions = grp["sessions_to_outcome"].median()

        outcome_by_rvol[cat] = {
            "N": n_c,
            "pct_of_sample": round((n_c / n_sessions) * 100.0, 1),
            "target_before_risk_win_rate": round(target_first, 2),
            "risk_before_target_loss_rate": round(risk_first, 2),
            "horizon_reached_pct": round(horizon_reached, 2),
            "median_sessions_to_outcome": float(round(med_sessions, 1)),
        }

    return {
        "n_session_records": n_sessions,
        "n_symbols": len(jsonl_files),
        "outcome_by_rvol": outcome_by_rvol,
    }


def main():
    print("=" * 80)
    print("CHRONOSENTIMENT: EMPIRICAL MULTI-LEVEL TICK STUDY")
    print("Ground-Truth Investigation across Micro, Intraday, and Session Ticks")
    print("=" * 80)

    # 1. Level 1 Micro-ticks
    level1_res = analyze_level1_micro_ticks(ARCHIVE_BTC_TICKS)

    # 2. Level 2 Intraday Trajectories
    level2_res = analyze_level2_intraday_trajectories(CRYPTO_CENSUS_B2, P4_INTRADAY_FLAT)

    # 3. Level 3 Session Horizons
    level3_res = analyze_level3_session_horizons(RECOMMENDATIONS_DIR)

    full_results = {
        "timestamp_utc": pd.Timestamp.now(tz="UTC").isoformat(),
        "level_1_micro_ticks": level1_res,
        "level_2_intraday_trajectories": level2_res,
        "level_3_session_horizons": level3_res,
    }

    out_json = WORKSPACE / "live_capture" / "shadow_validation" / "historical_tick_levels_study_results.json"
    with out_json.open("w", encoding="utf-8") as f:
        json.dump(full_results, f, indent=2)

    print("\n" + "=" * 80)
    print(f"Results successfully saved to: {out_json}")
    print("=" * 80)


if __name__ == "__main__":
    main()
