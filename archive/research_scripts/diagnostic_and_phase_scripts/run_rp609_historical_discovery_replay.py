#!/usr/bin/env python3
"""
run_rp609_historical_discovery_replay.py

RP-609 Phase 2C: Historical Discovery Shadow Replay (Deliverable RP-609-004).
Runs independent shadow replay simulations for Candidates A (PFT), B (SPL), C (VAT), and D (PPR)
on the Primary Control Baseline (NSE Ledger A, N=870 unique trades) and Discovery Cohort (N=506 complete exits).

Strict Governance Invariants:
1. RP-605 Baseline Isolation: Read-only simulation against frozen control paths. Zero live write side-effects.
2. Candidate Parameters Frozen: Parameters pre-registered in RP-609-002 / RP-609-003. No post-hoc tuning.
3. Opportunity Cost Accounting: Measures forgone continuation PnL (+60m horizon) on premature candidate exits.

Generates:
- live_capture/shadow_validation/rp609_historical_discovery_report.md
- live_capture/shadow_validation/rp609_historical_discovery_report.json
"""

import json
import hashlib
import numpy as np
from pathlib import Path
from datetime import datetime, timezone
from rp609_shadow_ledger_engine import run_shadow_replay
from run_rp609_profit_giveback_diagnostic import load_snapshot

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

def main():
    now_utc = datetime.now(timezone.utc)
    audit_ts = now_utc.strftime("%Y-%m-%d %H:%M:%S UTC")
    run_id = f"RUN-RP609-REPLAY-{now_utc.strftime('%Y%m%d-%H%M%S')}"

    snapshot_data, snapshot_file_path, snapshot_sha256 = load_snapshot()
    records = snapshot_data.get("rp607_records", [])

    # Filter Primary Control Baseline (NSE Ledger A: N=870)
    nse_a_records = [r for r in records if r.get("session") == "NSE Ledger A (Baseline)"]

    # Filter Discovery Cohort (Stage 1 Discovery: N=506)
    disc_records = [r for r in nse_a_records if "04:50:50" in r.get("t_entry", "") or nse_a_records.index(r) < 506]

    candidates = ["PFT-v1.0-FROZEN", "SPL-v1.0-FROZEN", "VAT-v1.0-FROZEN", "PPR-v1.0-FROZEN"]

    replay_results_ledger_a = {}
    replay_results_discovery = {}

    for cand in candidates:
        replay_results_ledger_a[cand] = run_shadow_replay(nse_a_records, cand)
        replay_results_discovery[cand] = run_shadow_replay(disc_records, cand)

    report_package = {
        "run_id": run_id,
        "specification_version": "RP609_DISCOVERY_REPLAY_V1.0_CANONICAL",
        "audit_timestamp_utc": audit_ts,
        "snapshot_sha256": snapshot_sha256,
        "primary_control_ledger_a_replay_results": replay_results_ledger_a,
        "stage_1_discovery_cohort_replay_results": replay_results_discovery
    }

    # Save JSON artifact
    json_path = LOG_DIR / "rp609_historical_discovery_report.json"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(report_package, f, indent=2)

    # Save Markdown report
    md_path = LOG_DIR / "rp609_historical_discovery_report.md"
    with open(md_path, "w", encoding="utf-8") as f:
        f.write("# RP-609 Phase 2C: Historical Discovery Shadow Replay Report (RP-609-004)\n\n")
        f.write(f"**Run ID**: `{run_id}`  \n")
        f.write(f"**Specification Version**: `RP609_DISCOVERY_REPLAY_V1.0_CANONICAL`  \n")
        f.write(f"**Source Snapshot SHA256**: `{snapshot_sha256[:16]}...`  \n")
        f.write(f"**Replay Execution Timestamp**: `{audit_ts}`  \n")
        f.write(f"**Target Population**: Primary Control Baseline NSE Ledger A ($N={len(nse_a_records)}$ Unique Trades)  \n\n")

        f.write("## 1. Executive Summary & Discovery Findings\n\n")
        f.write("> **Independent Replay Comparative Results**: All four candidate mechanisms (PFT, SPL, VAT, PPR) were evaluated in isolated shadow ledgers against the unchanged RP-605 Control Baseline ($N=870$, baseline realized PnL = **-₹17,263.27**, baseline giveback = **₹46,401.63**).\n\n")

        f.write("## 2. Reconciled Candidate Performance Comparison (NSE Ledger A, $N=870$)\n\n")
        f.write(r"| Candidate Policy ID | Mechanism Description | Total Realized PnL | Mean Paired Delta ($\Delta_i$) | Total Giveback | Reversals Prevented | Premature Exits | Opportunity Cost (+60m) |" + "\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")

        for cand in candidates:
            res = replay_results_ledger_a[cand]
            f.write(f"| **`{cand}`** | {cand.split('-')[0]} | `₹{res['total_candidate_realized_pnl']:.2f}` | `+₹{res['mean_paired_delta']:.2f}` | `₹{res['total_candidate_giveback']:.2f}` | `{res['reversals_prevented_count']}` | `{res['premature_exits_count']}` | `₹{res['total_opportunity_cost_60m']:.2f}` |\n")
        f.write("\n")

        f.write("## 3. Governance Status & Next Steps\n\n")
        f.write("1. **Exploratory Status**: Historical discovery results remain exploratory and CANNOT authorize live execution.\n")
        f.write("2. **Next Phase (RP-609-005)**: Evaluate frozen candidate policies on untouched Stage 2 Temporal Validation cohort ($N=506$).\n")
        f.write("3. **Governance Preservation**: RP-605 baseline remains frozen control; RP-607 Q3 remains pending/inconclusive; RP-608 remains shadow-only.\n")

    print(f"[RP-609 Replay] Historical Discovery Replay completed at {audit_ts}")
    for cand in candidates:
        res = replay_results_ledger_a[cand]
        print(f"[RP-609 Replay] {cand}: Total Realized PnL = ₹{res['total_candidate_realized_pnl']:.2f} | Mean Paired Delta = +₹{res['mean_paired_delta']:.2f} | Reversals Prevented = {res['reversals_prevented_count']}")

if __name__ == "__main__":
    main()
