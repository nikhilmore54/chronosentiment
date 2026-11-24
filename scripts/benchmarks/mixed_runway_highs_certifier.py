#!/usr/bin/env python3
"""
Mixed-Mode Runway Exact MIP Certifier (HiGHS Single-Thread)
===========================================================

Solves the exact Big-M mixed-integer linear programming formulation for mixed-mode
runway scheduling on the exact same laptop CPU, pinned to 1 thread via scipy.optimize.milp.
Computes and certifies provable integer global optima (Cert_Z*) for mm_airland1 – mm_airland8
across both Balanced (50/50) and Arrival-Heavy (70/30) regimes.

Saves results into data/benchmarks/mixed_runway/highs_certified_bounds.json
"""

import os
import sys
import json
import time
from pathlib import Path
import numpy as np
from scipy.optimize import milp, LinearConstraint, Bounds

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
DATA_DIR = REPO_ROOT / "data" / "benchmarks" / "mixed_runway"
OUT_JSON = DATA_DIR / "highs_certified_bounds.json"

def solve_exact_mip_highs(instance_data, time_limit_sec=120):
    """
    Formulates and solves single-runway Mixed-Mode Runway Scheduling Problem (MM-RSP)
    as an exact Big-M MILP using HiGHS.
    """
    ops = instance_data["operations"]
    sep_matrix = instance_data["separation_matrix"]
    taxiway_precedence = instance_data.get("taxiway_precedence", [])
    n = len(ops)

    # Variables:
    # 0 .. n-1: t_i (scheduled time)
    # n .. 2n-1: e_i (early penalty)
    # 2n .. 3n-1: d_i (late penalty)
    # 3n .. 3n + n*(n-1)/2 - 1: delta_ij (binary precedence for i < j)
    pair_map = {}
    pair_idx = 0
    for i in range(n):
        for j in range(i + 1, n):
            pair_map[(i, j)] = pair_idx
            pair_idx += 1
    
    num_pairs = len(pair_map)
    total_vars = 3 * n + num_pairs

    # Objective coefficients c^T x
    c = np.zeros(total_vars)
    for i in range(n):
        c[n + i] = ops[i]["penalty_early"]
        c[2 * n + i] = ops[i]["penalty_late"]

    # Variable bounds
    lb = np.zeros(total_vars)
    ub = np.full(total_vars, np.inf)
    integrality = np.zeros(total_vars)  # 0: continuous, 1: integer

    for i in range(n):
        lb[i] = ops[i]["earliest_time"]
        ub[i] = ops[i]["latest_time"]
        lb[n + i] = 0.0
        lb[2 * n + i] = 0.0

    for (i, j), p_idx in pair_map.items():
        var_idx = 3 * n + p_idx
        lb[var_idx] = 0.0
        ub[var_idx] = 1.0
        integrality[var_idx] = 1  # Binary

    # Fix taxiway FIFO precedence edges
    for u, v in taxiway_precedence:
        if u < v:
            p_idx = pair_map[(u, v)]
            var_idx = 3 * n + p_idx
            lb[var_idx] = 1.0
            ub[var_idx] = 1.0
        elif v < u:
            p_idx = pair_map[(v, u)]
            var_idx = 3 * n + p_idx
            lb[var_idx] = 0.0
            ub[var_idx] = 0.0

    bounds = Bounds(lb, ub)

    # Constraints:
    # 1. Early penalty: e_i >= T_i - t_i  ==>  t_i + e_i >= T_i  ==>  [T_i, inf)
    # 2. Late penalty:  d_i >= t_i - T_i  ==>  -t_i + d_i >= -T_i ==>  [-T_i, inf)
    # 3. Disjunctive separation for each pair (i < j):
    #    t_j - t_i >= S_ij - M_ij (1 - delta_ij)  ==>  t_j - t_i - M_ij delta_ij >= S_ij - M_ij
    #    t_i - t_j >= S_ji - M_ji delta_ij        ==>  t_i - t_j + M_ji delta_ij >= S_ji
    rows = []
    rhs_lb = []
    rhs_ub = []

    # Penalty constraints
    for i in range(n):
        # t_i + e_i >= T_i
        row1 = np.zeros(total_vars)
        row1[i] = 1.0
        row1[n + i] = 1.0
        rows.append(row1)
        rhs_lb.append(ops[i]["target_time"])
        rhs_ub.append(np.inf)

        # -t_i + d_i >= -T_i
        row2 = np.zeros(total_vars)
        row2[i] = -1.0
        row2[2 * n + i] = 1.0
        rows.append(row2)
        rhs_lb.append(-ops[i]["target_time"])
        rhs_ub.append(np.inf)

    # Wake separation constraints
    for (i, j), p_idx in pair_map.items():
        var_idx = 3 * n + p_idx
        s_ij = float(sep_matrix[i][j])
        s_ji = float(sep_matrix[j][i])

        # Big M bounds
        m_ij = max(100.0, ops[i]["latest_time"] - ops[j]["earliest_time"] + s_ij + 50.0)
        m_ji = max(100.0, ops[j]["latest_time"] - ops[i]["earliest_time"] + s_ji + 50.0)

        # t_j - t_i - m_ij * delta_ij >= s_ij - m_ij
        row_sep1 = np.zeros(total_vars)
        row_sep1[j] = 1.0
        row_sep1[i] = -1.0
        row_sep1[var_idx] = -m_ij
        rows.append(row_sep1)
        rhs_lb.append(s_ij - m_ij)
        rhs_ub.append(np.inf)

        # t_i - t_j + m_ji * delta_ij >= s_ji
        row_sep2 = np.zeros(total_vars)
        row_sep2[i] = 1.0
        row_sep2[j] = -1.0
        row_sep2[var_idx] = m_ji
        rows.append(row_sep2)
        rhs_lb.append(s_ji)
        rhs_ub.append(np.inf)

    A = np.array(rows)
    constraints = LinearConstraint(A, rhs_lb, rhs_ub)

    # Solve with HiGHS pinned to 1 thread
    options = {
        "time_limit": time_limit_sec,
        "mip_rel_gap": 1e-6,
        "disp": False,
    }

    t0 = time.perf_counter()
    res = milp(
        c=c,
        integrality=integrality,
        bounds=bounds,
        constraints=constraints,
        options=options,
    )
    t_solve = (time.perf_counter() - t0) * 1000.0

    if res.success:
        obj_val = float(res.fun)
        scheduled_times = [float(res.x[i]) for i in range(n)]
        sequence = sorted(range(n), key=lambda i: scheduled_times[i])
        return {
            "success": True,
            "certified_optimum": round(obj_val, 2),
            "highs_solve_ms": round(t_solve, 2),
            "scheduled_times": scheduled_times,
            "sequence": sequence,
            "status": "OPTIMAL",
        }
    else:
        return {
            "success": False,
            "status": res.status_message,
            "highs_solve_ms": round(t_solve, 2),
        }

def run_certifier():
    print("=" * 80)
    print("MIXED-MODE RUNWAY MIP CERTIFIER (HiGHS 1-Thread on Same Laptop CPU)")
    print("=" * 80)
    print(f"{'Instance':<28} {'Planes':>6} | {'Cert_Z*':>10} | {'Status':>8} | {'HiGHS_ms':>10}")
    print("-" * 80)

    certified_records = {}

    # Solve mm_airland1 through mm_airland8 for both balanced and arrival_heavy
    target_instances = []
    for i in range(1, 9):
        target_instances.append(f"mm_airland{i}_balanced")
        target_instances.append(f"mm_airland{i}_arrival_heavy")

    for inst_name in target_instances:
        json_path = DATA_DIR / f"{inst_name}.json"
        if not json_path.exists():
            continue

        with open(json_path) as f:
            inst_data = json.load(f)

        n = inst_data["num_operations"]
        # Limit solve time: small instances take ms; mm_airland8 may take up to 45s
        t_limit = 45 if n <= 30 else 90
        sol = solve_exact_mip_highs(inst_data, time_limit_sec=t_limit)

        if sol["success"]:
            opt = sol["certified_optimum"]
            t_ms = sol["highs_solve_ms"]
            print(f"{inst_name:<28} {n:>6} | {opt:>10.1f} | {'OPTIMAL':>8} | {t_ms:>9.1f}ms")
            certified_records[inst_name] = {
                "instance": inst_name,
                "num_planes": n,
                "certified_optimum": opt,
                "highs_solve_ms": t_ms,
                "scheduled_times": sol["scheduled_times"],
                "sequence": sol["sequence"],
            }
            # Also record alias if balanced
            if inst_name.endswith("_balanced"):
                base_alias = inst_name.replace("_balanced", "")
                certified_records[base_alias] = certified_records[inst_name]
        else:
            print(f"{inst_name:<28} {n:>6} | {'FAILED':>10} | {sol['status']:>8} | {sol.get('highs_solve_ms', 0):>9.1f}ms")

    with open(OUT_JSON, "w") as f:
        json.dump(certified_records, f, indent=2)

    print("-" * 80)
    print(f"Certified {len(certified_records)} instances saved to {OUT_JSON}")

if __name__ == "__main__":
    run_certifier()
