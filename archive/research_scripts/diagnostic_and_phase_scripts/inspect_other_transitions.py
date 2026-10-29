#!/usr/bin/env python3
import json

with open("live_capture/shadow_validation/rp608_trade_level_paired_audit_and_reproducibility_report.json") as f:
    data = json.load(f)

comps = data["matched_comparisons"]
print("Total matched trades:", len(comps))

loss_to_loss = [c for c in comps if c.get("outcome_transition") == "LOSS_TO_LOSS"]
print(f"\n--- LOSS TO LOSS TRADES ({len(loss_to_loss)}) ---")
for t in loss_to_loss:
    print(f"{t['symbol']} | Ctrl: {t['currency']}{t['control_rp605_pnl']} -> Shadow: {t['currency']}{t['shadow_rp608_pnl']} | Delta: {t['currency']}{t['pnl_delta']} | Exits: {t['control_exit_reason']} -> {t['shadow_exit_reason']} | Updates: v{t['boundary_version_final']}")

win_to_win = [c for c in comps if c.get("outcome_transition") == "WIN_TO_WIN"]
print(f"\n--- WIN TO WIN TRADES SUMMARY ({len(win_to_win)}) ---")
deltas = [w['pnl_delta'] for w in win_to_win]
print(f"Positive deltas: {len([d for d in deltas if d > 0])}, Zero deltas: {len([d for d in deltas if d == 0])}, Negative deltas: {len([d for d in deltas if d < 0])}")

# Print negative deltas in WIN_TO_WIN if any
neg_win_to_win = [w for w in win_to_win if w['pnl_delta'] < 0]
for t in neg_win_to_win:
    print(f"WIN_TO_WIN Negative Delta: {t['symbol']} | Ctrl: {t['currency']}{t['control_rp605_pnl']} -> Shadow: {t['currency']}{t['shadow_rp608_pnl']} | Delta: {t['currency']}{t['pnl_delta']} | Exits: {t['control_exit_reason']} -> {t['shadow_exit_reason']}")

