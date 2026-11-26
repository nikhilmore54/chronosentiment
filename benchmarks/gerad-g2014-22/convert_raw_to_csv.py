#!/usr/bin/env python3
"""
convert_raw_to_csv.py
---------------------
Converts the real GERAD G-2014-22 raw instance format into the UltraCrew
adapter CSV schema (flights.csv, crew.csv, duties.csv, pairings.csv,
swap_exchanges.csv).

Real format (from G1422-DataSets.zip):
  day_N.csv          — flight legs for day N (1-indexed)
  listOfBases.csv    — airports with base status and crew counts
  crew_avail_const.csv — crew available per base per day
  initialSolution.in — reference pairings (used to derive duties/pairings)

Usage:
  python3 convert_raw_to_csv.py --instance raw/instance1/instance1 --out instance1
  python3 convert_raw_to_csv.py --instance raw/instance2/instance2 --out instance2
  # etc.

Output directory will be created under benchmarks/gerad-g2014-22/<out>/
"""

import argparse
import csv
import os
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

EPOCH = datetime(2000, 1, 1, tzinfo=timezone.utc)  # GERAD uses year 2000


def parse_time(date_str: str, time_str: str) -> datetime:
    """Parse 'YYYY-MM-DD' + 'HH:MM' into a UTC datetime."""
    dt = datetime.strptime(f"{date_str.strip()} {time_str.strip()}", "%Y-%m-%d %H:%M")
    return dt.replace(tzinfo=timezone.utc)


def fmt_utc(dt: datetime) -> str:
    return dt.strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# Step 1: Parse all day_N.csv files → flat list of flight legs
# ---------------------------------------------------------------------------

def parse_flights(instance_dir: Path) -> list[dict]:
    """
    Returns list of dicts with keys:
      flight_id, origin, destination, departure_utc, arrival_utc,
      aircraft_type, block_minutes
    """
    flights = []
    day_files = sorted(
        instance_dir.glob("day_*.csv"),
        key=lambda p: int(re.search(r"day_(\d+)", p.name).group(1))
    )
    for day_file in day_files:
        with open(day_file, newline="") as f:
            reader = csv.reader(f)
            for row in reader:
                if not row or row[0].strip().startswith("#"):
                    continue
                # Format: leg_nb , airport_dep , date_dep , hour_dep , airport_arr , date_arr , hour_arr
                if len(row) < 7:
                    continue
                leg_id = row[0].strip()
                if leg_id.lower().startswith("leg"):
                    orig = row[1].strip()
                    date_dep = row[2].strip()
                    hour_dep = row[3].strip()
                    dest = row[4].strip()
                    date_arr = row[5].strip()
                    hour_arr = row[6].strip()
                    try:
                        dep = parse_time(date_dep, hour_dep)
                        arr = parse_time(date_arr, hour_arr)
                        block_min = int((arr - dep).total_seconds() / 60)
                        if block_min < 0:
                            # Overnight — add 1 day
                            arr += timedelta(days=1)
                            block_min = int((arr - dep).total_seconds() / 60)
                        flights.append({
                            "flight_id": leg_id,
                            "origin": orig,
                            "destination": dest,
                            "departure_utc": fmt_utc(dep),
                            "arrival_utc": fmt_utc(arr),
                            "aircraft_type": "A320",  # GERAD doesn't specify; A320-family assumed
                            "block_minutes": block_min,
                        })
                    except Exception as e:
                        print(f"  Warning: skipping leg {leg_id}: {e}", file=sys.stderr)
    return flights


# ---------------------------------------------------------------------------
# Step 2: Parse listOfBases.csv → bases and spoke airports
# ---------------------------------------------------------------------------

def parse_bases(instance_dir: Path) -> tuple[list[str], dict[str, int]]:
    """
    Returns:
      bases      — list of base airport codes (status == 1)
      base_crew  — dict base -> total crew count
    """
    bases = []
    base_crew = {}
    bases_file = instance_dir / "listOfBases.csv"
    if not bases_file.exists():
        return bases, base_crew
    with open(bases_file, newline="") as f:
        reader = csv.reader(f)
        for row in reader:
            if not row or row[0].strip().startswith("airport"):
                continue
            if len(row) < 3:
                continue
            airport = row[0].strip()
            status = row[1].strip()
            nb = row[2].strip()
            if status == "1":
                bases.append(airport)
                try:
                    base_crew[airport] = int(nb)
                except ValueError:
                    base_crew[airport] = 0
    return bases, base_crew


# ---------------------------------------------------------------------------
# Step 3: Parse crew_avail_const.csv → total crew per base
# ---------------------------------------------------------------------------

def parse_crew_avail(instance_dir: Path, bases: list[str]) -> dict[str, int]:
    """Returns dict base -> max daily crew available (peak across all days)."""
    avail_file = instance_dir / "crew_avail_const.csv"
    peak = {b: 0 for b in bases}
    if not avail_file.exists():
        return peak
    with open(avail_file, newline="") as f:
        reader = csv.reader(f)
        header_found = False
        col_map = {}
        for row in reader:
            if not row:
                continue
            first = row[0].strip().strip('"')
            if first.lower() == "base":
                # Header row: base , BASE1 , BASE2 , ...
                for i, cell in enumerate(row[1:], 1):
                    b = cell.strip()
                    if b in bases:
                        col_map[i] = b
                header_found = True
                continue
            if not header_found:
                continue
            if first.lower().startswith("day"):
                for col_idx, base in col_map.items():
                    try:
                        val = int(row[col_idx].strip())
                        if val > peak[base]:
                            peak[base] = val
                    except (ValueError, IndexError):
                        pass
    return peak


# ---------------------------------------------------------------------------
# Step 4: Parse initialSolution.in → pairings (list of leg sequences per base)
# ---------------------------------------------------------------------------

def parse_initial_solution(instance_dir: Path) -> list[dict]:
    """
    Returns list of pairing dicts:
      pairing_id, base, leg_ids (list of str)
    """
    sol_file = instance_dir / "initialSolution.in"
    if not sol_file.exists():
        return []
    pairings = []
    with open(sol_file) as f:
        content = f.read()
    # Pattern: Pairing N : Base BASEX : LEG_XX_YY , LEG_XX_YY , ...;
    pattern = re.compile(
        r"Pairing\s+(\d+)\s*:\s*Base\s+(\w+)\s*:\s*([^;]+);",
        re.IGNORECASE
    )
    for m in pattern.finditer(content):
        pid = int(m.group(1))
        base = m.group(2).strip()
        legs_raw = m.group(3).strip()
        leg_ids = [l.strip() for l in legs_raw.split(",") if l.strip()]
        pairings.append({
            "pairing_id": f"P{pid:04d}",
            "base": base,
            "leg_ids": leg_ids,
        })
    return pairings


# ---------------------------------------------------------------------------
# Step 5: Build crew records from base crew counts
# ---------------------------------------------------------------------------

FIRST_NAMES = [
    "Jean", "Marie", "Pierre", "Sophie", "Francois", "Isabelle", "Philippe",
    "Catherine", "Michel", "Nathalie", "Laurent", "Sylvie", "Nicolas", "Valerie",
    "Christophe", "Sandrine", "Stephane", "Celine", "Julien", "Aurelie",
    "Thomas", "Emilie", "Alexandre", "Camille", "Maxime", "Lucie", "Antoine",
    "Manon", "Romain", "Lea", "Quentin", "Chloe", "Florian", "Pauline",
    "Mathieu", "Julie", "Sebastien", "Laure", "Benoit", "Margot",
    "Guillaume", "Anais", "Thibault", "Elise", "Clement", "Ines",
    "Adrien", "Zoe", "Damien", "Amandine", "Hugo", "Clara", "Alexis",
    "Oceane", "Raphael", "Jade", "Theo", "Lola", "Baptiste", "Inès",
]
LAST_NAMES = [
    "Martin", "Bernard", "Dubois", "Thomas", "Robert", "Richard", "Petit",
    "Durand", "Leroy", "Moreau", "Simon", "Laurent", "Lefebvre", "Michel",
    "Garcia", "David", "Bertrand", "Roux", "Vincent", "Fournier",
    "Morel", "Girard", "Andre", "Lefevre", "Mercier", "Dupont", "Lambert",
    "Bonnet", "Francois", "Martinez", "Legrand", "Garnier", "Faure", "Rousseau",
    "Blanc", "Guerin", "Muller", "Henry", "Roussel", "Nicolas",
    "Perrin", "Morin", "Mathieu", "Clement", "Gauthier", "Dumont",
    "Lopez", "Fontaine", "Chevalier", "Robin", "Masson", "Giraud",
    "Caron", "Renard", "Schmitt", "Gilles", "Leclerc", "Collin",
]


def build_crew(bases: list[str], base_crew: dict[str, int]) -> list[dict]:
    """Build crew records from base crew counts."""
    import random
    rng = random.Random(20140122)
    crew = []
    cid = 1
    used_names = set()
    for base in bases:
        count = base_crew.get(base, 5)
        for _ in range(count):
            while True:
                fn = rng.choice(FIRST_NAMES)
                ln = rng.choice(LAST_NAMES)
                name = f"{fn} {ln}"
                if name not in used_names:
                    used_names.add(name)
                    break
            qual = rng.choice(["A319", "A320", "A321"])
            contract = rng.choice(["full_time", "full_time", "full_time", "part_time"])
            crew.append({
                "crew_id": f"C{cid:04d}",
                "name": name,
                "base": base,
                "qualification": qual,
                "contract_type": contract,
            })
            cid += 1
    return crew


# ---------------------------------------------------------------------------
# Step 6: Build duties from pairings (each pairing day = one duty)
# ---------------------------------------------------------------------------

def build_duties_and_pairings(
    raw_pairings: list[dict],
    flight_index: dict[str, dict],
    crew: list[dict],
) -> tuple[list[dict], list[dict]]:
    """
    Each raw pairing becomes one UltraCrew pairing.
    Within a pairing, consecutive legs on the same calendar day form one duty.
    Returns (duties, pairings).
    """
    # Map base -> crew_ids at that base
    base_crew_map: dict[str, list[str]] = {}
    for c in crew:
        base_crew_map.setdefault(c["base"], []).append(c["crew_id"])

    duties = []
    pairings_out = []
    did = 1
    crew_assignment_idx: dict[str, int] = {b: 0 for b in base_crew_map}

    for rp in raw_pairings:
        base = rp["base"]
        leg_ids = rp["leg_ids"]

        # Resolve legs that exist in flight_index
        resolved_legs = []
        for lid in leg_ids:
            if lid in flight_index:
                resolved_legs.append(flight_index[lid])
            else:
                # Try case-insensitive
                lid_lower = lid.lower()
                match = next((f for fid, f in flight_index.items() if fid.lower() == lid_lower), None)
                if match:
                    resolved_legs.append(match)

        if not resolved_legs:
            continue

        # Sort by departure
        resolved_legs.sort(key=lambda f: f["departure_utc"])

        # Group into duties by calendar day
        day_groups: dict[str, list[dict]] = {}
        for leg in resolved_legs:
            day_key = leg["departure_utc"][:10]  # YYYY-MM-DD
            day_groups.setdefault(day_key, []).append(leg)

        # Assign a crew member from this base (round-robin)
        crew_ids_at_base = base_crew_map.get(base, [])
        if not crew_ids_at_base:
            # Fall back to any crew
            crew_ids_at_base = [c["crew_id"] for c in crew]
        idx = crew_assignment_idx.get(base, 0)
        crew_id = crew_ids_at_base[idx % len(crew_ids_at_base)]
        crew_assignment_idx[base] = idx + 1

        pairing_duty_ids = []
        pairing_start = None
        pairing_end = None

        for day_key in sorted(day_groups.keys()):
            day_legs = sorted(day_groups[day_key], key=lambda f: f["departure_utc"])
            first_dep = datetime.strptime(day_legs[0]["departure_utc"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
            last_arr = datetime.strptime(day_legs[-1]["arrival_utc"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
            report = first_dep - timedelta(minutes=45)
            release = last_arr + timedelta(minutes=30)
            flight_ids_str = ",".join(l["flight_id"] for l in day_legs)

            duty_id = f"D{did:04d}"
            duties.append({
                "duty_id": duty_id,
                "crew_id": crew_id,
                "flight_ids": flight_ids_str,
                "report_utc": fmt_utc(report),
                "release_utc": fmt_utc(release),
            })
            pairing_duty_ids.append(duty_id)
            if pairing_start is None or report < pairing_start:
                pairing_start = report
            if pairing_end is None or release > pairing_end:
                pairing_end = release
            did += 1

        if pairing_duty_ids:
            pairings_out.append({
                "pairing_id": rp["pairing_id"],
                "crew_id": crew_id,
                "duty_ids": ",".join(pairing_duty_ids),
                "start_utc": fmt_utc(pairing_start),
                "end_utc": fmt_utc(pairing_end),
                "home_base": base,
            })

    return duties, pairings_out


# ---------------------------------------------------------------------------
# Step 7: Build swap_exchanges (adjacent pairings at same base)
# ---------------------------------------------------------------------------

def build_swaps(pairings: list[dict]) -> list[dict]:
    swaps = []
    sid = 1
    # Pair up consecutive pairings at the same base
    by_base: dict[str, list[dict]] = {}
    for p in pairings:
        by_base.setdefault(p["home_base"], []).append(p)
    for base, ps in by_base.items():
        for i in range(0, len(ps) - 1, 2):
            a = ps[i]
            b = ps[i + 1]
            a_duties = a["duty_ids"].split(",")
            b_duties = b["duty_ids"].split(",")
            if a_duties and b_duties:
                swaps.append({
                    "swap_id": f"S{sid:04d}",
                    "crew_id_a": a["crew_id"],
                    "crew_id_b": b["crew_id"],
                    "duty_id_a": a_duties[0],
                    "duty_id_b": b_duties[0],
                    "swap_type": "full_pairing",
                })
                sid += 1
    return swaps


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def convert(instance_dir: Path, out_dir: Path):
    out_dir.mkdir(parents=True, exist_ok=True)
    print(f"Converting {instance_dir} -> {out_dir}")

    # 1. Flights
    print("  Parsing flights...")
    flights = parse_flights(instance_dir)
    flight_index = {f["flight_id"]: f for f in flights}
    print(f"  Found {len(flights)} flight legs")

    # 2. Bases
    print("  Parsing bases...")
    bases, base_crew_counts = parse_bases(instance_dir)
    print(f"  Bases: {bases}")

    # 3. Crew availability (use as crew count per base)
    print("  Parsing crew availability...")
    avail_peak = parse_crew_avail(instance_dir, bases)
    # Use max(base_crew_counts, avail_peak) as crew count
    final_crew_counts = {}
    for b in bases:
        declared = base_crew_counts.get(b, 0)
        peak = avail_peak.get(b, 0)
        final_crew_counts[b] = max(declared, peak, 1)
    print(f"  Crew counts: {final_crew_counts}")

    # 4. Crew records
    print("  Building crew records...")
    crew = build_crew(bases, final_crew_counts)
    print(f"  Generated {len(crew)} crew members")

    # 5. Initial solution (pairings)
    print("  Parsing initial solution...")
    raw_pairings = parse_initial_solution(instance_dir)
    print(f"  Found {len(raw_pairings)} raw pairings")

    # 6. Duties and pairings
    print("  Building duties and pairings...")
    duties, pairings = build_duties_and_pairings(raw_pairings, flight_index, crew)
    print(f"  Generated {len(duties)} duties, {len(pairings)} pairings")

    # 7. Swaps
    swaps = build_swaps(pairings)
    print(f"  Generated {len(swaps)} swap pairs")

    # Write CSVs
    def write_csv(filename, rows, fieldnames):
        path = out_dir / filename
        with open(path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(rows)
        print(f"  Wrote {path} ({len(rows)} rows)")

    write_csv("flights.csv", flights, [
        "flight_id", "origin", "destination", "departure_utc", "arrival_utc",
        "aircraft_type", "block_minutes",
    ])
    write_csv("crew.csv", crew, [
        "crew_id", "name", "base", "qualification", "contract_type",
    ])
    write_csv("duties.csv", duties, [
        "duty_id", "crew_id", "flight_ids", "report_utc", "release_utc",
    ])
    write_csv("pairings.csv", pairings, [
        "pairing_id", "crew_id", "duty_ids", "start_utc", "end_utc", "home_base",
    ])
    write_csv("swap_exchanges.csv", swaps, [
        "swap_id", "crew_id_a", "crew_id_b", "duty_id_a", "duty_id_b", "swap_type",
    ])

    print(f"  Done. Output in {out_dir}/")
    return {
        "flights": len(flights),
        "crew": len(crew),
        "duties": len(duties),
        "pairings": len(pairings),
        "swaps": len(swaps),
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Convert GERAD G-2014-22 raw format to UltraCrew CSV schema")
    parser.add_argument("--instance", required=False, default=None, help="Path to raw instance directory (containing day_N.csv files)")
    parser.add_argument("--out", required=False, default=None, help="Output subdirectory name under benchmarks/gerad-g2014-22/")
    parser.add_argument("--all", action="store_true", help="Convert all instances 1-7")
    args = parser.parse_args()

    script_dir = Path(__file__).parent

    if args.all:
        summary = {}
        for i in range(1, 8):
            raw = script_dir / "raw" / f"instance{i}" / f"instance{i}"
            if raw.exists():
                out = script_dir / f"instance{i}"
                stats = convert(raw, out)
                summary[f"instance{i}"] = stats
            else:
                print(f"  Skipping instance{i} (not found at {raw})")
        print("\nSummary:")
        for inst, stats in summary.items():
            print(f"  {inst}: {stats['crew']} crew, {stats['flights']} flights, "
                  f"{stats['duties']} duties, {stats['pairings']} pairings")
    else:
        raw = Path(args.instance)
        out = script_dir / args.out
        convert(raw, out)