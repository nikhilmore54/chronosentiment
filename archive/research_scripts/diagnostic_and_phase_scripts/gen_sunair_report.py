#!/usr/bin/env python3
"""
S1-05: Generate sunair_report.json — enriched customer-facing schedule report.

Loads fixtures/demo/sunair_schedule.json (raw optimizer output) and
fixtures/demo/sunair_demo.json (scenario definition), then produces
fixtures/demo/sunair_report.json with human-readable KPIs, skill coverage
breakdown, per-worker workload summary, and full assignment list.

Usage:
    python3 scripts/gen_sunair_report.py
"""

import json
import sys
from pathlib import Path

SCHEDULE_PATH = Path("fixtures/demo/sunair_schedule.json")
SCENARIO_PATH = Path("fixtures/demo/sunair_demo.json")
OUTPUT_PATH   = Path("fixtures/demo/sunair_report.json")

GENERATED_AT  = "2026-07-22T22:52:00+05:30"


def main() -> None:
    # ── Load inputs ──────────────────────────────────────────────────────────
    if not SCHEDULE_PATH.exists():
        sys.exit(f"ERROR: {SCHEDULE_PATH} not found. Run the CLI demo first.")
    if not SCENARIO_PATH.exists():
        sys.exit(f"ERROR: {SCENARIO_PATH} not found.")

    schedule = json.loads(SCHEDULE_PATH.read_text())
    scenario = json.loads(SCENARIO_PATH.read_text())

    # ── Build lookups ────────────────────────────────────────────────────────
    shifts  = {s["id"]: s for s in scenario["shifts"]}
    workers = {w["id"]: w for w in scenario["workers"]}

    # ── Per-worker assignments and hours ─────────────────────────────────────
    # Keys: worker_id (int) → {shifts: [int], total_hours: int}
    worker_data: dict[int, dict] = {}
    for shift_id_str, worker_id in schedule["assignments"].items():
        shift_id = int(shift_id_str)
        shift    = shifts[shift_id]
        if worker_id not in worker_data:
            worker_data[worker_id] = {"shifts": [], "total_hours": 0}
        worker_data[worker_id]["shifts"].append(shift_id)
        worker_data[worker_id]["total_hours"] += shift["duration_hours"]

    # ── Skill coverage ───────────────────────────────────────────────────────
    skill_shifts: dict[str, dict] = {}
    for s in scenario["shifts"]:
        skill = s["required_skill"]
        skill_shifts.setdefault(skill, {"required": 0, "covered": 0})
        skill_shifts[skill]["required"] += 1

    for shift_id_str in schedule["assignments"]:
        shift = shifts[int(shift_id_str)]
        skill_shifts[shift["required_skill"]]["covered"] += 1

    # ── Workload balance stats ───────────────────────────────────────────────
    hours_list = [v["total_hours"] for v in worker_data.values()]
    mean_h = round(sum(hours_list) / len(hours_list), 1) if hours_list else 0.0

    # ── Worker summary rows ──────────────────────────────────────────────────
    worker_summary = []
    for wid, data in sorted(worker_data.items()):
        raw_skills = workers[wid]["skills"]
        # Skills may be stored as plain strings or as {"0": "SkillName"} dicts
        skills = [
            s["0"] if isinstance(s, dict) else s
            for s in raw_skills
        ]
        worker_summary.append({
            "worker_id":       wid,
            "skills":          skills,
            "shifts_assigned": len(data["shifts"]),
            "total_hours":     data["total_hours"],
        })

    # ── Assemble report ──────────────────────────────────────────────────────
    report = {
        "report_type":    "UltraCrew Schedule Report",
        "schema_version": "1.0",
        "generated":      GENERATED_AT,
        "scenario": {
            "name":                    "SunAir Demo",
            "planning_horizon_hours":  168,
            "total_workers":           len(scenario["workers"]),
            "total_shifts":            len(scenario["shifts"]),
            "rng_seed":                scenario["rng_seed"],
            "generation_limit":        scenario["generation_limit"],
        },
        "kpis": {
            "coverage_pct":    round(
                len(schedule["assignments"]) / len(scenario["shifts"]) * 100, 1
            ),
            "shifts_assigned": len(schedule["assignments"]),
            "shifts_total":    len(scenario["shifts"]),
            "hard_violations": schedule["hard_violations"],
            "rest_violations": schedule["rest_violations"],
            "fitness_score":   round(schedule["fitness"], 4),
            "fairness_penalty": round(schedule["fairness_penalty"], 4),
            "fatigue_penalty":  round(schedule["fatigue_penalty"], 4),
            "workload_balance": {
                "mean_hours_per_worker": mean_h,
                "min_hours_per_worker":  min(hours_list),
                "max_hours_per_worker":  max(hours_list),
            },
        },
        "skill_coverage": {
            skill: {
                "shifts_required": v["required"],
                "shifts_covered":  v["covered"],
                "coverage_pct":    round(v["covered"] / v["required"] * 100, 1),
            }
            for skill, v in sorted(skill_shifts.items())
        },
        "worker_summary": worker_summary,
        "assignments": [
            {"shift_id": int(k), "worker_id": v}
            for k, v in sorted(
                schedule["assignments"].items(), key=lambda x: int(x[0])
            )
        ],
    }

    # ── Write output ─────────────────────────────────────────────────────────
    OUTPUT_PATH.write_text(json.dumps(report, indent=2))
    print(f"Report written to {OUTPUT_PATH}")
    print(f"KPIs: {json.dumps(report['kpis'], indent=2)}")


if __name__ == "__main__":
    main()
