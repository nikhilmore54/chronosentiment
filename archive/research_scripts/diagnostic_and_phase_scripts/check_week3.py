import json
from pathlib import Path
ROOT = Path('/Users/nikhil/ChronoSentiment_MEGA_FINAL')
with open(ROOT / 'benchmarks/ultracrew/UB-001-v1.0.json') as f:
    ub = json.load(f)
shift_types = [('Morning', 7, 8), ('Evening', 15, 8), ('Night', 23, 8)]
for week in range(4):
    shifts = []
    for day in range(7):
        dt = 'weekend' if day in [5, 6] else 'weekday'
        cov = ub['coverage_requirements'][dt]
        for sn, sh, dur in shift_types:
            for skill, cnt in cov[sn].items():
                for _ in range(cnt):
                    shifts.append(dur)
    total_h = sum(shifts)
    n = 20
    mean = total_h / n
    base = total_h // (n * 8)
    rem = (total_h // 8) % n
    high_h = (base + 1) * 8
    low_h = base * 8
    hours = [high_h]*rem + [low_h]*(n-rem)
    mean2 = sum(hours)/n
    var = sum((h-mean2)**2 for h in hours)/n
    sc1 = var * 10
    print(f'Week {week+1}: {len(shifts)} shifts, {total_h}h, mean={mean:.2f}, bound: {rem}x{high_h}h + {n-rem}x{low_h}h, SC1={sc1:.4f}, max_fitness={10000-sc1:.4f}')