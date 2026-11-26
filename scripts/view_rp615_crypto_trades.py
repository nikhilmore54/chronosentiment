#!/usr/bin/env python3
"""
scripts/view_rp615_crypto_trades.py

Displays a formatted, human-readable table of all simulated crypto trades taken
across Track A (Primary BTC/ETH), Track B (Expansion SOL/BNB), and Historical Frozen Replay.
"""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
PRIMARY_JOURNAL = LOG_DIR / "rp615_crypto_shadow_trade_journal.jsonl"
EXPANSION_JOURNAL = LOG_DIR / "rp615_crypto_expansion_trade_journal.jsonl"
STAGE2_EXECUTION_LEDGER = LOG_DIR / "rp615_crypto_stage2_execution_ledger.csv"


def load_live_trades():
    live_trades = []
    for path in [PRIMARY_JOURNAL, EXPANSION_JOURNAL]:
        if path.exists():
            with path.open() as f:
                for line in f:
                    if line.strip():
                        try:
                            live_trades.append(json.loads(line))
                        except Exception:
                            pass
    return live_trades


def main():
    print("=" * 115)
    print("RP-615-CRYPTO: SIMULATED TRADE LEDGER & LIVE JOURNAL VIEWER")
    print("Governance: RP-605 Control (100% Live NSE) | RP-615-Crypto (0% Authority - Shadow Only)")
    print("=" * 115)

    live_trades = load_live_trades()
    print(f"\n[1. Live Prospective Shadow Journal — {len(live_trades)} Total Recorded Trade Actions]")
    print(f"{'Timestamp UTC':<20} | {'Cohort':<18} | {'Symbol':<8} | {'Arm':<24} | {'Side':<5} | {'Entry Price':<12} | {'Status':<15}")
    print("-" * 115)

    if not live_trades:
        print("  (No live trade events recorded yet)")
    else:
        for t in live_trades[-15:]:
            ts = t.get("timestamp_utc", "")[:19]
            cohort = t.get("cohort", "TRACK_A_PRIMARY")
            sym = t.get("symbol", "")
            arm = t.get("candidate_arm", "")
            side = t.get("side", "")
            entry = f"${float(t.get('entry_price', 0)):,.2f}"
            status = t.get("status", "")
            print(f"{ts:<20} | {cohort:<18} | {sym:<8} | {arm:<24} | {side:<5} | {entry:<12} | {status:<15}")

    print(f"\n[2. Historical Out-of-Sample Executions (Lead Arm C5 Fixed-Capacity Top-2%)]")
    print(f"{'Trade ID':<28} | {'Symbol':<8} | {'Side':<5} | {'Entry Price':<11} | {'Exit Price':<11} | {'Gross P&L':<10} | {'Friction':<9} | {'Net P&L':<10}")
    print("-" * 115)

    if STAGE2_EXECUTION_LEDGER.exists():
        with STAGE2_EXECUTION_LEDGER.open() as f:
            reader = csv.DictReader(f)
            c5_trades = [r for r in reader if r.get("candidate_arm") == "C5_Fixed_Capacity_Top2Pct"]
            for r in c5_trades[-10:]:
                tid = r.get("trade_id", "")
                sym = r.get("symbol", "")
                side = r.get("side", "")
                ep = f"${float(r.get('entry_price', 0)):,.2f}"
                xp = f"${float(r.get('exit_price', 0)):,.2f}"
                gp = f"${float(r.get('realized_gross_pnl', 0)):,.2f}"
                fr = f"${float(r.get('realized_friction', 18.0)):,.2f}"
                np_val = f"${float(r.get('realized_net_pnl', 0)):,.2f}"
                print(f"{tid:<28} | {sym:<8} | {side:<5} | {ep:<11} | {xp:<11} | {gp:<10} | {fr:<9} | {np_val:<10}")

    print("\n" + "=" * 115)
    print("Files you can open directly in IDE:")
    print(f"  • Primary Live Journal:   {PRIMARY_JOURNAL}")
    print(f"  • Expansion Live Journal: {EXPANSION_JOURNAL}")
    print(f"  • Stage 2 Replay Ledger:  {STAGE2_EXECUTION_LEDGER}")
    print("=" * 115)


if __name__ == "__main__":
    main()
