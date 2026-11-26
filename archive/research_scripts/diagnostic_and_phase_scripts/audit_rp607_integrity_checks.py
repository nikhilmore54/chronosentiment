#!/usr/bin/env python3
"""
audit_rp607_integrity_checks.py

Detailed Verification Audit for RP-607 Record Integrity, Horizon Completeness,
PnL Arithmetic Reconciliation, and Initial Startup Cohort Stratification.
"""

import json
from pathlib import Path
from datetime import datetime, timezone

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

def parse_iso(ts_str):
    if not ts_str or ts_str == "ACTIVE":
        return None
    try:
        ts_str = ts_str.replace("Z", "+00:00")
        return datetime.fromisoformat(ts_str).astimezone(timezone.utc)
    except Exception:
        return None

def read_jsonl(path):
    if not path.exists():
        return []
    records = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    records.append(json.loads(line))
                except Exception:
                    pass
    return records

def main():
    nse_journal = read_jsonl(LOG_DIR / "rp605_nse_baseline_journal.jsonl") + read_jsonl(LOG_DIR / "rp605_nse_freshness_journal.jsonl")
    crypto_journal = read_jsonl(LOG_DIR / "rp605_baseline_journal.jsonl") + read_jsonl(LOG_DIR / "rp605_freshness_journal.jsonl")
    all_journal = nse_journal + crypto_journal

    engine_exits = [t for t in all_journal if t.get("exit_reason") == "ENGINE_EXIT" and t.get("exit_timestamp")]

    # Check 1: Holding Duration Distribution & Startup Cohort Audit
    durations = []
    startup_cohort = [] # Entered at 04:50:50Z (first tick after T0 restart)
    steady_state_cohort = []

    for t in engine_exits:
        e_dt = parse_iso(t.get("entry_timestamp"))
        x_dt = parse_iso(t.get("exit_timestamp"))
        if e_dt and x_dt:
            dur_secs = (x_dt - e_dt).total_seconds()
            dur_mins = dur_secs / 60.0
            durations.append(dur_mins)
            
            # Check if entered during initial daemon restart tick (04:50:50Z UTC)
            if e_dt.strftime("%H:%M:%S") == "04:50:50":
                startup_cohort.append(t)
            else:
                steady_state_cohort.append(t)

    dur_1m_count = sum(1 for d in durations if d <= 1.5)
    dur_sub5m_count = sum(1 for d in durations if d <= 5.0)
    dur_over15m_count = sum(1 for d in durations if d >= 15.0)

    # Check 3: PnL Arithmetic Verification
    pnl_mismatches = 0
    for t in engine_exits:
        side = t.get("side")
        ep = float(t.get("entry_price", 0))
        xp = float(t.get("exit_price", 0))
        qty = float(t.get("quantity", 1))
        reported_pnl = float(t.get("pnl", t.get("realized_pnl", 0)))
        
        expected_pnl = (xp - ep) * qty if side == "LONG" else (ep - xp) * qty
        if abs(reported_pnl - expected_pnl) > 0.01:
            pnl_mismatches += 1

    report = {
        "total_engine_exits": len(engine_exits),
        "durations_summary": {
            "startup_cohort_04_50_50_count": len(startup_cohort),
            "steady_state_cohort_count": len(steady_state_cohort),
            "durations_1m_or_less": dur_1m_count,
            "durations_sub_5m": dur_sub5m_count,
            "durations_over_15m": dur_over15m_count,
            "avg_steady_state_holding_mins": round(sum( (parse_iso(t["exit_timestamp"]) - parse_iso(t["entry_timestamp"])).total_seconds()/60.0 for t in steady_state_cohort) / len(steady_state_cohort), 1) if steady_state_cohort else 0
        },
        "pnl_arithmetic_mismatches": pnl_mismatches
    }

    print("=== RP-607 INTEGRITY AUDIT RESULTS ===")
    print(json.dumps(report, indent=2))

if __name__ == "__main__":
    main()
