#!/usr/bin/env python3
"""
RP-412 Construction Funnel Analysis
====================================
Reads JSONL telemetry from a campaign run and produces a construction-phase
diagnostic report.

Records consumed:
  record_type = "construction"  — one per run (instance × seed)
  record_type = "generation"    — used for gen-0 valid_count cross-check

Outputs (written to --output-dir):
  construction_summary.csv      — per-instance construction metrics
  RP412_CONSTRUCTION_REPORT.md  — human-readable findings document

Usage:
  python scripts/rp412_construction_analysis.py \\
      --telemetry-dir /tmp/rp410_telemetry_v3 \\
      --output-dir docs/roadef/rp412_data
"""

import argparse
import csv
import json
import os
import sys
from collections import defaultdict
from pathlib import Path


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def load_jsonl(path: Path) -> list[dict]:
    records = []
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError as e:
                print(f"  WARN: {path.name}:{lineno}: {e}", file=sys.stderr)
    return records


def load_telemetry(telemetry_dir: Path) -> tuple[list[dict], list[dict]]:
    """Return (construction_records, generation_records) from all JSONL files."""
    construction = []
    generations = []
    gen_files = sorted(telemetry_dir.glob("rp410_generations_*.jsonl"))
    if not gen_files:
        print(f"ERROR: no rp410_generations_*.jsonl files in {telemetry_dir}", file=sys.stderr)
        sys.exit(1)
    for path in gen_files:
        for rec in load_jsonl(path):
            rt = rec.get("record_type", "")
            if rt == "construction":
                construction.append(rec)
            elif rt == "generation":
                generations.append(rec)
    return construction, generations


# ---------------------------------------------------------------------------
# Analysis
# ---------------------------------------------------------------------------

def analyse(construction: list[dict], generations: list[dict]) -> list[dict]:
    """
    Merge construction records with gen-0 generation records for cross-check.
    Returns a list of per-run summary dicts.
    """
    # Index gen-0 records by (instance, seed)
    gen0 = {}
    for rec in generations:
        if rec.get("generation", -1) == 0:
            key = (rec["instance"], rec["seed"])
            gen0[key] = rec

    rows = []
    for c in construction:
        instance = c["instance"]
        seed = c["seed"]
        pop = c["population_size"]
        valid = c["valid_count"]
        invalid = c["invalid_count"]
        ifr = c["initial_feasibility_rate"]
        any_feasible = c["any_feasible"]

        # Cross-check with gen-0 generation record
        key = (instance, seed)
        g0 = gen0.get(key)
        gen0_valid_check = g0["generation0_valid_count"] if g0 else None
        gen0_match = (gen0_valid_check == valid) if gen0_valid_check is not None else None

        # Count total generations run for this instance (proxy for throughput)
        total_gens = sum(
            1 for rec in generations
            if rec["instance"] == instance and rec["seed"] == seed
        )

        rows.append({
            "instance": instance,
            "seed": seed,
            "population_size": pop,
            "valid_count": valid,
            "invalid_count": invalid,
            "initial_feasibility_rate": round(ifr, 4),
            "any_feasible": any_feasible,
            "gen0_valid_check": gen0_valid_check,
            "gen0_match": gen0_match,
            "total_generations_run": total_gens,
            "capacity_violation_count": c.get("capacity_violation_count", 0),
            "budget_violation_count": c.get("budget_violation_count", 0),
            "repair_attempts": c.get("repair_attempts", 0),
            "repair_successes": c.get("repair_successes", 0),
        })

    # Sort by instance name
    rows.sort(key=lambda r: r["instance"])
    return rows


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def write_csv(rows: list[dict], output_dir: Path) -> Path:
    path = output_dir / "construction_summary.csv"
    if not rows:
        print("WARN: no construction records found; CSV will be empty.", file=sys.stderr)
        path.write_text("")
        return path
    fieldnames = list(rows[0].keys())
    with open(path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    return path


def write_report(rows: list[dict], output_dir: Path, telemetry_dir: Path) -> Path:
    path = output_dir / "RP412_CONSTRUCTION_REPORT.md"

    total_runs = len(rows)
    feasible_runs = sum(1 for r in rows if r["any_feasible"])
    infeasible_runs = total_runs - feasible_runs
    zero_valid_runs = sum(1 for r in rows if r["valid_count"] == 0)
    full_valid_runs = sum(1 for r in rows if r["valid_count"] == r["population_size"])

    # IFR statistics
    ifrs = [r["initial_feasibility_rate"] for r in rows]
    mean_ifr = sum(ifrs) / len(ifrs) if ifrs else 0.0
    min_ifr = min(ifrs) if ifrs else 0.0
    max_ifr = max(ifrs) if ifrs else 0.0

    # Instances with zero valid
    zero_valid_instances = sorted(
        set(r["instance"] for r in rows if r["valid_count"] == 0)
    )
    feasible_instances = sorted(
        set(r["instance"] for r in rows if r["any_feasible"])
    )

    # Gen-0 cross-check
    mismatches = [r for r in rows if r["gen0_match"] is False]

    lines = []
    lines.append("# RP-412 Construction Funnel Analysis")
    lines.append("")
    lines.append(f"**Telemetry source:** `{telemetry_dir}`")
    lines.append(f"**Runs analysed:** {total_runs}")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"Of {total_runs} runs across {len(set(r['instance'] for r in rows))} instances, "
        f"**{feasible_runs} ({100*feasible_runs//total_runs if total_runs else 0}%) produced at least one valid individual** "
        f"at construction time. "
        f"{zero_valid_runs} runs produced zero valid individuals — "
        f"the search never began for those instances."
    )
    lines.append("")
    lines.append(
        f"Mean Initial Feasibility Rate (IFR): **{mean_ifr:.1%}** "
        f"(range {min_ifr:.1%}–{max_ifr:.1%})."
    )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 1. Construction Feasibility by Instance")
    lines.append("")
    lines.append("| Instance | Valid | Invalid | IFR | Any Feasible | Gens Run |")
    lines.append("|----------|------:|--------:|----:|:------------:|---------:|")
    for r in rows:
        feasible_mark = "✓" if r["any_feasible"] else "✗"
        lines.append(
            f"| {r['instance']} | {r['valid_count']} | {r['invalid_count']} "
            f"| {r['initial_feasibility_rate']:.1%} | {feasible_mark} | {r['total_generations_run']} |"
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 2. Type I Failure Instances (zero valid at gen 0)")
    lines.append("")
    if zero_valid_instances:
        lines.append(
            "The following instances produced **no valid individuals** during construction. "
            "The evolutionary search never began for these instances. "
            "This is a Type I (construction) failure, not an evolutionary failure."
        )
        lines.append("")
        for inst in zero_valid_instances:
            lines.append(f"- `{inst}`")
    else:
        lines.append(
            "No instances produced zero valid individuals. "
            "All runs entered the evolutionary phase."
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 3. Feasible Instances")
    lines.append("")
    if feasible_instances:
        lines.append(
            f"{len(feasible_instances)} instances produced at least one valid individual:"
        )
        lines.append("")
        for inst in feasible_instances:
            r = next(x for x in rows if x["instance"] == inst)
            lines.append(
                f"- `{inst}`: IFR = {r['initial_feasibility_rate']:.1%} "
                f"({r['valid_count']}/{r['population_size']} valid)"
            )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 4. Gen-0 Cross-Check")
    lines.append("")
    lines.append(
        "The `generation0_valid_count` field in `GenerationRecord` (gen 0) should match "
        "`valid_count` in `ConstructionRecord`. Mismatches indicate a telemetry wiring bug."
    )
    lines.append("")
    if mismatches:
        lines.append(f"**{len(mismatches)} mismatches detected:**")
        lines.append("")
        for r in mismatches:
            lines.append(
                f"- `{r['instance']}` seed={r['seed']}: "
                f"construction.valid_count={r['valid_count']}, "
                f"gen0.generation0_valid_count={r['gen0_valid_check']}"
            )
    else:
        lines.append("All cross-checks passed — no mismatches.")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 5. Reserved Fields (RP-412 Phase 2)")
    lines.append("")
    lines.append(
        "The following fields are reserved for deeper evaluator instrumentation "
        "and are currently zero for all runs:"
    )
    lines.append("")
    lines.append("- `capacity_violation_count` — requires per-individual violation breakdown")
    lines.append("- `budget_violation_count` — requires per-individual violation breakdown")
    lines.append("- `repair_attempts` — repair is not yet a separate phase in this harness")
    lines.append("- `repair_successes` — repair is not yet a separate phase in this harness")
    lines.append("")
    lines.append(
        "When the evaluator exposes per-constraint violation counts, these fields will "
        "distinguish capacity violations from segment-budget violations, enabling "
        "targeted constructor improvements."
    )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 6. Implications for Research Programme")
    lines.append("")
    lines.append(
        "**If `valid_count = 0` for an instance:** the bottleneck is entirely in the "
        "Construction subsystem. Changing selection (RP-408) or variation operators (RP-409) "
        "cannot help. The constructor must be fixed first."
    )
    lines.append("")
    lines.append(
        "**If `valid_count > 0` but IFR is low:** the search begins but with a sparse "
        "feasible seed. Evolutionary pressure may be insufficient to maintain feasibility. "
        "This is the boundary between Type I and Type II failure."
    )
    lines.append("")
    lines.append(
        "**If IFR is high:** construction is not the bottleneck. "
        "Proceed to RP-411 (throughput) and RP-410B (candidate pipeline) to identify "
        "where search efficiency is lost."
    )

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="RP-412 Construction Funnel Analysis")
    parser.add_argument("--telemetry-dir", required=True, type=Path,
                        help="Directory containing rp410_*.jsonl telemetry files")
    parser.add_argument("--output-dir", required=True, type=Path,
                        help="Directory to write CSV and Markdown report")
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading telemetry from {args.telemetry_dir} ...")
    construction, generations = load_telemetry(args.telemetry_dir)
    print(f"  {len(construction)} construction records")
    print(f"  {len(generations)} generation records")

    if not construction:
        print("ERROR: no construction records found. "
              "Ensure the campaign was run with RP-412 instrumentation.", file=sys.stderr)
        sys.exit(1)

    print("Analysing ...")
    rows = analyse(construction, generations)

    csv_path = write_csv(rows, args.output_dir)
    print(f"  CSV written: {csv_path}")

    report_path = write_report(rows, args.output_dir, args.telemetry_dir)
    print(f"  Report written: {report_path}")

    # Print summary to stdout
    total = len(rows)
    feasible = sum(1 for r in rows if r["any_feasible"])
    zero = sum(1 for r in rows if r["valid_count"] == 0)
    print(f"\nSummary: {total} runs | {feasible} feasible | {zero} zero-valid (Type I failure)")


if __name__ == "__main__":
    main()