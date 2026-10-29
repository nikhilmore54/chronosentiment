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

def get_auc_and_ci(cohort, key, direction="less", n_boot=1000):
    y_true = [1 if r.get("lo_60", 0) > 0 else 0 for r in cohort]
    
    vals = []
    for r in cohort:
        mfe_pnl = float(r.get("mfe_pnl", 0.0))
        realized_pnl = float(r.get("realized_pnl", 0.0))
        if key == "F1":
            v = max(0.0, min(1.0, realized_pnl / mfe_pnl)) if mfe_pnl > 0 else 0.0
        elif key == "F2":
            scores = r.get("exit_scores", {})
            v = (float(scores.get("15m", 0.0)) + float(scores.get("30m", 0.0))) / 2.0
        elif key == "F3":
            v = float(r.get("holding_duration_mins", 0.0))
        elif key == "F4":
            v = float(r.get("post_retreat_delay_mins") or 0.0)
        vals.append(v)
        
    y_scores = [-v if direction == "less" else v for v in vals]
    
    # Compute base AUC
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
        return float(np.sum((fpr[1:] - fpr[:-1]) * (tpr[1:] + tpr[:-1]) / 2.0))

    base_auc = calc_auc(y_true, y_scores)
    
    # Bootstrap 1000 iterations
    rng = np.random.RandomState(42)
    auc_boot = []
    n = len(y_true)
    for _ in range(n_boot):
        idxs = rng.choice(n, size=n, replace=True)
        yt_b = [y_true[i] for i in idxs]
        ys_b = [y_scores[i] for i in idxs]
        if len(set(yt_b)) > 1:
            auc_boot.append(calc_auc(yt_b, ys_b))
            
    ci_low = float(np.percentile(auc_boot, 2.5))
    ci_high = float(np.percentile(auc_boot, 97.5))
    return round(base_auc, 4), round(ci_low, 4), round(ci_high, 4)

print("--- STAGE 1 DISCOVERY COHORT (N=506) ---")
for f_key in ["F1", "F2", "F3", "F4"]:
    auc, low, high = get_auc_and_ci(disc, f_key)
    print(f"{f_key}: AUC = {auc:.4f} | 95% CI = [{low:.4f}, {high:.4f}]")

print("\n--- STAGE 2 TEMPORAL VALIDATION COHORT (N=506) ---")
for f_key in ["F1", "F2", "F3", "F4"]:
    auc, low, high = get_auc_and_ci(val, f_key)
    print(f"{f_key}: AUC = {auc:.4f} | 95% CI = [{low:.4f}, {high:.4f}]")
