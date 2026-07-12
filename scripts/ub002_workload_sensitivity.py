"""
H8 — Workload Sensitivity
Sprint 8 / S8-OBJECTIVE-CHARACTERIZATION

Tests how sensitive Coralys is to different historical workload profiles by
varying only the historical_workloads field of UB-002. All other benchmark
fields (staff, shifts, coverage, constraints) remain frozen.

Profiles tested:
  - baseline:  UB-002 original (HIGH=0.994, MEDIUM=0.813, LOW=0.506)
  - balanced:  all workers at 0.75 (uniform moderate fatigue)
  - light:     all workers at 0.25 (uniform low fatigue)
  - heavy:     all workers at 0.95 (uniform high fatigue)
  - bimodal:   HIGH=0.95, MEDIUM=0.50, LOW=0.10 (wide spread)
  - uniform:   all workers at 0.50 (uniform mid fatigue)

Usage:
    python3 scripts/ub002_workload_sensitivity.py

Output:
    benchmarks/ultracrew/UB-002-H8-WORKLOAD-SENSITIVITY-v1.0.json
"""

import json
import statistics
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).parent.parent
UB002_PATH = ROOT / "benchmarks/ultracrew/UB-002-v1.0.json"
OUT_PATH = ROOT / "benchmarks/ultracrew/UB-002-H8-WORKLOAD-SENSITIVITY-v1.0.json"
API_URL = "http://localhost:3001/api/schedule"

SEEDS = [42, 43, 44]   # 3 seeds per profile — sufficient for sensitivity probe
GENERATION_LIMIT = 50
WEEK_INDEX = 0          # Week 1 — most stable week in S7 first run

# Worker group membership (from UB-002-v1.0 design)
HIGH_WORKERS = {1, 2, 3, 5}
MEDIUM_WORKERS = {4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 18}
LOW_WORKERS = {16, 17, 19, 20}
ALL_WORKERS = HIGH_WORKERS | MEDIUM_WORKERS | LOW_WORKERS


def load_ub002():
    with open(UB002_PATH) as f:
        return json.load(f)


def make_workload(worker_ids, fatigue_value, weeks=4, hours_per_week=32):
    """Build a historical_workloads dict with uniform fatigue for all workers."""
    # fatigue = mean(buffer) / 40.0, so mean(buffer) = fatigue * 40.0
    target_hours = fatigue_value * 40.0
    # Use 4 weeks of history all at target_hours
    return {
        str(wid): [round(target_hours)] * weeks
        for wid in worker_ids
    }


def make_bimodal_workload(high_ids, medium_ids, low_ids,
                          high_fatigue, medium_fatigue, low_fatigue, weeks=4):
    result = {}
    for wid in high_ids:
        h = round(high_fatigue * 40.0)
        result[str(wid)] = [h] * weeks
    for wid in medium_ids:
        h = round(medium_fatigue * 40.0)
        result[str(wid)] = [h] * weeks
    for wid in low_ids:
        h = round(low_fatigue * 40.0)
        result[str(wid)] = [h] * weeks
    return result


def build_profiles(ub):
    original_workloads = ub.get("historical_workloads")
    return {
        "baseline": original_workloads,
        "balanced": make_workload(ALL_WORKERS, 0.75),
        "light": make_workload(ALL_WORKERS, 0.25),
        "heavy": make_workload(ALL_WORKERS, 0.95),
        "bimodal": make_bimodal_workload(
            HIGH_WORKERS, MEDIUM_WORKERS, LOW_WORKERS,
            high_fatigue=0.95, medium_fatigue=0.50, low_fatigue=0.10
        ),
        "uniform": make_workload(ALL_WORKERS, 0.50),
    }


def build_payload(ub, week_index, seed, workloads):
    week = ub["weeks"][week_index]
    return {
        "staff": ub["staff"],
        "shifts": week["shifts"],
        "coverage_requirements": ub["coverage_requirements"],
        "constraints": ub["constraints"],
        "historical_workloads": workloads,
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
    count = 0
    for assignment in schedule:
        worker_id = assignment.get("worker_id") or assignment.get("staff_id")
        if worker_id in group_ids:
            count += 1
    return count


def extract_result(response, profile, seed):
    metrics = response.get("metrics", {})
    cr = response.get("constraint_report", {})
    schedule = response.get("schedule", [])

    high_shifts = count_group_shifts(schedule, HIGH_WORKERS)
    medium_shifts = count_group_shifts(schedule, MEDIUM_WORKERS)
    low_shifts = count_group_shifts(schedule, LOW_WORKERS)

    high_avg = high_shifts / len(HIGH_WORKERS)
    medium_avg = medium_shifts / len(MEDIUM_WORKERS)
    low_avg = low_shifts / len(LOW_WORKERS)

    return {
        "profile": profile,
        "seed": seed,
        "fitness": metrics.get("fitness", 0.0),
        "sc1": metrics.get("fairness_penalty", 0.0),
        "sc2": metrics.get("fatigue_penalty", 0.0),
        "hc_total": (
            cr.get("skill_violations", 0)
            + cr.get("coverage_violations", 0)
            + cr.get("hours_violations", 0)
            + cr.get("rest_violations", 0)
        ),
        "high_avg": round(high_avg, 3),
        "medium_avg": round(medium_avg, 3),
        "low_avg": round(low_avg, 3),
    }


def main():
    ub = load_ub002()
    profiles = build_profiles(ub)

    print(f"H8 — Workload sensitivity probe")
    print(f"Benchmark: UB-002-v1.0  Week: {WEEK_INDEX+1}  "
          f"Seeds: {SEEDS}  Gens: {GENERATION_LIMIT}")
    print(f"Profiles: {list(profiles.keys())}")
    print()

    all_results = []
    profile_summaries = {}

    for profile_name, workloads in profiles.items():
        print(f"  Profile: {profile_name}")
        profile_results = []
        for seed in SEEDS:
            payload = build_payload(ub, WEEK_INDEX, seed, workloads)
            t0 = time.time()
            try:
                response = call_api(payload)
                elapsed = time.time() - t0
                r = extract_result(response, profile_name, seed)
                r["runtime_s"] = round(elapsed, 2)
                profile_results.append(r)
                all_results.append(r)
                print(
                    f"    seed={seed}  fitness={r['fitness']:.1f}  "
                    f"SC1={r['sc1']:.1f}  SC2={r['sc2']:.1f}  "
                    f"HIGH={r['high_avg']:.2f}  MED={r['medium_avg']:.2f}  LOW={r['low_avg']:.2f}"
                )
            except Exception as e:
                print(f"    seed={seed}  ERROR: {e}")
                all_results.append({"profile": profile_name, "seed": seed, "error": str(e)})

        valid = [r for r in profile_results if "fitness" in r and r["hc_total"] == 0]
        if valid:
            profile_summaries[profile_name] = {
                "n_valid": len(valid),
                "fitness_mean": round(statistics.mean(r["fitness"] for r in valid), 2),
                "sc1_mean": round(statistics.mean(r["sc1"] for r in valid), 2),
                "sc2_mean": round(statistics.mean(r["sc2"] for r in valid), 2),
                "high_avg_mean": round(statistics.mean(r["high_avg"] for r in valid), 3),
                "medium_avg_mean": round(statistics.mean(r["medium_avg"] for r in valid), 3),
                "low_avg_mean": round(statistics.mean(r["low_avg"] for r in valid), 3),
            }
        print()

    # H8 verdict: compare SC2 range across profiles
    print("Profile summary:")
    print(f"  {'Profile':<12}  {'Fitness':>8}  {'SC1':>7}  {'SC2':>7}  "
          f"{'HIGH':>6}  {'MED':>6}  {'LOW':>6}")
    for pname, ps in profile_summaries.items():
        print(f"  {pname:<12}  {ps['fitness_mean']:>8.1f}  {ps['sc1_mean']:>7.1f}  "
              f"{ps['sc2_mean']:>7.1f}  {ps['high_avg_mean']:>6.2f}  "
              f"{ps['medium_avg_mean']:>6.2f}  {ps['low_avg_mean']:>6.2f}")

    sc2_values = [ps["sc2_mean"] for ps in profile_summaries.values()]
    sc2_range = max(sc2_values) - min(sc2_values) if sc2_values else 0

    if sc2_range > 200:
        verdict = "SENSITIVE"
        interpretation = (
            f"SC2 varies by {sc2_range:.1f} across profiles. "
            "Assignment distribution is sensitive to workload profile."
        )
    else:
        verdict = "STABLE"
        interpretation = (
            f"SC2 varies by only {sc2_range:.1f} across profiles. "
            "Assignment distribution is relatively insensitive to workload profile."
        )

    print(f"\nH8 verdict: {verdict}")
    print(f"  {interpretation}")

    output = {
        "benchmark": "UB-002-v1.0",
        "sprint": "S8",
        "hypothesis": "H8",
        "week": WEEK_INDEX + 1,
        "seeds": SEEDS,
        "generation_limit": GENERATION_LIMIT,
        "profiles_tested": list(profiles.keys()),
        "group_definitions": {
            "HIGH": sorted(HIGH_WORKERS),
            "MEDIUM": sorted(MEDIUM_WORKERS),
            "LOW": sorted(LOW_WORKERS),
        },
        "results": all_results,
        "profile_summaries": profile_summaries,
        "h8_verdict": verdict,
        "h8_interpretation": interpretation,
    }

    with open(OUT_PATH, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nResults written to {OUT_PATH.relative_to(ROOT)}")


if __name__ == "__main__":
    main()