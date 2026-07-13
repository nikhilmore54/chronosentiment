"""
H6 — Week 2 Reproducibility
Sprint 8 / S8-OBJECTIVE-CHARACTERIZATION

Runs UB-002 Week 2 across 20 seeds to determine whether the SC1 spike
observed in the Sprint 7 first run is stochastic or structural.

Usage:
    python3 scripts/ub002_seed_stability.py

Output:
    benchmarks/ultracrew/UB-002-H6-WEEK2-STABILITY-v1.0.json
"""

import json
import statistics
import time
import urllib.request
import urllib.error
from pathlib import Path

ROOT = Path(__file__).parent.parent
UB002_PATH = ROOT / "benchmarks/ultracrew/UB-002-v1.0.json"
OUT_PATH = ROOT / "benchmarks/ultracrew/UB-002-H6-WEEK2-STABILITY-v1.0.json"
API_URL = "http://localhost:3001/api/schedule"

SEEDS = list(range(1, 21))   # seeds 1–20
WEEK_INDEX = 1               # Week 2 (0-indexed)
GENERATION_LIMIT = 50


def load_ub002():
    with open(UB002_PATH) as f:
        return json.load(f)


SHIFT_TYPES = [("Morning", 7, 8), ("Evening", 15, 8), ("Night", 23, 8)]


def build_shifts(ub):
    """Build a week's shift list from coverage_requirements (same shifts every week)."""
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


def extract_metrics(response):
    metrics = response.get("metrics", {})
    cr = response.get("constraint_report", {})
    return {
        "fitness": metrics.get("fitness", 0.0),
        "sc1": metrics.get("fairness_penalty", 0.0),
        "sc2": metrics.get("fatigue_penalty", 0.0),
        "hc1": cr.get("hc1_violations", cr.get("skill_violations", 0)),
        "hc2": cr.get("hc2_violations", cr.get("coverage_violations", 0)),
        "hc3": cr.get("hc3_violations", cr.get("hours_violations", 0)),
        "rest": cr.get("rest_violations", 0),
    }


def main():
    ub = load_ub002()
    print(f"H6 — Week 2 seed stability probe")
    print(f"Benchmark: UB-002-v1.0  Week: 2  Seeds: {SEEDS[0]}–{SEEDS[-1]}  Gens: {GENERATION_LIMIT}")
    print()

    results = []
    for seed in SEEDS:
        payload = build_payload(ub, seed)
        t0 = time.time()
        try:
            response = call_api(payload)
            elapsed = time.time() - t0
            m = extract_metrics(response)
            m["seed"] = seed
            m["runtime_s"] = round(elapsed, 2)
            valid = (m["hc1"] + m["hc2"] + m["hc3"] + m["rest"]) == 0
            m["valid"] = valid
            results.append(m)
            print(
                f"  seed={seed:2d}  fitness={m['fitness']:8.1f}  "
                f"SC1={m['sc1']:6.1f}  SC2={m['sc2']:7.1f}  "
                f"HC={m['hc1']+m['hc2']+m['hc3']+m['rest']}  "
                f"valid={valid}  {elapsed:.1f}s"
            )
        except Exception as e:
            print(f"  seed={seed:2d}  ERROR: {e}")
            results.append({"seed": seed, "error": str(e)})

    # Summary statistics
    valid_results = [r for r in results if "fitness" in r and r.get("valid")]
    sc1_values = [r["sc1"] for r in valid_results]
    sc2_values = [r["sc2"] for r in valid_results]
    fitness_values = [r["fitness"] for r in valid_results]

    summary = {}
    if sc1_values:
        summary = {
            "n_valid": len(valid_results),
            "n_total": len(SEEDS),
            "sc1_mean": round(statistics.mean(sc1_values), 2),
            "sc1_stdev": round(statistics.stdev(sc1_values) if len(sc1_values) > 1 else 0.0, 2),
            "sc1_min": round(min(sc1_values), 2),
            "sc1_max": round(max(sc1_values), 2),
            "sc2_mean": round(statistics.mean(sc2_values), 2),
            "sc2_stdev": round(statistics.stdev(sc2_values) if len(sc2_values) > 1 else 0.0, 2),
            "fitness_mean": round(statistics.mean(fitness_values), 2),
            "fitness_stdev": round(statistics.stdev(fitness_values) if len(fitness_values) > 1 else 0.0, 2),
            "fitness_min": round(min(fitness_values), 2),
            "fitness_max": round(max(fitness_values), 2),
        }
        print()
        print("Summary (valid runs):")
        print(f"  SC1  mean={summary['sc1_mean']}  stdev={summary['sc1_stdev']}  "
              f"min={summary['sc1_min']}  max={summary['sc1_max']}")
        print(f"  SC2  mean={summary['sc2_mean']}  stdev={summary['sc2_stdev']}")
        print(f"  Fitness  mean={summary['fitness_mean']}  stdev={summary['fitness_stdev']}  "
              f"min={summary['fitness_min']}  max={summary['fitness_max']}")

        # H6 verdict
        # If SC1 stdev is low (< 20) and mean is near 81.6, Week 2 spike was stochastic
        if summary["sc1_stdev"] < 20 and summary["sc1_mean"] < 100:
            verdict = "OUTCOME_A_STOCHASTIC"
            interpretation = "Week 2 SC1 spike is stochastic. No optimizer change required."
        else:
            verdict = "OUTCOME_B_STRUCTURAL"
            interpretation = "Week 2 consistently produces higher SC1. Structural characteristic of UB-002."
        summary["h6_verdict"] = verdict
        summary["h6_interpretation"] = interpretation
        print(f"\nH6 verdict: {verdict}")
        print(f"  {interpretation}")

    output = {
        "benchmark": "UB-002-v1.0",
        "sprint": "S8",
        "hypothesis": "H6",
        "week": 2,
        "seeds": SEEDS,
        "generation_limit": GENERATION_LIMIT,
        "results": results,
        "summary": summary,
    }

    with open(OUT_PATH, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nResults written to {OUT_PATH.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
