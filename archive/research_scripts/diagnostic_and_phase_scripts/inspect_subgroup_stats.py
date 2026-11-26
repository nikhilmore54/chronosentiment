#!/usr/bin/env python3
import json
import numpy as np

with open("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json") as f:
    data = json.load(f)

records = data.get("rp607_records", [])
complete = [r for r in records if r.get("horizon_complete")]

ss_records = [r for r in complete if "04:50:50" not in r.get("t_entry", "")]
su_records = [r for r in complete if "04:50:50" in r.get("t_entry", "")]

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

def eval_subgroup(subgroup, name):
    print(f"\n--- {name} (N={len(subgroup)}) ---")
    y_true = [1 if r.get("lo_60", 0) > 0 else 0 for r in subgroup]
    
    # F1 Retained MFE Ratio <= 0.30
    vals = []
    for r in subgroup:
        mfe_pnl = float(r.get("mfe_pnl", 0.0))
        realized_pnl = float(r.get("realized_pnl", 0.0))
        v = max(0.0, min(1.0, realized_pnl / mfe_pnl)) if mfe_pnl > 0 else 0.0
        vals.append(v)
        
    y_scores_hypo = [-v for v in vals]
    y_scores_raw = [v for v in vals]
    
    raw_auc = calc_auc(y_true, y_scores_raw)
    hypo_auc = calc_auc(y_true, y_scores_hypo)
    
    group_a = [r for r in subgroup if (max(0.0, min(1.0, float(r.get("realized_pnl", 0.0)) / float(r.get("mfe_pnl", 1.0)))) if float(r.get("mfe_pnl", 0.0)) > 0 else 0.0) <= 0.30]
    group_b = [r for r in subgroup if (max(0.0, min(1.0, float(r.get("realized_pnl", 0.0)) / float(r.get("mfe_pnl", 1.0)))) if float(r.get("mfe_pnl", 0.0)) > 0 else 0.0) > 0.30]
    
    mean_lo60_a = np.mean([r["lo_60"] for r in group_a]) if group_a else 0.0
    mean_lo60_b = np.mean([r["lo_60"] for r in group_b]) if group_b else 0.0
    delta = mean_lo60_a - mean_lo60_b
    
    print(f"Count Group A (F1 <= 0.30): {len(group_a)} | Count Group B (F1 > 0.30): {len(group_b)}")
    print(f"Group A Mean LO60: {mean_lo60_a:.2f} | Group B Mean LO60: {mean_lo60_b:.2f} | Delta: {delta:.2f}")
    print(f"Raw AUC: {raw_auc:.4f} | Hypo AUC (Direction 'less'): {hypo_auc:.4f}")

eval_subgroup(ss_records, "Steady-State Subgroup (Complete N=824)")
eval_subgroup(su_records, "Startup Seeding Subgroup (Complete N=188)")
