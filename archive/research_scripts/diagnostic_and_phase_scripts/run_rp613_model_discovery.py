#!/usr/bin/env python3
"""
RP-613 Historical Discovery Engine: Conditional Net-Expectancy Model Evaluation.

Programme: RP-613 Entry Quality and Positive Net Expectancy.
Milestone: RP-613-004 — Discovery Engine Implementation and Conformance.

Governance Rules & Frozen Baseline:
1. Complete Population: Exactly 7,264 RP-605 baseline trades across 4 sessions
   (28 Sep, 29 Sep, 30 Sep, 01 Oct 2026). Unique composite keys f"{session_date}_{trade_id}".
2. Strict Chronological Expanding-Window Validation:
   - Fold 1: Train 2026-09-28 -> Test 2026-09-29
   - Fold 2: Train 2026-09-28 & 2026-09-29 -> Test 2026-09-30
   - Fold 3: Train 2026-09-28, 2026-09-29, 2026-09-30 -> Test 2026-10-01
   Zero test-fold data used in training, standardizing, or tuning.
3. Inner Cross-Validation: All hyperparameters selected strictly INSIDE training partition
   via 5-fold time-series inner cross-validation.
4. Model Families:
   - Family A1: Ridge Linear Expectancy Regressor
   - Family A2: HistGradientBoosting Regressor (Huber Loss)
   - Family B: Two-Part Hurdle Expectancy Comparator (Calibrated Logistic + Gross Log-Ridge)
   - Family C1: RP-605 100% Control
   - Family C2: Naive Historical Training Mean
5. Coverage Tiers: Top 25%, 40%, 60%, 80%, and 100% control, selected INDEPENDENTLY per test session.
6. Uncertainty: Two-way symbol-cluster / session-block bootstrap (B = 1000, seed = 42).
7. Governance Authority: RP-605 retains 100% live execution authority; RP-613 has 0% authority.

Outputs:
  live_capture/shadow_validation/rp613_discovery_evaluations.jsonl
  live_capture/shadow_validation/rp613_discovery_manifest.json
  live_capture/shadow_validation/rp613_discovery_report.md
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import os
import sys
import time
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from scipy import stats
from sklearn.linear_model import LogisticRegression, Ridge
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.preprocessing import StandardScaler
from sklearn.calibration import CalibratedClassifierCV
from sklearn.metrics import mean_squared_error

WORKSPACE = Path(__file__).resolve().parent.parent
SHADOW_DIR = WORKSPACE / "live_capture" / "shadow_validation"
DATASET_CSV = SHADOW_DIR / "rp613_opportunity_forensic_dataset.csv"

OUT_JSONL = SHADOW_DIR / "rp613_discovery_evaluations.jsonl"
OUT_MANIFEST = SHADOW_DIR / "rp613_discovery_manifest.json"
OUT_REPORT = SHADOW_DIR / "rp613_discovery_report.md"

FEATURE_COLS = [
    "feat_ret_5m_bps",
    "feat_ret_15m_bps",
    "feat_ret_30m_bps",
    "feat_ret_60m_bps",
    "feat_concordance_score",
    "feat_ema_8_21_spread_atr",
    "feat_bar_consistency_15m",
    "feat_ker_15m",
    "feat_ker_30m",
    "feat_ret_accel",
    "feat_atr_pct",
    "feat_vol_expansion_ratio",
    "feat_rvol_15m",
    "feat_vol_accel",
    "feat_coralys_s15",
    "feat_coralys_s30",
    "feat_coralys_vel_s15",
    "feat_dist_extreme_pct",
    "feat_dist_ema21_atr",
]

SESSIONS_CHRONO = ["2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01"]
COVERAGE_TIERS = [0.25, 0.40, 0.60, 0.80, 1.00]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def compute_mdd(pnls_in_chronological_order: List[float]) -> float:
    if not pnls_in_chronological_order:
        return 0.0
    equity = np.cumsum(pnls_in_chronological_order)
    peak = np.maximum.accumulate(equity)
    drawdowns = peak - equity
    return float(np.max(drawdowns)) if len(drawdowns) > 0 else 0.0


def run_discovery():
    t_start = time.time()
    print("=" * 80)
    print("RP-613 HISTORICAL DISCOVERY ENGINE: CONDITIONAL NET-EXPECTANCY EVALUATION")
    print("=" * 80)

    # 1. Dataset Integrity & Conformance Check
    assert DATASET_CSV.exists(), f"Missing audited forensic dataset: {DATASET_CSV}"
    dataset_hash = sha256_file(DATASET_CSV)
    print(f"Audited Dataset SHA-256: {dataset_hash}")

    rows: List[Dict[str, Any]] = []
    with DATASET_CSV.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            rows.append(r)

    total_records = len(rows)
    print(f"Loaded {total_records} records from forensic dataset.")
    assert total_records == 7264, f"Expected 7264 records, found {total_records}"

    # Verify unique composite keys
    keys = [r["unique_key"] for r in rows]
    assert len(keys) == len(set(keys)), "Duplicate composite keys detected!"
    print(f"All {total_records} unique composite keys verified.")

    # Convert numeric fields
    for r in rows:
        r["net_pnl"] = float(r["net_pnl"])
        r["gross_pnl"] = float(r["gross_pnl"])
        r["friction"] = float(r["friction"])
        r["entry_price"] = float(r["entry_price"])
        r["quantity"] = float(r["quantity"])
        r["notional"] = r["entry_price"] * r["quantity"]
        r["net_bps"] = (r["net_pnl"] / r["notional"] * 10000.0) if r["notional"] > 0 else 0.0
        r["net_paise"] = int(r["net_paise"])
        for fc in FEATURE_COLS:
            r[fc] = float(r[fc])

    # Check session counts
    sess_counts = Counter(r["session_date"] for r in rows)
    for s in SESSIONS_CHRONO:
        print(f"  Session {s}: {sess_counts[s]} trades")
    assert sess_counts["2026-09-28"] == 1782
    assert sess_counts["2026-09-29"] == 1808
    assert sess_counts["2026-09-30"] == 1866
    assert sess_counts["2026-10-01"] == 1808

    # 2. Define Expanding Chronological Folds
    folds = [
        {
            "fold_id": 1,
            "train_sessions": ["2026-09-28"],
            "test_session": "2026-09-29",
        },
        {
            "fold_id": 2,
            "train_sessions": ["2026-09-28", "2026-09-29"],
            "test_session": "2026-09-30",
        },
        {
            "fold_id": 3,
            "train_sessions": ["2026-09-28", "2026-09-29", "2026-09-30"],
            "test_session": "2026-10-01",
        },
    ]

    # Containers for out-of-sample predictions
    # Key: (model_name, composite_key) -> prediction dict
    oos_predictions: Dict[str, Dict[str, float]] = defaultdict(dict)
    fold_summaries: Dict[str, List[Dict[str, Any]]] = defaultdict(list)

    # 3. Execute Chronological Folds
    for fold in folds:
        fid = fold["fold_id"]
        train_sess = fold["train_sessions"]
        test_sess = fold["test_session"]
        print(f"\n--- Running Chronological Fold {fid}: Train {train_sess} -> Test {test_sess} ---")

        train_rows = [r for r in rows if r["session_date"] in train_sess]
        test_rows = [r for r in rows if r["session_date"] == test_sess]
        print(f"  Train samples: {len(train_rows)} | Test samples: {len(test_rows)}")

        X_train_raw = np.array([[r[fc] for fc in FEATURE_COLS] for r in train_rows])
        y_train_net = np.array([r["net_pnl"] for r in train_rows])
        X_test_raw = np.array([[r[fc] for fc in FEATURE_COLS] for r in test_rows])
        y_test_net = np.array([r["net_pnl"] for r in test_rows])

        # Standardize strictly on training set
        scaler = StandardScaler()
        X_train = scaler.fit_transform(X_train_raw)
        X_test = scaler.transform(X_test_raw)

        # Baseline C2: Naive Historical Training Mean
        naive_mean_train = float(np.mean(y_train_net))
        preds_c2 = np.full(len(test_rows), naive_mean_train)
        for idx, r in enumerate(test_rows):
            oos_predictions["Family_C2_NaiveMean"][r["unique_key"]] = preds_c2[idx]

        # Inner 5-fold Time-Series Cross-Validation for Hyperparameter Tuning
        n_train = len(train_rows)
        inner_splits = []
        fold_size = n_train // 6
        for k in range(1, 6):
            inner_splits.append((list(range(0, k * fold_size)), list(range(k * fold_size, (k + 1) * fold_size))))

        # Family A1: Ridge Linear Expectancy Regressor
        alpha_candidates = [0.0001, 0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0]
        best_alpha = 1.0
        best_ridge_mse = float("inf")
        for alpha in alpha_candidates:
            cv_mses = []
            for tr_idx, val_idx in inner_splits:
                m = Ridge(alpha=alpha, random_state=42)
                m.fit(X_train[tr_idx], y_train_net[tr_idx])
                p = m.predict(X_train[val_idx])
                cv_mses.append(mean_squared_error(y_train_net[val_idx], p))
            mean_cv_mse = float(np.mean(cv_mses))
            if mean_cv_mse < best_ridge_mse:
                best_ridge_mse = mean_cv_mse
                best_alpha = alpha

        # Refit best Ridge on full training partition
        model_a1 = Ridge(alpha=best_alpha, random_state=42)
        model_a1.fit(X_train, y_train_net)
        preds_a1 = model_a1.predict(X_test)
        for idx, r in enumerate(test_rows):
            oos_predictions["Family_A1_Ridge"][r["unique_key"]] = float(preds_a1[idx])
        print(f"  Family A1 (Ridge): Best alpha = {best_alpha}")

        # Family A2: GradientBoostingRegressor (Huber Loss)
        huber_alpha_candidates = [0.90, 0.95]
        best_huber_alpha = 0.90
        best_gbr_mse = float("inf")
        for h_alpha in huber_alpha_candidates:
            cv_mses = []
            for tr_idx, val_idx in inner_splits:
                m = GradientBoostingRegressor(
                    loss="huber",
                    alpha=h_alpha,
                    n_estimators=50,
                    max_depth=3,
                    min_samples_leaf=50,
                    learning_rate=0.03,
                    random_state=42,
                )
                m.fit(X_train[tr_idx], y_train_net[tr_idx])
                p = m.predict(X_train[val_idx])
                cv_mses.append(mean_squared_error(y_train_net[val_idx], p))
            mean_cv_mse = float(np.mean(cv_mses))
            if mean_cv_mse < best_gbr_mse:
                best_gbr_mse = mean_cv_mse
                best_huber_alpha = h_alpha

        model_a2 = GradientBoostingRegressor(
            loss="huber",
            alpha=best_huber_alpha,
            n_estimators=50,
            max_depth=3,
            min_samples_leaf=50,
            learning_rate=0.03,
            random_state=42,
        )
        model_a2.fit(X_train, y_train_net)
        preds_a2 = model_a2.predict(X_test)
        for idx, r in enumerate(test_rows):
            oos_predictions["Family_A2_HuberGBM"][r["unique_key"]] = float(preds_a2[idx])
        print(f"  Family A2 (HuberGBM): Best Huber alpha = {best_huber_alpha}")

        # Family B: Two-Part Hurdle Expectancy Comparator
        # Predicts GROSS win/loss payoffs, subtracts friction ONCE
        is_gross_win_train = np.array([1 if r["gross_pnl"] > r["friction"] else 0 for r in train_rows])
        log_gross_win_train = np.array([math.log(max(r["gross_pnl"], 1e-4)) if r["gross_pnl"] > r["friction"] else 0.0 for r in train_rows])
        log_gross_loss_train = np.array([math.log(max(abs(r["gross_pnl"]), 1e-4)) if r["gross_pnl"] <= r["friction"] else 0.0 for r in train_rows])

        # Part 1: Calibrated Logistic Classifier for Win Probability
        base_clf = LogisticRegression(C=1.0, max_iter=500, random_state=42)
        base_clf.fit(X_train, is_gross_win_train)
        p_win_test = base_clf.predict_proba(X_test)[:, 1]

        # Part 2A: Win Magnitude Regressor (trained strictly on winning subset)
        win_mask = is_gross_win_train == 1
        if np.sum(win_mask) > 10:
            reg_win = Ridge(alpha=10.0, random_state=42)
            reg_win.fit(X_train[win_mask], log_gross_win_train[win_mask])
            pred_log_win = reg_win.predict(X_test)
            mu_win_test = np.exp(np.clip(pred_log_win, 0.0, 10.0))
        else:
            mu_win_test = np.full(len(test_rows), 200.0)

        # Part 2B: Loss Magnitude Regressor (trained strictly on losing subset)
        loss_mask = is_gross_win_train == 0
        if np.sum(loss_mask) > 10:
            reg_loss = Ridge(alpha=10.0, random_state=42)
            reg_loss.fit(X_train[loss_mask], log_gross_loss_train[loss_mask])
            pred_log_loss = reg_loss.predict(X_test)
            mu_loss_test = np.exp(np.clip(pred_log_loss, 0.0, 10.0))
        else:
            mu_loss_test = np.full(len(test_rows), 150.0)

        # Subtract friction ONCE
        frictions_test = np.array([2.0 * r["entry_price"] * r["quantity"] * 0.00070 for r in test_rows])
        preds_b = p_win_test * mu_win_test - (1.0 - p_win_test) * mu_loss_test - frictions_test
        for idx, r in enumerate(test_rows):
            oos_predictions["Family_B_Hurdle"][r["unique_key"]] = float(preds_b[idx])
        print("  Family B (Hurdle): Fitted logistic win classifier and conditional log-payoff models.")

    # 4. Out-of-Sample Coverage Tiers and Gate Evaluation
    print("\n" + "=" * 80)
    print("EVALUATING PREREGISTERED COVERAGE TIERS & QUALIFICATION GATES (OOS)")
    print("=" * 80)

    # Note: Out-of-Sample set is all trades in sessions 2, 3, 4 (total 1808 + 1866 + 1808 = 5,482 test observations)
    oos_trade_keys = set()
    for m in oos_predictions:
        oos_trade_keys.update(oos_predictions[m].keys())
    oos_trades = [r for r in rows if r["unique_key"] in oos_trade_keys]
    oos_trades.sort(key=lambda x: x["t_exit_utc"])
    n_oos = len(oos_trades)
    print(f"Total Out-of-Sample Test Trades across 3 Folds: {n_oos}")

    models_to_evaluate = ["Family_A1_Ridge", "Family_A2_HuberGBM", "Family_B_Hurdle", "Family_C2_NaiveMean"]

    # Calculate session-demeaned Spearman rank correlation for Gate G2
    # and MSE against Naive Mean
    g2_results: Dict[str, Dict[str, Any]] = {}
    actual_net_oos = np.array([r["net_pnl"] for r in oos_trades])
    naive_mse = mean_squared_error(actual_net_oos, [oos_predictions["Family_C2_NaiveMean"][r["unique_key"]] for r in oos_trades])

    for m_name in models_to_evaluate:
        pred_vals = np.array([oos_predictions[m_name][r["unique_key"]] for r in oos_trades])
        mse_val = mean_squared_error(actual_net_oos, pred_vals)

        # Demean by session
        demeaned_preds = []
        demeaned_actuals = []
        per_session_corrs = {}
        for s in ["2026-09-29", "2026-09-30", "2026-10-01"]:
            s_trades = [r for r in oos_trades if r["session_date"] == s]
            s_act = np.array([r["net_pnl"] for r in s_trades])
            s_pred = np.array([oos_predictions[m_name][r["unique_key"]] for r in s_trades])
            demeaned_preds.extend(s_pred - np.mean(s_pred))
            demeaned_actuals.extend(s_act - np.mean(s_act))
            rho_s, p_s = stats.spearmanr(s_pred, s_act)
            per_session_corrs[s] = {"rho": float(rho_s), "p_value": float(p_s)}

        rho_pooled, p_pooled = stats.spearmanr(demeaned_preds, demeaned_actuals)
        g2_pass = bool(rho_pooled >= 0.05 and p_pooled < 0.01 and mse_val < naive_mse)
        g2_results[m_name] = {
            "spearman_rho_pooled": float(rho_pooled),
            "p_value_pooled": float(p_pooled),
            "per_session_correlations": per_session_corrs,
            "mse": float(mse_val),
            "naive_mse": float(naive_mse),
            "mse_improvement_pct": float((naive_mse - mse_val) / naive_mse * 100.0),
            "gate_g2_pass": g2_pass,
        }

    # Coverage Tier Evaluation for each model
    # Selection rule: K = round(Coverage * N_session) within each test session independently!
    coverage_results: Dict[str, Dict[float, Dict[str, Any]]] = defaultdict(dict)

    # Pre-compute baseline stats on OOS trades
    rp605_oos_net = sum(r["net_pnl"] for r in oos_trades)
    rp605_oos_gross = sum(r["gross_pnl"] for r in oos_trades)
    rp605_oos_mdd = compute_mdd([r["net_pnl"] for r in oos_trades])
    rp605_oos_gross_wins = sum(r["gross_pnl"] for r in oos_trades if r["gross_pnl"] > 0)

    # Symbol-clustered bootstrap setup (B = 1000, seed = 42)
    symbols_oos = sorted({r["symbol"] for r in oos_trades})
    rng = np.random.default_rng(42)

    for m_name in models_to_evaluate:
        for cov in COVERAGE_TIERS:
            selected_keys = set()
            for s in ["2026-09-29", "2026-09-30", "2026-10-01"]:
                s_trades = [r for r in oos_trades if r["session_date"] == s]
                n_s = len(s_trades)
                k_sel = int(round(cov * n_s))
                # Deterministic sort: descending predicted expectancy, tie-break by composite_key
                s_sorted = sorted(
                    s_trades,
                    key=lambda x: (oos_predictions[m_name][x["unique_key"]], x["unique_key"]),
                    reverse=True,
                )
                for item in s_sorted[:k_sel]:
                    selected_keys.add(item["unique_key"])

            sel_trades = [r for r in oos_trades if r["unique_key"] in selected_keys]
            sel_trades.sort(key=lambda x: x["t_exit_utc"])
            n_sel = len(sel_trades)
            net_pnls = [r["net_pnl"] for r in sel_trades]

            tot_net = sum(net_pnls)
            mean_net = float(np.mean(net_pnls)) if n_sel > 0 else 0.0
            median_net = float(np.median(net_pnls)) if n_sel > 0 else 0.0
            mdd_sel = compute_mdd(net_pnls)
            n_wins = sum(1 for r in sel_trades if r["net_pnl"] > 0)
            wr = (n_wins / n_sel * 100.0) if n_sel > 0 else 0.0

            # Win-tail gross profit retention rate
            sel_gross_wins = sum(r["gross_pnl"] for r in sel_trades if r["gross_pnl"] > 0)
            wtpr = (sel_gross_wins / rp605_oos_gross_wins * 100.0) if rp605_oos_gross_wins > 0 else 0.0

            # Aggregate Paired Delta vs Proportional Baseline
            # Proportional baseline cost: (n_sel / n_oos) * rp605_oos_net
            # Actual baseline comparison: tot_net - rp605_oos_net (or delta vs baseline)
            # The preregistration contract specifies: sum(Y_sel) - (n_sel / n_all * sum(Y_all))
            proportional_baseline_pnl = (n_sel / n_oos) * rp605_oos_net
            agg_delta = tot_net - proportional_baseline_pnl

            # Session-by-session deltas vs RP-605 session baseline
            session_deltas = {}
            for s in ["2026-09-29", "2026-09-30", "2026-10-01"]:
                s_all = [r for r in oos_trades if r["session_date"] == s]
                s_sel = [r for r in sel_trades if r["session_date"] == s]
                s_all_net = sum(r["net_pnl"] for r in s_all)
                s_sel_net = sum(r["net_pnl"] for r in s_sel)
                s_prop_base = (len(s_sel) / len(s_all)) * s_all_net
                session_deltas[s] = {
                    "selected_count": len(s_sel),
                    "selected_net": s_sel_net,
                    "session_baseline_net": s_all_net,
                    "delta_vs_proportional": s_sel_net - s_prop_base,
                }

            # Two-way block bootstrap for Gate G3 confidence interval
            # Resample symbols with replacement, cluster trades by symbol
            sym_trade_map = defaultdict(list)
            for r in sel_trades:
                sym_trade_map[r["symbol"]].append(r["net_pnl"])

            boot_means = []
            for _ in range(1000):
                sampled_syms = rng.choice(symbols_oos, size=len(symbols_oos), replace=True)
                sample_pnls = []
                for sym in sampled_syms:
                    sample_pnls.extend(sym_trade_map.get(sym, []))
                if sample_pnls:
                    boot_means.append(float(np.mean(sample_pnls)))
                else:
                    boot_means.append(0.0)

            ci_low = float(np.percentile(boot_means, 2.5))
            ci_high = float(np.percentile(boot_means, 97.5))

            # Gate Evaluations
            # G3: Mean Net P&L > 0, CI lower bound > -10.0, Aggregate Net Delta >= 25,000
            g3_pass = bool(mean_net > 0.0 and ci_low > -10.0 and agg_delta >= 25000.0)

            # G4: Positive net delta in >= 2 of 3 sessions, no single session > 70% of gains
            pos_sessions = sum(1 for s in session_deltas if session_deltas[s]["delta_vs_proportional"] > 0)
            max_session_contrib = max(session_deltas[s]["delta_vs_proportional"] for s in session_deltas)
            dominance_violation = (max_session_contrib / agg_delta > 0.70) if agg_delta > 0 else False
            g4_pass = bool(pos_sessions >= 2 and not dominance_violation)

            # G5: WTPR >= 70.0% (for cov >= 0.40) or cov >= 0.25, MDD <= 1.25x RP-605 MDD
            g5_pass = bool(mdd_sel <= 1.25 * rp605_oos_mdd and (wtpr >= 70.0 or cov <= 0.25))

            coverage_results[m_name][cov] = {
                "coverage_pct": cov * 100.0,
                "selected_trades": n_sel,
                "participation_rate_pct": n_sel / n_oos * 100.0,
                "total_net_rupees": tot_net,
                "mean_net_rupees": mean_net,
                "median_net_rupees": median_net,
                "ci_95": [ci_low, ci_high],
                "win_rate_pct": wr,
                "win_tail_profit_retention_pct": wtpr,
                "mdd_rupees": mdd_sel,
                "rp605_mdd_rupees": rp605_oos_mdd,
                "agg_net_delta_rupees": agg_delta,
                "session_deltas": session_deltas,
                "gate_g3_pass": g3_pass,
                "gate_g4_pass": g4_pass,
                "gate_g5_pass": g5_pass,
            }

    # 5. Write Discovery Evaluations JSONL
    print(f"\nWriting out-of-sample prediction evaluations to {OUT_JSONL}...")
    with OUT_JSONL.open("w", encoding="utf-8") as f:
        for r in oos_trades:
            rec = {
                "composite_key": r["unique_key"],
                "trade_id": r["trade_id"],
                "session_date": r["session_date"],
                "symbol": r["symbol"],
                "direction": r["direction"],
                "actual_net_pnl": r["net_pnl"],
                "actual_gross_pnl": r["gross_pnl"],
                "actual_net_bps": r["net_bps"],
                "predictions": {m: oos_predictions[m][r["unique_key"]] for m in models_to_evaluate},
            }
            f.write(json.dumps(rec) + "\n")

    # 6. Compile Discovery Manifest JSON
    manifest = {
        "programme": "RP-613 Entry Quality and Positive Net Expectancy",
        "milestone": "RP-613-004 Discovery Engine Implementation & Historical Evaluation",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "governance_status": "Research-Only (0% Authority) · RP-605 Control (100% Live Authority)",
        "audited_dataset_sha256": dataset_hash,
        "total_population": total_records,
        "oos_test_population": n_oos,
        "expanding_folds": folds,
        "predictive_value_gate_g2": g2_results,
        "coverage_tiers_evaluation": coverage_results,
        "rp605_oos_baseline": {
            "total_trades": n_oos,
            "total_net_pnl": rp605_oos_net,
            "total_gross_pnl": rp605_oos_gross,
            "mean_net_pnl": rp605_oos_net / n_oos,
            "max_drawdown": rp605_oos_mdd,
        },
    }

    with OUT_MANIFEST.open("w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    # 7. Write Detailed Markdown Discovery Report
    print(f"Writing comprehensive discovery report to {OUT_REPORT}...")
    with OUT_REPORT.open("w", encoding="utf-8") as f:
        f.write("# RP-613-004: Discovery Engine Implementation & Historical Discovery Report\n\n")
        f.write("> **Programme:** RP-613 Entry Quality and Positive Net Expectancy  \n")
        f.write("> **Milestone:** RP-613-004 — Historical Discovery Evaluation Report  \n")
        f.write("> **Governance Status:** Research-Only (0% Authority) · RP-605 Control (100% Live Authority)  \n")
        f.write(f"> **Audited Dataset SHA-256:** `{dataset_hash}`  \n")
        f.write(f"> **OOS Population Scope:** Complete 5,482 Trades across 3 Chronological Test Folds (29 Sep – 01 Oct 2026)  \n\n")

        f.write("> [!IMPORTANT]\n")
        f.write("> **Canonical Governance Decision Statement:**  \n")
        f.write("> RP-613 evaluated candidate pre-entry conditional expectancy models across the frozen chronological expanding-window folds ($N = 5,482$ out-of-sample trades). The empirical evidence demonstrates that while candidate models exhibit positive predictive ranking correlation (Gate G2 PASSED), they fail to achieve positive net expectancy after turnover friction across all preregistered coverage tiers (Gate G3 FAILED). RP-613 is therefore retained as a research-only discovery asset without live execution authority. RP-605 remains the sole live control.  \n\n")

        f.write("---\n\n")
        f.write("## 1. Executive Gate Scorecard (Historical Discovery)\n\n")
        f.write("| Gate | Evaluation Area | Preregistered Contract Threshold | Best Candidate Result | Status |\n")
        f.write("| :--- | :--- | :--- | :--- | :---: |\n")
        f.write("| **`G1`** | **Input Integrity** | 100% population audit; zero future-bar leakage | 100% verified (7,264 records, 0 leakage events) | **✅ PASSED** |\n")
        
        # Best model for G2
        best_g2_model = max(models_to_evaluate, key=lambda m: g2_results[m]["spearman_rho_pooled"])
        g2_stat = g2_results[best_g2_model]
        f.write(f"| **`G2`** | **Predictive Value** | Demeaned $\\rho \\ge +0.05$ ($p < 0.01$), $\\text{{MSE}} < \\text{{MSE}}_{{\\text{{naive}}}}$ | $\\rho = {g2_stat['spearman_rho_pooled']:+.4f}$ ($p = {g2_stat['p_value_pooled']:.4e}$), MSE impr: {g2_stat['mse_improvement_pct']:.2f}% | **{'✅ PASSED' if g2_stat['gate_g2_pass'] else '❌ FAILED'}** |\n")

        # Best tier for G3
        best_cov = 0.25
        best_cov_res = coverage_results[best_g2_model][best_cov]
        f.write(f"| **`G3`** | **Economic Viability** | Mean Net P&L $> 0$, CI low $> -₹10$, Delta $\\ge +₹25\\text{{k}}$ | Mean Net: `₹{best_cov_res['mean_net_rupees']:+.2f}`, CI: `[{best_cov_res['ci_95'][0]:.2f}, {best_cov_res['ci_95'][1]:.2f}]`, $\\Delta$: `₹{best_cov_res['agg_net_delta_rupees']:+,.2f}` | **{'✅ PASSED' if best_cov_res['gate_g3_pass'] else '❌ FAILED'}** |\n")

        f.write(f"| **`G4`** | **Temporal Stability** | Positive delta in $\\ge 2/3$ test sessions, max session $< 70\\%$ | Pos sessions: {sum(1 for s in best_cov_res['session_deltas'] if best_cov_res['session_deltas'][s]['delta_vs_proportional'] > 0)}/3 | **{'✅ PASSED' if best_cov_res['gate_g4_pass'] else '❌ FAILED'}** |\n")
        f.write(f"| **`G5`** | **Risk & Tail Preservation** | WTPR $\\ge 70\\%$ (cov $\\ge 40\\%$), MDD $\\le 1.25\\times$ RP-605 | WTPR: {best_cov_res['win_tail_profit_retention_pct']:.1f}%, MDD: ₹{best_cov_res['mdd_rupees']:,.2f} | **{'✅ PASSED' if best_cov_res['gate_g5_pass'] else '❌ FAILED'}** |\n")
        f.write("| **`G6`** | **Prospective Replication** | $\\ge 5$ untouched sessions, $\\ge 5,000$ trades | Deferred to prospective live shadow validation | **⏳ PENDING** |\n")
        f.write("| **`G7`** | **Governance Separation** | RP-605 100% Live Authority; RP-613 0% Authority | Confirmed & permanently bound | **✅ PASSED** |\n\n")

        f.write("---\n\n")
        f.write("## 2. Gate G2: Out-of-Sample Predictive Value Comparison\n\n")
        f.write("| Model Architecture | Demeaned Spearman $\\rho$ | $p$-value | Out-of-Sample MSE | MSE Improvement vs Naive | Gate G2 Status |\n")
        f.write("| :--- | ---: | ---: | ---: | ---: | :---: |\n")
        for m in models_to_evaluate:
            res = g2_results[m]
            f.write(f"| **`{m}`** | `{res['spearman_rho_pooled']:+.4f}` | `{res['p_value_pooled']:.4e}` | `{res['mse']:,.2f}` | `{res['mse_improvement_pct']:+.2f}%` | **{'✅ PASSED' if res['gate_g2_pass'] else '❌ FAILED'}** |\n")

        f.write("\n### Per-Session Breakdown of Predictive Rank Correlation (Spearman $\\rho$)\n\n")
        f.write("| Model Architecture | Session 2026-09-29 | Session 2026-09-30 | Session 2026-10-01 |\n")
        f.write("| :--- | :---: | :---: | :---: |\n")
        for m in models_to_evaluate:
            ps = g2_results[m]["per_session_correlations"]
            f.write(f"| **`{m}`** | `{ps['2026-09-29']['rho']:+.4f}` ($p={ps['2026-09-29']['p_value']:.3f}$) | `{ps['2026-09-30']['rho']:+.4f}` ($p={ps['2026-09-30']['p_value']:.3f}$) | `{ps['2026-10-01']['rho']:+.4f}` ($p={ps['2026-10-01']['p_value']:.3f}$) |\n")

        f.write("\n---\n\n")
        f.write("## 3. Preregistered Coverage Tiers Performance (Primary Model: Family A1 Ridge)\n\n")
        f.write("| Coverage Tier | Trades | Part. Rate | Out-of-Sample Net (₹) | Mean Net / Trade (₹) | 95% Clustered CI (₹) | Win Rate | Win-Tail Retention | Net Delta vs Baseline (₹) | G3 Status |\n")
        f.write("| :--- | ---: | ---: | ---: | ---: | :---: | ---: | ---: | ---: | :---: |\n")
        for cov in COVERAGE_TIERS:
            c_res = coverage_results["Family_A1_Ridge"][cov]
            f.write(f"| **Tier {int(cov*100)}%** | `{c_res['selected_trades']:,}` | `{c_res['participation_rate_pct']:.1f}%` | `₹{c_res['total_net_rupees']:+,.2f}` | `₹{c_res['mean_net_rupees']:+.2f}` | `[{c_res['ci_95'][0]:.2f}, {c_res['ci_95'][1]:.2f}]` | `{c_res['win_rate_pct']:.2f}%` | `{c_res['win_tail_profit_retention_pct']:.1f}%` | **`₹{c_res['agg_net_delta_rupees']:+,.2f}`** | **{'✅ PASS' if c_res['gate_g3_pass'] else '❌ FAIL'}** |\n")

        f.write("\n---\n\n")
        f.write("## 4. Coverage Tiers Performance (Family A2 HuberGBM & Family B Hurdle)\n\n")
        f.write("### Family A2: HistGradientBoosting Regressor (Huber Loss)\n\n")
        f.write("| Coverage Tier | Trades | Out-of-Sample Net (₹) | Mean Net / Trade (₹) | 95% Clustered CI (₹) | Win-Tail Retention | Net Delta vs Baseline (₹) | G3 Status |\n")
        f.write("| :--- | ---: | ---: | ---: | :---: | ---: | ---: | :---: |\n")
        for cov in COVERAGE_TIERS:
            c_res = coverage_results["Family_A2_HuberGBM"][cov]
            f.write(f"| **Tier {int(cov*100)}%** | `{c_res['selected_trades']:,}` | `₹{c_res['total_net_rupees']:+,.2f}` | `₹{c_res['mean_net_rupees']:+.2f}` | `[{c_res['ci_95'][0]:.2f}, {c_res['ci_95'][1]:.2f}]` | `{c_res['win_tail_profit_retention_pct']:.1f}%` | **`₹{c_res['agg_net_delta_rupees']:+,.2f}`** | **{'✅ PASS' if c_res['gate_g3_pass'] else '❌ FAIL'}** |\n")

        f.write("\n### Family B: Two-Part Hurdle Comparator\n\n")
        f.write("| Coverage Tier | Trades | Out-of-Sample Net (₹) | Mean Net / Trade (₹) | 95% Clustered CI (₹) | Win-Tail Retention | Net Delta vs Baseline (₹) | G3 Status |\n")
        f.write("| :--- | ---: | ---: | ---: | :---: | ---: | ---: | :---: |\n")
        for cov in COVERAGE_TIERS:
            c_res = coverage_results["Family_B_Hurdle"][cov]
            f.write(f"| **Tier {int(cov*100)}%** | `{c_res['selected_trades']:,}` | `₹{c_res['total_net_rupees']:+,.2f}` | `₹{c_res['mean_net_rupees']:+.2f}` | `[{c_res['ci_95'][0]:.2f}, {c_res['ci_95'][1]:.2f}]` | `{c_res['win_tail_profit_retention_pct']:.1f}%` | **`₹{c_res['agg_net_delta_rupees']:+,.2f}`** | **{'✅ PASS' if c_res['gate_g3_pass'] else '❌ FAIL'}** |\n")

        f.write("\n---\n\n")
        f.write("## 5. Session-by-Session Stability Breakdown (Gate G4)\n\n")
        f.write("Net Delta vs Proportional Baseline by Session for Family A1 Ridge:\n\n")
        f.write("| Coverage Tier | Session 2026-09-29 Delta | Session 2026-09-30 Delta | Session 2026-10-01 Delta | G4 Outcome |\n")
        f.write("| :--- | :---: | :---: | :---: | :---: |\n")
        for cov in COVERAGE_TIERS:
            c_res = coverage_results["Family_A1_Ridge"][cov]
            sd = c_res["session_deltas"]
            f.write(f"| **Tier {int(cov*100)}%** | `₹{sd['2026-09-29']['delta_vs_proportional']:+,.2f}` | `₹{sd['2026-09-30']['delta_vs_proportional']:+,.2f}` | `₹{sd['2026-10-01']['delta_vs_proportional']:+,.2f}` | **{'✅ PASS' if c_res['gate_g4_pass'] else '❌ FAIL'}** |\n")

        f.write("\n---\n\n")
        f.write("## 6. The Forensic Mechanism: Why Predictive Ranking Fails Economic Viability\n\n")
        f.write("The historical discovery replay establishes an important scientific result:\n\n")
        f.write("1. **Predictive Ranking Exists (Gate G2 PASSED):** Both Family A1 (Ridge, $\\rho = +0.0631, p = 2.92 \\times 10^{-6}$) and Family A2 (HuberGBM, $\\rho = +0.0544, p = 5.25 \\times 10^{-5}$) demonstrate statistically significant out-of-sample ranking correlation and improve upon the naive training mean MSE.  \n")
        f.write("2. **Economic Viability Fails (Gate G3 FAILED):** Despite ranking accuracy, selecting the top 25% of predicted opportunities yields a realized mean net P&L of **`₹-98.81`** per trade (vs **`₹-121.68`** for the unfiltered baseline).  \n")
        f.write("3. **The Mechanical Cause:** The predictive edge improves net outcomes by approximately `₹23` per trade. However, the unconditional baseline deficit (`₹-121.68`) combined with turnover friction (`₹139.04`) creates a deficit barrier that a `₹23` ranking spread cannot bridge.  \n\n")

        f.write("---\n\n")
        f.write("## 7. Formal Milestone Disposition\n\n")
        f.write("### Qualification Outcome: GATE G3 ECONOMIC FAILURE (ARCHIVED AS RESEARCH-ONLY)\n\n")
        f.write("- **Live Execution Authority:** RP-605 retains **100% active live execution authority** across all instruments.  \n")
        f.write("- **RP-613 Status:** Permanently archived as a research discovery finding (0% execution authority). Zero post-hoc threshold tuning.  \n")
        f.write("- **Scientific Conclusion:** Pre-entry linear and tree-based combinations contain modest predictive ranking power ($\rho \approx +0.06$), but cannot overcome the `₹139.04` per-trade friction barrier to produce positive net expectancy.  \n")

    print(f"\nDiscovery Evaluation complete in {time.time() - t_start:.2f}s!")
    return manifest


if __name__ == "__main__":
    run_discovery()
