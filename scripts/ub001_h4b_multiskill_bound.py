#!/usr/bin/env python3
"""
UB-001 H4b — True Lower Bound with Multi-Skill Worker Flexibility.

The H4 greedy analysis treated multi-skill workers as fixed to one skill pool,
producing SC1=153.6 — but the GA achieves 9918.4 (SC1+SC2=81.6), which is
better. This means the greedy bound was not a true lower bound.

H4b computes the true minimum-variance assignment by:
1. Treating multi-skill workers as flexible (can cover any of their skills)
2. Using a global greedy: assign each shift to the qualified worker with
   fewest current hours, regardless of skill group
3. Also computing the mathematical ideal: if 2656h were perfectly divisible
   among 20 workers, what is the minimum achievable variance?

This gives a tighter lower bound on SC1 that accounts for cross-skill flexibility.
"""
import json, math
from pathlib import Path
from collections import defaultdict

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')

with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': set(s['skills'])} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

# Build all 332 shifts (4 weeks)
all_shifts = []
sid = 1
for week in range(4):
    for day in range(7):
        dt = 'weekend' if day in [5, 6] else 'weekday'
        cov = ub['coverage_requirements'][dt]
        for sn, sh, dur in shift_types:
            for skill, cnt in cov[sn].items():
                for _ in range(cnt):
                    all_shifts.append({'id': sid, 'duration': dur, 'skill': skill})
                    sid += 1

total_hours = sum(s['duration'] for s in all_shifts)
n_workers = len(workers)
mean_hours = total_hours / n_workers

print(f'UB-001: {n_workers} workers, {len(all_shifts)} shifts, {total_hours}h total')
print(f'Mean hours per worker: {mean_hours:.2f}')
print()

# ── Mathematical ideal lower bound ───────────────────────────────────────────
# If total_hours were perfectly divisible among n_workers:
base = total_hours // n_workers
remainder = total_hours % n_workers
ideal_hours = [base + 1] * remainder + [base] * (n_workers - remainder)
ideal_mean = sum(ideal_hours) / n_workers
ideal_var = sum((h - ideal_mean)**2 for h in ideal_hours) / n_workers
ideal_sc1 = ideal_var * 10.0
ideal_fitness = 10000.0 - ideal_sc1

print(f'Mathematical ideal (perfect balance, ignoring all constraints):')
print(f'  Distribution: {base}h × {n_workers - remainder} workers, {base+1}h × {remainder} workers')
print(f'  Variance: {ideal_var:.6f}')
print(f'  SC1 penalty: {ideal_sc1:.6f}')
print(f'  Implied max fitness (SC2=0): {ideal_fitness:.6f}')
print()

# ── Global greedy (multi-skill aware, HC1 only) ───────────────────────────────
# Assign each shift to the skill-qualified worker with fewest current hours.
# Workers with multiple skills can cover any of their skills.
worker_hours_global = {w['id']: 0 for w in workers}

for shift in all_shifts:
    qualified_ids = [w['id'] for w in workers if shift['skill'] in w['skills']]
    chosen = min(qualified_ids, key=lambda wid: worker_hours_global[wid])
    worker_hours_global[chosen] += shift['duration']

hours_list_g = list(worker_hours_global.values())
mean_g = sum(hours_list_g) / len(hours_list_g)
var_g = sum((h - mean_g)**2 for h in hours_list_g) / len(hours_list_g)
sc1_g = var_g * 10.0
fitness_g = 10000.0 - sc1_g

print(f'Global greedy (multi-skill aware, HC1 only, no HC2/HC3/rest):')
print(f'  Hours per worker: {sorted(hours_list_g)}')
print(f'  Mean: {mean_g:.2f}, Variance: {var_g:.4f}')
print(f'  SC1 penalty: {sc1_g:.4f}')
print(f'  Implied max fitness (SC2=0): {fitness_g:.4f}')
print()

# ── Comparison ────────────────────────────────────────────────────────────────
ga_best = 9918.4
ga_residual = 10000.0 - ga_best

print(f'GA best fitness:          {ga_best}')
print(f'GA residual SC1+SC2:      {ga_residual:.1f}')
print(f'Global greedy SC1:        {sc1_g:.4f}  (implied max {fitness_g:.4f})')
print(f'Mathematical ideal SC1:   {ideal_sc1:.6f}  (implied max {ideal_fitness:.6f})')
print()

if fitness_g > ga_best + 0.1:
    gap = fitness_g - ga_best
    print(f'H4b REFUTED: Global greedy achieves {fitness_g:.2f} > GA best {ga_best}')
    print(f'  Gap = {gap:.2f} fitness units. GA has NOT found the optimum.')
    print(f'  The gap is likely due to HC2/HC3/rest constraints blocking optimal balance.')
elif abs(fitness_g - ga_best) < 1.0:
    print(f'H4b CONSISTENT: Global greedy ≈ GA best ({fitness_g:.2f} ≈ {ga_best})')
    print(f'  Current evidence is consistent with 9918.4 being near-optimal.')
else:
    print(f'H4b: Global greedy {fitness_g:.2f} < GA best {ga_best}')
    print(f'  GA outperforms global greedy — GA is exploiting structure greedy cannot.')

print()
print(f'SC2 note: GA residual includes SC2 (fatigue). If SC2=0 (no historical workload),')
print(f'  then SC1 alone = {ga_residual:.1f}. Compare to global greedy SC1 = {sc1_g:.4f}.')

# Save
out = ROOT / 'benchmarks/ultracrew/UB-001-H4B-MULTISKILL-v1.0.json'
with open(out, 'w') as f:
    json.dump({
        'benchmark': 'UB-001-v1.0',
        'run_date': '2026-07-13',
        'hypothesis': 'H4b-multiskill-lower-bound',
        'note': 'Global greedy (multi-skill aware), HC1 only, no HC2/HC3/rest',
        'total_hours': total_hours,
        'mean_hours': mean_hours,
        'mathematical_ideal': {
            'variance': ideal_var, 'sc1_penalty': ideal_sc1,
            'implied_max_fitness': ideal_fitness,
        },
        'global_greedy': {
            'hours_per_worker': sorted(hours_list_g),
            'variance': var_g, 'sc1_penalty': sc1_g,
            'implied_max_fitness': fitness_g,
        },
        'ga_result': {'best_fitness': ga_best, 'residual_sc1_plus_sc2': ga_residual},
        'gap_greedy_vs_ga': fitness_g - ga_best,
    }, f, indent=2)
print(f'\nWritten: {out}')