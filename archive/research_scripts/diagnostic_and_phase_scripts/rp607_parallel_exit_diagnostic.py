#!/usr/bin/env python3
"""
rp607_parallel_exit_diagnostic.py

Parallel Diagnostic Auditor for RP-607 Engine Exit Forensics.
Runs in non-intrusive monitoring mode alongside active RP-605 prospective validation daemons.

Evaluates:
- Realized PnL vs Peak MFE PnL achieved during holding
- Profit Conversion Failure Rate (MFE > 0 but Realized < 0)
- Protection Exit Rate (MFE <= 0 where exit prevented worse drawdown)
- Diagnostic breakdown across Ledger A (Baseline) vs Ledger B (Freshness)
"""

import json
from pathlib import Path
from datetime import datetime, timezone

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

def read_jsonl(path):
    if not path.exists():
        return []
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]

def analyze_session(journal_path, cards_path, session_name, curr_symbol="$"):
    completed_trades = read_jsonl(journal_path)
    cards = read_jsonl(cards_path)

    if not completed_trades or not cards:
        print(f"[{session_name}] No completed trades or decision cards recorded yet.")
        return

    # Map prices per timestamp per instrument
    price_tape = {}
    for c in cards:
        sym = c.get("instrument")
        ts = c.get("timestamp_utc")
        p = c.get("current_price")
        if sym and ts and p:
            if sym not in price_tape:
                price_tape[sym] = []
            price_tape[sym].append((ts, float(p)))

    profit_conversion_failures = []
    successful_protection_exits = []
    other_exits = []

    for t in completed_trades:
        sym = t.get("symbol")
        side = t.get("side")
        entry_p = float(t.get("entry_price", 0))
        exit_p = float(t.get("exit_price", 0))
        entry_ts = t.get("entry_timestamp")
        exit_ts = t.get("exit_timestamp")
        realized_pnl = float(t.get("pnl", t.get("counterfactual_pnl", 0)))
        qty = float(t.get("quantity", 1.0))
        reason = t.get("exit_reason", "UNKNOWN")

        # Find max favorable price (MFE) during holding window
        sym_prices = price_tape.get(sym, [])
        holding_prices = [p for ts, p in sym_prices if entry_ts <= ts <= exit_ts] if exit_ts != "ACTIVE" else [p for ts, p in sym_prices if entry_ts <= ts]
        
        if holding_prices:
            if side == "LONG":
                peak_p = max(holding_prices)
                mfe_pnl = (peak_p - entry_p) * qty
            else:
                peak_p = min(holding_prices)
                mfe_pnl = (entry_p - peak_p) * qty
        else:
            peak_p = entry_p
            mfe_pnl = 0.0

        mfe_giveback = mfe_pnl - realized_pnl if mfe_pnl > 0 else 0.0

        rec = {
            **t,
            "peak_mfe_pnl": round(mfe_pnl, 4),
            "realized_pnl": round(realized_pnl, 4),
            "mfe_giveback": round(mfe_giveback, 4),
            "conversion_efficiency": round(realized_pnl / mfe_pnl, 4) if mfe_pnl > 0 else 0.0
        }

        if mfe_pnl > 0 and realized_pnl < 0:
            profit_conversion_failures.append(rec)
        elif mfe_pnl <= 0 and realized_pnl < 0:
            successful_protection_exits.append(rec) # PROTECTED_BY_EXIT
        else:
            other_exits.append(rec)

    total_n = len(completed_trades)
    n_pcf = len(profit_conversion_failures)
    n_prot = len(successful_protection_exits)
    n_other = len(other_exits)

    pcf_pct = (n_pcf / total_n * 100) if total_n > 0 else 0.0
    prot_pct = (n_prot / total_n * 100) if total_n > 0 else 0.0
    other_pct = (n_other / total_n * 100) if total_n > 0 else 0.0

    avg_giveback = (sum(f["mfe_giveback"] for f in profit_conversion_failures) / n_pcf) if n_pcf > 0 else 0.0

    print(f"\n=================================================================")
    print(f"🔍 RP-607 PARALLEL EXIT DIAGNOSTIC REPORT: {session_name}")
    print(f"=================================================================")
    print(f"Total Completed Trades: {total_n}")
    print(f"  ├── PROFIT_CONVERSION_FAILURE (MFE > 0 but Exited Negative): {n_pcf} ({pcf_pct:.1f}%) | Avg Giveback: {curr_symbol}{avg_giveback:.2f}")
    print(f"  ├── PROTECTED_BY_EXIT (MFE <= 0, Adverse Entry Exited Negative): {n_prot} ({prot_pct:.1f}%)")
    print(f"  └── OTHER_UNRESOLVED (Profitable & Equilibrium Exits): {n_other} ({other_pct:.1f}%)")
    print(f"-----------------------------------------------------------------")
    
    if profit_conversion_failures:
        print("⚠️ TOP PROFIT CONVERSION FAILURES (MFE Giveback Analysis):")
        for f in profit_conversion_failures[:5]:
            print(f"  • {f['symbol']} ({f['side']}) | Peak MFE: +{curr_symbol}{f['peak_mfe_pnl']:.2f} -> Realized: -{curr_symbol}{abs(f['realized_pnl']):.2f} | MFE Giveback: {curr_symbol}{f['mfe_giveback']:.2f} | Reason: {f['exit_reason']}")

def main():
    analyze_session(
        LOG_DIR / "rp605_nse_baseline_journal.jsonl",
        LOG_DIR / "rp605_nse_decision_cards_log.jsonl",
        "NSE Indian Equities (Ledger A Baseline)",
        curr_symbol="₹"
    )
    analyze_session(
        LOG_DIR / "rp605_nse_freshness_journal.jsonl",
        LOG_DIR / "rp605_nse_decision_cards_log.jsonl",
        "NSE Indian Equities (Ledger B Freshness)",
        curr_symbol="₹"
    )
    analyze_session(
        LOG_DIR / "rp605_baseline_journal.jsonl",
        LOG_DIR / "rp605_decision_cards_log.jsonl",
        "24/7 Crypto (Ledger A Baseline)",
        curr_symbol="$"
    )

if __name__ == "__main__":
    main()
