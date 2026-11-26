#!/usr/bin/env python3
"""Generate REC-BASELINE-001 from /recommendations/v1/latest JSON."""
import json
import sys
from collections import Counter

with open('/tmp/rec_latest.json') as f:
    d = json.load(f)

recs = d.get('recommendations', [])
buy = sorted([r for r in recs if r.get('action') == 'Buy'], key=lambda x: -x.get('rank_score', 0))
watch = sorted([r for r in recs if r.get('action') == 'Watch'], key=lambda x: -x.get('rank_score', 0))
no_trade = sorted([r for r in recs if r.get('action') == 'NoTrade'], key=lambda x: -x.get('rank_score', 0))

print(f"TOTAL={len(recs)} BUY={len(buy)} WATCH={len(watch)} NO_TRADE={len(no_trade)}")
print()

# Distributions (actionable only)
actionable = buy + watch
scores = [r.get('rank_score', 0) for r in actionable]
rrs = [r.get('adaptive_rr', 0) for r in actionable if r.get('adaptive_rr')]
hors = [r.get('adaptive_horizon_sessions', 0) for r in actionable if r.get('adaptive_horizon_sessions')]
ns = [r.get('sample_size', 0) for r in actionable]
rates = [r.get('target_rate', 0) for r in actionable]
degs = Counter(r.get('degradation_level', '?') for r in actionable)
states_all = Counter(r.get('coralys_state', '?') for r in recs)
dirs_all = Counter(r.get('direction', '?') for r in recs)

def pct(lst, p):
    lst2 = sorted(lst)
    if not lst2:
        return 0
    idx = max(0, min(len(lst2)-1, int(len(lst2)*p/100)))
    return lst2[idx]

def mean(lst):
    return sum(lst)/len(lst) if lst else 0

print("=== DISTRIBUTIONS (actionable only, n=60) ===")
print(f"rank_score:        min={min(scores):.4f}  p25={pct(scores,25):.4f}  median={pct(scores,50):.4f}  p75={pct(scores,75):.4f}  max={max(scores):.4f}  mean={mean(scores):.4f}")
print(f"adaptive_rr:       min={min(rrs):.2f}  p25={pct(rrs,25):.2f}  median={pct(rrs,50):.2f}  p75={pct(rrs,75):.2f}  max={max(rrs):.2f}  mean={mean(rrs):.2f}")
print(f"horizon_sessions:  min={min(hors):.1f}  p25={pct(hors,25):.1f}  median={pct(hors,50):.1f}  p75={pct(hors,75):.1f}  max={max(hors):.1f}  mean={mean(hors):.1f}")
print(f"sample_size:       min={min(ns)}  p25={pct(ns,25)}  median={pct(ns,50)}  p75={pct(ns,75)}  max={max(ns)}  mean={mean(ns):.1f}")
print(f"target_rate:       min={min(rates):.3f}  p25={pct(rates,25):.3f}  median={pct(rates,50):.3f}  p75={pct(rates,75):.3f}  max={max(rates):.3f}  mean={mean(rates):.3f}")
print(f"degradation:       {dict(sorted(degs.items()))}")
print(f"coralys_state:     {dict(sorted(states_all.items()))}")
print(f"direction:         {dict(sorted(dirs_all.items()))}")
print()

print("=== BUY (rank_score desc) ===")
print(f"{'ticker':<20} {'ref':>10} {'tgt':>10} {'risk':>10} {'rr':>5} {'hor':>5} {'n':>5} {'deg':<14} {'rate':>6} {'score':>7}  state  dir")
for r in buy:
    print(f"{r['instrument']:<20} {r.get('reference_price',0):>10.2f} {r.get('adaptive_target',0):>10.2f} {r.get('adaptive_risk',0):>10.2f} {r.get('adaptive_rr',0):>5.2f} {r.get('adaptive_horizon_sessions',0):>5.1f} {r.get('sample_size',0):>5d} {r.get('degradation_level','?'):<14} {r.get('target_rate',0):>6.3f} {r.get('rank_score',0):>7.4f}  {r.get('coralys_state','?')}  {r.get('direction','?')}")
print()

print("=== WATCH (rank_score desc) ===")
print(f"{'ticker':<20} {'ref':>10} {'tgt':>10} {'risk':>10} {'rr':>5} {'hor':>5} {'n':>5} {'deg':<14} {'rate':>6} {'score':>7}  state  dir")
for r in watch:
    print(f"{r['instrument']:<20} {r.get('reference_price',0):>10.2f} {r.get('adaptive_target',0):>10.2f} {r.get('adaptive_risk',0):>10.2f} {r.get('adaptive_rr',0):>5.2f} {r.get('adaptive_horizon_sessions',0):>5.1f} {r.get('sample_size',0):>5d} {r.get('degradation_level','?'):<14} {r.get('target_rate',0):>6.3f} {r.get('rank_score',0):>7.4f}  {r.get('coralys_state','?')}  {r.get('direction','?')}")
print()

print("=== NO_TRADE ===")
print(f"{'ticker':<20} {'ref':>10} {'score':>7}  state  dir  deg")
for r in no_trade:
    print(f"{r['instrument']:<20} {r.get('reference_price',0):>10.2f} {r.get('rank_score',0):>7.4f}  {r.get('coralys_state','?')}  {r.get('direction','?')}  {r.get('degradation_level','?')}")