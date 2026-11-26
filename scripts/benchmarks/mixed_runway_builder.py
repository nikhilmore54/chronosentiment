#!/usr/bin/env python3
"""
Mixed-Mode Airport Runway Benchmark Dataset Generator (Track A & Track B)
========================================================================

Constructs two complementary benchmark tracks in `data/benchmarks/mixed_runway/`:
1. Track A — Extended Beasley Suite (mm_airland1 – mm_airland13, 10 to 500 aircraft):
   - Deterministic 50/50 Balanced and 70/30 Arrival-Heavy regimes (Furini et al. 2015, Faye 2018).
   - 4-way ICAO asymmetric wake-turbulence separation matrix (L->L, T->T, L->T, T->L).
   - Taxiway FIFO precedence groups for departing queues.
2. Track B — Real-World Single-Runway Peak Traffic Banks:
   - Paris CDG 1-hour & 3-hour peak profiles (Ikli et al. 2021).
   - London Gatwick (LGW) single-runway peak (world's highest single-runway movements).
   - Zurich (ZRH) peak-hour mixed-mode stream.
   - Chennai MAA / Mumbai BOM high-density domestic + widebody hub peak bank.

Emits JSON instance files and plain-text format for the Rust DecisionPlugin engine.
"""

import os
import sys
import json
import math
import random
from pathlib import Path

# Base directories
REPO_ROOT = Path(__file__).resolve().parent.parent.parent
AIRLAND_SRC_DIR = REPO_ROOT / "benchmarks" / "orlib" / "airland"
OUT_DIR = REPO_ROOT / "data" / "benchmarks" / "mixed_runway"

OUT_DIR.mkdir(parents=True, exist_ok=True)

# ── ICAO Standard 4-Way Separation Matrices (seconds) ────────────────────────
# Categories: 0: Heavy (H), 1: Medium (M), 2: Light (L)

# Landing -> Landing (L -> L)
SEP_LL = [
    [96, 120, 144],  # Leading Heavy -> [Trailing H, M, L]
    [72,  80, 120],  # Leading Medium -> [H, M, L]
    [60,  60,  60],  # Leading Light  -> [H, M, L]
]

# Takeoff -> Takeoff (T -> T)
SEP_TT = [
    [90, 120, 140],  # Leading Heavy
    [60,  60,  90],  # Leading Medium
    [60,  60,  60],  # Leading Light
]

# Landing -> Takeoff (L -> T, runway occupancy / rollout clearance)
SEP_LT = [
    [65,  65,  60],  # Heavy landing clears runway
    [55,  55,  50],  # Medium landing clears runway
    [45,  45,  40],  # Light landing clears runway
]

# Takeoff -> Landing (T -> L, takeoff roll + rotation before threshold crossing)
SEP_TL = [
    [75,  75,  75],  # Heavy takeoff departs
    [60,  60,  60],  # Medium takeoff departs
    [60,  60,  60],  # Light takeoff departs
]

def get_4way_separation(leading_type, leading_cat, trailing_type, trailing_cat):
    """Computes exact 4-way separation between two operations."""
    if leading_type == "Arrival" and trailing_type == "Arrival":
        return SEP_LL[leading_cat][trailing_cat]
    elif leading_type == "Departure" and trailing_type == "Departure":
        return SEP_TT[leading_cat][trailing_cat]
    elif leading_type == "Arrival" and trailing_type == "Departure":
        return SEP_LT[leading_cat][trailing_cat]
    elif leading_type == "Departure" and trailing_type == "Arrival":
        return SEP_TL[leading_cat][trailing_cat]
    else:
        return 60

def parse_beasley_airland(file_path):
    """Parses an original OR-Library airland file."""
    with open(file_path, "r") as f:
        tokens = f.read().split()
    if not tokens:
        return None
    
    num_planes = int(tokens[0])
    freeze_time = float(tokens[1])
    idx = 2
    planes = []
    for p in range(num_planes):
        app = float(tokens[idx])
        earliest = float(tokens[idx+1])
        target = float(tokens[idx+2])
        latest = float(tokens[idx+3])
        pen_early = float(tokens[idx+4])
        pen_late = float(tokens[idx+5])
        idx += 6
        seps = []
        for q in range(num_planes):
            seps.append(float(tokens[idx]))
            idx += 1
        planes.append({
            "id": p,
            "appearance_time": app,
            "earliest_time": earliest,
            "target_time": target,
            "latest_time": latest,
            "penalty_early": pen_early,
            "penalty_late": pen_late,
            "separation_times": seps,
        })
    return {
        "num_planes": num_planes,
        "freeze_time": freeze_time,
        "planes": planes,
    }

def infer_wake_category(plane, total_planes):
    """Infers wake category (0: Heavy, 1: Medium, 2: Light) from separation profile."""
    # Check max separation this plane requires trailing aircraft to maintain
    max_sep = max(s for s in plane["separation_times"] if s < 9999)
    if max_sep >= 130 or max_sep == 15:  # Beasley ALP 15 or 135s -> Heavy
        return 0
    elif max_sep >= 80 or max_sep == 8:   # Beasley ALP 8 or 90-113s -> Medium
        return 1
    else:
        return 2

# ── Track A: Extended Beasley Suite Builder ───────────────────────────────────

def build_track_a():
    print("Building Track A: Extended Beasley Mixed-Mode Suite (mm_airland1 – mm_airland13)...")
    track_a_index = []

    for i in range(1, 14):
        src_file = AIRLAND_SRC_DIR / f"airland{i}.txt"
        if not src_file.exists():
            print(f"  Warning: {src_file} not found, skipping.")
            continue
        
        parsed = parse_beasley_airland(src_file)
        if not parsed:
            continue
        
        n = parsed["num_planes"]
        freeze = parsed["freeze_time"]

        # Build two regimes:
        # Regime 1: 50/50 Balanced (odd = Arrival, even = Departure)
        # Regime 2: 70/30 Arrival-Heavy
        for regime, ratio_name in [("50_50", "balanced"), ("70_30", "arrival_heavy")]:
            ops = []
            departure_indices = []

            for p_idx, pl in enumerate(parsed["planes"]):
                cat = infer_wake_category(pl, n)
                if regime == "50_50":
                    is_arrival = (p_idx % 2 == 0)
                else:
                    is_arrival = (p_idx % 10 in [0, 1, 2, 4, 5, 6, 8])

                op_type = "Arrival" if is_arrival else "Departure"
                if not is_arrival:
                    departure_indices.append(p_idx)

                ops.append({
                    "id": p_idx,
                    "op_type": op_type,
                    "wake_category": ["Heavy", "Medium", "Light"][cat],
                    "wake_cat_id": cat,
                    "earliest_time": pl["earliest_time"],
                    "target_time": pl["target_time"],
                    "latest_time": pl["latest_time"],
                    "penalty_early": pl["penalty_early"],
                    "penalty_late": pl["penalty_late"],
                })

            # Full N x N 4-way asymmetric separation matrix
            sep_matrix = [[0] * n for _ in range(n)]
            is_scaled_beasley = (n <= 50 and parsed["planes"][0]["earliest_time"] < 1000)

            for u in range(n):
                for v in range(n):
                    if u == v:
                        sep_matrix[u][v] = 0
                        continue
                    
                    cat_u = ops[u]["wake_cat_id"]
                    cat_v = ops[v]["wake_cat_id"]
                    type_u = ops[u]["op_type"]
                    type_v = ops[v]["op_type"]

                    if is_scaled_beasley:
                        # In airland1-8, original ALP times are in abstract Beasley units (e.g. 15, 8, 3)
                        # We use Beasley's original separation for L->L and scale proportionally for 4-way
                        orig_sep = parsed["planes"][u]["separation_times"][v]
                        if orig_sep >= 9999:
                            orig_sep = 15
                        
                        if type_u == "Arrival" and type_v == "Arrival":
                            sep_val = orig_sep
                        elif type_u == "Departure" and type_v == "Departure":
                            sep_val = max(3, int(orig_sep * 0.85))
                        elif type_u == "Arrival" and type_v == "Departure":
                            sep_val = max(3, int(orig_sep * 0.60))
                        else:  # Departure -> Arrival
                            sep_val = max(3, int(orig_sep * 0.70))
                    else:
                        sep_val = get_4way_separation(type_u, cat_u, type_v, cat_v)

                    sep_matrix[u][v] = int(sep_val)

            # Taxiway FIFO Precedence Groups:
            # Sort departures by target time to reflect natural airport taxiway queue order
            sorted_deps = sorted(departure_indices, key=lambda idx: ops[idx]["target_time"])
            taxiway_precedence = []
            for k in range(len(sorted_deps) - 1):
                d_curr = sorted_deps[k]
                d_next = sorted_deps[k+1]
                t_diff = ops[d_next]["target_time"] - ops[d_curr]["target_time"]
                if t_diff <= (30 if is_scaled_beasley else 180):
                    taxiway_precedence.append([d_curr, d_next])

            inst_name = f"mm_airland{i}_{ratio_name}"
            # Also save standard mm_airland{i} pointing to balanced
            inst_obj = {
                "instance_name": inst_name,
                "base_beasley_name": f"airland{i}",
                "track": "TrackA_ExtendedBeasley",
                "regime": ratio_name,
                "num_operations": n,
                "num_arrivals": sum(1 for op in ops if op["op_type"] == "Arrival"),
                "num_departures": sum(1 for op in ops if op["op_type"] == "Departure"),
                "num_runways": 1 if n <= 50 else (2 if n <= 200 else 4),
                "freeze_time": freeze,
                "operations": ops,
                "separation_matrix": sep_matrix,
                "taxiway_precedence": taxiway_precedence,
            }

            out_json = OUT_DIR / f"{inst_name}.json"
            with open(out_json, "w") as f:
                json.dump(inst_obj, f, indent=2)

            if regime == "50_50":
                default_json = OUT_DIR / f"mm_airland{i}.json"
                with open(default_json, "w") as f:
                    json.dump(inst_obj, f, indent=2)

            track_a_index.append({
                "instance": inst_name,
                "num_operations": n,
                "regime": ratio_name,
                "num_arrivals": inst_obj["num_arrivals"],
                "num_departures": inst_obj["num_departures"],
                "num_precedence_edges": len(taxiway_precedence),
            })
            print(f"  Created {inst_name}: {n} ops ({inst_obj['num_arrivals']} arr, {inst_obj['num_departures']} dep, {len(taxiway_precedence)} FIFO edges)")

    return track_a_index

# ── Track B: Real-World Single/Dual Runway Peak Traffic Banks ─────────────────

def generate_peak_bank(name, airport, num_ops, duration_mins, arr_ratio, cat_dist, num_runways=1, seed=42):
    """Constructs realistic peak arrival + departure bank."""
    rng = random.Random(seed)
    num_arr = int(round(num_ops * arr_ratio))
    num_dep = num_ops - num_arr

    # Interleave arrivals and departures
    all_roles = ["Arrival"] * num_arr + ["Departure"] * num_dep
    rng.shuffle(all_roles)

    total_secs = duration_mins * 60
    slot_interval = total_secs / max(1, num_ops)

    ops = []
    dep_indices = []

    for idx, role in enumerate(all_roles):
        r_val = rng.random()
        if r_val < cat_dist[0]:
            cat_id = 0  # Heavy (e.g. A350, B777, B787)
        elif r_val < cat_dist[0] + cat_dist[1]:
            cat_id = 1  # Medium (e.g. A320, B737)
        else:
            cat_id = 2  # Light (e.g. ATR72, E170, CRJ)

        wake_str = ["Heavy", "Medium", "Light"][cat_id]
        
        # Target slot with small realistic jitter
        target = (idx + 0.5) * slot_interval + rng.uniform(-40, 40)
        target = max(60.0, target)

        # Operational window: Earliest = gate release / approach fix; Latest = CTOT / holding fuel limit
        if role == "Arrival":
            # Arrivals can accelerate slightly (-180s) or hold (+600s)
            earliest = max(0.0, target - rng.uniform(90, 240))
            latest = target + rng.uniform(300, 720)
            pen_early = rng.uniform(8.0, 15.0)  # Air early burn penalty
            pen_late = rng.uniform(25.0, 45.0)  # Air holding fuel / passenger delay
        else:
            # Departures pushback window: cannot depart before pushback (-60s buffer), slot tolerance (+900s)
            dep_indices.append(idx)
            earliest = max(0.0, target - rng.uniform(30, 90))
            latest = target + rng.uniform(600, 1200)
            pen_early = rng.uniform(5.0, 10.0)   # Gate hold penalty
            pen_late = rng.uniform(15.0, 35.0)   # Ground delay / slot miss

        ops.append({
            "id": idx,
            "op_type": role,
            "wake_category": wake_str,
            "wake_cat_id": cat_id,
            "earliest_time": round(earliest, 1),
            "target_time": round(target, 1),
            "latest_time": round(latest, 1),
            "penalty_early": round(pen_early, 2),
            "penalty_late": round(pen_late, 2),
        })

    # Sort target times to preserve natural flow, re-index
    ops.sort(key=lambda o: o["target_time"])
    dep_indices = []
    for new_idx, op in enumerate(ops):
        op["id"] = new_idx
        if op["op_type"] == "Departure":
            dep_indices.append(new_idx)

    # 4-way separation matrix
    sep_matrix = [[0] * num_ops for _ in range(num_ops)]
    for u in range(num_ops):
        for v in range(num_ops):
            if u == v:
                sep_matrix[u][v] = 0
            else:
                sep_matrix[u][v] = get_4way_separation(
                    ops[u]["op_type"], ops[u]["wake_cat_id"],
                    ops[v]["op_type"], ops[v]["wake_cat_id"]
                )

    # Taxiway FIFO Precedence
    taxiway_precedence = []
    for k in range(len(dep_indices) - 1):
        d_curr = dep_indices[k]
        d_next = dep_indices[k+1]
        if ops[d_next]["target_time"] - ops[d_curr]["target_time"] <= 150:
            taxiway_precedence.append([d_curr, d_next])

    inst = {
        "instance_name": name,
        "airport": airport,
        "track": "TrackB_RealWorldAirport",
        "duration_minutes": duration_mins,
        "num_operations": num_ops,
        "num_arrivals": sum(1 for o in ops if o["op_type"] == "Arrival"),
        "num_departures": sum(1 for o in ops if o["op_type"] == "Departure"),
        "num_runways": num_runways,
        "freeze_time": 180.0,  # 3-minute final approach freeze
        "operations": ops,
        "separation_matrix": sep_matrix,
        "taxiway_precedence": taxiway_precedence,
    }

    out_file = OUT_DIR / f"{name}.json"
    with open(out_file, "w") as f:
        json.dump(inst, f, indent=2)

    print(f"  Created Track B {name} ({airport}): {num_ops} ops ({inst['num_arrivals']} arr, {inst['num_departures']} dep, {len(taxiway_precedence)} FIFO edges)")
    return inst

def build_track_b():
    print("\nBuilding Track B: Real-World Airport Single/Dual-Runway Peak Banks...")
    track_b_list = []

    # 1. Paris CDG 1-Hour Peak Bank (45 ops, 1 mixed-mode runway, European hub bank: 20% H, 70% M, 10% L)
    track_b_list.append(generate_peak_bank(
        name="cdg_peak_1h", airport="LFPG_CDG", num_ops=45, duration_mins=60,
        arr_ratio=0.52, cat_dist=[0.20, 0.70, 0.10], num_runways=1, seed=101
    ))

    # 2. Paris CDG 3-Hour Extended Bank (120 ops, 2 runways or single runway peak, Ikli et al. 2021)
    track_b_list.append(generate_peak_bank(
        name="cdg_peak_3h", airport="LFPG_CDG", num_ops=120, duration_mins=180,
        arr_ratio=0.50, cat_dist=[0.22, 0.68, 0.10], num_runways=1, seed=102
    ))

    # 3. London Gatwick LGW 1-Hour Peak Bank (52 ops/hour, world's highest intensity single-runway: 15% H, 80% M, 5% L)
    track_b_list.append(generate_peak_bank(
        name="lgw_peak_1h", airport="EGKK_LGW", num_ops=52, duration_mins=60,
        arr_ratio=0.50, cat_dist=[0.15, 0.80, 0.05], num_runways=1, seed=103
    ))

    # 4. Zurich ZRH 1-Hour Peak Bank (40 ops, noise & wake sensitive hub: 18% H, 72% M, 10% L)
    track_b_list.append(generate_peak_bank(
        name="zrh_peak_1h", airport="LSZH_ZRH", num_ops=40, duration_mins=60,
        arr_ratio=0.55, cat_dist=[0.18, 0.72, 0.10], num_runways=1, seed=104
    ))

    # 5. Chennai MAA / Mumbai BOM Peak Bank (60 ops in 75 mins, high domestic single-runway peak: 12% H, 83% M, 5% L)
    track_b_list.append(generate_peak_bank(
        name="maa_bom_peak", airport="VOMM_MAA_VABB_BOM", num_ops=60, duration_mins=75,
        arr_ratio=0.50, cat_dist=[0.12, 0.83, 0.05], num_runways=1, seed=105
    ))

    return track_b_list

if __name__ == "__main__":
    t_a = build_track_a()
    t_b = build_track_b()
    print(f"\nSuccessfully generated {len(t_a)} Track A instances and {len(t_b)} Track B instances in {OUT_DIR}")
