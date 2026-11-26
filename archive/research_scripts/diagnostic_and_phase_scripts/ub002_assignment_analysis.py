"""
H7 — SC2 Influence Stability
Sprint 8 / S8-OBJECTIVE-CHARACTERIZATION

Runs UB-002 across 20 seeds × 4 weeks to determine whether the SC2-driven
assignment distribution (HIGH < MEDIUM < LOW shifts/week) is stable across
optimizer seeds or varies significantly.

Usage:
    python3 scripts/ub002_assignment_analysis.py

Output:
    benchmarks/ultracrew/UB-002-H7-ASSIGNMENT-STABILITY-v1.0.json
"""

import json
import statistics
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).parent.parent
UB002_PATH = ROOT / "benchmarks/ultracrew/UB-002-v1.0.json"
OUT_PATH = ROOT / "benchmarks/ultracrew/UB-002-H7-ASSIGNMENT-STABILITY-v1.0.json"
API_URL = "http://localhost:3001/api/schedule"

SEEDS = list(range(1, 21))
GENERATION_LIMIT = 50

# Worker group membership (from UB-002-v1.0 design)
HIGH_WORKERS = {1, 2, 3, 5}
MEDIUM_WORKERS = {4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 18}
LOW_WORKERS = {16, 17, 19, 20}


def load_ub002():
    with open(UB002_PATH) as f:
        return json.load(f)


SHIFT_TYPES = [("Morning", 7, 8), ("Evening", 15, 8), ("Night", 23, 8)]


def build_shifts(ub):
    shifts = []
    sid = 1
    for day in range(7):
        dt = "weekend" if day in [5, 6] else "weekday"
        cov = ub["coverage_requirements"][dt]
        for sn, sh, dur in SHIFT_TYPES:
            for skill, cnt in cov[sn].items():
                for _ in range(cnt):
                    shifts.append({
                        "id": sid,
                        "start_hour": day * 24 + sh,
                        "duration_hours": dur,
                        "required_skill": skill,
                    })
                    sid += 1
    return shifts


def build_payload(ub, seed):
    workers = [{"id": i + 1, "skills": s["skills"]} for i, s in enumerate(ub["staff"])]
    shifts = build_shifts(ub)
    return {
        "workers": workers,
        "shifts": shifts,
        "historical_workloads": ub.get("historical_workloads"),
        "rng_seed": seed,
        "generation_limit": GENERATION_LIMIT,
    }


def call_api(payload):
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        API_URL,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read())


def count_group_shifts(schedule, group_ids):
    """Count total shifts assigned to workers in group_ids.

    schedule may be:
      - a dict {shift_id: worker_id} (API returns this format)
      - a list of dicts [{worker_id: int, ...}]
      - a list of strings ["worker_id:shift_id"]
    """
    count = 0
    items = schedule.values() if isinstance(schedule, dict) else schedule
    for item in items:
        if isinstance(item, dict):
            worker_id = item.get("worker_id") or item.get("staff_id")
        elif isinstance(item, (int, float)):
            worker_id = int(item)
        elif isinstance(item, str):
            try:
                worker_id = int(item.split(":")[0])
            except (ValueError, IndexError):
                worker_id = None
        else:
            worker_id = None
        if worker_id in group_ids:
            count += 1
    return count


def extract_result(response, seed, week_index):
    metrics = response.get("metrics", {})
    cr = response.get("constraint_report", {})
    schedule = response.get("schedule", [])

    high_shifts = count_group_shifts(schedule, HIGH_WORKERS)
    medium_shifts = count_group_shifts(schedule, MEDIUM_WORKERS)
    low_shifts = count_group_shifts(schedule, LOW_WORKERS)

    # Normalize to per-worker averages
    high_avg = high_shifts / len(HIGH_WORKERS)
    medium_avg = medium_shifts / len(MEDIUM_WORKERS)
    low_avg = low_shifts / len(LOW_WORKERS)

    return {
        "seed": seed,
        "week": week_index + 1,
        "fitness": metrics.get("fitness", 0.0),
        "sc1": metrics.get("fairness_penalty", 0.0),
        "sc2": metrics.get("fatigue_penalty", 0.0),
        "hc_total": (
            cr.get("hc1_violations", cr.get("skill_violations", 0))
            + cr.get("hc2_violations", cr.get("coverage_violations", 0))
            + cr.get("hc3_violations", cr.get("hours_violations", 0))
            + cr.get("rest_violations", 0)
        ),
        "high_shifts_total": high_shifts,
        "medium_shifts_total": medium_shifts,
        "low_shifts_total": low_shifts,
        "high_avg": round(high_avg, 3),
        "medium_avg": round(medium_avg, 3),
        "low_avg": round(low_avg, 3),
        "ordering_correct": high_avg < medium_avg < low_avg,
    }


def main():
    ub = load_ub002()
    n_weeks = 4  # UB-002 runs 4 identical weeks (same coverage_requirements)
    print(f"H7 — SC2 influence stability probe")
    print(f"Benchmark: UB-002-v1.0  Weeks: 1–{n_weeks}  Seeds: {SEEDS[0]}–{SEEDS[-1]}  Gens: {GENERATION_LIMIT}")
    print()

    all_results = []
    for seed in SEEDS:
        seed_results = []
        for week_index in range(n_weeks):
            payload = build_payload(ub, seed * 100 + week_index)
            t0 = time.time()
            try:
                response = call_api(payload)
                elapsed = time.time() - t0
                r = extract_result(response, seed, week_index)
                r["runtime_s"] = round(elapsed, 2)
                seed_results.append(r)
                all_results.append(r)
                print(
                    f"  seed={seed:2d} wk={week_index+1}  "
                    f"HIGH={r['high_avg']:.2f}  MED={r['medium_avg']:.2f}  LOW={r['low_avg']:.2f}  "
                    f"order={'✓' if r['ordering_correct'] else '✗'}  "
                    f"SC1={r['sc1']:.1f}  SC2={r['sc2']:.1f}"
                )
            except Exception as e:
                print(f"  seed={seed:2d} wk={week_index+1}  ERROR: {e}")
                all_results.append({"seed": seed, "week": week_index + 1, "error": str(e)})

    # Summary
    valid = [r for r in all_results if "ordering_correct" in r and r["hc_total"] == 0]
    ordering_correct_count = sum(1 for r in valid if r["ordering_correct"])
    ordering_total = len(valid)

    high_avgs = [r["high_avg"] for r in valid]
    medium_avgs = [r["medium_avg"] for r in valid]
    low_avgs = [r["low_avg"] for r in valid]

    summary = {}
    if valid:
        summary = {
            "n_valid": ordering_total,
            "n_total": len(SEEDS) * n_weeks,
            "ordering_correct_count": ordering_correct_count,
            "ordering_correct_pct": round(100 * ordering_correct_count / ordering_total, 1),
            "high_avg_mean": round(statistics.mean(high_avgs), 3),
            "high_avg_stdev": round(statistics.stdev(high_avgs) if len(high_avgs) > 1 else 0.0, 3),
            "medium_avg_mean": round(statistics.mean(medium_avgs), 3),
            "medium_avg_stdev": round(statistics.stdev(medium_avgs) if len(medium_avgs) > 1 else 0.0, 3),
            "low_avg_mean": round(statistics.mean(low_avgs), 3),
            "low_avg_stdev": round(statistics.stdev(low_avgs) if len(low_avgs) > 1 else 0.0, 3),
        }

        # H7 verdict
        if summary["ordering_correct_pct"] >= 90:
            verdict = "CONFIRMED"
            interpretation = (
                f"HIGH < MEDIUM < LOW holds in {summary['ordering_correct_pct']}% of valid runs. "
                "SC2 influence is stable across seeds."
            )
        else:
            verdict = "OPEN"
            interpretation = (
                f"HIGH < MEDIUM < LOW holds in only {summary['ordering_correct_pct']}% of valid runs. "
                "SC2 influence is not consistently stable. Investigate."
            )
        summary["h7_verdict"] = verdict
        summary["h7_interpretation"] = interpretation

        print()
        print("Summary:")
        print(f"  Ordering correct: {ordering_correct_count}/{ordering_total} ({summary['ordering_correct_pct']}%)")
        print(f"  HIGH  mean={summary['high_avg_mean']}  stdev={summary['high_avg_stdev']}")
        print(f"  MED   mean={summary['medium_avg_mean']}  stdev={summary['medium_avg_stdev']}")
        print(f"  LOW   mean={summary['low_avg_mean']}  stdev={summary['low_avg_stdev']}")
        print(f"\nH7 verdict: {verdict}")
        print(f"  {interpretation}")

    output = {
        "benchmark": "UB-002-v1.0",
        "sprint": "S8",
        "hypothesis": "H7",
        "seeds": SEEDS,
        "generation_limit": GENERATION_LIMIT,
        "group_definitions": {
            "HIGH": sorted(HIGH_WORKERS),
            "MEDIUM": sorted(MEDIUM_WORKERS),
            "LOW": sorted(LOW_WORKERS),
        },
        "results": all_results,
        "summary": summary,
    }

    with open(OUT_PATH, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nResults written to {OUT_PATH.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
