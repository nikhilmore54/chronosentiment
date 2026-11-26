#!/usr/bin/env python3
import json
import numpy as np
from collections import defaultdict

with open("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json") as f:
    data = json.load(f)

records = data.get("rp607_records", [])
complete = [r for r in records if r.get("horizon_complete")]

# Group complete exits by symbol
sym_groups = defaultdict(list)
for r in complete:
    sym_groups[r.get("symbol")].append(r)

def calc_auc(yt, ys):
    desc = np.argsort(ys, kind='mergesort')[::-1]
    yt_s, ys_s = np.array(yt)[desc], np.array(ys)[desc]
    threshold_idxs = np.r_[np.where(np.diff(ys_s))[0], yt_s.size - 1]
    tps = np.r_[0, np.cumsum(yt_s)[threshold_idxs]]
    fps = np.r_[0, 1 + threshold_idxs - tps[1:]]
    if fps[-1] == 0 or tps[-1] == 0:
        return 0.5
    fpr = fps / fps[-1]
    tpr = tps / tps[-1]
    if hasattr(np, 'trapezoid'):
        return float(np.trapezoid(tpr, fpr))
    elif hasattr(np, 'trapz'):
        return float(np.trapz(tpr, fpr))
    else:
        return float(np.sum((fpr[1:] - fpr[:-1]) * (tpr[1:] + tpr[:-1]) / 2.0))

symbol_f1_aucs = []
print(f"Total symbols in dataset: {len(sym_groups)}")
print(f"Symbols with >= 5 complete exits: {len([s for s, rows in sym_groups.items() if len(rows) >= 5])}")

for sym, rows in sorted(sym_groups.items(), key=lambda x: len(x[1]), reverse=True):
    if len(rows) < 5:
        continue
    y_true = [1 if r.get("lo_60", 0) > 0 else 0 for r in rows]
    if len(set(y_true)) < 2:
        # Single class symbol
        continue
    f1_vals = [max(0.0, min(1.0, float(r.get("realized_pnl", 0.0)) / float(r.get("mfe_pnl", 1.0)))) if float(r.get("mfe_pnl", 0.0)) > 0 else 0.0 for r in rows]
    hypo_scores = [-v for v in f1_vals]
    auc = calc_auc(y_true, hypo_scores)
    symbol_f1_aucs.append((sym, len(rows), auc))

print("\n--- TOP & BOTTOM SYMBOL F1 HYPO AUCs (min 5 exits) ---")
for sym, count, auc in symbol_f1_aucs:
    print(f"{sym:<15} | Exits: {count:>3} | F1 Hypo AUC: {auc:.4f}")

auc_list = [a for s, c, a in symbol_f1_aucs]
if auc_list:
    print(f"\nSymbol F1 AUC Stats (N={len(auc_list)} symbols):")
    print(f"  Mean AUC: {np.mean(auc_list):.4f}")
    print(f"  Median AUC: {np.median(auc_list):.4f}")
    print(f"  Min AUC: {np.min(auc_list):.4f}")
    print(f"  Max AUC: {np.max(auc_list):.4f}")
    print(f"  IQR: {np.percentile(auc_list, 75) - np.percentile(auc_list, 25):.4f}")
    print(f"  % Symbols with AUC >= 0.60: {sum(1 for a in auc_list if a >= 0.60)/len(auc_list)*100:.1f}%")
