import json
import numpy as np
from pathlib import Path

path = Path("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json")
with open(path) as f:
    data = json.load(f)

complete = [r for r in data["rp607_records"] if r.get("horizon_complete")]

matrix = []
for r in complete:
    lo_60 = float(r.get("lo_60"))
    mfe_pnl = float(r.get("mfe_pnl", 0.0))
    realized_pnl = float(r.get("realized_pnl", 0.0))
    retained_mfe_ratio = max(0.0, min(1.0, realized_pnl / mfe_pnl)) if mfe_pnl > 0 else 0.0
    matrix.append({
        "symbol": r["symbol"],
        "t_condition": r.get("t_condition") or r.get("t_exit"),
        "t_entry": r.get("t_entry"),
        "F1_retained_mfe_ratio": round(retained_mfe_ratio, 4),
        "lo_60": lo_60,
        "binary_continuation": 1 if lo_60 > 0 else 0
    })

matrix.sort(key=lambda x: x["t_condition"])
split_idx = len(matrix) // 2
val_cohort = matrix[split_idx:]

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

def eval_f1(rows):
    yt = [r["binary_continuation"] for r in rows]
    ys = [-r["F1_retained_mfe_ratio"] for r in rows]
    auc = calc_auc(yt, ys)
    # Stratified bootstrap
    pos_idxs = [i for i, y in enumerate(yt) if y == 1]
    neg_idxs = [i for i, y in enumerate(yt) if y == 0]
    rng = np.random.RandomState(42)
    boots = []
    for _ in range(500):
        pos_b = rng.choice(pos_idxs, size=len(pos_idxs), replace=True)
        neg_b = rng.choice(neg_idxs, size=len(neg_idxs), replace=True)
        idxs = np.concatenate([pos_b, neg_b])
        yt_b = [yt[i] for i in idxs]
        ys_b = [ys[i] for i in idxs]
        boots.append(calc_auc(yt_b, ys_b))
    ci_low = float(np.percentile(boots, 2.5))
    ci_high = float(np.percentile(boots, 97.5))
    return round(auc, 4), [round(ci_low, 4), round(ci_high, 4)]

counts_full = {}
for r in matrix:
    counts_full[r["symbol"]] = counts_full.get(r["symbol"], 0) + 1

sorted_full = sorted(counts_full.items(), key=lambda x: x[1], reverse=True)
top3 = [t[0] for t in sorted_full[:3]]
top5 = [t[0] for t in sorted_full[:5]]

val_ex_top3 = [r for r in val_cohort if r["symbol"] not in top3]
val_ex_top5 = [r for r in val_cohort if r["symbol"] not in top5]

full_ex_top3 = [r for r in matrix if r["symbol"] not in top3]
full_ex_top5 = [r for r in matrix if r["symbol"] not in top5]

auc_full_val, ci_full_val = eval_f1(val_cohort)
auc_top3_val, ci_top3_val = eval_f1(val_ex_top3)
auc_top5_val, ci_top5_val = eval_f1(val_ex_top5)

print(f"Full Stage 2 Val (N=506): AUC={auc_full_val}, 95% CI={ci_full_val}")
print(f"Top 3 Excluded ({top3}): Full Rem=43, Val Rem=19, Val Rem Count={len(val_ex_top3)} -> AUC={auc_top3_val}, 95% CI={ci_top3_val}")
print(f"Top 5 Excluded ({top5}): Full Rem=71, Val Rem=37, Val Rem Count={len(val_ex_top5)} -> AUC={auc_top5_val}, 95% CI={ci_top5_val}")
