#!/usr/bin/env python3
"""
scripts/research/evaluate_shrp001_a1_predictive_study.py

RP-608 Structural Hypothesis Research Plan (SHRP-001)
Stage 1: Gate A Predictive Feasibility Study for Hypothesis A1

Hypothesis A1: Intraday Volatility-State Transition (Compression -> Expansion)
Evaluation Partition: Strictly In-Sample Development (Sessions 1-40: 2026-06-22 to 2026-08-18).
Data Frequency: Primary 15-Minute Completed OHLCV Bars.

Governing Invariants:
- Sessions 41-67 strictly SEALED & UNTOUCHED.
- Live order authority: Strictly 0.0%.
- Two-Gate protocol: Zero trading strategy / stops / take-profit optimization. Pure predictive study.
"""

from __future__ import annotations

import json
import math
from datetime import datetime, timezone, timedelta, date
from pathlib import Path
from typing import Dict, List, Tuple, Any
import numpy as np

WORKSPACE_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
CACHE_15M_DIR = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m"
OUTPUT_DIR = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_FILE = OUTPUT_DIR / "rp608_shrp001_a1_predictive_study_results.json"

IST_OFFSET = timedelta(hours=5, minutes=30)
DEV_START_DATE = "2026-06-22"
DEV_END_DATE = "2026-08-18"


NSE_2026_HOLIDAYS = {
    date(2026, 1, 26), date(2026, 3, 3), date(2026, 3, 26),
    date(2026, 4, 2), date(2026, 4, 14), date(2026, 5, 1),
    date(2026, 5, 28), date(2026, 6, 17), date(2026, 7, 2),
    date(2026, 8, 15), date(2026, 10, 2), date(2026, 10, 20),
    date(2026, 11, 8), date(2026, 11, 9), date(2026, 11, 24),
    date(2026, 12, 25)
}


def parse_ist_datetime(ts: float) -> Tuple[str, int, date]:
    """Converts unix timestamp to (session_date_str, bar_minute_of_day, date_obj)."""
    ts_sec = ts / 1000.0 if ts > 1e11 else ts
    dt_utc = datetime.fromtimestamp(ts_sec, tz=timezone.utc)
    dt_ist = dt_utc + IST_OFFSET
    d = dt_ist.date()
    date_str = dt_ist.strftime("%Y-%m-%d")
    minute_of_day = dt_ist.hour * 60 + dt_ist.minute
    return date_str, minute_of_day, d


def load_development_candles() -> Tuple[List[str], Dict[str, Dict[str, List[Dict[str, float]]]]]:
    """Loads and structures 15m candles for Sessions 1-40 across all symbols."""
    symbols = []
    symbol_sessions: Dict[str, Dict[str, List[Dict[str, float]]]] = {}
    all_sessions_set = set()

    for p in sorted(CACHE_15M_DIR.glob("*.NS.json")):
        sym = p.stem.replace(".json", "")
        with open(p, "r", encoding="utf-8") as f:
            raw_candles = json.load(f)

        sessions_map: Dict[str, List[Dict[str, float]]] = {}
        for c in raw_candles:
            ts = c.get("timestamp")
            if ts is None or c.get("open") is None or c.get("close") is None:
                continue
            d_str, minute, d_obj = parse_ist_datetime(ts)
            if d_obj.weekday() >= 5 or d_obj in NSE_2026_HOLIDAYS:
                continue
            if DEV_START_DATE <= d_str <= DEV_END_DATE:
                all_sessions_set.add(d_str)
                if d_str not in sessions_map:
                    sessions_map[d_str] = []
                sessions_map[d_str].append({
                    "open": float(c["open"]),
                    "high": float(c["high"]),
                    "low": float(c["low"]),
                    "close": float(c["close"]),
                    "volume": float(c.get("volume", 0.0)),
                    "timestamp": ts,
                    "minute": minute
                })

        # Ensure candles within each session are sorted by timestamp
        for d in sessions_map:
            sessions_map[d].sort(key=lambda x: x["timestamp"])

        symbol_sessions[sym] = sessions_map
        symbols.append(sym)

    sorted_dev_sessions = sorted(list(all_sessions_set))
    return sorted_dev_sessions, symbol_sessions


def run_a1_predictive_study():
    print("=" * 80)
    print("  RP-608 SHRP-001: GATE A PREDICTIVE STUDY — HYPOTHESIS A1")
    print("  Intraday Volatility-State Transition (15-Minute Data)")
    print("  Development Window: Sessions 1–40 (2026-06-22 to 2026-08-18)")
    print("=" * 80)

    dev_sessions, symbol_sessions = load_development_candles()
    print(f"Loaded {len(symbol_sessions)} symbols across {len(dev_sessions)} development sessions.")
    assert len(dev_sessions) == 40, f"Expected 40 development sessions, found {len(dev_sessions)}"

    # Step 1: Pre-calculate per-symbol compression thresholds
    # Compression metric: 4-bar rolling normalized range: (max(H_k-3..k) - min(L_k-3..k)) / C_k
    symbol_ranges: Dict[str, List[float]] = {s: [] for s in symbol_sessions}

    for s, sessions in symbol_sessions.items():
        for d in dev_sessions:
            bars = sessions.get(d, [])
            for i in range(3, len(bars)):
                w = bars[i-3:i+1]
                h_max = max(b["high"] for b in w)
                l_min = min(b["low"] for b in w)
                c_curr = w[-1]["close"]
                if c_curr > 0:
                    symbol_ranges[s].append((h_max - l_min) / c_curr)

    # Calculate 20th percentile (quintile 1) compression threshold per symbol
    symbol_comp_thresh = {
        s: float(np.percentile(symbol_ranges[s], 20)) if len(symbol_ranges[s]) > 20 else 0.005
        for s in symbol_ranges
    }

    # Step 2: Signal Generation & Outcome Measurement
    # Primary horizon: h=4 (60m). Secondary horizon: h=8 (120m)
    signals = []
    baseline_bars = []

    for s, sessions in symbol_sessions.items():
        thresh = symbol_comp_thresh[s]
        for d_idx, d in enumerate(dev_sessions):
            bars = sessions.get(d, [])
            n_bars = len(bars)
            for i in range(3, n_bars):
                # 4-bar compression check at bar i-1
                w = bars[i-3:i+1]
                h_max = max(b["high"] for b in w)
                l_min = min(b["low"] for b in w)
                c_curr = w[-1]["close"]
                norm_range = (h_max - l_min) / c_curr if c_curr > 0 else 1.0

                # Check if bar i triggers expansion out of compression
                # Trigger bar k is bar i
                bar_k = bars[i]
                bar_prev = bars[i-1]

                # Record baseline bar outcomes if horizon fits within the day
                for h in (4, 8):
                    if i + h < n_bars:
                        ret_fwd = (bars[i+h]["close"] - bar_k["close"]) / bar_k["close"]
                        baseline_bars.append({
                            "symbol": s,
                            "session": d,
                            "session_idx": d_idx,
                            "bar_idx": i,
                            "horizon": h,
                            "fwd_ret": ret_fwd,
                            "abs_excursion": abs(ret_fwd)
                        })

                # Check signal condition
                # 1) Compression: rolling range <= 20th percentile
                # 2) Expansion trigger: Close breaks prev High or Low
                is_compressed = (norm_range <= thresh)
                is_expansion_up = (bar_k["close"] > bar_prev["high"])
                is_expansion_down = (bar_k["close"] < bar_prev["low"])

                if is_compressed and (is_expansion_up or is_expansion_down):
                    direction = +1 if is_expansion_up else -1
                    candle_sign = +1 if bar_k["close"] >= bar_k["open"] else -1

                    # Record signal outcome only if horizon completes inside the same session
                    outcome_4 = None
                    outcome_8 = None
                    if i + 4 < n_bars:
                        ret_4 = (bars[i+4]["close"] - bar_k["close"]) / bar_k["close"]
                        outcome_4 = {
                            "fwd_ret": ret_4,
                            "abs_excursion": abs(ret_4),
                            "signed_ret": direction * ret_4,
                            "is_correct_dir": (direction * ret_4 > 0)
                        }

                    if i + 8 < n_bars:
                        ret_8 = (bars[i+8]["close"] - bar_k["close"]) / bar_k["close"]
                        outcome_8 = {
                            "fwd_ret": ret_8,
                            "abs_excursion": abs(ret_8),
                            "signed_ret": direction * ret_8,
                            "is_correct_dir": (direction * ret_8 > 0)
                        }

                    signals.append({
                        "symbol": s,
                        "session": d,
                        "session_idx": d_idx,
                        "bar_idx": i,
                        "minute": bar_k["minute"],
                        "direction": direction,
                        "candle_sign": candle_sign,
                        "norm_range": norm_range,
                        "h4": outcome_4,
                        "h8": outcome_8
                    })

    # Step 3: Statistical Analysis across Folds & Session Clusters
    # 5 chronological folds of 8 sessions each
    fold_assignments = [d_idx // 8 for d_idx in range(40)]

    def analyze_horizon(h_key: str, h_val: int):
        sig_items = [s for s in signals if s[h_key] is not None]
        base_items = [b for b in baseline_bars if b["horizon"] == h_val]

        sig_abs = np.array([s[h_key]["abs_excursion"] * 10000 for s in sig_items])  # bps
        base_abs = np.array([b["abs_excursion"] * 10000 for b in base_items])  # bps
        sig_signed = np.array([s[h_key]["signed_ret"] * 10000 for s in sig_items])  # bps
        sig_dir_correct = np.array([1 if s[h_key]["is_correct_dir"] else 0 for s in sig_items])

        # Excursion effect size (Cohen's d)
        mean_sig_abs = float(np.mean(sig_abs))
        mean_base_abs = float(np.mean(base_abs))
        std_sig_abs = float(np.std(sig_abs, ddof=1))
        std_base_abs = float(np.std(base_abs, ddof=1))
        pooled_std = math.sqrt(((len(sig_abs)-1)*std_sig_abs**2 + (len(base_abs)-1)*std_base_abs**2) / (len(sig_abs) + len(base_abs) - 2))
        cohens_d = (mean_sig_abs - mean_base_abs) / pooled_std if pooled_std > 0 else 0.0

        # Session-clustered standard errors (clustering on session_idx)
        sessions_unique = sorted(list(set(s["session_idx"] for s in sig_items)))
        session_signed_means = []
        session_win_rates = []
        for s_idx in sessions_unique:
            s_signed = [s[h_key]["signed_ret"] * 10000 for s in sig_items if s["session_idx"] == s_idx]
            s_dirs = [1 if s[h_key]["is_correct_dir"] else 0 for s in sig_items if s["session_idx"] == s_idx]
            if s_signed:
                session_signed_means.append(np.mean(s_signed))
                session_win_rates.append(np.mean(s_dirs))

        cluster_mean_signed = float(np.mean(session_signed_means))
        cluster_se_signed = float(np.std(session_signed_means, ddof=1) / math.sqrt(len(session_signed_means)))
        t_stat_signed = cluster_mean_signed / cluster_se_signed if cluster_se_signed > 0 else 0.0

        win_rate = float(np.mean(sig_dir_correct)) * 100.0
        cluster_se_win = float(np.std(session_win_rates, ddof=1) / math.sqrt(len(session_win_rates))) * 100.0

        # Fold breakdown
        fold_stats = []
        for f in range(5):
            f_sigs = [s for s in sig_items if (s["session_idx"] // 8) == f]
            if f_sigs:
                f_abs = [s[h_key]["abs_excursion"] * 10000 for s in f_sigs]
                f_signed = [s[h_key]["signed_ret"] * 10000 for s in f_sigs]
                f_win = [1 if s[h_key]["is_correct_dir"] else 0 for s in f_sigs]
                fold_stats.append({
                    "fold": f + 1,
                    "count": len(f_sigs),
                    "mean_abs_excursion_bps": float(np.mean(f_abs)),
                    "mean_signed_ret_bps": float(np.mean(f_signed)),
                    "win_rate_pct": float(np.mean(f_win)) * 100.0
                })

        return {
            "eligible_signals_count": len(sig_items),
            "baseline_observations_count": len(base_items),
            "mean_signal_abs_excursion_bps": mean_sig_abs,
            "mean_baseline_abs_excursion_bps": mean_base_abs,
            "absolute_expansion_diff_bps": mean_sig_abs - mean_base_abs,
            "cohens_d": cohens_d,
            "mean_signed_ret_bps": float(np.mean(sig_signed)),
            "cluster_robust_mean_signed_bps": cluster_mean_signed,
            "cluster_robust_se_signed_bps": cluster_se_signed,
            "cluster_t_stat": t_stat_signed,
            "directional_win_rate_pct": win_rate,
            "cluster_se_win_rate_pct": cluster_se_win,
            "fold_breakdown": fold_stats
        }

    h4_results = analyze_horizon("h4", 4)
    h8_results = analyze_horizon("h8", 8)

    # Gate A Evaluation against Falsification Standards:
    # 1. Cohen's d >= 0.20
    # 2. Directional win rate >= 51.0%
    # 3. Mean signed forward gross return >= +18.0 bps
    h4_passed = (
        h4_results["cohens_d"] >= 0.20 and
        h4_results["directional_win_rate_pct"] >= 51.0 and
        h4_results["mean_signed_ret_bps"] >= 18.0
    )

    gate_decision = "PASS" if h4_passed else "FALSIFIED"

    results_payload = {
        "hypothesis_id": "SHRP-001-A1-VOL-EXPANSION",
        "protocol": "RP-608-SHRP-001",
        "gate": "GATE_A_PREDICTIVE_FEASIBILITY",
        "evaluation_partition": "Sessions 1-40 (2026-06-22 to 2026-08-18)",
        "holdout_status": "SEALED_AND_UNTOUCHED (Sessions 41-67)",
        "live_order_authority": "0.0%",
        "total_symbols_evaluated": len(symbol_sessions),
        "total_development_sessions": len(dev_sessions),
        "total_signals_generated": len(signals),
        "primary_horizon_4_bars_60m": h4_results,
        "secondary_horizon_8_bars_120m": h8_results,
        "falsification_criteria": {
            "cohens_d_threshold": 0.20,
            "directional_win_rate_min_pct": 51.0,
            "gross_return_hurdle_bps": 18.0
        },
        "gate_decision": gate_decision,
        "verdict_reasoning": (
            f"Primary horizon (h=4, 60m): Cohen's d={h4_results['cohens_d']:.4f} (req >= 0.20), "
            f"Win Rate={h4_results['directional_win_rate_pct']:.2f}% (req >= 51.0%), "
            f"Mean Signed Return={h4_results['mean_signed_ret_bps']:.2f} bps (req >= +18.0 bps). "
            f"Gate A Status: {gate_decision}."
        )
    }

    with open(RESULTS_FILE, "w", encoding="utf-8") as f:
        json.dump(results_payload, f, indent=2)

    print("\n--- RESULTS SUMMARY ---")
    print(f"Total Signals: {len(signals)}")
    print(f"Primary Horizon (h=4, 60m):")
    print(f"  Signal Abs Excursion: {h4_results['mean_signal_abs_excursion_bps']:.2f} bps vs Baseline: {h4_results['mean_baseline_abs_excursion_bps']:.2f} bps")
    print(f"  Expansion Effect Size (Cohen's d): {h4_results['cohens_d']:.4f}")
    print(f"  Directional Win Rate: {h4_results['directional_win_rate_pct']:.2f}% (SE: {h4_results['cluster_se_win_rate_pct']:.2f}%)")
    print(f"  Mean Signed Return: {h4_results['mean_signed_ret_bps']:.2f} bps (Cluster t-stat: {h4_results['cluster_t_stat']:.2f})")
    print(f"\nSecondary Horizon (h=8, 120m):")
    print(f"  Signal Abs Excursion: {h8_results['mean_signal_abs_excursion_bps']:.2f} bps vs Baseline: {h8_results['mean_baseline_abs_excursion_bps']:.2f} bps")
    print(f"  Expansion Effect Size (Cohen's d): {h8_results['cohens_d']:.4f}")
    print(f"  Directional Win Rate: {h8_results['directional_win_rate_pct']:.2f}% (SE: {h8_results['cluster_se_win_rate_pct']:.2f}%)")
    print(f"  Mean Signed Return: {h8_results['mean_signed_ret_bps']:.2f} bps (Cluster t-stat: {h8_results['cluster_t_stat']:.2f})")
    print(f"\nGate Decision: {gate_decision}")
    print(f"Saved results to {RESULTS_FILE}")


if __name__ == "__main__":
    run_a1_predictive_study()
