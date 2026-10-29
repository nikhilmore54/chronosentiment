"""
Cross-Asset Ecological Law Comparison
Tests Pre-Bias Toxicity and Elasticity Age across asset classes.
Architecture FROZEN — no modifications.
"""
import json

datasets = {
    "Crypto (1m training)": "observatory/data.json",
    "Crypto OOS (5m)": "observatory/oos_data.json",
    "Equities (SPY/QQQ/NVDA)": "observatory/xasset_equities.json",
    "Commodities (Gold/Oil/Silver)": "observatory/xasset_commodities.json",
}

print("=" * 80)
print("  CROSS-ASSET ECOLOGICAL LAW COMPARISON — Architecture FROZEN")
print("=" * 80)

results = {}

for label, path in datasets.items():
    with open(path) as f:
        data = json.load(f)
    
    trades = data["trades"]
    s = data["summary"]
    
    if not trades or s["total_trades"] == 0:
        print(f"\n  {label}: NO TRADES — skipped")
        continue
    
    print(f"\n{'─'*80}")
    print(f"  {label}")
    print(f"  Trades: {s['total_trades']}  Win Rate: {s['win_rate']}%  Expectancy: {s['expectancy_bps']} bps")
    print(f"  Exit: {s['exit_distribution']}")
    
    r = {"label": label, "n": s["total_trades"]}
    
    # TEST: Pre-Bias Toxicity
    low = [t for t in trades if abs(t.get("bias", 99)) < 0.2]
    high = [t for t in trades if abs(t.get("bias", 0)) > 0.35]
    if low and high:
        lb = sum(t["pnl_bps"] for t in low) / len(low)
        hb = sum(t["pnl_bps"] for t in high) / len(high)
        lb_wr = len([t for t in low if t["pnl_bps"] > 0]) / len(low) * 100
        hb_wr = len([t for t in high if t["pnl_bps"] > 0]) / len(high) * 100
        survives = lb > hb
        r["bias_law"] = survives
        r["bias_detail"] = f"Low={lb:+.1f} ({lb_wr:.0f}% n={len(low)}), High={hb:+.1f} ({hb_wr:.0f}% n={len(high)})"
        sym = "✅" if survives else "❌"
        print(f"  Pre-Bias: {sym} Low bias={lb:+.1f} bps ({lb_wr:.0f}% WR, n={len(low)}), High bias={hb:+.1f} bps ({hb_wr:.0f}% WR, n={len(high)})")
    else:
        print(f"  Pre-Bias: ⚠️  Insufficient data (low={len(low)}, high={len(high)})")
        r["bias_law"] = None
    
    # TEST: Elasticity Age
    fresh = [t for t in trades if t.get("age", 99) <= 10]
    stale = [t for t in trades if t.get("age", 0) > 15]
    if fresh and stale:
        fp = sum(t["pnl_bps"] for t in fresh) / len(fresh)
        sp = sum(t["pnl_bps"] for t in stale) / len(stale)
        survives = fp > sp
        r["age_law"] = survives
        sym = "✅" if survives else "❌"
        print(f"  Age Decay: {sym} Fresh={fp:+.1f} bps (n={len(fresh)}), Stale={sp:+.1f} bps (n={len(stale)})")
    else:
        print(f"  Age Decay: ⚠️  Insufficient data (fresh={len(fresh)}, stale={len(stale)})")
        r["age_law"] = None
    
    # Topology summary by exit
    exits = {}
    for t in trades:
        e = t.get("exit_type", "?")
        if e not in exits: exits[e] = []
        exits[e].append(t)
    
    print(f"  {'Exit':<15} {'N':>4} {'Efficiency':>12} {'Bias':>8} {'Age':>6}")
    for ex in ["TakeProfit", "TrailingStop", "Mortality", "StopLoss"]:
        if ex in exits:
            g = exits[ex]
            n = len(g)
            eff = sum(t.get("eff", 0) for t in g) / n
            bias = sum(t.get("bias", 0) for t in g) / n
            age = sum(t.get("age", 0) for t in g) / n
            print(f"  {ex:<15} {n:>4} {eff:>12.4f} {bias:>+8.4f} {age:>6.1f}")
    
    results[label] = r

# === FINAL VERDICT ===
print(f"\n{'='*80}")
print(f"  UNIVERSALITY VERDICT")
print(f"{'='*80}")
print(f"\n  {'Asset Class':<35} {'Pre-Bias':>10} {'Age Decay':>10}")
print(f"  {'─'*55}")
for label, r in results.items():
    bias = "✅" if r.get("bias_law") == True else ("❌" if r.get("bias_law") == False else "⚠️")
    age = "✅" if r.get("age_law") == True else ("❌" if r.get("age_law") == False else "⚠️")
    print(f"  {label:<35} {bias:>10} {age:>10}")

print(f"\n  Pre-Bias Toxicity:")
survived = sum(1 for r in results.values() if r.get("bias_law") == True)
tested = sum(1 for r in results.values() if r.get("bias_law") is not None)
print(f"    Survived {survived}/{tested} ecologies")

print(f"\n  Elasticity Age:")
survived = sum(1 for r in results.values() if r.get("age_law") == True)
tested = sum(1 for r in results.values() if r.get("age_law") is not None)
print(f"    Survived {survived}/{tested} ecologies")
