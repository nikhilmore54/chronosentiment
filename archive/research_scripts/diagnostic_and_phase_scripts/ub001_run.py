#!/usr/bin/env python3
"""
UB-001 Baseline Run Script
Builds the canonical benchmark payload from UB-001-v1.0.json,
POSTs to the ultracrew_server, and writes the result to
benchmarks/ultracrew/UB-001-BASELINE-v1.0.json
"""
import json
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).parent.parent

def build_week_payload(ub, week_index, generation_limit=200):
    """Build a single-week payload (start_hour 0..167) for the given week index."""
    workers = [{'id': i+1, 'skills': s['skills']} for i, s in enumerate(ub['staff'])]

    shifts = []
    sid = 1
    shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]
    # 7 days per week, start_hour within 0..167
    for day_in_week in range(7):
        abs_day = week_index * 7 + day_in_week
        day_type = 'weekend' if day_in_week in [5, 6] else 'weekday'
        cov = ub['coverage_requirements'][day_type]
        for shift_name, start_h, dur in shift_types:
            slot_cov = cov[shift_name]
            for skill, count in slot_cov.items():
                for _ in range(count):
                    hour = day_in_week * 24 + start_h  # 0..167
                    shifts.append({
                        'id': sid,
                        'start_hour': hour,
                        'duration_hours': dur,
                        'required_skill': skill
                    })
                    sid += 1

    return workers, shifts, {
        'workers': workers,
        'shifts': shifts,
        'historical_workloads': None,
        'rng_seed': 42 + week_index,
        'generation_limit': generation_limit
    }

def post_payload(payload):
    req = urllib.request.Request(
        'http://localhost:3001/api/schedule',
        data=json.dumps(payload).encode(),
        headers={'Content-Type': 'application/json'},
        method='POST'
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read())


def run():
    with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
        ub = json.load(f)

    NUM_WEEKS = 4
    week_results = []
    total_shifts_all = 0
    total_hc1 = total_hc2 = total_hc3 = total_rest = 0
    total_runtime_ms = 0
    all_valid = True

    for week in range(NUM_WEEKS):
        workers, shifts, payload = build_week_payload(ub, week, generation_limit=200)
        total_shifts_all += len(shifts)
        print(f'Week {week+1}/4: {len(workers)} workers, {len(shifts)} shifts — POSTing...', flush=True)

        d = post_payload(payload)
        cr = d['constraint_report']
        gens = d['telemetry']['generations']

        total_hc1 += cr['hc1_violations']
        total_hc2 += cr['hc2_violations']
        total_hc3 += cr['hc3_violations']
        total_rest += cr['rest_violations']
        total_runtime_ms += gens[-1]['elapsed_time_ms']
        if not cr['is_valid']:
            all_valid = False

        # Generation profile for this week
        checkpoints = [0, 24, 49, 99, 149, 199]
        profile = []
        for i in checkpoints:
            if i < len(gens):
                g = gens[i]
                profile.append({
                    'generation': g['generation'],
                    'best_fitness': g['best_fitness'],
                    'average_fitness': g['average_fitness'],
                    'hard_violations': g['hard_violations'],
                    'elapsed_time_ms': g['elapsed_time_ms']
                })

        week_results.append({
            'week': week + 1,
            'shifts': len(shifts),
            'hc1': cr['hc1_violations'],
            'hc2': cr['hc2_violations'],
            'hc3': cr['hc3_violations'],
            'rest': cr['rest_violations'],
            'is_valid': cr['is_valid'],
            'best_fitness': d['metrics']['fitness'],
            'runtime_ms': gens[-1]['elapsed_time_ms'],
            'generation_profile': profile,
            'convergence': _assess_convergence(profile)
        })

        print(f'  HC1={cr["hc1_violations"]} HC2={cr["hc2_violations"]} HC3={cr["hc3_violations"]} '
              f'Rest={cr["rest_violations"]} Valid={cr["is_valid"]} '
              f'Fitness={d["metrics"]["fitness"]:.1f} Runtime={gens[-1]["elapsed_time_ms"]}ms', flush=True)

    # Aggregate PAS
    hard_total = total_hc1 + total_hc2 + total_hc3 + total_rest
    pas = round(max(0.0, (total_shifts_all - hard_total) / total_shifts_all * 100), 1) if total_shifts_all > 0 else 0.0

    baseline = {
        'benchmark': 'UB-001-v1.0',
        'run_date': '2026-07-12',
        'optimizer_commit': '44b29cec',
        'hypothesis': 'H1-skill-aware-initialization',
        'note': '28-day benchmark run as 4 independent weekly optimizations (server model is single-week)',
        'config': {
            'workers': len(workers),
            'total_shifts': total_shifts_all,
            'weeks': NUM_WEEKS,
            'generation_limit': 200
        },
        'aggregate': {
            'hc1_violations': total_hc1,
            'hc2_violations': total_hc2,
            'hc3_violations': total_hc3,
            'rest_violations': total_rest,
            'all_weeks_valid': all_valid,
            'pas_estimate': pas,
            'total_runtime_ms': total_runtime_ms
        },
        'weeks': week_results
    }

    out_path = ROOT / 'benchmarks/ultracrew/UB-001-BASELINE-v1.0.json'
    with open(out_path, 'w') as f:
        json.dump(baseline, f, indent=2)

    # Print summary
    print()
    print('=' * 60)
    print('UB-001 BASELINE RUN COMPLETE — 4 weeks aggregated')
    print('=' * 60)
    print(f'Total shifts: {total_shifts_all}  Workers: 20  Weeks: 4')
    print(f'HC1={total_hc1} HC2={total_hc2} HC3={total_hc3} Rest={total_rest}')
    print(f'All valid={all_valid}  PAS≈{pas}%  Total runtime={total_runtime_ms}ms')
    print()
    print(f'{"Wk":>3} {"Shifts":>7} {"HC1":>4} {"HC2":>4} {"HC3":>4} {"Rest":>5} {"Valid":>6} {"Fitness":>9} {"ms":>7} {"Convergence"}')
    print('-' * 75)
    for w in week_results:
        print(f'{w["week"]:>3} {w["shifts"]:>7} {w["hc1"]:>4} {w["hc2"]:>4} {w["hc3"]:>4} {w["rest"]:>5} '
              f'{str(w["is_valid"]):>6} {w["best_fitness"]:>9.1f} {w["runtime_ms"]:>7} {w["convergence"]}')
    print()
    print(f'Written: {out_path}')
    sys.stdout.flush()

def _assess_convergence(profile):
    if len(profile) < 3:
        return 'insufficient_data'
    early = profile[1]['best_fitness'] if len(profile) > 1 else profile[0]['best_fitness']
    mid   = profile[2]['best_fitness'] if len(profile) > 2 else early
    late  = profile[-1]['best_fitness']
    delta_early_to_mid  = mid - early
    delta_mid_to_late   = late - mid
    if delta_mid_to_late < 1.0 and delta_early_to_mid > 10.0:
        return 'early_convergence'
    elif delta_mid_to_late > 50.0:
        return 'still_improving'
    elif late >= 9990.0:
        return 'saturated'
    else:
        return 'gradual_improvement'

if __name__ == '__main__':
    run()