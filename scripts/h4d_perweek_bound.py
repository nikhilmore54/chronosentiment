import json
from pathlib import Path
from collections import defaultdict

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')
with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': set(s['skills'])} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

print("Per-week integer lower bound analysis")
print("=" * 60)
print()

total_sc1_bound = 0.0
total_sc1_ga = 0.0
ga_fitness_per_week = [9918.4, 9918.4, 9918.4, 9918.4]

for week in range(4):
    shifts = []
    sid = 1
    for day in range(7):
        dt = 'weekend' if day in [5, 6] else 'weekday'
        cov = ub['coverage_requirements'][dt]
        for sn, sh, dur in shift_types:
            for skill, cnt in cov[sn].items():
                for _ in range(cnt):
                    shifts.append({'id': sid, 'duration': dur, 'skill': skill})
                    sid += 1

    n_shifts = len(shifts)
    total_hours = sum(s['duration'] for s in shifts)
    n_workers = len(workers)
    mean_h = total_hours / n_workers

    base = total_hours // n_workers
    remainder = total_hours % n_workers
    ideal_hours = [base + 1] * remainder + [base] * (n_workers - remainder)
    ideal_mean = sum(ideal_hours) / n_workers
    ideal_var = sum((h - ideal_mean)**2 for h in ideal_hours) / n_workers
    ideal_sc1 = ideal_var * 10.0

    n_shifts_per_worker_high = (total_hours // 8) // n_workers + (1 if (total_hours // 8) % n_workers > 0 else 0)
    n_shifts_per_worker_low = (total_hours // 8) // n_workers
    high_count = (total_hours // 8) % n_workers
    low_count = n_workers - high_count
    int_hours = [n_shifts_per_worker_high * 8] * high_count + [n_shifts_per_worker_low * 8] * low_count
    int_mean = sum(int_hours) / n_workers
    int_var = sum((h - int_mean)**2 for h in int_hours) / n_workers
    int_sc1 = int_var * 10.0
    int_max_fitness = 10000.0 - int_sc1

    ga_residual = 10000.0 - ga_fitness_per_week[week]

    print(f"Week {week+1}: {n_shifts} shifts, {total_hours}h total, mean={mean_h:.2f}h/worker")
    print(f"  Integer 8h bound: {high_count} workers at {n_shifts_per_worker_high*8}h, "
          f"{low_count} workers at {n_shifts_per_worker_low*8}h")
    print(f"  Integer SC1={int_sc1:.4f}, implied max fitness={int_max_fitness:.4f}")
    print(f"  GA fitness={ga_fitness_per_week[week]}, GA residual SC1+SC2={ga_residual:.1f}")
    if ga_fitness_per_week[week] > int_max_fitness + 0.1:
        print(f"  => GA EXCEEDS integer bound by {ga_fitness_per_week[week] - int_max_fitness:.2f}")
        print(f"     This means SC2 (fatigue) is NEGATIVE (bonus) or SC1 < integer bound.")
    elif abs(ga_fitness_per_week[week] - int_max_fitness) < 1.0:
        print(f"  => GA ≈ integer bound (within 1.0 fitness unit)")
    else:
        gap = int_max_fitness - ga_fitness_per_week[week]
        print(f"  => Gap: {gap:.2f} fitness units below integer bound")
    print()

    total_sc1_bound += int_sc1
    total_sc1_ga += ga_residual

print(f"Summary across 4 weeks:")
print(f"  Total integer SC1 bound: {total_sc1_bound:.4f}")
print(f"  Total GA residual SC1+SC2: {total_sc1_ga:.1f}")
print(f"  Difference: {total_sc1_ga - total_sc1_bound:.4f}")
print()
print("Note: GA residual includes SC2 (fatigue penalty from historical workload).")
print("If historical_workloads=None (as in UB-001 runs), SC2=0 and residual=SC1 only.")