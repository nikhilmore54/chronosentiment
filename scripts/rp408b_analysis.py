#!/usr/bin/env python3
"""
RP-408B Analysis Script
=======================
Paired A/B comparison: Scalar vs. Lexicographic comparator.

Inputs:
  /tmp/rp408b/scalar/results.json
  /tmp/rp408b/lexicographic/results.json
  /tmp/rp408b/scalar/rp408b_generations_<inst>.jsonl
  /tmp/rp408b/lexicographic/rp408b_generations_<inst>.jsonl

Outputs (docs/roadef/rp408b_data/):
  summary_table.csv          — per-instance paired results
  aggregate_stats.json       — Win/Loss/Tie counts, mean delta obj
  survival_funnel_scalar.csv
  survival_funnel_lex.csv
  pe_decomposition.csv       — Peak/Shoulder/Transition PE per arm
  osr_delta.csv              — per-instance OSR delta (Lex - Scalar)
  rp408b_analysis.log        — narrative log

Analysis levels:
  Level 1 — Outcome: Win/Loss/Tie on best_obj per instance
  Level 2 — Mechanism: Peak PE, Shoulder PE, Transition PE per arm
  Level 3 — Safety: valid_rate, IFR, stagnation profile
"""

import json
import os
import sys
import csv
import math
from collections import defaultdict

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
BASE_DIR   = "/tmp/rp408b"
SCALAR_DIR = os.path.join(BASE_DIR, "scalar")
LEX_DIR    = os.path.join(BASE_DIR, "lexicographic")
OUT_DIR    = "docs/roadef/rp408b_data"
os.makedirs(OUT_DIR, exist_ok=True)

LOG_PATH = os.path.join(OUT_DIR, "rp408b_analysis.log")
log_lines = []

def log(msg=""):
    print(msg)
    log_lines.append(msg)

def flush_log():
    with open(LOG_PATH, "w") as f:
        f.write("\n".join(log_lines) + "\n")

# ---------------------------------------------------------------------------
# Load results.json
# ---------------------------------------------------------------------------
def load_results(arm_dir):
    path = os.path.join(arm_dir, "results.json")
    if not os.path.exists(path):
        log(f"  WARNING: {path} not found")
        return []
    with open(path) as f:
        data = json.load(f)
    # results.json is a dict with a "results" list
    if isinstance(data, dict):
        return data.get("results", [])
    return data

# ---------------------------------------------------------------------------
# Load generation JSONL for one instance
# ---------------------------------------------------------------------------
def load_generations(arm_dir, inst_name):
    path = os.path.join(arm_dir, f"rp408b_generations_{inst_name}.jsonl")
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
# Survival funnel from generation records
# ---------------------------------------------------------------------------
def compute_survival_funnel(gen_records):
    """
    Returns dict with:
      total_candidates, valid_count_gen0, peak_pe, shoulder_pe,
      transition_pe, tail_pe, mixed_pe, neutral_pe,
      total_moves, peak_osr, shoulder_osr, transition_osr
    """
    if not gen_records:
        return {}

    gen0 = [r for r in gen_records if r.get("record_type") == "generation" and r.get("generation") == 0]
    all_gens = [r for r in gen_records if r.get("record_type") == "generation"]
    construction = [r for r in gen_records if r.get("record_type") == "construction"]

    ifr = construction[0].get("initial_feasibility_rate", 0.0) if construction else 0.0
    pop_size = all_gens[0].get("population_size", 0) if all_gens else 0

    # Aggregate move zone counts across all generations
    total_peak       = sum(r.get("moves_peak", 0)       for r in all_gens)
    total_shoulder   = sum(r.get("moves_shoulder", 0)   for r in all_gens)
    total_transition = sum(r.get("moves_transition", 0) for r in all_gens)
    total_tail       = sum(r.get("moves_tail", 0)       for r in all_gens)
    total_mixed      = sum(r.get("moves_mixed", 0)      for r in all_gens)
    total_neutral    = sum(r.get("moves_neutral", 0)    for r in all_gens)
    total_moves      = total_peak + total_shoulder + total_transition + total_tail + total_mixed + total_neutral

    # PE = proportion of improvement moves in that zone
    def pe(zone_count):
        return zone_count / total_moves if total_moves > 0 else 0.0

    # OSR = zone_moves / total_generations (rate of zone improvement per generation)
    n_gens = len(all_gens)
    def osr(zone_count):
        return zone_count / n_gens if n_gens > 0 else 0.0

    # Valid rate at final generation
    last_gen = all_gens[-1] if all_gens else {}
    final_valid_rate = (last_gen.get("valid_count", 0) / pop_size) if pop_size > 0 else 0.0

    # Stagnation: fraction of generations with stagnation > 0
    stagnation_gens = sum(1 for r in all_gens if r.get("stagnation", 0) > 0)
    stagnation_rate = stagnation_gens / n_gens if n_gens > 0 else 0.0

    return {
        "n_generations":    n_gens,
        "ifr":              ifr,
        "final_valid_rate": final_valid_rate,
        "stagnation_rate":  stagnation_rate,
        "total_moves":      total_moves,
        "peak_moves":       total_peak,
        "shoulder_moves":   total_shoulder,
        "transition_moves": total_transition,
        "tail_moves":       total_tail,
        "mixed_moves":      total_mixed,
        "neutral_moves":    total_neutral,
        "peak_pe":          pe(total_peak),
        "shoulder_pe":      pe(total_shoulder),
        "transition_pe":    pe(total_transition),
        "tail_pe":          pe(total_tail),
        "mixed_pe":         pe(total_mixed),
        "neutral_pe":       pe(total_neutral),
        "peak_osr":         osr(total_peak),
        "shoulder_osr":     osr(total_shoulder),
        "transition_osr":   osr(total_transition),
    }

# ---------------------------------------------------------------------------
# Main analysis
# ---------------------------------------------------------------------------
def main():
    log("=" * 70)
    log("RP-408B Analysis: Scalar vs. Lexicographic Comparator A/B Benchmark")
    log("=" * 70)
    log()

    scalar_results = load_results(SCALAR_DIR)
    lex_results    = load_results(LEX_DIR)

    if not scalar_results:
        log("ERROR: No scalar results found. Check /tmp/rp408b/scalar/results.json")
        flush_log()
        sys.exit(1)
    if not lex_results:
        log("ERROR: No lexicographic results found. Check /tmp/rp408b/lexicographic/results.json")
        flush_log()
        sys.exit(1)

    log(f"Scalar arm:        {len(scalar_results)} instances")
    log(f"Lexicographic arm: {len(lex_results)} instances")
    log()

    # Index by instance name
    scalar_by_name = {r["name"]: r for r in scalar_results}
    lex_by_name    = {r["name"]: r for r in lex_results}

    all_names = sorted(set(scalar_by_name) | set(lex_by_name))
    log(f"Instances in union: {len(all_names)}")
    log()

    # ---------------------------------------------------------------------------
    # Level 1: Outcome — Win/Loss/Tie per instance
    # ---------------------------------------------------------------------------
    log("─" * 70)
    log("LEVEL 1 — OUTCOME: Win/Loss/Tie on best_obj (lower = better)")
    log("─" * 70)
    log(f"{'Instance':<20} {'Scalar obj':>12} {'Lex obj':>12} {'Delta':>10} {'Winner':>12}")
    log(f"{'─'*20} {'─'*12} {'─'*12} {'─'*10} {'─'*12}")

    wins_lex = 0
    wins_scalar = 0
    ties = 0
    invalid_both = 0
    summary_rows = []

    for name in all_names:
        sr = scalar_by_name.get(name)
        lr = lex_by_name.get(name)
        if sr is None or lr is None:
            log(f"  {name}: missing in one arm — skipped")
            continue

        s_obj   = sr["best_obj"] if sr["best_obj"] is not None else float("inf")
        l_obj   = lr["best_obj"] if lr["best_obj"] is not None else float("inf")
        s_valid = sr["valid"]
        l_valid = lr["valid"]

        # Determine winner
        if not s_valid and not l_valid:
            winner = "Tie(invalid)"
            invalid_both += 1
            ties += 1
            delta = 0.0
        elif s_valid and not l_valid:
            winner = "Scalar"
            wins_scalar += 1
            delta = float("inf")
        elif l_valid and not s_valid:
            winner = "Lex"
            wins_lex += 1
            delta = float("-inf")
        else:
            # Both valid — lower obj wins
            delta = l_obj - s_obj  # negative = Lex better
            if abs(delta) < 1e-6:
                winner = "Tie"
                ties += 1
            elif l_obj < s_obj:
                winner = "Lex"
                wins_lex += 1
            else:
                winner = "Scalar"
                wins_scalar += 1

        delta_str = f"{delta:+.4f}" if math.isfinite(delta) else ("−∞" if delta == float("-inf") else "+∞")
        log(f"  {name:<18} {s_obj:>12.4f} {l_obj:>12.4f} {delta_str:>10} {winner:>12}")

        summary_rows.append({
            "instance":       name,
            "scalar_obj":     s_obj,
            "lex_obj":        l_obj,
            "delta_obj":      delta if math.isfinite(delta) else None,
            "scalar_valid":   s_valid,
            "lex_valid":      l_valid,
            "winner":         winner,
            "scalar_runtime": sr.get("runtime_ms", 0),
            "lex_runtime":    lr.get("runtime_ms", 0),
            "scalar_gens":    sr.get("generations", sr.get("generations_run", 0)),
            "lex_gens":       lr.get("generations", lr.get("generations_run", 0)),
        })

    log()
    log(f"  Lex wins:    {wins_lex}")
    log(f"  Scalar wins: {wins_scalar}")
    log(f"  Ties:        {ties}  (of which both-invalid: {invalid_both})")
    total_paired = wins_lex + wins_scalar + ties
    log(f"  Total paired: {total_paired}")
    if total_paired > 0:
        log(f"  Lex win rate: {wins_lex/total_paired*100:.1f}%")

    # Mean delta obj (valid-only pairs)
    valid_deltas = [r["delta_obj"] for r in summary_rows
                    if r["delta_obj"] is not None and r["scalar_valid"] and r["lex_valid"]]
    if valid_deltas:
        mean_delta = sum(valid_deltas) / len(valid_deltas)
        log(f"  Mean delta obj (Lex−Scalar, valid pairs): {mean_delta:+.4f}")
        log(f"  (negative = Lex better on average)")

    # ---------------------------------------------------------------------------
    # Level 2: Mechanism — PE decomposition per arm
    # ---------------------------------------------------------------------------
    log()
    log("─" * 70)
    log("LEVEL 2 — MECHANISM: PE Decomposition (Scalar vs. Lexicographic)")
    log("─" * 70)

    scalar_funnels = {}
    lex_funnels    = {}

    for name in all_names:
        if name not in scalar_by_name or name not in lex_by_name:
            continue
        sg = load_generations(SCALAR_DIR, name)
        lg = load_generations(LEX_DIR,    name)
        scalar_funnels[name] = compute_survival_funnel(sg)
        lex_funnels[name]    = compute_survival_funnel(lg)

    # Aggregate PE across all instances
    def mean_field(funnels, field):
        vals = [f[field] for f in funnels.values() if field in f and f[field] is not None]
        return sum(vals) / len(vals) if vals else 0.0

    pe_fields = ["peak_pe", "shoulder_pe", "transition_pe", "tail_pe", "mixed_pe", "neutral_pe"]
    osr_fields = ["peak_osr", "shoulder_osr", "transition_osr"]

    log(f"  {'Metric':<22} {'Scalar':>10} {'Lex':>10} {'Delta':>10}")
    log(f"  {'─'*22} {'─'*10} {'─'*10} {'─'*10}")
    for field in pe_fields + osr_fields:
        s_val = mean_field(scalar_funnels, field)
        l_val = mean_field(lex_funnels,    field)
        delta = l_val - s_val
        log(f"  {field:<22} {s_val:>10.4f} {l_val:>10.4f} {delta:>+10.4f}")

    log()
    log("  IFR and valid rate:")
    for field in ["ifr", "final_valid_rate", "stagnation_rate"]:
        s_val = mean_field(scalar_funnels, field)
        l_val = mean_field(lex_funnels,    field)
        delta = l_val - s_val
        log(f"  {field:<22} {s_val:>10.4f} {l_val:>10.4f} {delta:>+10.4f}")

    # ---------------------------------------------------------------------------
    # Level 3: Safety — valid rate, IFR, stagnation
    # ---------------------------------------------------------------------------
    log()
    log("─" * 70)
    log("LEVEL 3 — SAFETY: Per-instance valid rate and stagnation")
    log("─" * 70)
    log(f"  {'Instance':<20} {'S_IFR':>8} {'L_IFR':>8} {'S_valid%':>10} {'L_valid%':>10} {'S_stag%':>9} {'L_stag%':>9}")
    log(f"  {'─'*20} {'─'*8} {'─'*8} {'─'*10} {'─'*10} {'─'*9} {'─'*9}")
    for name in all_names:
        sf = scalar_funnels.get(name, {})
        lf = lex_funnels.get(name, {})
        log(f"  {name:<20} "
            f"{sf.get('ifr',0):>8.3f} {lf.get('ifr',0):>8.3f} "
            f"{sf.get('final_valid_rate',0)*100:>9.1f}% {lf.get('final_valid_rate',0)*100:>9.1f}% "
            f"{sf.get('stagnation_rate',0)*100:>8.1f}% {lf.get('stagnation_rate',0)*100:>8.1f}%")

    # ---------------------------------------------------------------------------
    # Write CSV outputs
    # ---------------------------------------------------------------------------
    # summary_table.csv
    summary_path = os.path.join(OUT_DIR, "summary_table.csv")
    with open(summary_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(summary_rows[0].keys()) if summary_rows else [])
        writer.writeheader()
        writer.writerows(summary_rows)
    log()
    log(f"  Written: {summary_path}")

    # aggregate_stats.json
    agg = {
        "lex_wins":    wins_lex,
        "scalar_wins": wins_scalar,
        "ties":        ties,
        "invalid_both": invalid_both,
        "total_paired": total_paired,
        "lex_win_rate": wins_lex / total_paired if total_paired > 0 else 0.0,
        "mean_delta_obj_valid_pairs": mean_delta if valid_deltas else None,
        "scalar_mean_peak_pe":       mean_field(scalar_funnels, "peak_pe"),
        "lex_mean_peak_pe":          mean_field(lex_funnels,    "peak_pe"),
        "scalar_mean_shoulder_pe":   mean_field(scalar_funnels, "shoulder_pe"),
        "lex_mean_shoulder_pe":      mean_field(lex_funnels,    "shoulder_pe"),
        "scalar_mean_peak_osr":      mean_field(scalar_funnels, "peak_osr"),
        "lex_mean_peak_osr":         mean_field(lex_funnels,    "peak_osr"),
        "scalar_mean_ifr":           mean_field(scalar_funnels, "ifr"),
        "lex_mean_ifr":              mean_field(lex_funnels,    "ifr"),
    }
    agg_path = os.path.join(OUT_DIR, "aggregate_stats.json")
    with open(agg_path, "w") as f:
        json.dump(agg, f, indent=2)
    log(f"  Written: {agg_path}")

    # pe_decomposition.csv
    pe_rows = []
    for name in all_names:
        sf = scalar_funnels.get(name, {})
        lf = lex_funnels.get(name, {})
        row = {"instance": name}
        for field in pe_fields + osr_fields + ["ifr", "final_valid_rate", "stagnation_rate", "n_generations", "total_moves"]:
            row[f"scalar_{field}"] = sf.get(field, None)
            row[f"lex_{field}"]    = lf.get(field, None)
        pe_rows.append(row)
    pe_path = os.path.join(OUT_DIR, "pe_decomposition.csv")
    if pe_rows:
        with open(pe_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(pe_rows[0].keys()))
            writer.writeheader()
            writer.writerows(pe_rows)
    log(f"  Written: {pe_path}")

    # osr_delta.csv
    osr_rows = []
    for name in all_names:
        sf = scalar_funnels.get(name, {})
        lf = lex_funnels.get(name, {})
        osr_rows.append({
            "instance":          name,
            "scalar_peak_osr":   sf.get("peak_osr", None),
            "lex_peak_osr":      lf.get("peak_osr", None),
            "delta_peak_osr":    (lf.get("peak_osr", 0) - sf.get("peak_osr", 0))
                                  if sf.get("peak_osr") is not None and lf.get("peak_osr") is not None else None,
            "scalar_shoulder_osr": sf.get("shoulder_osr", None),
            "lex_shoulder_osr":    lf.get("shoulder_osr", None),
            "delta_shoulder_osr":  (lf.get("shoulder_osr", 0) - sf.get("shoulder_osr", 0))
                                    if sf.get("shoulder_osr") is not None and lf.get("shoulder_osr") is not None else None,
        })
    osr_path = os.path.join(OUT_DIR, "osr_delta.csv")
    if osr_rows:
        with open(osr_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(osr_rows[0].keys()))
            writer.writeheader()
            writer.writerows(osr_rows)
    log(f"  Written: {osr_path}")

    log()
    log("=" * 70)
    log("RP-408B analysis complete.")
    log("=" * 70)
    flush_log()

if __name__ == "__main__":
    main()