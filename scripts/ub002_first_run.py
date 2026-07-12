import json, time, urllib.request
from pathlib import Path

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')

with open(ROOT / 'benchmarks/ultracrew/UB-002-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': s['skills']} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

high_workers = set(ub['workload_design']['high_workers'])
low_workers  = set(ub['workload_design']['low_workers'])
medium_workers = set(ub['workload_design']['medium_workers'])

print('UB-002 First Measurement')
print('Sprint 7 H5: SC2 fatigue penalty active (historical_workloads non-null)')
print()
print('Workload groups:')
print(f'  HIGH   workers {sorted(high_workers)}: fatigue=0.994 (mean=39.75h/40h)')
print(f'  MEDIUM workers {sorted(medium_workers)}: fatigue=0.813 (mean=32.5h/40h)')
print(f'  LOW    workers {sorted(low_workers)}: fatigue=0.506 (mean=20.25h/40h)')
print()
print('Expected SC2 per worker per week at 32h assignment:')
print('  HIGH: 0.994 * 32 * 2.0 = 63.6')
print('  MEDIUM: 0.813 * 32 * 2.0 = 52.0')
print('  LOW: 0.506 * 32 * 2.0 = 32.4')
print()

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

    payload = {
        'workers': workers,
        'shifts': shifts,
        'historical_workloads': ub['historical_workloads'],
        'rng_seed': 42 + week,
        'generation_limit': 50,
    }

    print(f'Week {week+1}/4: {len(shifts)} shifts, 50 gens...', flush=True)
    req = urllib.request.Request(
        'http://localhost:3001/api/schedule',
        data=json.dumps(payload).encode(),
        headers={'Content-Type': 'application/json'},
        method='POST'
    )
    t0 = time.time()
    with urllib.request.urlopen(req, timeout=60) as r:
        d = json.loads(r.read())
    ms = int((time.time() - t0) * 1000)

    cr = d['constraint_report']
    m = d['metrics']
    gens = d['telemetry']['generations']
    g0 = gens[0]
    gl = gens[-1]

    fitness = m['fitness']
    sc1 = m.get('fairness_penalty', None)
    sc2 = m.get('fatigue_penalty', None)

    total_hc1 += cr['hc1_violations']
    total_hc2 += cr['hc2_violations']
    total_hc3 += cr['hc3_violations']
    total_rest += cr['rest_violations']
    total_ms += ms
    if not cr['is_valid']:
        all_valid = False

    schedule = d['schedule']
    assignments_by_worker = {}
    for shift_id, worker_id in schedule.items():
        wid = int(worker_id)
        assignments_by_worker[wid] = assignments_by_worker.get(wid, 0) + 1

    high_avg = sum(assignments_by_worker.get(w, 0) for w in high_workers) / len(high_workers)
    medium_avg = sum(assignments_by_worker.get(w, 0) for w in medium_workers) / len(medium_workers)
    low_avg = sum(assignments_by_worker.get(w, 0) for w in low_workers) / len(low_workers)

    week_rows.append({
        'week': week+1, 'shifts': len(shifts),
        'hc1': cr['hc1_violations'], 'hc2': cr['hc2_violations'],
        'hc3': cr['hc3_violations'], 'rest': cr['rest_violations'],
        'valid': cr['is_valid'], 'fitness': fitness, 'sc1': sc1, 'sc2': sc2,
        'ms': ms, 'g0_avg': g0['average_fitness'], 'g49_avg': gl['average_fitness'],
        'high_avg_shifts': high_avg, 'medium_avg_shifts': medium_avg, 'low_avg_shifts': low_avg,
    })

    print(f'  HC1={cr["hc1_violations"]} HC2={cr["hc2_violations"]} HC3={cr["hc3_violations"]} '
          f'Rest={cr["rest_violations"]} Valid={cr["is_valid"]}')
    print(f'  Fitness={fitness:.1f}  SC1={sc1:.2f}  SC2={sc2:.2f}')
    print(f'  G0avg={g0["average_fitness"]:.1f}  G49avg={gl["average_fitness"]:.1f}  {ms}ms')
    print(f'  Avg shifts: HIGH={high_avg:.2f}  MEDIUM={medium_avg:.2f}  LOW={low_avg:.2f}')
    print()

print('='*70)
print(f'UB-002 FIRST MEASUREMENT — 4 weeks, 50 gens each')
print(f'Workers=20  Total shifts={total_shifts}  Total runtime={total_ms}ms')
print(f'HC1={total_hc1} HC2={total_hc2} HC3={total_hc3} Rest={total_rest}')
print(f'All valid={all_valid}')
print()
print('Week-by-week SC decomposition:')
print(f'{"Wk":>3}  {"Fitness":>9}  {"SC1":>8}  {"SC2":>8}  {"HIGH":>6}  {"MED":>6}  {"LOW":>6}')
print('-'*60)
for r in week_rows:
    print(f'{r["week"]:>3}  {r["fitness"]:>9.1f}  {r["sc1"]:>8.2f}  {r["sc2"]:>8.2f}  '
          f'{r["high_avg_shifts"]:>6.2f}  {r["medium_avg_shifts"]:>6.2f}  {r["low_avg_shifts"]:>6.2f}')

print()
print('SC2 influence check:')
avg_high_shifts = sum(r['high_avg_shifts'] for r in week_rows) / 4
avg_low_shifts  = sum(r['low_avg_shifts']  for r in week_rows) / 4
avg_med_shifts  = sum(r['medium_avg_shifts'] for r in week_rows) / 4
print(f'  Mean shifts/week: HIGH={avg_high_shifts:.2f}  MEDIUM={avg_med_shifts:.2f}  LOW={avg_low_shifts:.2f}')
if avg_low_shifts > avg_high_shifts:
    print(f'  SC2 influence CONFIRMED: LOW workers get more shifts than HIGH workers')
    print(f'  Difference: LOW-HIGH = {avg_low_shifts - avg_high_shifts:.2f} shifts/week')
else:
    print(f'  SC2 influence NOT CONFIRMED: HIGH workers not protected from loading')

out = ROOT / 'benchmarks/ultracrew/UB-002-FIRST-RUN-v1.0.json'
with open(out, 'w') as f:
    json.dump({
        'benchmark': 'UB-002-v1.0',
        'run_date': '2026-07-13',
        'hypothesis': 'H5: SC2 fatigue penalty active and influences assignments',
        'weeks': week_rows,
        'summary': {
            'total_hc1': total_hc1, 'total_hc2': total_hc2,
            'total_hc3': total_hc3, 'total_rest': total_rest,
            'all_valid': all_valid,
            'avg_high_shifts': avg_high_shifts,
            'avg_medium_shifts': avg_med_shifts,
            'avg_low_shifts': avg_low_shifts,
        }
    }, f, indent=2)
print(f'Written: {out}')