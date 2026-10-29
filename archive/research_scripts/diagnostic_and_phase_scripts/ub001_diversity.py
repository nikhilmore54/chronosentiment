#!/usr/bin/env python3
"""
UB-001 Diversity Probe — Sprint 5 H3a measurement instrument.

Runs UB-001 (4 weeks, 50 gens) and reports unique_genomes at sampled
generations (0, 10, 25, 49) to diagnose premature convergence.

H3a hypothesis: by Gen20, unique_genomes < 10 (out of 50), indicating
diversity collapse is the bottleneck preventing Gen49best improvement.
"""
import json, urllib.request
from pathlib import Path

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')

with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': s['skills']} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

SAMPLE_GENS = [0, 10, 25, 49]

all_weeks = []

for week in range(4):
    shifts = []
    sid = 1
    for day in range(7):
        dt = 'weekend' if day in [5, 6] else 'weekday'
        cov = ub['coverage_requirements'][dt]
        for sn, sh, dur in shift_types:
            for skill, cnt in cov[sn].items():
                for _ in range(cnt):
                    shifts.append({'id': sid, 'start_hour': day*24+sh, 'duration_hours': dur, 'required_skill': skill})
                    sid += 1

    payload = {'workers': workers, 'shifts': shifts, 'historical_workloads': None,
                'rng_seed': 42+week, 'generation_limit': 50}
    print(f'Week {week+1}/4: {len(shifts)} shifts, 50 gens...', flush=True)
    req = urllib.request.Request('http://localhost:3001/api/schedule',
        data=json.dumps(payload).encode(), headers={'Content-Type': 'application/json'}, method='POST')
    with urllib.request.urlopen(req, timeout=120) as r:
        d = json.loads(r.read())

    gens = d['telemetry']['generations']
    pop_size = 50  # known population size

    sampled = {}
    for g in gens:
        gi = g['generation']
        if gi in SAMPLE_GENS:
            sampled[gi] = g.get('unique_genomes', None)

    cr = d['constraint_report']
    g0 = gens[0]
    gl = gens[-1]

    print(f'  HC1={cr["hc1_violations"]} HC2={cr["hc2_violations"]} HC3={cr["hc3_violations"]} '
          f'Rest={cr["rest_violations"]} Fitness={d["metrics"]["fitness"]:.1f} '
          f'G0avg={g0["average_fitness"]:.1f} G49avg={gl["average_fitness"]:.1f}')
    print(f'  Diversity (unique/50): ', end='')
    for gi in SAMPLE_GENS:
        u = sampled.get(gi, '?')
        print(f'Gen{gi}={u}', end='  ')
    print()

    all_weeks.append({
        'week': week+1,
        'shifts': len(shifts),
        'hc1': cr['hc1_violations'], 'hc2': cr['hc2_violations'],
        'hc3': cr['hc3_violations'], 'rest': cr['rest_violations'],
        'fitness': d['metrics']['fitness'],
        'gen0_avg': g0['average_fitness'], 'gen49_avg': gl['average_fitness'],
        'gen49_best': gl['best_fitness'],
        'diversity_profile': {str(gi): sampled.get(gi) for gi in SAMPLE_GENS},
        'all_generations': [{'gen': g['generation'], 'unique': g.get('unique_genomes'),
                              'best': g['best_fitness'], 'avg': g['average_fitness']}
                             for g in gens],
    })

print()
print('=' * 70)
print('H3a DIVERSITY PROFILE — UB-001, 50 gens, pop=50')
print()
hdr = f'{"Wk":>3}  {"Gen0":>6}  {"Gen10":>6}  {"Gen25":>6}  {"Gen49":>6}  {"G49best":>9}  {"G49avg":>9}'
print(hdr)
print('-' * len(hdr))
for w in all_weeks:
    dp = w['diversity_profile']
    print(f'{w["week"]:>3}  {str(dp.get("0","?")):>6}  {str(dp.get("10","?")):>6}  '
          f'{str(dp.get("25","?")):>6}  {str(dp.get("49","?")):>6}  '
          f'{w["gen49_best"]:>9.1f}  {w["gen49_avg"]:>9.1f}')

print()
print('H3a hypothesis threshold: unique_genomes < 10 at Gen20 → diversity collapse confirmed')

out = ROOT / 'benchmarks/ultracrew/UB-001-DIVERSITY-v1.0.json'
with open(out, 'w') as f:
    json.dump({
        'benchmark': 'UB-001-v1.0',
        'run_date': '2026-07-12',
        'hypothesis': 'H3a-diversity-probe',
        'note': '4 weekly runs, 50 gens each, debug binary, unique_genomes per generation',
        'population_size': 50,
        'sample_generations': SAMPLE_GENS,
        'weeks': all_weeks,
    }, f, indent=2)
print(f'Written: {out}')