#!/usr/bin/env python3
"""
test_rp614_gate1_reconciliation.py

Gate 1 Friction Reconciliation Test Harness.
Compares:
- Formula A: Turnover basis (3.5 bps per leg): (entry_px + exit_px) * qty * 0.00035
- Formula B: Entry notional basis (7.0 bps on entry): entry_px * qty * 0.00070
- Aggregation Path 1: Float-exact summation
- Aggregation Path 2: Integer paise accumulation sum(round(net_pnl * 100))

Validates against the full canonical RP-605 NSE journal (N = 7,264 trades).
"""

import json
from pathlib import Path
import numpy as np

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
JOURNAL_FILE = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "rp605_nse_baseline_journal.jsonl"

def run_gate1_reconciliation():
    print("=" * 80)
    print("🔬 RP-614 GATE 1: FRICTION RECONCILIATION TEST HARNESS")
    print("=" * 80)

    if not JOURNAL_FILE.exists():
        print(f"❌ Error: Journal file not found at {JOURNAL_FILE}")
        return

    with open(JOURNAL_FILE, "r") as f:
        trades = [json.loads(line) for line in f if line.strip()]

    total_n = len(trades)
    print(f"📖 Loaded {total_n:,} canonical trades from {JOURNAL_FILE.name}")

    tot_entry_notional = 0.0
    tot_exit_notional = 0.0
    tot_turnover = 0.0
    tot_gross_pnl = 0.0

    fric_a_float_sum = 0.0
    fric_b_float_sum = 0.0
    
    net_a_float_sum = 0.0
    net_b_float_sum = 0.0

    net_a_paise_sum = 0
    net_b_paise_sum = 0

    net_a_wins = 0
    net_b_wins = 0

    gross_wins = 0

    per_trade_diffs = []

    for t in trades:
        ep = float(t["entry_price"])
        xp = float(t["exit_price"])
        qty = float(t["quantity"])
        gp = float(t["pnl"])

        n_entry = ep * qty
        n_exit = xp * qty
        turnover = (ep + xp) * qty

        tot_entry_notional += n_entry
        tot_exit_notional += n_exit
        tot_turnover += turnover
        tot_gross_pnl += gp

        if gp > 0:
            gross_wins += 1

        # Formula A: Turnover basis (3.5 bps per leg)
        fric_a = turnover * 0.00035
        net_a = gp - fric_a
        fric_a_float_sum += fric_a
        net_a_float_sum += net_a
        net_a_paise_sum += round(net_a * 100)
        if net_a > 0:
            net_a_wins += 1

        # Formula B: Entry notional basis (7.0 bps fixed on entry)
        fric_b = n_entry * 0.00070
        net_b = gp - fric_b
        fric_b_float_sum += fric_b
        net_b_float_sum += net_b
        net_b_paise_sum += round(net_b * 100)
        if net_b > 0:
            net_b_wins += 1

        per_trade_diffs.append(abs(fric_a - fric_b))

    avg_entry_notional = tot_entry_notional / total_n
    avg_turnover = tot_turnover / total_n

    print("\n--- POPULATION NOTIONAL TOTALS ---")
    print(f"Total Entry Notional:  ₹{tot_entry_notional:,.2f} (Avg: ₹{avg_entry_notional:,.2f})")
    print(f"Total Exit Notional:   ₹{tot_exit_notional:,.2f}")
    print(f"Total Two-Way Turnover:₹{tot_turnover:,.2f} (Avg: ₹{avg_turnover:,.2f})")
    print(f"Total Gross P&L:       ₹{tot_gross_pnl:+,.2f} ({gross_wins:,} Gross Wins, {gross_wins/total_n*100:.2f}%)")

    print("\n--- FORMULA A vs FORMULA B COMPARISON ---")
    print(f"Formula A (Turnover 3.5 bps/leg):")
    print(f"  • Total Friction:        ₹{fric_a_float_sum:,.2f} (Avg: ₹{fric_a_float_sum/total_n:.4f}/trade)")
    print(f"  • Net Realized P&L (Path 1 Float):  ₹{net_a_float_sum:+,.2f}")
    print(f"  • Net Realized P&L (Path 2 Paise):  ₹{net_a_paise_sum/100.0:+,.2f} ({net_a_paise_sum:,} paise)")
    print(f"  • Aggregation Variance (Path 1 vs 2): ₹{abs(net_a_float_sum - net_a_paise_sum/100.0):.4f} ({abs(round(net_a_float_sum*100) - net_a_paise_sum)} paise)")
    print(f"  • Net Winners:           {net_a_wins:,} ({net_a_wins/total_n*100:.2f}%)")

    print(f"\nFormula B (Entry Notional 7.0 bps):")
    print(f"  • Total Friction:        ₹{fric_b_float_sum:,.2f} (Avg: ₹{fric_b_float_sum/total_n:.4f}/trade)")
    print(f"  • Net Realized P&L (Path 1 Float):  ₹{net_b_float_sum:+,.2f}")
    print(f"  • Net Realized P&L (Path 2 Paise):  ₹{net_b_paise_sum/100.0:+,.2f} ({net_b_paise_sum:,} paise)")
    print(f"  • Aggregation Variance (Path 1 vs 2): ₹{abs(net_b_float_sum - net_b_paise_sum/100.0):.4f} ({abs(round(net_b_float_sum*100) - net_b_paise_sum)} paise)")
    print(f"  • Net Winners:           {net_b_wins:,} ({net_b_wins/total_n*100:.2f}%)")

    fric_diff_total = abs(fric_a_float_sum - fric_b_float_sum)
    max_trade_diff = max(per_trade_diffs)
    mean_trade_diff = np.mean(per_trade_diffs)

    print("\n--- FORMULA CONVERGENCE EVIDENCE ---")
    print(f"Total Friction Delta (A vs B):  ₹{fric_diff_total:,.2f} ({fric_diff_total / fric_a_float_sum * 100:.4f}% relative diff)")
    print(f"Max Per-Trade Friction Delta:   ₹{max_trade_diff:.4f}")
    print(f"Mean Per-Trade Friction Delta:  ₹{mean_trade_diff:.4f}")
    print(f"Net Win Rate Delta:             {abs(net_a_wins - net_b_wins)} trades ({abs(net_a_wins - net_b_wins)/total_n*100:.3f}%)")

    print("\n" + "=" * 80)
    print("✅ GATE 1 RECONCILIATION TEST COMPLETED")
    print("=" * 80)

if __name__ == "__main__":
    run_gate1_reconciliation()
