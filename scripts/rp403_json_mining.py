#!/usr/bin/env python3
"""
RP-403 Phase 1A: JSON Mining Script
Mine existing solution JSONs for setA-08, setA-12, setA-17 diagnostic data.

Investigates:
- setA-12: Why did RP-401C produce finite (26.1166), RP-401D regress to inf, RP-402 remain inf?
- setA-17: Budget sensitivity -- why does budget=1 cause infeasibility?
- setA-08: Why did RP-401D (48.6693) regress to inf in RP-402?
"""

import json
import os
from collections import defaultdict

BASE = "adapters/roadef/repo/challenge-roadef-2026-main/setA"
INSTANCES = ["setA-08", "setA-12", "setA-17"]
SOLVERS = ["rp401c", "rp401d", "rp402"]


def load_srpaths(instance, solver):
    path = os.path.join(BASE, f"{instance}-srpaths-{solver}.json")
    raw = json.load(open(path))
    by_demand = defaultdict(dict)
    for entry in raw["srpaths"]:
        by_demand[entry["d"]][entry["t"]] = entry["w"]
    return by_demand


def edge_set(wp):
    if not wp or len(wp) < 2:
        return frozenset()
    return frozenset(zip(wp[:-1], wp[1:]))


def sym_diff(wp_a, wp_b):
    return len(edge_set(wp_a).symmetric_difference(edge_set(wp_b)))


def analyse_instance(inst):
    print(f"\n{'='*65}")
    print(f"INSTANCE: {inst}")
    print(f"{'='*65}")

    data = {s: load_srpaths(inst, s) for s in SOLVERS}
    demands = sorted(data["rp401c"].keys())
    n = len(demands)
    print(f"  Demands: {n}")

    for solver, by_d in data.items():
        t0_lens = [len(by_d[d].get(0, [])) for d in demands]
        t1_lens = [len(by_d[d].get(1, [])) for d in demands]
        n_shared = sum(1 for d in demands if by_d[d].get(0) == by_d[d].get(1))
        n_adapted = sum(
            1 for d in demands
            if 1 in by_d[d] and by_d[d].get(0) != by_d[d].get(1)
        )
        costs = [sym_diff(by_d[d].get(0), by_d[d].get(1)) for d in demands]
        total_cost = sum(costs)
        nonzero = sum(1 for c in costs if c > 0)
        print(
            f"  [{solver}] "
            f"t0 len: min={min(t0_lens)} max={max(t0_lens)} mean={sum(t0_lens)/n:.1f} | "
            f"t1 len: min={min(t1_lens)} max={max(t1_lens)} mean={sum(t1_lens)/n:.1f} | "
            f"shared={n_shared} adapted={n_adapted} | "
            f"total_transition_cost={total_cost} nonzero_demands={nonzero}"
        )

    # Cross-solver t0 divergence
    print("  --- t0 divergence ---")
    for i, sa in enumerate(SOLVERS):
        for sb in SOLVERS[i + 1:]:
            diff = sum(1 for d in demands if data[sa][d].get(0) != data[sb][d].get(0))
            print(f"    {sa} vs {sb}: {diff}/{n} demands differ ({100*diff/n:.1f}%)")

    # Cross-solver t1 divergence
    print("  --- t1 divergence ---")
    for i, sa in enumerate(SOLVERS):
        for sb in SOLVERS[i + 1:]:
            diff = sum(1 for d in demands if data[sa][d].get(1) != data[sb][d].get(1))
            print(f"    {sa} vs {sb}: {diff}/{n} demands differ ({100*diff/n:.1f}%)")

    # First t0 divergence between rp401c and rp402
    first_div = next(
        (d for d in demands if data["rp401c"][d].get(0) != data["rp402"][d].get(0)),
        None,
    )
    if first_div is not None:
        print(f"  --- First t0 divergence rp401c vs rp402: demand {first_div} ---")
        print(f"    rp401c t0: {data['rp401c'][first_div].get(0)}")
        print(f"    rp402  t0: {data['rp402'][first_div].get(0)}")
    else:
        print(f"  --- rp401c and rp402 have IDENTICAL t0 paths for all {n} demands ---")

    # setA-12 specific: adaptation analysis
    if inst == "setA-12":
        print("  --- setA-12: adaptation analysis per solver ---")
        for solver in SOLVERS:
            by_d = data[solver]
            adapted = [
                (d, by_d[d].get(0), by_d[d].get(1))
                for d in demands
                if 1 in by_d[d] and by_d[d].get(0) != by_d[d].get(1)
            ]
            print(f"    [{solver}] adapted demands: {len(adapted)}/{n}")
            for d, t0, t1 in adapted[:3]:
                print(f"      demand {d}: t0={t0} -> t1={t1} cost={sym_diff(t0, t1)}")

    # setA-08 specific: rp401d vs rp402 t1 regression
    if inst == "setA-08":
        print("  --- setA-08: rp401d vs rp402 t1 differences ---")
        diff_t1 = [
            (d, data["rp401d"][d].get(1), data["rp402"][d].get(1))
            for d in demands
            if data["rp401d"][d].get(1) != data["rp402"][d].get(1)
        ]
        print(f"    t1 differences: {len(diff_t1)}/{n}")
        for d, t1d, t1_402 in diff_t1[:5]:
            c_d = sym_diff(data["rp401d"][d].get(0), t1d)
            c_402 = sym_diff(data["rp402"][d].get(0), t1_402)
            print(
                f"      demand {d}: rp401d t1={t1d} (cost={c_d}) | "
                f"rp402 t1={t1_402} (cost={c_402})"
            )
        cost_d = sum(
            sym_diff(data["rp401d"][d].get(0), data["rp401d"][d].get(1))
            for d in demands
        )
        cost_402 = sum(
            sym_diff(data["rp402"][d].get(0), data["rp402"][d].get(1))
            for d in demands
        )
        print(f"    rp401d total transition cost: {cost_d}")
        print(f"    rp402  total transition cost: {cost_402}")

        # Also compare t0 paths between rp401d and rp402 for setA-08
        diff_t0 = sum(
            1 for d in demands if data["rp401d"][d].get(0) != data["rp402"][d].get(0)
        )
        print(f"    rp401d vs rp402 t0 differences: {diff_t0}/{n}")

    # setA-17 specific: all solvers give inf -- check if any adaptation attempted
    if inst == "setA-17":
        print("  --- setA-17: adaptation attempted? ---")
        for solver in SOLVERS:
            by_d = data[solver]
            adapted = sum(
                1 for d in demands
                if 1 in by_d[d] and by_d[d].get(0) != by_d[d].get(1)
            )
            total_cost = sum(
                sym_diff(by_d[d].get(0), by_d[d].get(1)) for d in demands
            )
            print(f"    [{solver}] adapted={adapted}/{n} total_cost={total_cost}")


def main():
    print("RP-403 Phase 1A -- JSON Evidence Mining")
    print(f"Base: {BASE}")
    for inst in INSTANCES:
        analyse_instance(inst)
    print(f"\n{'='*65}")
    print("Mining complete.")


if __name__ == "__main__":
    main()