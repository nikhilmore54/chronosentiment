#!/usr/bin/env python3
"""
scripts/emit_rp615_crypto_live_metrics.py

Computes and formats the real-time Hit Rate & Payoff Ratio Decision Dashboard
across all candidate arms (C0–C5) and instruments (BTCUSDT, ETHUSDT, SOLUSDT, BNBUSDT).
"""

from __future__ import annotations

import csv
import json
import math
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, List

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
PRIMARY_JOURNAL = LOG_DIR / "rp615_crypto_shadow_trade_journal.jsonl"
EXPANSION_JOURNAL = LOG_DIR / "rp615_crypto_expansion_trade_journal.jsonl"
TELEMETRY_JSON = LOG_DIR / "rp615_crypto_shadow_telemetry.json"
CANDIDATE_REPORT_MD = LOG_DIR / "rp615_crypto_stage2_candidate_report.md"
STAGE2_EXECUTION_LEDGER = LOG_DIR / "rp615_crypto_stage2_execution_ledger.csv"

CANDIDATES = [
    "C0_Reference",
    "C1_Point_Expectancy",
    "C2_Dynamic_BreakEven",
    "C3_Uncertainty_LCB",
    "C4_Dispersion_Adjusted",
    "C5_Fixed_Capacity_Top2Pct",
]

PAIRS = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT"]


def load_all_completed_trades() -> List[Dict[str, Any]]:
    trades = []
    # Load Stage 2 Replay Executions (Historical Out-of-Sample Reference)
    if STAGE2_EXECUTION_LEDGER.exists():
        with STAGE2_EXECUTION_LEDGER.open() as f:
            reader = csv.DictReader(f)
            for r in reader:
                trades.append({
                    "source": "HISTORICAL_REPLAY",
                    "candidate_arm": r["candidate_arm"],
                    "symbol": r["symbol"],
                    "realized_net_pnl": float(r["realized_net_pnl"]),
                    "is_win": 1 if float(r["realized_net_pnl"]) > 0 else 0,
                })

    # Load Prospective Live Closed Trades
    for j_path, cohort_name in [(PRIMARY_JOURNAL, "TRACK_A_PRIMARY"), (EXPANSION_JOURNAL, "TRACK_B_EXPANSION")]:
        if j_path.exists():
            with j_path.open() as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        item = json.loads(line)
                        status = item.get("status", "")
                        if status in ("SIMULATED_CLOSE", "SIMULATED_CLOSED"):
                            pnl = float(item.get("net_pnl", item.get("realized_net_pnl", 0.0)))
                            trades.append({
                                "source": "LIVE_PROSPECTIVE",
                                "cohort": cohort_name,
                                "candidate_arm": item["candidate_arm"],
                                "symbol": item["symbol"],
                                "realized_net_pnl": pnl,
                                "is_win": 1 if pnl > 0 else 0,
                            })
                    except Exception:
                        pass
    return trades


def compute_metrics(trades: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(trades)
    if n == 0:
        return {
            "n_trades": 0, "hit_rate": 0.0, "avg_win": 0.0, "avg_loss": 0.0,
            "payoff_ratio": 0.0, "profit_factor": 0.0, "expectancy": 0.0, "be_hit_rate": 50.0
        }

    wins = [t for t in trades if t["is_win"] == 1]
    losses = [t for t in trades if t["is_win"] == 0]
    n_wins = len(wins)
    n_losses = len(losses)

    hit_rate = (n_wins / n * 100.0) if n > 0 else 0.0
    tot_win_pnl = sum(t["realized_net_pnl"] for t in wins)
    tot_loss_pnl = sum(abs(t["realized_net_pnl"]) for t in losses)

    avg_win = (tot_win_pnl / n_wins) if n_wins > 0 else 0.0
    avg_loss = (tot_loss_pnl / n_losses) if n_losses > 0 else 0.0
    payoff_ratio = (avg_win / avg_loss) if avg_loss > 0 else 0.0
    profit_factor = (tot_win_pnl / tot_loss_pnl) if tot_loss_pnl > 0 else (99.0 if tot_win_pnl > 0 else 0.0)
    net_pnl = sum(t["realized_net_pnl"] for t in trades)
    expectancy = (net_pnl / n) if n > 0 else 0.0
    be_hit_rate = (1.0 / (1.0 + payoff_ratio) * 100.0) if payoff_ratio > 0 else 50.0

    return {
        "n_trades": n,
        "n_wins": n_wins,
        "n_losses": n_losses,
        "hit_rate": round(hit_rate, 2),
        "avg_win": round(avg_win, 2),
        "avg_loss": round(avg_loss, 2),
        "payoff_ratio": round(payoff_ratio, 2),
        "profit_factor": round(profit_factor, 2),
        "net_pnl": round(net_pnl, 2),
        "expectancy": round(expectancy, 2),
        "be_hit_rate": round(be_hit_rate, 2),
    }


def main():
    trades = load_all_completed_trades()
    print("=" * 95)
    print("RP-615-CRYPTO: HIT RATE & PAYOFF RATIO DECISION DASHBOARD")
    print("Governance: RP-605 Control (100% Live NSE) | RP-615-Crypto (0% Authority - Shadow Only)")
    print("=" * 95)

    # 1. Live Daemon Stream Telemetry Status
    if TELEMETRY_JSON.exists():
        try:
            with TELEMETRY_JSON.open() as f:
                telem = json.load(f)
            ts = telem.get("timestamp_utc", "UNKNOWN")
            status = telem.get("status", "UNKNOWN")
            active_pos = telem.get("active_positions", {})
            cum_pnl = telem.get("cumulative_net_pnl", {})
            print(f"\n[Live Shadow Daemon Telemetry — Timestamp UTC: {ts}]")
            print(f"  Daemon Health: {status} | Execution Authority: 0% Live (Paper Shadow Replay)")
            print(f"  Active Open Positions:")
            for cand, pos_dict in active_pos.items():
                open_pairs = [sym for sym, tid in pos_dict.items() if tid is not None]
                pairs_str = ", ".join(open_pairs) if open_pairs else "None (Flat / Waiting for High Expectancy)"
                pnl_val = cum_pnl.get(cand, 0.0)
                print(f"    • {cand:<26}: {pairs_str:<45} | Live Realized Net: ${pnl_val:>7.2f}")
        except Exception as e:
            print(f"  Telemetry warning: {e}")

    # 2. Master Combined Performance Scorecard
    print("\n[Master Candidate Arms Scorecard — Combined Historical Replay + Live Prospective]")
    print(f"{'Candidate Arm':<26} | {'Trades':<6} | {'Hit Rate':<8} | {'Avg Win':<9} | {'Avg Loss':<9} | {'Payoff':<7} | {'PF':<5} | {'Exp/Trade':<10} | {'BE Hit Rate':<11}")
    print("-" * 110)

    for cand in CANDIDATES:
        c_trades = [t for t in trades if t["candidate_arm"] == cand]
        m = compute_metrics(c_trades)
        print(f"{cand:<26} | {m['n_trades']:<6} | {m['hit_rate']:>6.2f}% | ${m['avg_win']:>7.2f} | ${m['avg_loss']:>7.2f} | {m['payoff_ratio']:>5.2f}:1 | {m['profit_factor']:>4.2f} | ${m['expectancy']:>8.2f} | {m['be_hit_rate']:>9.2f}%")

    print("\n[Instrument Stratification — Lead Arm C5 Fixed-Capacity Top-2%]")
    print(f"{'Instrument':<12} | {'Trades':<6} | {'Hit Rate':<8} | {'Avg Win':<9} | {'Avg Loss':<9} | {'Payoff':<7} | {'PF':<5} | {'Net P&L':<10} | {'Exp/Trade':<10}")
    print("-" * 95)

    for sym in ["BTCUSDT", "ETHUSDT"]:
        s_trades = [t for t in trades if t["candidate_arm"] == "C5_Fixed_Capacity_Top2Pct" and t["symbol"] == sym]
        m = compute_metrics(s_trades)
        print(f"{sym:<12} | {m['n_trades']:<6} | {m['hit_rate']:>6.2f}% | ${m['avg_win']:>7.2f} | ${m['avg_loss']:>7.2f} | {m['payoff_ratio']:>5.2f}:1 | {m['profit_factor']:>4.2f} | ${m['net_pnl']:>8.2f} | ${m['expectancy']:>8.2f}")

    print("\n" + "=" * 95)
    print("Notice: To poll Binance for the latest 1m live bar, run:")
    print("  python3 scripts/run_rp615_crypto_shadow_daemon.py --single-run")
    print("Or to run the continuous real-time background daemon:")
    print("  python3 scripts/run_rp615_crypto_shadow_daemon.py --daemon --interval 60")
    print("=" * 95)


if __name__ == "__main__":
    main()
