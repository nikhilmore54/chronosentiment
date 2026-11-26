#!/usr/bin/env python3
"""Pre-flight integrity checks for CVD-001 dry-run payload."""
import json
import statistics
from pathlib import Path

payload_path = Path(__file__).parent.parent / "data" / "cvd001" / "CVD-001-INSTANCE1-PAYLOAD-DRY-RUN.json"

with open(payload_path) as f:
    p = json.load(f)

issues = []

# 1. Workload distribution
hours = [w["hours"] for w in p["historical_workloads"]]
hours_sorted = sorted(hours)
print("=== Workload Distribution ===")
print(f"  min:    {min(hours):.4f}h")
print(f"  max:    {max(hours):.4f}h")
print(f"  mean:   {sum(hours)/len(hours):.4f}h")
print(f"  median: {hours_sorted[len(hours)//2]:.4f}h")
print(f"  stddev: {statistics.stdev(hours):.4f}h")

# 2. Horizon check (744h = 31 days)
violations = [s for s in p["shifts"] if s["start_hour"] + s["duration_hours"] > 744]
print(f"\n=== Horizon Check (744h) ===")
print(f"  Shifts exceeding horizon: {len(violations)}")
if violations:
    for v in violations[:3]:
        end = v["start_hour"] + v["duration_hours"]
        print(f"    id={v['id']} start={v['start_hour']} dur={v['duration_hours']} end={end:.3f}")
    issues.append(f"{len(violations)} shifts exceed 744h horizon")

# 3. Skill coverage
worker_skills = {s for w in p["workers"] for s in w["skills"]}
required_skills = {s["required_skill"] for s in p["shifts"]}
covered = required_skills <= worker_skills
print(f"\n=== Skill Coverage ===")
print(f"  Worker skills:   {worker_skills}")
print(f"  Required skills: {required_skills}")
print(f"  All covered:     {covered}")
if not covered:
    issues.append(f"Uncovered skills: {required_skills - worker_skills}")

# 4. Unique IDs
worker_ids = [w["id"] for w in p["workers"]]
shift_ids  = [s["id"] for s in p["shifts"]]
w_unique = len(set(worker_ids)) == len(worker_ids)
s_unique = len(set(shift_ids)) == len(shift_ids)
print(f"\n=== ID Uniqueness ===")
print(f"  Worker IDs unique: {w_unique}  ({len(worker_ids)} workers)")
print(f"  Shift IDs unique:  {s_unique}  ({len(shift_ids)} shifts)")
if not w_unique:
    issues.append("Duplicate worker IDs")
if not s_unique:
    issues.append("Duplicate shift IDs")

# 5. Zero/negative duration
zero_dur = [s for s in p["shifts"] if s["duration_hours"] <= 0]
print(f"\n=== Zero/Negative Duration ===")
print(f"  Shifts with duration <= 0: {len(zero_dur)}")
if zero_dur:
    issues.append(f"{len(zero_dur)} shifts with non-positive duration")

# Summary
print()
if not issues:
    print("=== ALL CHECKS PASSED ===")
else:
    print(f"=== {len(issues)} ISSUE(S) FOUND ===")
    for i in issues:
        print(f"  - {i}")