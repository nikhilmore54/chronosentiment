#!/usr/bin/env python3
"""
Ecology Signature Atlas — Phase 1: extract propagation fingerprints from replay archives.

Produces first-class observability artifacts (no new causal mechanics):
  metadata/ecology_signatures.jsonl   — one row per synchronized barrier (ts)
  metadata/ecology_atlas_summary.json — cohort-level aggregates + transition sketch

Usage:
  python3 scripts/ecology_signature_atlas.py --batch-id 900 --run-label replay_equiv
  python3 scripts/ecology_signature_atlas.py --archive-dir state_archive/batches/batch_003
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from statistics import mean, pstdev

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from run_nse_cohort import resolve_archive_dir
from verify_cohort_baseline import iter_archive_records, load_manifest


def _safe_mean(vals: list[float]) -> float | None:
    return round(mean(vals), 6) if vals else None


def _safe_stdev(vals: list[float]) -> float | None:
    return round(pstdev(vals), 6) if len(vals) > 1 else 0.0 if vals else None


def _pct(vals: list[float], p: float) -> float | None:
    if not vals:
        return None
    s = sorted(vals)
    i = min(len(s) - 1, max(0, int(p * (len(s) - 1))))
    return round(s[i], 6)


def window_signature(ts: int, records: list[dict]) -> dict:
    """Aggregate ecology fingerprint for one synchronized barrier."""
    n = len(records)
    corridors = [r for r in records if r.get("corridor")]
    corridor_rate = len(corridors) / n if n else 0.0

    velocities = [float(r["velocity"]) for r in records]
    accelerations = [float(r["acceleration"]) for r in records]
    turn_angles = [float(r["turn_angle"]) for r in records]
    entropies = [float(r["entropy"]) for r in records]
    hazards = [float(r["hazard_rate"]) for r in records]
    survivals = [float(r["survival_probability"]) for r in records]
    dwells = [float(r.get("dwell_duration", 0)) for r in records]
    queue_p = [float(r.get("queue_pressure", 0)) for r in records]
    spread_e = [float(r.get("spread_elasticity", 0)) for r in records]
    pc1s = [float(r["pc1"]) for r in records]
    pc2s = [float(r["pc2"]) for r in records]
    dists = [float(r.get("dist_to_centroid", 0)) for r in records]

    prec_entropy = [float(r.get("precursor_entropy_expansion", 0)) for r in records]
    prec_density = [float(r.get("precursor_density_thinning", 0)) for r in records]
    prec_curv = [float(r.get("precursor_curvature_destabilization", 0)) for r in records]
    prec_decay = [float(r.get("precursor_decay_velocity", 0)) for r in records]

    states = Counter(r.get("state", "") for r in records)
    instabilities = Counter(r.get("instability_type", "STABLE") for r in records)

    # Cross-asset coherence proxies at this barrier
    pc1_spread = (max(pc1s) - min(pc1s)) if pc1s else 0.0
    pc2_spread = (max(pc2s) - min(pc2s)) if pc2s else 0.0
    if len(pc1s) > 1:
        m1, m2 = mean(pc1s), mean(pc2s)
        cov = sum((a - m1) * (b - m2) for a, b in zip(pc1s, pc2s)) / len(pc1s)
        v1 = sum((a - m1) ** 2 for a in pc1s) / len(pc1s)
        v2 = sum((b - m2) ** 2 for b in pc2s) / len(pc2s)
        propagation_corr = cov / math.sqrt(v1 * v2) if v1 > 1e-12 and v2 > 1e-12 else 0.0
    else:
        propagation_corr = 0.0

    # Compression / pre-release proxies
    low_velocity_frac = sum(1 for v in velocities if v < 0.05) / n if n else 0.0
    entropy_collapse = _safe_mean(entropies) or 0.0
    density_thin_mean = _safe_mean(prec_density) or 0.0

    # Ecological entropy: state + instability distribution
    state_entropy = 0.0
    for c in states.values():
        p = c / n
        if p > 0:
            state_entropy -= p * math.log(p)
    inst_entropy = 0.0
    for c in instabilities.values():
        p = c / n
        if p > 0:
            inst_entropy -= p * math.log(p)

    dominant_state = states.most_common(1)[0][0] if states else "UNKNOWN"
    dominant_instability = instabilities.most_common(1)[0][0] if instabilities else "STABLE"

    return {
        "ts": ts,
        "symbol_count": n,
        "corridor_rate": round(corridor_rate, 4),
        "dominant_state": dominant_state,
        "dominant_instability": dominant_instability,
        "state_entropy": round(state_entropy, 4),
        "instability_entropy": round(inst_entropy, 4),
        "volatility_texture": {
            "velocity_mean": _safe_mean(velocities),
            "velocity_stdev": _safe_stdev(velocities),
            "acceleration_mean": _safe_mean(accelerations),
            "turn_angle_mean": _safe_mean(turn_angles),
            "turn_angle_p90": _pct(turn_angles, 0.9),
            "entropy_mean": _safe_mean(entropies),
            "entropy_stdev": _safe_stdev(entropies),
        },
        "propagation_texture": {
            "corridor_rate": round(corridor_rate, 4),
            "dist_to_centroid_mean": _safe_mean(dists),
            "continuation_density": round(sum(1 for v in velocities if v > 0.05) / n, 4) if n else 0.0,
            "directional_smoothness": round(1.0 - min(1.0, (_safe_mean(turn_angles) or 0) / 180.0), 4),
            "topology_elasticity": _safe_stdev(dists),
        },
        "temporal_structure": {
            "dwell_mean": _safe_mean(dwells),
            "hazard_mean": _safe_mean(hazards),
            "survival_mean": _safe_mean(survivals),
            "survival_min": round(min(survivals), 4) if survivals else None,
            "decay_velocity_mean": _safe_mean(prec_decay),
        },
        "participation_structure": {
            "queue_pressure_mean": _safe_mean(queue_p),
            "spread_elasticity_mean": _safe_mean(spread_e),
            "spread_elasticity_p90": _pct(spread_e, 0.9),
        },
        "cross_asset_ecology": {
            "pc1_spread": round(pc1_spread, 4),
            "pc2_spread": round(pc2_spread, 4),
            "pc1_pc2_correlation": round(propagation_corr, 4),
        },
        "compression_metrics": {
            "low_velocity_fraction": round(low_velocity_frac, 4),
            "entropy_collapse_proxy": round(entropy_collapse, 4),
            "density_thinning_mean": round(density_thin_mean, 4),
            "curvature_destabilization_mean": _safe_mean(prec_curv),
            "precursor_entropy_expansion_mean": _safe_mean(prec_entropy),
        },
        "feature_vector": [
            corridor_rate,
            _safe_mean(velocities) or 0.0,
            _safe_stdev(velocities) or 0.0,
            _safe_mean(turn_angles) or 0.0,
            _safe_mean(entropies) or 0.0,
            _safe_mean(hazards) or 0.0,
            _safe_mean(survivals) or 0.0,
            pc1_spread,
            pc2_spread,
            propagation_corr,
            low_velocity_frac,
            density_thin_mean,
            state_entropy,
            inst_entropy,
        ],
    }


def build_transition_sketch(windows: list[dict]) -> dict:
    """Dominant-state transitions between consecutive barriers."""
    transitions: Counter[tuple[str, str]] = Counter()
    for i in range(1, len(windows)):
        a = windows[i - 1]["dominant_state"]
        b = windows[i]["dominant_state"]
        transitions[(a, b)] += 1
    return {
        "transition_counts": {f"{a}->{b}": c for (a, b), c in transitions.items()},
        "total_steps": max(0, len(windows) - 1),
    }


def extract_atlas(archive_dir: Path, cohort: set[str] | None) -> tuple[list[dict], dict]:
    by_ts: dict[int, list[dict]] = defaultdict(list)
    for item in iter_archive_records(archive_dir, cohort):
        if not item["ok"]:
            continue
        by_ts[int(item["record"]["ts"])].append(item["record"])

    windows = [window_signature(ts, recs) for ts, recs in sorted(by_ts.items())]

    summary = {
        "archive_dir": str(archive_dir),
        "barrier_count": len(windows),
        "ts_first": windows[0]["ts"] if windows else None,
        "ts_last": windows[-1]["ts"] if windows else None,
        "corridor_rate_mean": _safe_mean([w["corridor_rate"] for w in windows]),
        "corridor_rate_stdev": _safe_stdev([w["corridor_rate"] for w in windows]),
        "state_entropy_mean": _safe_mean([w["state_entropy"] for w in windows]),
        "dominant_state_counts": dict(
            Counter(w["dominant_state"] for w in windows)
        ),
        "transitions": build_transition_sketch(windows),
        "extracted_at_utc": datetime.now(timezone.utc).isoformat(),
    }
    return windows, summary


def main() -> int:
    parser = argparse.ArgumentParser(description="Ecology signature atlas — Phase 1 extraction")
    parser.add_argument("--batch-id", type=int, default=0)
    parser.add_argument("--run-label", default="")
    parser.add_argument("--archive-dir", default="", help="Override archive path")
    args = parser.parse_args()

    if args.archive_dir:
        archive_dir = Path(args.archive_dir)
    elif args.batch_id:
        archive_dir = resolve_archive_dir(args.batch_id, False, args.run_label)
    else:
        print("❌ Provide --archive-dir or --batch-id", file=sys.stderr)
        return 1

    if not (archive_dir / "raw").exists():
        print(f"❌ No raw archive at {archive_dir}", file=sys.stderr)
        return 1

    manifest = load_manifest(archive_dir, args.run_label or None)
    windows, summary = extract_atlas(archive_dir, cohort=None)
    if not windows:
        print("❌ No synchronized barriers found in archive", file=sys.stderr)
        return 1

    meta = archive_dir / "metadata"
    meta.mkdir(parents=True, exist_ok=True)
    sig_path = meta / "ecology_signatures.jsonl"
    sum_path = meta / "ecology_atlas_summary.json"

    with open(sig_path, "w") as f:
        for w in windows:
            f.write(json.dumps(w, sort_keys=True) + "\n")

    if manifest:
        summary["ingestion_manifest"] = {
            "timeline_fingerprint": manifest.get("timeline_fingerprint"),
            "processed_ticks": manifest.get("processed_ticks"),
            "corridor_rate": manifest.get("corridor_rate"),
        }

    with open(sum_path, "w") as f:
        json.dump(summary, f, indent=2)

    print("=" * 60)
    print("ECOLOGY SIGNATURE ATLAS — Phase 1")
    print("=" * 60)
    print(f"  Archive        : {archive_dir}")
    print(f"  Barriers       : {len(windows)}")
    print(f"  ts range       : {summary['ts_first']} → {summary['ts_last']}")
    print(f"  corridor μ±σ   : {summary['corridor_rate_mean']} / {summary['corridor_rate_stdev']}")
    print(f"  Signatures     : {sig_path}")
    print(f"  Summary        : {sum_path}")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
