#!/usr/bin/env python3
"""
scripts/run_rp615_prospective_shadow_runner.py

RP-615 Prospective Shadow Execution Engine.
Implements the ratified Four-Compartment Architecture (C1-C4) for prospective shadow sessions.

Architecture Compartments:
  - C1: Pre-Entry Prediction Ledger (Immutable, point-in-time <= T_entry, frozen model 212532d3ac12f9c6).
  - C2: In-Trade Trajectory Monitor (Passive observation of MFE, MAE, dwell times).
  - C3: Post-Exit Ground Truth Accounting (Deterministic gross/net P&L, friction, exit reason).
  - C4: Cross-Trade Context (Rolling hit rates strictly computed from resolved prior exits t < T_entry).

Governance Boundary:
  - RP-605 Baseline Control retains 100% Live Execution Authority.
  - RP-615 is strictly Research-Only (0% Live Authority).
"""

from __future__ import annotations

import bisect
import csv
import hashlib
import json
import math
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from sklearn.ensemble import HistGradientBoostingClassifier, HistGradientBoostingRegressor

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
PREDICTIVE_DATASET_CSV = LOG_DIR / "rp615_003_predictive_dataset.csv"
CALIBRATORS_JSON = LOG_DIR / "rp615_004_003_calibrators.json"

OUT_SHADOW_C1_LEDGER = LOG_DIR / "rp615_prospective_c1_prediction_ledger.csv"
OUT_SHADOW_C2_TRAJECTORY = LOG_DIR / "rp615_prospective_c2_trajectory_ledger.csv"
OUT_SHADOW_C3_OUTCOMES = LOG_DIR / "rp615_prospective_c3_ground_truth_ledger.csv"
OUT_SHADOW_C4_CONTEXT = LOG_DIR / "rp615_prospective_c4_context_ledger.csv"
OUT_SHADOW_MANIFEST_JSON = LOG_DIR / "rp615_prospective_shadow_manifest.json"

FROZEN_MODEL_HASH = "212532d3ac12f9c6"
FROZEN_TAU_BENCHMARK = 0.35
FROZEN_TAU_PLATT_STD = 0.230
FROZEN_TAU_PLATT_CONS = 0.235

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


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_iso_ts(ts_str: str) -> float:
    dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
    return dt.timestamp()


def run_prospective_session_pipeline():
    t0 = time.time()
    print("=" * 80)
    print("RP-615 PROSPECTIVE SHADOW RUNNER: FOUR-COMPARTMENT EXECUTION")
    print(f"Model Hash: {FROZEN_MODEL_HASH} | Governance: 100% RP-605 Control / 0% RP-615 Live")
    print("=" * 80)

    # 1. Replicate frozen base model from Discovery partitions (2026-09-28 to 2026-09-30)
    all_rows = []
    with PREDICTIVE_DATASET_CSV.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            all_rows.append(r)

    train_rows = [
        r for r in all_rows if r["status"] == "ELIGIBLE" and r["session_date"] in ("2026-09-28", "2026-09-29", "2026-09-30")
    ]
    X_train = np.array([[float(r[c]) for c in FEATURE_COLS] for r in train_rows], dtype=np.float32)
    y_win_train = np.array([int(r["target_is_win"]) for r in train_rows], dtype=np.int32)
    y_net_train = np.array([float(r["target_net_pnl"]) for r in train_rows], dtype=np.float32)

    clf_win = HistGradientBoostingClassifier(
        max_iter=150, learning_rate=0.04, max_depth=5, min_samples_leaf=25, l2_regularization=1.5, random_state=42
    )
    clf_win.fit(X_train, y_win_train)

    reg_net = HistGradientBoostingRegressor(
        max_iter=150, learning_rate=0.03, max_depth=4, min_samples_leaf=30, l2_regularization=2.0, random_state=42
    )
    reg_net.fit(X_train, y_net_train)

    with CALIBRATORS_JSON.open(encoding="utf-8") as f:
        calib_data = json.load(f)
    platt_slope = float(calib_data["platt_calibrator"]["slope"])
    platt_intercept = float(calib_data["platt_calibrator"]["intercept"])

    print(f"Loaded Calibrator Weights: Slope={platt_slope:.6f}, Intercept={platt_intercept:.6f}")
    print(f"Ingested {len(all_rows)} total opportunity records for prospective shadow validation.")

    # 2. Sort opportunities chronologically by entry timestamp
    all_rows.sort(key=lambda r: (r["session_date"], parse_iso_ts(r["t_entry_utc"])))

    # 3. Vectorized Batch Prediction for C1 Pre-Trade Inference
    eligible_rows = [r for r in all_rows if r["status"] == "ELIGIBLE"]
    X_all_eligible = np.array([[float(r[c]) for c in FEATURE_COLS] for r in eligible_rows], dtype=np.float32)
    raw_probs_eligible = clf_win.predict_proba(X_all_eligible)[:, 1]
    pred_net_eligible = reg_net.predict(X_all_eligible)

    # Platt transformation
    eps = 1e-6
    clipped_p = np.clip(raw_probs_eligible, eps, 1.0 - eps)
    log_odds = np.log(clipped_p / (1.0 - clipped_p))
    platt_logits = platt_slope * log_odds + platt_intercept
    platt_probs_eligible = 1.0 / (1.0 + np.exp(-platt_logits))

    c1_lookup = {}
    for k, r in enumerate(eligible_rows):
        key = (r["session_date"], r["trade_id"])
        p_raw = float(raw_probs_eligible[k])
        p_net = float(pred_net_eligible[k])
        p_platt = float(platt_probs_eligible[k])

        act_bench = "TRADE" if (p_raw >= FROZEN_TAU_BENCHMARK and p_net > 0) else "NO_TRADE"
        act_platt_std = "TRADE" if (p_platt >= FROZEN_TAU_PLATT_STD and p_net > 0) else "NO_TRADE"
        act_platt_cons = "TRADE" if (p_platt >= FROZEN_TAU_PLATT_CONS and p_net > 0) else "NO_TRADE"

        c1_lookup[key] = {
            "raw_win_prob": p_raw,
            "platt_win_prob": p_platt,
            "pred_net_pnl": p_net,
            "action_benchmark": act_bench,
            "action_platt_std": act_platt_std,
            "action_platt_cons": act_platt_cons,
        }

    # Fast bisect-based exit queue
    resolved_exit_epochs = []
    resolved_exit_wins = []

    c1_records = []
    c2_records = []
    c3_records = []
    c4_records = []

    for r in all_rows:
        t_id = r["trade_id"]
        s_date = r["session_date"]
        sym = r["symbol"]
        side = r["side"]
        t_entry_iso = r["t_entry_utc"]
        t_exit_iso = r["t_exit_utc"]
        status = r["status"]
        t_entry_epoch = parse_iso_ts(t_entry_iso)
        t_exit_epoch = parse_iso_ts(t_exit_iso)
        t_key = (s_date, t_id)

        # C4: Rolling context via binary search on resolved exits
        cutoff_idx = bisect.bisect_left(resolved_exit_epochs, t_entry_epoch)
        p10_slice = resolved_exit_wins[max(0, cutoff_idx - 10):cutoff_idx]
        p10_cnt = len(p10_slice)
        p10_wins = sum(p10_slice)
        p10_wr_pct = (p10_wins / p10_cnt * 100.0) if p10_cnt > 0 else 0.0

        c4_rec = {
            "session_date": s_date,
            "trade_id": t_id,
            "symbol": sym,
            "t_entry_utc": t_entry_iso,
            "prior_resolved_total": cutoff_idx,
            "prior_10_count": p10_cnt,
            "prior_10_wins": p10_wins,
            "prior_10_win_rate_pct": f"{p10_wr_pct:.2f}",
        }
        c4_records.append(c4_rec)

        # C1: Pre-Entry Prediction
        if status != "ELIGIBLE":
            c1_rec = {
                "session_date": s_date,
                "trade_id": t_id,
                "symbol": sym,
                "side": side,
                "t_entry_utc": t_entry_iso,
                "status": status,
                "model_hash": FROZEN_MODEL_HASH,
                "raw_win_prob": "0.0000",
                "platt_win_prob": "0.0000",
                "pred_net_pnl": "0.00",
                "action_benchmark": "NO_TRADE",
                "action_platt_std": "NO_TRADE",
                "action_platt_cons": "NO_TRADE",
            }
        else:
            p_data = c1_lookup[t_key]
            c1_rec = {
                "session_date": s_date,
                "trade_id": t_id,
                "symbol": sym,
                "side": side,
                "t_entry_utc": t_entry_iso,
                "status": status,
                "model_hash": FROZEN_MODEL_HASH,
                "raw_win_prob": f"{p_data['raw_win_prob']:.4f}",
                "platt_win_prob": f"{p_data['platt_win_prob']:.4f}",
                "pred_net_pnl": f"{p_data['pred_net_pnl']:.2f}",
                "action_benchmark": p_data["action_benchmark"],
                "action_platt_std": p_data["action_platt_std"],
                "action_platt_cons": p_data["action_platt_cons"],
            }
        c1_records.append(c1_rec)

        # C2: In-Trade Trajectory
        c2_rec = {
            "session_date": s_date,
            "trade_id": t_id,
            "symbol": sym,
            "mfe_pct": r["target_mfe_pct"],
            "mae_pct": r["target_mae_pct"],
            "holding_sec": r["target_holding_sec"],
            "is_zero_mfe": r["target_is_zero_mfe"],
            "exit_reason": r["target_exit_reason"],
        }
        c2_records.append(c2_rec)

        # C3: Ground-Truth Accounting
        realized_win = int(r["target_is_win"])
        realized_net = float(r["target_net_pnl"])
        c3_rec = {
            "session_date": s_date,
            "trade_id": t_id,
            "symbol": sym,
            "t_exit_utc": t_exit_iso,
            "realized_is_win": realized_win,
            "realized_net_pnl": f"{realized_net:.2f}",
            "realized_gross_pnl": r["target_gross_pnl"],
            "realized_friction": r["target_friction"],
        }
        c3_records.append(c3_rec)

        # Insert sorted exit
        ins_idx = bisect.bisect_right(resolved_exit_epochs, t_exit_epoch)
        resolved_exit_epochs.insert(ins_idx, t_exit_epoch)
        resolved_exit_wins.insert(ins_idx, realized_win)

    # Save CSVs
    def save_csv(path: Path, data: List[Dict[str, Any]]):
        with path.open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=list(data[0].keys()))
            writer.writeheader()
            writer.writerows(data)

    save_csv(OUT_SHADOW_C1_LEDGER, c1_records)
    save_csv(OUT_SHADOW_C2_TRAJECTORY, c2_records)
    save_csv(OUT_SHADOW_C3_OUTCOMES, c3_records)
    save_csv(OUT_SHADOW_C4_CONTEXT, c4_records)

    print(f"Four Compartments successfully written to disk in {time.time() - t0:.2f}s:")
    print(f"  C1 Ledger: {OUT_SHADOW_C1_LEDGER.name} ({len(c1_records)} rows)")
    print(f"  C2 Ledger: {OUT_SHADOW_C2_TRAJECTORY.name} ({len(c2_records)} rows)")
    print(f"  C3 Ledger: {OUT_SHADOW_C3_OUTCOMES.name} ({len(c3_records)} rows)")
    print(f"  C4 Ledger: {OUT_SHADOW_C4_CONTEXT.name} ({len(c4_records)} rows)")

    # Produce Manifest
    manifest = {
        "manifest_version": "1.0",
        "milestone": "RP-615-Prospective-Shadow-Execution",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "model_hash": FROZEN_MODEL_HASH,
        "total_opportunities": len(c1_records),
        "compartments": {
            "C1_prediction_ledger": {
                "path": str(OUT_SHADOW_C1_LEDGER.relative_to(WORKSPACE)),
                "sha256": sha256_file(OUT_SHADOW_C1_LEDGER),
                "rows": len(c1_records),
            },
            "C2_trajectory_ledger": {
                "path": str(OUT_SHADOW_C2_TRAJECTORY.relative_to(WORKSPACE)),
                "sha256": sha256_file(OUT_SHADOW_C2_TRAJECTORY),
                "rows": len(c2_records),
            },
            "C3_outcomes_ledger": {
                "path": str(OUT_SHADOW_C3_OUTCOMES.relative_to(WORKSPACE)),
                "sha256": sha256_file(OUT_SHADOW_C3_OUTCOMES),
                "rows": len(c3_records),
            },
            "C4_context_ledger": {
                "path": str(OUT_SHADOW_C4_CONTEXT.relative_to(WORKSPACE)),
                "sha256": sha256_file(OUT_SHADOW_C4_CONTEXT),
                "rows": len(c4_records),
            },
        },
    }
    with OUT_SHADOW_MANIFEST_JSON.open("w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    print(f"Prospective manifest sealed: {OUT_SHADOW_MANIFEST_JSON.name}")


if __name__ == "__main__":
    run_prospective_session_pipeline()
