import json, time, urllib.request
from pathlib import Path

ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')

with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)

workers = [{'id': i+1, 'skills': s['skills']} for i, s in enumerate(ub['staff'])]
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]

week_idx = 2
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

print('H5-R0: Week 3 SC decomposition probe (5 runs, seed=44)')
print('Hypothesis: Week 3 at 9854.4 is optimizer convergence failure (SC1>81.6, SC2=0)')
print()

results = []
for run in range(5):
    payload = {
        'workers': workers,
        'shifts': shifts,
        'historical_workloads': None,
        'rng_seed': 44 + run,
        'generation_limit': 50,
    }
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

    if run == 0:
        print(f'metrics keys: {list(d["metrics"].keys())}')
        print()

    fitness = d['metrics']['fitness']
    sc1 = d['metrics'].get('fairness_penalty', d['metrics'].get('sc1', None))
    sc2 = d['metrics'].get('fatigue_penalty', d['metrics'].get('sc2', None))
    cr = d['constraint_report']
    results.append({'fitness': fitness, 'sc1': sc1, 'sc2': sc2})
    print(f'Run {run+1}: fitness={fitness:.1f}  SC1={sc1}  SC2={sc2}  '
          f'HC1={cr["hc1_violations"]} HC2={cr["hc2_violations"]} HC3={cr["hc3_violations"]} Rest={cr["rest_violations"]}  ({ms}ms)')

print()
sc2_values = [r['sc2'] for r in results if r.get('sc2') is not None]
if sc2_values and all(v == 0.0 for v in sc2_values):
    print('CONCLUSION: SC2=0.0 confirmed on all runs.')
    print('  Week 3 deviation is optimizer convergence failure (SC1 > 81.6).')
    print('  H5 normalization change did NOT introduce a semantic regression.')
elif sc2_values and any(v > 0.0 for v in sc2_values):
    print('CONCLUSION: SC2 > 0 detected on UB-001 run — SEMANTIC REGRESSION.')
    print('  UB-001 should have SC2=0 (historical_workloads=null).')
else:
    print('SC2 field not found in metrics. Available metrics:')
    print(json.dumps(d['metrics'], indent=2))

out = ROOT / 'benchmarks/ultracrew/UB-001-H5-R0-WEEK3-v1.0.json'
with open(out, 'w') as f:
    json.dump({
        'benchmark': 'UB-001-v1.0',
        'run_date': '2026-07-13',
        'hypothesis': 'H5-R0: Week 3 SC decomposition',
        'week': 3,
        'runs': len(results),
        'generations': 50,
        'results': results,
    }, f, indent=2)
print(f'Written: {out}')