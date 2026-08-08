#!/usr/bin/env python3
"""
RP-410B Candidate Pipeline Funnel Analysis
==========================================
Reads JSONL move/candidate records from a campaign run and produces a
candidate pipeline funnel report.

Records consumed:
  record_type = "candidate"  — one per generated child per generation

Outputs (written to --output-dir):
  candidate_funnel.csv          — per-instance funnel metrics
  operator_funnel.csv           — per-operator funnel metrics
  RP410B_CANDIDATE_REPORT.md    — human-readable findings document

Usage:
  python scripts/rp410b_candidate_analysis.py \\
      --telemetry-dir /tmp/rp410_telemetry_v3 \\
      --output-dir docs/roadef/rp410b_data
"""

import argparse
import csv
import json
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


def load_candidate_records(telemetry_dir: Path) -> list[dict]:
    records = []
    move_files = sorted(telemetry_dir.glob("rp410_moves_*.jsonl"))
    if not move_files:
        print(f"ERROR: no rp410_moves_*.jsonl files in {telemetry_dir}", file=sys.stderr)
        sys.exit(1)
    for path in move_files:
        for rec in load_jsonl(path):
            if rec.get("record_type") == "candidate":
                records.append(rec)
    return records


def pct(num: int, denom: int) -> str:
    if denom == 0:
        return "—"
    return f"{100.0 * num / denom:.2f}%"


def pct_f(num: int, denom: int) -> float:
    if denom == 0:
        return 0.0
    return 100.0 * num / denom


# ---------------------------------------------------------------------------
# Analysis
# ---------------------------------------------------------------------------

MOVE_CLASSES = ["peak", "shoulder", "transition", "tail", "mixed", "neutral"]


def funnel_stats(candidates: list[dict]) -> dict:
    """Compute funnel statistics for a list of candidate records."""
    total = len(candidates)
    valid = sum(1 for c in candidates if c.get("valid", False))
    accepted = sum(1 for c in candidates if c.get("accepted", False))

    # Zone-class breakdown (among valid candidates)
    zone_counts = defaultdict(int)
    zone_valid = defaultdict(int)
    zone_accepted = defaultdict(int)
    for c in candidates:
        mc = c.get("move_class", "neutral")
        zone_counts[mc] += 1
        if c.get("valid", False):
            zone_valid[mc] += 1
        if c.get("accepted", False):
            zone_accepted[mc] += 1

    peak_generated = zone_counts.get("peak", 0)
    peak_valid = zone_valid.get("peak", 0)
    peak_accepted = zone_accepted.get("peak", 0)

    shoulder_generated = zone_counts.get("shoulder", 0)
    shoulder_valid = zone_valid.get("shoulder", 0)
    shoulder_accepted = zone_accepted.get("shoulder", 0)

    return {
        "total": total,
        "valid": valid,
        "accepted": accepted,
        "validity_rate": pct_f(valid, total),
        "acceptance_rate": pct_f(accepted, total),
        "peak_generated": peak_generated,
        "peak_valid": peak_valid,
        "peak_accepted": peak_accepted,
        "shoulder_generated": shoulder_generated,
        "shoulder_valid": shoulder_valid,
        "shoulder_accepted": shoulder_accepted,
    }


def per_instance_funnel(candidates: list[dict]) -> list[dict]:
    by_instance = defaultdict(list)
    for c in candidates:
        by_instance[c["instance"]].append(c)

    rows = []
    for instance, cands in sorted(by_instance.items()):
        stats = funnel_stats(cands)
        rows.append({"instance": instance, **stats})
    return rows


def per_operator_funnel(candidates: list[dict]) -> list[dict]:
    by_op = defaultdict(list)
    for c in candidates:
        by_op[c.get("operator", "unknown")].append(c)

    rows = []
    for op, cands in sorted(by_op.items()):
        stats = funnel_stats(cands)
        rows.append({"operator": op, **stats})
    return rows


def global_funnel(candidates: list[dict]) -> dict:
    return funnel_stats(candidates)


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
    candidates: list[dict],
    instance_rows: list[dict],
    operator_rows: list[dict],
    output_dir: Path,
    telemetry_dir: Path,
) -> Path:
    path = output_dir / "RP410B_CANDIDATE_REPORT.md"

    gf = global_funnel(candidates)
    total = gf["total"]

    lines = []
    lines.append("# RP-410B Candidate Pipeline Funnel Analysis")
    lines.append("")
    lines.append(f"**Telemetry source:** `{telemetry_dir}`")
    lines.append(f"**Total candidates observed:** {total:,}")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        "The candidate pipeline funnel tracks every generated child from "
        "variation through to global-best acceptance. "
        "This is the first dataset that separates generation failure from "
        "repair failure from selection failure."
    )
    lines.append("")
    lines.append("**Global funnel:**")
    lines.append("")
    lines.append(f"| Stage | Count | Conversion |")
    lines.append(f"|-------|------:|-----------:|")
    lines.append(f"| Generated | {total:,} | 100% |")
    lines.append(f"| Valid | {gf['valid']:,} | {pct(gf['valid'], total)} |")
    lines.append(f"| Accepted (improves global best) | {gf['accepted']:,} | {pct(gf['accepted'], total)} |")
    lines.append(f"| Peak-improving generated | {gf['peak_generated']:,} | {pct(gf['peak_generated'], total)} |")
    lines.append(f"| Peak-improving valid | {gf['peak_valid']:,} | {pct(gf['peak_valid'], total)} |")
    lines.append(f"| Peak-improving accepted | {gf['peak_accepted']:,} | {pct(gf['peak_accepted'], total)} |")
    lines.append(f"| Shoulder-improving generated | {gf['shoulder_generated']:,} | {pct(gf['shoulder_generated'], total)} |")
    lines.append(f"| Shoulder-improving valid | {gf['shoulder_valid']:,} | {pct(gf['shoulder_valid'], total)} |")
    lines.append(f"| Shoulder-improving accepted | {gf['shoulder_accepted']:,} | {pct(gf['shoulder_accepted'], total)} |")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 1. Per-Operator Funnel")
    lines.append("")
    lines.append(
        "For each variation operator, the funnel shows how many candidates "
        "were generated, how many were valid, and how many improved the global best."
    )
    lines.append("")
    lines.append("| Operator | Generated | Valid | Valid% | Accepted | Accept% | Peak Gen | Peak Accept |")
    lines.append("|----------|----------:|------:|-------:|---------:|--------:|---------:|------------:|")
    for r in operator_rows:
        lines.append(
            f"| {r['operator']} "
            f"| {r['total']:,} "
            f"| {r['valid']:,} "
            f"| {pct(r['valid'], r['total'])} "
            f"| {r['accepted']:,} "
            f"| {pct(r['accepted'], r['total'])} "
            f"| {r['peak_generated']:,} "
            f"| {r['peak_accepted']:,} |"
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 2. Per-Instance Funnel")
    lines.append("")
    lines.append("| Instance | Generated | Valid% | Accepted% | Peak Gen | Peak Accept |")
    lines.append("|----------|----------:|-------:|----------:|---------:|------------:|")
    for r in instance_rows:
        lines.append(
            f"| {r['instance']} "
            f"| {r['total']:,} "
            f"| {pct(r['valid'], r['total'])} "
            f"| {pct(r['accepted'], r['total'])} "
            f"| {r['peak_generated']:,} "
            f"| {r['peak_accepted']:,} |"
        )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 3. Peak Zone Analysis")
    lines.append("")
    lines.append(
        "The Peak zone (rank-1 arc saturation improvement) is the primary "
        "objective of the search. This section isolates where Peak candidates "
        "are lost in the pipeline."
    )
    lines.append("")
    if gf["peak_generated"] == 0:
        lines.append(
            "**No Peak-improving candidates were generated in this campaign.** "
            "This means the variation operators are not producing candidates that "
            "improve the rank-1 arc saturation. "
            "This is a generation failure, not a selection failure. "
            "Changing the selection objective (RP-408) cannot help if no Peak "
            "candidates are being generated."
        )
    else:
        peak_survival = pct(gf["peak_valid"], gf["peak_generated"])
        peak_acceptance = pct(gf["peak_accepted"], gf["peak_valid"]) if gf["peak_valid"] > 0 else "—"
        lines.append(
            f"Peak candidates generated: {gf['peak_generated']:,}. "
            f"Of these, {gf['peak_valid']:,} ({peak_survival}) were valid after evaluation. "
            f"Of valid Peak candidates, {gf['peak_accepted']:,} ({peak_acceptance}) "
            "improved the global best."
        )
        lines.append("")
        if gf["peak_valid"] == 0:
            lines.append(
                "**All Peak candidates were invalid.** "
                "The repair/evaluation step is eliminating Peak improvements. "
                "This is a repair failure, not a selection failure."
            )
        elif gf["peak_accepted"] == 0:
            lines.append(
                "**Valid Peak candidates exist but none were accepted.** "
                "The selection objective is rejecting Peak improvements. "
                "This is a selection failure — RP-408 (lexicographic objective) "
                "is the correct next experiment."
            )
        else:
            lines.append(
                "Peak candidates are flowing through the full pipeline. "
                "The search is capable of Peak improvement."
            )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 4. Interpretation Framework")
    lines.append("")
    lines.append(
        "The funnel determines which subsystem is the bottleneck for Peak improvement:"
    )
    lines.append("")
    lines.append(
        "**Generation failure** (`peak_generated = 0`): "
        "Variation operators cannot produce Peak-improving candidates. "
        "Fix: change crossover/mutation to target rank-1 arcs."
    )
    lines.append("")
    lines.append(
        "**Repair failure** (`peak_valid = 0` but `peak_generated > 0`): "
        "Peak candidates are generated but invalidated by evaluation/repair. "
        "Fix: improve repair to preserve Peak improvements."
    )
    lines.append("")
    lines.append(
        "**Selection failure** (`peak_accepted = 0` but `peak_valid > 0`): "
        "Valid Peak candidates exist but the scalar fitness function rejects them. "
        "Fix: RP-408 lexicographic objective."
    )
    lines.append("")
    lines.append(
        "**No failure** (`peak_accepted > 0`): "
        "The search is capable of Peak improvement. "
        "Investigate why the global best is not improving further."
    )

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="RP-410B Candidate Pipeline Funnel Analysis")
    parser.add_argument("--telemetry-dir", required=True, type=Path,
                        help="Directory containing rp410_*.jsonl telemetry files")
    parser.add_argument("--output-dir", required=True, type=Path,
                        help="Directory to write CSV and Markdown report")
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading candidate records from {args.telemetry_dir} ...")
    candidates = load_candidate_records(args.telemetry_dir)
    print(f"  {len(candidates):,} candidate records")

    if not candidates:
        print(
            "ERROR: no candidate records found. "
            "Ensure the campaign was run with RP-410B instrumentation.",
            file=sys.stderr,
        )
        sys.exit(1)

    print("Analysing ...")
    instance_rows = per_instance_funnel(candidates)
    operator_rows = per_operator_funnel(candidates)

    write_csv(instance_rows, args.output_dir / "candidate_funnel.csv")
    print(f"  CSV written: {args.output_dir / 'candidate_funnel.csv'}")

    write_csv(operator_rows, args.output_dir / "operator_funnel.csv")
    print(f"  CSV written: {args.output_dir / 'operator_funnel.csv'}")

    report_path = write_report(
        candidates, instance_rows, operator_rows, args.output_dir, args.telemetry_dir
    )
    print(f"  Report written: {report_path}")

    # Print summary
    gf = global_funnel(candidates)
    total = gf["total"]
    print(f"\nGlobal funnel: {total:,} generated → "
          f"{gf['valid']:,} valid ({pct(gf['valid'], total)}) → "
          f"{gf['accepted']:,} accepted ({pct(gf['accepted'], total)})")
    print(f"Peak: {gf['peak_generated']:,} generated → "
          f"{gf['peak_valid']:,} valid → "
          f"{gf['peak_accepted']:,} accepted")


if __name__ == "__main__":
    main()