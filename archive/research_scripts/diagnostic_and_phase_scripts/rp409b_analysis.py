#!/usr/bin/env python3
"""
RP-409B Analysis Script
=======================
Compares Uniform vs PeakTargeted mutation strategies across 20 instances.

Three-level analysis framework (per reviewer):
  Level 1 — Outcome:   final objective, win counts, Δobj statistics
  Level 2 — Mechanism: zone move counts (Peak/Shoulder/Transition/Tail/Mixed/Neutral)
                        from GenerationRecord; mutation ACR from MoveRecord
  Level 3 — Safety:    valid rate, construction IFR, generation count, stagnation

Schema (from actual telemetry):
  GenerationRecord: best_obj, moves_peak, moves_shoulder, moves_transition,
                    moves_tail, moves_mixed, moves_neutral, mutation_count,
                    crossover_count, stagnation, valid_count, total_gen_time_ms, ...
  MoveRecord:       operator, move_class, deltas{delta_rank1,...}, new_obj, prev_obj
  ConstructionRecord: initial_feasibility_rate, valid_count, invalid_count
  results.json:     dict with key "results" -> list of instance dicts

Usage:
  python3 scripts/rp409b_analysis.py \\
      --data /tmp/rp409b_campaign \\
      --out  docs/roadef/rp409b_data

Outputs:
  summary.txt          — executive summary (three-level)
  results_wide.csv     — one row per instance, both strategies side-by-side
  zone_moves.csv       — cumulative zone move counts by strategy × instance
  move_acr.csv         — ACR by operator × zone × strategy
  instance_detail.csv  — per-instance detail including scaling metrics
"""

import argparse
import json
import os
import sys
import csv
import math
from collections import defaultdict
from pathlib import Path

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def load_jsonl(path):
    records = []
    if not os.path.exists(path):
        return records
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    records.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
    return records

def load_results(strategy_dir):
    """Load results.json — returns list of instance result dicts."""
    path = os.path.join(strategy_dir, "results.json")
    if not os.path.exists(path):
        return []
    with open(path) as f:
        data = json.load(f)
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        return data.get("results", [])
    return []

def safe_div(a, b, default=0.0):
    return a / b if b > 0 else default

def mean(xs):
    xs = [x for x in xs if x is not None and not (isinstance(x, float) and math.isnan(x))]
    return sum(xs) / len(xs) if xs else float('nan')

def median(xs):
    xs = sorted(x for x in xs if x is not None and not (isinstance(x, float) and math.isnan(x)))
    n = len(xs)
    if n == 0:
        return float('nan')
    if n % 2 == 1:
        return xs[n // 2]
    return (xs[n // 2 - 1] + xs[n // 2]) / 2.0

def stdev(xs):
    xs = [x for x in xs if x is not None and not (isinstance(x, float) and math.isnan(x))]
    if len(xs) < 2:
        return float('nan')
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))

def sign_test_p(deltas):
    """One-sided sign test: P(PT < U) = P(delta < 0). Returns fraction of negatives."""
    neg = sum(1 for d in deltas if d < -1e-9)
    pos = sum(1 for d in deltas if d > 1e-9)
    n = neg + pos
    return neg, pos, n

ZONES = ["peak", "shoulder", "transition", "tail", "mixed", "neutral"]
OPERATORS = ["mutation", "crossover", "crossover+mutation", "elite", "initial"]

# ---------------------------------------------------------------------------
# Per-instance analysis
# ---------------------------------------------------------------------------

def analyse_instance(name, strategy, data_dir, result_row):
    """
    Returns a dict of metrics for one instance × strategy.
    result_row: the dict from results.json for this instance.
    """
    strat_dir = os.path.join(data_dir, strategy)
    moves_path = os.path.join(strat_dir, f"rp409b_moves_{name}.jsonl")
    gens_path  = os.path.join(strat_dir, f"rp409b_generations_{name}.jsonl")
    cons_path  = os.path.join(strat_dir, f"rp409b_construction_{name}.jsonl")

    moves = load_jsonl(moves_path)
    gens  = load_jsonl(gens_path)
    cons  = load_jsonl(cons_path)

    n_generations = len(gens)
    n_moves = len(moves)

    # Construction metrics
    ifr = cons[0].get("initial_feasibility_rate", float('nan')) if cons else float('nan')
    cons_valid = cons[0].get("valid_count", 0) if cons else 0
    cons_invalid = cons[0].get("invalid_count", 0) if cons else 0

    # Final objective from result_row (authoritative)
    final_obj = result_row.get("best_obj", float('nan'))
    valid = result_row.get("valid", False)
    runtime_ms = result_row.get("runtime_ms", 0)
    termination = result_row.get("termination_reason", "unknown")
    n_gens_result = result_row.get("generations", n_generations)

    # Instance size from result_row
    num_demands = result_row.get("num_demands", 0)
    num_nodes   = result_row.get("num_nodes", 0)
    num_links   = result_row.get("num_links", 0)

    # ms per generation
    ms_per_gen = safe_div(runtime_ms, n_gens_result) if n_gens_result > 0 else float('nan')

    # Zone move counts from GenerationRecord (cumulative across all generations)
    # Each generation record has moves_peak etc. = moves accepted in THAT generation
    zone_counts = defaultdict(int)
    total_mutation_count = 0
    total_crossover_count = 0
    max_stagnation = 0
    for g in gens:
        for zone in ZONES:
            zone_counts[zone] += g.get(f"moves_{zone}", 0)
        total_mutation_count  += g.get("mutation_count", 0)
        total_crossover_count += g.get("crossover_count", 0)
        max_stagnation = max(max_stagnation, g.get("stagnation", 0))

    total_zone_moves = sum(zone_counts[z] for z in ZONES)

    # Zone APS (Accepted Promotion Share) from generation records
    aps = {zone: safe_div(zone_counts[zone], total_zone_moves) for zone in ZONES}

    # ACR from MoveRecord: operator → zone → count
    op_zone = defaultdict(lambda: defaultdict(int))
    op_total = defaultdict(int)
    for m in moves:
        op  = m.get("operator", "unknown").lower()
        mc  = m.get("move_class", "neutral").lower()
        op_zone[op][mc] += 1
        op_total[op] += 1

    acr = {}
    for op in op_zone:
        acr[op] = {zone: safe_div(op_zone[op][zone], op_total[op]) for zone in ZONES}

    # Mutation Peak ACR (primary RP-409B metric)
    mut_peak_acr = acr.get("mutation", {}).get("peak", 0.0)
    mut_peak_abs = op_zone.get("mutation", {}).get("peak", 0)
    mut_total    = op_total.get("mutation", 0)

    return {
        "instance": name,
        "strategy": strategy,
        "valid": valid,
        "final_obj": final_obj,
        "runtime_ms": runtime_ms,
        "n_generations": n_gens_result,
        "ms_per_gen": ms_per_gen,
        "termination": termination,
        "num_demands": num_demands,
        "num_nodes": num_nodes,
        "num_links": num_links,
        "ifr": ifr,
        "cons_valid": cons_valid,
        "cons_invalid": cons_invalid,
        "n_moves": n_moves,
        "total_zone_moves": total_zone_moves,
        "total_mutation_count": total_mutation_count,
        "total_crossover_count": total_crossover_count,
        "max_stagnation": max_stagnation,
        "zone_counts": dict(zone_counts),
        "aps": aps,
        "acr": acr,
        "op_zone": dict(op_zone),
        "op_total": dict(op_total),
        "mut_peak_acr": mut_peak_acr,
        "mut_peak_abs": mut_peak_abs,
        "mut_total": mut_total,
        "peak_aps": aps.get("peak", 0.0),
        "shoulder_aps": aps.get("shoulder", 0.0),
        "transition_aps": aps.get("transition", 0.0),
        "tail_aps": aps.get("tail", 0.0),
        "mixed_aps": aps.get("mixed", 0.0),
        "neutral_aps": aps.get("neutral", 0.0),
    }

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="RP-409B A/B mutation strategy analysis")
    parser.add_argument("--data", required=True, help="Campaign output directory")
    parser.add_argument("--out",  required=True, help="Output directory for analysis artefacts")
    args = parser.parse_args()

    data_dir = args.data
    out_dir  = args.out
    os.makedirs(out_dir, exist_ok=True)

    # Load results from both arms
    u_results_list  = load_results(os.path.join(data_dir, "uniform"))
    pt_results_list = load_results(os.path.join(data_dir, "peak_targeted"))

    if not u_results_list:
        print(f"ERROR: No results found in {data_dir}/uniform/", file=sys.stderr)
        sys.exit(1)
    if not pt_results_list:
        print(f"ERROR: No results found in {data_dir}/peak_targeted/", file=sys.stderr)
        sys.exit(1)

    # Index by instance name
    u_by_name  = {r["name"]: r for r in u_results_list}
    pt_by_name = {r["name"]: r for r in pt_results_list}
    all_names  = sorted(set(u_by_name) | set(pt_by_name))
    common_names = sorted(set(u_by_name) & set(pt_by_name))

    print(f"Uniform instances:       {len(u_by_name)}")
    print(f"PeakTargeted instances:  {len(pt_by_name)}")
    print(f"Common instances:        {len(common_names)}")

    # Per-instance analysis
    rows_u  = {}
    rows_pt = {}
    for name in common_names:
        rows_u[name]  = analyse_instance(name, "uniform",       data_dir, u_by_name[name])
        rows_pt[name] = analyse_instance(name, "peak_targeted", data_dir, pt_by_name[name])

    # -----------------------------------------------------------------------
    # Classify pairs
    # -----------------------------------------------------------------------
    both_valid   = [n for n in common_names if rows_u[n]["valid"] and rows_pt[n]["valid"]]
    both_invalid = [n for n in common_names if not rows_u[n]["valid"] and not rows_pt[n]["valid"]]
    pt_only_valid = [n for n in common_names if not rows_u[n]["valid"] and rows_pt[n]["valid"]]
    u_only_valid  = [n for n in common_names if rows_u[n]["valid"] and not rows_pt[n]["valid"]]

    # Δobj for both-valid pairs (negative = PT better)
    delta_objs = {n: rows_pt[n]["final_obj"] - rows_u[n]["final_obj"] for n in both_valid}
    pt_wins = [n for n in both_valid if delta_objs[n] < -1e-9]
    u_wins  = [n for n in both_valid if delta_objs[n] > 1e-9]
    ties    = [n for n in both_valid if abs(delta_objs[n]) <= 1e-9]

    delta_list = [delta_objs[n] for n in both_valid]
    neg, pos, n_sign = sign_test_p(delta_list)

    # -----------------------------------------------------------------------
    # results_wide.csv
    # -----------------------------------------------------------------------
    wide_path = os.path.join(out_dir, "results_wide.csv")
    with open(wide_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow([
            "instance", "num_nodes", "num_links", "num_demands",
            "u_valid", "pt_valid",
            "u_final_obj", "pt_final_obj", "delta_obj",
            "u_n_gens", "pt_n_gens",
            "u_ms_per_gen", "pt_ms_per_gen",
            "u_runtime_ms", "pt_runtime_ms",
            "u_ifr", "pt_ifr",
            "u_peak_aps", "pt_peak_aps",
            "u_shoulder_aps", "pt_shoulder_aps",
            "u_mut_peak_acr", "pt_mut_peak_acr",
            "u_mut_peak_abs", "pt_mut_peak_abs",
            "u_mut_total", "pt_mut_total",
            "u_termination", "pt_termination",
        ])
        for name in common_names:
            u = rows_u[name]
            p = rows_pt[name]
            delta = (p["final_obj"] - u["final_obj"]) if (u["valid"] and p["valid"]) else float('nan')
            w.writerow([
                name, u["num_nodes"], u["num_links"], u["num_demands"],
                u["valid"], p["valid"],
                f"{u['final_obj']:.6f}" if u["valid"] else "inf",
                f"{p['final_obj']:.6f}" if p["valid"] else "inf",
                f"{delta:.6f}" if not math.isnan(delta) else "n/a",
                u["n_generations"], p["n_generations"],
                f"{u['ms_per_gen']:.1f}", f"{p['ms_per_gen']:.1f}",
                u["runtime_ms"], p["runtime_ms"],
                f"{u['ifr']:.4f}", f"{p['ifr']:.4f}",
                f"{u['peak_aps']:.4f}", f"{p['peak_aps']:.4f}",
                f"{u['shoulder_aps']:.4f}", f"{p['shoulder_aps']:.4f}",
                f"{u['mut_peak_acr']:.4f}", f"{p['mut_peak_acr']:.4f}",
                u["mut_peak_abs"], p["mut_peak_abs"],
                u["mut_total"], p["mut_total"],
                u["termination"], p["termination"],
            ])
    print(f"Written: {wide_path}")

    # -----------------------------------------------------------------------
    # zone_moves.csv — cumulative zone move counts by strategy × instance
    # -----------------------------------------------------------------------
    zone_path = os.path.join(out_dir, "zone_moves.csv")
    with open(zone_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["strategy", "instance"] + ZONES + ["total", "peak_frac", "shoulder_frac"])
        for strategy, rows in [("uniform", rows_u), ("peak_targeted", rows_pt)]:
            for name in common_names:
                r = rows[name]
                zc = r["zone_counts"]
                total = r["total_zone_moves"]
                row = [strategy, name]
                row += [zc.get(z, 0) for z in ZONES]
                row += [total,
                        f"{safe_div(zc.get('peak',0), total):.4f}",
                        f"{safe_div(zc.get('shoulder',0), total):.4f}"]
                w.writerow(row)
    print(f"Written: {zone_path}")

    # -----------------------------------------------------------------------
    # move_acr.csv — ACR by operator × zone × strategy (mean across instances)
    # -----------------------------------------------------------------------
    acr_path = os.path.join(out_dir, "move_acr.csv")
    with open(acr_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["strategy", "operator", "zone", "mean_acr", "median_acr", "n_instances"])
        for strategy, rows in [("uniform", rows_u), ("peak_targeted", rows_pt)]:
            for op in OPERATORS:
                for zone in ZONES:
                    vals = [rows[n]["acr"].get(op, {}).get(zone, 0.0) for n in common_names]
                    w.writerow([strategy, op, zone,
                                f"{mean(vals):.4f}", f"{median(vals):.4f}", len(vals)])
    print(f"Written: {acr_path}")

    # -----------------------------------------------------------------------
    # instance_detail.csv — per-instance scaling and mechanism detail
    # -----------------------------------------------------------------------
    detail_path = os.path.join(out_dir, "instance_detail.csv")
    with open(detail_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow([
            "instance", "num_nodes", "num_links", "num_demands",
            "strategy", "valid", "final_obj", "n_gens", "ms_per_gen",
            "total_zone_moves", "peak_moves", "shoulder_moves",
            "mutation_count", "crossover_count",
            "mut_peak_abs", "mut_total", "mut_peak_acr",
            "peak_aps", "shoulder_aps", "max_stagnation", "ifr",
        ])
        for name in common_names:
            for strategy, rows in [("uniform", rows_u), ("peak_targeted", rows_pt)]:
                r = rows[name]
                zc = r["zone_counts"]
                w.writerow([
                    name, r["num_nodes"], r["num_links"], r["num_demands"],
                    strategy, r["valid"],
                    f"{r['final_obj']:.6f}" if r["valid"] else "inf",
                    r["n_generations"], f"{r['ms_per_gen']:.1f}",
                    r["total_zone_moves"], zc.get("peak", 0), zc.get("shoulder", 0),
                    r["total_mutation_count"], r["total_crossover_count"],
                    r["mut_peak_abs"], r["mut_total"], f"{r['mut_peak_acr']:.4f}",
                    f"{r['peak_aps']:.4f}", f"{r['shoulder_aps']:.4f}",
                    r["max_stagnation"], f"{r['ifr']:.4f}",
                ])
    print(f"Written: {detail_path}")

    # -----------------------------------------------------------------------
    # summary.txt — three-level executive summary
    # -----------------------------------------------------------------------
    summary_path = os.path.join(out_dir, "summary.txt")
    with open(summary_path, "w") as f:
        def p(s=""):
            print(s, file=f)
            print(s)

        p("=" * 72)
        p("RP-409B ANALYSIS SUMMARY")
        p("Uniform vs PeakTargeted Mutation Strategy — A/B Experiment")
        p("Seed: 42  |  Instances: 20  |  Date: 2026-08-06")
        p("=" * 72)
        p()
        p("─" * 72)
        p("LEVEL 1 — OUTCOME")
        p("─" * 72)
        p(f"  Total instances:          {len(common_names)}")
        p(f"  Both valid:               {len(both_valid)}")
        p(f"  Both invalid:             {len(both_invalid)}")
        p(f"  PeakTargeted-only valid:  {len(pt_only_valid)}  {pt_only_valid}")
        p(f"  Uniform-only valid:       {len(u_only_valid)}  {u_only_valid}")
        p()
        p(f"  Among {len(both_valid)} both-valid pairs:")
        p(f"    PT wins (obj lower):    {len(pt_wins)}  {pt_wins}")
        p(f"    U  wins (obj lower):    {len(u_wins)}  {u_wins}")
        p(f"    Ties:                   {len(ties)}")
        p()
        if delta_list:
            p(f"  Δobj (PT − U) statistics (n={len(delta_list)}):")
            p(f"    mean   = {mean(delta_list):+.4f}")
            p(f"    median = {median(delta_list):+.4f}")
            p(f"    stdev  = {stdev(delta_list):.4f}")
            p(f"    min    = {min(delta_list):+.4f}")
            p(f"    max    = {max(delta_list):+.4f}")
            p(f"  Sign test: {neg} negative / {pos} positive / {n_sign} total")
            p(f"  (negative = PT better; fraction PT better = {safe_div(neg,n_sign):.2f})")
        p()
        p("  Per-instance Δobj (PT − U, both-valid only):")
        for name in both_valid:
            d = delta_objs[name]
            winner = "PT" if d < -1e-9 else ("U" if d > 1e-9 else "tie")
            p(f"    {name:12s}  Δ={d:+.4f}  [{winner}]  "
              f"U={rows_u[name]['final_obj']:.4f}  PT={rows_pt[name]['final_obj']:.4f}")
        p()
        p("─" * 72)
        p("LEVEL 2 — MECHANISM")
        p("─" * 72)
        p()
        p("  Zone APS (mean across all instances, including invalid):")
        p(f"  {'Zone':12s}  {'Uniform':>10s}  {'PeakTargeted':>12s}  {'Δ':>8s}")
        for zone in ZONES:
            key = f"{zone}_aps"
            u_vals  = [rows_u[n].get(key, 0.0)  for n in common_names]
            pt_vals = [rows_pt[n].get(key, 0.0) for n in common_names]
            mu = mean(u_vals)
            mpt = mean(pt_vals)
            p(f"  {zone:12s}  {mu:10.4f}  {mpt:12.4f}  {mpt-mu:+8.4f}")
        p()
        p("  Mutation Peak ACR (fraction of mutation moves in Peak zone):")
        u_mut_peak = [rows_u[n]["mut_peak_acr"]  for n in common_names]
        pt_mut_peak = [rows_pt[n]["mut_peak_acr"] for n in common_names]
        p(f"    Uniform      mean={mean(u_mut_peak):.4f}  median={median(u_mut_peak):.4f}")
        p(f"    PeakTargeted mean={mean(pt_mut_peak):.4f}  median={median(pt_mut_peak):.4f}")
        p()
        p("  Mutation Peak absolute count (mean across instances):")
        u_mut_abs  = [rows_u[n]["mut_peak_abs"]  for n in common_names]
        pt_mut_abs = [rows_pt[n]["mut_peak_abs"] for n in common_names]
        p(f"    Uniform      mean={mean(u_mut_abs):.2f}")
        p(f"    PeakTargeted mean={mean(pt_mut_abs):.2f}")
        p()
        p("  Total mutation count (mean across instances):")
        u_mut_tot  = [rows_u[n]["mut_total"]  for n in common_names]
        pt_mut_tot = [rows_pt[n]["mut_total"] for n in common_names]
        p(f"    Uniform      mean={mean(u_mut_tot):.1f}")
        p(f"    PeakTargeted mean={mean(pt_mut_tot):.1f}")
        p()
        p("─" * 72)
        p("LEVEL 3 — SAFETY")
        p("─" * 72)
        p()
        p("  Valid rate:")
        u_valid_count  = sum(1 for n in common_names if rows_u[n]["valid"])
        pt_valid_count = sum(1 for n in common_names if rows_pt[n]["valid"])
        p(f"    Uniform      {u_valid_count}/{len(common_names)}")
        p(f"    PeakTargeted {pt_valid_count}/{len(common_names)}")
        p()
        p("  Construction IFR (should be identical — same seed formula):")
        u_ifr  = [rows_u[n]["ifr"]  for n in common_names]
        pt_ifr = [rows_pt[n]["ifr"] for n in common_names]
        p(f"    Uniform      mean={mean(u_ifr):.4f}")
        p(f"    PeakTargeted mean={mean(pt_ifr):.4f}")
        p()
        p("  Generation count (mean):")
        u_gens  = [rows_u[n]["n_generations"]  for n in common_names]
        pt_gens = [rows_pt[n]["n_generations"] for n in common_names]
        p(f"    Uniform      mean={mean(u_gens):.1f}  median={median(u_gens):.1f}")
        p(f"    PeakTargeted mean={mean(pt_gens):.1f}  median={median(pt_gens):.1f}")
        p()
        p("  ms/generation (mean, both-valid only):")
        u_mspg  = [rows_u[n]["ms_per_gen"]  for n in both_valid]
        pt_mspg = [rows_pt[n]["ms_per_gen"] for n in both_valid]
        p(f"    Uniform      mean={mean(u_mspg):.0f}ms")
        p(f"    PeakTargeted mean={mean(pt_mspg):.0f}ms")
        p()
        p("  Max stagnation (mean):")
        u_stag  = [rows_u[n]["max_stagnation"]  for n in common_names]
        pt_stag = [rows_pt[n]["max_stagnation"] for n in common_names]
        p(f"    Uniform      mean={mean(u_stag):.1f}")
        p(f"    PeakTargeted mean={mean(pt_stag):.1f}")
        p()
        p("─" * 72)
        p("FIVE KEY INSTANCES (per reviewer)")
        p("─" * 72)
        key_instances = ["setA-07", "setA-10", "setA-12", "setA-15", "setA-18"]
        for name in key_instances:
            if name not in common_names:
                p(f"  {name}: not in common set")
                continue
            u = rows_u[name]
            pt = rows_pt[name]
            p(f"  {name}  nodes={u['num_nodes']} links={u['num_links']} demands={u['num_demands']}")
            u_obj_str  = f"{u['final_obj']:.4f}"  if u['final_obj']  is not None else "inf"
            pt_obj_str = f"{pt['final_obj']:.4f}" if pt['final_obj'] is not None else "inf"
            p(f"    U:  valid={u['valid']}  obj={u_obj_str}  gens={u['n_generations']}  ms/gen={u['ms_per_gen']:.0f}")
            p(f"    PT: valid={pt['valid']}  obj={pt_obj_str}  gens={pt['n_generations']}  ms/gen={pt['ms_per_gen']:.0f}")
            if u["valid"] and pt["valid"]:
                d = pt["final_obj"] - u["final_obj"]
                p(f"    Δobj={d:+.4f}  peak_aps U={u['peak_aps']:.4f} PT={pt['peak_aps']:.4f}")
                p(f"    mut_peak_abs U={u['mut_peak_abs']} PT={pt['mut_peak_abs']}")
            elif not u["valid"] and pt["valid"]:
                p(f"    ** PT rescued this instance (U invalid, PT valid) **")
            elif u["valid"] and not pt["valid"]:
                p(f"    ** PT regressed this instance (U valid, PT invalid) **")
            else:
                p(f"    Both invalid")
            p()
        p("=" * 72)
        p("END OF SUMMARY")
        p("=" * 72)

    print(f"Written: {summary_path}")
    print(f"\nAll outputs written to: {out_dir}")

if __name__ == "__main__":
    main()