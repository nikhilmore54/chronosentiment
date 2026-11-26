#!/usr/bin/env python3
"""
scripts/research/counterfactual_55_trades_revaluation.py

Milestone 2 (RP-615-EE-003): Historical 55-Trade Counterfactual Re-Evaluation

Evaluates the settled prospective Track-A cohort (N = 55 trades) across:
1. Baseline Reference: Fixed 18 bps Taker Friction ($18/trade).
2. Volume-Trend Filter (Lens 1): Pruning setups with decaying volume (VolAccel < 0, RVOL < 1.0).
3. Empirical Maker Execution (Lens 2): Maker fee (2.0 bps) + adverse drift (0.50 bps) = 2.5 bps.
4. Combined Optimization (Lens 3): C5 Ranking (Q3/Q4) + Volume-Trend Filter + Maker Economics.
5. Fill Feasibility Haircut (Lens 4): Queue-clearance fill sensitivity (100% vs 80% vs 60%).

Outputs:
  - live_capture/shadow_validation/rp615_counterfactual_revaluation_results.json
"""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Any, Dict, List, Tuple

import numpy as np
import pandas as pd

WORKSPACE = Path(__file__).resolve().parent.parent.parent
JOURNAL_PATH = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_crypto_shadow_trade_journal.jsonl"
CARDS_PATH = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_crypto_shadow_decision_cards.jsonl"
OUTPUT_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_counterfactual_revaluation_results.json"

NOTIONAL_PER_TRADE = 10000.0  # $10,000 spot trade notional
TAKER_FRICTION_BPS = 18.0
MAKER_FEE_BPS = 2.0
EMPIRICAL_ADVERSE_SELECTION_BPS = 0.50
NET_MAKER_FRICTION_BPS = MAKER_FEE_BPS + EMPIRICAL_ADVERSE_SELECTION_BPS  # 2.5 bps


def load_dataset() -> Tuple[pd.DataFrame, pd.DataFrame]:
    # 1. Load closed C0_Reference trades
    closes = []
    with JOURNAL_PATH.open("r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                item = json.loads(line)
                if item.get("status") == "SIMULATED_CLOSE" and item.get("candidate_arm") == "C0_Reference":
                    closes.append(item)

    # 2. Load decision cards
    cards = {}
    with CARDS_PATH.open("r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                c = json.loads(line)
                t_id = c.get("trade_id")
                if t_id:
                    cards[t_id] = c

    records = []
    for c in closes:
        tid = c["trade_id"]
        stripped_tid = tid.replace("C0_Reference-", "")
        card = cards.get(tid) or cards.get(stripped_tid)
        
        feats = card.get("features", {}) if card else {}
        scores = card.get("scores", {}) if card else {}
        
        p_cal = float(scores.get("p_calibrated", 0.0))
        pred_net = float(scores.get("pred_net_pnl", 0.0))
        hurdle_score = p_cal * pred_net

        records.append({
            "trade_id": tid,
            "timestamp_utc": c.get("timestamp_utc"),
            "symbol": c["symbol"],
            "side": c["side"],
            "entry_price": float(c["entry_price"]),
            "exit_price": float(c["exit_price"]),
            "gross_pnl": float(c["gross_pnl"]),
            "gross_return_bps": (float(c["gross_pnl"]) / NOTIONAL_PER_TRADE) * 10000.0,
            "taker_friction_dollars": float(c["friction_dollars"]),
            "taker_net_pnl": float(c["net_pnl"]),
            # Microstructure features
            "rvol_15m": float(feats.get("F04_rvol_15m", 1.0)),
            "vol_accel": float(feats.get("F12_vol_accel", 0.0)),
            "ret_15m": float(feats.get("F01_ret_15m", 0.0)),
            "atr_norm": float(feats.get("F05_atr_norm", 0.0)),
            "sigma_60": float(feats.get("F06_sigma_60", 0.0)),
            # Model scores
            "p_calibrated": p_cal,
            "pred_net_pnl": pred_net,
            "c5_hurdle_score": hurdle_score,
        })

    df_all = pd.DataFrame(records)
    df_55 = df_all.iloc[:55].copy().reset_index(drop=True)
    return df_55, df_all


def compute_metrics(df_sub: pd.DataFrame, friction_bps: float) -> Dict[str, Any]:
    n = len(df_sub)
    if n == 0:
        return {
            "n_trades": 0, "win_rate": 0.0, "total_gross_pnl": 0.0, "total_friction": 0.0,
            "total_net_pnl": 0.0, "mean_net_pnl": 0.0, "profit_factor": 0.0, "expectancy_bps": 0.0
        }

    friction_per_trade = NOTIONAL_PER_TRADE * (friction_bps / 10000.0)
    gross_pnl = df_sub["gross_pnl"]
    net_pnl = gross_pnl - friction_per_trade

    wins = net_pnl[net_pnl > 0]
    losses = net_pnl[net_pnl <= 0]
    n_wins = len(wins)
    n_losses = len(losses)
    win_rate = (n_wins / n) * 100.0

    tot_gross = float(gross_pnl.sum())
    tot_fric = float(n * friction_per_trade)
    tot_net = float(net_pnl.sum())
    mean_net = float(net_pnl.mean())

    gross_wins = gross_pnl[gross_pnl > 0].sum()
    gross_losses = abs(gross_pnl[gross_pnl < 0].sum())
    gross_pf = (gross_wins / max(gross_losses, 1e-4)) if gross_losses > 0 else 99.0

    net_wins = wins.sum()
    net_losses = abs(losses.sum())
    net_pf = (net_wins / max(net_losses, 1e-4)) if net_losses > 0 else (99.0 if net_wins > 0 else 0.0)

    # Standard deviation of net returns for Sharpe ratio calculation
    std_net = float(net_pnl.std()) if len(net_pnl) > 1 else 1.0
    sharpe = (mean_net / std_net) * math.sqrt(n) if std_net > 0 else 0.0

    return {
        "n_trades": n,
        "n_wins": n_wins,
        "n_losses": n_losses,
        "win_rate_pct": round(win_rate, 2),
        "total_gross_pnl": round(tot_gross, 2),
        "total_friction_dollars": round(tot_fric, 2),
        "total_net_pnl": round(tot_net, 2),
        "mean_net_pnl_per_trade": round(mean_net, 2),
        "mean_gross_bps": round(float(df_sub["gross_return_bps"].mean()), 2),
        "mean_net_bps": round(float((net_pnl / NOTIONAL_PER_TRADE).mean() * 10000.0), 2),
        "gross_profit_factor": round(gross_pf, 2),
        "net_profit_factor": round(net_pf, 2),
        "sharpe_ratio": round(sharpe, 2),
    }


def run_counterfactual_analysis():
    print("=" * 95)
    print("RP-615-EE-003: HISTORICAL 55-TRADE COUNTERFACTUAL RE-EVALUATION")
    print("Ground-Truth Simulation: Volume Trend Conditioning & Maker Execution Economics")
    print("=" * 95)

    df_55, df_127 = load_dataset()
    print(f"Loaded {len(df_55)} original Track-A settled trades (and {len(df_127)} total to date).")

    # -------------------------------------------------------------
    # Scenario 1: The Baseline Reference (18 bps Taker, Zero Selectivity)
    # -------------------------------------------------------------
    s1_baseline = compute_metrics(df_55, TAKER_FRICTION_BPS)

    # -------------------------------------------------------------
    # Scenario 2: Volume-Trend Conditioning Only (Taker 18 bps)
    # Filter out decaying volume: Require VolAccel >= 0.0 and RVOL >= 0.80
    # -------------------------------------------------------------
    mask_vol_expanding = (df_55["vol_accel"] >= 0.0) & (df_55["rvol_15m"] >= 0.80)
    df_vol_expanded = df_55[mask_vol_expanding]
    df_vol_decaying = df_55[~mask_vol_expanding]
    s2_vol_expanded = compute_metrics(df_vol_expanded, TAKER_FRICTION_BPS)
    s2_vol_decaying = compute_metrics(df_vol_decaying, TAKER_FRICTION_BPS)

    # -------------------------------------------------------------
    # Scenario 3: Empirical Maker Execution Only (2.5 bps net friction, All 55 trades)
    # -------------------------------------------------------------
    s3_maker_all = compute_metrics(df_55, NET_MAKER_FRICTION_BPS)

    # -------------------------------------------------------------
    # Scenario 4: C5 Ranking Conditioning (Top 50% Scores: Q3 & Q4) under Taker vs Maker
    # -------------------------------------------------------------
    median_c5_score = df_55["c5_hurdle_score"].median()
    df_c5_top_half = df_55[df_55["c5_hurdle_score"] >= median_c5_score]
    df_c5_bot_half = df_55[df_55["c5_hurdle_score"] < median_c5_score]
    s4_c5_top_taker = compute_metrics(df_c5_top_half, TAKER_FRICTION_BPS)
    s4_c5_top_maker = compute_metrics(df_c5_top_half, NET_MAKER_FRICTION_BPS)

    # -------------------------------------------------------------
    # Scenario 5: The Combined Strategy (C5 Top Half + Volume Trend + Maker Economics)
    # -------------------------------------------------------------
    mask_combined = (df_55["c5_hurdle_score"] >= median_c5_score) & (df_55["vol_accel"] >= 0.0)
    df_combined = df_55[mask_combined]
    s5_combined_maker = compute_metrics(df_combined, NET_MAKER_FRICTION_BPS)
    s5_combined_taker = compute_metrics(df_combined, TAKER_FRICTION_BPS)

    # -------------------------------------------------------------
    # Scenario 6: Queue Clearance Sensitivity (80% and 60% Fill Probability)
    # -------------------------------------------------------------
    # If 20% of orders fail queue clearance, they incur $0 friction and $0 P&L
    s6_maker_80pct_fill = {
        "assumed_fill_rate": "80%",
        "expected_net_pnl": round(s5_combined_maker["total_net_pnl"] * 0.80, 2),
        "mean_net_per_attempted_trade": round(s5_combined_maker["mean_net_pnl_per_trade"] * 0.80, 2),
        "profit_factor": s5_combined_maker["net_profit_factor"],
    }
    s6_maker_60pct_fill = {
        "assumed_fill_rate": "60%",
        "expected_net_pnl": round(s5_combined_maker["total_net_pnl"] * 0.60, 2),
        "mean_net_per_attempted_trade": round(s5_combined_maker["mean_net_pnl_per_trade"] * 0.60, 2),
        "profit_factor": s5_combined_maker["net_profit_factor"],
    }

    # Print Comparison Table
    print("\n" + "=" * 95)
    print(f"{'Execution & Selectivity Scenario':<42} | {'N':>4} | {'Win%':>6} | {'Gross PnL':>10} | {'Net PnL':>10} | {'Net PF':>6} | {'Net/Trd':>8}")
    print("-" * 95)

    scenarios = [
        ("1. Baseline (Unfiltered C0 @ 18 bps Taker)", s1_baseline),
        ("2A. Decaying Volume (VolAccel < 0 @ 18 bps)", s2_vol_decaying),
        ("2B. Expanding Volume (VolAccel >= 0 @ 18 bps)", s2_vol_expanded),
        ("3. Maker Economics (All 55 @ 2.5 bps)", s3_maker_all),
        ("4A. C5 Top Half (Q3+Q4 @ 18 bps Taker)", s4_c5_top_taker),
        ("4B. C5 Top Half (Q3+Q4 @ 2.5 bps Maker)", s4_c5_top_maker),
        ("5A. Combined (C5 + VolAccel >= 0 @ 18 bps)", s5_combined_taker),
        ("5B. Combined (C5 + VolAccel >= 0 @ 2.5 bps)", s5_combined_maker),
    ]

    for label, res in scenarios:
        print(f"{label:<42} | {res['n_trades']:4d} | {res['win_rate_pct']:5.1f}% | ${res['total_gross_pnl']:>9.2f} | ${res['total_net_pnl']:>9.2f} | {res['net_profit_factor']:6.2f} | ${res['mean_net_pnl_per_trade']:>7.2f}")

    print("=" * 95)

    results_payload = {
        "cohort": "TRACK_A_PRIMARY_55_SETTLED",
        "sample_size": 55,
        "scenarios": {
            "s1_baseline_taker_18bps": s1_baseline,
            "s2_volume_decaying_taker_18bps": s2_vol_decaying,
            "s2_volume_expanding_taker_18bps": s2_vol_expanded,
            "s3_maker_all_2_5bps": s3_maker_all,
            "s4_c5_top_half_taker_18bps": s4_c5_top_taker,
            "s4_c5_top_half_maker_2_5bps": s4_c5_top_maker,
            "s5_combined_taker_18bps": s5_combined_taker,
            "s5_combined_maker_2_5bps": s5_combined_maker,
            "s6_queue_clearance_sensitivity": {
                "fill_rate_80pct": s6_maker_80pct_fill,
                "fill_rate_60pct": s6_maker_60pct_fill,
            }
        }
    }

    with OUTPUT_JSON.open("w", encoding="utf-8") as f:
        json.dump(results_payload, f, indent=2)

    print(f"\nFull counterfactual results saved to: {OUTPUT_JSON}")


if __name__ == "__main__":
    run_counterfactual_analysis()
