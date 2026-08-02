#!/usr/bin/env python3
"""
rp401_populate_baseline_history.py

Reads RP-401C and RP-401D solution files and evaluates them against the
empty solution to populate BASELINE_HISTORY.md with real numbers.

Usage:
    python3 scripts/rp401_populate_baseline_history.py

Requires: the rp401c and rp401d srpaths JSON files to exist in setA/.
Outputs: prints a markdown table row for each instance.

Note: This script reads the solution files but does NOT re-run the evaluator.
It relies on the objective values printed by the binaries during execution.
To get those values, capture the binary output:
    cargo run --bin rp401c_ecmp_construction --release > /tmp/rp401c_output.txt
    cargo run --bin rp401d_ecmp_path_selection --release > /tmp/rp401d_output.txt
Then run this script with those files as input.
"""

import sys
import re
import os

SET_DIR = "adapters/roadef/repo/challenge-roadef-2026-main/setA"

def parse_binary_output(filepath):
    """Parse the tabular output from rp401c or rp401d binary."""
    results = {}
    if not os.path.exists(filepath):
        return results
    with open(filepath) as f:
        for line in f:
            # Match lines like: setA-01     73.4619          inf   improved
            m = re.match(r'\s*(setA-\d+)\s+(\S+)\s+(\S+)\s+(\S+)', line)
            if m:
                inst = m.group(1)
                our_obj = m.group(2)
                empty_obj = m.group(3)
                delta = m.group(4)
                results[inst] = {
                    'our_obj': our_obj,
                    'empty_obj': empty_obj,
                    'delta': delta,
                }
    return results

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

def main():
    rp401c_file = sys.argv[1] if len(sys.argv) > 1 else "/tmp/rp401c_output.txt"
    rp401d_file = sys.argv[2] if len(sys.argv) > 2 else "/tmp/rp401d_output.txt"

    rp401c = parse_binary_output(rp401c_file)
    rp401d = parse_binary_output(rp401d_file)

    print("## RP-401C Results")
    print()
    print("| Instance | Our obj | Empty obj | vs Empty | Finite | srpaths |")
    print("|----------|---------|-----------|----------|--------|---------|")
    improved_c = 0
    for i in range(1, 21):
        inst = f"setA-{i:02d}"
        r = rp401c.get(inst, {})
        our = r.get('our_obj', 'pending')
        empty = r.get('empty_obj', 'pending')
        delta = r.get('delta', 'pending')
        finite = "✓" if our not in ('inf', 'pending', '—') else ("→ empty" if delta in ('=', 'improved') else "pending")
        srpaths = count_srpaths(inst, 'rp401c')
        if delta not in ('pending', '=', 'n/a') and delta.startswith('-'):
            improved_c += 1
        print(f"| {inst} | {our} | {empty} | {delta} | {finite} | {srpaths} |")
    print()
    print(f"Improved vs empty: {improved_c}/20")
    print()

    print("## RP-401D Results")
    print()
    print("| Instance | Our obj | Empty obj | vs Empty | Finite | srpaths |")
    print("|----------|---------|-----------|----------|--------|---------|")
    improved_d = 0
    for i in range(1, 21):
        inst = f"setA-{i:02d}"
        r = rp401d.get(inst, {})
        our = r.get('our_obj', 'pending')
        empty = r.get('empty_obj', 'pending')
        delta = r.get('delta', 'pending')
        finite = "✓" if our not in ('inf', 'pending', '—') else ("→ empty" if delta in ('=', 'improved') else "pending")
        srpaths = count_srpaths(inst, 'rp401d')
        if delta not in ('pending', '=', 'n/a') and delta.startswith('-'):
            improved_d += 1
        print(f"| {inst} | {our} | {empty} | {delta} | {finite} | {srpaths} |")
    print()
    print(f"Improved vs empty: {improved_d}/20")

if __name__ == "__main__":
    main()