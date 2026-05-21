#!/usr/bin/env python3
"""
Ecology validation — falsification checks on existing atlas artifacts only.

No new mechanics. Reads ecology_signatures.jsonl + ecology_transition_graph.json.

Usage:
  python3 scripts/ecology_validate.py --cohort 003: 900:replay_equiv
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from statistics import mean

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from ecology_cohort_compare import parse_cohort
from ecology_transition_atlas import (
    PhaseCalibration,
    calibrate_from_windows,
    classify_ecology_phase,
    load_signatures,
)
from run_nse_cohort import resolve_archive_dir


def diagnose_classification(windows: list[dict]) -> dict:
    """Which heuristic rules fire — fixed vs cohort-relative survival threshold."""
    cal = calibrate_from_windows(windows)
    rule_hits_fixed = Counter()
    rule_hits_calibrated = Counter()
    phases_fixed = []
    phases_cal = []
    surv_mins = []

    for i, w in enumerate(windows):
        prev = windows[i - 1] if i > 0 else None
        cm = w.get("compression_metrics", {})
        pt = w.get("propagation_texture", {})
        ts = w.get("temporal_structure", {})
        vol = w.get("volatility_texture", {})

        low_v = float(cm.get("low_velocity_fraction", 0))
        corridor = float(w.get("corridor_rate", 0))
        survival_min = float(ts.get("survival_min") or ts.get("survival_mean") or 1.0)
        surv_mins.append(survival_min)
        vel = float(vol.get("velocity_mean") or 0)
        prec_exp = float(cm.get("precursor_entropy_expansion_mean") or 0)
        density = float(cm.get("density_thinning_mean") or 0)
        prev_corridor = float((prev or {}).get("corridor_rate", 0))

        if survival_min < 0.55:
            rule_hits_fixed["EXHAUSTION_survival_min"] += 1
        if survival_min < cal.survival_exhaustion_threshold:
            rule_hits_calibrated["EXHAUSTION_survival_min"] += 1
        if corridor < 0.35 and prev_corridor > 0.6:
            rule_hits_fixed["EXHAUSTION_corridor_drop"] += 1
            rule_hits_calibrated["EXHAUSTION_corridor_drop"] += 1

        phases_fixed.append(classify_ecology_phase(w, prev, PhaseCalibration()))
        phases_cal.append(classify_ecology_phase(w, prev, cal))

    corridor_rates = [float(w.get("corridor_rate", 0)) for w in windows]

    return {
        "phase_counts_calibrated": dict(Counter(phases_cal)),
        "phase_counts_fixed_0.55": dict(Counter(phases_fixed)),
        "cohort_survival_threshold_p10": cal.survival_exhaustion_threshold,
        "rule_hits_fixed_0.55": dict(rule_hits_fixed),
        "rule_hits_calibrated": dict(rule_hits_calibrated),
        "survival_min_distribution": {
            "mean": round(mean(surv_mins), 4),
            "below_0.55": sum(1 for s in surv_mins if s < 0.55),
            "below_0.55_pct": round(sum(1 for s in surv_mins if s < 0.55) / max(len(surv_mins), 1), 4),
            "below_cohort_p10": sum(1 for s in surv_mins if s < cal.survival_exhaustion_threshold),
            "below_cohort_p10_pct": round(
                sum(1 for s in surv_mins if s < cal.survival_exhaustion_threshold)
                / max(len(surv_mins), 1),
                4,
            ),
        },
        "corridor_rate_mean": round(mean(corridor_rates), 4) if corridor_rates else None,
    }


def dwell_hazard_by_phase(windows: list[dict]) -> dict:
    cal = calibrate_from_windows(windows)
    by_phase: dict[str, list[dict]] = defaultdict(list)
    for i, w in enumerate(windows):
        prev = windows[i - 1] if i > 0 else None
        phase = classify_ecology_phase(w, prev, cal)
        ts = w.get("temporal_structure", {})
        by_phase[phase].append(
            {
                "dwell": float(ts.get("dwell_mean") or 0),
                "hazard": float(ts.get("hazard_mean") or 0),
                "survival": float(ts.get("survival_mean") or 0),
            }
        )

    out = {}
    for phase, rows in by_phase.items():
        out[phase] = {
            "n": len(rows),
            "dwell_mean": round(mean(r["dwell"] for r in rows), 4) if rows else None,
            "hazard_mean": round(mean(r["hazard"] for r in rows), 4) if rows else None,
            "survival_mean": round(mean(r["survival"] for r in rows), 4) if rows else None,
        }
    return out


def curvature_sanity(graph: dict) -> dict:
    series = graph.get("phase_curvature_series", [])
    if len(series) < 3:
        return {"status": "insufficient_data"}

    steep_at_change = 0
    steep_not_change = 0
    for i in range(1, len(series)):
        phase_changed = series[i].get("phase") != series[i - 1].get("phase")
        steep = series[i].get("transition_steepness", 0) > 1.5
        if phase_changed and steep:
            steep_at_change += 1
        elif not phase_changed and steep:
            steep_not_change += 1

    return {
        "steep_at_phase_change": steep_at_change,
        "steep_without_phase_change": steep_not_change,
        "interpretation": (
            "curvature_leads_transitions"
            if steep_at_change > steep_not_change
            else "curvature_mostly_noise"
            if steep_not_change > steep_at_change * 2
            else "mixed"
        ),
    }


def compression_validation(graph: dict) -> dict:
    lineages = graph.get("compression_lineages_top", [])
    windows = graph.get("windows", [])
    comp_runs = []
    run = 0
    for w in windows:
        if w.get("ecology_phase") == "COMPRESSION":
            run += 1
        elif run:
            comp_runs.append(run)
            run = 0
    if run:
        comp_runs.append(run)

    return {
        "compression_barrier_count": sum(1 for w in windows if w.get("ecology_phase") == "COMPRESSION"),
        "compression_run_lengths": comp_runs,
        "compression_run_mean_bars": round(mean(comp_runs), 3) if comp_runs else None,
        "lineage_paths": lineages,
    }


def validate_cohort(batch_id: int, run_label: str) -> dict:
    archive = resolve_archive_dir(batch_id, False, run_label)
    name = f"batch_{batch_id:03d}" + (f"/{run_label}" if run_label else "")
    sig_path = archive / "metadata" / "ecology_signatures.jsonl"
    graph_path = archive / "metadata" / "ecology_transition_graph.json"

    windows = load_signatures(sig_path)
    graph = json.loads(graph_path.read_text()) if graph_path.exists() else {}

    return {
        "cohort": name,
        "barrier_count": len(windows),
        "classification_diagnosis": diagnose_classification(windows),
        "dwell_hazard_by_phase": dwell_hazard_by_phase(windows),
        "curvature_sanity": curvature_sanity(graph),
        "compression_validation": compression_validation(graph),
        "transition_top": graph.get("transitions", [])[:8],
        "mortality_proxy": graph.get("mortality_proxy_exhaustion_rate"),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ecology atlas artifacts")
    parser.add_argument(
        "--cohort",
        nargs="+",
        default=["003:", "900:replay_equiv"],
    )
    parser.add_argument(
        "--out",
        default="state_archive/metadata/ecology_validation_report.json",
    )
    args = parser.parse_args()

    reports = []
    for spec in args.cohort:
        bid, label = parse_cohort(spec)
        sig = resolve_archive_dir(bid, False, label) / "metadata" / "ecology_signatures.jsonl"
        if not sig.exists():
            print(f"⚠️  skip {spec}: no signatures", file=sys.stderr)
            continue
        reports.append(validate_cohort(bid, label))

    payload = {
        "mode": "consolidation_and_falsification",
        "cohorts": reports,
        "built_at_utc": datetime.now(timezone.utc).isoformat(),
    }

    if len(reports) >= 2:
        payload["cross_cohort_note"] = (
            "Use phase_counts_calibrated (cohort p10 survival) not fixed 0.55; "
            "see survival_min_distribution per cohort"
        )

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "w") as f:
        json.dump(payload, f, indent=2)

    print("=" * 60)
    print("ECOLOGY VALIDATION (falsification mode)")
    print("=" * 60)
    for r in reports:
        d = r["classification_diagnosis"]
        print(f"\n  {r['cohort']} ({r['barrier_count']} barriers)")
        print(f"    phases (calibrated p10): {d['phase_counts_calibrated']}")
        print(f"    phases (fixed 0.55):     {d['phase_counts_fixed_0.55']}")
        print(f"    cohort survival p10 threshold: {d['cohort_survival_threshold_p10']}")
        sm = d["survival_min_distribution"]
        print(f"    survival_min<0.55: {sm['below_0.55']}/{r['barrier_count']} ({sm['below_0.55_pct']:.1%})")
        print(
            f"    survival_min<p10: {sm['below_cohort_p10']}/{r['barrier_count']} "
            f"({sm['below_cohort_p10_pct']:.1%})"
        )
        print(f"    curvature: {r['curvature_sanity'].get('interpretation')}")
        cv = r["compression_validation"]
        print(f"    compression barriers: {cv['compression_barrier_count']} runs={cv['compression_run_lengths']}")
    print(f"\n  Report: {out}")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
