#!/usr/bin/env python3
"""
scripts/emit_rp615_crypto_prospective_scorecard.py

Computes and emits the PURE PROSPECTIVE SHADOW SCORECARD (excluding historical replay)
stratified across Track A (Primary Validation: BTC, ETH) and Track B (Expansion Research: SOL, BNB).

Features:
  1. Pure Prospective Accounting: Evaluates exclusively completed live shadow trades.
  2. Strict Cohort Separation: Track A (C4 acceptance) vs Track B (Exploratory).
  3. Formal Metrics: Wilson 95% CI on Hit Rate, Net Profit Factor, Expectancy, and Average Win/Loss.
  4. Active Position Forensic Accounting: Entry timestamps, holding durations, and settlement deadlines.
"""

from __future__ import annotations

import json
import math
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
PRIMARY_JOURNAL = LOG_DIR / "rp615_crypto_shadow_trade_journal.jsonl"
EXPANSION_JOURNAL = LOG_DIR / "rp615_crypto_expansion_trade_journal.jsonl"
DAEMON_STATE_JSON = LOG_DIR / "rp615_crypto_shadow_daemon_state.json"
TELEMETRY_JSON = LOG_DIR / "rp615_crypto_shadow_telemetry.json"

CANDIDATES = [
    "C0_Reference",
    "C1_Point_Expectancy",
    "C2_Dynamic_BreakEven",
    "C3_Uncertainty_LCB",
    "C4_Dispersion_Adjusted",
    "C5_Fixed_Capacity_Top2Pct",
]

PRIMARY_PAIRS = ["BTCUSDT", "ETHUSDT"]
EXPANSION_PAIRS = ["SOLUSDT", "BNBUSDT"]


def wilson_ci(k: int, n: int, confidence: float = 0.95) -> Tuple[float, float]:
    if n == 0:
        return 0.0, 0.0
    z = 1.95996  # 95% confidence
    p_hat = k / n
    denom = 1.0 + (z**2) / n
    center = (p_hat + (z**2) / (2 * n)) / denom
    margin = (z * math.sqrt((p_hat * (1.0 - p_hat) / n) + ((z**2) / (4 * (n**2))))) / denom
    lower = max(0.0, center - margin) * 100.0
    upper = min(1.0, center + margin) * 100.0
    return round(lower, 2), round(upper, 2)


def load_prospective_closed_trades() -> List[Dict[str, Any]]:
    trades = []
    for j_path, cohort_name in [(PRIMARY_JOURNAL, "TRACK_A_PRIMARY"), (EXPANSION_JOURNAL, "TRACK_B_EXPANSION")]:
        if j_path.exists():
            with j_path.open(encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        item = json.loads(line)
                        if item.get("status") in ("SIMULATED_CLOSE", "SIMULATED_CLOSED"):
                            pnl = float(item.get("net_pnl", item.get("realized_net_pnl", 0.0)))
                            gross = float(item.get("gross_pnl", item.get("realized_gross_pnl", 0.0)))
                            fric = float(item.get("friction_dollars", 18.0))
                            trades.append({
                                "cohort": cohort_name,
                                "candidate_arm": item["candidate_arm"],
                                "trade_id": item["trade_id"],
                                "symbol": item["symbol"],
                                "side": item["side"],
                                "entry_price": float(item["entry_price"]),
                                "exit_price": float(item.get("exit_price", item["entry_price"])),
                                "gross_pnl": gross,
                                "friction_dollars": fric,
                                "net_pnl": pnl,
                                "is_win": 1 if pnl > 0 else 0,
                                "holding_seconds": float(item.get("holding_seconds", 3600.0)),
                            })
                    except Exception:
                        pass
    return trades


def compute_cohort_metrics(trades: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(trades)
    if n == 0:
        return {
            "n_trades": 0, "n_wins": 0, "n_losses": 0, "hit_rate": 0.0, "ci_lower": 0.0, "ci_upper": 0.0,
            "tot_gross": 0.0, "tot_friction": 0.0, "net_pnl": 0.0, "avg_win": 0.0, "avg_loss": 0.0,
            "payoff_ratio": 0.0, "profit_factor": 0.0, "expectancy": 0.0
        }

    wins = [t for t in trades if t["is_win"] == 1]
    losses = [t for t in trades if t["is_win"] == 0]
    n_wins = len(wins)
    n_losses = len(losses)

    hit_rate = (n_wins / n * 100.0) if n > 0 else 0.0
    ci_l, ci_u = wilson_ci(n_wins, n)

    tot_gross = sum(t["gross_pnl"] for t in trades)
    tot_friction = sum(t["friction_dollars"] for t in trades)
    tot_net = sum(t["net_pnl"] for t in trades)

    win_pnl = sum(t["net_pnl"] for t in wins)
    loss_pnl = sum(abs(t["net_pnl"]) for t in losses)

    avg_win = (win_pnl / n_wins) if n_wins > 0 else 0.0
    avg_loss = (loss_pnl / n_losses) if n_losses > 0 else 0.0
    payoff = (avg_win / avg_loss) if avg_loss > 0 else 0.0
    pf = (win_pnl / loss_pnl) if loss_pnl > 0 else (99.0 if win_pnl > 0 else 0.0)
    expectancy = (tot_net / n) if n > 0 else 0.0

    return {
        "n_trades": n,
        "n_wins": n_wins,
        "n_losses": n_losses,
        "hit_rate": round(hit_rate, 2),
        "ci_lower": ci_l,
        "ci_upper": ci_u,
        "tot_gross": round(tot_gross, 2),
        "tot_friction": round(tot_friction, 2),
        "net_pnl": round(tot_net, 2),
        "avg_win": round(avg_win, 2),
        "avg_loss": round(avg_loss, 2),
        "payoff_ratio": round(payoff, 2),
        "profit_factor": round(pf, 2),
        "expectancy": round(expectancy, 2),
    }


def emit_scorecard():
    trades = load_prospective_closed_trades()
    print("=" * 115)
    print("RP-615-CRYPTO: PURE PROSPECTIVE SHADOW SCORECARD (UNPOOLED TRACK A & B)")
    print("Governance: RP-605 Control (100% Live NSE) | RP-615-Crypto (0% Authority - Paper Shadow Only)")
    print("=" * 115)

    # 1. Active Positions & Settlement Deadlines
    if DAEMON_STATE_JSON.exists():
        with DAEMON_STATE_JSON.open(encoding="utf-8") as f:
            state = json.load(f)
        open_details = state.get("open_trade_details", {})
        print("\n[1. Active Position Lifecycle & Settlement Deadlines]")
        print(f"{'Candidate Arm':<26} | {'Symbol':<8} | {'Side':<5} | {'Entry UTC':<24} | {'Entry Price':<11} | {'Settlement Deadline (+3600s)'}")
        print("-" * 115)
        active_found = False
        for cand, sym_dict in open_details.items():
            for sym, t_info in sym_dict.items():
                if t_info:
                    active_found = True
                    entry_dt = datetime.fromisoformat(t_info["entry_time_utc"])
                    deadline = entry_dt.timestamp() + 3600.0
                    deadline_iso = datetime.fromtimestamp(deadline, tz=timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
                    print(f"{cand:<26} | {sym:<8} | {t_info['side']:<5} | {entry_dt.strftime('%Y-%m-%d %H:%M:%S UTC'):<24} | ${t_info['entry_price']:>9,.2f} | {deadline_iso}")
        if not active_found:
            print("  No active open positions (All arms flat).")

    # 2. Track A Primary Prospective Scorecard
    print("\n[2. Track A Primary Validation Cohort (BTCUSDT, ETHUSDT) — Canonical C4 Target]")
    print(f"{'Candidate Arm':<26} | {'Trades':<6} | {'Hit Rate (95% Wilson CI)':<26} | {'Gross P&L':<10} | {'Friction':<9} | {'Net P&L':<10} | {'PF':<5} | {'Exp/Trade':<10}")
    print("-" * 115)
    track_a_trades = [t for t in trades if t["cohort"] == "TRACK_A_PRIMARY"]
    for cand in CANDIDATES:
        c_trades = [t for t in track_a_trades if t["candidate_arm"] == cand]
        m = compute_cohort_metrics(c_trades)
        ci_str = f"{m['hit_rate']:>5.1f}% [{m['ci_lower']:>4.1f}%, {m['ci_upper']:>4.1f}%]" if m['n_trades'] > 0 else "N/A"
        print(f"{cand:<26} | {m['n_trades']:<6} | {ci_str:<26} | ${m['tot_gross']:>8.2f} | ${m['tot_friction']:>7.2f} | ${m['net_pnl']:>8.2f} | {m['profit_factor']:>4.2f} | ${m['expectancy']:>8.2f}")

    # 3. Track B Expansion Research Prospective Scorecard
    print("\n[3. Track B Expansion Research Cohort (SOLUSDT, BNBUSDT) — Exploratory Sandbox]")
    print(f"{'Candidate Arm':<26} | {'Trades':<6} | {'Hit Rate (95% Wilson CI)':<26} | {'Gross P&L':<10} | {'Friction':<9} | {'Net P&L':<10} | {'PF':<5} | {'Exp/Trade':<10}")
    print("-" * 115)
    track_b_trades = [t for t in trades if t["cohort"] == "TRACK_B_EXPANSION"]
    for cand in CANDIDATES:
        c_trades = [t for t in track_b_trades if t["candidate_arm"] == cand]
        m = compute_cohort_metrics(c_trades)
        ci_str = f"{m['hit_rate']:>5.1f}% [{m['ci_lower']:>4.1f}%, {m['ci_upper']:>4.1f}%]" if m['n_trades'] > 0 else "N/A"
        print(f"{cand:<26} | {m['n_trades']:<6} | {ci_str:<26} | ${m['tot_gross']:>8.2f} | ${m['tot_friction']:>7.2f} | ${m['net_pnl']:>8.2f} | {m['profit_factor']:>4.2f} | ${m['expectancy']:>8.2f}")

    # 4. Checkpoint Counter
    c5_track_a = [t for t in track_a_trades if t["candidate_arm"] == "C5_Fixed_Capacity_Top2Pct"]
    print("\n[4. Formal Stage C4 Acceptance Checkpoint Counter (Arm C5 on Track A Only)]")
    print(f"  • Checkpoint 1 Progress (Calibration & Brier Audit): {len(c5_track_a)} / 30 Completed C5 Trades ({len(c5_track_a)/30.0*100.0:.1f}%)")
    print(f"  • Checkpoint 2 Progress (Formal Gating Review):      {len(c5_track_a)} / 150 Completed C5 Trades ({len(c5_track_a)/150.0*100.0:.1f}%)")
    print("=" * 115)


if __name__ == "__main__":
    emit_scorecard()
