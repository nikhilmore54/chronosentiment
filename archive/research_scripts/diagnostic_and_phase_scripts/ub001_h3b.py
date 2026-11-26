#!/usr/bin/env python3
"""
UB-001 H3b — Long-run plateau test (200 generations, release binary).

H3b hypothesis: The plateau at G49best=9918.4 is a fitness landscape feature,
not a generation-count limit. Running 200 generations will not move best fitness
above 9918.4.

Reports best, average, and unique_genomes at Gen0, Gen50, Gen100, Gen150, Gen199
to distinguish:
  - "elite pinned, average still climbing" -> operators exploiting same basin
  - "everything converged"                 -> true local optimum
"""
import json, urllib.request
from pathlib import Path

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')
GEN_LIMIT = 200
SAMPLE_GENS = [0, 50, 100, 150, 199]

with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': s['skills']} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

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
                    shifts.append({'id': sid, 'start_hour': day*24+sh,
                                   'duration_hours': dur, 'required_skill': skill})
                    sid += 1

    payload = {'workers': workers, 'shifts': shifts, 'historical_workloads': None,
                'rng_seed': 42+week, 'generation_limit': GEN_LIMIT}
    print(f'Week {week+1}/4: {len(shifts)} shifts, {GEN_LIMIT} gens (release)...', flush=True)
    req = urllib.request.Request('http://localhost:3001/api/schedule',
        data=json.dumps(payload).encode(),
        headers={'Content-Type': 'application/json'}, method='POST')
    with urllib.request.urlopen(req, timeout=300) as r:
        d = json.loads(r.read())

    gens = d['telemetry']['generations']
    cr = d['constraint_report']
    g0 = gens[0]
    gl = gens[-1]

    sampled = {}
    for g in gens:
        gi = g['generation']
        if gi in SAMPLE_GENS:
            sampled[gi] = {
                'best': g['best_fitness'],
                'avg': g['average_fitness'],
                'unique': g.get('unique_genomes', None),
            }

    print(f'  HC1={cr["hc1_violations"]} HC2={cr["hc2_violations"]} '
          f'HC3={cr["hc3_violations"]} Rest={cr["rest_violations"]} '
          f'Fitness={d["metrics"]["fitness"]:.1f} '
          f'G0avg={g0["average_fitness"]:.1f} G{GEN_LIMIT-1}avg={gl["average_fitness"]:.1f} '
          f'{gl["elapsed_time_ms"]}ms')
    print(f'  {"Gen":>5}  {"Best":>9}  {"Avg":>9}  {"Unique":>7}')
    for gi in SAMPLE_GENS:
        s = sampled.get(gi, {})
        print(f'  {gi:>5}  {s.get("best", "?"):>9}  {s.get("avg", "?"):>9}  {str(s.get("unique","?")):>7}')

    all_weeks.append({
        'week': week+1,
        'shifts': len(shifts),
        'hc1': cr['hc1_violations'], 'hc2': cr['hc2_violations'],
        'hc3': cr['hc3_violations'], 'rest': cr['rest_violations'],
        'fitness': d['metrics']['fitness'],
        'runtime_ms': gl['elapsed_time_ms'],
        'gen0_avg': g0['average_fitness'],
        f'gen{GEN_LIMIT-1}_avg': gl['average_fitness'],
        f'gen{GEN_LIMIT-1}_best': gl['best_fitness'],
        'sampled': {str(gi): sampled.get(gi) for gi in SAMPLE_GENS},
        'all_generations': [{'gen': g['generation'], 'best': g['best_fitness'],
                              'avg': g['average_fitness'],
                              'unique': g.get('unique_genomes')} for g in gens],
    })

print()
print('=' * 75)
print(f'H3b LONG-RUN PROFILE — UB-001, {GEN_LIMIT} gens, pop=100, release binary')
print()
hdr = (f'{"Wk":>3}  {"G0best":>9}  {"G0avg":>9}  '
       f'{"G50best":>9}  {"G100best":>9}  {"G150best":>9}  {"G199best":>9}  '
       f'{"G199avg":>9}  {"ms":>7}')
print(hdr)
print('-' * len(hdr))
for w in all_weeks:
    sp = w['sampled']
    def b(gi): return f'{sp[str(gi)]["best"]:.1f}' if sp.get(str(gi)) else '?'
    def a(gi): return f'{sp[str(gi)]["avg"]:.1f}' if sp.get(str(gi)) else '?'
    print(f'{w["week"]:>3}  {b(0):>9}  {a(0):>9}  '
          f'{b(50):>9}  {b(100):>9}  {b(150):>9}  {b(199):>9}  '
          f'{a(199):>9}  {w["runtime_ms"]:>7}')

print()
print('H3b hypothesis: G199best == 9918.4 (plateau holds) -> operator redesign warranted')
print('H3b refuted if: G199best > 9950 -> longer runs sufficient')

out = ROOT / 'benchmarks/ultracrew/UB-001-H3B-v1.0.json'
with open(out, 'w') as f:
    json.dump({
        'benchmark': 'UB-001-v1.0',
        'run_date': '2026-07-12',
        'hypothesis': 'H3b-long-run-plateau',
        'note': f'4 weekly runs, {GEN_LIMIT} gens each, release binary, pop=100',
        'generation_limit': GEN_LIMIT,
        'population_size': 100,
        'sample_generations': SAMPLE_GENS,
        'weeks': all_weeks,
    }, f, indent=2)
print(f'Written: {out}')