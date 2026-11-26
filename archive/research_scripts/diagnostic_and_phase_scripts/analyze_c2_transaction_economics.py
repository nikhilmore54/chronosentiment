#!/usr/bin/env python3
"""
scripts/analyze_c2_transaction_economics.py

RP-615: Dedicated C2 Transaction Economics & Friction Sensitivity Diagnostic Engine.

Objectives:
  1. Maintain 100% frozen model sanctity (Zero tuning, zero threshold modification).
  2. Granular Trade-Level Forensic Ledger for candidate arm C2_Dynamic_BreakEven.
  3. Gross-vs-Net Survival Curve & Edge Concentration Test (Scenario A: Broad vs Scenario B: Concentrated).
  4. Counterfactual Friction Sensitivity Curve (0 to 20 bps) - Explicitly non-qualifying diagnostic.
  5. Multi-dimensional stratification (by symbol, session date, trade direction, holding time).
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
OUT_REPORT_JSON = WORKSPACE / "audit" / "rp615_reconciliation" / "c2_transaction_economics_report.json"
OUT_REPORT_MD = WORKSPACE / "audit" / "rp615_reconciliation" / "c2_transaction_economics_report.md"


def load_c2_trades() -> List[Dict[str, Any]]:
    trades = []
    for j_path, cohort_name in [(PRIMARY_JOURNAL, "TRACK_A_PRIMARY"), (EXPANSION_JOURNAL, "TRACK_B_EXPANSION")]:
        if not j_path.exists():
            continue
        with j_path.open(encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    item = json.loads(line)
                    if item.get("candidate_arm") == "C2_Dynamic_BreakEven" and item.get("status") in ("SIMULATED_CLOSE", "SIMULATED_CLOSED"):
                        gross = float(item.get("gross_pnl", item.get("realized_gross_pnl", 0.0)))
                        fric = float(item.get("friction_dollars", 18.0))
                        net = float(item.get("net_pnl", item.get("realized_net_pnl", gross - fric)))
                        entry_p = float(item["entry_price"])
                        exit_p = float(item.get("exit_price", entry_p))
                        entry_ts = item.get("entry_time", item.get("entry_timestamp_utc", ""))
                        exit_ts = item.get("exit_time", item.get("exit_timestamp_utc", ""))
                        
                        # Return in basis points relative to assumed $10,000 position size (where $18 = 18 bps)
                        # Assumed trade notional = $10,000.00
                        trade_notional = 10000.0
                        gross_bps = (gross / trade_notional) * 10000.0
                        
                        trades.append({
                            "cohort": cohort_name,
                            "trade_id": item["trade_id"],
                            "symbol": item["symbol"],
                            "side": item["side"],
                            "entry_price": entry_p,
                            "exit_price": exit_p,
                            "gross_pnl": gross,
                            "friction_18bps": fric,
                            "net_pnl_18bps": net,
                            "gross_return_bps": round(gross_bps, 2),
                            "gross_to_friction_ratio": round(gross / fric, 2) if fric > 0 else 0.0,
                            "holding_seconds": float(item.get("holding_seconds", 3600.0)),
                            "entry_timestamp": entry_ts,
                            "exit_timestamp": exit_ts,
                        })
                except Exception:
                    pass
    return trades


def run_friction_sensitivity(trades: List[Dict[str, Any]], notional: float = 10000.0) -> List[Dict[str, Any]]:
    bps_grid = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
    n_trades = len(trades)
    tot_gross = sum(t["gross_pnl"] for t in trades)
    
    results = []
    for bps in bps_grid:
        fee_per_trade = (bps / 10000.0) * notional
        tot_fee = fee_per_trade * n_trades
        tot_net = tot_gross - tot_fee
        exp_per_trade = tot_net / n_trades if n_trades > 0 else 0.0
        n_winners = sum(1 for t in trades if (t["gross_pnl"] - fee_per_trade) > 0)
        win_rate = (n_winners / n_trades * 100.0) if n_trades > 0 else 0.0
        
        gains = sum(t["gross_pnl"] - fee_per_trade for t in trades if (t["gross_pnl"] - fee_per_trade) > 0)
        losses = abs(sum(t["gross_pnl"] - fee_per_trade for t in trades if (t["gross_pnl"] - fee_per_trade) < 0))
        pf = (gains / losses) if losses > 0 else (99.0 if gains > 0 else 0.0)
        
        results.append({
            "friction_bps": bps,
            "fee_per_trade_usd": round(fee_per_trade, 2),
            "total_friction_usd": round(tot_fee, 2),
            "total_gross_usd": round(tot_gross, 2),
            "total_net_usd": round(tot_net, 2),
            "net_exp_per_trade_usd": round(exp_per_trade, 2),
            "n_winning_trades": n_winners,
            "win_rate_pct": round(win_rate, 2),
            "profit_factor": round(pf, 2)
        })
    return results


def analyze_alpha_distribution(trades: List[Dict[str, Any]]) -> Dict[str, Any]:
    if not trades:
        return {}
    
    n = len(trades)
    sorted_trades = sorted(trades, key=lambda x: x["gross_pnl"], reverse=True)
    gross_pnls = [t["gross_pnl"] for t in sorted_trades]
    tot_gross = sum(gross_pnls)
    
    # Winners vs Losers on gross basis
    gross_winners = [p for p in gross_pnls if p > 0]
    gross_losers = [p for p in gross_pnls if p < 0]
    gross_flats = [p for p in gross_pnls if p == 0]
    
    # Top contributions
    top_1_contrib = gross_pnls[0] if n >= 1 else 0.0
    top_3_contrib = sum(gross_pnls[:3]) if n >= 3 else sum(gross_pnls)
    top_10pct_n = max(1, math.ceil(n * 0.10))
    top_10pct_contrib = sum(gross_pnls[:top_10pct_n])
    top_20pct_n = max(1, math.ceil(n * 0.20))
    top_20pct_contrib = sum(gross_pnls[:top_20pct_n])
    
    # Survival thresholds
    cleared_0bps = sum(1 for t in trades if t["gross_pnl"] > 0)
    cleared_6bps = sum(1 for t in trades if t["gross_pnl"] > 6.0)
    cleared_12bps = sum(1 for t in trades if t["gross_pnl"] > 12.0)
    cleared_18bps = sum(1 for t in trades if t["gross_pnl"] > 18.0)
    
    return {
        "n_total_trades": n,
        "total_gross_pnl": round(tot_gross, 2),
        "mean_gross_pnl": round(tot_gross / n, 2),
        "median_gross_pnl": round(sorted_trades[n // 2]["gross_pnl"], 2),
        "gross_win_count": len(gross_winners),
        "gross_loss_count": len(gross_losers),
        "gross_flat_count": len(gross_flats),
        "gross_win_rate_pct": round(len(gross_winners) / n * 100.0, 2),
        "top_1_trade_gross": round(top_1_contrib, 2),
        "top_1_share_pct": round(top_1_contrib / tot_gross * 100.0, 2) if tot_gross > 0 else 0.0,
        "top_3_trades_gross": round(top_3_contrib, 2),
        "top_3_share_pct": round(top_3_contrib / tot_gross * 100.0, 2) if tot_gross > 0 else 0.0,
        "top_10pct_trades_gross": round(top_10pct_contrib, 2),
        "top_10pct_share_pct": round(top_10pct_contrib / tot_gross * 100.0, 2) if tot_gross > 0 else 0.0,
        "top_20pct_trades_gross": round(top_20pct_contrib, 2),
        "top_20pct_share_pct": round(top_20pct_contrib / tot_gross * 100.0, 2) if tot_gross > 0 else 0.0,
        "survival_hurdles": {
            "cleared_0bps_count": cleared_0bps,
            "cleared_0bps_pct": round(cleared_0bps / n * 100.0, 2),
            "cleared_6bps_count": cleared_6bps,
            "cleared_6bps_pct": round(cleared_6bps / n * 100.0, 2),
            "cleared_12bps_count": cleared_12bps,
            "cleared_12bps_pct": round(cleared_12bps / n * 100.0, 2),
            "cleared_18bps_count": cleared_18bps,
            "cleared_18bps_pct": round(cleared_18bps / n * 100.0, 2),
        }
    }


def stratify_by_dimension(trades: List[Dict[str, Any]], dim_key: str) -> Dict[str, Dict[str, Any]]:
    groups = defaultdict(list)
    for t in trades:
        val = t.get(dim_key, "UNKNOWN")
        groups[val].append(t)
        
    out = {}
    for k, grp in groups.items():
        n = len(grp)
        gross = sum(t["gross_pnl"] for t in grp)
        fric = sum(t["friction_18bps"] for t in grp)
        net = sum(t["net_pnl_18bps"] for t in grp)
        n_wins = sum(1 for t in grp if t["net_pnl_18bps"] > 0)
        n_gross_wins = sum(1 for t in grp if t["gross_pnl"] > 0)
        out[k] = {
            "n_trades": n,
            "gross_pnl": round(gross, 2),
            "friction_18bps": round(fric, 2),
            "net_pnl_18bps": round(net, 2),
            "gross_exp_per_trade": round(gross / n, 2),
            "net_exp_per_trade": round(net / n, 2),
            "gross_win_rate_pct": round(n_gross_wins / n * 100.0, 2),
            "net_win_rate_pct": round(n_wins / n * 100.0, 2),
        }
    return out


def main():
    trades = load_c2_trades()
    print("=" * 80)
    print("RP-615: C2 TRANSACTION ECONOMICS & FRICTION SENSITIVITY DIAGNOSTIC")
    print(f"Total Completed C2 Trades: {len(trades)}")
    print("=" * 80)
    
    if not trades:
        print("No completed C2 trades found.")
        return
        
    track_a = [t for t in trades if t["cohort"] == "TRACK_A_PRIMARY"]
    track_b = [t for t in trades if t["cohort"] == "TRACK_B_EXPANSION"]
    
    # Run analyses
    sensitivity_combined = run_friction_sensitivity(trades)
    sensitivity_track_a = run_friction_sensitivity(track_a)
    sensitivity_track_b = run_friction_sensitivity(track_b)
    
    dist_combined = analyze_alpha_distribution(trades)
    dist_track_a = analyze_alpha_distribution(track_a)
    dist_track_b = analyze_alpha_distribution(track_b)
    
    by_symbol = stratify_by_dimension(trades, "symbol")
    by_side = stratify_by_dimension(trades, "side")
    
    # Group by date
    for t in trades:
        ts = t.get("entry_timestamp", "")
        t["session_date"] = ts[:10] if len(ts) >= 10 else "UNKNOWN"
    by_date = stratify_by_dimension(trades, "session_date")
    
    report_data = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "n_total_c2_trades": len(trades),
        "n_track_a_trades": len(track_a),
        "n_track_b_trades": len(track_b),
        "aggregate_summary": {
            "combined_gross_usd": round(sum(t["gross_pnl"] for t in trades), 2),
            "combined_friction_18bps_usd": round(sum(t["friction_18bps"] for t in trades), 2),
            "combined_net_18bps_usd": round(sum(t["net_pnl_18bps"] for t in trades), 2),
            "gross_expectancy_usd": round(sum(t["gross_pnl"] for t in trades) / len(trades), 2),
            "net_expectancy_18bps_usd": round(sum(t["net_pnl_18bps"] for t in trades) / len(trades), 2),
        },
        "friction_sensitivity_matrix_combined": sensitivity_combined,
        "friction_sensitivity_matrix_track_a": sensitivity_track_a,
        "friction_sensitivity_matrix_track_b": sensitivity_track_b,
        "alpha_distribution_combined": dist_combined,
        "alpha_distribution_track_a": dist_track_a,
        "alpha_distribution_track_b": dist_track_b,
        "stratification": {
            "by_symbol": by_symbol,
            "by_side": by_side,
            "by_session_date": by_date,
        },
        "trade_level_ledger": trades
    }
    
    OUT_REPORT_JSON.parent.mkdir(parents=True, exist_ok=True)
    with OUT_REPORT_JSON.open("w", encoding="utf-8") as f:
        json.dump(report_data, f, indent=2)
        
    print(f"\n[OK] Transaction economics report written to: {OUT_REPORT_JSON}")


if __name__ == "__main__":
    main()
