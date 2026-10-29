#!/usr/bin/env python3
"""
RP-609 entry-quality and early-exit discrimination evaluation.

Exploratory. No promotion. No new entry filter.

Pre-specified before the later-window results are computed:

Entry experiment
- Label: retrospective zero-MFE (lifecycle MFE == 0). Not known at entry.
- F15 risk score = -feature_f15_at_entry. Staler entries score higher.
- F30 risk score = -feature_pre_freshness_30. Same direction.
- Card freshness risk score = -feature_pre_freshness_15. Same direction.
- Side-aligned 15m, 30m, and 1d scores: risk = -aligned score.
  Alignment is score for LONG and -score for SHORT.
- The only operating point is the existing frozen freshness gate:
  flag when F15 < -0.01. Thresholds are not searched.
- Multivariate model, fit only on the frozen 364 and frozen before the
  later window is scored: intercept plus F15, pre-freshness 30, and the
  side-aligned 15m score. No feature dropping after seeing AUC.
- ATR, volatility, regime, and a field named signal strength are not tested.

Early-exit experiment (separate)
- At 5, 10, and 15 minutes, the rule flags a trade when no favourable
  excursion has been observed yet.
- The outcome is favourable excursion in the next 60 minutes.
- Lifecycle zero-MFE, Initial Move, and Peak MFE are not inputs.
- Risk score for AUC is -mark-to-market P&L at the horizon.

Chronological check
- Later window: NSE Ledger A entries strictly after the frozen holdout's
  last entry. Same calendar session. The model and the gate are not refit.
"""

from __future__ import annotations

import json
from collections import defaultdict
from datetime import datetime, timezone

import numpy as np

from build_rp609_entry_time_discrimination_dataset import (
    composite_key,
    load_cards,
    load_frozen_holdout,
    load_journal,
    load_snapshot,
    materialize_records,
    parse_ts,
    rp606_index,
)

FROZEN_F15_GATE = -0.01
BOOTSTRAPS = 1000
BOOTSTRAP_SEED = 42
LATER_WINDOW_NOTE = (
    "Entries strictly after the frozen holdout's last entry on 2026-09-28. "
    "This is a later slice of the same NSE session, not a new trading day."
)

OUT_JSON_NAME = "rp609_entry_discrimination_evaluation.json"
OUT_MD_NAME = "rp609_entry_discrimination_evaluation.md"


def auc(y_true, y_scores):
    y_true = np.asarray(y_true, dtype=int)
    y_scores = np.asarray(y_scores, dtype=float)
    if len(y_true) < 2 or len(np.unique(y_true)) < 2:
        return None
    order = np.argsort(y_scores, kind="mergesort")[::-1]
    y_true = y_true[order]
    y_scores = y_scores[order]
    distinct = np.where(np.diff(y_scores))[0]
    thresholds = np.r_[distinct, y_true.size - 1]
    tps = np.cumsum(y_true)[thresholds]
    fps = 1 + thresholds - tps
    tps = np.r_[0, tps]
    fps = np.r_[0, fps]
    if fps[-1] == 0 or tps[-1] == 0:
        return None
    fpr = fps / fps[-1]
    tpr = tps / tps[-1]
    return float(np.trapezoid(tpr, fpr))


def cluster_auc_ci(rows, y, scores, n_boot=BOOTSTRAPS, seed=BOOTSTRAP_SEED):
    point = auc(y, scores)
    clusters = defaultdict(list)
    for index, row in enumerate(rows):
        clusters[row["symbol"]].append(index)
    keys = list(clusters)
    sizes = sorted(len(members) for members in clusters.values())
    cluster_summary = {
        "clusters": len(keys),
        "trades_per_cluster_min": sizes[0] if sizes else 0,
        "trades_per_cluster_median": sizes[len(sizes) // 2] if sizes else 0,
        "trades_per_cluster_max": sizes[-1] if sizes else 0,
    }
    if point is None or len(keys) < 2:
        return {
            "auc": point,
            "ci95": None,
            "bootstraps_requested": n_boot,
            "bootstraps_used": 0,
            "bootstraps_dropped_one_class": 0,
            "clusters": len(keys),
            "cluster_summary": cluster_summary,
        }
    rng = np.random.default_rng(seed)
    draws = []
    dropped = 0
    y_arr = np.asarray(y)
    s_arr = np.asarray(scores)
    for _ in range(n_boot):
        chosen = rng.choice(len(keys), size=len(keys), replace=True)
        idxs = []
        for key_index in chosen:
            idxs.extend(clusters[keys[key_index]])
        estimate = auc(y_arr[idxs], s_arr[idxs])
        if estimate is not None:
            draws.append(estimate)
        else:
            dropped += 1
    if not draws:
        return {
            "auc": round(point, 4),
            "ci95": None,
            "bootstraps_requested": n_boot,
            "bootstraps_used": 0,
            "bootstraps_dropped_one_class": dropped,
            "clusters": len(keys),
            "cluster_summary": cluster_summary,
        }
    low, high = np.percentile(draws, [2.5, 97.5])
    return {
        "auc": round(point, 4),
        "ci95": [round(float(low), 4), round(float(high), 4)],
        "bootstraps_requested": n_boot,
        "bootstraps_used": len(draws),
        "bootstraps_dropped_one_class": dropped,
        "clusters": len(keys),
        "cluster_summary": cluster_summary,
        "ci_excludes_0_5": bool(low > 0.5 or high < 0.5),
    }


def numeric_rows(rows, column, transform):
    kept = []
    values = []
    labels = []
    for row in rows:
        raw = row.get(column)
        if raw is None or raw == "":
            continue
        score = transform(row, float(raw))
        if score is None:
            continue
        kept.append(row)
        values.append(score)
        labels.append(int(row["label_zero_mfe"]))
    return kept, labels, values


def aligned_score(row, raw):
    return raw if row["side"] == "LONG" else -raw


def identity(row, raw):
    return raw


def negated(row, raw):
    return -raw


def gate_metrics(rows, flag_fn):
    flagged = []
    kept = []
    for row in rows:
        if row.get("feature_f15_at_entry") is None:
            continue
        if flag_fn(row):
            flagged.append(row)
        else:
            kept.append(row)
    zero_all = [row for row in flagged + kept if int(row["label_zero_mfe"]) == 1]
    pos_all = [row for row in flagged + kept if int(row["label_zero_mfe"]) == 0]
    zero_flagged = [row for row in flagged if int(row["label_zero_mfe"]) == 1]
    pos_flagged = [row for row in flagged if int(row["label_zero_mfe"]) == 0]

    def pnl(group):
        return round(sum(float(row["label_realized_pnl"] or 0.0) for row in group), 2)

    n_flagged = len(flagged)
    n_zero = len(zero_all)
    n_pos = len(pos_all)
    return {
        "eligible": len(flagged) + len(kept),
        "flagged": n_flagged,
        "flag_rate": round(n_flagged / (n_flagged + len(kept)), 4) if flagged or kept else None,
        "precision_zero_mfe": round(len(zero_flagged) / n_flagged, 4) if n_flagged else None,
        "recall_zero_mfe": round(len(zero_flagged) / n_zero, 4) if n_zero else None,
        "profitable_movers_incorrectly_filtered": round(len(pos_flagged) / n_pos, 4) if n_pos else None,
        "positive_mfe_flagged": len(pos_flagged),
        "positive_mfe_total": n_pos,
        "zero_mfe_flagged": len(zero_flagged),
        "zero_mfe_total": n_zero,
        "realized_pnl_flagged": pnl(flagged),
        "realized_pnl_kept": pnl(kept),
        "base_rate_zero_mfe": round(n_zero / (n_zero + n_pos), 4) if (n_zero + n_pos) else None,
    }


def calibration_by_quintile(labels, scores):
    labels = np.asarray(labels, dtype=float)
    scores = np.asarray(scores, dtype=float)
    if len(labels) < 5 or len(np.unique(scores)) < 2:
        return []
    order = np.argsort(scores, kind="mergesort")
    folds = np.array_split(order, 5)
    table = []
    for fold_index, idxs in enumerate(folds, start=1):
        if len(idxs) == 0:
            continue
        table.append({
            "bin": fold_index,
            "n": int(len(idxs)),
            "mean_risk_score": round(float(scores[idxs].mean()), 4),
            "observed_zero_mfe_rate": round(float(labels[idxs].mean()), 4),
        })
    return table


def fit_logistic(x_matrix, y):
    x_matrix = np.asarray(x_matrix, dtype=float)
    y = np.asarray(y, dtype=float)
    n_obs, n_features = x_matrix.shape
    beta = np.zeros(n_features)
    for _ in range(30):
        eta = np.clip(x_matrix @ beta, -20, 20)
        probability = 1.0 / (1.0 + np.exp(-eta))
        weight = np.maximum(probability * (1.0 - probability), 1e-6)
        working = eta + (y - probability) / weight
        weighted = x_matrix * weight[:, None]
        hessian = x_matrix.T @ weighted + 1e-8 * np.eye(n_features)
        updated = np.linalg.solve(hessian, x_matrix.T @ (weight * working))
        if np.max(np.abs(updated - beta)) < 1e-8:
            beta = updated
            break
        beta = updated
    return beta


def predict_logistic(x_matrix, beta):
    eta = np.clip(np.asarray(x_matrix, dtype=float) @ beta, -20, 20)
    return 1.0 / (1.0 + np.exp(-eta))


def model_matrix(rows, means=None, stds=None):
    raw = []
    kept = []
    for row in rows:
        f15 = row.get("feature_f15_at_entry")
        f30 = row.get("feature_pre_freshness_30")
        score = row.get("feature_entry_15m_score")
        if f15 is None or f30 is None or score is None:
            continue
        aligned = aligned_score(row, float(score))
        raw.append([float(f15), float(f30), aligned])
        kept.append(row)
    if not raw:
        return kept, None, None, None, None
    matrix = np.asarray(raw, dtype=float)
    if means is None:
        means = matrix.mean(axis=0)
        stds = matrix.std(axis=0)
        stds = np.where(stds < 1e-8, 1.0, stds)
    scaled = (matrix - means) / stds
    design = np.column_stack([np.ones(len(scaled)), scaled])
    labels = np.asarray([int(row["label_zero_mfe"]) for row in kept], dtype=float)
    return kept, design, labels, means, stds


def brier(labels, probabilities):
    labels = np.asarray(labels, dtype=float)
    probabilities = np.asarray(probabilities, dtype=float)
    return round(float(np.mean((probabilities - labels) ** 2)), 4)


def probability_calibration(labels, probabilities):
    labels = np.asarray(labels, dtype=float)
    probabilities = np.asarray(probabilities, dtype=float)
    order = np.argsort(probabilities, kind="mergesort")
    folds = np.array_split(order, 5)
    table = []
    for fold_index, idxs in enumerate(folds, start=1):
        if len(idxs) == 0:
            continue
        table.append({
            "bin": fold_index,
            "n": int(len(idxs)),
            "mean_predicted": round(float(probabilities[idxs].mean()), 4),
            "observed_zero_mfe_rate": round(float(labels[idxs].mean()), 4),
        })
    return table


def entry_feature_report(rows):
    specs = [
        ("F15 journal freshness", "feature_f15_at_entry", negated, "Staler F15 (more negative) has higher zero-MFE risk."),
        ("F30 pre-entry freshness", "feature_pre_freshness_30", negated, "Staler 30m pre-entry freshness has higher zero-MFE risk."),
        ("F15 card freshness", "feature_pre_freshness_15", negated, "Staler 15m card freshness has higher zero-MFE risk."),
        ("Side-aligned 15m score", "feature_entry_15m_score", lambda row, raw: -aligned_score(row, raw), "Weaker side-aligned 15m score has higher zero-MFE risk."),
        ("Side-aligned 30m score", "feature_entry_30m_score", lambda row, raw: -aligned_score(row, raw), "Weaker side-aligned 30m score has higher zero-MFE risk."),
        ("Side-aligned 1d score", "feature_entry_1d_score", lambda row, raw: -aligned_score(row, raw), "Weaker side-aligned 1d score has higher zero-MFE risk."),
    ]
    reports = []
    for name, column, transform, direction in specs:
        kept, labels, scores = numeric_rows(rows, column, transform)
        estimate = cluster_auc_ci(kept, labels, scores)
        reports.append({
            "feature": name,
            "column": column,
            "direction": direction,
            "n": len(kept),
            "auc": estimate,
            "calibration_quintiles_of_risk": calibration_by_quintile(labels, scores),
        })
    return reports


def early_exit_report(rows):
    reports = []
    for horizon in (5, 10, 15):
        seen_col = f"obs_{horizon}m_favourable_seen"
        later_col = f"outcome_{horizon}m_later_favourable"
        mtm_col = f"obs_{horizon}m_mtm_pnl"
        eligible = [
            row for row in rows
            if row.get(seen_col) is not None and row.get(later_col) is not None and row.get(mtm_col) is not None
        ]
        flagged = [row for row in eligible if int(row[seen_col]) == 0]
        quiet = [row for row in eligible if int(row[later_col]) == 0]
        flagged_quiet = [row for row in flagged if int(row[later_col]) == 0]
        flagged_later = [row for row in flagged if int(row[later_col]) == 1]
        positive = [row for row in eligible if int(row["label_zero_mfe"]) == 0]
        positive_flat = [row for row in positive if int(row[seen_col]) == 0]
        y = [1 - int(row[later_col]) for row in eligible]
        scores = [-float(row[mtm_col]) for row in eligible]
        estimate = cluster_auc_ci(eligible, y, scores)
        reports.append({
            "horizon_min": horizon,
            "n": len(eligible),
            "flagged_no_favourable_yet": len(flagged),
            "precision_no_later_favourable": round(len(flagged_quiet) / len(flagged), 4) if flagged else None,
            "recall_no_later_favourable": round(len(flagged_quiet) / len(quiet), 4) if quiet else None,
            "later_favourable_closed_early": round(len(flagged_later) / len(flagged), 4) if flagged else None,
            "later_favourable_count_among_flagged": len(flagged_later),
            "lifecycle_positive_mfe_still_flat": len(positive_flat),
            "lifecycle_positive_mfe": len(positive),
            "profitable_movers_incorrectly_filtered": round(len(positive_flat) / len(positive), 4) if positive else None,
            "auc_adverse_mtm_predicts_no_later_favourable": estimate,
        })
    return reports


def later_records(nse_a, holdout):
    hold_keys = {composite_key(row) for row in holdout}
    hold_max = max(parse_ts(row["t_entry"]) for row in holdout)
    selected = []
    seen = set()
    for row in nse_a:
        key = composite_key(row)
        if key in hold_keys or key in seen:
            continue
        if parse_ts(row["t_entry"]) <= hold_max:
            continue
        seen.add(key)
        selected.append(row)
    return selected, hold_max


def population_blurb(rows):
    zero = [row for row in rows if int(row["label_zero_mfe"]) == 1]
    positive = [row for row in rows if int(row["label_zero_mfe"]) == 0]
    return {
        "rows": len(rows),
        "joined_f15": sum(1 for row in rows if row.get("feature_f15_at_entry") is not None),
        "zero_mfe": len(zero),
        "positive_mfe": len(positive),
        "zero_mfe_rate": round(len(zero) / len(rows), 4) if rows else None,
        "realized_pnl": round(sum(float(row["label_realized_pnl"] or 0.0) for row in rows), 2),
        "symbols": len({row["symbol"] for row in rows}),
    }


def markdown(report):
    lines = [
        "# RP-609 Entry and Early-Exit Discrimination",
        "",
        "**Disposition:** Exploratory. No promotion. No entry filter. PFT, SPL, VAT, and PPR are unchanged. RP-605 stays the frozen control.",
        "",
        "The frozen 364-trade holdout is the cohort already described by the zero-MFE forensic profile. Numbers on that cohort describe association. The later window is scored with the same pre-specified direction, the same frozen F15 gate, and the logistic coefficients fit only on the 364.",
        "",
        f"Later window: {report['later_window']['definition']}",
        "",
        "## Populations",
        "",
        f"- Frozen holdout: {report['holdout']['population']}",
        f"- Later window: {report['later_window']['population']}",
        "",
        "## Entry features on the frozen holdout",
        "",
        "AUC uses the pre-specified risk direction. A value below 0.50 means that direction does not rank zero-MFE trades above positive-MFE trades. The interval is a ticker-cluster bootstrap.",
        "",
        "| Feature | N | AUC | 95% CI | CI excludes 0.50 |",
        "| :--- | ---: | ---: | :--- | :--- |",
    ]
    for item in report["holdout"]["univariate"]:
        ci = item["auc"]["ci95"]
        ci_text = "n/a" if not ci else f"[{ci[0]:.3f}, {ci[1]:.3f}]"
        lines.append(
            f"| {item['feature']} | {item['n']} | {item['auc']['auc']:.3f} | {ci_text} | {item['auc']['ci_excludes_0_5']} |"
        )
    gate = report["holdout"]["frozen_f15_gate"]
    lines.extend([
        "",
        "## Frozen F15 gate (F15 < -0.01)",
        "",
        "This gate already exists on the freshness ledger. It is not a new threshold.",
        "",
        "| Cohort | Flagged | Precision | Recall | Positive-MFE filtered | Base rate |",
        "| :--- | ---: | ---: | ---: | ---: | ---: |",
    ])
    for label, block in (
        ("Frozen holdout", report["holdout"]["frozen_f15_gate"]),
        ("Later window", report["later_window"]["frozen_f15_gate"]),
    ):
        lines.append(
            f"| {label} | {block['flagged']} / {block['eligible']} | {block['precision_zero_mfe']} | {block['recall_zero_mfe']} | {block['profitable_movers_incorrectly_filtered']} | {block['base_rate_zero_mfe']} |"
        )
    lines.extend([
        "",
        f"Holdout realized P&L of flagged trades: {gate['realized_pnl_flagged']}. Kept trades: {gate['realized_pnl_kept']}.",
        f"Later-window realized P&L of flagged trades: {report['later_window']['frozen_f15_gate']['realized_pnl_flagged']}. Kept trades: {report['later_window']['frozen_f15_gate']['realized_pnl_kept']}.",
        "",
        "## Pre-specified logistic model",
        "",
        "Fit on the frozen holdout only. Features: F15, pre-entry 30m freshness, side-aligned 15m score. Coefficients were not refit on the later window.",
        "",
        f"- In-sample AUC on the fit cohort: {report['model']['insample_auc']}",
        f"- Later-window AUC: {report['model']['later_auc']}",
        f"- Later-window Brier score: {report['model']['later_brier']}",
        f"- Rows used: fit {report['model']['fit_n']}, later {report['model']['later_n']}",
        "",
        "Later-window calibration (mean predicted probability vs observed zero-MFE rate):",
        "",
        "| Bin | N | Predicted | Observed |",
        "| :--- | ---: | ---: | ---: |",
    ])
    for row in report["model"]["later_calibration"]:
        lines.append(
            f"| {row['bin']} | {row['n']} | {row['mean_predicted']:.3f} | {row['observed_zero_mfe_rate']:.3f} |"
        )
    lines.extend([
        "",
        "## Early-exit rule: no favourable price yet",
        "",
        "A flagged trade has no favourable excursion by the horizon. Precision is the share of those trades that also stay unfavourable for the next 60 minutes. The last column is the share of lifecycle positive-MFE trades that are still flat at the horizon.",
        "",
        "| Cohort | Horizon | N | Precision | Recall | Later favourable if closed | Positive-MFE still flat | AUC of adverse mark |",
        "| :--- | ---: | ---: | ---: | ---: | ---: | ---: | :--- |",
    ])
    for cohort_name, block in (
        ("Holdout", report["holdout"]["early_exit"]),
        ("Later window", report["later_window"]["early_exit"]),
    ):
        for item in block:
            estimate = item["auc_adverse_mtm_predicts_no_later_favourable"]
            ci = estimate["ci95"]
            auc_text = "n/a" if estimate["auc"] is None else f"{estimate['auc']:.3f}"
            if ci:
                auc_text += f" [{ci[0]:.3f}, {ci[1]:.3f}]"
            lines.append(
                f"| {cohort_name} | {item['horizon_min']} | {item['n']} | {item['precision_no_later_favourable']} | {item['recall_no_later_favourable']} | {item['later_favourable_closed_early']} | {item['profitable_movers_incorrectly_filtered']} | {auc_text} |"
            )
    lines.extend([
        "",
        "## Not tested",
        "",
        "- ATR, volatility, market regime, and a field named signal strength. Those columns are empty.",
        "- Time of day. It stays a descriptive context column.",
        "- A new trading day. The later window is the afternoon of 2026-09-28.",
        "",
        "No rule from this evaluation is authorized for live entries or for a change to RP-605.",
        "",
    ])
    return "\n".join(lines)


def main():
    from pathlib import Path

    snapshot, _sha = load_snapshot()
    nse_a, holdout_records, missing_ids, pnl_mismatches = load_frozen_holdout(snapshot)
    if missing_ids or pnl_mismatches:
        raise SystemExit(f"Frozen holdout failed reconciliation: missing={len(missing_ids)} pnl={len(pnl_mismatches)}")
    journal = load_journal()
    freshness = rp606_index(snapshot)
    cards = load_cards()
    holdout_rows, _hold_integrity = materialize_records(holdout_records, journal, freshness, cards)
    selected, hold_max = later_records(nse_a, holdout_records)
    later_rows, later_integrity = materialize_records(selected, journal, freshness, cards)

    hold_kept, hold_design, hold_y, means, stds = model_matrix(holdout_rows)
    beta = fit_logistic(hold_design, hold_y)
    hold_prob = predict_logistic(hold_design, beta)
    later_kept, later_design, later_y, _means, _stds = model_matrix(later_rows, means, stds)
    later_prob = predict_logistic(later_design, beta)

    stale = lambda row: float(row["feature_f15_at_entry"]) < FROZEN_F15_GATE
    report = {
        "disposition": "EXPLORATORY — no promotion",
        "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC"),
        "bootstrap": {"replicates": BOOTSTRAPS, "seed": BOOTSTRAP_SEED, "cluster": "symbol"},
        "not_tested": ["atr", "volatility", "regime", "signal_strength", "time_of_day_filter", "new_trading_day"],
        "holdout": {
            "population": population_blurb(holdout_rows),
            "univariate": entry_feature_report(holdout_rows),
            "frozen_f15_gate": gate_metrics(holdout_rows, stale),
            "early_exit": early_exit_report(holdout_rows),
        },
        "later_window": {
            "definition": LATER_WINDOW_NOTE,
            "holdout_last_entry_utc": hold_max.isoformat(),
            "population": population_blurb(later_rows),
            "missing_journal": len(later_integrity["missing_journal"]),
            "missing_entry_card": len(later_integrity["missing_entry_card"]),
            "univariate": entry_feature_report(later_rows),
            "frozen_f15_gate": gate_metrics(later_rows, stale),
            "early_exit": early_exit_report(later_rows),
        },
        "model": {
            "features": [
                "feature_f15_at_entry",
                "feature_pre_freshness_30",
                "side_aligned_feature_entry_15m_score",
            ],
            "fit_cohort": "frozen 364 holdout",
            "fit_n": len(hold_kept),
            "later_n": len(later_kept),
            "coefficients_intercept_then_standardized_features": [round(float(value), 4) for value in beta],
            "insample_auc": cluster_auc_ci(hold_kept, hold_y.astype(int), hold_prob),
            "later_auc": cluster_auc_ci(later_kept, later_y.astype(int), later_prob),
            "later_brier": brier(later_y, later_prob),
            "later_calibration": probability_calibration(later_y, later_prob),
        },
    }

    out_dir = Path(__file__).resolve().parent.parent / "live_capture" / "shadow_validation"
    (out_dir / OUT_JSON_NAME).write_text(json.dumps(report, indent=2), encoding="utf-8")
    (out_dir / OUT_MD_NAME).write_text(markdown(report), encoding="utf-8")
    print(json.dumps({
        "holdout_rows": report["holdout"]["population"]["rows"],
        "later_rows": report["later_window"]["population"]["rows"],
        "later_joined_f15": report["later_window"]["population"]["joined_f15"],
        "holdout_f15_auc": report["holdout"]["univariate"][0]["auc"],
        "later_f15_auc": report["later_window"]["univariate"][0]["auc"],
        "holdout_gate": report["holdout"]["frozen_f15_gate"],
        "later_gate": report["later_window"]["frozen_f15_gate"],
        "model_later_auc": report["model"]["later_auc"],
        "model_later_brier": report["model"]["later_brier"],
    }, indent=2))


if __name__ == "__main__":
    main()
