#!/usr/bin/env python3
"""
Ecology Transition Atlas — Phase 1b: persistence, transitions, compression lineages.

Reads metadata/ecology_signatures.jsonl (run ecology_signature_atlas.py first).
Outcome-agnostic; no PnL labels.

Outputs:
  metadata/ecology_transition_graph.json

Usage:
  python3 scripts/ecology_transition_atlas.py --batch-id 900 --run-label replay_equiv
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from dataclasses import dataclass
from statistics import mean

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from run_nse_cohort import resolve_archive_dir


def load_signatures(path: Path) -> list[dict]:
    return [json.loads(ln) for ln in path.read_text().splitlines() if ln.strip()]


def euclidean(a: list[float], b: list[float]) -> float:
    n = min(len(a), len(b))
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(n)))


@dataclass
class PhaseCalibration:
    """Cohort-relative thresholds — stability, not artificial class balance."""

    survival_exhaustion_quantile: float = 0.10
    survival_exhaustion_threshold: float = 0.55  # set per cohort from signatures


def _survival_min_value(w: dict) -> float:
    ts = w.get("temporal_structure", {})
    return float(ts.get("survival_min") or ts.get("survival_mean") or 1.0)


def calibrate_from_windows(windows: list[dict], quantile: float = 0.10) -> PhaseCalibration:
    vals = sorted(_survival_min_value(w) for w in windows)
    if not vals:
        return PhaseCalibration(survival_exhaustion_quantile=quantile, survival_exhaustion_threshold=0.55)
    idx = max(0, int(quantile * (len(vals) - 1)))
    return PhaseCalibration(
        survival_exhaustion_quantile=quantile,
        survival_exhaustion_threshold=round(vals[idx], 6),
    )


def classify_ecology_phase(
    w: dict, prev: dict | None, cal: PhaseCalibration | None = None
) -> str:
    """Heuristic propagation phase — discovery labels, not supervised outcomes."""
    cal = cal or PhaseCalibration()
    cm = w.get("compression_metrics", {})
    pt = w.get("propagation_texture", {})
    ts = w.get("temporal_structure", {})
    vol = w.get("volatility_texture", {})

    low_v = float(cm.get("low_velocity_fraction", 0))
    corridor = float(w.get("corridor_rate", 0))
    survival_min = _survival_min_value(w)
    vel = float(vol.get("velocity_mean") or 0)
    prec_exp = float(cm.get("precursor_entropy_expansion_mean") or 0)
    density = float(cm.get("density_thinning_mean") or 0)

    if low_v >= 0.55 and corridor < 0.65 and density >= 0.5:
        return "COMPRESSION"
    if survival_min < cal.survival_exhaustion_threshold or (
        corridor < 0.35 and (prev or {}).get("corridor_rate", 0) > 0.6
    ):
        return "EXHAUSTION"
    if corridor >= 0.8 and vel >= 0.04:
        return "PERSISTENCE"
    if prec_exp > 0.015 or (vel > 0.08 and low_v < 0.4):
        return "EXPANSION"
    return "TRANSITIONAL"


def persistence_runs(phases: list[str]) -> dict[str, list[int]]:
    """Run lengths per phase label."""
    runs: dict[str, list[int]] = defaultdict(list)
    if not phases:
        return runs
    cur = phases[0]
    length = 1
    for p in phases[1:]:
        if p == cur:
            length += 1
        else:
            runs[cur].append(length)
            cur = p
            length = 1
    runs[cur].append(length)
    return {k: v for k, v in runs.items()}


def build_transition_graph(windows: list[dict], survival_quantile: float = 0.10) -> dict:
    cal = calibrate_from_windows(windows, survival_quantile)
    phases = []
    for i, w in enumerate(windows):
        prev = windows[i - 1] if i > 0 else None
        phases.append(classify_ecology_phase(w, prev, cal))

    # Annotate each window
    annotated = []
    for w, phase in zip(windows, phases):
        annotated.append({"ts": w["ts"], "ecology_phase": phase, "dominant_state": w["dominant_state"]})

    # Transition counts + persistence after transition
    trans_counts: Counter[tuple[str, str]] = Counter()
    trans_persist_after: dict[tuple[str, str], list[int]] = defaultdict(list)

    for i in range(1, len(phases)):
        a, b = phases[i - 1], phases[i]
        trans_counts[(a, b)] += 1
        run_len = 1
        j = i
        while j < len(phases) and phases[j] == b:
            run_len += 1
            j += 1
        trans_persist_after[(a, b)].append(run_len)

    transitions = []
    for (a, b), count in sorted(trans_counts.items()):
        pers = trans_persist_after[(a, b)]
        transitions.append(
            {
                "from": a,
                "to": b,
                "count": count,
                "avg_persistence_bars": round(mean(pers), 3) if pers else 0.0,
                "max_persistence_bars": max(pers) if pers else 0,
            }
        )

    runs = persistence_runs(phases)
    persistence_summary = {
        phase: {
            "run_count": len(lengths),
            "avg_run_bars": round(mean(lengths), 3) if lengths else 0.0,
            "max_run_bars": max(lengths) if lengths else 0,
        }
        for phase, lengths in runs.items()
    }

    # Nearest ecological neighbor (feature space) between adjacent barriers
    neighbors = []
    for i in range(1, len(windows)):
        dist = euclidean(windows[i - 1]["feature_vector"], windows[i]["feature_vector"])
        neighbors.append(
            {
                "ts_from": windows[i - 1]["ts"],
                "ts_to": windows[i]["ts"],
                "feature_distance": round(dist, 6),
                "phase_from": phases[i - 1],
                "phase_to": phases[i],
            }
        )

    # Compression → expansion lineage paths
    lineage_counts: Counter[str] = Counter()
    for i in range(2, len(phases)):
        trip = f"{phases[i-2]}->{phases[i-1]}->{phases[i]}"
        if phases[i - 2] == "COMPRESSION":
            lineage_counts[trip] += 1

    compression_lineages = [
        {"path": path, "count": c} for path, c in lineage_counts.most_common(20)
    ]

    # Phase-space curvature: velocity & acceleration in feature_vector trajectory
    curvatures = []
    for i in range(1, len(windows)):
        v_prev = windows[i - 1]["feature_vector"]
        v_cur = windows[i]["feature_vector"]
        v_next = windows[i + 1]["feature_vector"] if i + 1 < len(windows) else v_cur
        d1 = euclidean(v_prev, v_cur)
        d2 = euclidean(v_cur, v_next)
        accel = round(d2 - d1, 6)
        steep = round(abs(d2 - d1) / max(d1, 1e-9), 4)
        curvatures.append(
            {
                "ts": windows[i]["ts"],
                "velocity": round(d1, 6),
                "acceleration": accel,
                "transition_steepness": steep,
                "phase": phases[i],
            }
        )
    curvature_summary = {
        "velocity_mean": round(mean(c["velocity"] for c in curvatures), 6) if curvatures else None,
        "velocity_p90": round(sorted(c["velocity"] for c in curvatures)[int(0.9 * (len(curvatures) - 1))], 6)
        if curvatures
        else None,
        "acceleration_mean": round(mean(c["acceleration"] for c in curvatures), 6)
        if curvatures
        else None,
        "steep_transition_count": sum(1 for c in curvatures if c["transition_steepness"] > 1.5),
    }

    phase_counts = dict(Counter(phases))
    mortality_proxy = round(
        sum(1 for p in phases if p == "EXHAUSTION") / max(len(phases), 1), 4
    )
    recurrence = {
        phase: round(sum(1 for p in phases if p == phase) / max(len(phases), 1), 4)
        for phase in phase_counts
    }

    return {
        "barrier_count": len(windows),
        "ts_first": windows[0]["ts"] if windows else None,
        "ts_last": windows[-1]["ts"] if windows else None,
        "phase_calibration": {
            "survival_exhaustion_quantile": cal.survival_exhaustion_quantile,
            "survival_exhaustion_threshold": cal.survival_exhaustion_threshold,
            "note": "cohort-relative; bottom quantile of survival_min marks EXHAUSTION",
        },
        "phase_counts": phase_counts,
        "phase_recurrence_density": recurrence,
        "mortality_proxy_exhaustion_rate": mortality_proxy,
        "persistence_summary": persistence_summary,
        "transitions": transitions,
        "adjacent_feature_neighbors": neighbors[:50],
        "adjacent_neighbor_distance_mean": round(
            mean(n["feature_distance"] for n in neighbors), 6
        )
        if neighbors
        else None,
        "compression_lineages_top": compression_lineages,
        "ecology_phase_curvature": curvature_summary,
        "phase_curvature_series": curvatures[:100],
        "windows": annotated,
        "built_at_utc": datetime.now(timezone.utc).isoformat(),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Ecology transition & persistence graph")
    parser.add_argument("--batch-id", type=int, default=0)
    parser.add_argument("--run-label", default="")
    parser.add_argument("--archive-dir", default="")
    parser.add_argument(
        "--survival-quantile",
        type=float,
        default=0.10,
        help="Cohort-relative survival_min quantile for EXHAUSTION (default p10)",
    )
    args = parser.parse_args()

    if args.archive_dir:
        archive_dir = Path(args.archive_dir)
    elif args.batch_id:
        archive_dir = resolve_archive_dir(args.batch_id, False, args.run_label)
    else:
        print("❌ Provide --archive-dir or --batch-id", file=sys.stderr)
        return 1

    sig_path = archive_dir / "metadata" / "ecology_signatures.jsonl"
    if not sig_path.exists():
        print(f"❌ Missing {sig_path} — run ecology_signature_atlas.py first", file=sys.stderr)
        return 1

    windows = load_signatures(sig_path)
    graph = build_transition_graph(windows, args.survival_quantile)

    out_path = archive_dir / "metadata" / "ecology_transition_graph.json"
    with open(out_path, "w") as f:
        json.dump(graph, f, indent=2)

    print("=" * 60)
    print("ECOLOGY TRANSITION ATLAS")
    print("=" * 60)
    print(f"  Archive     : {archive_dir}")
    print(f"  Barriers    : {graph['barrier_count']}")
    pc = graph.get("phase_calibration", {})
    print(f"  Calibration : survival p{int(pc.get('survival_exhaustion_quantile', 0.1)*100)} "
          f"threshold={pc.get('survival_exhaustion_threshold')}")
    print(f"  Phases      : {graph['phase_counts']}")
    print(f"  Exhaustion μ: {graph['mortality_proxy_exhaustion_rate']}")
    print(f"  Top transitions:")
    for t in graph["transitions"][:8]:
        print(
            f"    {t['from']} → {t['to']}: {t['count']} "
            f"(avg persist {t['avg_persistence_bars']} bars)"
        )
    print(f"  Output      : {out_path}")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
