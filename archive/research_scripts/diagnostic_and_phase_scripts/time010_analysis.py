#!/usr/bin/env python3
"""
TIME-010 Analysis Script — time010_analysis.py

Pre-specified analysis of the TIME-009 prospective observation dataset.
Implements the TIME-010 protocol (docs/TIME010_PROTOCOL.md) exactly.

GOVERNANCE:
  - This script was committed BEFORE any COMPLETE observations were available.
  - No analytical choices may be made after seeing outcome data.
  - The script refuses to overwrite a frozen latest_run.json.
  - All statistical tests and thresholds are pre-specified in the protocol.

Usage:
    python3 scripts/time010_analysis.py \
        --dataset  time_machine/analysis/TIME009/prospective_evidence.csv \
        --output   time_machine/analysis/TIME010

Exit code: 0 = analysis complete, 1 = error or NOT_ESTIMABLE
"""

import argparse
import csv
import json
import math
import os
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

# ── Pre-specified constants (TIME-010 protocol, section 3–7) ─────────────────

SIGNIFICANCE_THRESHOLD = 0.05          # p < 0.05 (section 3)
MIN_ELIGIBLE_ROWS = 20                 # section 7
MIN_COHORT_DATES = 3                   # section 5
CONSISTENCY_FRACTION = 0.67            # ≥ceil(0.67 × N) (section 5)
BOOTSTRAP_RESAMPLES = 10_000           # section 3.2
PRODUCER = "time010_analysis.v1"
PROTOCOL_VERSION = "TIME010-v1.0"

# ── Eligibility (verbatim from TIME-009 AC-T9-05, section 2.2) ───────────────

ELIGIBLE_CERT_STATUSES = {"CERTIFIED", "DEGRADED"}
ELIGIBLE_EVIDENCE_CLASSES = {"Favourable", "Mixed"}
INELIGIBLE_EXIT_REASONS = {"AMBIGUOUS", "INSUFFICIENT_DATA", "NO_TRADE"}


def parse_args():
    p = argparse.ArgumentParser(description="TIME-010 analysis (pre-specified protocol)")
    p.add_argument(
        "--dataset",
        default="time_machine/analysis/TIME009/prospective_evidence.csv",
        help="Path to prospective_evidence.csv from time009_dataset.py",
    )
    p.add_argument(
        "--output",
        default="time_machine/analysis/TIME010",
        help="Output directory for analysis artifacts",
    )
    p.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing output (only for re-runs before conclusion is frozen)",
    )
    return p.parse_args()


# ── Data loading ──────────────────────────────────────────────────────────────

def load_dataset(path: Path):
    """Load prospective_evidence.csv. Returns list of row dicts."""
    if not path.exists():
        print(f"[time010] ERROR: dataset not found: {path}", file=sys.stderr)
        return None
    rows = []
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows.append(row)
    return rows


def coerce_row(row: dict) -> dict:
    """Coerce CSV string values to appropriate types."""
    def to_float(v):
        try:
            return float(v) if v not in ("", "None", "null") else None
        except (ValueError, TypeError):
            return None

    def to_bool(v):
        if isinstance(v, bool):
            return v
        if v in ("True", "true", "1"):
            return True
        if v in ("False", "false", "0"):
            return False
        return None

    def to_int(v):
        try:
            return int(v) if v not in ("", "None", "null") else None
        except (ValueError, TypeError):
            return None

    return {
        **row,
        "target_reached": to_bool(row.get("target_reached")),
        "risk_reached": to_bool(row.get("risk_reached")),
        "horizon_reached": to_bool(row.get("horizon_reached")),
        "ambiguous": to_bool(row.get("ambiguous")),
        "actual_mfe": to_float(row.get("actual_mfe")),
        "actual_mae": to_float(row.get("actual_mae")),
        "realized_return": to_float(row.get("realized_return")),
        "reference_price": to_float(row.get("reference_price")),
        "eligible_for_primary_comparison": to_bool(row.get("eligible_for_primary_comparison")),
        "sample_size": to_int(row.get("sample_size")),
        "sessions_to_outcome": to_int(row.get("sessions_to_outcome")),
    }


# ── Statistical tests (pre-specified) ────────────────────────────────────────

def two_proportion_z_test(n1, k1, n2, k2):
    """
    One-sided two-proportion z-test: H1: p1 > p2.
    Returns (z_stat, p_value).
    Pre-specified for Q1 (section 3.1).
    """
    import math
    if n1 == 0 or n2 == 0:
        return None, None
    p1 = k1 / n1
    p2 = k2 / n2
    p_pool = (k1 + k2) / (n1 + n2)
    se = math.sqrt(p_pool * (1 - p_pool) * (1 / n1 + 1 / n2))
    if se == 0:
        return None, None
    z = (p1 - p2) / se
    # One-sided p-value (H1: p1 > p2)
    from math import erfc, sqrt
    p_val = 0.5 * erfc(z / sqrt(2))  # P(Z > z) = 1 - Phi(z)
    # erfc(x) = 2*(1-Phi(x*sqrt(2))), so P(Z>z) = 0.5*erfc(z/sqrt(2))
    return z, p_val


def mann_whitney_u(x, y):
    """
    One-sided Mann-Whitney U test: H1: x > y (stochastic dominance).
    Returns (u_stat, p_value, cles).
    Pre-specified for Q2 (section 3.2).
    Uses normal approximation for large samples.
    """
    import math
    n1, n2 = len(x), len(y)
    if n1 == 0 or n2 == 0:
        return None, None, None

    # Count concordant pairs
    u = sum(1 for xi in x for yj in y if xi > yj) + 0.5 * sum(1 for xi in x for yj in y if xi == yj)
    cles = u / (n1 * n2)  # Common Language Effect Size

    # Normal approximation
    mu_u = n1 * n2 / 2
    sigma_u = math.sqrt(n1 * n2 * (n1 + n2 + 1) / 12)
    if sigma_u == 0:
        return u, None, cles
    z = (u - mu_u) / sigma_u
    from math import erfc, sqrt
    p_val = 0.5 * erfc(z / sqrt(2))  # one-sided P(U > expected)
    return u, p_val, cles


def wilson_ci(k, n, z=1.96):
    """Wilson score 95% CI for a proportion."""
    if n == 0:
        return None, None
    p = k / n
    denom = 1 + z**2 / n
    centre = (p + z**2 / (2 * n)) / denom
    margin = z * math.sqrt(p * (1 - p) / n + z**2 / (4 * n**2)) / denom
    return max(0.0, centre - margin), min(1.0, centre + margin)


def bootstrap_mean_ci(values, n_resamples=BOOTSTRAP_RESAMPLES, seed=42):
    """Bootstrap 95% CI for the mean."""
    import random
    rng = random.Random(seed)
    n = len(values)
    if n == 0:
        return None, None
    means = []
    for _ in range(n_resamples):
        sample = [rng.choice(values) for _ in range(n)]
        means.append(sum(sample) / n)
    means.sort()
    lo = means[int(0.025 * n_resamples)]
    hi = means[int(0.975 * n_resamples)]
    return lo, hi


# ── Cohort consistency ────────────────────────────────────────────────────────

def cohort_consistency(rows_by_cohort, metric_fn, class_a="Favourable", class_b="Mixed"):
    """
    Compute cohort consistency: number of cohort dates where class_a > class_b.
    Returns (n_consistent, n_estimable, threshold, is_consistent).
    Pre-specified in section 5.
    """
    n_consistent = 0
    n_estimable = 0
    for cohort, rows in sorted(rows_by_cohort.items()):
        a_rows = [r for r in rows if r["evidence_class"] == class_a]
        b_rows = [r for r in rows if r["evidence_class"] == class_b]
        if not a_rows or not b_rows:
            continue  # cohort not estimable for this comparison
        n_estimable += 1
        val_a = metric_fn(a_rows)
        val_b = metric_fn(b_rows)
        if val_a is not None and val_b is not None and val_a > val_b:
            n_consistent += 1

    threshold = math.ceil(CONSISTENCY_FRACTION * n_estimable) if n_estimable >= MIN_COHORT_DATES else None
    is_consistent = (n_consistent >= threshold) if threshold is not None else None
    return n_consistent, n_estimable, threshold, is_consistent


# ── Research conclusion classification (section 6) ───────────────────────────

def classify_conclusion(
    n_eligible, n_cohort_dates,
    q1_sig, q1_consistent,
    q2_sig, q2_consistent,
):
    if n_eligible < MIN_ELIGIBLE_ROWS or n_cohort_dates < MIN_COHORT_DATES:
        return "NOT_ESTIMABLE"
    if q1_sig and q2_sig and q1_consistent and q2_consistent:
        return "POSITIVE"
    if (q1_sig and q1_consistent) or (q2_sig and q2_consistent):
        return "PARTIAL"
    if not q1_sig and not q2_sig:
        return "NEGATIVE"
    # One or both significant but consistency not met
    return "INCONCLUSIVE"


# ── Report generation ─────────────────────────────────────────────────────────

def fmt_pct(v):
    return f"{v*100:.1f}%" if v is not None else "—"

def fmt_f(v, d=4):
    return f"{v:.{d}f}" if v is not None else "—"

def fmt_n(v):
    return str(v) if v is not None else "—"


def write_report(path: Path, ctx: dict):
    """Write the TIME-010 analysis report following the template in section 9."""
    lines = []
    a = lines.append

    a("# TIME-010 Prospective Analysis Report")
    a("")
    a(f"**Experiment:** TIME-010  ")
    a(f"**Protocol:** docs/TIME010_PROTOCOL.md  ")
    a(f"**Run at:** {ctx['run_at']}  ")
    a(f"**Dataset:** {ctx['dataset_source']}  ")
    a(f"**Producer:** {ctx['producer']}  ")
    a("")
    a("---")
    a("")

    # Section 1: Header
    a("## 1. Dataset summary")
    a("")
    a(f"| Field | Value |")
    a(f"|---|---|")
    a(f"| N total rows | {ctx['n_total_rows']} |")
    a(f"| N eligible rows | {ctx['n_eligible_rows']} |")
    a(f"| N Favourable eligible | {ctx['n_favourable_eligible']} |")
    a(f"| N Mixed eligible | {ctx['n_mixed_eligible']} |")
    a(f"| N cohort dates | {ctx['n_cohort_dates']} |")
    a(f"| N cohort dates estimable (both classes) | {ctx['n_cohort_dates_estimable']} |")
    a("")

    # Section 2: Eligibility accounting
    a("## 2. Eligibility accounting")
    a("")
    a("| Cohort date | Evidence class | N total | N eligible | N ineligible | Ineligibility reasons |")
    a("|---|---|---|---|---|---|")
    for row in ctx["eligibility_table"]:
        a(f"| {row['cohort']} | {row['ec']} | {row['n_total']} | {row['n_eligible']} | {row['n_ineligible']} | {row['reasons']} |")
    a("")

    # Section 3: Q1
    a("## 3. Primary Q1 — Target attainment rate (Favourable vs Mixed)")
    a("")
    a("| Evidence class | N | Target reached | Rate | 95% CI (Wilson) |")
    a("|---|---|---|---|---|")
    a(f"| Favourable | {ctx['q1_n_fav']} | {ctx['q1_k_fav']} | {fmt_pct(ctx['q1_rate_fav'])} | [{fmt_pct(ctx['q1_ci_fav'][0])}, {fmt_pct(ctx['q1_ci_fav'][1])}] |")
    a(f"| Mixed | {ctx['q1_n_mix']} | {ctx['q1_k_mix']} | {fmt_pct(ctx['q1_rate_mix'])} | [{fmt_pct(ctx['q1_ci_mix'][0])}, {fmt_pct(ctx['q1_ci_mix'][1])}] |")
    a("")
    a(f"**Test:** Two-proportion z-test (one-sided, H1: Favourable > Mixed)  ")
    a(f"**z-statistic:** {fmt_f(ctx['q1_z'], 3)}  ")
    a(f"**p-value:** {fmt_f(ctx['q1_p'], 4)}  ")
    a(f"**Significant (p < {SIGNIFICANCE_THRESHOLD}):** {'YES' if ctx['q1_significant'] else 'NO'}  ")
    a(f"**Cohort consistency:** {ctx['q1_n_consistent']}/{ctx['q1_n_estimable']} cohort dates Fav > Mix (threshold: {ctx['q1_threshold']})  ")
    a(f"**Cohort consistent:** {'YES' if ctx['q1_cohort_consistent'] else ('NO' if ctx['q1_cohort_consistent'] is False else 'NOT_ESTIMABLE')}  ")
    a("")

    # Section 4: Q2
    a("## 4. Primary Q2 — Realized return (Favourable vs Mixed)")
    a("")
    a("| Evidence class | N | Mean | Median | SD | 95% CI (bootstrap) |")
    a("|---|---|---|---|---|---|")
    a(f"| Favourable | {ctx['q2_n_fav']} | {fmt_f(ctx['q2_mean_fav'])} | {fmt_f(ctx['q2_median_fav'])} | {fmt_f(ctx['q2_sd_fav'])} | [{fmt_f(ctx['q2_ci_fav'][0])}, {fmt_f(ctx['q2_ci_fav'][1])}] |")
    a(f"| Mixed | {ctx['q2_n_mix']} | {fmt_f(ctx['q2_mean_mix'])} | {fmt_f(ctx['q2_median_mix'])} | {fmt_f(ctx['q2_sd_mix'])} | [{fmt_f(ctx['q2_ci_mix'][0])}, {fmt_f(ctx['q2_ci_mix'][1])}] |")
    a("")
    a(f"**Test:** Mann-Whitney U (one-sided, H1: Favourable > Mixed)  ")
    a(f"**U-statistic:** {fmt_f(ctx['q2_u'], 1)}  ")
    a(f"**CLES:** {fmt_f(ctx['q2_cles'], 3)}  ")
    a(f"**p-value:** {fmt_f(ctx['q2_p'], 4)}  ")
    a(f"**Significant (p < {SIGNIFICANCE_THRESHOLD}):** {'YES' if ctx['q2_significant'] else 'NO'}  ")
    a(f"**Cohort consistency:** {ctx['q2_n_consistent']}/{ctx['q2_n_estimable']} cohort dates Fav > Mix (threshold: {ctx['q2_threshold']})  ")
    a(f"**Cohort consistent:** {'YES' if ctx['q2_cohort_consistent'] else ('NO' if ctx['q2_cohort_consistent'] is False else 'NOT_ESTIMABLE')}  ")
    a("")

    # Section 5: Secondary
    a("## 5. Secondary endpoints")
    a("")
    a("### Q3 — MFE and MAE by evidence class")
    a("")
    a("| Evidence class | N | Mean MFE | Median MFE | Mean MAE | Median MAE |")
    a("|---|---|---|---|---|---|")
    for ec, stats in ctx["q3_table"].items():
        a(f"| {ec} | {stats['n']} | {fmt_f(stats['mean_mfe'])} | {fmt_f(stats['median_mfe'])} | {fmt_f(stats['mean_mae'])} | {fmt_f(stats['median_mae'])} |")
    a("")
    a("### Q4 — Direction stratification")
    a("")
    a("| Direction | Evidence class | N | Target rate | Mean return |")
    a("|---|---|---|---|---|")
    for (direction, ec), stats in sorted(ctx["q4_table"].items()):
        a(f"| {direction} | {ec} | {stats['n']} | {fmt_pct(stats['target_rate'])} | {fmt_f(stats['mean_return'])} |")
    a("")
    a("### Q5 — DEGRADED vs CERTIFIED stratification")
    a("")
    a("| Cert status | Evidence class | N | Target rate | Mean return |")
    a("|---|---|---|---|---|")
    for (cs, ec), stats in sorted(ctx["q5_table"].items()):
        a(f"| {cs} | {ec} | {stats['n']} | {fmt_pct(stats['target_rate'])} | {fmt_f(stats['mean_return'])} |")
    a("")
    a("### Q6 — Exit reason distribution")
    a("")
    a("| Exit reason | Favourable | Mixed | Unfavourable |")
    a("|---|---|---|---|")
    all_reasons = sorted(ctx["q6_table"].keys())
    for reason in all_reasons:
        row = ctx["q6_table"][reason]
        a(f"| {reason} | {row.get('Favourable', 0)} | {row.get('Mixed', 0)} | {row.get('Unfavourable', 0)} |")
    a("")
    a("### Q7 — Cohort date breakdown")
    a("")
    a("| Cohort date | N eligible | N Fav | N Mix | Fav target rate | Mix target rate | Fav > Mix |")
    a("|---|---|---|---|---|---|---|")
    for row in ctx["q7_table"]:
        a(f"| {row['cohort']} | {row['n_eligible']} | {row['n_fav']} | {row['n_mix']} | {fmt_pct(row['fav_rate'])} | {fmt_pct(row['mix_rate'])} | {'✓' if row['fav_beats_mix'] else '✗'} |")
    a("")

    # Section 6: Conclusion
    a("## 6. Research conclusion")
    a("")
    conclusion = ctx["research_conclusion"]
    a(f"**Classification:** `{conclusion}`")
    a("")
    if conclusion == "NOT_ESTIMABLE":
        a(f"> TIME-010 cannot produce a research conclusion because the eligible dataset contains fewer than {MIN_ELIGIBLE_ROWS} rows or fewer than {MIN_COHORT_DATES} cohort dates.")
    elif conclusion == "POSITIVE":
        a("> Both primary endpoints (Q1 target attainment, Q2 realized return) are statistically significant and cohort-consistent. The frozen evidence classification shows prospective discrimination of forward outcomes.")
    elif conclusion == "PARTIAL":
        a("> Exactly one primary endpoint is statistically significant and cohort-consistent. The evidence for prospective discrimination is partial and should not be treated as equivalent to a POSITIVE result.")
    elif conclusion == "NEGATIVE":
        a("> Neither primary endpoint is statistically significant. The frozen evidence classification does not demonstrate prospective discrimination of forward outcomes.")
    elif conclusion == "INCONCLUSIVE":
        a("> One or both primary endpoints are statistically significant, but the cohort consistency criterion is not met. The result cannot be classified as POSITIVE or PARTIAL.")
    a("")

    # Frozen conclusion paragraph (section 9, item 8)
    a("### Frozen conclusion paragraph")
    a("")
    a("> **TIME-010 research conclusion (frozen):** " + ctx["frozen_conclusion_text"])
    a("")

    # Section 7: Limitations
    a("## 7. Limitations")
    a("")
    a("The following limitations apply to this analysis:")
    a("")
    a(f"- **Prospective horizon:** The observation horizon is 3–4 NSE trading sessions per decision. This is a short-term horizon and results may not generalise to longer holding periods.")
    a(f"- **Cohort count:** {ctx['n_cohort_dates']} cohort dates accumulated. The cohort consistency criterion requires ≥{MIN_COHORT_DATES} estimable cohort dates.")
    a(f"- **Eligibility exclusions:** {ctx['n_total_rows'] - ctx['n_eligible_rows']} rows excluded from primary comparison (AMBIGUOUS, INSUFFICIENT_DATA, NO_TRADE, or non-CERTIFIED/DEGRADED).")
    a(f"- **DEGRADED inclusion:** DEGRADED decisions are included in the primary comparison per AC-T9-11. See Q5 for stratified results.")
    a(f"- **Single cohort date:** All 204 decisions were admitted on 2026-08-20. The cohort consistency criterion evaluates ordering across cohort dates; with a single cohort date, this criterion is not estimable.")
    a(f"- **No Unfavourable primary comparison:** Unfavourable decisions had 0 eligible rows (consistent with TIME-008 finding).")
    a("")

    path.write_text("\n".join(lines))


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    args = parse_args()
    dataset_path = Path(args.dataset)
    output_dir = Path(args.output)

    print("[time010] TIME-010 Analysis — Pre-specified Protocol")
    print("[time010] ============================================")
    print(f"[time010] dataset: {dataset_path}")
    print(f"[time010] output:  {output_dir}")

    # Check for frozen output (section 7: NOT_ESTIMABLE writes conclusion_frozen=false,
    # so only a final estimable run blocks re-execution)
    summary_path = output_dir / "latest_run.json"
    if summary_path.exists() and not args.force:
        existing = json.loads(summary_path.read_text())
        if existing.get("conclusion_frozen") is True:
            print("[time010] ERROR: latest_run.json has conclusion_frozen=true — refusing to overwrite")
            print("[time010] The final TIME-010 conclusion has already been produced and frozen.")
            print("[time010] Use --force only for debugging; do not use --force in production.")
            return 1
        else:
            print(f"[time010] NOTE: overwriting non-final latest_run.json (conclusion_frozen=false, conclusion={existing.get('research_conclusion')})")

    # Load dataset
    raw_rows = load_dataset(dataset_path)
    if raw_rows is None:
        return 1

    rows = [coerce_row(r) for r in raw_rows]
    n_total = len(rows)
    print(f"[time010] n_total_rows={n_total}")

    # Apply eligibility (read verbatim from artifact field)
    eligible = [r for r in rows if r.get("eligible_for_primary_comparison") is True]
    n_eligible = len(eligible)
    print(f"[time010] n_eligible_rows={n_eligible}")

    # Check minimum sample size
    if n_eligible < MIN_ELIGIBLE_ROWS:
        print(f"[time010] NOT_ESTIMABLE: n_eligible={n_eligible} < {MIN_ELIGIBLE_ROWS}")
        conclusion = "NOT_ESTIMABLE"
        _write_not_estimable(output_dir, summary_path, dataset_path, n_total, n_eligible, conclusion)
        return 1

    # Split by evidence class
    fav = [r for r in eligible if r["evidence_class"] == "Favourable"]
    mix = [r for r in eligible if r["evidence_class"] == "Mixed"]
    print(f"[time010] n_favourable_eligible={len(fav)} n_mixed_eligible={len(mix)}")

    # Cohort dates
    cohort_dates = sorted(set(r["cohort_date"] for r in eligible))
    n_cohort_dates = len(cohort_dates)
    print(f"[time010] n_cohort_dates={n_cohort_dates}")

    if n_cohort_dates < MIN_COHORT_DATES:
        print(f"[time010] NOTE: n_cohort_dates={n_cohort_dates} < {MIN_COHORT_DATES} — cohort consistency not estimable")

    # Group eligible rows by cohort date
    by_cohort = defaultdict(list)
    for r in eligible:
        by_cohort[r["cohort_date"]].append(r)

    # ── Q1: Target attainment rate ────────────────────────────────────────────
    q1_n_fav = len(fav)
    q1_k_fav = sum(1 for r in fav if r["target_reached"] is True)
    q1_n_mix = len(mix)
    q1_k_mix = sum(1 for r in mix if r["target_reached"] is True)
    q1_rate_fav = q1_k_fav / q1_n_fav if q1_n_fav > 0 else None
    q1_rate_mix = q1_k_mix / q1_n_mix if q1_n_mix > 0 else None
    q1_ci_fav = wilson_ci(q1_k_fav, q1_n_fav)
    q1_ci_mix = wilson_ci(q1_k_mix, q1_n_mix)
    q1_z, q1_p = two_proportion_z_test(q1_n_fav, q1_k_fav, q1_n_mix, q1_k_mix)
    q1_significant = (q1_p is not None and q1_p < SIGNIFICANCE_THRESHOLD)

    q1_n_consistent, q1_n_estimable, q1_threshold, q1_cohort_consistent = cohort_consistency(
        by_cohort,
        lambda rows: sum(1 for r in rows if r["target_reached"] is True) / len(rows) if rows else None,
    )

    q1_p_str = f"{q1_p:.4f}" if q1_p is not None else "N/A"
    print(f"[time010] Q1: Fav={fmt_pct(q1_rate_fav)} Mix={fmt_pct(q1_rate_mix)} p={q1_p_str} sig={q1_significant}")

    # ── Q2: Realized return ───────────────────────────────────────────────────
    fav_ret = [r["realized_return"] for r in fav if r["realized_return"] is not None]
    mix_ret = [r["realized_return"] for r in mix if r["realized_return"] is not None]

    def safe_mean(v): return sum(v) / len(v) if v else None
    def safe_median(v):
        if not v: return None
        s = sorted(v)
        n = len(s)
        return (s[n//2 - 1] + s[n//2]) / 2 if n % 2 == 0 else s[n//2]
    def safe_sd(v):
        if len(v) < 2: return None
        m = safe_mean(v)
        return math.sqrt(sum((x - m)**2 for x in v) / (len(v) - 1))

    q2_n_fav = len(fav_ret)
    q2_n_mix = len(mix_ret)
    q2_mean_fav = safe_mean(fav_ret)
    q2_mean_mix = safe_mean(mix_ret)
    q2_median_fav = safe_median(fav_ret)
    q2_median_mix = safe_median(mix_ret)
    q2_sd_fav = safe_sd(fav_ret)
    q2_sd_mix = safe_sd(mix_ret)
    q2_ci_fav = bootstrap_mean_ci(fav_ret)
    q2_ci_mix = bootstrap_mean_ci(mix_ret)
    q2_u, q2_p, q2_cles = mann_whitney_u(fav_ret, mix_ret)
    q2_significant = (q2_p is not None and q2_p < SIGNIFICANCE_THRESHOLD)

    q2_n_consistent, q2_n_estimable, q2_threshold, q2_cohort_consistent = cohort_consistency(
        by_cohort,
        lambda rows: safe_median([r["realized_return"] for r in rows if r["realized_return"] is not None]),
    )

    q2_p_str = f"{q2_p:.4f}" if q2_p is not None else "N/A"
    print(f"[time010] Q2: Fav_mean={fmt_f(q2_mean_fav)} Mix_mean={fmt_f(q2_mean_mix)} p={q2_p_str} sig={q2_significant}")

    # ── Research conclusion ───────────────────────────────────────────────────
    conclusion = classify_conclusion(
        n_eligible, n_cohort_dates,
        q1_significant, q1_cohort_consistent,
        q2_significant, q2_cohort_consistent,
    )
    print(f"[time010] research_conclusion={conclusion}")

    # Frozen conclusion text
    frozen_text = _build_frozen_conclusion(
        conclusion, q1_rate_fav, q1_rate_mix, q1_p, q1_significant, q1_cohort_consistent,
        q2_mean_fav, q2_mean_mix, q2_p, q2_significant, q2_cohort_consistent,
        n_eligible, n_cohort_dates,
    )

    # ── Secondary endpoints ───────────────────────────────────────────────────
    # Q3 — MFE/MAE by evidence class (all eligible rows)
    q3_table = {}
    for ec in ("Favourable", "Mixed", "Unfavourable"):
        ec_rows = [r for r in rows if r["evidence_class"] == ec]
        mfe_vals = [r["actual_mfe"] for r in ec_rows if r["actual_mfe"] is not None]
        mae_vals = [r["actual_mae"] for r in ec_rows if r["actual_mae"] is not None]
        q3_table[ec] = {
            "n": len(ec_rows),
            "mean_mfe": safe_mean(mfe_vals),
            "median_mfe": safe_median(mfe_vals),
            "mean_mae": safe_mean(mae_vals),
            "median_mae": safe_median(mae_vals),
        }

    # Q4 — Direction stratification
    q4_table = {}
    for direction in ("LONG", "SHORT"):
        for ec in ("Favourable", "Mixed"):
            subset = [r for r in eligible if r["direction"] == direction and r["evidence_class"] == ec]
            ret_vals = [r["realized_return"] for r in subset if r["realized_return"] is not None]
            n_target = sum(1 for r in subset if r["target_reached"] is True)
            q4_table[(direction, ec)] = {
                "n": len(subset),
                "target_rate": n_target / len(subset) if subset else None,
                "mean_return": safe_mean(ret_vals),
            }

    # Q5 — DEGRADED vs CERTIFIED stratification
    q5_table = {}
    for cs in ("CERTIFIED", "DEGRADED"):
        for ec in ("Favourable", "Mixed"):
            subset = [r for r in eligible if r["certification_status"] == cs and r["evidence_class"] == ec]
            ret_vals = [r["realized_return"] for r in subset if r["realized_return"] is not None]
            n_target = sum(1 for r in subset if r["target_reached"] is True)
            q5_table[(cs, ec)] = {
                "n": len(subset),
                "target_rate": n_target / len(subset) if subset else None,
                "mean_return": safe_mean(ret_vals),
            }

    # Q6 — Exit reason distribution
    q6_table = defaultdict(lambda: {"Favourable": 0, "Mixed": 0, "Unfavourable": 0})
    for r in rows:
        reason = r.get("exit_reason", "UNKNOWN") or "UNKNOWN"
        ec = r.get("evidence_class", "UNKNOWN")
        if ec in ("Favourable", "Mixed", "Unfavourable"):
            q6_table[reason][ec] += 1

    # Q7 — Cohort date breakdown
    q7_table = []
    for cohort in sorted(set(r["cohort_date"] for r in rows)):
        cohort_elig = [r for r in eligible if r["cohort_date"] == cohort]
        fav_c = [r for r in cohort_elig if r["evidence_class"] == "Favourable"]
        mix_c = [r for r in cohort_elig if r["evidence_class"] == "Mixed"]
        fav_rate = sum(1 for r in fav_c if r["target_reached"] is True) / len(fav_c) if fav_c else None
        mix_rate = sum(1 for r in mix_c if r["target_reached"] is True) / len(mix_c) if mix_c else None
        fav_beats_mix = (fav_rate is not None and mix_rate is not None and fav_rate > mix_rate)
        q7_table.append({
            "cohort": cohort,
            "n_eligible": len(cohort_elig),
            "n_fav": len(fav_c),
            "n_mix": len(mix_c),
            "fav_rate": fav_rate,
            "mix_rate": mix_rate,
            "fav_beats_mix": fav_beats_mix,
        })

    # Eligibility accounting table
    eligibility_table = []
    for cohort in sorted(set(r["cohort_date"] for r in rows)):
        for ec in ("Favourable", "Mixed", "Unfavourable"):
            subset = [r for r in rows if r["cohort_date"] == cohort and r["evidence_class"] == ec]
            elig = [r for r in subset if r.get("eligible_for_primary_comparison") is True]
            inelig = [r for r in subset if r.get("eligible_for_primary_comparison") is not True]
            reasons = []
            for r in inelig:
                exit_r = r.get("exit_reason", "")
                cert = r.get("certification_status", "")
                if exit_r in INELIGIBLE_EXIT_REASONS:
                    reasons.append(exit_r)
                elif cert not in ELIGIBLE_CERT_STATUSES:
                    reasons.append(f"cert={cert}")
                elif ec not in ELIGIBLE_EVIDENCE_CLASSES:
                    reasons.append("evidence_class_excluded")
            reason_str = ", ".join(sorted(set(reasons))) if reasons else "—"
            if subset:
                eligibility_table.append({
                    "cohort": cohort, "ec": ec,
                    "n_total": len(subset), "n_eligible": len(elig),
                    "n_ineligible": len(inelig), "reasons": reason_str,
                })

    # ── Build context and write outputs ──────────────────────────────────────
    run_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S%.6fZ")
    run_id = f"TIME010-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"

    n_cohort_dates_estimable = q1_n_estimable  # same for Q1 and Q2

    ctx = {
        "run_at": run_at,
        "run_id": run_id,
        "dataset_source": str(dataset_path),
        "producer": PRODUCER,
        "n_total_rows": n_total,
        "n_eligible_rows": n_eligible,
        "n_favourable_eligible": len(fav),
        "n_mixed_eligible": len(mix),
        "n_cohort_dates": n_cohort_dates,
        "n_cohort_dates_estimable": n_cohort_dates_estimable,
        "q1_n_fav": q1_n_fav, "q1_k_fav": q1_k_fav,
        "q1_n_mix": q1_n_mix, "q1_k_mix": q1_k_mix,
        "q1_rate_fav": q1_rate_fav, "q1_rate_mix": q1_rate_mix,
        "q1_ci_fav": q1_ci_fav, "q1_ci_mix": q1_ci_mix,
        "q1_z": q1_z, "q1_p": q1_p,
        "q1_significant": q1_significant,
        "q1_n_consistent": q1_n_consistent, "q1_n_estimable": q1_n_estimable,
        "q1_threshold": q1_threshold, "q1_cohort_consistent": q1_cohort_consistent,
        "q2_n_fav": q2_n_fav, "q2_n_mix": q2_n_mix,
        "q2_mean_fav": q2_mean_fav, "q2_mean_mix": q2_mean_mix,
        "q2_median_fav": q2_median_fav, "q2_median_mix": q2_median_mix,
        "q2_sd_fav": q2_sd_fav, "q2_sd_mix": q2_sd_mix,
        "q2_ci_fav": q2_ci_fav, "q2_ci_mix": q2_ci_mix,
        "q2_u": q2_u, "q2_p": q2_p, "q2_cles": q2_cles,
        "q2_significant": q2_significant,
        "q2_n_consistent": q2_n_consistent, "q2_n_estimable": q2_n_estimable,
        "q2_threshold": q2_threshold, "q2_cohort_consistent": q2_cohort_consistent,
        "research_conclusion": conclusion,
        "frozen_conclusion_text": frozen_text,
        "q3_table": q3_table,
        "q4_table": q4_table,
        "q5_table": q5_table,
        "q6_table": dict(q6_table),
        "q7_table": q7_table,
        "eligibility_table": eligibility_table,
    }

    output_dir.mkdir(parents=True, exist_ok=True)

    # Write analysis report
    report_path = output_dir / "analysis_report.md"
    write_report(report_path, ctx)
    print(f"[time010] report written: {report_path}")

    # Write latest_run.json
    summary = {
        "experiment_id": "TIME010",
        "run_id": run_id,
        "run_at": run_at,
        "producer": PRODUCER,
        "dataset_source": str(dataset_path),
        "n_total_rows": n_total,
        "n_eligible_rows": n_eligible,
        "n_cohort_dates": n_cohort_dates,
        "n_favourable_eligible": len(fav),
        "n_mixed_eligible": len(mix),
        "q1_target_rate_favourable": q1_rate_fav,
        "q1_target_rate_mixed": q1_rate_mix,
        "q1_p_value": q1_p,
        "q1_significant": q1_significant,
        "q1_cohort_consistency_n": q1_n_consistent,
        "q1_cohort_consistency_threshold": q1_threshold,
        "q1_cohort_consistent": q1_cohort_consistent,
        "q2_return_mean_favourable": q2_mean_fav,
        "q2_return_mean_mixed": q2_mean_mix,
        "q2_p_value": q2_p,
        "q2_significant": q2_significant,
        "q2_cohort_consistency_n": q2_n_consistent,
        "q2_cohort_consistency_threshold": q2_threshold,
        "q2_cohort_consistent": q2_cohort_consistent,
        "research_conclusion": conclusion,
        "conclusion_frozen": conclusion != "NOT_ESTIMABLE",
        "protocol_version": PROTOCOL_VERSION,
        "bootstrap_seed": 42,  # deterministic seed for Q2 bootstrap CI (section 7a)
        "prohibited_actions_acknowledged": True,
    }
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(summary, indent=2, default=str))
    print(f"[time010] summary written: {summary_path}")
    print(f"[time010] result=OK research_conclusion={conclusion}")
    return 0


# ── Helper: write NOT_ESTIMABLE summary ──────────────────────────────────────

def _write_not_estimable(output_dir, summary_path, dataset_path, n_total, n_eligible, conclusion):
    """Write a non-final NOT_ESTIMABLE status.

    conclusion_frozen is explicitly False so that subsequent runs (as more
    COMPLETE observations accumulate) are not blocked. The immutable conclusion
    slot is only consumed when a final estimable result is produced.
    See TIME-010 protocol section 7 (NOT_ESTIMABLE execution semantics).
    """
    run_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S%.6fZ")
    run_id = f"TIME010-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
    output_dir.mkdir(parents=True, exist_ok=True)
    summary = {
        "experiment_id": "TIME010",
        "run_id": run_id,
        "run_at": run_at,
        "producer": PRODUCER,
        "dataset_source": str(dataset_path),
        "n_total_rows": n_total,
        "n_eligible_rows": n_eligible,
        "research_conclusion": conclusion,
        "conclusion_frozen": False,   # NOT_ESTIMABLE never freezes the conclusion
        "protocol_version": PROTOCOL_VERSION,
        "not_estimable_reason": f"n_eligible={n_eligible} < {MIN_ELIGIBLE_ROWS}",
    }
    summary_path.write_text(json.dumps(summary, indent=2))
    print(f"[time010] summary written (non-final, conclusion_frozen=false): {summary_path}")


# ── Helper: build frozen conclusion text ─────────────────────────────────────

def _build_frozen_conclusion(
    conclusion,
    q1_rate_fav, q1_rate_mix, q1_p, q1_sig, q1_consistent,
    q2_mean_fav, q2_mean_mix, q2_p, q2_sig, q2_consistent,
    n_eligible, n_cohort_dates,
):
    def pct(v): return f"{v*100:.1f}%" if v is not None else "N/A"
    def fp(v): return f"{v:.4f}" if v is not None else "N/A"
    def fm(v): return f"{v:.4f}" if v is not None else "N/A"

    if conclusion == "NOT_ESTIMABLE":
        return (
            f"TIME-010 could not produce a research conclusion because the eligible dataset "
            f"contained {n_eligible} rows across {n_cohort_dates} cohort dates, "
            f"below the pre-specified minimum of {MIN_ELIGIBLE_ROWS} rows and {MIN_COHORT_DATES} cohort dates."
        )
    if conclusion == "POSITIVE":
        return (
            f"TIME-010 found that the frozen Coralys evidence classification demonstrates "
            f"statistically significant and cohort-consistent prospective discrimination of forward outcomes. "
            f"Favourable decisions achieved a target attainment rate of {pct(q1_rate_fav)} vs {pct(q1_rate_mix)} for Mixed "
            f"(p={fp(q1_p)}, cohort-consistent={q1_consistent}), and a mean realized return of {fm(q2_mean_fav)} vs {fm(q2_mean_mix)} "
            f"(p={fp(q2_p)}, cohort-consistent={q2_consistent}). "
            f"These findings are based on {n_eligible} eligible prospective observations across {n_cohort_dates} cohort dates."
        )
    if conclusion == "PARTIAL":
        return (
            f"TIME-010 found partial evidence that the frozen Coralys evidence classification "
            f"discriminates forward outcomes prospectively. "
            f"Q1 (target attainment): Favourable {pct(q1_rate_fav)} vs Mixed {pct(q1_rate_mix)}, p={fp(q1_p)}, significant={q1_sig}, cohort-consistent={q1_consistent}. "
            f"Q2 (realized return): Favourable mean {fm(q2_mean_fav)} vs Mixed {fm(q2_mean_mix)}, p={fp(q2_p)}, significant={q2_sig}, cohort-consistent={q2_consistent}. "
            f"A PARTIAL result does not justify modification of Coralys or claims of predictive utility."
        )
    if conclusion == "NEGATIVE":
        return (
            f"TIME-010 did not find statistically significant prospective discrimination of forward outcomes "
            f"by the frozen Coralys evidence classification. "
            f"Q1 (target attainment): Favourable {pct(q1_rate_fav)} vs Mixed {pct(q1_rate_mix)}, p={fp(q1_p)}. "
            f"Q2 (realized return): Favourable mean {fm(q2_mean_fav)} vs Mixed {fm(q2_mean_mix)}, p={fp(q2_p)}. "
            f"These findings do not justify modification of Coralys, retrospective threshold selection, "
            f"or claims of predictive or economic utility. "
            f"Based on {n_eligible} eligible prospective observations across {n_cohort_dates} cohort dates."
        )
    if conclusion == "INCONCLUSIVE":
        return (
            f"TIME-010 produced an inconclusive result: one or both primary endpoints showed statistical "
            f"significance but the pre-specified cohort consistency criterion was not met. "
            f"Q1: p={fp(q1_p)}, significant={q1_sig}, cohort-consistent={q1_consistent}. "
            f"Q2: p={fp(q2_p)}, significant={q2_sig}, cohort-consistent={q2_consistent}. "
            f"An INCONCLUSIVE result does not justify modification of Coralys."
        )
    return f"TIME-010 conclusion: {conclusion}."


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    sys.exit(main())