#!/usr/bin/env python3
"""
compare_gerad.py — 10h Layover Threshold Experiment (GERAD G-2014-22)

Implements the controlled A/B comparison described in:
  docs/research/UltraCrew_Layover_Threshold_Experiment.md

The experiment works at the FLIGHT LEG level, mirroring the UltraCrew
pairings_handler pipeline:

  1. Load all flight legs per crew member (via duties.csv → flights.csv)
  2. Group legs into FDPs: consecutive legs with ground time < LAYOVER_REST_HOURS
     belong to the same FDP.
  3. Group FDPs into pairings: consecutive FDPs with inter-FDP rest
     < HOME_BASE_REST_HOURS (34h) belong to the same pairing.

This is run twice:
  Condition A (baseline):     LAYOVER_REST_HOURS = 8.0  (current UltraCrew default)
  Condition B (experimental): LAYOVER_REST_HOURS = 10.0 (GENCOL paper value)

Compares both conditions against the GERAD reference pairings on:
  - Pairing count
  - Pairing count ratio vs reference
  - Multi-FDP pairing ratio
  - Mean pairing span (days)

Usage:
    python3 compare_gerad.py [--instances-dir PATH] [--output {table,json,csv}]
"""

import argparse
import csv
import json
import os
import sys
from datetime import datetime, timezone
from collections import defaultdict

BENCHMARK_DIR = os.path.join(os.path.dirname(__file__), "benchmarks", "gerad-g2014-22")
HOME_BASE_REST_HOURS = 34.0  # pairing boundary — fixed, not varied in this experiment


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def parse_utc(s: str) -> datetime:
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def load_flights(instance_dir: str) -> dict:
    """Load flights.csv. Returns {flight_id: flight_dict}."""
    flights = {}
    with open(os.path.join(instance_dir, "flights.csv")) as f:
        for row in csv.DictReader(f):
            flights[row["flight_id"]] = {
                "flight_id": row["flight_id"],
                "departure_utc": parse_utc(row["departure_utc"]),
                "arrival_utc": parse_utc(row["arrival_utc"]),
            }
    return flights


def load_crew_flights(instance_dir: str, flights: dict) -> dict:
    """
    Load duties.csv and resolve flight_ids to flight objects.
    Returns {crew_id: [sorted flight dicts with crew_id attached]}.
    Each flight dict has: flight_id, departure_utc, arrival_utc, crew_id.
    """
    crew_flights = defaultdict(list)
    with open(os.path.join(instance_dir, "duties.csv")) as f:
        for row in csv.DictReader(f):
            crew_id = row["crew_id"]
            flight_ids = [fid.strip() for fid in row["flight_ids"].strip('"').split(",")]
            for fid in flight_ids:
                if fid in flights:
                    leg = dict(flights[fid])
                    leg["crew_id"] = crew_id
                    crew_flights[crew_id].append(leg)
    # Sort each crew's legs by departure time
    for crew_id in crew_flights:
        crew_flights[crew_id].sort(key=lambda f: f["departure_utc"])
    return crew_flights


def load_reference_pairings(instance_dir: str) -> list:
    """Load pairings.csv (GERAD reference). Returns list of pairing dicts."""
    pairings = []
    with open(os.path.join(instance_dir, "pairings.csv")) as f:
        for row in csv.DictReader(f):
            duty_ids = [d.strip() for d in row["duty_ids"].strip('"').split(",")]
            pairings.append({
                "pairing_id": row["pairing_id"],
                "crew_id": row["crew_id"],
                "duty_count": len(duty_ids),
                "start_utc": parse_utc(row["start_utc"]),
                "end_utc": parse_utc(row["end_utc"]),
            })
    return pairings


# ---------------------------------------------------------------------------
# Pairing grouping (mirrors UltraCrew pairings_handler at flight-leg level)
# ---------------------------------------------------------------------------

def group_flights_into_pairings(crew_flights: dict, layover_rest_hours: float) -> list:
    """
    Group flight legs into FDPs and then into pairings.

    Step 1 — FDP grouping:
      A new FDP starts when the ground time between consecutive legs
      >= layover_rest_hours. Ground time = next departure - prev arrival.

    Step 2 — Pairing grouping:
      A new pairing starts when the inter-FDP rest >= HOME_BASE_REST_HOURS (34h).
      Inter-FDP rest = next FDP first departure - prev FDP last arrival.
    """
    pairings = []
    pairing_counter = 0

    for crew_id, legs in crew_flights.items():
        if not legs:
            continue

        # Step 1: group legs into FDPs
        fdps = []
        current_fdp = [legs[0]]
        for i in range(1, len(legs)):
            prev = legs[i - 1]
            curr = legs[i]
            ground_time_h = (curr["departure_utc"] - prev["arrival_utc"]).total_seconds() / 3600.0
            if ground_time_h >= layover_rest_hours:
                fdps.append(current_fdp)
                current_fdp = [curr]
            else:
                current_fdp.append(curr)
        fdps.append(current_fdp)

        # Step 2: group FDPs into pairings
        current_pairing_fdps = [fdps[0]]
        for i in range(1, len(fdps)):
            prev_fdp = fdps[i - 1]
            curr_fdp = fdps[i]
            inter_fdp_rest_h = (
                curr_fdp[0]["departure_utc"] - prev_fdp[-1]["arrival_utc"]
            ).total_seconds() / 3600.0

            if inter_fdp_rest_h >= HOME_BASE_REST_HOURS:
                pairing_counter += 1
                pairings.append(_make_pairing(pairing_counter, crew_id, current_pairing_fdps))
                current_pairing_fdps = [curr_fdp]
            else:
                current_pairing_fdps.append(curr_fdp)

        pairing_counter += 1
        pairings.append(_make_pairing(pairing_counter, crew_id, current_pairing_fdps))

    return pairings


def _make_pairing(counter: int, crew_id: str, fdps: list) -> dict:
    first_leg = fdps[0][0]
    last_leg = fdps[-1][-1]
    span_days = (last_leg["arrival_utc"] - first_leg["departure_utc"]).total_seconds() / 86400.0
    return {
        "pairing_id": f"UC{counter:05d}",
        "crew_id": crew_id,
        "fdp_count": len(fdps),
        "leg_count": sum(len(fdp) for fdp in fdps),
        "start_utc": first_leg["departure_utc"],
        "end_utc": last_leg["arrival_utc"],
        "span_days": span_days,
    }


# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------

def compute_metrics(pairings: list) -> dict:
    n = len(pairings)
    if n == 0:
        return {"count": 0, "multi_fdp_ratio": 0.0, "mean_span_days": 0.0}
    multi_fdp = sum(1 for p in pairings if p["fdp_count"] > 1)
    mean_span = sum(p["span_days"] for p in pairings) / n
    return {
        "count": n,
        "multi_fdp_ratio": round(multi_fdp / n, 4),
        "mean_span_days": round(mean_span, 3),
    }


def compute_reference_metrics(ref_pairings: list) -> dict:
    n = len(ref_pairings)
    if n == 0:
        return {"count": 0, "multi_duty_ratio": 0.0, "mean_span_days": 0.0}
    multi_duty = sum(1 for p in ref_pairings if p["duty_count"] > 1)
    spans = [(p["end_utc"] - p["start_utc"]).total_seconds() / 86400.0 for p in ref_pairings]
    return {
        "count": n,
        "multi_duty_ratio": round(multi_duty / n, 4),
        "mean_span_days": round(sum(spans) / n, 3),
    }


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

def print_table(rows: list) -> None:
    cols = [
        ("instance",        10),
        ("ref_count",        9),
        ("8h_count",         8),
        ("10h_count",        9),
        ("8h_ratio",         8),
        ("10h_ratio",        9),
        ("ref_multi",       10),
        ("8h_multi",         9),
        ("10h_multi",       10),
        ("ref_span_d",       9),
        ("8h_span_d",        9),
        ("10h_span_d",      10),
    ]
    header = "  ".join(name.ljust(w) for name, w in cols)
    sep    = "  ".join("-" * w for _, w in cols)
    print(header)
    print(sep)
    for r in rows:
        line = "  ".join([
            r["instance"].ljust(10),
            str(r["ref_count"]).ljust(9),
            str(r["8h_count"]).ljust(8),
            str(r["10h_count"]).ljust(9),
            f"{r['8h_ratio']:.3f}".ljust(8),
            f"{r['10h_ratio']:.3f}".ljust(9),
            f"{r['ref_multi']:.3f}".ljust(10),
            f"{r['8h_multi']:.3f}".ljust(9),
            f"{r['10h_multi']:.3f}".ljust(10),
            f"{r['ref_span_d']:.2f}".ljust(9),
            f"{r['8h_span_d']:.2f}".ljust(9),
            f"{r['10h_span_d']:.2f}".ljust(10),
        ])
        print(line)


def print_summary(rows: list) -> None:
    avg_8h  = sum(r["8h_ratio"]  for r in rows) / len(rows)
    avg_10h = sum(r["10h_ratio"] for r in rows) / len(rows)
    delta   = avg_10h - avg_8h
    print()
    print(f"Instances evaluated   : {len(rows)}")
    print(f"Avg pairing ratio 8h  : {avg_8h:.3f}  ({avg_8h*100:.1f}% of reference)")
    print(f"Avg pairing ratio 10h : {avg_10h:.3f}  ({avg_10h*100:.1f}% of reference)")
    print(f"Delta (10h - 8h)      : {delta:+.3f}  ({delta*100:+.1f}pp)")
    if abs(delta) < 0.001:
        print("Interpretation: threshold change has no measurable effect on pairing count.")
    elif delta > 0.05:
        print("Interpretation: threshold change has a meaningful effect on pairing count (>5pp).")
    elif delta > 0.01:
        print("Interpretation: threshold change has a modest effect on pairing count (1-5pp).")
    else:
        print("Interpretation: threshold change has minimal effect on pairing count (<1pp).")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="10h Layover Threshold Experiment — GERAD G-2014-22 (flight-leg level)"
    )
    parser.add_argument(
        "--instances-dir",
        default=BENCHMARK_DIR,
        help="Path to gerad-g2014-22 directory (default: benchmarks/gerad-g2014-22)",
    )
    parser.add_argument(
        "--output",
        choices=["table", "json", "csv"],
        default="table",
    )
    args = parser.parse_args()

    base = args.instances_dir
    instance_names = sorted(
        d for d in os.listdir(base)
        if d.startswith("instance") and os.path.isdir(os.path.join(base, d))
    )

    if not instance_names:
        sys.exit(f"ERROR: No instance directories found in {base}")

    rows = []
    for name in instance_names:
        idir = os.path.join(base, name)
        try:
            flights      = load_flights(idir)
            crew_flights = load_crew_flights(idir, flights)
            ref_pairings = load_reference_pairings(idir)
        except FileNotFoundError as e:
            print(f"WARNING: Skipping {name}: {e}", file=sys.stderr)
            continue

        pairings_8h  = group_flights_into_pairings(crew_flights, layover_rest_hours=8.0)
        pairings_10h = group_flights_into_pairings(crew_flights, layover_rest_hours=10.0)

        ref  = compute_reference_metrics(ref_pairings)
        m8h  = compute_metrics(pairings_8h)
        m10h = compute_metrics(pairings_10h)

        ref_count = ref["count"]
        rows.append({
            "instance":    name,
            "ref_count":   ref_count,
            "8h_count":    m8h["count"],
            "10h_count":   m10h["count"],
            "8h_ratio":    round(m8h["count"]  / ref_count, 4) if ref_count else 0.0,
            "10h_ratio":   round(m10h["count"] / ref_count, 4) if ref_count else 0.0,
            "ref_multi":   ref["multi_duty_ratio"],
            "8h_multi":    m8h["multi_fdp_ratio"],
            "10h_multi":   m10h["multi_fdp_ratio"],
            "ref_span_d":  ref["mean_span_days"],
            "8h_span_d":   m8h["mean_span_days"],
            "10h_span_d":  m10h["mean_span_days"],
        })

    if not rows:
        sys.exit("ERROR: No instances were successfully evaluated.")

    if args.output == "table":
        print_table(rows)
        print_summary(rows)
    elif args.output == "json":
        print(json.dumps(rows, indent=2, default=str))
    elif args.output == "csv":
        fieldnames = list(rows[0].keys())
        w = csv.DictWriter(sys.stdout, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)

    return 0


if __name__ == "__main__":
    sys.exit(main())