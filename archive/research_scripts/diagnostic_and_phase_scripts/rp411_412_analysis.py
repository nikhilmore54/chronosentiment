#!/usr/bin/env python3
"""
RP-411 / RP-412 Baseline Analysis
==================================
Reads generation records and construction records from a telemetry directory
and produces:

  RP-411 (Execution Throughput):
    - Per-phase timing breakdown: selection_ms, crossover_ms, mutation_ms,
      eval_ms, telemetry_ms, other_ms, total_gen_ms
    - Generations-per-second and evaluations-per-second
    - Time budget utilisation (fraction of runtime in each phase)
    - Stagnation profile (generations until termination)

  RP-412 (Construction Diagnostics):
    - Initial Feasibility Rate (IFR) per instance
    - capacity_violation_count distribution
    - any_feasible flag summary

Usage:
    python3 scripts/rp411_412_analysis.py \
        --telemetry-dir /tmp/rp411_baseline \
        --output-dir docs/roadef/rp411_412_data/
"""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path
from statistics import mean, median, stdev


# ---------------------------------------------------------------------------
# Loaders
# ---------------------------------------------------------------------------

def load_generation_records(telemetry_dir: Path) -> list[dict]:
    recs = []
    for f in sorted(telemetry_dir.glob("rp410_generations_*.jsonl")):
        with open(f) as fh:
            for line in fh:
                line = line.strip()
                if line:
                    try:
                        r = json.loads(line)
                        if r.get("record_type") == "generation":
                            recs.append(r)
                    except json.JSONDecodeError:
                        pass
    return recs


def load_construction_records(telemetry_dir: Path) -> list[dict]:
    recs = []
    for f in sorted(telemetry_dir.glob("rp410_generations_*.jsonl")):
        with open(f) as fh:
            for line in fh:
                line = line.strip()
                if line:
                    try:
                        r = json.loads(line)
                        if r.get("record_type") == "construction":
                            recs.append(r)
                    except json.JSONDecodeError:
                        pass
    return recs


# ---------------------------------------------------------------------------
# RP-411: Execution Throughput
# ---------------------------------------------------------------------------

def compute_timing_breakdown(gen_recs: list[dict]) -> dict:
    """Aggregate per-phase timing across all generations and instances."""
    phases = ["eval_time_ms", "crossover_time_ms", "mutation_time_ms",
              "selection_time_ms", "telemetry_time_ms", "other_time_ms",
              "total_gen_time_ms"]
    totals: dict[str, float] = {p: 0.0 for p in phases}
    count = 0
    for r in gen_recs:
        for p in phases:
            totals[p] += r.get(p, 0.0)
        count += 1

    if count == 0:
        return {"total_generations": 0}

    # Fraction of total_gen_time_ms spent in each phase
    total_ms = totals["total_gen_time_ms"]
    fractions: dict[str, float] = {}
    if total_ms > 0:
        for p in phases:
            fractions[f"{p}_frac"] = totals[p] / total_ms
    else:
        # Fall back to eval as proxy for total
        total_ms = totals["eval_time_ms"] + totals["crossover_time_ms"] + \
                   totals["mutation_time_ms"] + totals["selection_time_ms"] + \
                   totals["other_time_ms"]
        if total_ms > 0:
            for p in phases:
                fractions[f"{p}_frac"] = totals[p] / total_ms

    return {
        "total_generations": count,
        **{f"total_{p}": totals[p] for p in phases},
        **fractions,
    }


def compute_per_instance_timing(gen_recs: list[dict]) -> list[dict]:
    """Per-instance timing summary: total gens, total eval_ms, gens/s."""
    by_instance: dict[str, list[dict]] = defaultdict(list)
    for r in gen_recs:
        by_instance[r.get("instance", "unknown")].append(r)

    rows = []
    for inst, recs in sorted(by_instance.items()):
        total_gens = len(recs)
        total_eval_ms = sum(r.get("eval_time_ms", 0.0) for r in recs)
        total_xo_ms = sum(r.get("crossover_time_ms", 0.0) for r in recs)
        total_mut_ms = sum(r.get("mutation_time_ms", 0.0) for r in recs)
        total_sel_ms = sum(r.get("selection_time_ms", 0.0) for r in recs)
        total_gen_ms = sum(r.get("total_gen_time_ms", 0.0) for r in recs)
        # Stagnation at last generation
        last_stagnation = recs[-1].get("stagnation", 0) if recs else 0
        # Population size (constant)
        pop_size = recs[0].get("population_size", 0) if recs else 0
        # Total evaluations = total_gens * pop_size (approx)
        total_evals = total_gens * pop_size
        # Gens per second (using total_gen_ms as denominator)
        gens_per_s = (total_gens / (total_gen_ms / 1000.0)) if total_gen_ms > 0 else 0.0
        evals_per_s = (total_evals / (total_gen_ms / 1000.0)) if total_gen_ms > 0 else 0.0

        rows.append({
            "instance": inst,
            "total_gens": total_gens,
            "pop_size": pop_size,
            "total_evals": total_evals,
            "total_eval_ms": total_eval_ms,
            "total_xo_ms": total_xo_ms,
            "total_mut_ms": total_mut_ms,
            "total_sel_ms": total_sel_ms,
            "total_gen_ms": total_gen_ms,
            "gens_per_s": gens_per_s,
            "evals_per_s": evals_per_s,
            "final_stagnation": last_stagnation,
        })
    return rows


def compute_stagnation_profile(gen_recs: list[dict]) -> dict:
    """Distribution of stagnation values at termination (last gen per instance)."""
    by_instance: dict[str, list[dict]] = defaultdict(list)
    for r in gen_recs:
        by_instance[r.get("instance", "unknown")].append(r)

    final_stagnations = []
    final_gens = []
    for inst, recs in by_instance.items():
        if recs:
            final_stagnations.append(recs[-1].get("stagnation", 0))
            final_gens.append(recs[-1].get("generation", 0))

    if not final_stagnations:
        return {}

    return {
        "instance_count": len(final_stagnations),
        "mean_final_stagnation": mean(final_stagnations),
        "median_final_stagnation": median(final_stagnations),
        "max_final_stagnation": max(final_stagnations),
        "mean_final_gen": mean(final_gens),
        "median_final_gen": median(final_gens),
        "max_final_gen": max(final_gens),
        # Fraction that hit stagnation limit (stagnation >= 20 = NoImprovement)
        "stagnation_terminated_count": sum(1 for s in final_stagnations if s >= 20),
        "generation_terminated_count": sum(1 for g in final_gens if g >= 199),
    }


# ---------------------------------------------------------------------------
# RP-412: Construction Diagnostics
# ---------------------------------------------------------------------------

def compute_construction_summary(construction_recs: list[dict]) -> dict:
    """Aggregate construction diagnostics across all instances."""
    if not construction_recs:
        return {}

    ifrs = [r.get("initial_feasibility_rate", 0.0) for r in construction_recs]
    cap_viols = [r.get("capacity_violation_count", 0) for r in construction_recs]
    any_feasible_count = sum(1 for r in construction_recs if r.get("any_feasible", False))
    pop_sizes = [r.get("population_size", 0) for r in construction_recs]

    return {
        "instance_count": len(construction_recs),
        "mean_ifr": mean(ifrs),
        "median_ifr": median(ifrs),
        "min_ifr": min(ifrs),
        "max_ifr": max(ifrs),
        "stdev_ifr": stdev(ifrs) if len(ifrs) > 1 else 0.0,
        "any_feasible_count": any_feasible_count,
        "all_feasible_count": sum(1 for r in construction_recs
                                  if r.get("initial_feasibility_rate", 0.0) >= 1.0),
        "mean_capacity_violation_count": mean(cap_viols),
        "total_capacity_violations": sum(cap_viols),
        "mean_population_size": mean(pop_sizes),
    }


def compute_per_instance_construction(construction_recs: list[dict]) -> list[dict]:
    rows = []
    for r in sorted(construction_recs, key=lambda x: x.get("instance", "")):
        rows.append({
            "instance": r.get("instance", ""),
            "population_size": r.get("population_size", 0),
            "valid_count": r.get("valid_count", 0),
            "invalid_count": r.get("invalid_count", 0),
            "initial_feasibility_rate": r.get("initial_feasibility_rate", 0.0),
            "any_feasible": r.get("any_feasible", False),
            "capacity_violation_count": r.get("capacity_violation_count", 0),
            "budget_violation_count": r.get("budget_violation_count", 0),
        })
    return rows


# ---------------------------------------------------------------------------
# CSV writers
# ---------------------------------------------------------------------------

def write_csv(rows: list[dict], path: Path) -> None:
    if not rows:
        path.write_text("")
        return
    headers = list(rows[0].keys())
    lines = [",".join(headers)]
    for row in rows:
        lines.append(",".join(str(row.get(h, "")) for h in headers))
    path.write_text("\n".join(lines) + "\n")


def write_timing_breakdown_csv(breakdown: dict, path: Path) -> None:
    rows = [{"metric": k, "value": v} for k, v in breakdown.items()]
    write_csv(rows, path)


def write_construction_summary_csv(summary: dict, path: Path) -> None:
    rows = [{"metric": k, "value": v} for k, v in summary.items()]
    write_csv(rows, path)


# ---------------------------------------------------------------------------
# Report generator
# ---------------------------------------------------------------------------

def generate_report(
    timing_breakdown: dict,
    per_instance_timing: list[dict],
    stagnation_profile: dict,
    construction_summary: dict,
    per_instance_construction: list[dict],
    telemetry_dir: str,
) -> str:
    lines = []
    lines.append("# RP-411 / RP-412 Baseline Analysis Report")
    lines.append("")
    lines.append(f"**Telemetry directory:** `{telemetry_dir}`")
    lines.append(f"**Total generation records:** {timing_breakdown.get('total_generations', 0):,}")
    lines.append(f"**Instances:** {stagnation_profile.get('instance_count', 0)}")
    lines.append("")

    # --- RP-411: Timing breakdown ---
    lines.append("## 1. Aggregate Phase Timing (RP-411)")
    lines.append("")
    lines.append("| Phase | Total ms | Fraction |")
    lines.append("|-------|----------|----------|")
    phases = [
        ("eval_time_ms", "Evaluation"),
        ("crossover_time_ms", "Crossover"),
        ("mutation_time_ms", "Mutation"),
        ("selection_time_ms", "Selection"),
        ("telemetry_time_ms", "Telemetry"),
        ("other_time_ms", "Other"),
        ("total_gen_time_ms", "Total"),
    ]
    for key, label in phases:
        total_key = f"total_{key}"
        frac_key = f"{key}_frac"
        total_ms = timing_breakdown.get(total_key, 0.0)
        frac = timing_breakdown.get(frac_key, 0.0)
        lines.append(f"| {label} | {total_ms:,.1f} | {100*frac:.2f}% |")
    lines.append("")

    # --- RP-411: Per-instance timing ---
    lines.append("## 2. Per-Instance Timing (RP-411)")
    lines.append("")
    lines.append("| Instance | Gens | Pop | Total Evals | Eval ms | Gens/s | Evals/s | Final Stagnation |")
    lines.append("|----------|------|-----|-------------|---------|--------|---------|-----------------|")
    for row in per_instance_timing:
        lines.append(
            f"| {row['instance']} | {row['total_gens']} | {row['pop_size']} | "
            f"{row['total_evals']:,} | {row['total_eval_ms']:,.1f} | "
            f"{row['gens_per_s']:.2f} | {row['evals_per_s']:.1f} | {row['final_stagnation']} |"
        )
    lines.append("")

    # --- RP-411: Stagnation profile ---
    lines.append("## 3. Stagnation Profile (RP-411)")
    lines.append("")
    if stagnation_profile:
        lines.append(f"- Instances: {stagnation_profile.get('instance_count', 0)}")
        lines.append(f"- Mean final stagnation: {stagnation_profile.get('mean_final_stagnation', 0):.1f}")
        lines.append(f"- Median final stagnation: {stagnation_profile.get('median_final_stagnation', 0):.1f}")
        lines.append(f"- Max final stagnation: {stagnation_profile.get('max_final_stagnation', 0)}")
        lines.append(f"- Mean final generation: {stagnation_profile.get('mean_final_gen', 0):.1f}")
        lines.append(f"- Median final generation: {stagnation_profile.get('median_final_gen', 0):.1f}")
        lines.append(f"- Max final generation: {stagnation_profile.get('max_final_gen', 0)}")
        lines.append(f"- Terminated by NoImprovement (stagnation ≥ 20): {stagnation_profile.get('stagnation_terminated_count', 0)}")
        lines.append(f"- Terminated by GenerationLimit (gen ≥ 199): {stagnation_profile.get('generation_terminated_count', 0)}")
    lines.append("")

    # --- RP-412: Construction summary ---
    lines.append("## 4. Construction Diagnostics Summary (RP-412)")
    lines.append("")
    if construction_summary:
        lines.append(f"- Instances: {construction_summary.get('instance_count', 0)}")
        lines.append(f"- Mean IFR: {100*construction_summary.get('mean_ifr', 0):.2f}%")
        lines.append(f"- Median IFR: {100*construction_summary.get('median_ifr', 0):.2f}%")
        lines.append(f"- Min IFR: {100*construction_summary.get('min_ifr', 0):.2f}%")
        lines.append(f"- Max IFR: {100*construction_summary.get('max_ifr', 0):.2f}%")
        lines.append(f"- StdDev IFR: {100*construction_summary.get('stdev_ifr', 0):.2f}%")
        lines.append(f"- Instances with any_feasible=true: {construction_summary.get('any_feasible_count', 0)}")
        lines.append(f"- Instances with IFR=100%: {construction_summary.get('all_feasible_count', 0)}")
        lines.append(f"- Mean capacity_violation_count: {construction_summary.get('mean_capacity_violation_count', 0):.1f}")
        lines.append(f"- Total capacity violations: {construction_summary.get('total_capacity_violations', 0)}")
    lines.append("")

    # --- RP-412: Per-instance construction ---
    lines.append("## 5. Per-Instance Construction Diagnostics (RP-412)")
    lines.append("")
    lines.append("| Instance | Pop | Valid | Invalid | IFR | Any Feasible | Cap Violations |")
    lines.append("|----------|-----|-------|---------|-----|--------------|----------------|")
    for row in per_instance_construction:
        lines.append(
            f"| {row['instance']} | {row['population_size']} | {row['valid_count']} | "
            f"{row['invalid_count']} | {100*row['initial_feasibility_rate']:.1f}% | "
            f"{row['any_feasible']} | {row['capacity_violation_count']} |"
        )
    lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="RP-411/412 baseline analysis")
    parser.add_argument("--telemetry-dir", type=Path, required=True,
                        help="Directory containing rp410_generations_*.jsonl files")
    parser.add_argument("--output-dir", type=Path, required=True,
                        help="Directory to write CSV and report outputs")
    args = parser.parse_args()

    telemetry_dir: Path = args.telemetry_dir
    output_dir: Path = args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading generation records from {telemetry_dir} ...")
    gen_recs = load_generation_records(telemetry_dir)
    if not gen_recs:
        print("ERROR: No generation records found.", file=sys.stderr)
        sys.exit(1)
    print(f"  Loaded {len(gen_recs):,} generation records.")

    print("Loading construction records ...")
    construction_recs = load_construction_records(telemetry_dir)
    print(f"  Loaded {len(construction_recs)} construction records.")

    print("Computing RP-411 timing breakdown ...")
    timing_breakdown = compute_timing_breakdown(gen_recs)

    print("Computing per-instance timing ...")
    per_instance_timing = compute_per_instance_timing(gen_recs)

    print("Computing stagnation profile ...")
    stagnation_profile = compute_stagnation_profile(gen_recs)

    print("Computing RP-412 construction summary ...")
    construction_summary = compute_construction_summary(construction_recs)

    print("Computing per-instance construction diagnostics ...")
    per_instance_construction = compute_per_instance_construction(construction_recs)

    print(f"Writing CSVs to {output_dir} ...")
    write_timing_breakdown_csv(timing_breakdown, output_dir / "timing_breakdown.csv")
    write_csv(per_instance_timing, output_dir / "per_instance_timing.csv")
    write_csv([{"metric": k, "value": v} for k, v in stagnation_profile.items()],
              output_dir / "stagnation_profile.csv")
    write_construction_summary_csv(construction_summary, output_dir / "construction_summary.csv")
    write_csv(per_instance_construction, output_dir / "per_instance_construction.csv")

    print("Generating report ...")
    report = generate_report(
        timing_breakdown=timing_breakdown,
        per_instance_timing=per_instance_timing,
        stagnation_profile=stagnation_profile,
        construction_summary=construction_summary,
        per_instance_construction=per_instance_construction,
        telemetry_dir=str(telemetry_dir),
    )
    report_path = output_dir / "RP411_412_BASELINE_REPORT.md"
    report_path.write_text(report)
    print(f"Report written to {report_path}")
    print("Done.")


if __name__ == "__main__":
    main()