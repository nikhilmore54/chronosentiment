#!/usr/bin/env python3
import json
import numpy as np

with open("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json") as f:
    data = json.load(f)

records = data.get("rp607_records", [])
complete = [r for r in records if r.get("horizon_complete")]
complete_sorted = sorted(complete, key=lambda x: x.get("t_condition") or x.get("t_exit"))

split_idx = len(complete_sorted) // 2
disc = complete_sorted[:split_idx]
val = complete_sorted[split_idx:]

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

def eval_cohort(cohort, name):
    print(f"\n--- {name} (N={len(cohort)}) ---")
    y_true = [1 if r.get("lo_60", 0) > 0 else 0 for r in cohort]
    
    features = [
        ("F1_retained_mfe_ratio", lambda r: max(0.0, min(1.0, float(r.get("realized_pnl", 0.0)) / float(r.get("mfe_pnl", 1.0)))) if float(r.get("mfe_pnl", 0.0)) > 0 else 0.0, "less"),
        ("F2_exit_momentum_score", lambda r: (float(r.get("exit_scores", {}).get("15m", 0.0)) + float(r.get("exit_scores", {}).get("30m", 0.0))) / 2.0, "greater"),
        ("F3_holding_duration_mins", lambda r: float(r.get("holding_duration_mins", 0.0)), "greater"),
        ("F4_post_retreat_delay_mins", lambda r: float(r.get("post_retreat_delay_mins") or 0.0), "greater")
    ]
    
    for f_name, extractor, hypo_dir in features:
        raw_vals = [extractor(r) for r in cohort]
        raw_auc = calc_auc(y_true, raw_vals)
        
        hypo_vals = [-v if hypo_dir == "less" else v for v in raw_vals]
        hypo_auc = calc_auc(y_true, hypo_vals)
        
        print(f"{f_name:<28} | Hypo Dir: {hypo_dir:<7} | Raw AUC: {raw_auc:.4f} | Hypo AUC: {hypo_auc:.4f}")

eval_cohort(disc, "Stage 1 Discovery Cohort")
eval_cohort(val, "Stage 2 Temporal Validation Cohort")
