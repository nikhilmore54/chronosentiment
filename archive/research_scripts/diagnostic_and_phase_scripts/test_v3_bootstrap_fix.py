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

def eval_feature_fixed(cohort_rows, feature_key, threshold_val, direction="less", n_boot=1000):
    y_true = [1 if r.get("lo_60", 0) > 0 else 0 for r in cohort_rows]
    
    raw_vals = []
    for r in cohort_rows:
        mfe_pnl = float(r.get("mfe_pnl", 0.0))
        realized_pnl = float(r.get("realized_pnl", 0.0))
        dur_mins = float(r.get("holding_duration_mins", 0.0))
        post_retreat_delay = float(r.get("post_retreat_delay_mins") or 0.0)
        scores = r.get("exit_scores", {})
        avg_score = (float(scores.get("15m", 0.0)) + float(scores.get("30m", 0.0))) / 2.0
        
        if feature_key == "F1_retained_mfe_ratio":
            v = max(0.0, min(1.0, realized_pnl / mfe_pnl)) if mfe_pnl > 0 else 0.0
        elif feature_key == "F2_exit_momentum_score":
            v = avg_score
        elif feature_key == "F3_holding_duration_mins":
            v = dur_mins
        elif feature_key == "F4_post_retreat_delay_mins":
            v = post_retreat_delay
        raw_vals.append(v)
        
    hypo_scores = [-v if direction == "less" else v for v in raw_vals]
    
    raw_auc = calc_auc(y_true, raw_vals)
    hypo_auc = calc_auc(y_true, hypo_scores)
    
    # Bootstrap strictly using hypo_scores
    rng = np.random.RandomState(42)
    auc_boots = []
    n = len(y_true)
    for _ in range(n_boot):
        idxs = rng.choice(n, size=n, replace=True)
        yt_b = [y_true[i] for i in idxs]
        ys_b = [hypo_scores[i] for i in idxs]
        if len(set(yt_b)) > 1:
            auc_boots.append(calc_auc(yt_b, ys_b))
            
    ci_low = float(np.percentile(auc_boots, 2.5)) if auc_boots else hypo_auc
    ci_high = float(np.percentile(auc_boots, 97.5)) if auc_boots else hypo_auc
    
    return {
        "feature_key": feature_key,
        "direction": direction,
        "raw_auc": round(raw_auc, 4),
        "hypo_auc": round(hypo_auc, 4),
        "ci_95": [round(ci_low, 4), round(ci_high, 4)],
        "auc_in_ci": ci_low <= hypo_auc <= ci_high
    }

print("--- STAGE 1 DISCOVERY COHORT FIXED (N=506) ---")
features_config = [
    ("F1_retained_mfe_ratio", 0.30, "less"),
    ("F2_exit_momentum_score", 0.00, "greater"),
    ("F3_holding_duration_mins", 15.0, "greater"),
    ("F4_post_retreat_delay_mins", 5.0, "greater")
]

for fk, th, dr in features_config:
    res = eval_feature_fixed(disc, fk, th, dr)
    print(f"{fk:<28} | Raw AUC: {res['raw_auc']:.4f} | Hypo AUC: {res['hypo_auc']:.4f} | CI: {res['ci_95']} | In CI: {res['auc_in_ci']}")

print("\n--- STAGE 2 TEMPORAL VALIDATION COHORT FIXED (N=506) ---")
for fk, th, dr in features_config:
    res = eval_feature_fixed(val, fk, th, dr)
    print(f"{fk:<28} | Raw AUC: {res['raw_auc']:.4f} | Hypo AUC: {res['hypo_auc']:.4f} | CI: {res['ci_95']} | In CI: {res['auc_in_ci']}")
