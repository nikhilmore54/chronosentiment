#!/usr/bin/env python3
"""
CVD-001 Adapter — Sprint 9 Milestone 2
Strategy A: Flight Leg → Coralys Shift (one leg = one shift)

Adapter assumptions (per SCHEMA-MAPPING-v1.0.md Section 5):
  A1  One flight leg = one Coralys shift
  A2  All crew uniformly qualified ("Crew" skill)
  A3  Pairings ignored; legs treated as independent
  A4  Deadhead legs (PAL_) excluded from shifts payload; logged for audit
  A5  Credited hours used as surrogate for historical workload
  A6  Contract type defaults to FullTime
  A7  Home bases retained for reference only; base-return not enforced
  A8  Worker IDs assigned positionally (1–33)

Pipeline stages:
  1  Read archive / locate instance directory
  2  Parse flight legs from day_1.csv … day_31.csv
  3  Build workers from listOfBases.csv + solution_0
  4  Build shifts from parsed legs (A1, A2, A4)
  5  Build historical workloads from creditedHours (A5)
  6  Validate payload (schema check, counts)
  7  Emit Coralys JSON payload
  8  POST to localhost:3001 and record response
"""

import csv
import json
import os
import re
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path

import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

INSTANCE_DIR = Path(__file__).parent.parent / "data" / "cvd001" / "instance1"
OUTPUT_DIR   = Path(__file__).parent.parent / "data" / "cvd001"
API_URL      = "http://localhost:3001/api/schedule"
EPOCH        = datetime(2000, 1, 1)          # day_1.csv date origin
GENERATION_LIMIT = 200
RNG_SEED     = 42
SKILL        = "Crew"                        # A2: uniform skill

# ---------------------------------------------------------------------------
# Stage 1 — Locate instance directory
# ---------------------------------------------------------------------------

def stage1_locate(instance_dir: Path) -> Path:
    print(f"\n[Stage 1] Locating instance directory: {instance_dir}")
    if not instance_dir.exists():
        raise FileNotFoundError(f"Instance directory not found: {instance_dir}")
    day_files = sorted(instance_dir.glob("day_*.csv"))
    print(f"  Found {len(day_files)} day CSV files")
    required = ["listOfBases.csv", "crew_avail_const.csv",
                "credit_constrains.csv", "creditedHours", "solution_0"]
    for f in required:
        path = instance_dir / f
        if not path.exists():
            print(f"  WARNING: expected file missing: {f}")
        else:
            print(f"  OK: {f}")
    return instance_dir

# ---------------------------------------------------------------------------
# Stage 2 — Parse flight legs
# ---------------------------------------------------------------------------

def parse_time(date_str: str, time_str: str) -> datetime:
    """Parse '2000-01-05' + '14:30' into a datetime."""
    return datetime.strptime(f"{date_str.strip()} {time_str.strip()}", "%Y-%m-%d %H:%M")

def to_start_hour(dt: datetime) -> float:
    """Hours elapsed since EPOCH (2000-01-01 00:00)."""
    delta = dt - EPOCH
    return delta.total_seconds() / 3600.0

def stage2_parse_legs(instance_dir: Path) -> list[dict]:
    """
    Returns list of leg records:
      {leg_id, dep_airport, dep_dt, arr_airport, arr_dt,
       start_hour, duration_hours, source_file, source_line,
       is_deadhead}
    """
    print("\n[Stage 2] Parsing flight legs from day CSVs")
    legs = []
    day_files = sorted(instance_dir.glob("day_*.csv"),
                       key=lambda p: int(re.search(r"day_(\d+)", p.name).group(1)))

    deadhead_count = 0
    active_count   = 0

    for day_file in day_files:
        with open(day_file, newline="", encoding="utf-8") as f:
            reader = csv.reader(f)
            for lineno, row in enumerate(reader, start=1):
                # Skip header and blank lines
                if not row or row[0].strip().startswith("#"):
                    continue
                row = [c.strip() for c in row]
                if len(row) < 7:
                    continue
                leg_id, dep_ap, dep_date, dep_time, arr_ap, arr_date, arr_time = row[:7]
                leg_id = leg_id.strip()
                if not leg_id:
                    continue

                is_deadhead = leg_id.startswith("PAL_")

                try:
                    dep_dt = parse_time(dep_date, dep_time)
                    arr_dt = parse_time(arr_date, arr_time)
                except ValueError as e:
                    print(f"  WARNING: parse error in {day_file.name}:{lineno}: {e}")
                    continue

                duration = (arr_dt - dep_dt).total_seconds() / 3600.0
                if duration < 0:
                    # Cross-midnight: add 24h
                    duration += 24.0

                leg = {
                    "leg_id":         leg_id,
                    "dep_airport":    dep_ap,
                    "dep_dt":         dep_dt.isoformat(),
                    "arr_airport":    arr_ap,
                    "arr_dt":         arr_dt.isoformat(),
                    "start_hour":     round(to_start_hour(dep_dt), 4),
                    "duration_hours": round(duration, 4),
                    "source_file":    day_file.name,
                    "source_line":    lineno,
                    "is_deadhead":    is_deadhead,
                }
                legs.append(leg)

                if is_deadhead:
                    deadhead_count += 1
                else:
                    active_count += 1

    print(f"  Total legs parsed:    {len(legs)}")
    print(f"  Active legs (LEG_):   {active_count}")
    print(f"  Deadhead legs (PAL_): {deadhead_count}  [A4: excluded from payload]")
    return legs

# ---------------------------------------------------------------------------
# Stage 3 — Build workers
# ---------------------------------------------------------------------------

def parse_solution_bases(solution_path: Path) -> dict[str, str]:
    """
    Parse solution_0 to extract EMP_ID → base mapping.
    Line format: schedule N EMP_ID (BASE) : ...
    """
    emp_base = {}
    pattern = re.compile(r"schedule\s+\d+\s+(EMP\d+)\s+\((\w+)\)")
    with open(solution_path, encoding="utf-8") as f:
        for line in f:
            m = pattern.search(line)
            if m:
                emp_id, base = m.group(1), m.group(2)
                emp_base[emp_id] = base
    return emp_base

def stage3_build_workers(instance_dir: Path) -> list[dict]:
    """
    Returns list of worker records:
      {worker_id, emp_id, base, skills, source}
    Worker IDs are positional 1–33 (A8).
    """
    print("\n[Stage 3] Building workers")

    # Read base crew counts
    bases_file = instance_dir / "listOfBases.csv"
    base_counts = {}
    with open(bases_file, newline="", encoding="utf-8") as f:
        reader = csv.reader(f)
        next(reader)  # skip header
        for row in reader:
            row = [c.strip() for c in row]
            if len(row) >= 3 and row[1] == "1":
                base_counts[row[0]] = int(row[2])

    print(f"  Bases: {base_counts}")
    total_crew = sum(base_counts.values())
    print(f"  Total crew: {total_crew}")

    # Read EMP→base from solution_0
    solution_path = instance_dir / "solution_0"
    emp_base = parse_solution_bases(solution_path)
    print(f"  EMP IDs found in solution_0: {len(emp_base)}")

    # Build positional worker list (A8)
    # Sort EMP IDs numerically; assign worker_id 1..N
    emp_ids_sorted = sorted(emp_base.keys(), key=lambda e: int(e[3:]))
    workers = []
    for idx, emp_id in enumerate(emp_ids_sorted, start=1):
        base = emp_base.get(emp_id, "UNKNOWN")
        workers.append({
            "worker_id": idx,
            "emp_id":    emp_id,
            "base":      base,       # A7: retained for reference only
            "skills":    [SKILL],    # A2: uniform "Crew" skill
            "source":    "solution_0",
        })

    print(f"  Workers built: {len(workers)}")
    for w in workers[:5]:
        print(f"    worker_id={w['worker_id']}  emp_id={w['emp_id']}  base={w['base']}")
    if len(workers) > 5:
        print(f"    ... ({len(workers) - 5} more)")

    return workers

# ---------------------------------------------------------------------------
# Stage 4 — Build shifts
# ---------------------------------------------------------------------------

def stage4_build_shifts(legs: list[dict]) -> tuple[list[dict], list[dict]]:
    """
    Returns (shifts, excluded_deadheads).
    Shifts: one per active leg (A1, A2, A4).
    Excluded deadheads: logged for audit (A4).
    """
    print("\n[Stage 4] Building shifts from active legs")
    shifts    = []
    excluded  = []
    shift_id  = 1

    for leg in legs:
        if leg["is_deadhead"]:
            excluded.append(leg)
            continue
        shift = {
            "id":             shift_id,
            "leg_id":         leg["leg_id"],       # provenance
            "start_hour":     leg["start_hour"],
            "duration_hours": leg["duration_hours"],
            "required_skill": SKILL,               # A2
            "dep_airport":    leg["dep_airport"],  # provenance
            "arr_airport":    leg["arr_airport"],  # provenance
            "source_file":    leg["source_file"],  # provenance
            "source_line":    leg["source_line"],  # provenance
        }
        shifts.append(shift)
        shift_id += 1

    print(f"  Shifts built:          {len(shifts)}")
    print(f"  Deadheads excluded:    {len(excluded)}  [A4]")
    if shifts:
        print(f"  start_hour range:      {shifts[0]['start_hour']} – {shifts[-1]['start_hour']}")
        print(f"  duration range:        "
              f"{min(s['duration_hours'] for s in shifts):.3f}h – "
              f"{max(s['duration_hours'] for s in shifts):.3f}h")
    return shifts, excluded

# ---------------------------------------------------------------------------
# Stage 5 — Build historical workloads
# ---------------------------------------------------------------------------

def parse_credited_hours(credited_path: Path) -> dict[int, float]:
    """
    Parse creditedHours file.
    Format: 'Schedule N(BASE) :\n---------> credited hours : X.X\n...'
    Returns {schedule_number: credited_hours}.
    """
    schedule_hours = {}
    current_n = None
    pattern_sched = re.compile(r"Schedule\s+(\d+)")
    pattern_hours = re.compile(r"credited hours\s*:\s*([\d.]+)")
    with open(credited_path, encoding="utf-8") as f:
        for line in f:
            m = pattern_sched.search(line)
            if m:
                current_n = int(m.group(1))
            m2 = pattern_hours.search(line)
            if m2 and current_n is not None:
                schedule_hours[current_n] = float(m2.group(1))
    return schedule_hours

def stage5_build_workloads(instance_dir: Path,
                           workers: list[dict]) -> list[dict]:
    """
    Returns historical_workloads list.
    Credited hours used as surrogate for historical workload (A5).
    """
    print("\n[Stage 5] Building historical workloads from creditedHours")
    credited_path = instance_dir / "creditedHours"
    schedule_hours = parse_credited_hours(credited_path)
    print(f"  Schedules found in creditedHours: {len(schedule_hours)}")

    # Map schedule_number → worker_id positionally
    # Schedule 1 → worker 1, Schedule 2 → worker 2, etc.
    workloads = []
    for w in workers:
        wid = w["worker_id"]
        hours = schedule_hours.get(wid, 0.0)
        workloads.append({
            "worker_id": wid,
            "hours":     hours,
            "source":    "creditedHours",  # provenance
            "note":      "surrogate for historical workload (A5)",
        })

    if workloads:
        hours_vals = [wl["hours"] for wl in workloads]
        print(f"  Workload range: {min(hours_vals):.1f}h – {max(hours_vals):.1f}h")
        print(f"  Workload mean:  {sum(hours_vals)/len(hours_vals):.1f}h")

    return workloads

# ---------------------------------------------------------------------------
# Stage 6 — Validate payload
# ---------------------------------------------------------------------------

def stage6_validate(workers: list[dict],
                    shifts: list[dict],
                    workloads: list[dict]) -> bool:
    """
    Basic schema validation before submitting to Coralys.
    Returns True if valid.
    """
    print("\n[Stage 6] Validating payload")
    errors = []

    if not workers:
        errors.append("No workers")
    if not shifts:
        errors.append("No shifts")

    for w in workers:
        if not w.get("worker_id"):
            errors.append(f"Worker missing id: {w}")
        if not w.get("skills"):
            errors.append(f"Worker {w['worker_id']} has no skills")

    for s in shifts:
        if not s.get("id"):
            errors.append(f"Shift missing id: {s}")
        if s.get("duration_hours", 0) <= 0:
            errors.append(f"Shift {s['id']} has non-positive duration: {s['duration_hours']}")
        if s.get("start_hour", -1) < 0:
            errors.append(f"Shift {s['id']} has negative start_hour: {s['start_hour']}")

    worker_ids = {w["worker_id"] for w in workers}
    for wl in workloads:
        if wl["worker_id"] not in worker_ids:
            errors.append(f"Workload references unknown worker_id: {wl['worker_id']}")

    if errors:
        print(f"  VALIDATION FAILED ({len(errors)} errors):")
        for e in errors:
            print(f"    ERROR: {e}")
        return False

    print(f"  Workers:   {len(workers)}  OK")
    print(f"  Shifts:    {len(shifts)}  OK")
    print(f"  Workloads: {len(workloads)}  OK")
    print("  Validation PASSED")
    return True

# ---------------------------------------------------------------------------
# Stage 7 — Emit Coralys JSON payload
# ---------------------------------------------------------------------------

def stage7_build_payload(workers: list[dict],
                         shifts: list[dict],
                         workloads: list[dict]) -> dict:
    """
    Build the Coralys API payload.
    Strips provenance fields from workers/shifts before submission.
    """
    print("\n[Stage 7] Building Coralys JSON payload")

    api_workers = [
        {"id": w["worker_id"], "skills": w["skills"]}
        for w in workers
    ]
    api_shifts = [
        {
            "id":             s["id"],
            "start_hour":     s["start_hour"],
            "duration_hours": s["duration_hours"],
            "required_skill": s["required_skill"],
        }
        for s in shifts
    ]
    api_workloads = [
        {"worker_id": wl["worker_id"], "hours": wl["hours"]}
        for wl in workloads
    ]

    payload = {
        "workers":             api_workers,
        "shifts":              api_shifts,
        "historical_workloads": api_workloads,
        "rng_seed":            RNG_SEED,
        "generation_limit":    GENERATION_LIMIT,
    }

    print(f"  Payload workers:   {len(api_workers)}")
    print(f"  Payload shifts:    {len(api_shifts)}")
    print(f"  Payload workloads: {len(api_workloads)}")
    print(f"  rng_seed:          {RNG_SEED}")
    print(f"  generation_limit:  {GENERATION_LIMIT}")

    return payload

# ---------------------------------------------------------------------------
# Stage 8 — POST to Coralys and record response
# ---------------------------------------------------------------------------

def stage8_submit(payload: dict,
                  workers: list[dict],
                  shifts: list[dict],
                  excluded: list[dict],
                  workloads: list[dict]) -> dict:
    """
    POST payload to Coralys API and record full result.
    """
    print(f"\n[Stage 8] Submitting to Coralys API: {API_URL}")
    t0 = time.time()
    try:
        resp = requests.post(API_URL, json=payload, timeout=300)
        elapsed = time.time() - t0
        resp.raise_for_status()
        result = resp.json()
    except requests.exceptions.ConnectionError:
        print(f"  ERROR: Cannot connect to {API_URL}")
        print("  Is the Coralys API running? Start with: cargo run --bin coralys-api")
        return {}
    except requests.exceptions.Timeout:
        print("  ERROR: Request timed out after 300s")
        return {}
    except Exception as e:
        print(f"  ERROR: {e}")
        return {}

    print(f"  HTTP status:  {resp.status_code}")
    print(f"  Elapsed:      {elapsed:.2f}s")

    # Extract metrics
    metrics  = result.get("metrics", {})
    schedule = result.get("schedule", {})
    constraints = result.get("constraint_report", {})

    fitness  = metrics.get("fitness", metrics.get("total_fitness", "N/A"))
    sc1      = metrics.get("fairness_penalty", metrics.get("sc1", "N/A"))
    sc2      = metrics.get("fatigue_penalty",  metrics.get("sc2", "N/A"))
    hc1      = constraints.get("hc1_violations", constraints.get("skill_violations", "N/A"))
    hc2      = constraints.get("hc2_violations", constraints.get("coverage_violations", "N/A"))
    hc3      = constraints.get("hc3_violations", constraints.get("hours_violations", "N/A"))
    rest_v   = constraints.get("rest_violations", "N/A")

    print(f"\n  === Optimization Results ===")
    print(f"  Fitness:      {fitness}")
    print(f"  SC1 (fair):   {sc1}")
    print(f"  SC2 (fatigue):{sc2}")
    print(f"  HC1 violations: {hc1}")
    print(f"  HC2 violations: {hc2}")
    print(f"  HC3 violations: {hc3}")
    print(f"  Rest violations:{rest_v}")

    # Compute credited hours from schedule
    if isinstance(schedule, dict):
        assigned_shifts = set(schedule.values())
        credited = sum(
            s["duration_hours"]
            for s in shifts
            if s["id"] in assigned_shifts
        )
        print(f"\n  Assigned shifts:  {len(schedule)}")
        print(f"  Total credited h: {credited:.2f}h")

    # Build full result record
    record = {
        "meta": {
            "timestamp":        datetime.utcnow().isoformat() + "Z",
            "instance":         "instance1",
            "strategy":         "A",
            "adapter_version":  "1.0",
            "rng_seed":         RNG_SEED,
            "generation_limit": GENERATION_LIMIT,
            "elapsed_seconds":  round(elapsed, 3),
        },
        "payload_summary": {
            "workers":            len(payload["workers"]),
            "shifts":             len(payload["shifts"]),
            "workloads":          len(payload["historical_workloads"]),
            "deadheads_excluded": len(excluded),
        },
        "api_response": result,
        "adapter_assumptions": [
            "A1: one flight leg = one Coralys shift",
            "A2: all crew uniformly qualified (Crew skill)",
            "A3: pairings ignored",
            "A4: deadhead legs excluded from payload",
            "A5: credited hours used as workload surrogate",
            "A6: contract type defaults to FullTime",
            "A7: home bases retained for reference only",
            "A8: worker IDs assigned positionally (1-33)",
        ],
    }

    # Save result
    out_path = OUTPUT_DIR / "CVD-001-INSTANCE1-RESULT-v1.0.json"
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(record, f, indent=2)
    print(f"\n  Result saved: {out_path}")

    # Save deadhead audit log
    audit_path = OUTPUT_DIR / "CVD-001-INSTANCE1-DEADHEAD-AUDIT.json"
    with open(audit_path, "w", encoding="utf-8") as f:
        json.dump(excluded, f, indent=2)
    print(f"  Deadhead audit: {audit_path}  ({len(excluded)} legs)")

    return record

# ---------------------------------------------------------------------------
# Dry-run mode (no API call)
# ---------------------------------------------------------------------------

def dry_run(workers, shifts, excluded, workloads, payload):
    """Print payload summary without calling the API."""
    print("\n[DRY RUN] Payload ready — not submitting to API")
    out_path = OUTPUT_DIR / "CVD-001-INSTANCE1-PAYLOAD-DRY-RUN.json"
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    print(f"  Payload saved: {out_path}")

    audit_path = OUTPUT_DIR / "CVD-001-INSTANCE1-DEADHEAD-AUDIT.json"
    with open(audit_path, "w", encoding="utf-8") as f:
        json.dump(excluded, f, indent=2)
    print(f"  Deadhead audit: {audit_path}  ({len(excluded)} legs)")
    print("\n  To submit: run without --dry-run flag")

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    dry = "--dry-run" in sys.argv
    print("=" * 60)
    print("CVD-001 Adapter — Strategy A (Flight Leg → Shift)")
    print("=" * 60)

    instance_dir = stage1_locate(INSTANCE_DIR)
    legs         = stage2_parse_legs(instance_dir)
    workers      = stage3_build_workers(instance_dir)
    shifts, excl = stage4_build_shifts(legs)
    workloads    = stage5_build_workloads(instance_dir, workers)
    valid        = stage6_validate(workers, shifts, workloads)

    if not valid:
        print("\nAborting: payload validation failed.")
        sys.exit(1)

    payload = stage7_build_payload(workers, shifts, workloads)

    if dry:
        dry_run(workers, shifts, excl, workloads, payload)
    else:
        stage8_submit(payload, workers, shifts, excl, workloads)

    print("\n[Done]")

if __name__ == "__main__":
    main()