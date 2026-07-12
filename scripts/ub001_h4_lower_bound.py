#!/usr/bin/env python3
"""
UB-001 H4 — Analytical Lower Bound on SC1 Penalty.

H4 hypothesis: The minimum achievable SC1 penalty on UB-001 under the current
objective function (variance(worker_hours) * 10.0) equals approximately 81.6,
meaning 9918.4 is the best achievable fitness given the benchmark's skill and
coverage constraints.

Method:
  1. Ignore HC2/HC3/rest — compute only the minimum-variance assignment of
     UB-001's 332 shifts to 20 workers subject to HC1 (skill match only).
  2. Use a greedy min-hours-first assignment: for each shift, assign it to the
     skill-qualified worker with the fewest current hours. This is the standard
     greedy lower bound for makespan/variance minimisation.
  3. Also compute the true mathematical lower bound: if total hours were
     perfectly divisible among qualified workers, what would the variance be?
  4. Compare both bounds to the GA's residual penalty of 81.6.

Outcomes:
  A. Lower bound ≈ 81.6 → GA has found the optimum; 9918.4 is optimal.
  B. Lower bound < 81.6 → GA has not found the optimum; gap exists.
  C. Lower bound = 0    → Perfect balance is achievable; objective or operators
                          are missing something fundamental.
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

print(f'UB-001: {len(workers)} workers, {len(all_shifts)} shifts')
print()

# ── Skill distribution analysis ──────────────────────────────────────────────
skill_hours = defaultdict(int)
skill_shifts = defaultdict(int)
for s in all_shifts:
    skill_hours[s['skill']] += s['duration']
    skill_shifts[s['skill']] += 1

print('Skill demand:')
for skill in sorted(skill_hours):
    qualified = [w for w in workers if skill in w['skills']]
    print(f'  {skill}: {skill_shifts[skill]} shifts, {skill_hours[skill]}h total, '
          f'{len(qualified)} qualified workers')
print()

# ── True mathematical lower bound ────────────────────────────────────────────
# For each skill group, the minimum variance contribution is achieved when
# hours are distributed as evenly as possible among qualified workers.
# Since workers can cover multiple skills, this is a relaxation (lower bound).
# We compute it per-skill independently (ignoring cross-skill interactions).

print('Per-skill minimum variance (independent relaxation):')
total_min_variance_contribution = 0.0
for skill in sorted(skill_hours):
    qualified = [w for w in workers if skill in w['skills']]
    n = len(qualified)
    total_h = skill_hours[skill]
    # Perfect distribution: each worker gets total_h / n hours
    # Variance of perfect distribution = 0 if divisible, else small
    base = total_h // n
    remainder = total_h % n
    # remainder workers get (base+1), rest get base
    hours = [base+1]*remainder + [base]*(n-remainder)
    mean = sum(hours) / n
    var = sum((h - mean)**2 for h in hours) / n
    print(f'  {skill}: {n} workers, {total_h}h total -> '
          f'ideal dist {base}+{base+1} ({remainder} workers), variance={var:.4f}')
    total_min_variance_contribution += var

print()

# ── Greedy min-hours-first assignment (HC1 only) ─────────────────────────────
# Assign each shift to the skill-qualified worker with fewest current hours.
# This is a practical lower bound — achievable without HC2/HC3/rest constraints.
worker_hours_greedy = {w['id']: 0 for w in workers}

for shift in all_shifts:
    qualified_ids = [w['id'] for w in workers if shift['skill'] in w['skills']]
    # Pick worker with minimum current hours
    chosen = min(qualified_ids, key=lambda wid: worker_hours_greedy[wid])
    worker_hours_greedy[chosen] += shift['duration']

hours_list = list(worker_hours_greedy.values())
mean_g = sum(hours_list) / len(hours_list)
variance_g = sum((h - mean_g)**2 for h in hours_list) / len(hours_list)
sc1_greedy = variance_g * 10.0
fitness_greedy = 10000.0 - sc1_greedy  # assuming SC2=0 (no historical fatigue)

print('Greedy min-hours-first assignment (HC1 only, no HC2/HC3/rest):')
print(f'  Hours per worker: {sorted(hours_list)}')
print(f'  Mean hours: {mean_g:.2f}')
print(f'  Variance: {variance_g:.4f}')
print(f'  SC1 penalty (variance * 10): {sc1_greedy:.4f}')
print(f'  Implied max fitness (SC2=0): {fitness_greedy:.4f}')
print()

# ── GA result comparison ──────────────────────────────────────────────────────
ga_best = 9918.4
ga_sc1_plus_sc2 = 10000.0 - ga_best
print(f'GA best fitness: {ga_best}')
print(f'GA residual SC1+SC2 penalty: {ga_sc1_plus_sc2:.1f}')
print(f'Greedy SC1 lower bound: {sc1_greedy:.4f}')
print()

if sc1_greedy < ga_sc1_plus_sc2 - 0.1:
    gap = ga_sc1_plus_sc2 - sc1_greedy
    print(f'H4 REFUTED: Greedy achieves SC1={sc1_greedy:.2f} < GA residual {ga_sc1_plus_sc2:.1f}')
    print(f'  Gap = {gap:.2f} fitness units. GA has NOT found the optimum.')
    print(f'  Theoretical max fitness (greedy, SC2=0): {fitness_greedy:.2f}')
elif abs(sc1_greedy - ga_sc1_plus_sc2) < 1.0:
    print(f'H4 CONSISTENT: Greedy SC1 ≈ GA residual ({sc1_greedy:.2f} ≈ {ga_sc1_plus_sc2:.1f})')
    print(f'  Current evidence is consistent with 9918.4 being near-optimal.')
    print(f'  Note: SC2 (fatigue) contributes to GA residual; true SC1 alone may be lower.')
else:
    print(f'H4 INCONCLUSIVE: Greedy SC1={sc1_greedy:.2f}, GA residual={ga_sc1_plus_sc2:.1f}')

print()
print('Note: Greedy bound ignores HC2/HC3/rest. Real GA must satisfy all constraints,')
print('so the true achievable SC1 may be higher than the greedy bound.')

# Save results
out = ROOT / 'benchmarks/ultracrew/UB-001-H4-LOWER-BOUND-v1.0.json'
with open(out, 'w') as f:
    json.dump({
        'benchmark': 'UB-001-v1.0',
        'run_date': '2026-07-13',
        'hypothesis': 'H4-analytical-lower-bound',
        'note': 'Greedy min-hours-first assignment, HC1 only, no HC2/HC3/rest',
        'workers': len(workers),
        'total_shifts': len(all_shifts),
        'skill_demand': {k: {'shifts': skill_shifts[k], 'hours': skill_hours[k]} for k in skill_hours},
        'greedy_result': {
            'hours_per_worker': sorted(hours_list),
            'mean_hours': mean_g,
            'variance': variance_g,
            'sc1_penalty': sc1_greedy,
            'implied_max_fitness': fitness_greedy,
        },
        'ga_result': {
            'best_fitness': ga_best,
            'residual_sc1_plus_sc2': ga_sc1_plus_sc2,
        },
        'gap': ga_sc1_plus_sc2 - sc1_greedy,
    }, f, indent=2)
print(f'\nWritten: {out}')