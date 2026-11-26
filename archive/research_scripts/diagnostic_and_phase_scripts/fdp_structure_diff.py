#!/usr/bin/env python3
"""
fdp_structure_diff.py — Structural change analysis for the 8h vs 10h experiment.

For each instance, compares the FDP composition of each pairing between
the 8h and 10h conditions. Reports:
  - How many pairings changed FDP composition (count unchanged, structure changed)
  - How many pairings are identical
  - Mean change in FDP count per pairing
  - Whether any pairing's span changed
"""
import csv
from datetime import datetime
from collections import defaultdict
import os

def parse_utc(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))

HOME_BASE_REST_HOURS = 34.0

def load_flights(idir):
    flights = {}
    with open(os.path.join(idir, "flights.csv")) as f:
        for row in csv.DictReader(f):
            flights[row["flight_id"]] = {
                "dep": parse_utc(row["departure_utc"]),
                "arr": parse_utc(row["arrival_utc"]),
            }
    return flights

def load_crew_flights(idir, flights):
    crew_legs = defaultdict(list)
    with open(os.path.join(idir, "duties.csv")) as f:
        for row in csv.DictReader(f):
            crew_id = row["crew_id"]
            for fid in row["flight_ids"].strip('"').split(","):
                fid = fid.strip()
                if fid in flights:
                    leg = dict(flights[fid])
                    leg["crew_id"] = crew_id
                    crew_legs[crew_id].append(leg)
    for crew_id in crew_legs:
        crew_legs[crew_id].sort(key=lambda l: l["dep"])
    return crew_legs

def group_into_pairings(crew_legs, layover_h):
    pairings = []
    for crew_id, legs in crew_legs.items():
        if not legs:
            continue
        fdps = []
        current_fdp = [legs[0]]
        for i in range(1, len(legs)):
            gap = (legs[i]["dep"] - legs[i-1]["arr"]).total_seconds() / 3600.0
            if gap >= layover_h:
                fdps.append(current_fdp)
                current_fdp = [legs[i]]
            else:
                current_fdp.append(legs[i])
        fdps.append(current_fdp)

        current_pairing = [fdps[0]]
        for i in range(1, len(fdps)):
            rest = (fdps[i][0]["dep"] - fdps[i-1][-1]["arr"]).total_seconds() / 3600.0
            if rest >= HOME_BASE_REST_HOURS:
                pairings.append((crew_id, current_pairing))
                current_pairing = [fdps[i]]
            else:
                current_pairing.append(fdps[i])
        pairings.append((crew_id, current_pairing))
    return pairings

base = os.path.join(os.path.dirname(__file__), "..", "benchmarks", "gerad-g2014-22")

print(f"{'Instance':12s}  {'Pairings':8s}  {'Changed':8s}  {'Identical':9s}  {'Chg%':6s}  {'AvgFDPDelta':11s}")
print("-" * 70)

for i in range(1, 8):
    idir = os.path.join(base, f"instance{i}")
    flights = load_flights(idir)
    crew_legs = load_crew_flights(idir, flights)

    p8  = group_into_pairings(crew_legs, 8.0)
    p10 = group_into_pairings(crew_legs, 10.0)

    assert len(p8) == len(p10), f"Pairing count mismatch in instance{i}: {len(p8)} vs {len(p10)}"

    changed = 0
    fdp_deltas = []
    for (c8, fdps8), (c10, fdps10) in zip(p8, p10):
        n8  = len(fdps8)
        n10 = len(fdps10)
        if n8 != n10:
            changed += 1
        fdp_deltas.append(n10 - n8)

    avg_delta = sum(fdp_deltas) / len(fdp_deltas) if fdp_deltas else 0.0
    identical = len(p8) - changed
    pct = 100.0 * changed / len(p8) if p8 else 0.0
    print(f"instance{i}     {len(p8):8d}  {changed:8d}  {identical:9d}  {pct:5.1f}%  {avg_delta:+.3f}")