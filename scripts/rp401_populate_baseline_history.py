#!/usr/bin/env python3
"""
rp401_populate_baseline_history.py

Reads RP-401C and RP-401D binary output files and prints:
  1. Per-instance markdown tables for BASELINE_HISTORY.md
  2. Summary statistics table (instances improved, feasibility transitions,
     mean/median objective, best improvement, runtime, oracle calls)

Usage:
    python3 scripts/rp401_populate_baseline_history.py \\
        /tmp/rp401c_output.txt /tmp/rp401d_output.txt

To capture binary output:
    cargo run --manifest-path adapters/roadef/Cargo.toml \\
        --bin rp401c_ecmp_construction --release 2>/dev/null | tee /tmp/rp401c_output.txt
    cargo run --manifest-path adapters/roadef/Cargo.toml \\
        --bin rp401d_ecmp_path_selection --release 2>/dev/null | tee /tmp/rp401d_output.txt
"""

import sys
import re
import os
import statistics

SET_DIR = "adapters/roadef/repo/challenge-roadef-2026-main/setA"


def parse_binary_output(filepath):
    """Parse the tabular output from rp401c or rp401d binary.

    Handles three formats:
      RP-401C (5 cols): setA-01  53.3172  inf  improved  1234
      RP-401D (6 cols): setA-01  53.0880  see rp401c  inf  n/a  93
        — col3 is RP-401C reference (skipped), col4=empty_obj, col5=delta, col6=ms
      Old (4 cols):     setA-01  53.3172  inf  improved
    """
    results = {}
    if not os.path.exists(filepath):
        return results
    with open(filepath) as f:
        for line in f:
            line = line.rstrip()
            # RP-401D 6-column format: instance  d_obj  rp401c_obj  empty_obj  delta  ms
            # Detected by presence of "see rp401c" in col3
            m6 = re.match(
                r'\s*(setA-\d+)\s+(\S+)\s+see rp401c\s+(\S+)\s+(\S+)\s+(\d+)', line
            )
            if m6:
                inst = m6.group(1)
                our_obj = m6.group(2)
                empty_obj = m6.group(3)
                delta_raw = m6.group(4)
                ms = m6.group(5)
                # Normalise delta: n/a means inf→inf or inf→finite
                if delta_raw == 'n/a':
                    # Determine from our_obj and empty_obj
                    if our_obj == 'inf' and empty_obj == 'inf':
                        delta = 'both inf'
                    elif our_obj != 'inf' and empty_obj == 'inf':
                        delta = 'improved'
                    else:
                        delta = delta_raw
                else:
                    delta = delta_raw
                results[inst] = {
                    'our_obj': our_obj,
                    'empty_obj': empty_obj,
                    'delta': delta,
                    'ms': ms,
                }
                continue

            # RP-401C / old format: instance  our_obj  empty_obj  delta  [ms]
            # delta may be two words: "both inf"
            m = re.match(
                r'\s*(setA-\d+)\s+(\S+)\s+(\S+)\s+(both inf|improved|=|\S+)(?:\s+(\d+))?', line
            )
            if m:
                inst = m.group(1)
                our_obj = m.group(2)
                empty_obj = m.group(3)
                delta = m.group(4)
                ms = m.group(5)  # may be None for old format
                results[inst] = {
                    'our_obj': our_obj,
                    'empty_obj': empty_obj,
                    'delta': delta,
                    'ms': ms,
                }
    return results


def parse_total_runtime(filepath):
    """Extract total runtime from summary line if present."""
    if not os.path.exists(filepath):
        return None
    with open(filepath) as f:
        for line in f:
            # Look for lines like "Total runtime: 1234 ms" or similar
            m = re.search(r'[Tt]otal.*?(\d+)\s*(?:ms|s)', line)
            if m:
                return m.group(1)
    return None


def count_srpaths(inst, suffix):
    """Count number of srpaths in a solution file."""
    path = f"{SET_DIR}/{inst}-srpaths-{suffix}.json"
    if not os.path.exists(path):
        return "—"
    try:
        import json
        with open(path) as f:
            data = json.load(f)
        return str(len(data.get('srpaths', [])))
    except Exception:
        return "?"


def compute_summary(results, label):
    """Compute summary statistics for a solver run."""
    improved = 0          # our obj < empty obj (finite improvement)
    inf_to_finite = 0     # baseline was inf, ours is finite
    both_inf = 0          # both inf
    unchanged = 0         # equal finite or equal inf
    finite_objs = []      # our finite objective values
    finite_deltas = []    # delta values where both are finite
    best_inst = None
    best_delta = 0.0
    total_ms = 0
    ms_count = 0

    for i in range(1, 21):
        inst = f"setA-{i:02d}"
        r = results.get(inst, {})
        our = r.get('our_obj', 'pending')
        empty = r.get('empty_obj', 'pending')
        delta_str = r.get('delta', 'pending')
        ms = r.get('ms')

        if ms:
            try:
                total_ms += int(ms)
                ms_count += 1
            except ValueError:
                pass

        if our == 'pending' or empty == 'pending':
            continue

        our_finite = our not in ('inf', '—')
        empty_finite = empty not in ('inf', '—')

        if our_finite:
            try:
                our_val = float(our)
                finite_objs.append(our_val)
            except ValueError:
                pass

        if our_finite and empty_finite:
            try:
                our_val = float(our)
                empty_val = float(empty)
                delta = our_val - empty_val
                finite_deltas.append(delta)
                if delta < -0.001:
                    improved += 1
                    if delta < best_delta:
                        best_delta = delta
                        best_inst = inst
                elif delta > 0.001:
                    pass  # regression
                else:
                    unchanged += 1
            except ValueError:
                pass
        elif our_finite and not empty_finite:
            # inf → finite
            inf_to_finite += 1
            improved += 1
        elif not our_finite and not empty_finite:
            both_inf += 1
        elif not our_finite and empty_finite:
            pass  # regression (we made it worse)

    mean_obj = statistics.mean(finite_objs) if finite_objs else None
    median_obj = statistics.median(finite_objs) if finite_objs else None

    return {
        'improved': improved,
        'inf_to_finite': inf_to_finite,
        'both_inf': both_inf,
        'unchanged': unchanged,
        'finite_count': len(finite_objs),
        'mean_obj': mean_obj,
        'median_obj': median_obj,
        'best_inst': best_inst,
        'best_delta': best_delta,
        'total_ms': total_ms if ms_count > 0 else None,
        'ms_count': ms_count,
    }


def print_instance_table(results, label, suffix):
    print(f"## {label} — Per-Instance Results")
    print()
    print("| Instance | Our obj | Empty obj | vs Empty | Finite | ms | srpaths |")
    print("|----------|---------|-----------|----------|--------|----|---------|")
    for i in range(1, 21):
        inst = f"setA-{i:02d}"
        r = results.get(inst, {})
        our = r.get('our_obj', 'pending')
        empty = r.get('empty_obj', 'pending')
        delta = r.get('delta', 'pending')
        ms = r.get('ms', '—') or '—'
        our_finite = our not in ('inf', 'pending', '—')
        if our_finite:
            finite_mark = "✓"
        elif delta in ('both inf', 'improved', '='):
            finite_mark = "→ empty"
        elif delta == 'pending':
            finite_mark = "pending"
        else:
            finite_mark = "→ empty"
        srpaths = count_srpaths(inst, suffix)
        print(f"| {inst} | {our} | {empty} | {delta} | {finite_mark} | {ms} | {srpaths} |")
    print()


def print_summary_table(rp401c_summary, rp401d_summary):
    print("## Summary Statistics")
    print()
    print("| Metric | Baseline v1.0 | RP-401C | RP-401D |")
    print("|--------|---------------|---------|---------|")

    def fmt(v, fmt_str="{:.2f}"):
        return fmt_str.format(v) if v is not None else "pending"

    c = rp401c_summary
    d = rp401d_summary

    print(f"| Instances improved / 20 | 3 | {c['improved']} | {d['improved']} |")
    print(f"| Previously ∞ → finite | 0 | {c['inf_to_finite']} | {d['inf_to_finite']} |")
    print(f"| Both still ∞ | 17 | {c['both_inf']} | {d['both_inf']} |")
    print(f"| Finite instances (our sol) | 3 | {c['finite_count']} | {d['finite_count']} |")
    print(f"| Mean obj (finite instances) | ~244 | {fmt(c['mean_obj'])} | {fmt(d['mean_obj'])} |")
    print(f"| Median obj (finite instances) | ~159 | {fmt(c['median_obj'])} | {fmt(d['median_obj'])} |")

    best_c = f"{c['best_inst']}: {c['best_delta']:.2f}" if c['best_inst'] else "pending"
    best_d = f"{d['best_inst']}: {d['best_delta']:.2f}" if d['best_inst'] else "pending"
    print(f"| Best improvement (finite delta) | setA-16: −3,355,441 | {best_c} | {best_d} |")

    rt_c = f"{c['total_ms']} ms" if c['total_ms'] is not None else "pending"
    rt_d = f"{d['total_ms']} ms" if d['total_ms'] is not None else "pending"
    print(f"| Total runtime | < 1s | {rt_c} | {rt_d} |")
    print(f"| Oracle calls | 0 | Σ D² | Σ D×K (K=5) |")
    print()

    # Best improvement note for setA-13
    if c['best_inst']:
        print(f"> **Best improvement:** {c['best_inst']} reduced from baseline by {abs(c['best_delta']):.2f} objective units.")
        print()


def main():
    rp401c_file = sys.argv[1] if len(sys.argv) > 1 else "/tmp/rp401c_output.txt"
    rp401d_file = sys.argv[2] if len(sys.argv) > 2 else "/tmp/rp401d_output.txt"

    rp401c = parse_binary_output(rp401c_file)
    rp401d = parse_binary_output(rp401d_file)

    print_instance_table(rp401c, "RP-401C (Ground-Truth Construction)", "rp401c")
    print_instance_table(rp401d, "RP-401D (Efficiency Recovery)", "rp401d")

    c_summary = compute_summary(rp401c, "RP-401C")
    d_summary = compute_summary(rp401d, "RP-401D")
    print_summary_table(c_summary, d_summary)


if __name__ == "__main__":
    main()