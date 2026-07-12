#!/usr/bin/env python3
import json, urllib.request
from pathlib import Path

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')

with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': s['skills']} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

total_hc1 = total_hc2 = total_hc3 = total_rest = 0
total_shifts = 0
total_ms = 0
all_valid = True
week_rows = []

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
    total_shifts += len(shifts)
    payload = {'workers': workers, 'shifts': shifts, 'historical_workloads': None, 'rng_seed': 42+week, 'generation_limit': 50}
    print(f'Week {week+1}/4: {len(shifts)} shifts, 50 gens...', flush=True)
    req = urllib.request.Request('http://localhost:3001/api/schedule',
        data=json.dumps(payload).encode(), headers={'Content-Type': 'application/json'}, method='POST')
    with urllib.request.urlopen(req, timeout=60) as r:
        d = json.loads(r.read())
    cr = d['constraint_report']
    gens = d['telemetry']['generations']
    g0 = gens[0]
    gl = gens[-1]
    total_hc1 += cr['hc1_violations']
    total_hc2 += cr['hc2_violations']
    total_hc3 += cr['hc3_violations']
    total_rest += cr['rest_violations']
    total_ms += gl['elapsed_time_ms']
    if not cr['is_valid']:
        all_valid = False
    week_rows.append((week+1, len(shifts), cr['hc1_violations'], cr['hc2_violations'], cr['hc3_violations'],
                      cr['rest_violations'], cr['is_valid'], d['metrics']['fitness'], gl['elapsed_time_ms'],
                      g0['best_fitness'], g0['average_fitness'], gl['best_fitness'], gl['average_fitness']))
    print(f'  HC1={cr["hc1_violations"]} HC2={cr["hc2_violations"]} HC3={cr["hc3_violations"]} '
          f'Rest={cr["rest_violations"]} Valid={cr["is_valid"]} Fitness={d["metrics"]["fitness"]:.1f} '
          f'G0avg={g0["average_fitness"]:.1f} G49avg={gl["average_fitness"]:.1f} {gl["elapsed_time_ms"]}ms', flush=True)

hard = total_hc1 + total_hc2 + total_hc3 + total_rest
pas = round(max(0.0, (total_shifts - hard) / total_shifts * 100), 1)

print(f'\n{"="*60}')
print(f'UB-001 BASELINE — 4 weeks, 50 gens each, debug binary')
print(f'Workers=20  Total shifts={total_shifts}  Total runtime={total_ms}ms')
print(f'HC1={total_hc1} HC2={total_hc2} HC3={total_hc3} Rest={total_rest}')
print(f'All valid={all_valid}  PAS={pas}%')
print()
hdr = f'{"Wk":>3} {"Shifts":>7} {"HC1":>4} {"HC2":>4} {"HC3":>4} {"Rest":>5} {"Valid":>6} {"Fitness":>9} {"ms":>7} {"G0best":>8} {"G0avg":>8} {"G49best":>8} {"G49avg":>8}'
print(hdr)
print('-' * len(hdr))
for r in week_rows:
    print(f'{r[0]:>3} {r[1]:>7} {r[2]:>4} {r[3]:>4} {r[4]:>4} {r[5]:>5} {str(r[6]):>6} {r[7]:>9.1f} {r[8]:>7} {r[9]:>8.1f} {r[10]:>8.1f} {r[11]:>8.1f} {r[12]:>8.1f}')

baseline = {
    'benchmark': 'UB-001-v1.0',
    'run_date': '2026-07-12',
    'optimizer_commit': '44b29cec',
    'hypothesis': 'H1-skill-aware-initialization',
    'note': '4 weekly runs, 50 gens each, debug binary',
    'config': {'workers': 20, 'total_shifts': total_shifts, 'weeks': 4, 'generation_limit': 50},
    'aggregate': {
        'hc1_violations': total_hc1, 'hc2_violations': total_hc2,
        'hc3_violations': total_hc3, 'rest_violations': total_rest,
        'all_weeks_valid': all_valid, 'pas_estimate': pas, 'total_runtime_ms': total_ms
    },
    'weeks': [{'week': r[0], 'shifts': r[1], 'hc1': r[2], 'hc2': r[3], 'hc3': r[4], 'rest': r[5],
               'is_valid': r[6], 'best_fitness': r[7], 'runtime_ms': r[8],
               'gen0_best': r[9], 'gen0_avg': r[10], 'gen49_best': r[11], 'gen49_avg': r[12]}
              for r in week_rows]
}
out = ROOT / 'benchmarks/ultracrew/UB-001-BASELINE-v1.0.json'
with open(out, 'w') as f:
    json.dump(baseline, f, indent=2)
print(f'\nWritten: {out}')