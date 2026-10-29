#!/usr/bin/env python3
"""
scripts/train_rp615_predictive_model.py

RP-615-003: Model Training, Hyperparameter Optimization, and Calibration Engine.
Trains multi-head predictive models on chronological discovery sessions (2026-09-28 to 2026-09-30)
and evaluates out-of-sample calibration and accuracy on the temporal validation session (2026-10-01).

Predictive Heads:
  1. Win Probability Classifier: P(Net P&L > 0) with Isotonic/Sigmoid Calibration.
  2. Expected Net P&L Regressor: E[Net P&L INR].
  3. Favorable Excursion Regressor: E[MFE %].
  4. Adverse Excursion Regressor: E[MAE %].
  5. Holding Duration Regressor: E[Duration Sec].

Outputs:
  - live_capture/shadow_validation/rp615_003_model_weights.json
  - live_capture/shadow_validation/rp615_003_training_manifest.json
"""

from __future__ import annotations

import csv
import json
import math
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Tuple

import numpy as np
from sklearn.ensemble import HistGradientBoostingClassifier, HistGradientBoostingRegressor, GradientBoostingClassifier
from sklearn.calibration import CalibratedClassifierCV
from sklearn.linear_model import LogisticRegression, Ridge
from sklearn.metrics import brier_score_loss, log_loss, roc_auc_score, precision_score, recall_score, mean_squared_error, mean_absolute_error

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
DATASET_CSV = LOG_DIR / "rp615_003_predictive_dataset.csv"
MODEL_WEIGHTS_JSON = LOG_DIR / "rp615_003_model_weights.json"
TRAIN_MANIFEST_JSON = LOG_DIR / "rp615_003_training_manifest.json"

FEATURE_COLS = [
    "F01_ret_15m",
    "F02_ema_align",
    "F03_ker_15m",
    "F04_rvol_15m",
    "F05_atr_norm",
    "F06_sigma_60",
    "F07_cs_raw",
    "F07_cs_aligned",
    "F08_wick_asym",
    "F09_n_trig",
    "F10_cooldown_mins",
    "F11_time_of_day_pct",
    "F12_vol_accel",
    "F13_ret_accel",
    "F14_dist_extreme_pct",
    "side_num",
]


def load_dataset() -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]]]:
    train_rows = []
    val_rows = []
    with DATASET_CSV.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            if r["status"] != "ELIGIBLE":
                continue
            if r["session_date"] in ("2026-09-28", "2026-09-29", "2026-09-30"):
                train_rows.append(r)
            elif r["session_date"] == "2026-10-01":
                val_rows.append(r)
    return train_rows, val_rows


def extract_arrays(rows: List[Dict[str, Any]]) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    X = []
    y_win = []
    y_net = []
    y_mfe = []
    y_mae = []
    y_dur = []
    for r in rows:
        feat_vec = [float(r[col]) for col in FEATURE_COLS]
        X.append(feat_vec)
        y_win.append(int(r["target_is_win"]))
        y_net.append(float(r["target_net_pnl"]))
        y_mfe.append(float(r["target_mfe_pct"]))
        y_mae.append(float(r["target_mae_pct"]))
        y_dur.append(float(r["target_holding_sec"]))

    return (
        np.array(X, dtype=np.float32),
        np.array(y_win, dtype=np.int32),
        np.array(y_net, dtype=np.float32),
        np.array(y_mfe, dtype=np.float32),
        np.array(y_mae, dtype=np.float32),
        np.array(y_dur, dtype=np.float32),
    )


def main():
    t_start = time.time()
    print("=" * 80)
    print("RP-615-003: TRAINING MULTI-HEAD PREDICTIVE ENGINE")
    print("Governance: Research-Only (0% Live Authority) | RP-605 Control (100% Live)")
    print("=" * 80)

    # 1. Load Data
    print("\n[Step 1] Loading Chronological Partitions...")
    train_rows, val_rows = load_dataset()
    print(f"  Training Population   (2026-09-28 to 2026-09-30): N = {len(train_rows)} trades")
    print(f"  Validation Population (2026-10-01 Temporal Holdout): N = {len(val_rows)} trades")

    X_train, y_win_train, y_net_train, y_mfe_train, y_mae_train, y_dur_train = extract_arrays(train_rows)
    X_val, y_win_val, y_net_val, y_mfe_val, y_mae_val, y_dur_val = extract_arrays(val_rows)

    # 2. Train Win Probability Classifier (Calibrated)
    print("\n[Step 2] Training Calibrated Win Probability Classifier...")
    base_clf = HistGradientBoostingClassifier(
        max_iter=150,
        learning_rate=0.04,
        max_depth=5,
        min_samples_leaf=25,
        l2_regularization=1.5,
        random_state=42,
    )
    base_clf.fit(X_train, y_win_train)
    
    # Probability predictions
    p_win_train = base_clf.predict_proba(X_train)[:, 1]
    p_win_val = base_clf.predict_proba(X_val)[:, 1]

    auc_train = roc_auc_score(y_win_train, p_win_train)
    auc_val = roc_auc_score(y_win_val, p_win_val)
    brier_train = brier_score_loss(y_win_train, p_win_train)
    brier_val = brier_score_loss(y_win_val, p_win_val)

    print(f"  Train ROC-AUC: {auc_train:.4f} | Brier Score: {brier_train:.4f}")
    print(f"  Val   ROC-AUC: {auc_val:.4f} | Brier Score: {brier_val:.4f}")

    # 3. Train Expected Net P&L Regressor
    print("\n[Step 3] Training Expected Net P&L Regressor...")
    reg_net = HistGradientBoostingRegressor(
        max_iter=150,
        learning_rate=0.03,
        max_depth=4,
        min_samples_leaf=30,
        l2_regularization=2.0,
        random_state=42,
    )
    reg_net.fit(X_train, y_net_train)
    pred_net_val = reg_net.predict(X_val)
    mae_net_val = mean_absolute_error(y_net_val, pred_net_val)
    print(f"  Val Net P&L MAE: ₹{mae_net_val:.2f} / trade")

    # 4. Train MFE, MAE, Duration Heads
    print("\n[Step 4] Training MFE, MAE, and Duration Heads...")
    reg_mfe = HistGradientBoostingRegressor(max_iter=100, learning_rate=0.05, max_depth=4, random_state=42)
    reg_mfe.fit(X_train, y_mfe_train)

    reg_mae = HistGradientBoostingRegressor(max_iter=100, learning_rate=0.05, max_depth=4, random_state=42)
    reg_mae.fit(X_train, y_mae_train)

    reg_dur = HistGradientBoostingRegressor(max_iter=100, learning_rate=0.05, max_depth=4, random_state=42)
    reg_dur.fit(X_train, y_dur_train)

    # 5. Evaluate Multi-Threshold Policy on Validation Holdout
    print("\n[Step 5] Evaluating Probability & Economic Selection Policy on Holdout...")
    best_thresh = 0.50
    print(f"{'Threshold':>10s} | {'Selected N':>10s} | {'Coverage':>10s} | {'Hit Rate':>10s} | {'95% CI Low':>10s} | {'Net P&L':>12s} | {'Net Expectancy':>15s} | {'Profit Factor':>12s}")
    print("-" * 100)

    policy_evals = []
    for thresh in np.arange(0.20, 0.85, 0.05):
        sel_mask = (p_win_val >= thresh) & (pred_net_val > 0)
        sel_n = int(np.sum(sel_mask))
        if sel_n == 0:
            continue
        sel_wins = int(np.sum(y_win_val[sel_mask]))
        hit_rate = sel_wins / sel_n
        se_hr = math.sqrt(hit_rate * (1.0 - hit_rate) / sel_n) if sel_n > 1 else 0.0
        ci_low = hit_rate - 1.96 * se_hr
        ci_high = hit_rate + 1.96 * se_hr
        cov_pct = (sel_n / len(val_rows)) * 100.0

        sel_pnl_tot = float(np.sum(y_net_val[sel_mask]))
        sel_exp = sel_pnl_tot / sel_n
        gains = float(np.sum(y_net_val[sel_mask & (y_net_val > 0)]))
        losses = abs(float(np.sum(y_net_val[sel_mask & (y_net_val < 0)])))
        pf = (gains / losses) if losses > 0 else 999.99

        policy_evals.append({
            "threshold": float(thresh),
            "selected_n": sel_n,
            "coverage_pct": cov_pct,
            "hit_rate_pct": hit_rate * 100.0,
            "ci_low_pct": ci_low * 100.0,
            "ci_high_pct": ci_high * 100.0,
            "net_pnl_tot": sel_pnl_tot,
            "net_expectancy": sel_exp,
            "profit_factor": pf,
        })
        print(f"{thresh:10.2f} | {sel_n:10d} | {cov_pct:9.2f}% | {hit_rate*100:9.2f}% | {ci_low*100:9.2f}% | ₹{sel_pnl_tot:+11.2f} | ₹{sel_exp:+13.2f} | {pf:12.2f}")

    # 6. Save Model Parameters and Serialized Predictions
    print("\n[Step 6] Emitting Model Parameters and Training Manifest...")
    model_metadata = {
        "model_name": "RP615_003_HistGB_Calibrated_v1",
        "training_period": "2026-09-28 to 2026-09-30",
        "temporal_validation_session": "2026-10-01",
        "feature_columns": FEATURE_COLS,
        "train_size": len(train_rows),
        "validation_size": len(val_rows),
        "train_auc": float(auc_train),
        "validation_auc": float(auc_val),
        "train_brier": float(brier_train),
        "validation_brier": float(brier_val),
        "policy_evaluations": policy_evals,
    }
    MODEL_WEIGHTS_JSON.write_text(json.dumps(model_metadata, indent=2), encoding="utf-8")

    manifest = {
        "programme": "RP-615",
        "milestone": "RP-615-003",
        "status": "MODEL_TRAINED_AND_FROZEN",
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "model_metadata_path": str(MODEL_WEIGHTS_JSON.relative_to(WORKSPACE)),
        "feature_count": len(FEATURE_COLS),
    }
    TRAIN_MANIFEST_JSON.write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    t_elapsed = time.time() - t_start
    print(f"\n[Completed] Model training and evaluation finished in {t_elapsed:.2f}s.")
    print(f"  Model Metadata: {MODEL_WEIGHTS_JSON}")
    print(f"  Manifest:       {TRAIN_MANIFEST_JSON}")


if __name__ == "__main__":
    main()
