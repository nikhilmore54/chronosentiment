#!/usr/bin/env python3
"""
Generate UB-002-v1.0.json — UB-001 + historical workload (SC2 > 0).

UB-002 is identical to UB-001 in every structural dimension:
  - 20 workers, same skills and contracts
  - 4-week horizon, 83 shifts/week, all 8h duration
  - Same coverage requirements

The single change from UB-001: historical_workloads is non-null.
This activates the SC2 fatigue penalty (historical_fatigue * hours * 2.0).

Historical workload design — skill-balanced distribution:
  HIGH/MEDIUM/LOW groups each contain one worker from each skill category
  so that fatigue is the only manipulated variable (not skill topology).

  Worker skill categories (from UB-001):
    Nurse+ICU:        1, 6, 11, 15, 20
    SeniorNurse+ICU:  3, 16
    Nurse-only:       2, 4, 7, 9, 10, 13, 14, 17, 18
    SeniorNurse-only: 5, 8, 12, 19

  HIGH (4 workers — one from each skill category):
    1  (Nurse+ICU), 3 (SeniorNurse+ICU), 2 (Nurse-only), 5 (SeniorNurse-only)
    Prior load: ~40h/week

  LOW (4 workers — one from each skill category):
    20 (Nurse+ICU), 16 (SeniorNurse+ICU), 17 (Nurse-only), 19 (SeniorNurse-only)
    Prior load: ~20h/week

  MEDIUM (12 workers — all remaining):
    Prior load: ~32h/week

This design ensures that if the optimizer reduces HIGH workers' assignments,
the cause is SC2 (fatigue penalty), not skill topology flexibility.

Sprint 7 research question:
  Can Coralys optimize workload fairness (SC1) while simultaneously
  minimizing historical fatigue (SC2 > 0) without introducing hard
  constraint violations?

Success criteria (behavioral, not a single fitness value):
  1. HC1 = HC2 = HC3 = Rest = 0 (hard constraints still satisfied)
  2. SC2 demonstrably influences assignments (HIGH workers get fewer shifts
     than LOW workers, controlling for skill availability)
  3. Optimizer converges reliably across all 4 weeks
  4. SC1 and SC2 trade-off is measurable and documented
"""
import json
from pathlib import Path

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')

with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub001 = json.load(f)

# Skill-balanced HIGH/LOW/MEDIUM groups
# Each group contains one worker from each skill category
high_workers = {1, 3, 2, 5}    # Nurse+ICU, SeniorNurse+ICU, Nurse-only, SeniorNurse-only
low_workers  = {20, 16, 17, 19} # Nurse+ICU, SeniorNurse+ICU, Nurse-only, SeniorNurse-only
# MEDIUM: all remaining 12 workers

historical_workloads = {}
for i in range(1, 21):
    if i in high_workers:
        historical_workloads[str(i)] = [40.0, 38.0, 42.0, 39.0]
    elif i in low_workers:
        historical_workloads[str(i)] = [20.0, 22.0, 18.0, 21.0]
    else:
        historical_workloads[str(i)] = [32.0, 33.0, 31.0, 34.0]

medium_workers = sorted(set(range(1, 21)) - high_workers - low_workers)

ub002 = dict(ub001)
ub002['benchmark_id'] = 'UB-002-v1.0'
ub002['derived_from'] = 'UB-001-v1.0'
ub002['description'] = (
    'UB-001 + historical workload (SC2 > 0). '
    'Identical structure to UB-001; single change is non-null historical_workloads. '
    'HIGH/MEDIUM/LOW fatigue groups are skill-balanced to isolate SC2 as the '
    'only manipulated variable. Tests multi-objective optimization: SC1 (fairness) '
    'vs SC2 (fatigue).'
)
ub002['historical_workloads'] = historical_workloads
ub002['workload_design'] = {
    'high_workers': sorted(high_workers),
    'medium_workers': medium_workers,
    'low_workers': sorted(low_workers),
    'prior_weeks': 4,
    'skill_balance_note': (
        'Each of HIGH and LOW contains exactly one worker from each skill category '
        '(Nurse+ICU, SeniorNurse+ICU, Nurse-only, SeniorNurse-only). '
        'This ensures fatigue is the only manipulated variable.'
    ),
    'high_prior_hours': [40.0, 38.0, 42.0, 39.0],
    'medium_prior_hours': [32.0, 33.0, 31.0, 34.0],
    'low_prior_hours': [20.0, 22.0, 18.0, 21.0],
}

out = ROOT / 'benchmarks/ultracrew/UB-002-v1.0.json'
with open(out, 'w') as f:
    json.dump(ub002, f, indent=2)

print(f'Written: {out}')
print()
print('UB-002 workload design (skill-balanced):')
print(f'  HIGH  workers {sorted(high_workers)}: ~40h/week prior load')
print(f'  LOW   workers {sorted(low_workers)}: ~20h/week prior load')
print(f'  MEDIUM workers {medium_workers}: ~32h/week prior load')
print()
print('Skill category membership:')
skill_map = {
    'Nurse+ICU':        [1, 6, 11, 15, 20],
    'SeniorNurse+ICU':  [3, 16],
    'Nurse-only':       [2, 4, 7, 9, 10, 13, 14, 17, 18],
    'SeniorNurse-only': [5, 8, 12, 19],
}
for cat, members in skill_map.items():
    h = [w for w in members if w in high_workers]
    l = [w for w in members if w in low_workers]
    m = [w for w in members if w in medium_workers]
    print(f'  {cat}: HIGH={h} LOW={l} MEDIUM={m}')
print()
print('SC2 fatigue penalty = historical_fatigue * hours_this_week * 2.0')
print('historical_fatigue = mean of prior weekly hours (from WorkforceEcology)')
print()
print('Expected SC2 per worker per week at 32h assignment:')
for label, prior in [('HIGH', [40.0, 38.0, 42.0, 39.0]),
                      ('MEDIUM', [32.0, 33.0, 31.0, 34.0]),
                      ('LOW', [20.0, 22.0, 18.0, 21.0])]:
    fatigue = sum(prior) / len(prior)
    sc2 = fatigue * 32 * 2.0
    print(f'  {label}: fatigue={fatigue:.2f}, SC2@32h={sc2:.1f}')