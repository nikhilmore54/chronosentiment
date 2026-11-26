#!/usr/bin/env python3
"""
RP-409A Operator Attribution Analysis
======================================
Per-operator end-to-end attribution table and operator transition matrix.

Inputs:
  /tmp/rp408b/scalar/rp408b_moves_<inst>.jsonl   — MoveRecord stream (scalar arm)
  /tmp/rp408b/scalar/results.json                — instance summary

Outputs (docs/roadef/rp409a_data/):
  operator_attribution.csv   — per-operator: Peak COR, Peak PE, Peak OSR,
                               Shoulder COR/PE/OSR, Transition COR/PE/OSR,
                               Mean ΔObjective, Accepted Moves, Zone breakdown
  transition_matrix.csv      — operator × zone → next-move zone (Markov counts)
  zone_sequence.csv          — per-instance zone sequence (for Markov analysis)
  rp409a_analysis.log        — narrative log

Definitions:
  COR  = Contribution Rate = moves_in_zone / total_moves_for_operator
  PE   = Promotion Efficiency = moves_in_zone / total_moves_all_operators (global share)
  OSR  = Objective Step Rate = moves_in_zone / n_generations
  Mean ΔObjective = mean(prev_obj - new_obj) per accepted move (positive = improvement)
"""

import json
import os
import csv
import math
from collections import defaultdict

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
SCALAR_DIR = "/tmp/rp408b/scalar"
OUT_DIR    = "docs/roadef/rp409a_data"
os.makedirs(OUT_DIR, exist_ok=True)

LOG_PATH = os.path.join(OUT_DIR, "rp409a_analysis.log")
log_lines = []

def log(msg=""):
    print(msg)
    log_lines.append(msg)

def flush_log():
    with open(LOG_PATH, "w") as f:
        f.write("\n".join(log_lines) + "\n")

ZONES = ["peak", "shoulder", "transition", "tail", "mixed", "neutral"]
OPERATORS = ["crossover", "mutation"]

# ---------------------------------------------------------------------------
# Load results.json
# ---------------------------------------------------------------------------
def load_results():
    path = os.path.join(SCALAR_DIR, "results.json")
    with open(path) as f:
        data = json.load(f)
    return {r["name"]: r for r in data.get("results", [])}

# ---------------------------------------------------------------------------
# Load moves JSONL for one instance
# ---------------------------------------------------------------------------
def load_moves(inst_name):
    path = os.path.join(SCALAR_DIR, f"rp408b_moves_{inst_name}.jsonl")
    if not os.path.exists(path):
        return []
    records = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return records

# ---------------------------------------------------------------------------
# Per-operator attribution for one instance
# ---------------------------------------------------------------------------
def compute_operator_attribution(moves, n_generations):
    """
    Returns dict keyed by operator, each containing:
      total_moves, zone_counts{}, zone_delta_obj{},
      total_delta_obj, mean_delta_obj
    """
    stats = {}
    for op in OPERATORS:
        stats[op] = {
            "total_moves": 0,
            "zone_counts": defaultdict(int),
            "zone_delta_obj": defaultdict(list),
            "total_delta_obj": 0.0,
            "delta_obj_list": [],
        }

    for m in moves:
        op = m.get("operator", "unknown")
        if op not in stats:
            stats[op] = {
                "total_moves": 0,
                "zone_counts": defaultdict(int),
                "zone_delta_obj": defaultdict(list),
                "total_delta_obj": 0.0,
                "delta_obj_list": [],
            }
        zone = m.get("move_class", "unknown")
        prev_obj = m.get("prev_obj") or 0.0
        new_obj  = m.get("new_obj")  or 0.0
        delta_obj = prev_obj - new_obj  # positive = improvement

        stats[op]["total_moves"] += 1
        stats[op]["zone_counts"][zone] += 1
        stats[op]["zone_delta_obj"][zone].append(delta_obj)
        stats[op]["total_delta_obj"] += delta_obj
        stats[op]["delta_obj_list"].append(delta_obj)

    total_all_moves = sum(s["total_moves"] for s in stats.values())

    result = {}
    for op, s in stats.items():
        n = s["total_moves"]
        row = {
            "total_moves": n,
            "total_delta_obj": s["total_delta_obj"],
            "mean_delta_obj": s["total_delta_obj"] / n if n > 0 else 0.0,
        }
        for zone in ZONES:
            zc = s["zone_counts"].get(zone, 0)
            # COR = zone_moves / operator_total_moves
            row[f"{zone}_cor"] = zc / n if n > 0 else 0.0
            # PE = zone_moves / total_all_moves (global share)
            row[f"{zone}_pe"] = zc / total_all_moves if total_all_moves > 0 else 0.0
            # OSR = zone_moves / n_generations
            row[f"{zone}_osr"] = zc / n_generations if n_generations > 0 else 0.0
            row[f"{zone}_count"] = zc
            # Mean delta obj for this zone
            zdl = s["zone_delta_obj"].get(zone, [])
            row[f"{zone}_mean_delta_obj"] = sum(zdl) / len(zdl) if zdl else 0.0
        result[op] = row

    return result, total_all_moves

# ---------------------------------------------------------------------------
# Operator transition matrix for one instance
# ---------------------------------------------------------------------------
def compute_transition_matrix(moves):
    """
    Returns dict: (from_zone, to_zone) → count
    Transition = consecutive accepted moves (gen N → gen N+1).
    """
    matrix = defaultdict(int)
    prev_zone = None
    for m in sorted(moves, key=lambda x: x.get("generation", 0)):
        zone = m.get("move_class", "unknown")
        if prev_zone is not None:
            matrix[(prev_zone, zone)] += 1
        prev_zone = zone
    return matrix

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    log("=" * 70)
    log("RP-409A Operator Attribution Analysis")
    log("=" * 70)
    log()

    results_by_name = load_results()
    all_names = sorted(results_by_name.keys())
    log(f"Instances: {len(all_names)}")
    log()

    # Aggregate across all instances
    agg_op_stats = {op: defaultdict(float) for op in OPERATORS}
    agg_op_counts = {op: 0 for op in OPERATORS}
    agg_transition = defaultdict(int)
    instance_rows = []

    for name in all_names:
        r = results_by_name[name]
        moves = load_moves(name)
        if not moves:
            log(f"  {name}: no moves — skipped")
            continue

        n_gens = r.get("generations", 1)
        op_attr, total_moves = compute_operator_attribution(moves, n_gens)
        trans = compute_transition_matrix(moves)

        # Accumulate transition matrix
        for (fz, tz), cnt in trans.items():
            agg_transition[(fz, tz)] += cnt

        # Accumulate per-operator stats
        for op, row in op_attr.items():
            if op not in agg_op_stats:
                agg_op_stats[op] = defaultdict(float)
                agg_op_counts[op] = 0
            agg_op_counts[op] += 1
            for k, v in row.items():
                if isinstance(v, (int, float)):
                    agg_op_stats[op][k] += v

        # Per-instance row for CSV
        for op, row in op_attr.items():
            instance_rows.append({
                "instance": name,
                "operator": op,
                **row,
            })

    # ---------------------------------------------------------------------------
    # Aggregate operator attribution table
    # ---------------------------------------------------------------------------
    log("─" * 70)
    log("OPERATOR ATTRIBUTION TABLE (mean across instances)")
    log("─" * 70)
    log()

    # Header
    log(f"  {'Operator':<12} {'Moves':>7} {'MeanΔObj':>10} "
        f"{'PeakCOR':>9} {'PeakPE':>8} {'PeakOSR':>9} "
        f"{'ShldrCOR':>9} {'ShldrPE':>8} {'ShldrOSR':>9} "
        f"{'TransCOR':>9} {'TransPE':>8} {'TransOSR':>9}")
    log(f"  {'─'*12} {'─'*7} {'─'*10} "
        f"{'─'*9} {'─'*8} {'─'*9} "
        f"{'─'*9} {'─'*8} {'─'*9} "
        f"{'─'*9} {'─'*8} {'─'*9}")

    agg_summary_rows = []
    for op in OPERATORS:
        n = agg_op_counts.get(op, 0)
        if n == 0:
            continue
        s = agg_op_stats[op]
        def mean(k): return s[k] / n if n > 0 else 0.0

        row = {
            "operator":          op,
            "mean_total_moves":  mean("total_moves"),
            "mean_delta_obj":    mean("mean_delta_obj"),
            "peak_cor":          mean("peak_cor"),
            "peak_pe":           mean("peak_pe"),
            "peak_osr":          mean("peak_osr"),
            "shoulder_cor":      mean("shoulder_cor"),
            "shoulder_pe":       mean("shoulder_pe"),
            "shoulder_osr":      mean("shoulder_osr"),
            "transition_cor":    mean("transition_cor"),
            "transition_pe":     mean("transition_pe"),
            "transition_osr":    mean("transition_osr"),
            "tail_cor":          mean("tail_cor"),
            "tail_pe":           mean("tail_pe"),
            "mixed_cor":         mean("mixed_cor"),
            "mixed_pe":          mean("mixed_pe"),
            "neutral_cor":       mean("neutral_cor"),
            "neutral_pe":        mean("neutral_pe"),
        }
        agg_summary_rows.append(row)

        log(f"  {op:<12} {mean('total_moves'):>7.1f} {mean('mean_delta_obj'):>10.4f} "
            f"{mean('peak_cor'):>9.4f} {mean('peak_pe'):>8.4f} {mean('peak_osr'):>9.4f} "
            f"{mean('shoulder_cor'):>9.4f} {mean('shoulder_pe'):>8.4f} {mean('shoulder_osr'):>9.4f} "
            f"{mean('transition_cor'):>9.4f} {mean('transition_pe'):>8.4f} {mean('transition_osr'):>9.4f}")

    log()

    # ---------------------------------------------------------------------------
    # Zone breakdown per operator
    # ---------------------------------------------------------------------------
    log("─" * 70)
    log("ZONE BREAKDOWN PER OPERATOR (mean COR across instances)")
    log("─" * 70)
    log()
    log(f"  {'Operator':<12} {'Peak':>8} {'Shoulder':>10} {'Transition':>12} {'Tail':>8} {'Mixed':>8} {'Neutral':>9}")
    log(f"  {'─'*12} {'─'*8} {'─'*10} {'─'*12} {'─'*8} {'─'*8} {'─'*9}")
    for row in agg_summary_rows:
        op = row["operator"]
        n = agg_op_counts.get(op, 1)
        log(f"  {op:<12} "
            f"{row['peak_cor']:>8.4f} "
            f"{row['shoulder_cor']:>10.4f} "
            f"{row['transition_cor']:>12.4f} "
            f"{row['tail_cor']:>8.4f} "
            f"{row['mixed_cor']:>8.4f} "
            f"{row['neutral_cor']:>9.4f}")

    log()

    # ---------------------------------------------------------------------------
    # Operator transition matrix
    # ---------------------------------------------------------------------------
    log("─" * 70)
    log("OPERATOR TRANSITION MATRIX (zone → zone, aggregated across instances)")
    log("─" * 70)
    log("(Rows = from-zone, Columns = to-zone, values = move counts)")
    log()

    all_zones_seen = sorted(set(fz for fz, _ in agg_transition) | set(tz for _, tz in agg_transition))
    header = f"  {'From\\To':<14}" + "".join(f"{z:>12}" for z in all_zones_seen)
    log(header)
    log("  " + "─" * (14 + 12 * len(all_zones_seen)))

    for fz in all_zones_seen:
        row_total = sum(agg_transition[(fz, tz)] for tz in all_zones_seen)
        row_str = f"  {fz:<14}"
        for tz in all_zones_seen:
            cnt = agg_transition[(fz, tz)]
            pct = cnt / row_total * 100 if row_total > 0 else 0.0
            row_str += f"{cnt:>8}({pct:>3.0f}%)"
        log(row_str)

    log()

    # ---------------------------------------------------------------------------
    # Limiting operator identification
    # ---------------------------------------------------------------------------
    log("─" * 70)
    log("LIMITING OPERATOR IDENTIFICATION")
    log("─" * 70)
    log()
    log("An operator is 'limiting' for a zone if its COR for that zone is")
    log("substantially lower than the other operator's COR for the same zone.")
    log()

    for zone in ["peak", "shoulder", "transition"]:
        cors = {row["operator"]: row[f"{zone}_cor"] for row in agg_summary_rows}
        if len(cors) < 2:
            continue
        ops = list(cors.keys())
        v0, v1 = cors[ops[0]], cors[ops[1]]
        if v0 + v1 < 1e-9:
            log(f"  {zone.upper()}: no moves in this zone")
            continue
        dominant = ops[0] if v0 > v1 else ops[1]
        limiting = ops[1] if v0 > v1 else ops[0]
        ratio = max(v0, v1) / max(min(v0, v1), 1e-9)
        log(f"  {zone.upper():12}: dominant={dominant} (COR={max(v0,v1):.4f}), "
            f"limiting={limiting} (COR={min(v0,v1):.4f}), ratio={ratio:.1f}×")

    log()

    # ---------------------------------------------------------------------------
    # Write CSV outputs
    # ---------------------------------------------------------------------------
    # operator_attribution.csv (per-instance)
    if instance_rows:
        attr_path = os.path.join(OUT_DIR, "operator_attribution.csv")
        with open(attr_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(instance_rows[0].keys()))
            writer.writeheader()
            writer.writerows(instance_rows)
        log(f"  Written: {attr_path}")

    # operator_attribution_summary.csv (aggregated)
    if agg_summary_rows:
        summ_path = os.path.join(OUT_DIR, "operator_attribution_summary.csv")
        with open(summ_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(agg_summary_rows[0].keys()))
            writer.writeheader()
            writer.writerows(agg_summary_rows)
        log(f"  Written: {summ_path}")

    # transition_matrix.csv
    trans_rows = []
    for fz in all_zones_seen:
        row = {"from_zone": fz}
        for tz in all_zones_seen:
            row[f"to_{tz}"] = agg_transition[(fz, tz)]
        trans_rows.append(row)
    if trans_rows:
        trans_path = os.path.join(OUT_DIR, "transition_matrix.csv")
        with open(trans_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(trans_rows[0].keys()))
            writer.writeheader()
            writer.writerows(trans_rows)
        log(f"  Written: {trans_path}")

    log()
    log("=" * 70)
    log("RP-409A analysis complete.")
    log("=" * 70)
    flush_log()

if __name__ == "__main__":
    main()