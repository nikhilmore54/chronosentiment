#!/usr/bin/env python3
import json
from collections import Counter

with open('/tmp/rec_latest.json') as f:
    data = json.load(f)

recs = data
if isinstance(data, dict):
    for k in ('recommendations', 'data', 'items', 'results'):
        if k in data:
            recs = data[k]
            break

# Extract SHORT population
shorts = [r for r in recs if str(r.get('direction', '')).upper() == 'SHORT']
print(f"=== SHORT POPULATION ANALYSIS ===")
print(f"Total SHORT decisions: {len(shorts)}")
print()

# evidence_class distribution
ec_counts = Counter(r.get('evidence_class', 'MISSING') for r in shorts)
print("Evidence Class Distribution:")
for ec, cnt in sorted(ec_counts.items()):
    print(f"  {ec}: {cnt}")
print()

# action distribution within SHORTs
action_counts = Counter(r.get('action', 'MISSING') for r in shorts)
print("Action Distribution (within SHORTs):")
for a, cnt in sorted(action_counts.items()):
    print(f"  {a}: {cnt}")
print()

# degradation_level distribution
deg_counts = Counter(r.get('degradation_level', 'MISSING') for r in shorts)
print("Degradation Level Distribution:")
for d, cnt in sorted(deg_counts.items()):
    print(f"  {d}: {cnt}")
print()

# Full per-ticker table
print(f"{'Ticker':<8} {'EvidClass':<14} {'Action':<10} {'RR':>6} {'TargRate':>9} {'DegLevel':<12} {'Trend':<10} {'Momentum'}")
print("-" * 90)
for r in sorted(shorts, key=lambda x: x.get('evidence_class', '') + x.get('instrument', '')):
    ticker = r.get('instrument', '?')
    ec = r.get('evidence_class', '?')
    action = r.get('action', '?')
    rr = r.get('adaptive_rr')
    tr = r.get('target_rate')
    deg = r.get('degradation_level', '?')
    trend = r.get('trend', '?')
    mom = r.get('momentum', '?')
    rr_str = f"{rr:.2f}" if rr is not None else "None"
    tr_str = f"{tr:.3f}" if tr is not None else "None"
    print(f"{ticker:<8} {ec:<14} {action:<10} {rr_str:>6} {tr_str:>9} {deg:<12} {trend:<10} {mom}")

print()
# Favourable SHORTs — would qualify as SELL under symmetric policy
fav_shorts = [r for r in shorts if r.get('evidence_class') == 'Favourable']
print(f"=== FAVOURABLE SHORTs (SELL candidates under symmetric policy): {len(fav_shorts)} ===")
for r in fav_shorts:
    print(f"  {r['instrument']}: RR={r.get('adaptive_rr')}, target_rate={r.get('target_rate')}, action={r.get('action')}, deg={r.get('degradation_level')}")

# R:R stats by evidence class
print()
print("=== R:R Stats by Evidence Class ===")
for ec in ['Favourable', 'Mixed', 'Unfavourable']:
    subset = [r for r in shorts if r.get('evidence_class') == ec]
    rrs = [r['adaptive_rr'] for r in subset if r.get('adaptive_rr') is not None]
    trs = [r['target_rate'] for r in subset if r.get('target_rate') is not None]
    if rrs:
        avg_rr = sum(rrs) / len(rrs)
        avg_tr = sum(trs) / len(trs) if trs else 0
        print(f"  {ec} (n={len(subset)}): RR min={min(rrs):.2f} max={max(rrs):.2f} avg={avg_rr:.2f} | TargetRate avg={avg_tr:.3f}")
    else:
        print(f"  {ec} (n={len(subset)}): no RR data")

# Also show all directions for context
print()
print("=== ALL DIRECTIONS (context) ===")
dir_counts = Counter(r.get('direction', 'MISSING') for r in recs)
for d, cnt in sorted(dir_counts.items()):
    print(f"  {d}: {cnt}")