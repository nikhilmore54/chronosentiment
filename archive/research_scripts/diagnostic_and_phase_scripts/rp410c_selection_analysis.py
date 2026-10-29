#!/usr/bin/env python3
"""
RP-410C Selection Analysis
==========================
Phase 1: Survival funnel, PE decomposition, stage loss rates.
Phase 2: Tournament PE by zone, Population PE by zone, Elite PE by zone,
         GlobalBest PE by zone, rejection reason frequency.

Usage:
    python3 scripts/rp410c_selection_analysis.py \
        --telemetry-dir /tmp/rp410c_v2_validate \
        --output-dir docs/roadef/rp410c_data_v2 \
        [--phase2]

File naming in telemetry dir:
    rp410_moves_<instance>_rand.jsonl       -- CandidateRecord (record_type="candidate")
    rp410_generations_<instance>_rand.jsonl -- GenerationRecord (record_type="generation"|"construction")
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


# ---------------------------------------------------------------------------
# Zone classification
# ---------------------------------------------------------------------------

def classify_zone(rec: dict) -> str:
    """Classify a candidate record into Peak / Shoulder / Transition / Tail."""
    deltas = rec.get("deltas") or {}
    if deltas.get("delta_rank1", 0.0) > 0.0:
        return "Peak"
    if deltas.get("delta_2_20", 0.0) > 0.0:
        return "Shoulder"
    if deltas.get("delta_21_100", 0.0) > 0.0:
        return "Transition"
    return "Tail"


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_candidate_records(telemetry_dir: Path) -> list[dict]:
    """Load all CandidateRecord entries from rp410_moves_*.jsonl files."""
    records: list[dict] = []
    files = sorted(telemetry_dir.glob("rp410_moves_*.jsonl"))
    if not files:
        # Fallback: try legacy naming rp410_candidates_*.jsonl
        files = sorted(telemetry_dir.glob("rp410_candidates_*.jsonl"))
    if not files:
        print(f"WARNING: No moves/candidates files found in {telemetry_dir}", file=sys.stderr)
        return records
    for path in files:
        with open(path) as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    rec = json.loads(line)
                    if rec.get("record_type") == "candidate":
                        records.append(rec)
                except json.JSONDecodeError:
                    pass
    return records


def load_generation_records(telemetry_dir: Path) -> list[dict]:
    """Load all GenerationRecord entries from rp410_generations_*.jsonl files."""
    records: list[dict] = []
    files = sorted(telemetry_dir.glob("rp410_generations_*.jsonl"))
    if not files:
        print(f"WARNING: No generations files found in {telemetry_dir}", file=sys.stderr)
        return records
    for path in files:
        with open(path) as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    rec = json.loads(line)
                    if rec.get("record_type") in ("generation", "construction"):
                        records.append(rec)
                except json.JSONDecodeError:
                    pass
    return records


# ---------------------------------------------------------------------------
# Phase 1 — Survival funnel
# ---------------------------------------------------------------------------

def compute_survival_funnel(records: list[dict]) -> dict:
    """
    Compute the five-stage survival funnel counts across all records.

    Stages (in order):
      1. Generated   — all candidate records
      2. TournamentWin — won_tournament == True
      3. Population  — decision_stage == "Population"
      4. Elite       — decision_stage == "Elite"
      5. GlobalBest  — decision_stage == "GlobalBest"
    """
    total = len(records)
    won_tourn = sum(1 for r in records if r.get("won_tournament"))
    entered_pop = sum(1 for r in records if r.get("decision_stage") == "Population")
    entered_elite = sum(1 for r in records if r.get("decision_stage") == "Elite")
    global_best = sum(1 for r in records if r.get("decision_stage") == "GlobalBest")

    def pct(n: int, d: int) -> float:
        return round(100.0 * n / d, 3) if d else 0.0

    return {
        "generated": total,
        "tournament_winners": won_tourn,
        "entered_population": entered_pop,
        "entered_elite": entered_elite,
        "became_global_best": global_best,
        "pct_won_tournament": pct(won_tourn, total),
        "pct_entered_pop": pct(entered_pop, won_tourn),
        "pct_entered_elite": pct(entered_elite, entered_pop),
        "pct_global_best": pct(global_best, entered_elite),
        "overall_osr": pct(global_best, total),
    }


# ---------------------------------------------------------------------------
# Phase 1 — Stage loss rates
# ---------------------------------------------------------------------------

def compute_stage_loss_rates(records: list[dict]) -> list[dict]:
    """
    Compute per-stage loss rates: how many candidates are lost at each stage.
    Returns a list of row dicts suitable for CSV output.
    """
    total = len(records)
    won_tourn = sum(1 for r in records if r.get("won_tournament"))
    entered_pop = sum(1 for r in records if r.get("decision_stage") == "Population")
    entered_elite = sum(1 for r in records if r.get("decision_stage") == "Elite")
    global_best = sum(1 for r in records if r.get("decision_stage") == "GlobalBest")

    def loss_rate(lost: int, pool: int) -> float:
        return round(100.0 * lost / pool, 3) if pool else 0.0

    rows = [
        {
            "stage": "Tournament",
            "pool": total,
            "survivors": won_tourn,
            "lost": total - won_tourn,
            "loss_rate_pct": loss_rate(total - won_tourn, total),
        },
        {
            "stage": "Promotion",
            "pool": won_tourn,
            "survivors": entered_pop,
            "lost": won_tourn - entered_pop,
            "loss_rate_pct": loss_rate(won_tourn - entered_pop, won_tourn),
        },
        {
            "stage": "Elite",
            "pool": entered_pop,
            "survivors": entered_elite,
            "lost": entered_pop - entered_elite,
            "loss_rate_pct": loss_rate(entered_pop - entered_elite, entered_pop),
        },
        {
            "stage": "GlobalBest",
            "pool": entered_elite,
            "survivors": global_best,
            "lost": entered_elite - global_best,
            "loss_rate_pct": loss_rate(entered_elite - global_best, entered_elite),
        },
    ]
    return rows


# ---------------------------------------------------------------------------
# Phase 1 — Operator PE decomposition
# ---------------------------------------------------------------------------

def compute_operator_pe(records: list[dict]) -> list[dict]:
    """
    Per-operator Promotion Efficiency: fraction of tournament winners that
    entered the population, broken down by operator.
    """
    op_won: Counter = Counter()
    op_pop: Counter = Counter()
    for r in records:
        op = r.get("operator", "unknown")
        if r.get("won_tournament"):
            op_won[op] += 1
            if r.get("decision_stage") == "Population":
                op_pop[op] += 1

    rows = []
    for op in sorted(op_won.keys()):
        won = op_won[op]
        pop = op_pop[op]
        rows.append({
            "operator": op,
            "tournament_wins": won,
            "entered_population": pop,
            "pe_pct": round(100.0 * pop / won, 3) if won else 0.0,
        })
    return rows


# ---------------------------------------------------------------------------
# Phase 1 — Stage frequency (decision_stage × reason)
# ---------------------------------------------------------------------------

def compute_stage_freq(records: list[dict]) -> list[dict]:
    """Count (decision_stage, reason) pairs."""
    counts: Counter = Counter()
    for r in records:
        stage = r.get("decision_stage") or "None"
        reason = r.get("reason") or "None"
        counts[(stage, reason)] += 1
    rows = [
        {"decision_stage": s, "reason": rs, "count": c}
        for (s, rs), c in sorted(counts.items(), key=lambda x: -x[1])
    ]
    return rows


# ---------------------------------------------------------------------------
# Phase 1 — Objective stats
# ---------------------------------------------------------------------------

def compute_obj_stats(records: list[dict]) -> dict:
    """Basic stats on objective values for valid candidates."""
    objs = [r["obj"] for r in records if r.get("valid") and r.get("obj") is not None]
    if not objs:
        return {"count": 0, "min": None, "max": None, "mean": None}
    return {
        "count": len(objs),
        "min": round(min(objs), 6),
        "max": round(max(objs), 6),
        "mean": round(sum(objs) / len(objs), 6),
    }


# ---------------------------------------------------------------------------
# Phase 2 — Tournament PE by zone
# ---------------------------------------------------------------------------

def compute_tournament_pe_by_zone(records: list[dict]) -> dict:
    """
    For each zone (Peak/Shoulder/Transition/Tail), compute:
      - total candidates entering tournament
      - tournament winners
      - tournament win rate (PE_tournament)

    Uses all records with decision_stage == "Tournament" (losers) plus
    tournament winners (won_tournament == True).
    """
    # All candidates that participated in a tournament
    # = those with decision_stage "Tournament" (losers) + tournament winners
    tourn_participants = [
        r for r in records
        if r.get("decision_stage") == "Tournament" or r.get("won_tournament")
    ]

    zone_total: Counter = Counter()
    zone_wins: Counter = Counter()

    for r in tourn_participants:
        z = classify_zone(r)
        zone_total[z] += 1
        if r.get("won_tournament"):
            zone_wins[z] += 1

    rows = {}
    for z in ("Peak", "Shoulder", "Transition", "Tail"):
        total = zone_total[z]
        wins = zone_wins[z]
        rows[z] = {
            "zone": z,
            "participants": total,
            "winners": wins,
            "win_rate_pct": round(100.0 * wins / total, 3) if total else 0.0,
        }
    return rows


# ---------------------------------------------------------------------------
# Phase 2 — Population PE by zone
# ---------------------------------------------------------------------------

def compute_population_pe_by_zone(records: list[dict]) -> dict:
    """
    For each zone, compute:
      - tournament winners (pool entering Promotion stage)
      - those that entered the population
      - Promotion PE = entered_pop / tournament_winners
    """
    zone_winners: Counter = Counter()
    zone_pop: Counter = Counter()

    for r in records:
        if not r.get("won_tournament"):
            continue
        z = classify_zone(r)
        zone_winners[z] += 1
        if r.get("decision_stage") == "Population":
            zone_pop[z] += 1

    rows = {}
    for z in ("Peak", "Shoulder", "Transition", "Tail"):
        winners = zone_winners[z]
        pop = zone_pop[z]
        rows[z] = {
            "zone": z,
            "tournament_winners": winners,
            "entered_population": pop,
            "promotion_pe_pct": round(100.0 * pop / winners, 3) if winners else 0.0,
        }
    return rows


# ---------------------------------------------------------------------------
# Phase 2 — Elite PE by zone
# ---------------------------------------------------------------------------

def compute_elite_pe_by_zone(records: list[dict]) -> dict:
    """
    For each zone, compute:
      - candidates that entered the population (pool entering Elite stage)
      - those that entered the elite
      - Elite PE = entered_elite / entered_population
    """
    zone_pop: Counter = Counter()
    zone_elite: Counter = Counter()

    for r in records:
        if r.get("decision_stage") == "Population":
            z = classify_zone(r)
            zone_pop[z] += 1
        elif r.get("decision_stage") == "Elite":
            z = classify_zone(r)
            zone_pop[z] += 1  # Elite records also passed through Population
            zone_elite[z] += 1

    rows = {}
    for z in ("Peak", "Shoulder", "Transition", "Tail"):
        pop = zone_pop[z]
        elite = zone_elite[z]
        rows[z] = {
            "zone": z,
            "entered_population": pop,
            "entered_elite": elite,
            "elite_pe_pct": round(100.0 * elite / pop, 3) if pop else 0.0,
        }
    return rows


# ---------------------------------------------------------------------------
# Phase 2 — GlobalBest PE by zone
# ---------------------------------------------------------------------------

def compute_globalbest_pe_by_zone(records: list[dict]) -> dict:
    """
    For each zone, compute:
      - candidates that entered the elite (pool entering GlobalBest stage)
      - those that became global best
      - GlobalBest PE = became_global_best / entered_elite
    """
    zone_elite: Counter = Counter()
    zone_gb: Counter = Counter()

    for r in records:
        if r.get("decision_stage") == "Elite":
            z = classify_zone(r)
            zone_elite[z] += 1
        elif r.get("decision_stage") == "GlobalBest":
            z = classify_zone(r)
            zone_elite[z] += 1  # GlobalBest also passed through Elite
            zone_gb[z] += 1

    rows = {}
    for z in ("Peak", "Shoulder", "Transition", "Tail"):
        elite = zone_elite[z]
        gb = zone_gb[z]
        rows[z] = {
            "zone": z,
            "entered_elite": elite,
            "became_global_best": gb,
            "globalbest_pe_pct": round(100.0 * gb / elite, 3) if elite else 0.0,
        }
    return rows


# ---------------------------------------------------------------------------
# Phase 2 — Rejection reason frequency by zone
# ---------------------------------------------------------------------------

def compute_reason_frequency(records: list[dict]) -> list[dict]:
    """
    Count (decision_stage, reason, zone) triples.
    """
    counts: Counter = Counter()
    for r in records:
        stage = r.get("decision_stage") or "None"
        reason = r.get("reason") or "None"
        z = classify_zone(r)
        counts[(stage, reason, z)] += 1

    rows = [
        {"decision_stage": s, "reason": rs, "zone": z, "count": c}
        for (s, rs, z), c in sorted(counts.items(), key=lambda x: -x[1])
    ]
    return rows


# ---------------------------------------------------------------------------
# Phase 2 — Population slot distribution by zone
# ---------------------------------------------------------------------------

def compute_population_slot_dist(records: list[dict]) -> list[dict]:
    """
    For candidates that entered the population, show slot distribution by zone.
    population_slot is an integer (0-indexed position in population array).
    """
    zone_slots: dict[str, list[int]] = defaultdict(list)
    for r in records:
        if r.get("decision_stage") in ("Population", "Elite", "GlobalBest"):
            slot = r.get("population_slot")
            if slot is not None:
                z = classify_zone(r)
                zone_slots[z].append(slot)

    rows = []
    for z in ("Peak", "Shoulder", "Transition", "Tail"):
        slots = zone_slots[z]
        if slots:
            rows.append({
                "zone": z,
                "count": len(slots),
                "min_slot": min(slots),
                "max_slot": max(slots),
                "mean_slot": round(sum(slots) / len(slots), 2),
            })
        else:
            rows.append({"zone": z, "count": 0, "min_slot": None, "max_slot": None, "mean_slot": None})
    return rows


# ---------------------------------------------------------------------------
# Phase 2 — OSR by zone (end-to-end)
# ---------------------------------------------------------------------------

def compute_osr_by_zone(records: list[dict]) -> list[dict]:
    """
    Overall Success Rate by zone: GlobalBest / Generated.
    Also compute per-stage PE for each zone in one table.
    """
    zone_generated: Counter = Counter()
    zone_tourn_win: Counter = Counter()
    zone_pop: Counter = Counter()
    zone_elite: Counter = Counter()
    zone_gb: Counter = Counter()

    for r in records:
        z = classify_zone(r)
        zone_generated[z] += 1
        if r.get("won_tournament"):
            zone_tourn_win[z] += 1
        stage = r.get("decision_stage")
        if stage == "Population":
            zone_pop[z] += 1
        elif stage == "Elite":
            zone_pop[z] += 1
            zone_elite[z] += 1
        elif stage == "GlobalBest":
            zone_pop[z] += 1
            zone_elite[z] += 1
            zone_gb[z] += 1

    def pct(n: int, d: int) -> float:
        return round(100.0 * n / d, 4) if d else 0.0

    rows = []
    for z in ("Peak", "Shoulder", "Transition", "Tail"):
        gen = zone_generated[z]
        tw = zone_tourn_win[z]
        pop = zone_pop[z]
        elite = zone_elite[z]
        gb = zone_gb[z]
        rows.append({
            "zone": z,
            "generated": gen,
            "tournament_winners": tw,
            "entered_population": pop,
            "entered_elite": elite,
            "became_global_best": gb,
            "pe_tournament_pct": pct(tw, gen),
            "pe_promotion_pct": pct(pop, tw),
            "pe_elite_pct": pct(elite, pop),
            "pe_globalbest_pct": pct(gb, elite),
            "osr_pct": pct(gb, gen),
        })
    return rows


# ---------------------------------------------------------------------------
# CSV writers
# ---------------------------------------------------------------------------

def write_csv(rows: list[dict], path: Path) -> None:
    if not rows:
        path.write_text("")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def write_funnel_csv(funnel: dict, path: Path) -> None:
    rows = [{"metric": k, "value": v} for k, v in funnel.items()]
    write_csv(rows, path)


def write_stage_loss_csv(rows: list[dict], path: Path) -> None:
    write_csv(rows, path)


def write_operator_pe_csv(rows: list[dict], path: Path) -> None:
    write_csv(rows, path)


def write_stage_freq_csv(rows: list[dict], path: Path) -> None:
    write_csv(rows, path)


def write_obj_stats_csv(stats: dict, path: Path) -> None:
    rows = [{"metric": k, "value": v} for k, v in stats.items()]
    write_csv(rows, path)


def write_tournament_pe_csv(rows: dict, path: Path) -> None:
    write_csv(list(rows.values()), path)


def write_population_pe_csv(rows: dict, path: Path) -> None:
    write_csv(list(rows.values()), path)


def write_elite_pe_csv(rows: dict, path: Path) -> None:
    write_csv(list(rows.values()), path)


def write_globalbest_pe_csv(rows: dict, path: Path) -> None:
    write_csv(list(rows.values()), path)


def write_reason_csv(rows: list[dict], path: Path) -> None:
    write_csv(rows, path)


def write_population_slot_csv(rows: list[dict], path: Path) -> None:
    write_csv(rows, path)


def write_osr_by_zone_csv(rows: list[dict], path: Path) -> None:
    write_csv(rows, path)


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def generate_report(
    funnel: dict,
    loss_rows: list[dict],
    op_data: list[dict],
    stage_freq: list[dict],
    obj_stats: dict,
    metrics: dict,
    telemetry_dir: str,
    phase2: bool = False,
    tourn_pe: dict | None = None,
    pop_pe: dict | None = None,
    elite_pe: dict | None = None,
    gb_pe: dict | None = None,
    reason_freq: list[dict] | None = None,
    pop_slot_dist: list[dict] | None = None,
    osr_by_zone: list[dict] | None = None,
) -> str:
    lines: list[str] = []
    a = lines.append

    a("# RP-410C Selection Analysis Report")
    a("")
    a(f"**Telemetry directory:** `{telemetry_dir}`")
    a(f"**Total candidate records:** {metrics['total_records']:,}")
    a(f"**Instances:** {metrics['instance_count']}")
    a(f"**Phase 2 analysis:** {'enabled' if phase2 else 'disabled'}")
    a("")

    # --- Survival funnel ---
    a("## 1. Survival Funnel")
    a("")
    a("| Stage | Count | Rate |")
    a("|-------|-------|------|")
    a(f"| Generated | {funnel['generated']:,} | 100% |")
    a(f"| Tournament Winners | {funnel['tournament_winners']:,} | {funnel['pct_won_tournament']}% |")
    a(f"| Entered Population | {funnel['entered_population']:,} | {funnel['pct_entered_pop']}% of winners |")
    a(f"| Entered Elite | {funnel['entered_elite']:,} | {funnel['pct_entered_elite']}% of pop |")
    a(f"| Became Global Best | {funnel['became_global_best']:,} | {funnel['pct_global_best']}% of elite |")
    a(f"| **Overall OSR** | {funnel['became_global_best']:,} | **{funnel['overall_osr']}%** |")
    a("")

    # --- Stage loss rates ---
    a("## 2. Stage Loss Rates")
    a("")
    a("| Stage | Pool | Survivors | Lost | Loss Rate |")
    a("|-------|------|-----------|------|-----------|")
    for row in loss_rows:
        a(f"| {row['stage']} | {row['pool']:,} | {row['survivors']:,} | {row['lost']:,} | {row['loss_rate_pct']}% |")
    a("")

    # --- Operator PE ---
    a("## 3. Operator Promotion Efficiency")
    a("")
    a("| Operator | Tournament Wins | Entered Population | PE% |")
    a("|----------|----------------|-------------------|-----|")
    for row in op_data:
        a(f"| {row['operator']} | {row['tournament_wins']:,} | {row['entered_population']:,} | {row['pe_pct']}% |")
    a("")

    # --- Stage frequency ---
    a("## 4. Decision Stage × Reason Frequency")
    a("")
    a("| Stage | Reason | Count |")
    a("|-------|--------|-------|")
    for row in stage_freq[:20]:
        a(f"| {row['decision_stage']} | {row['reason']} | {row['count']:,} |")
    a("")

    # --- Objective stats ---
    a("## 5. Objective Value Statistics (valid candidates)")
    a("")
    a(f"- Count: {obj_stats['count']:,}")
    a(f"- Min: {obj_stats['min']}")
    a(f"- Max: {obj_stats['max']}")
    a(f"- Mean: {obj_stats['mean']}")
    a("")

    if phase2:
        a("---")
        a("")
        a("## Phase 2 — DecisionEvent Analysis")
        a("")

        # --- OSR by zone ---
        if osr_by_zone:
            a("### 6. End-to-End OSR by Zone")
            a("")
            a("| Zone | Generated | Tourn Win | Pop | Elite | GlobalBest | PE_Tourn | PE_Promo | PE_Elite | PE_GB | OSR |")
            a("|------|-----------|-----------|-----|-------|------------|----------|----------|----------|-------|-----|")
            for row in osr_by_zone:
                a(f"| {row['zone']} | {row['generated']:,} | {row['tournament_winners']:,} | "
                  f"{row['entered_population']:,} | {row['entered_elite']:,} | {row['became_global_best']:,} | "
                  f"{row['pe_tournament_pct']}% | {row['pe_promotion_pct']}% | "
                  f"{row['pe_elite_pct']}% | {row['pe_globalbest_pct']}% | {row['osr_pct']}% |")
            a("")

        # --- Tournament PE by zone ---
        if tourn_pe:
            a("### 7. Tournament PE by Zone")
            a("")
            a("| Zone | Participants | Winners | Win Rate |")
            a("|------|-------------|---------|----------|")
            for z in ("Peak", "Shoulder", "Transition", "Tail"):
                row = tourn_pe[z]
                a(f"| {z} | {row['participants']:,} | {row['winners']:,} | {row['win_rate_pct']}% |")
            a("")

        # --- Population PE by zone ---
        if pop_pe:
            a("### 8. Promotion PE by Zone")
            a("")
            a("| Zone | Tournament Winners | Entered Population | Promotion PE |")
            a("|------|-------------------|-------------------|--------------|")
            for z in ("Peak", "Shoulder", "Transition", "Tail"):
                row = pop_pe[z]
                a(f"| {z} | {row['tournament_winners']:,} | {row['entered_population']:,} | {row['promotion_pe_pct']}% |")
            a("")

        # --- Elite PE by zone ---
        if elite_pe:
            a("### 9. Elite PE by Zone")
            a("")
            a("| Zone | Entered Population | Entered Elite | Elite PE |")
            a("|------|-------------------|--------------|----------|")
            for z in ("Peak", "Shoulder", "Transition", "Tail"):
                row = elite_pe[z]
                a(f"| {z} | {row['entered_population']:,} | {row['entered_elite']:,} | {row['elite_pe_pct']}% |")
            a("")

        # --- GlobalBest PE by zone ---
        if gb_pe:
            a("### 10. GlobalBest PE by Zone")
            a("")
            a("| Zone | Entered Elite | Became GlobalBest | GlobalBest PE |")
            a("|------|--------------|------------------|---------------|")
            for z in ("Peak", "Shoulder", "Transition", "Tail"):
                row = gb_pe[z]
                a(f"| {z} | {row['entered_elite']:,} | {row['became_global_best']:,} | {row['globalbest_pe_pct']}% |")
            a("")

        # --- Reason frequency by zone ---
        if reason_freq:
            a("### 11. Rejection Reason Frequency by Zone (top 30)")
            a("")
            a("| Stage | Reason | Zone | Count |")
            a("|-------|--------|------|-------|")
            for row in reason_freq[:30]:
                a(f"| {row['decision_stage']} | {row['reason']} | {row['zone']} | {row['count']:,} |")
            a("")

        # --- Population slot distribution ---
        if pop_slot_dist:
            a("### 12. Population Slot Distribution by Zone")
            a("")
            a("| Zone | Count | Min Slot | Max Slot | Mean Slot |")
            a("|------|-------|----------|----------|-----------|")
            for row in pop_slot_dist:
                a(f"| {row['zone']} | {row['count']:,} | {row['min_slot']} | {row['max_slot']} | {row['mean_slot']} |")
            a("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="RP-410C Selection Analysis")
    parser.add_argument(
        "--telemetry-dir",
        required=True,
        type=Path,
        help="Directory containing rp410_moves_*.jsonl and rp410_generations_*.jsonl files",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=Path,
        help="Directory to write CSV outputs and report",
    )
    parser.add_argument(
        "--phase2",
        action="store_true",
        default=False,
        help="Enable Phase 2 analysis sections (Tournament PE, Population PE, Elite PE, GlobalBest PE, reason frequency)",
    )
    args = parser.parse_args()

    telemetry_dir: Path = args.telemetry_dir
    output_dir: Path = args.output_dir
    phase2: bool = args.phase2

    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading candidate records from {telemetry_dir} ...")
    records = load_candidate_records(telemetry_dir)
    if not records:
        print("ERROR: No candidate records loaded. Aborting.", file=sys.stderr)
        sys.exit(1)

    instances = {r.get("instance") for r in records}
    print(f"  Loaded {len(records):,} records from {len(instances)} instances.")

    metrics = {
        "total_records": len(records),
        "instance_count": len(instances),
    }

    # --- Phase 1 computations ---
    print("Computing survival funnel ...")
    funnel = compute_survival_funnel(records)

    print("Computing stage loss rates ...")
    loss_rows = compute_stage_loss_rates(records)

    print("Computing operator PE ...")
    op_data = compute_operator_pe(records)

    print("Computing stage frequency ...")
    stage_freq = compute_stage_freq(records)

    print("Computing objective stats ...")
    obj_stats = compute_obj_stats(records)

    # --- Phase 2 computations ---
    tourn_pe = None
    pop_pe = None
    elite_pe = None
    gb_pe = None
    reason_freq = None
    pop_slot_dist = None
    osr_by_zone = None

    if phase2:
        print("Phase 2: Computing tournament PE by zone ...")
        tourn_pe = compute_tournament_pe_by_zone(records)

        print("Phase 2: Computing population PE by zone ...")
        pop_pe = compute_population_pe_by_zone(records)

        print("Phase 2: Computing elite PE by zone ...")
        elite_pe = compute_elite_pe_by_zone(records)

        print("Phase 2: Computing GlobalBest PE by zone ...")
        gb_pe = compute_globalbest_pe_by_zone(records)

        print("Phase 2: Computing reason frequency ...")
        reason_freq = compute_reason_frequency(records)

        print("Phase 2: Computing population slot distribution ...")
        pop_slot_dist = compute_population_slot_dist(records)

        print("Phase 2: Computing OSR by zone ...")
        osr_by_zone = compute_osr_by_zone(records)

    # --- Write CSVs ---
    print(f"Writing CSVs to {output_dir} ...")
    write_funnel_csv(funnel, output_dir / "funnel.csv")
    write_stage_loss_csv(loss_rows, output_dir / "stage_loss.csv")
    write_operator_pe_csv(op_data, output_dir / "operator_pe.csv")
    write_stage_freq_csv(stage_freq, output_dir / "stage_freq.csv")
    write_obj_stats_csv(obj_stats, output_dir / "obj_stats.csv")

    if phase2:
        write_tournament_pe_csv(tourn_pe, output_dir / "tournament_pe_by_zone.csv")
        write_population_pe_csv(pop_pe, output_dir / "population_pe_by_zone.csv")
        write_elite_pe_csv(elite_pe, output_dir / "elite_pe_by_zone.csv")
        write_globalbest_pe_csv(gb_pe, output_dir / "globalbest_pe_by_zone.csv")
        write_reason_csv(reason_freq, output_dir / "reason_freq_by_zone.csv")
        write_population_slot_csv(pop_slot_dist, output_dir / "population_slot_dist.csv")
        write_osr_by_zone_csv(osr_by_zone, output_dir / "osr_by_zone.csv")

    # --- Generate report ---
    print("Generating report ...")
    report = generate_report(
        funnel=funnel,
        loss_rows=loss_rows,
        op_data=op_data,
        stage_freq=stage_freq,
        obj_stats=obj_stats,
        metrics=metrics,
        telemetry_dir=str(telemetry_dir),
        phase2=phase2,
        tourn_pe=tourn_pe,
        pop_pe=pop_pe,
        elite_pe=elite_pe,
        gb_pe=gb_pe,
        reason_freq=reason_freq,
        pop_slot_dist=pop_slot_dist,
        osr_by_zone=osr_by_zone,
    )

    report_path = output_dir / "RP410C_PHASE2_ANALYSIS_REPORT.md"
    report_path.write_text(report)
    print(f"Report written to {report_path}")
    print("Done.")


if __name__ == "__main__":
    main()