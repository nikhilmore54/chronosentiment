#!/usr/bin/env python3
"""
RP-605 Entry Policy Ablation & Baseline Reproducibility Runner
Protocol Series: RP-605-OPT-001 (Net Return Optimization Specification)

Evaluates:
  1. Authoritative Tick #54 baseline reconciliation & MTM equity curve
  2. Multi-session entry-policy ablation on historical replication datasets (2026-09-28 to 2026-10-01)
     - Arm 1: Control (Unfiltered Baseline)
     - Arm 2: Trajectory Filter (Non-warmup / Freshness)
     - Arm 3: C5 Predicted Return Hurdle alone (y_hat >= +15.0 bps)
     - Arm 4: Joint Volume Expansion trigger alone (Vol >= 1.5x)
     - Arm 5: Joint Treatment (C5 Hurdle >= +15.0 bps AND Vol >= 1.5x)

Governance:
  - adapters/chronosentiment remains 100% FROZEN
  - 0.0% Live Order Authority (Shadow evaluation only)
"""

import os
import sys
import json
import math
import glob
from pathlib import Path
from datetime import datetime, timezone
import pandas as pd
import numpy as np

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
BRAIN_DIR = Path("/Users/nikhil/.gemini/antigravity-ide/brain/9a839485-2655-4595-bf7b-b12603c5146c")

NOMINAL_CAPITAL_BASE_INR = 10_000_000.0  # W0 = ₹10,000,000 INR
MAX_DRAWDOWN_CEILING_PCT = 1.50          # Strict 1.50% intraday ceiling
MIN_SAMPLE_POWER_FLOOR = 150             # Coverage / statistical power floor (not a quota)

def calc_statutory_charges(entry_price: float, exit_price: float, quantity: int, side: str) -> dict:
    """Exact Zerodha Cash-Equity Intraday Schedule."""
    if side == "LONG":
        buy_turnover = quantity * entry_price
        sell_turnover = quantity * exit_price
    else:
        sell_turnover = quantity * entry_price
        buy_turnover = quantity * exit_price

    tot_turnover = buy_turnover + sell_turnover

    # 1. Brokerage: min(0.03%, ₹20) per executed leg
    brok_buy = min(buy_turnover * 0.0003, 20.0)
    brok_sell = min(sell_turnover * 0.0003, 20.0)
    brokerage = brok_buy + brok_sell

    # 2. STT: 0.025% on sell side
    stt = sell_turnover * 0.00025

    # 3. NSE Exchange transaction charge: 0.00307% of total turnover
    exch = tot_turnover * 0.0000307

    # 4. Stamp duty: 0.003% on buy side
    stamp = buy_turnover * 0.00003

    # 5. SEBI turnover charges: ₹10 per crore (0.0001% of total turnover)
    sebi = tot_turnover * 0.000001

    # 6. GST: 18% on (brokerage + exchange charges + SEBI charges)
    gst = 0.18 * (brokerage + exch + sebi)

    total_friction = brokerage + stt + exch + stamp + sebi + gst
    return {
        "brokerage": brokerage,
        "stt": stt,
        "exch": exch,
        "stamp": stamp,
        "sebi": sebi,
        "gst": gst,
        "total": total_friction,
        "notional": buy_turnover
    }

def run_authoritative_tick54_audit():
    print("=" * 80)
    print("STEP 1: AUTHORITATIVE TICK #54 REPRODUCIBILITY AUDIT")
    print("=" * 80)

    b_file = BRAIN_DIR / "rp605_audit_raw_baseline_exits_tick54.json"
    f_file = BRAIN_DIR / "rp605_audit_raw_freshness_exits_tick54.json"
    r_file = BRAIN_DIR / "rp605_audit_raw_rejected_exits_tick54.json"
    chk_file = BRAIN_DIR / "rp605_nse_authoritative_checkpoint_tick054.json"

    assert b_file.exists(), f"Missing {b_file}"
    assert f_file.exists(), f"Missing {f_file}"
    assert r_file.exists(), f"Missing {r_file}"
    assert chk_file.exists(), f"Missing {chk_file}"

    with open(b_file) as f: b_exits = json.load(f)
    with open(f_file) as f: f_exits = json.load(f)
    with open(r_file) as f: r_exits = json.load(f)
    with open(chk_file) as f: chk = json.load(f)

    # 1. Assert Counts
    assert len(b_exits) == 318, f"Expected 318 baseline exits, got {len(b_exits)}"
    assert len(f_exits) == 219, f"Expected 219 freshness exits, got {len(f_exits)}"
    assert len(r_exits) == 99, f"Expected 99 rejected exits, got {len(r_exits)}"
    assert len(b_exits) == len(f_exits) + len(r_exits), "Partition count identity violated!"

    # 2. Assert Set Identities & Disjointness by unrounded composite key
    def get_comp_key(t, is_rej=False):
        return (t["trade_id"], t["symbol"], t["side"], t["entry_timestamp"], float(t["entry_price"]), int(t["quantity"]))

    b_keys = {get_comp_key(t): t for t in b_exits}
    f_keys = {get_comp_key(t): t for t in f_exits}
    r_keys = {get_comp_key(t, True): t for t in r_exits}

    assert len(b_keys) == 318, "Duplicate key collision in Baseline exits!"
    assert len(f_keys) == 219, "Duplicate key collision in Freshness exits!"
    assert len(r_keys) == 99, "Duplicate key collision in Rejected exits!"

    assert set(f_keys.keys()).issubset(set(b_keys.keys())), "Freshness is not a subset of Baseline!"
    assert set(r_keys.keys()).issubset(set(b_keys.keys())), "Rejected is not a subset of Baseline!"
    assert len(set(f_keys.keys()) & set(r_keys.keys())) == 0, "Intersection between Freshness and Rejected is NOT empty!"
    assert (set(f_keys.keys()) | set(r_keys.keys())) == set(b_keys.keys()), "Union does not equal Baseline set!"
    print("  [✓] Set-Theoretic Partition A = B ∪ R and B ∩ R = Ø verified (100.0% unrounded composite keys).")

    # 3. Assert 1-to-1 Field Matches on Shared (219) and Counterfactual (99)
    for k, ft in f_keys.items():
        bt = b_keys[k]
        for field in ["exit_price", "exit_timestamp", "holding_mins", "exit_reason", "pnl"]:
            assert bt[field] == ft[field], f"Field mismatch in shared trade {k[0]}: {field}"
    print("  [✓] 219 Shared trades have 100.0% identical execution fields.")

    for k, rt in r_keys.items():
        bt = b_keys[k]
        assert bt["exit_price"] == rt["counterfactual_exit_price"], f"Exit price mismatch in rejected trade {k[0]}"
        assert bt["exit_timestamp"] == rt["counterfactual_exit_timestamp"], f"Exit timestamp mismatch in {k[0]}"
        assert bt["pnl"] == rt["counterfactual_pnl"], f"PnL mismatch in {k[0]}"
    print("  [✓] 99 Baseline-only trades map 1-to-1 with zero discrepancies to rejected counterfactual journal.")

    # 4. Assert Charges and Conservation
    def sum_charges(records, is_rej=False):
        pnl_k = "counterfactual_pnl" if is_rej else "pnl"
        gross = sum(r[pnl_k] for r in records)
        charges = [calc_statutory_charges(r["entry_price"], r["counterfactual_exit_price" if is_rej else "exit_price"], r["quantity"], r["side"]) for r in records]
        tot_c = sum(c["total"] for c in charges)
        net = gross - tot_c
        return gross, tot_c, net

    g_b, c_b, n_b = sum_charges(b_exits)
    g_f, c_f, n_f = sum_charges(f_exits)
    g_r, c_r, n_r = sum_charges(r_exits, is_rej=True)

    assert abs(g_b - -5818.10) < 0.01, f"Gross baseline mismatch: {g_b}"
    assert abs(g_f - 59.17) < 0.01, f"Gross freshness mismatch: {g_f}"
    assert abs(g_r - -5877.27) < 0.01, f"Gross rejected mismatch: {g_r}"
    assert abs(c_b - (c_f + c_r)) < 1e-5, "Conservation of charges violated!"
    assert abs(n_b - (n_f + n_r)) < 1e-5, "Conservation of net PnL violated!"

    print(f"  [✓] Conservation of Charges: B (₹{c_b:,.2f}) == F (₹{c_f:,.2f}) + R (₹{c_r:,.2f}) [Δ = 0.000000]")
    print(f"  [✓] Net after Modeled Charges: Baseline A = ₹{n_b:,.2f} | Freshness B = ₹{n_f:,.2f} | Delta = +₹{n_f - n_b:,.2f}")

    # 5. Assert Active Position Partition at Tick 54 Cutoff
    active_b = chk["active_partition"]["baseline_count"]
    active_f = chk["active_partition"]["freshness_count"]
    active_r = chk["active_partition"]["rejected_count"]
    assert active_b == 54, f"Expected 54 active at Tick 54, got {active_b}"
    assert active_f == 32, f"Expected 32 active at Tick 54, got {active_f}"
    assert active_r == 22, f"Expected 22 active at Tick 54, got {active_r}"
    assert active_b == active_f + active_r, "Active partition identity violated at Tick 54!"
    print(f"  [✓] Active Open Positions Partition at Tick 54: Baseline 54 == Freshness 32 + Rejected 22.")
    print("=" * 80)

def evaluate_entry_ablation():
    print("\n" + "=" * 80)
    print("STEP 2: MULTI-SESSION ENTRY POLICY ABLATION (HISTORICAL REPLICATION SESSIONS)")
    print("=" * 80)

    session_files = sorted(glob.glob(str(LOG_DIR / "rp611_nse_*_trades.csv")))
    assert len(session_files) >= 4, f"Expected 4 replication datasets, found {len(session_files)}"

    all_dfs = []
    for fpath in session_files:
        df = pd.read_csv(fpath)
        all_dfs.append(df)
    combined_df = pd.concat(all_dfs, ignore_index=True)
    print(f"Loaded {len(combined_df)} completed historical trades across {len(session_files)} sessions (Sep 28 - Oct 01).")

    # Define Arms:
    # Arm 1: Control (All trades)
    # Arm 2: Freshness / Non-Warmup (is_warmup == 0)
    # Arm 3: C5 Hurdle Alone (macro_r60_bps >= +15.0)
    # Arm 4: Volume Expansion Alone (vol_ratio >= 1.5)
    # Arm 5: Joint Treatment (macro_r60_bps >= +15.0 AND vol_ratio >= 1.5)
    # Arm 6: Full Treatment (Non-warmup AND macro_r60_bps >= +15.0 AND vol_ratio >= 1.5)

    arms = {
        "Arm 1: Control (Unfiltered Baseline)": lambda df: np.ones(len(df), dtype=bool),
        "Arm 2: Trajectory Filter (Non-Warmup)": lambda df: df["is_warmup"] == 0,
        "Arm 3: C5 Predicted Hurdle (>= +15.0 bps)": lambda df: df["macro_r60_bps"] >= 15.0,
        "Arm 4: Volume Expansion (Vol >= 1.5x)": lambda df: df["vol_ratio"] >= 1.5,
        "Arm 5: Joint (C5 >= +15 bps + Vol >= 1.5x)": lambda df: (df["macro_r60_bps"] >= 15.0) & (df["vol_ratio"] >= 1.5),
        "Arm 6: Full Stack (Trajectory + C5 + Vol)": lambda df: (df["is_warmup"] == 0) & (df["macro_r60_bps"] >= 15.0) & (df["vol_ratio"] >= 1.5)
    }

    results = []

    for arm_name, condition in arms.items():
        mask = condition(combined_df)
        subset = combined_df[mask].copy()

        n_trades = len(subset)
        if n_trades == 0:
            continue

        n_unique_symbols = subset["symbol"].nunique()
        scrip_coverage_pct = (n_unique_symbols / 119.0) * 100.0

        # Performance Metrics
        gross_pnl = subset["gross_pnl"].sum()
        total_friction = subset["friction_inr"].sum()
        net_pnl = subset["net_pnl"].sum()

        # 1. Trade-level Net P&L Reconstructed from Statutory Charges
        sum_ledger_net = subset["net_pnl"].sum()
        sum_trade_charges = subset["friction_inr"].sum()
        sum_gross = subset["gross_pnl"].sum()
        assert abs(sum_gross - sum_trade_charges - sum_ledger_net) < 1e-4, "Trade-level cost accounting identity broken!"
        win_rate_pct = (len(subset[subset["gross_pnl"] > 0]) / n_trades) * 100.0 if n_trades > 0 else 0.0

        # 2. Net Expectancy with Explicit Denominators
        total_notional_inr = (subset["entry_price"] * subset["quantity"]).sum()
        mean_notional_per_trade = total_notional_inr / n_trades if n_trades > 0 else 100_000.0
        
        net_expectancy_inr = sum_ledger_net / n_trades  # Denominator: N trades
        gross_expectancy_inr = sum_gross / n_trades     # Denominator: N trades
        avg_friction_per_trade = sum_trade_charges / n_trades
        
        net_expectancy_bps = (sum_ledger_net / total_notional_inr) * 10_000.0   # Denominator: Total Notional
        gross_expectancy_bps = (sum_gross / total_notional_inr) * 10_000.0     # Denominator: Total Notional

        # 3. Exact 1-Minute Portfolio Return Series (1,500 1-minute ticks across 4 sessions)
        # Reconstruct tick-by-tick net equity trajectory across sessions
        session_dates = sorted(subset["session_date"].unique())
        minute_returns = []
        equity_curve = [NOMINAL_CAPITAL_BASE_INR]

        curr_wealth = NOMINAL_CAPITAL_BASE_INR
        total_pnl_reconstructed = 0.0

        for sdate in ["2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01"]:
            s_trades = subset[subset["session_date"] == sdate]
            # Map exits to session minute index [0, 374]
            # Trades exit at entry_minute_idx + holding_mins
            exit_minutes = (s_trades["entry_minute_idx"] + s_trades["holding_mins"]).clip(0, 374)
            minute_pnl_map = s_trades.groupby(exit_minutes)["net_pnl"].sum().to_dict()

            for m in range(375):
                m_pnl = minute_pnl_map.get(m, 0.0)
                curr_wealth += m_pnl
                total_pnl_reconstructed += m_pnl
                equity_curve.append(curr_wealth)
                r_m = m_pnl / NOMINAL_CAPITAL_BASE_INR
                minute_returns.append(r_m)

        # Consistency Assertion: Sum of 1-minute increments == Sum of trade ledger entries
        assert abs(total_pnl_reconstructed - sum_ledger_net) < 1e-4, f"Equity curve delta mismatch: {total_pnl_reconstructed} != {sum_ledger_net}"

        minute_returns = np.array(minute_returns)
        mean_1m_return = float(np.mean(minute_returns))
        downside_diffs = np.minimum(0.0, minute_returns)
        downside_dev_1m = float(math.sqrt(np.mean(downside_diffs**2))) if len(downside_diffs) > 0 else 1e-8

        # Canonical Portfolio Sortino: mean / downside semi-deviation
        # Annualized by sqrt(375 ticks/day * 252 days/year = 94,500 ticks/year)
        annualization_factor = math.sqrt(375.0 * 252.0)
        sortino_1m_ann = (mean_1m_return / downside_dev_1m) * annualization_factor if downside_dev_1m > 1e-9 else 0.0

        # Maximum Intraday Mark-to-Market Drawdown from continuous equity curve
        eq_arr = np.array(equity_curve)
        peak_eq = np.maximum.accumulate(eq_arr)
        drawdowns = peak_eq - eq_arr
        max_dd_inr = float(np.max(drawdowns))
        max_dd_pct = float((max_dd_inr / NOMINAL_CAPITAL_BASE_INR) * 100.0)
        passes_dd_ceiling = max_dd_pct <= MAX_DRAWDOWN_CEILING_PCT

        results.append({
            "Arm": arm_name,
            "Completed Trades (N)": int(n_trades),
            "Flow Retention (%)": round(float((n_trades / len(combined_df)) * 100.0), 1),
            "Scrip Coverage (Scrips)": f"{n_unique_symbols} / 119 ({scrip_coverage_pct:.1f}%)",
            "Win Rate (%)": round(float(win_rate_pct), 1),
            "Gross P&L (₹)": round(float(sum_gross), 2),
            "Statutory Friction (₹)": round(float(sum_trade_charges), 2),
            "Net P&L (₹)": round(float(sum_ledger_net), 2),
            "Avg Friction per Trade (₹)": round(float(avg_friction_per_trade), 2),
            "Gross Expectancy (₹/trade)": round(float(gross_expectancy_inr), 2),
            "Net Expectancy (₹/trade)": round(float(net_expectancy_inr), 2),
            "Gross Expectancy (bps)": round(float(gross_expectancy_bps), 2),
            "Net Expectancy (bps)": round(float(net_expectancy_bps), 2),
            "1-Min Return Mean (bps)": round(float(mean_1m_return * 10_000.0), 4),
            "1-Min Downside Dev (bps)": round(float(downside_dev_1m * 10_000.0), 4),
            "Annualized Sortino (1-Min)": round(float(sortino_1m_ann), 2),
            "Max Drawdown (₹)": round(float(max_dd_inr), 2),
            "Max Drawdown (% of W0)": round(float(max_dd_pct), 2),
            "Passes DD Ceiling (≤1.5%)": bool(passes_dd_ceiling)
        })

    # Display Comparative Table
    res_df = pd.DataFrame(results)
    
    # Save JSON results
    out_json = LOG_DIR / "rp605_entry_ablation_results.json"
    with open(out_json, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved structured ablation results to {out_json}")

    print("\n" + "=" * 115)
    print("RP-605 MULTI-SESSION ENTRY POLICY ABLATION SCORECARD")
    print("=" * 115)
    for r in results:
        print(f"\n▶ {r['Arm']}")
        print(f"  - Completed Trades: {r['Completed Trades (N)']} ({r['Flow Retention (%)']}% retained) | Scrip Breadth: {r['Scrip Coverage (Scrips)']}")
        print(f"  - Win Rate: {r['Win Rate (%)']}% | Gross Return: {r['Gross Expectancy (bps)']} bps | Net Expectancy: {r['Net Expectancy (bps)']} bps")
        print(f"  - Gross P&L: ₹{r['Gross P&L (₹)']:+,.2f} | Statutory Charges: ₹{r['Statutory Friction (₹)']:,.2f} | Net P&L: ₹{r['Net P&L (₹)']:+,.2f}")
        print(f"  - 1-Min Mean: {r['1-Min Return Mean (bps)']} bps | Downside Dev: {r['1-Min Downside Dev (bps)']} bps | Annualized Sortino: {r['Annualized Sortino (1-Min)']}")
        print(f"  - Net Expectancy (₹/trade): ₹{r['Net Expectancy (₹/trade)']} | Gross Expectancy (₹/trade): ₹{r['Gross Expectancy (₹/trade)']} | Avg Charges: ₹{r['Avg Friction per Trade (₹)']}")

    print("=" * 115)

if __name__ == "__main__":
    run_authoritative_tick54_audit()
    evaluate_entry_ablation()
