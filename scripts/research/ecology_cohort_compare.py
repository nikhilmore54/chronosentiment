#!/usr/bin/env python3
"""
Cross-cohort ecological consistency — compare transition geometry across frozen runs.

Outcome-agnostic. Reads ecology_transition_graph.json from each cohort archive.

Usage:
  python3 scripts/ecology_cohort_compare.py \\
    --cohort 003: 900:replay_equiv 003:live
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from run_nse_cohort import resolve_archive_dir


def parse_cohort(spec: str) -> tuple[int, str]:
    """Format: batch_id or batch_id:run_label e.g. 900:replay_equiv"""
    if ":" in spec:
        bid, label = spec.split(":", 1)
        return int(bid), label
    return int(spec), ""


def load_graph(batch_id: int, run_label: str) -> tuple[str, dict]:
    archive = resolve_archive_dir(batch_id, False, run_label)
    path = archive / "metadata" / "ecology_transition_graph.json"
    if not path.exists():
        raise FileNotFoundError(path)
    name = f"batch_{batch_id:03d}" + (f"/{run_label}" if run_label else "")
    return name, json.loads(path.read_text())


def transition_rates(graph: dict) -> dict[str, float]:
    total = sum(t["count"] for t in graph.get("transitions", []))
    if not total:
        return {}
    return {
        f"{t['from']}->{t['to']}": round(t["count"] / total, 4)
        for t in graph["transitions"]
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Cross-cohort ecology consistency")
    parser.add_argument(
        "--cohort",
        nargs="+",
        default=["003:", "900:replay_equiv", "003:live"],
        help="Cohort specs: batch_id or batch_id:run_label",
    )
    parser.add_argument(
        "--out",
        default="state_archive/metadata/ecology_cohort_comparison.json",
    )
    args = parser.parse_args()

    cohorts: dict[str, dict] = {}
    for spec in args.cohort:
        bid, label = parse_cohort(spec)
        try:
            name, graph = load_graph(bid, label)
            cohorts[name] = graph
        except FileNotFoundError as e:
            print(f"⚠️  skip {spec}: {e}", file=sys.stderr)

    if len(cohorts) < 2:
        print("❌ Need at least 2 cohorts with ecology_transition_graph.json", file=sys.stderr)
        return 1

    comparison = {
        "cohorts": {},
        "cross_cohort": {},
        "built_at_utc": datetime.now(timezone.utc).isoformat(),
    }

    all_trans_keys: set[str] = set()
    for name, g in cohorts.items():
        rates = transition_rates(g)
        all_trans_keys.update(rates.keys())
        comparison["cohorts"][name] = {
            "barrier_count": g.get("barrier_count"),
            "phase_counts": g.get("phase_counts"),
            "phase_recurrence": g.get("phase_recurrence_density"),
            "mortality_proxy": g.get("mortality_proxy_exhaustion_rate"),
            "curvature": g.get("ecology_phase_curvature"),
            "compression_lineages_top": g.get("compression_lineages_top", [])[:5],
            "transition_rates_top": dict(
                sorted(rates.items(), key=lambda x: -x[1])[:12]
            ),
        }

    # Pairwise transition-rate divergence (L1 on shared keys)
    names = list(cohorts.keys())
    divergences = []
    for i in range(len(names)):
        for j in range(i + 1, len(names)):
            a, b = names[i], names[j]
            ra = transition_rates(cohorts[a])
            rb = transition_rates(cohorts[b])
            keys = set(ra) | set(rb)
            l1 = sum(abs(ra.get(k, 0) - rb.get(k, 0)) for k in keys)
            divergences.append({"a": a, "b": b, "transition_l1_divergence": round(l1, 4)})

    comparison["cross_cohort"]["transition_divergence"] = divergences

    # Shared compression lineages
    lineage_sets = {
        name: {x["path"] for x in g.get("compression_lineages_top", [])}
        for name, g in cohorts.items()
    }
    if lineage_sets:
        shared = set.intersection(*lineage_sets.values()) if lineage_sets else set()
        comparison["cross_cohort"]["shared_compression_lineages"] = sorted(shared)

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(comparison, f, indent=2)

    print("=" * 60)
    print("ECOLOGY COHORT COMPARISON")
    print("=" * 60)
    for name, c in comparison["cohorts"].items():
        print(f"\n  {name}:")
        print(f"    barriers={c['barrier_count']} mortality={c['mortality_proxy']}")
        print(f"    phases={c['phase_counts']}")
        if c.get("curvature"):
            print(f"    curvature velocity_μ={c['curvature'].get('velocity_mean')}")
    print("\n  Transition divergence (lower = more similar geometry):")
    for d in divergences:
        print(f"    {d['a']} vs {d['b']}: L1={d['transition_l1_divergence']}")
    print(f"\n  Report: {out_path}")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
