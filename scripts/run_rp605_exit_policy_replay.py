#!/usr/bin/env python3
"""
RP-605-EXIT-001: Counterfactual Exit Policy Replay Runner
Protocol Series: RP-605 (NSE Prospective Freshness Validation)

Compares:
  - Policy 0: Control Baseline (Original exit, fixed holding / engine exit)
  - Policy 1: Trailing Profit Lock (Activates at MFE >= +15 bps / ₹150; locks 50% of peak MFE)
  - Policy 2: Volatility Stop Loss (1.5x ATR stop truncation)
  - Policy 3: Hybrid (1.5x ATR Stop + Trailing Profit Lock)

Validation Design:
  - In-Sample Development Set: Sessions 2026-09-28 & 2026-09-29 (1,007 trades)
  - Out-of-Sample Holdout Set: Sessions 2026-09-30 & 2026-10-01 (1,022 trades)
  - Strict Paired Evaluation on Identical Entry Opportunities (Zero entry alteration)

Governance:
  - adapters/chronosentiment remains 100% FROZEN (13/13 discipline invariants)
  - 0.0% Live Order Authority (Shadow Evaluation Only)
"""

import os
import sys
import glob
import json
import math
from pathlib import Path
import pandas as pd
import numpy as np

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
BRAIN_DIR = Path("/Users/nikhil/.gemini/antigravity-ide/brain/9a839485-2655-4595-bf7b-b12603c5146c")

def load_identical_entries():
    files = sorted(glob.glob(str(LOG_DIR / "rp611_nse_*_trades.csv")))
    assert len(files) >= 4, f"Expected 4 sessions, found {len(files)}"
    
    dfs = [pd.read_csv(f) for f in files]
    df = pd.concat(dfs, ignore_index=True)
    assert len(df) == 2029, f"Expected 2029 trades, got {len(df)}"
    return df

def simulate_exit_policy(df_entries: pd.DataFrame, policy_name: str, activation_thresh: float = 150.0, giveback_pct: float = 0.50):
    """
    Simulates counterfactual exit outcomes on identical entry opportunities:
      - Trailing Lock: If MFE >= activation_thresh, locks (1 - giveback_pct) * MFE.
        If the original exit captured more than the lock, keep original (let winner run).
        If the original exit faded below the lock, exit at the locked floor.
      - Stop Loss: If MAE < stop_dist, truncates loss at stop_dist.
    """
    results = []

    for idx, row in df_entries.iterrows():
        orig_gross = float(row["gross_pnl"])
        mfe = float(row["mfe_pnl"])
        mae = float(row["mae_pnl"])
        friction = float(row["friction_inr"])
        stop_dist_bps = float(row.get("stop_dist_bps", 50.0))
        notional = float(row["entry_price"] * row["quantity"])
        stop_dist_inr = -abs((stop_dist_bps / 10000.0) * notional)

        new_gross = orig_gross
        exit_modified = False
        exit_rule_triggered = "BASELINE_EXIT"

        if policy_name == "Policy 0: Control Baseline":
            new_gross = orig_gross
            exit_modified = False
            exit_rule_triggered = row["exit_reason"]

        elif policy_name == "Policy 1: Trailing Profit Lock":
            if mfe >= activation_thresh:
                locked_gross = (1.0 - giveback_pct) * mfe
                if locked_gross > orig_gross:
                    new_gross = locked_gross
                    exit_modified = True
                    exit_rule_triggered = "TRAILING_LOCK_EXIT"

        elif policy_name == "Policy 2: 1.5x ATR Volatility Stop":
            # Truncate loss if mae was worse than stop distance
            if orig_gross < stop_dist_inr and mae <= stop_dist_inr:
                new_gross = stop_dist_inr
                exit_modified = True
                exit_rule_triggered = "VOLATILITY_STOP_EXIT"

        elif policy_name == "Policy 3: Hybrid (Stop Loss + Trailing Lock)":
            # First check if trailing lock protects profit
            if mfe >= activation_thresh:
                locked_gross = (1.0 - giveback_pct) * mfe
                if locked_gross > orig_gross:
                    new_gross = locked_gross
                    exit_modified = True
                    exit_rule_triggered = "TRAILING_LOCK_EXIT"
            # If not protected by trailing lock, check stop loss truncation
            if not exit_modified and orig_gross < stop_dist_inr and mae <= stop_dist_inr:
                new_gross = stop_dist_inr
                exit_modified = True
                exit_rule_triggered = "VOLATILITY_STOP_EXIT"

        new_net = new_gross - friction

        results.append({
            "trade_id": row["trade_id"],
            "session_date": row["session_date"],
            "symbol": row["symbol"],
            "orig_gross": orig_gross,
            "orig_net": float(row["net_pnl"]),
            "replay_gross": round(new_gross, 2),
            "replay_net": round(new_net, 2),
            "friction": friction,
            "notional": notional,
            "exit_modified": exit_modified,
            "exit_rule": exit_rule_triggered
        })

    return pd.DataFrame(results)

def evaluate_performance(df_res: pd.DataFrame, name: str):
    n = len(df_res)
    gross_pnl = df_res["replay_gross"].sum()
    friction = df_res["friction"].sum()
    net_pnl = df_res["replay_net"].sum()

    gross_wins = df_res[df_res["replay_gross"] > 0]
    net_wins = df_res[df_res["replay_net"] > 0]
    gross_losses = df_res[df_res["replay_gross"] < 0]
    net_losses = df_res[df_res["replay_net"] < 0]

    gross_wr = (len(gross_wins) / n) * 100.0
    net_wr = (len(net_wins) / n) * 100.0

    mean_win = gross_wins["replay_gross"].mean() if len(gross_wins) > 0 else 0.0
    mean_loss = gross_losses["replay_gross"].abs().mean() if len(gross_losses) > 0 else 0.0
    payoff_ratio = (mean_win / mean_loss) if mean_loss > 0 else 0.0

    net_exp_inr = net_pnl / n
    total_notional = df_res["notional"].sum()
    net_exp_bps = (net_pnl / total_notional) * 10000.0

    n_mod = df_res["exit_modified"].sum()

    return {
        "Policy": name,
        "Total Trades": int(n),
        "Modified Exits (%)": round((n_mod / n) * 100.0, 1),
        "Gross Win Rate (%)": round(gross_wr, 1),
        "Net Win Rate (%)": round(net_wr, 1),
        "Gross P&L (₹)": round(gross_pnl, 2),
        "Statutory Charges (₹)": round(friction, 2),
        "Net P&L (₹)": round(net_pnl, 2),
        "Avg Win (₹)": round(mean_win, 2),
        "Avg Loss (₹)": round(mean_loss, 2),
        "Payoff Ratio": round(payoff_ratio, 2),
        "Net Exp (₹/trade)": round(net_exp_inr, 2),
        "Net Exp (bps)": round(net_exp_bps, 2)
    }

def main():
    print("=" * 100)
    print("RP-605-EXIT-001: COUNTERFACTUAL EXIT POLICY REPLAY EXPERIMENT")
    print("=" * 100)

    df_entries = load_identical_entries()
    print(f"Loaded {len(df_entries)} frozen entry records across 4 sessions.")

    # In-Sample (Sep 28 & 29) vs Out-of-Sample (Sep 30 & Oct 01)
    is_mask = df_entries["session_date"].isin(["2026-09-28", "2026-09-29"])
    oos_mask = df_entries["session_date"].isin(["2026-09-30", "2026-10-01"])

    df_is = df_entries[is_mask].copy()
    df_oos = df_entries[oos_mask].copy()
    print(f"Development Set (In-Sample): {len(df_is)} trades | Validation Set (Out-of-Sample): {len(df_oos)} trades\n")

    policies = [
        "Policy 0: Control Baseline",
        "Policy 1: Trailing Profit Lock",
        "Policy 2: 1.5x ATR Volatility Stop",
        "Policy 3: Hybrid (Stop Loss + Trailing Lock)"
    ]

    all_pooled_results = []
    is_results = []
    oos_results = []

    for pol in policies:
        res_pooled = simulate_exit_policy(df_entries, pol)
        res_is = simulate_exit_policy(df_is, pol)
        res_oos = simulate_exit_policy(df_oos, pol)

        all_pooled_results.append(evaluate_performance(res_pooled, pol))
        is_results.append(evaluate_performance(res_is, pol))
        oos_results.append(evaluate_performance(res_oos, pol))

    # Print Formatted Comparison Table
    print("=== POOLED OVERALL PERFORMANCE (N = 2,029 TRADES) ===")
    pooled_df = pd.DataFrame(all_pooled_results)
    baseline_net = pooled_df.loc[0, "Net P&L (₹)"]
    pooled_df["Paired Delta (₹)"] = pooled_df["Net P&L (₹)"] - baseline_net
    print(pooled_df[["Policy", "Modified Exits (%)", "Net Win Rate (%)", "Gross P&L (₹)", "Net P&L (₹)", "Paired Delta (₹)", "Net Exp (bps)", "Payoff Ratio"]].to_string(index=False))

    print("\n=== IN-SAMPLE DEVELOPMENT SET (Sep 28 - 29, N = 1,007) ===")
    is_df = pd.DataFrame(is_results)
    is_df["Paired Delta (₹)"] = is_df["Net P&L (₹)"] - is_df.loc[0, "Net P&L (₹)"]
    print(is_df[["Policy", "Net Win Rate (%)", "Net P&L (₹)", "Paired Delta (₹)", "Net Exp (bps)", "Payoff Ratio"]].to_string(index=False))

    print("\n=== OUT-OF-SAMPLE VALIDATION SET (Sep 30 - Oct 01, N = 1,022) ===")
    oos_df = pd.DataFrame(oos_results)
    oos_df["Paired Delta (₹)"] = oos_df["Net P&L (₹)"] - oos_df.loc[0, "Net P&L (₹)"]
    print(oos_df[["Policy", "Net Win Rate (%)", "Net P&L (₹)", "Paired Delta (₹)", "Net Exp (bps)", "Payoff Ratio"]].to_string(index=False))

    # Save to JSON
    out_file = LOG_DIR / "rp605_exit_replay_results.json"
    with open(out_file, "w") as f:
        json.dump({
            "pooled": all_pooled_results,
            "in_sample": is_results,
            "out_of_sample": oos_results
        }, f, indent=2)
    print(f"\nSaved structured exit replay results to {out_file}")
    print("=" * 100)

if __name__ == "__main__":
    main()
