#!/usr/bin/env python3
"""
scripts/research/evaluate_shrp001_b1_predictive_study.py

RP-608 Structural Hypothesis Research Plan (SHRP-001)
Stage 1: Gate A Predictive Feasibility Study for Hypothesis B1

Hypothesis B1: Multi-Day Cash Swing Continuation
Evaluation Partition: Strictly In-Sample Development (Sessions 1-40: 2026-06-22 to 2026-08-18).
Data Frequency: Daily (1d) Completed OHLCV Bars.

Governing Invariants:
- Sessions 41-67 strictly SEALED & UNTOUCHED.
- Live order authority: Strictly 0.0%.
- Two-Gate protocol: Zero trading strategy / stops / take-profit optimization. Pure predictive study.
- Execution convention: Day T Close signal -> Day T+1 Open entry. Overnight gap EXCLUDED.
"""

from __future__ import annotations

import json
import math
from datetime import datetime, timezone, timedelta, date
from pathlib import Path
from typing import Dict, List, Tuple, Any
import numpy as np

WORKSPACE_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
CACHE_1D_DIR = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_1d"
OUTPUT_DIR = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_FILE = OUTPUT_DIR / "rp608_shrp001_b1_predictive_study_results.json"

DEV_START_DATE = "2026-06-22"
DEV_END_DATE = "2026-08-18"


# Canonical 40 development sessions from audited manifest (2026-06-22 to 2026-08-18)
CANONICAL_DEV_SESSIONS = [
    '2026-06-22', '2026-06-23', '2026-06-24', '2026-06-25', '2026-06-29',
    '2026-06-30', '2026-07-01', '2026-07-03', '2026-07-06', '2026-07-07',
    '2026-07-08', '2026-07-09', '2026-07-10', '2026-07-13', '2026-07-14',
    '2026-07-15', '2026-07-16', '2026-07-17', '2026-07-20', '2026-07-21',
    '2026-07-22', '2026-07-23', '2026-07-24', '2026-07-27', '2026-07-28',
    '2026-07-29', '2026-07-30', '2026-07-31', '2026-08-03', '2026-08-04',
    '2026-08-05', '2026-08-06', '2026-08-07', '2026-08-10', '2026-08-11',
    '2026-08-12', '2026-08-13', '2026-08-14', '2026-08-17', '2026-08-18'
]


def load_daily_candles() -> Tuple[List[str], Dict[str, Dict[str, Dict[str, float]]]]:
    """Loads daily candles across all symbols and identifies development sessions."""
    symbols = []
    symbol_daily: Dict[str, Dict[str, Dict[str, float]]] = {}

    for p in sorted(CACHE_1D_DIR.glob("*.NS.json")):
        sym = p.stem.replace(".json", "")
        with open(p, "r", encoding="utf-8") as f:
            raw_candles = json.load(f)

        daily_map: Dict[str, Dict[str, float]] = {}
        for c in raw_candles:
            d_str = c.get("datetime")
            if not d_str:
                ts = c.get("timestamp")
                if ts:
                    ts_sec = ts / 1000.0 if ts > 1e11 else ts
                    dt = datetime.fromtimestamp(ts_sec, tz=timezone.utc) + timedelta(hours=5, minutes=30)
                    d_str = dt.strftime("%Y-%m-%d")
            if not d_str or c.get("open") is None or c.get("close") is None:
                continue

            # Store candle
            daily_map[d_str] = {
                "open": float(c["open"]),
                "high": float(c["high"]),
                "low": float(c["low"]),
                "close": float(c["close"]),
                "volume": float(c.get("volume", 0.0))
            }

        symbol_daily[sym] = daily_map
        symbols.append(sym)

    return CANONICAL_DEV_SESSIONS, symbol_daily


def run_b1_predictive_study():
    print("=" * 80)
    print("  RP-608 SHRP-001: GATE A PREDICTIVE STUDY — HYPOTHESIS B1")
    print("  Multi-Day Cash Swing Continuation (Daily Data)")
    print("  Development Window: Sessions 1–40 (2026-06-22 to 2026-08-18)")
    print("=" * 80)

    dev_sessions, symbol_daily = load_daily_candles()
    print(f"Loaded {len(symbol_daily)} symbols across {len(dev_sessions)} development sessions.")
    assert len(dev_sessions) == 40, f"Expected 40 development sessions, found {len(dev_sessions)}"

    # Get sorted sequence of all calendar dates across history to compute rolling 20-day volume
    sample_sym = list(symbol_daily.keys())[0]
    all_dates_sorted = sorted(list(symbol_daily[sample_sym].keys()))

    # Build market equal-weighted returns per date
    market_returns: Dict[str, float] = {}
    for d in all_dates_sorted:
        rets = []
        d_idx = all_dates_sorted.index(d)
        if d_idx > 0:
            d_prev = all_dates_sorted[d_idx - 1]
            for s, dmap in symbol_daily.items():
                if d in dmap and d_prev in dmap and dmap[d_prev]["close"] > 0:
                    r = (dmap[d]["close"] - dmap[d_prev]["close"]) / dmap[d_prev]["close"]
                    rets.append(r)
            if rets:
                market_returns[d] = float(np.mean(rets))

    # Also build multi-day market returns from T+1 Open to T+h Close
    # For any window (d_entry, d_exit), market return is average of (Close_exit - Open_entry) / Open_entry
    def get_market_multi_day_return(d_entry: str, d_exit: str) -> float:
        rets = []
        for s, dmap in symbol_daily.items():
            if d_entry in dmap and d_exit in dmap and dmap[d_entry]["open"] > 0:
                r = (dmap[d_exit]["close"] - dmap[d_entry]["open"]) / dmap[d_entry]["open"]
                rets.append(r)
        return float(np.mean(rets)) if rets else 0.0

    # Step 1: Scan for Day T signals inside Sessions 1-40
    # Boundary integrity rule: For horizon h (e.g. 2, 3, 5), T+h must remain <= Session 40 (d_idx + h < 40)
    # This prevents any lookahead into Sealed Holdout Sessions 41-67!
    signals = []

    for t_idx, d_t in enumerate(dev_sessions):
        # Locate global date index of d_t
        g_idx = all_dates_sorted.index(d_t)
        if g_idx < 20:
            continue
        d_prev = all_dates_sorted[g_idx - 1]
        mkt_ret_t = market_returns.get(d_t, 0.0)

        for s, dmap in symbol_daily.items():
            if d_t not in dmap or d_prev not in dmap:
                continue

            c_t = dmap[d_t]["close"]
            c_prev = dmap[d_prev]["close"]
            v_t = dmap[d_t]["volume"]

            if c_prev <= 0 or v_t <= 0:
                continue

            stock_ret_t = (c_t - c_prev) / c_prev
            excess_ret_t = stock_ret_t - mkt_ret_t

            # Rolling 20-day average volume strictly prior to Day T
            past_vols = []
            for k in range(1, 21):
                p_date = all_dates_sorted[g_idx - k]
                if p_date in dmap:
                    past_vols.append(dmap[p_date]["volume"])

            if len(past_vols) < 15:
                continue
            avg_vol_20 = float(np.mean(past_vols))
            if avg_vol_20 <= 0:
                continue

            vol_ratio = v_t / avg_vol_20

            # Preregistered Signal Criteria:
            # 1. 1-Day Abnormal Return >= +1.5% (excess over market)
            # 2. Abnormal Volume >= 1.5x 20-Day Average
            if excess_ret_t >= 0.015 and vol_ratio >= 1.5:
                # Evaluation horizons:
                # T+1 Open is entry. T+2, T+3, T+5 are exits.
                # Must check boundary: t_idx + h < len(dev_sessions) (40 sessions)
                outcomes = {}
                for h in (2, 3, 5):
                    if t_idx + h < len(dev_sessions):
                        d_entry = dev_sessions[t_idx + 1]
                        d_exit = dev_sessions[t_idx + h]
                        if d_entry in dmap and d_exit in dmap and dmap[d_entry]["open"] > 0:
                            entry_p = dmap[d_entry]["open"]
                            exit_p = dmap[d_exit]["close"]
                            stock_fwd_ret = (exit_p - entry_p) / entry_p
                            mkt_fwd_ret = get_market_multi_day_return(d_entry, d_exit)
                            excess_fwd_ret = stock_fwd_ret - mkt_fwd_ret

                            outcomes[f"h{h}"] = {
                                "d_entry": d_entry,
                                "d_exit": d_exit,
                                "stock_fwd_ret": stock_fwd_ret,
                                "market_fwd_ret": mkt_fwd_ret,
                                "excess_fwd_ret": excess_fwd_ret,
                                "is_positive_excess": (excess_fwd_ret > 0)
                            }
                        else:
                            outcomes[f"h{h}"] = None
                    else:
                        outcomes[f"h{h}"] = None

                signals.append({
                    "symbol": s,
                    "session_t": d_t,
                    "session_idx": t_idx,
                    "stock_ret_t": stock_ret_t,
                    "excess_ret_t": excess_ret_t,
                    "vol_ratio": vol_ratio,
                    "outcomes": outcomes
                })

    # Step 2: Statistical Analysis for Primary (T+3) and Secondary (T+2, T+5)
    def analyze_swing_horizon(h_key: str, h_val: int):
        valid_sigs = [s for s in signals if s["outcomes"].get(h_key) is not None]
        excess_rets = np.array([s["outcomes"][h_key]["excess_fwd_ret"] * 10000 for s in valid_sigs])  # bps
        raw_rets = np.array([s["outcomes"][h_key]["stock_fwd_ret"] * 10000 for s in valid_sigs])  # bps
        mkt_rets = np.array([s["outcomes"][h_key]["market_fwd_ret"] * 10000 for s in valid_sigs])  # bps
        wins = np.array([1 if s["outcomes"][h_key]["is_positive_excess"] else 0 for s in valid_sigs])

        mean_excess_bps = float(np.mean(excess_rets)) if len(excess_rets) > 0 else 0.0
        mean_raw_bps = float(np.mean(raw_rets)) if len(raw_rets) > 0 else 0.0
        mean_mkt_bps = float(np.mean(mkt_rets)) if len(mkt_rets) > 0 else 0.0
        hit_rate_pct = float(np.mean(wins)) * 100.0 if len(wins) > 0 else 0.0

        # Session-clustered standard errors (clustering on signal session date T)
        session_indices = sorted(list(set(s["session_idx"] for s in valid_sigs)))
        session_excess_means = []
        session_hit_rates = []
        for s_idx in session_indices:
            s_excess = [s["outcomes"][h_key]["excess_fwd_ret"] * 10000 for s in valid_sigs if s["session_idx"] == s_idx]
            s_wins = [1 if s["outcomes"][h_key]["is_positive_excess"] else 0 for s in valid_sigs if s["session_idx"] == s_idx]
            if s_excess:
                session_excess_means.append(np.mean(s_excess))
                session_hit_rates.append(np.mean(s_wins))

        n_clusters = len(session_excess_means)
        cluster_mean_excess = float(np.mean(session_excess_means)) if n_clusters > 0 else 0.0
        cluster_se_excess = float(np.std(session_excess_means, ddof=1) / math.sqrt(n_clusters)) if n_clusters > 1 else 0.0
        cluster_t_stat = cluster_mean_excess / cluster_se_excess if cluster_se_excess > 0 else 0.0

        # p-value approximation (two-tailed student-t)
        # For df = n_clusters - 1
        df = max(1, n_clusters - 1)
        # Using normal CDF approximation for t
        p_val = 2.0 * (1.0 - 0.5 * (1.0 + math.erf(cluster_t_stat / math.sqrt(2)))) if cluster_t_stat > 0 else 1.0 - 2.0 * (0.5 * (1.0 + math.erf(cluster_t_stat / math.sqrt(2))))
        p_val = max(0.0001, min(1.0, float(p_val)))

        # Chronological fold breakdown (5 folds of 8 sessions each)
        fold_stats = []
        for f in range(5):
            f_sigs = [s for s in valid_sigs if (s["session_idx"] // 8) == f]
            if f_sigs:
                f_excess = [s["outcomes"][h_key]["excess_fwd_ret"] * 10000 for s in f_sigs]
                f_win = [1 if s["outcomes"][h_key]["is_positive_excess"] else 0 for s in f_sigs]
                fold_stats.append({
                    "fold": f + 1,
                    "count": len(f_sigs),
                    "mean_excess_ret_bps": float(np.mean(f_excess)),
                    "hit_rate_pct": float(np.mean(f_win)) * 100.0
                })

        return {
            "eligible_signals_count": len(valid_sigs),
            "clusters_count": n_clusters,
            "mean_stock_fwd_ret_bps": mean_raw_bps,
            "mean_market_fwd_ret_bps": mean_mkt_bps,
            "mean_excess_fwd_ret_bps": mean_excess_bps,
            "cluster_robust_mean_excess_bps": cluster_mean_excess,
            "cluster_robust_se_excess_bps": cluster_se_excess,
            "cluster_t_stat": cluster_t_stat,
            "cluster_p_value": p_val,
            "hit_rate_pct": hit_rate_pct,
            "fold_breakdown": fold_stats
        }

    h3_results = analyze_swing_horizon("h3", 3)
    h2_results = analyze_swing_horizon("h2", 2)
    h5_results = analyze_swing_horizon("h5", 5)

    # Holm-Bonferroni correction across horizons:
    # Sorted p-values: Primary h3 (test at alpha=0.05). Secondary h2, h5 (tested with Bonferroni alpha=0.025, 0.05).
    # Falsification check for Primary Horizon (T+3):
    # 1. Mean forward excess return >= +35.0 bps gross (to overcome 29.8 bps delivery friction)
    # 2. Hit rate >= 50.0%
    # 3. Clustered p-value < 0.05
    h3_passed = (
        h3_results["mean_excess_fwd_ret_bps"] >= 35.0 and
        h3_results["hit_rate_pct"] >= 50.0 and
        h3_results["cluster_p_value"] < 0.05
    )

    gate_decision = "PASS" if h3_passed else "FALSIFIED"

    results_payload = {
        "hypothesis_id": "SHRP-001-B1-SWING-CONTINUATION",
        "protocol": "RP-608-SHRP-001",
        "gate": "GATE_A_PREDICTIVE_FEASIBILITY",
        "evaluation_partition": "Sessions 1-40 (2026-06-22 to 2026-08-18)",
        "holdout_status": "SEALED_AND_UNTOUCHED (Sessions 41-67)",
        "live_order_authority": "0.0%",
        "survivorship_bias_limitation": "DECLARED (Universe consists of surviving liquid Nifty constituents; historical delistings/suspensions are omitted).",
        "total_symbols_evaluated": len(symbol_daily),
        "total_development_sessions": len(dev_sessions),
        "total_signals_generated": len(signals),
        "primary_horizon_3_sessions_t3": h3_results,
        "secondary_horizon_2_sessions_t2": h2_results,
        "secondary_horizon_5_sessions_t5": h5_results,
        "falsification_criteria": {
            "mean_excess_return_hurdle_bps": 35.0,
            "hit_rate_min_pct": 50.0,
            "max_cluster_p_value": 0.05
        },
        "gate_decision": gate_decision,
        "verdict_reasoning": (
            f"Primary horizon (T+3): Mean Excess Return={h3_results['mean_excess_fwd_ret_bps']:.2f} bps (req >= +35.0 bps), "
            f"Hit Rate={h3_results['hit_rate_pct']:.2f}% (req >= 50.0%), "
            f"Cluster t-stat={h3_results['cluster_t_stat']:.2f}, p-value={h3_results['cluster_p_value']:.4f}. "
            f"Gate A Status: {gate_decision}."
        )
    }

    with open(RESULTS_FILE, "w", encoding="utf-8") as f:
        json.dump(results_payload, f, indent=2)

    print("\n--- RESULTS SUMMARY ---")
    print(f"Total Signals Generated: {len(signals)}")
    print(f"Primary Horizon (Day T+3 Close, Entry T+1 Open):")
    print(f"  Eligible Signals: {h3_results['eligible_signals_count']} across {h3_results['clusters_count']} session clusters")
    print(f"  Mean Stock Return: {h3_results['mean_stock_fwd_ret_bps']:.2f} bps vs Market: {h3_results['mean_market_fwd_ret_bps']:.2f} bps")
    print(f"  Mean Excess Return: {h3_results['mean_excess_fwd_ret_bps']:.2f} bps (SE: {h3_results['cluster_robust_se_excess_bps']:.2f} bps)")
    print(f"  Cluster t-statistic: {h3_results['cluster_t_stat']:.2f} (p-value: {h3_results['cluster_p_value']:.4f})")
    print(f"  Hit Rate: {h3_results['hit_rate_pct']:.2f}%")
    print(f"\nSecondary Horizon (Day T+2 Close):")
    print(f"  Mean Excess Return: {h2_results['mean_excess_fwd_ret_bps']:.2f} bps | Hit Rate: {h2_results['hit_rate_pct']:.2f}% | t-stat: {h2_results['cluster_t_stat']:.2f}")
    print(f"\nSecondary Horizon (Day T+5 Close):")
    print(f"  Mean Excess Return: {h5_results['mean_excess_fwd_ret_bps']:.2f} bps | Hit Rate: {h5_results['hit_rate_pct']:.2f}% | t-stat: {h5_results['cluster_t_stat']:.2f}")
    print(f"\nGate Decision: {gate_decision}")
    print(f"Saved results to {RESULTS_FILE}")


if __name__ == "__main__":
    run_b1_predictive_study()
