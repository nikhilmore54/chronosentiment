#!/usr/bin/env python3
"""
RP-410A — Evolutionary Search Dynamics Characterisation
========================================================
Loads JSONL telemetry produced by the RP-410 instrumented campaign and
produces a structured evidence report covering:

  A. Zone distribution (all accepted moves, all instances)
  B. Zone evolution per generation (per instance)
  C. Collapsed-basin vs Shape-competition comparison
  D. Operator fingerprints (crossover / mutation / crossover+mutation)
  E. SDI and MLU evolution
  F. Diversity evolution (unique_fitness_count)

Usage:
    python3 scripts/rp410a_analysis.py \
        --telemetry-dir /tmp/rp410_telemetry \
        --output-dir    docs/roadef/rp410a_data

The script writes:
    rp410a_zone_distribution.csv
    rp410a_zone_evolution.csv
    rp410a_basin_comparison.csv
    rp410a_operator_fingerprints.csv
    rp410a_generation_summary.csv
    RP410A_SEARCH_DYNAMICS_REPORT.md
"""

import argparse
import csv
import json
import os
import sys
from collections import defaultdict
from pathlib import Path

# ---------------------------------------------------------------------------
# Instance classification (from RP-406C)
# ---------------------------------------------------------------------------
COLLAPSED_BASIN_INSTANCES = {
    "setA-02", "setA-04", "setA-05", "setA-06", "setA-07", "setA-08",
}

def instance_class(name: str) -> str:
    base = name.split("_seed")[0]
    return "collapsed_basin" if base in COLLAPSED_BASIN_INSTANCES else "shape_competition"

# ---------------------------------------------------------------------------
# JSONL loading
# ---------------------------------------------------------------------------
def load_jsonl(path: Path) -> list[dict]:
    records = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    records.append(json.loads(line))
                except json.JSONDecodeError as e:
                    print(f"  WARN: skipping malformed line in {path.name}: {e}", file=sys.stderr)
    return records

def load_all_telemetry(telemetry_dir: Path) -> tuple[list[dict], list[dict]]:
    moves = []
    generations = []
    for f in sorted(telemetry_dir.glob("rp410_moves_*.jsonl")):
        moves.extend(load_jsonl(f))
    for f in sorted(telemetry_dir.glob("rp410_generations_*.jsonl")):
        generations.extend(load_jsonl(f))
    print(f"Loaded {len(moves)} move records and {len(generations)} generation records.")
    return moves, generations

# ---------------------------------------------------------------------------
# Analysis A: Zone distribution
# ---------------------------------------------------------------------------
def analyse_zone_distribution(moves: list[dict]) -> dict:
    """Count accepted moves by zone class, overall and per instance."""
    overall = defaultdict(int)
    per_instance = defaultdict(lambda: defaultdict(int))
    per_class = defaultdict(lambda: defaultdict(int))

    for m in moves:
        mc = m.get("move_class", "neutral")
        inst = m.get("instance", "unknown")
        ic = instance_class(inst)
        overall[mc] += 1
        per_instance[inst][mc] += 1
        per_class[ic][mc] += 1

    return {
        "overall": dict(overall),
        "per_instance": {k: dict(v) for k, v in per_instance.items()},
        "per_class": {k: dict(v) for k, v in per_class.items()},
    }

# ---------------------------------------------------------------------------
# Analysis B: Zone evolution per generation
# ---------------------------------------------------------------------------
def analyse_zone_evolution(generations: list[dict]) -> list[dict]:
    """Return per-generation zone histogram rows."""
    rows = []
    for g in generations:
        rows.append({
            "instance": g.get("instance", ""),
            "seed": g.get("seed", 0),
            "generation": g.get("generation", 0),
            "best_obj": g.get("best_obj", float("inf")),
            "best_mlu": g.get("best_mlu", float("inf")),
            "best_sdi": g.get("best_sdi", 0.0),
            "unique_fitness_count": g.get("unique_fitness_count", 0),
            "valid_count": g.get("valid_count", 0),
            "stagnation": g.get("stagnation", 0),
            "moves_peak": g.get("moves_peak", 0),
            "moves_shoulder": g.get("moves_shoulder", 0),
            "moves_transition": g.get("moves_transition", 0),
            "moves_tail": g.get("moves_tail", 0),
            "moves_mixed": g.get("moves_mixed", 0),
            "moves_neutral": g.get("moves_neutral", 0),
            "crossover_count": g.get("crossover_count", 0),
            "mutation_count": g.get("mutation_count", 0),
            "instance_class": instance_class(g.get("instance", "")),
        })
    return rows

# ---------------------------------------------------------------------------
# Analysis C: Collapsed-basin vs Shape-competition comparison
# ---------------------------------------------------------------------------
def analyse_basin_comparison(moves: list[dict], generations: list[dict]) -> dict:
    """Aggregate move distributions and generation stats by instance class."""
    move_dist = defaultdict(lambda: defaultdict(int))
    gen_stats = defaultdict(lambda: defaultdict(list))

    for m in moves:
        ic = instance_class(m.get("instance", ""))
        mc = m.get("move_class", "neutral")
        move_dist[ic][mc] += 1

    for g in generations:
        ic = instance_class(g.get("instance", ""))
        gen_stats[ic]["best_sdi"].append(g.get("best_sdi", 0.0))
        gen_stats[ic]["best_mlu"].append(g.get("best_mlu", float("inf")))
        gen_stats[ic]["unique_fitness_count"].append(g.get("unique_fitness_count", 0))
        gen_stats[ic]["stagnation"].append(g.get("stagnation", 0))

    def safe_mean(lst):
        # Filter out None values (JSON null from non-finite floats like inf)
        clean = [x for x in lst if x is not None]
        return sum(clean) / len(clean) if clean else 0.0

    summary = {}
    for ic in set(list(move_dist.keys()) + list(gen_stats.keys())):
        total_moves = sum(move_dist[ic].values())
        summary[ic] = {
            "total_moves": total_moves,
            "move_dist": dict(move_dist[ic]),
            "move_pct": {
                k: round(100.0 * v / total_moves, 2) if total_moves > 0 else 0.0
                for k, v in move_dist[ic].items()
            },
            "avg_sdi": round(safe_mean(gen_stats[ic]["best_sdi"]), 6),
            "avg_mlu": round(safe_mean(gen_stats[ic]["best_mlu"]), 6),
            "avg_diversity": round(safe_mean(gen_stats[ic]["unique_fitness_count"]), 2),
            "avg_stagnation": round(safe_mean(gen_stats[ic]["stagnation"]), 2),
        }
    return summary

# ---------------------------------------------------------------------------
# Analysis D: Operator fingerprints
# ---------------------------------------------------------------------------
def analyse_operator_fingerprints(moves: list[dict]) -> dict:
    """Zone distribution per operator."""
    op_dist = defaultdict(lambda: defaultdict(int))
    for m in moves:
        op = m.get("operator", "unknown")
        mc = m.get("move_class", "neutral")
        op_dist[op][mc] += 1

    result = {}
    for op, dist in op_dist.items():
        total = sum(dist.values())
        result[op] = {
            "total": total,
            "dist": dict(dist),
            "pct": {
                k: round(100.0 * v / total, 2) if total > 0 else 0.0
                for k, v in dist.items()
            },
        }
    return result

# ---------------------------------------------------------------------------
# CSV writers
# ---------------------------------------------------------------------------
ZONE_CLASSES = ["peak", "shoulder", "transition", "tail", "mixed", "neutral"]

def write_zone_distribution_csv(dist: dict, output_dir: Path):
    path = output_dir / "rp410a_zone_distribution.csv"
    with open(path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["scope", "instance_or_class"] + ZONE_CLASSES + ["total"])
        # Overall
        overall = dist["overall"]
        total = sum(overall.values())
        w.writerow(["overall", "ALL"] + [overall.get(z, 0) for z in ZONE_CLASSES] + [total])
        # Per instance class
        for ic, d in dist["per_class"].items():
            total = sum(d.values())
            w.writerow(["class", ic] + [d.get(z, 0) for z in ZONE_CLASSES] + [total])
        # Per instance
        for inst, d in sorted(dist["per_instance"].items()):
            total = sum(d.values())
            w.writerow(["instance", inst] + [d.get(z, 0) for z in ZONE_CLASSES] + [total])
    print(f"  Written: {path}")

def write_zone_evolution_csv(rows: list[dict], output_dir: Path):
    path = output_dir / "rp410a_zone_evolution.csv"
    if not rows:
        print("  WARN: no generation records to write.")
        return
    fieldnames = list(rows[0].keys())
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"  Written: {path}")

def write_basin_comparison_csv(summary: dict, output_dir: Path):
    path = output_dir / "rp410a_basin_comparison.csv"
    with open(path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["instance_class", "total_moves"] + ZONE_CLASSES +
                   [f"{z}_pct" for z in ZONE_CLASSES] +
                   ["avg_sdi", "avg_mlu", "avg_diversity", "avg_stagnation"])
        for ic, s in sorted(summary.items()):
            w.writerow(
                [ic, s["total_moves"]] +
                [s["move_dist"].get(z, 0) for z in ZONE_CLASSES] +
                [s["move_pct"].get(z, 0.0) for z in ZONE_CLASSES] +
                [s["avg_sdi"], s["avg_mlu"], s["avg_diversity"], s["avg_stagnation"]]
            )
    print(f"  Written: {path}")

def write_operator_fingerprints_csv(fp: dict, output_dir: Path):
    path = output_dir / "rp410a_operator_fingerprints.csv"
    with open(path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["operator", "total"] + ZONE_CLASSES + [f"{z}_pct" for z in ZONE_CLASSES])
        for op, s in sorted(fp.items()):
            w.writerow(
                [op, s["total"]] +
                [s["dist"].get(z, 0) for z in ZONE_CLASSES] +
                [s["pct"].get(z, 0.0) for z in ZONE_CLASSES]
            )
    print(f"  Written: {path}")

# ---------------------------------------------------------------------------
# Markdown report generator
# ---------------------------------------------------------------------------
def _fmt(val, spec=".4f"):
    """Format a numeric value, returning 'null' for None and 'inf' for infinity."""
    if val is None:
        return "null"
    try:
        if val != val or abs(val) == float("inf"):  # NaN or inf
            return "inf"
        return format(val, spec)
    except (TypeError, ValueError):
        return str(val)


def write_markdown_report(
    dist: dict,
    basin: dict,
    fp: dict,
    gen_rows: list[dict],
    output_dir: Path,
):
    path = output_dir / "RP410A_SEARCH_DYNAMICS_REPORT.md"

    overall = dist["overall"]
    total_moves = sum(overall.values())

    def pct(n):
        return f"{100.0 * n / total_moves:.1f}%" if total_moves > 0 else "N/A"

    lines = [
        "# RP-410A — Evolutionary Search Dynamics Characterisation",
        "",
        "## Status",
        "",
        "**Generated automatically by `scripts/rp410a_analysis.py`.**",
        "",
        "---",
        "",
        "## A. Zone Distribution — All Accepted Moves",
        "",
        "| Zone | Count | % |",
        "|------|------:|--:|",
    ]
    for z in ZONE_CLASSES:
        n = overall.get(z, 0)
        lines.append(f"| {z.capitalize()} | {n} | {pct(n)} |")
    lines += [
        f"| **Total** | **{total_moves}** | 100% |",
        "",
        "---",
        "",
        "## B. Collapsed Basin vs Shape Competition",
        "",
        "| Metric | Collapsed Basin | Shape Competition |",
        "|--------|----------------:|------------------:|",
    ]
    cb = basin.get("collapsed_basin", {})
    sc = basin.get("shape_competition", {})
    for z in ZONE_CLASSES:
        cb_pct = cb.get("move_pct", {}).get(z, 0.0)
        sc_pct = sc.get("move_pct", {}).get(z, 0.0)
        lines.append(f"| {z.capitalize()} % | {cb_pct:.1f}% | {sc_pct:.1f}% |")
    lines += [
        f"| Avg SDI | {cb.get('avg_sdi', 'N/A')} | {sc.get('avg_sdi', 'N/A')} |",
        f"| Avg MLU | {cb.get('avg_mlu', 'N/A')} | {sc.get('avg_mlu', 'N/A')} |",
        f"| Avg Diversity | {cb.get('avg_diversity', 'N/A')} | {sc.get('avg_diversity', 'N/A')} |",
        f"| Avg Stagnation | {cb.get('avg_stagnation', 'N/A')} | {sc.get('avg_stagnation', 'N/A')} |",
        "",
        "---",
        "",
        "## C. Operator Fingerprints",
        "",
        "| Operator | Total | Peak % | Shoulder % | Transition % | Tail % | Mixed % | Neutral % |",
        "|----------|------:|-------:|-----------:|-------------:|-------:|--------:|----------:|",
    ]
    for op, s in sorted(fp.items()):
        p = s["pct"]
        lines.append(
            f"| {op} | {s['total']} "
            f"| {p.get('peak', 0):.1f}% "
            f"| {p.get('shoulder', 0):.1f}% "
            f"| {p.get('transition', 0):.1f}% "
            f"| {p.get('tail', 0):.1f}% "
            f"| {p.get('mixed', 0):.1f}% "
            f"| {p.get('neutral', 0):.1f}% |"
        )
    lines += [
        "",
        "---",
        "",
        "## D. Generation Summary",
        "",
        f"Total generation records: {len(gen_rows)}",
        "",
    ]

    # Per-instance summary table
    inst_gens = defaultdict(list)
    for r in gen_rows:
        inst_gens[r["instance"]].append(r)

    lines += [
        "| Instance | Class | Gens | Final SDI | Final MLU | Total Moves | Peak | Shoulder | Transition | Tail |",
        "|----------|-------|-----:|----------:|----------:|------------:|-----:|---------:|-----------:|-----:|",
    ]
    for inst, rows in sorted(inst_gens.items()):
        last = rows[-1]
        total_m = sum(r["moves_peak"] + r["moves_shoulder"] + r["moves_transition"] +
                      r["moves_tail"] + r["moves_mixed"] + r["moves_neutral"] for r in rows)
        tp = sum(r["moves_peak"] for r in rows)
        ts = sum(r["moves_shoulder"] for r in rows)
        tt = sum(r["moves_transition"] for r in rows)
        ta = sum(r["moves_tail"] for r in rows)
        ic = instance_class(inst)
        lines.append(
            f"| {inst} | {ic} | {len(rows)} "
            f"| {_fmt(last['best_sdi'])} | {_fmt(last['best_mlu'])} "
            f"| {total_m} | {tp} | {ts} | {tt} | {ta} |"
        )

    lines += [
        "",
        "---",
        "",
        "## E. Hypothesis Assessment",
        "",
        "### H1 — Transition/Tail dominance",
        "",
        "**Prediction:** ≥ 80% of accepted moves improve Transition or Tail zones.",
        "",
        f"**Observed:** Transition = {overall.get('transition', 0)} ({pct(overall.get('transition', 0))}), "
        f"Tail = {overall.get('tail', 0)} ({pct(overall.get('tail', 0))})",
        "",
        "**Status:** TBD — compare against 80% threshold.",
        "",
        "### H2 — Shoulder improvements rare after generation 50",
        "",
        "**Prediction:** Shoulder move frequency drops sharply after generation 50.",
        "",
        "**Status:** TBD — inspect `rp410a_zone_evolution.csv` for shoulder trend.",
        "",
        "### H3 — Collapsed-basin instances never generate Peak improvements",
        "",
        f"**Observed:** Collapsed Basin Peak % = {cb.get('move_pct', {}).get('peak', 0.0):.1f}%",
        "",
        "**Status:** TBD — compare against Shape Competition Peak %.",
        "",
        "### H4 — Different operators produce different zone fingerprints",
        "",
        "**Status:** TBD — inspect operator fingerprint table above.",
        "",
        "---",
        "",
        "## F. Data Files",
        "",
        "| File | Contents |",
        "|------|----------|",
        "| `rp410a_zone_distribution.csv` | Move counts by zone, overall and per instance |",
        "| `rp410a_zone_evolution.csv` | Per-generation zone histogram for every run |",
        "| `rp410a_basin_comparison.csv` | Collapsed Basin vs Shape Competition aggregates |",
        "| `rp410a_operator_fingerprints.csv` | Zone distribution per operator |",
        "",
        "---",
        "",
        "*End of RP-410A report.*",
    ]

    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  Written: {path}")

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="RP-410A analysis")
    parser.add_argument("--telemetry-dir", default="/tmp/rp410_telemetry",
                        help="Directory containing rp410_*.jsonl files")
    parser.add_argument("--output-dir", default="docs/roadef/rp410a_data",
                        help="Directory to write CSV and Markdown output")
    args = parser.parse_args()

    telemetry_dir = Path(args.telemetry_dir)
    output_dir = Path(args.output_dir)

    if not telemetry_dir.exists():
        print(f"ERROR: telemetry directory not found: {telemetry_dir}", file=sys.stderr)
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading telemetry from: {telemetry_dir}")
    moves, generations = load_all_telemetry(telemetry_dir)

    if not moves and not generations:
        print("ERROR: no telemetry records found. Has the campaign been run with RP410_TELEMETRY_DIR set?",
              file=sys.stderr)
        sys.exit(1)

    print("\nRunning analyses...")

    dist = analyse_zone_distribution(moves)
    gen_rows = analyse_zone_evolution(generations)
    basin = analyse_basin_comparison(moves, generations)
    fp = analyse_operator_fingerprints(moves)

    print("\nWriting output files...")
    write_zone_distribution_csv(dist, output_dir)
    write_zone_evolution_csv(gen_rows, output_dir)
    write_basin_comparison_csv(basin, output_dir)
    write_operator_fingerprints_csv(fp, output_dir)
    write_markdown_report(dist, basin, fp, gen_rows, output_dir)

    print("\nSummary:")
    overall = dist["overall"]
    total = sum(overall.values())
    print(f"  Total accepted moves: {total}")
    for z in ZONE_CLASSES:
        n = overall.get(z, 0)
        pct = 100.0 * n / total if total > 0 else 0.0
        print(f"  {z:12s}: {n:6d}  ({pct:.1f}%)")

    print(f"\nOutput written to: {output_dir}")

if __name__ == "__main__":
    main()