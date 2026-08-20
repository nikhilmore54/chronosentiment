#!/usr/bin/env python3
"""
TIME-008 — Discrimination Analysis
Reads time_machine/cohorts/aggregate_evidence.csv and produces Q1-Q4 analysis
artifacts in time_machine/analysis/TIME008/, following exactly the questions
and thresholds defined in docs/TIME008_ANALYSIS_SPEC.md.

Governing rule: analysis-only. No changes to any upstream pipeline.
"""

import csv
import json
import math
import os
import sys
from collections import defaultdict
from datetime import datetime, timezone

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
INPUT_CSV = os.path.join(PROJECT_ROOT, "time_machine", "cohorts", "aggregate_evidence.csv")
OUTPUT_DIR = os.path.join(PROJECT_ROOT, "time_machine", "analysis", "TIME008")

EXPECTED_ROWS = 612
EXPECTED_COHORTS = {"T1", "T2", "T3", "T4", "T5", "T6"}
EXPECTED_ROWS_PER_COHORT = 102
CONSISTENCY_THRESHOLD = 4   # >=4/6 cohorts for "consistent"
INCONSISTENCY_THRESHOLD = 3  # >=3/6 reversals for "inconsistent"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def parse_bool(s):
    return s.strip().lower() == "true"

def parse_float(s):
    try:
        return float(s.strip())
    except (ValueError, AttributeError):
        return float("nan")

def mean(values):
    vals = [v for v in values if not (isinstance(v, float) and math.isnan(v))]
    if not vals:
        return None
    return sum(vals) / len(vals)

def rate_true(bools):
    if not bools:
        return None
    return sum(1 for b in bools if b is True) / len(bools)

def tercile_boundaries(values):
    sorted_vals = sorted(v for v in values if not math.isnan(v))
    n = len(sorted_vals)
    if n < 3:
        return None, None
    low_upper = sorted_vals[n // 3]
    high_lower = sorted_vals[(2 * n) // 3]
    return low_upper, high_lower

def tercile_label(value, low_upper, high_lower):
    if math.isnan(value):
        return None
    if value <= low_upper:
        return "low"
    elif value <= high_lower:
        return "medium"
    else:
        return "high"

def summarise_group(rows):
    eligible = [r for r in rows if r["eligible_for_primary_comparison"]]
    full = rows

    target_reached_eligible = [r["target_reached"] for r in eligible]
    risk_reached_eligible = [r["risk_reached"] for r in eligible]
    realized_return_eligible = [r["realized_return"] for r in eligible]

    actual_mfe_full = [r["actual_mfe"] for r in full]
    actual_mae_full = [r["actual_mae"] for r in full]
    exit_reason_counts = defaultdict(int)
    for r in full:
        exit_reason_counts[r["exit_reason"]] += 1

    return {
        "n_total": len(full),
        "n_eligible": len(eligible),
        "primary": {
            "target_reached_rate": rate_true(target_reached_eligible),
            "risk_reached_rate": rate_true(risk_reached_eligible),
            "realized_return_mean": mean(realized_return_eligible),
        },
        "secondary": {
            "actual_mfe_mean": mean(actual_mfe_full),
            "actual_mae_mean": mean(actual_mae_full),
            "exit_reason_counts": dict(exit_reason_counts),
        },
    }

def consistency_verdict(n_hold, total_cohorts=6):
    if n_hold >= CONSISTENCY_THRESHOLD:
        return "consistent"
    elif (total_cohorts - n_hold) >= INCONSISTENCY_THRESHOLD:
        return "inconsistent"
    else:
        return "no_detectable_pattern"

def ordering_holds(class_stats, ordering, metric):
    """Check if ordering (descending) holds for metric across class_stats dict."""
    vals = []
    for cls in ordering:
        if cls not in class_stats:
            return False
        v = class_stats[cls]["primary"].get(metric)
        if v is None:
            return False
        vals.append(v)
    for i in range(len(vals) - 1):
        if vals[i] < vals[i + 1]:
            return False
    return True

# ---------------------------------------------------------------------------
# Load and validate data
# ---------------------------------------------------------------------------

def load_data():
    rows = []
    seen_ids = set()
    seen_cohort_ticker = set()

    with open(INPUT_CSV, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            eid = row["evidence_row_id"].strip()
            cohort = row["cohort_label"].strip()
            ticker = row["ticker"].strip()

            assert eid not in seen_ids, f"Duplicate evidence_row_id: {eid}"
            seen_ids.add(eid)

            ct_key = (cohort, ticker)
            assert ct_key not in seen_cohort_ticker, f"Duplicate (cohort, ticker): {ct_key}"
            seen_cohort_ticker.add(ct_key)

            parsed = {
                "evidence_row_id": eid,
                "cohort_label": cohort,
                "as_of_cohort": row["as_of_cohort"].strip(),
                "as_of": row["as_of"].strip(),
                "ticker": ticker,
                "direction": row["direction"].strip(),
                "action": row["action"].strip(),
                "evidence_class": row["evidence_class"].strip(),
                "target_rate": parse_float(row["target_rate"]),
                "sample_size": int(row["sample_size"].strip()),
                "degradation_level": row["degradation_level"].strip(),
                "adaptive_rr": parse_float(row["adaptive_rr"]),
                "adaptive_horizon_sessions": parse_float(row["adaptive_horizon_sessions"]),
                "reference_price": parse_float(row["reference_price"]),
                "adaptive_target": parse_float(row["adaptive_target"]),
                "adaptive_risk": parse_float(row["adaptive_risk"]),
                "exit_reason": row["exit_reason"].strip(),
                "sessions_to_outcome": parse_float(row["sessions_to_outcome"]),
                "target_reached": parse_bool(row["target_reached"]),
                "risk_reached": parse_bool(row["risk_reached"]),
                "horizon_reached": parse_bool(row["horizon_reached"]),
                "actual_mfe": parse_float(row["actual_mfe"]),
                "actual_mae": parse_float(row["actual_mae"]),
                "realized_return": parse_float(row["realized_return"]),
                "eligible_for_primary_comparison": parse_bool(row["eligible_for_primary_comparison"]),
            }
            rows.append(parsed)

    assert len(rows) == EXPECTED_ROWS, f"Expected {EXPECTED_ROWS} rows, got {len(rows)}"
    cohorts_present = set(r["cohort_label"] for r in rows)
    assert cohorts_present == EXPECTED_COHORTS, f"Cohorts mismatch: {cohorts_present}"
    for cohort in EXPECTED_COHORTS:
        n = sum(1 for r in rows if r["cohort_label"] == cohort)
        assert n == EXPECTED_ROWS_PER_COHORT, f"Cohort {cohort}: expected {EXPECTED_ROWS_PER_COHORT} rows, got {n}"

    print(f"[time008] Loaded {len(rows)} rows. Invariants OK.")
    return rows

# ---------------------------------------------------------------------------
# Q1 — Evidence-class discrimination
# ---------------------------------------------------------------------------

def run_q1(rows):
    ORDERING_POSITIVE = ["Favourable", "Mixed", "Unfavourable"]
    ORDERING_RISK = ["Unfavourable", "Mixed", "Favourable"]

    def analyse_by_class(subset_rows):
        by_class = defaultdict(list)
        for r in subset_rows:
            by_class[r["evidence_class"]].append(r)
        return {cls: summarise_group(cls_rows) for cls, cls_rows in by_class.items()}

    pooled_stats = analyse_by_class(rows)

    cohort_stats = {}
    for cohort in sorted(EXPECTED_COHORTS):
        cohort_rows = [r for r in rows if r["cohort_label"] == cohort]
        cohort_stats[cohort] = analyse_by_class(cohort_rows)

    def check_ordering(metric, ordering):
        n_hold = 0
        cohort_results = {}
        for cohort in sorted(EXPECTED_COHORTS):
            holds = ordering_holds(cohort_stats[cohort], ordering, metric)
            cohort_results[cohort] = holds
            if holds:
                n_hold += 1
        return {
            "ordering": ordering,
            "metric": metric,
            "cohorts_holding": n_hold,
            "cohort_detail": cohort_results,
            "verdict": consistency_verdict(n_hold),
        }

    consistency = {
        "target_reached_Fav_gt_Mix_gt_Unf": check_ordering("target_reached_rate", ORDERING_POSITIVE),
        "risk_reached_Unf_gt_Mix_gt_Fav": check_ordering("risk_reached_rate", ORDERING_RISK),
        "realized_return_Fav_gt_Mix_gt_Unf": check_ordering("realized_return_mean", ORDERING_POSITIVE),
    }

    return {
        "question": "Q1 — Evidence-class discrimination",
        "pooled": pooled_stats,
        "per_cohort": cohort_stats,
        "consistency": consistency,
    }

# ---------------------------------------------------------------------------
# Q2 — Direction asymmetry
# ---------------------------------------------------------------------------

def run_q2(rows):
    ORDERING_POSITIVE = ["Favourable", "Mixed", "Unfavourable"]
    ORDERING_RISK = ["Unfavourable", "Mixed", "Favourable"]

    def analyse_by_class(subset_rows):
        by_class = defaultdict(list)
        for r in subset_rows:
            by_class[r["evidence_class"]].append(r)
        return {cls: summarise_group(cls_rows) for cls, cls_rows in by_class.items()}

    directions = ["LONG", "SHORT"]
    pooled_by_direction = {}
    cohort_by_direction = {}

    for direction in directions:
        dir_rows = [r for r in rows if r["direction"] == direction]
        pooled_by_direction[direction] = analyse_by_class(dir_rows)
        cohort_by_direction[direction] = {}
        for cohort in sorted(EXPECTED_COHORTS):
            cohort_dir_rows = [r for r in dir_rows if r["cohort_label"] == cohort]
            cohort_by_direction[direction][cohort] = analyse_by_class(cohort_dir_rows)

    def check_ordering_direction(direction, metric, ordering):
        n_hold = 0
        cohort_results = {}
        for cohort in sorted(EXPECTED_COHORTS):
            holds = ordering_holds(cohort_by_direction[direction][cohort], ordering, metric)
            cohort_results[cohort] = holds
            if holds:
                n_hold += 1
        return {
            "direction": direction,
            "ordering": ordering,
            "metric": metric,
            "cohorts_holding": n_hold,
            "cohort_detail": cohort_results,
            "verdict": consistency_verdict(n_hold),
        }

    consistency = {}
    for direction in directions:
        consistency[direction] = {
            "target_reached_Fav_gt_Mix_gt_Unf": check_ordering_direction(direction, "target_reached_rate", ORDERING_POSITIVE),
            "risk_reached_Unf_gt_Mix_gt_Fav": check_ordering_direction(direction, "risk_reached_rate", ORDERING_RISK),
            "realized_return_Fav_gt_Mix_gt_Unf": check_ordering_direction(direction, "realized_return_mean", ORDERING_POSITIVE),
        }

    direction_counts = {d: sum(1 for r in rows if r["direction"] == d) for d in directions}

    return {
        "question": "Q2 — Direction asymmetry",
        "direction_counts": direction_counts,
        "pooled_by_direction": pooled_by_direction,
        "cohort_by_direction": cohort_by_direction,
        "consistency": consistency,
    }

# ---------------------------------------------------------------------------
# Q3 — Action vs underlying decision
# ---------------------------------------------------------------------------

def run_q3(rows):
    evidence_classes = ["Favourable", "Mixed", "Unfavourable", "Insufficient"]

    def analyse_action_within_class(cls_rows):
        by_action = defaultdict(list)
        for r in cls_rows:
            by_action[r["action"]].append(r)
        return {action: summarise_group(action_rows) for action, action_rows in by_action.items()}

    pooled = {}
    for cls in evidence_classes:
        cls_rows = [r for r in rows if r["evidence_class"] == cls]
        if cls_rows:
            pooled[cls] = analyse_action_within_class(cls_rows)

    per_cohort = {}
    for cohort in sorted(EXPECTED_COHORTS):
        cohort_rows = [r for r in rows if r["cohort_label"] == cohort]
        per_cohort[cohort] = {}
        for cls in evidence_classes:
            cls_rows = [r for r in cohort_rows if r["evidence_class"] == cls]
            if cls_rows:
                per_cohort[cohort][cls] = analyse_action_within_class(cls_rows)

    def check_action_comparison(cls, action_a, action_b, metric):
        n_hold = 0
        cohort_results = {}
        for cohort in sorted(EXPECTED_COHORTS):
            cls_data = per_cohort[cohort].get(cls, {})
            a_stats = cls_data.get(action_a)
            b_stats = cls_data.get(action_b)
            if a_stats is None or b_stats is None:
                cohort_results[cohort] = None
                continue
            a_val = a_stats["primary"].get(metric)
            b_val = b_stats["primary"].get(metric)
            if a_val is None or b_val is None:
                cohort_results[cohort] = None
                continue
            holds = a_val > b_val
            cohort_results[cohort] = holds
            if holds:
                n_hold += 1
        valid_cohorts = sum(1 for v in cohort_results.values() if v is not None)
        return {
            "evidence_class": cls,
            "comparison": f"{action_a} > {action_b}",
            "metric": metric,
            "cohorts_holding": n_hold,
            "cohorts_with_data": valid_cohorts,
            "cohort_detail": cohort_results,
            "verdict": consistency_verdict(n_hold, total_cohorts=valid_cohorts) if valid_cohorts >= 4 else "insufficient_data",
        }

    specific_comparisons = {
        "Favourable_Buy_gt_Watch_target": check_action_comparison("Favourable", "Buy", "Watch", "target_reached_rate"),
        "Mixed_Watch_gt_NoTrade_target": check_action_comparison("Mixed", "Watch", "NoTrade", "target_reached_rate"),
        "Favourable_Buy_gt_Watch_return": check_action_comparison("Favourable", "Buy", "Watch", "realized_return_mean"),
        "Mixed_Watch_gt_NoTrade_return": check_action_comparison("Mixed", "Watch", "NoTrade", "realized_return_mean"),
    }

    action_counts = defaultdict(int)
    for r in rows:
        action_counts[r["action"]] += 1

    return {
        "question": "Q3 — Action vs underlying decision",
        "action_counts": dict(action_counts),
        "pooled_by_class_then_action": pooled,
        "per_cohort": per_cohort,
        "specific_comparisons": specific_comparisons,
    }

# ---------------------------------------------------------------------------
# Q4 — R:R interaction
# ---------------------------------------------------------------------------

def run_q4(rows):
    ORDERING_POSITIVE = ["Favourable", "Mixed", "Unfavourable"]
    ORDERING_RISK = ["Unfavourable", "Mixed", "Favourable"]

    all_rr = [r["adaptive_rr"] for r in rows]
    low_upper, high_lower = tercile_boundaries(all_rr)
    print(f"[time008] Q4 adaptive_rr tercile boundaries: low<={low_upper:.6f}, medium<={high_lower:.6f}, high>{high_lower:.6f}")

    for r in rows:
        r["rr_tercile"] = tercile_label(r["adaptive_rr"], low_upper, high_lower)

    tercile_labels = ["low", "medium", "high"]

    def analyse_by_class(subset_rows):
        by_class = defaultdict(list)
        for r in subset_rows:
            by_class[r["evidence_class"]].append(r)
        return {cls: summarise_group(cls_rows) for cls, cls_rows in by_class.items()}

    pooled_by_tercile = {}
    for tercile in tercile_labels:
        t_rows = [r for r in rows if r["rr_tercile"] == tercile]
        pooled_by_tercile[tercile] = {
            "n": len(t_rows),
            "by_evidence_class": analyse_by_class(t_rows),
        }

    per_cohort_by_tercile = {}
    for cohort in sorted(EXPECTED_COHORTS):
        cohort_rows = [r for r in rows if r["cohort_label"] == cohort]
        per_cohort_by_tercile[cohort] = {}
        for tercile in tercile_labels:
            t_rows = [r for r in cohort_rows if r["rr_tercile"] == tercile]
            per_cohort_by_tercile[cohort][tercile] = {
                "n": len(t_rows),
                "by_evidence_class": analyse_by_class(t_rows),
            }

    def check_ordering_in_tercile(tercile, metric, ordering):
        n_hold = 0
        cohort_results = {}
        for cohort in sorted(EXPECTED_COHORTS):
            class_stats = per_cohort_by_tercile[cohort][tercile]["by_evidence_class"]
            holds = ordering_holds(class_stats, ordering, metric)
            cohort_results[cohort] = holds
            if holds:
                n_hold += 1
        return {
            "tercile": tercile,
            "ordering": ordering,
            "metric": metric,
            "cohorts_holding": n_hold,
            "cohort_detail": cohort_results,
            "verdict": consistency_verdict(n_hold),
        }

    consistency = {}
    for tercile in tercile_labels:
        consistency[tercile] = {
            "target_reached_Fav_gt_Mix_gt_Unf": check_ordering_in_tercile(tercile, "target_reached_rate", ORDERING_POSITIVE),
            "risk_reached_Unf_gt_Mix_gt_Fav": check_ordering_in_tercile(tercile, "risk_reached_rate", ORDERING_RISK),
            "realized_return_Fav_gt_Mix_gt_Unf": check_ordering_in_tercile(tercile, "realized_return_mean", ORDERING_POSITIVE),
        }

    return {
        "question": "Q4 — R:R interaction",
        "rr_tercile_boundaries": {
            "low_upper": low_upper,
            "high_lower": high_lower,
            "note": "Computed once over all 612 rows; fixed thereafter",
        },
        "pooled_by_tercile": pooled_by_tercile,
        "per_cohort_by_tercile": per_cohort_by_tercile,
        "consistency": consistency,
    }

# ---------------------------------------------------------------------------
# Cohort consistency summary
# ---------------------------------------------------------------------------

def build_cohort_consistency_summary(q1, q2, q3, q4):
    return {
        "Q1": {k: v["verdict"] for k, v in q1["consistency"].items()},
        "Q2_LONG": {k: v["verdict"] for k, v in q2["consistency"]["LONG"].items()},
        "Q2_SHORT": {k: v["verdict"] for k, v in q2["consistency"]["SHORT"].items()},
        "Q3_specific": {k: v["verdict"] for k, v in q3["specific_comparisons"].items()},
        "Q4_low": {k: v["verdict"] for k, v in q4["consistency"]["low"].items()},
        "Q4_medium": {k: v["verdict"] for k, v in q4["consistency"]["medium"].items()},
        "Q4_high": {k: v["verdict"] for k, v in q4["consistency"]["high"].items()},
    }

# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def fmt_pct(v):
    if v is None:
        return "N/A"
    return f"{v*100:.1f}%"

def fmt_f(v, decimals=4):
    if v is None:
        return "N/A"
    return f"{v:.{decimals}f}"

def build_report(rows, q1, q2, q3, q4, consistency_summary):
    n_eligible = sum(1 for r in rows if r["eligible_for_primary_comparison"])
    n_total = len(rows)

    lines = []
    lines.append("# TIME-008 — Discrimination Analysis Report")
    lines.append("")
    lines.append(f"**Generated:** {datetime.now(timezone.utc).isoformat()}")
    lines.append(f"**Input:** `time_machine/cohorts/aggregate_evidence.csv`")
    lines.append(f"**Total rows:** {n_total}  |  **Eligible for primary comparison:** {n_eligible}")
    lines.append(f"**Cohorts:** {', '.join(sorted(EXPECTED_COHORTS))}")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Governing constraints")
    lines.append("")
    lines.append("- Analysis-only. No upstream pipeline changes permitted.")
    lines.append("- Consistency threshold: >=4/6 cohorts for 'consistent', >=3/6 reversals for 'inconsistent'.")
    lines.append("- Primary outcomes: `target_reached`, `risk_reached`, `realized_return` (eligible rows only).")
    lines.append("- Secondary outcomes: `actual_mfe`, `actual_mae`, `exit_reason` (full population).")
    lines.append("- `eligible_for_primary_comparison` is a filter, not an outcome.")
    lines.append("- R:R terciles computed once over all 612 rows; fixed thereafter.")
    lines.append("")
    lines.append("---")
    lines.append("")

    # ---- Q1 ----
    lines.append("## Q1 — Evidence-class discrimination")
    lines.append("")
    lines.append("**Question:** Does evidence_class correspond to materially different forward outcomes?")
    lines.append("")
    lines.append("### Pooled results (primary: eligible rows only)")
    lines.append("")
    lines.append("| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |")
    lines.append("|---|---|---|---|---|---|")
    for cls in ["Favourable", "Mixed", "Unfavourable", "Insufficient"]:
        if cls in q1["pooled"]:
            s = q1["pooled"][cls]
            p = s["primary"]
            lines.append(
                f"| {cls} | {s['n_total']} | {s['n_eligible']} | "
                f"{fmt_pct(p['target_reached_rate'])} | "
                f"{fmt_pct(p['risk_reached_rate'])} | "
                f"{fmt_f(p['realized_return_mean'])} |"
            )
    lines.append("")
    lines.append("### Secondary outcomes (full population)")
    lines.append("")
    lines.append("| Evidence Class | N total | MFE mean | MAE mean |")
    lines.append("|---|---|---|---|")
    for cls in ["Favourable", "Mixed", "Unfavourable", "Insufficient"]:
        if cls in q1["pooled"]:
            s = q1["pooled"][cls]
            sec = s["secondary"]
            lines.append(
                f"| {cls} | {s['n_total']} | "
                f"{fmt_f(sec['actual_mfe_mean'])} | "
                f"{fmt_f(sec['actual_mae_mean'])} |"
            )
    lines.append("")
    lines.append("### Consistency verdicts (>=4/6 cohorts = consistent)")
    lines.append("")
    for key, chk in q1["consistency"].items():
        lines.append(f"- **{key}**: {chk['cohorts_holding']}/6 cohorts → **{chk['verdict']}**")
        for cohort in sorted(EXPECTED_COHORTS):
            lines.append(f"  - {cohort}: {chk['cohort_detail'][cohort]}")
    lines.append("")

    # ---- Q2 ----
    lines.append("## Q2 — Direction asymmetry")
    lines.append("")
    lines.append("**Question:** Does discrimination differ between LONG and SHORT decisions?")
    lines.append("")
    dir_counts = q2["direction_counts"]
    lines.append(f"Direction counts: LONG={dir_counts.get('LONG', 0)}, SHORT={dir_counts.get('SHORT', 0)}")
    lines.append("")
    for direction in ["LONG", "SHORT"]:
        lines.append(f"### {direction} — Pooled by evidence class")
        lines.append("")
        lines.append("| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |")
        lines.append("|---|---|---|---|---|---|")
        for cls in ["Favourable", "Mixed", "Unfavourable", "Insufficient"]:
            if cls in q2["pooled_by_direction"].get(direction, {}):
                s = q2["pooled_by_direction"][direction][cls]
                p = s["primary"]
                lines.append(
                    f"| {cls} | {s['n_total']} | {s['n_eligible']} | "
                    f"{fmt_pct(p['target_reached_rate'])} | "
                    f"{fmt_pct(p['risk_reached_rate'])} | "
                    f"{fmt_f(p['realized_return_mean'])} |"
                )
        lines.append("")
        lines.append(f"**Consistency verdicts ({direction}):**")
        for key, chk in q2["consistency"][direction].items():
            lines.append(f"- {key}: {chk['cohorts_holding']}/6 → **{chk['verdict']}**")
        lines.append("")

    # ---- Q3 ----
    lines.append("## Q3 — Action vs underlying decision")
    lines.append("")
    lines.append("**Question:** Does action contain predictive information beyond evidence_class?")
    lines.append("")
    ac = q3["action_counts"]
    lines.append(f"Action counts (all 612): " + ", ".join(f"{k}={v}" for k, v in sorted(ac.items())))
    lines.append("")
    for cls in ["Favourable", "Mixed", "Unfavourable", "Insufficient"]:
        if cls in q3["pooled_by_class_then_action"]:
            lines.append(f"### Pooled: within {cls} by action")
            lines.append("")
            lines.append("| Action | N total | N eligible | Target Reached | Risk Reached | Realized Return |")
            lines.append("|---|---|---|---|---|---|")
            for action, s in sorted(q3["pooled_by_class_then_action"][cls].items()):
                p = s["primary"]
                lines.append(
                    f"| {action} | {s['n_total']} | {s['n_eligible']} | "
                    f"{fmt_pct(p['target_reached_rate'])} | "
                    f"{fmt_pct(p['risk_reached_rate'])} | "
                    f"{fmt_f(p['realized_return_mean'])} |"
                )
            lines.append("")
    lines.append("### Specific comparisons (pre-specified)")
    lines.append("")
    for key, chk in q3["specific_comparisons"].items():
        lines.append(
            f"- **{key}**: {chk['comparison']} within {chk['evidence_class']} "
            f"({chk['metric']}): {chk['cohorts_holding']}/{chk['cohorts_with_data']} cohorts → **{chk['verdict']}**"
        )
    lines.append("")

    # ---- Q4 ----
    lines.append("## Q4 — R:R interaction")
    lines.append("")
    lines.append("**Question:** Does the relationship between T0 evidence and outcome depend on adaptive_rr?")
    lines.append("")
    bounds = q4["rr_tercile_boundaries"]
    lines.append(f"R:R tercile boundaries (global, 612 rows): low<={bounds['low_upper']:.4f}, medium<={bounds['high_lower']:.4f}, high>{bounds['high_lower']:.4f}")
    lines.append("")
    for tercile in ["low", "medium", "high"]:
        t_data = q4["pooled_by_tercile"][tercile]
        lines.append(f"### {tercile.capitalize()} R:R tercile (n={t_data['n']})")
        lines.append("")
        lines.append("| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |")
        lines.append("|---|---|---|---|---|---|")
        for cls in ["Favourable", "Mixed", "Unfavourable", "Insufficient"]:
            if cls in t_data["by_evidence_class"]:
                s = t_data["by_evidence_class"][cls]
                p = s["primary"]
                lines.append(
                    f"| {cls} | {s['n_total']} | {s['n_eligible']} | "
                    f"{fmt_pct(p['target_reached_rate'])} | "
                    f"{fmt_pct(p['risk_reached_rate'])} | "
                    f"{fmt_f(p['realized_return_mean'])} |"
                )
        lines.append("")
        lines.append(f"**Consistency verdicts ({tercile} R:R tercile):**")
        for key, chk in q4["consistency"][tercile].items():
            lines.append(f"- {key}: {chk['cohorts_holding']}/6 → **{chk['verdict']}**")
        lines.append("")

    # ---- Consistency summary ----
    lines.append("## Cohort consistency summary")
    lines.append("")
    lines.append("| Question | Check | Verdict |")
    lines.append("|---|---|---|")
    for section, checks in consistency_summary.items():
        for key, verdict in checks.items():
            lines.append(f"| {section} | {key} | **{verdict}** |")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Bounded conclusion")
    lines.append("")
    lines.append(
        "TIME-008 establishes descriptive evidence about discrimination—or lack thereof—"
        "within the frozen historical experiment. It does not establish predictive validity, "
        "economic utility, or deployment readiness."
    )
    lines.append("")
    lines.append("TIME-008 may NOT conclude:")
    lines.append("- That Coralys has proven predictive power")
    lines.append("- That the system is economically useful")
    lines.append("- That any threshold should be changed")
    lines.append("- That any algorithm should be modified")
    lines.append("- That the evidence is sufficient for live deployment")
    lines.append("")

    return "\n".join(lines)

# ---------------------------------------------------------------------------
# Write JSON helper
# ---------------------------------------------------------------------------

def write_json(path, data):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, default=str)
    print(f"[time008] Written: {path}")

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    rows = load_data()

    print("[time008] Running Q1...")
    q1 = run_q1(rows)

    print("[time008] Running Q2...")
    q2 = run_q2(rows)

    print("[time008] Running Q3...")
    q3 = run_q3(rows)

    print("[time008] Running Q4...")
    q4 = run_q4(rows)

    print("[time008] Building consistency summary...")
    consistency_summary = build_cohort_consistency_summary(q1, q2, q3, q4)

    print("[time008] Building report...")
    report_md = build_report(rows, q1, q2, q3, q4, consistency_summary)

    # Write artifacts
    write_json(os.path.join(OUTPUT_DIR, "q1_evidence_class_discrimination.json"), q1)
    write_json(os.path.join(OUTPUT_DIR, "q2_direction_asymmetry.json"), q2)
    write_json(os.path.join(OUTPUT_DIR, "q3_action_vs_decision.json"), q3)
    write_json(os.path.join(OUTPUT_DIR, "q4_rr_interaction.json"), q4)
    write_json(os.path.join(OUTPUT_DIR, "cohort_consistency_summary.json"), consistency_summary)

    report_path = os.path.join(OUTPUT_DIR, "analysis_report.md")
    with open(report_path, "w", encoding="utf-8") as f:
        f.write(report_md)
    print(f"[time008] Written: {report_path}")

    run_ts = datetime.now(timezone.utc).isoformat()
    latest_run = {
        "run_timestamp": run_ts,
        "input_csv": INPUT_CSV,
        "input_rows": EXPECTED_ROWS,
        "cohorts": sorted(EXPECTED_COHORTS),
        "rows_per_cohort": EXPECTED_ROWS_PER_COHORT,
        "n_eligible": sum(1 for r in rows if r["eligible_for_primary_comparison"]),
        "consistency_threshold": CONSISTENCY_THRESHOLD,
        "inconsistency_threshold": INCONSISTENCY_THRESHOLD,
        "artifacts": [
            "q1_evidence_class_discrimination.json",
            "q2_direction_asymmetry.json",
            "q3_action_vs_decision.json",
            "q4_rr_interaction.json",
            "cohort_consistency_summary.json",
            "analysis_report.md",
        ],
        "governing_rule": "Analysis-only. No upstream pipeline changes permitted.",
    }
    write_json(os.path.join(OUTPUT_DIR, "latest_run.json"), latest_run)

    print(f"[time008] All artifacts written to {OUTPUT_DIR}")
    print(f"[time008] Run complete at {run_ts}")

if __name__ == "__main__":
    main()