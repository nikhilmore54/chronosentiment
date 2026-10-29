#!/usr/bin/env python3
import json

with open("live_capture/shadow_validation/rp608_trade_level_paired_audit_and_reproducibility_report.json") as f:
    data = json.load(f)

comps = data["matched_comparisons"]
print("Total matched trades:", len(comps))
print("Transitions:", data.get("transitions_matrix"))

loss_to_win = [c for c in comps if c.get("outcome_transition") == "LOSS_TO_WIN"]
print(f"\n--- LOSS TO WIN TRADES ({len(loss_to_win)}) ---")
for t in loss_to_win:
    sym = t.get("symbol")
    curr = t.get("currency")
    ctrl_pnl = t.get("control_rp605_pnl")
    sh_pnl = t.get("shadow_rp608_pnl")
    delta = t.get("pnl_delta")
    ctrl_exit = t.get("control_exit_reason")
    sh_exit = t.get("shadow_exit_reason")
    applied = t.get("recommendation_breakdown", {}).get("applied")
    mfe = t.get("mfe_pnl")
    mae = t.get("mae_pnl")
    v_final = t.get("boundary_version_final")
    print(f"{sym:<12} | Ctrl: {curr}{ctrl_pnl:>8.2f} -> Shadow: {curr}{sh_pnl:>8.2f} | Delta: {curr}{delta:>8.2f} | Exits: {ctrl_exit} -> {sh_exit} | Updates: v{v_final} ({applied} applied) | MFE/MAE: {curr}{mfe}/{curr}{mae}")
