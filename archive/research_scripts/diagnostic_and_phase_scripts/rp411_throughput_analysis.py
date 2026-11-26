#!/usr/bin/env python3
"""
RP-411 Throughput Analysis
===========================
Reads JSONL generation records from a campaign run and produces a per-phase
timing breakdown report.

Records consumed:
  record_type = "generation"  — one per generation per run

Outputs (written to --output-dir):
  throughput_summary.csv        — per-instance throughput metrics
  phase_breakdown.csv           — per-instance mean phase timing
  RP411_THROUGHPUT_REPORT.md    — human-readable findings document

Usage:
  python scripts/rp411_throughput_analysis.py \\
      --telemetry-dir /tmp/rp410_telemetry_v3 \\
      --output-dir docs/roadef/rp411_data
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


def load_generation_records(telemetry_dir: Path) -> list[dict]:
    records = []
    gen_files = sorted(telemetry_dir.glob("rp410_generations_*.jsonl"))
    if not gen_files:
        print(f"ERROR: no rp410_generations_*.jsonl files in {telemetry_dir}", file=sys.stderr)
        sys.exit(1)
    for path in gen_files:
        for rec in load_jsonl(path):
            if rec.get("record_type") == "generation":
                records.append(rec)
    return records


# ---------------------------------------------------------------------------
# Analysis
# ---------------------------------------------------------------------------

PHASE_FIELDS = [
    "eval_time_ms",
    "crossover_time_ms",
    "mutation_time_ms",
    "repair_time_ms",
    "selection_time_ms",
    "telemetry_time_ms",
    "other_time_ms",
    "total_gen_time_ms",
]


def analyse_throughput(records: list[dict]) -> tuple[list[dict], list[dict]]:
    """
    Returns:
      throughput_rows: per-instance summary (gens/min, total time, etc.)
      phase_rows: per-instance mean phase timing and % breakdown
    """
    # Group by (instance, seed)
    by_run: dict[tuple, list[dict]] = defaultdict(list)
    for rec in records:
        key = (rec["instance"], rec["seed"])
        by_run[key].append(rec)

    throughput_rows = []
    phase_rows = []

    for (instance, seed), run_recs in sorted(by_run.items()):
        n = len(run_recs)
        if n == 0:
            continue

        # Throughput: use total_gen_time_ms to estimate gens/min
        total_gen_ms = sum(r.get("total_gen_time_ms", 0.0) for r in run_recs)
        total_gen_s = total_gen_ms / 1000.0
        gens_per_min = (n / total_gen_s * 60.0) if total_gen_s > 0 else 0.0

        # Mean phase times
        means = {}
        for field in PHASE_FIELDS:
            vals = [r.get(field, 0.0) for r in run_recs]
            means[field] = sum(vals) / len(vals) if vals else 0.0

        mean_total = means["total_gen_time_ms"]

        # Phase percentages (of total_gen_time_ms)
        pcts = {}
        for field in PHASE_FIELDS:
            if field == "total_gen_time_ms":
                continue
            pcts[field] = (means[field] / mean_total * 100.0) if mean_total > 0 else 0.0

        throughput_rows.append({
            "instance": instance,
            "seed": seed,
            "generations_run": n,
            "total_gen_time_ms": round(total_gen_ms, 1),
            "mean_gen_time_ms": round(means["total_gen_time_ms"], 3),
            "gens_per_min": round(gens_per_min, 2),
        })

        phase_row = {
            "instance": instance,
            "seed": seed,
            "generations_run": n,
            "mean_total_ms": round(mean_total, 3),
        }
        for field in PHASE_FIELDS:
            if field == "total_gen_time_ms":
                continue
            phase_row[f"mean_{field}"] = round(means[field], 3)
            phase_row[f"pct_{field}"] = round(pcts[field], 1)
        phase_rows.append(phase_row)

    return throughput_rows, phase_rows


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def write_csv(rows: list[dict], path: Path):
    if not rows:
        path.write_text("")
        return
    with open(path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def write_report(
    throughput_rows: list[dict],
    phase_rows: list[dict],
    output_dir: Path,
    telemetry_dir: Path,
) -> Path:
    path = output_dir / "RP411_THROUGHPUT_REPORT.md"

    total_runs = len(throughput_rows)
    if not throughput_rows:
        path.write_text("# RP-411 Throughput Report\n\nNo generation records found.\n")
        return path

    gpm_vals = [r["gens_per_min"] for r in throughput_rows]
    min_gpm = min(gpm_vals)
    max_gpm = max(gpm_vals)
    mean_gpm = sum(gpm_vals) / len(gpm_vals)
    ratio = max_gpm / min_gpm if min_gpm > 0 else float("inf")

    # Slowest and fastest instances
    slowest = min(throughput_rows, key=lambda r: r["gens_per_min"])
    fastest = max(throughput_rows, key=lambda r: r["gens_per_min"])

    # Mean phase breakdown across all runs
    phase_fields_no_total = [f for f in PHASE_FIELDS if f != "total_gen_time_ms"]
    global_means = {}
    global_pcts = {}
    for field in phase_fields_no_total:
        mean_key = f"mean_{field}"
        pct_key = f"pct_{field}"
        vals = [r.get(mean_key, 0.0) for r in phase_rows]
        pct_vals = [r.get(pct_key, 0.0) for r in phase_rows]
        global_means[field] = sum(vals) / len(vals) if vals else 0.0
        global_pcts[field] = sum(pct_vals) / len(pct_vals) if pct_vals else 0.0

    lines = []
    lines.append("# RP-411 Execution Throughput Analysis")
    lines.append("")
    lines.append(f"**Telemetry source:** `{telemetry_dir}`")
    lines.append(f"**Runs analysed:** {total_runs}")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"Generation throughput varies by **{ratio:.0f}×** across instances "
        f"(fastest: {max_gpm:.1f} gens/min on `{fastest['instance']}`, "
        f"slowest: {min_gpm:.2f} gens/min on `{slowest['instance']}`). "
        f"Mean throughput: {mean_gpm:.1f} gens/min."
    )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 1. Per-Instance Throughput")
    lines.append("")
    lines.append("| Instance | Gens Run | Mean Gen (ms) | Gens/min |")
    lines.append("|----------|----------:|--------------:|---------:|")
    for r in sorted(throughput_rows, key=lambda x: x["gens_per_min"], reverse=True):
        lines.append(
            f"| {r['instance']} | {r['generations_run']} "
            f"| {r['mean_gen_time_ms']:.3f} | {r['gens_per_min']:.2f} |"
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 2. Phase Breakdown (mean across all runs)")
    lines.append("")
    lines.append(
        "Each generation is decomposed into phases. "
        "Percentages are of `total_gen_time_ms`."
    )
    lines.append("")
    lines.append("| Phase | Mean (ms) | % of total |")
    lines.append("|-------|----------:|-----------:|")
    phase_labels = {
        "eval_time_ms": "Evaluation",
        "crossover_time_ms": "Crossover",
        "mutation_time_ms": "Mutation",
        "repair_time_ms": "Repair",
        "selection_time_ms": "Selection",
        "telemetry_time_ms": "Telemetry",
        "other_time_ms": "Other",
    }
    for field in phase_fields_no_total:
        label = phase_labels.get(field, field)
        lines.append(
            f"| {label} | {global_means[field]:.3f} | {global_pcts[field]:.1f}% |"
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 3. Per-Instance Phase Breakdown")
    lines.append("")
    lines.append("| Instance | Eval% | Crossover% | Mutation% | Repair% | Selection% | Telemetry% | Other% |")
    lines.append("|----------|------:|-----------:|----------:|--------:|-----------:|-----------:|-------:|")
    for r in sorted(phase_rows, key=lambda x: x["instance"]):
        lines.append(
            f"| {r['instance']} "
            f"| {r.get('pct_eval_time_ms', 0):.1f}% "
            f"| {r.get('pct_crossover_time_ms', 0):.1f}% "
            f"| {r.get('pct_mutation_time_ms', 0):.1f}% "
            f"| {r.get('pct_repair_time_ms', 0):.1f}% "
            f"| {r.get('pct_selection_time_ms', 0):.1f}% "
            f"| {r.get('pct_telemetry_time_ms', 0):.1f}% "
            f"| {r.get('pct_other_time_ms', 0):.1f}% |"
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 4. Interpretation")
    lines.append("")

    # Identify dominant phase
    dominant_field = max(phase_fields_no_total, key=lambda f: global_pcts[f])
    dominant_label = phase_labels.get(dominant_field, dominant_field)
    dominant_pct = global_pcts[dominant_field]

    lines.append(
        f"The dominant phase is **{dominant_label}** at {dominant_pct:.1f}% of generation time. "
    )
    if dominant_field == "eval_time_ms":
        lines.append(
            "Evaluation dominance is expected for network-routing problems where each "
            "fitness call requires a full flow computation. "
            "Optimisation effort should focus on incremental evaluation or caching "
            "before addressing variation operators."
        )
    elif dominant_field in ("crossover_time_ms", "mutation_time_ms"):
        lines.append(
            "Variation operator dominance is unexpected for this problem class. "
            "Investigate whether genome representation or operator implementation "
            "is causing excessive allocation."
        )
    else:
        lines.append(
            "Investigate the dominant phase before optimising other subsystems."
        )
    lines.append("")
    lines.append(
        f"The {ratio:.0f}× throughput range across instances indicates that "
        "instance size (number of demands, links, time slots) is the primary "
        "throughput driver. Larger instances require more evaluation time per generation."
    )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 5. Timing Accuracy Note")
    lines.append("")
    lines.append(
        "The selection/crossover/mutation phase is timed as a single block and "
        "attributed proportionally by operator count (10% selection, 90% split by "
        "crossover/mutation ratio). This is an approximation. "
        "The `repair_time_ms` field is currently zero — repair is not yet a separate "
        "phase in this harness. The `telemetry_time_ms` field captures only the "
        "`emit_generation` call; `emit_candidate` overhead is included in `other_time_ms`."
    )

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="RP-411 Throughput Analysis")
    parser.add_argument("--telemetry-dir", required=True, type=Path,
                        help="Directory containing rp410_*.jsonl telemetry files")
    parser.add_argument("--output-dir", required=True, type=Path,
                        help="Directory to write CSV and Markdown report")
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading generation records from {args.telemetry_dir} ...")
    records = load_generation_records(args.telemetry_dir)
    print(f"  {len(records)} generation records")

    if not records:
        print("ERROR: no generation records found.", file=sys.stderr)
        sys.exit(1)

    # Check whether timing fields are present
    sample = records[0]
    if "eval_time_ms" not in sample:
        print(
            "ERROR: generation records do not contain RP-411 timing fields. "
            "Ensure the campaign was run with RP-411 instrumentation.",
            file=sys.stderr,
        )
        sys.exit(1)

    print("Analysing ...")
    throughput_rows, phase_rows = analyse_throughput(records)

    write_csv(throughput_rows, args.output_dir / "throughput_summary.csv")
    print(f"  CSV written: {args.output_dir / 'throughput_summary.csv'}")

    write_csv(phase_rows, args.output_dir / "phase_breakdown.csv")
    print(f"  CSV written: {args.output_dir / 'phase_breakdown.csv'}")

    report_path = write_report(throughput_rows, phase_rows, args.output_dir, args.telemetry_dir)
    print(f"  Report written: {report_path}")

    # Print summary
    if throughput_rows:
        gpm_vals = [r["gens_per_min"] for r in throughput_rows]
        min_gpm = min(gpm_vals)
        max_gpm = max(gpm_vals)
        spread = f"{max_gpm/min_gpm:.0f}×" if min_gpm > 0 else "∞"
        print(f"\nThroughput range: {min_gpm:.2f}–{max_gpm:.1f} gens/min ({spread} spread)")


if __name__ == "__main__":
    main()