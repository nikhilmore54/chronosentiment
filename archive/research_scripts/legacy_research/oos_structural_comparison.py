"""
OOS vs Training: Structural Property Comparison
Tests whether topology laws survive outside the training window.
Architecture is FROZEN — no modifications allowed.
"""
import json

def load_data(path):
    with open(path) as f:
        return json.load(f)

def analyze(data, label):
    trades = data["trades"]
    s = data["summary"]
    
    print(f"\n{'='*70}")
    print(f"  {label}")
    print(f"{'='*70}")
    print(f"  Trades: {s['total_trades']}  |  Win Rate: {s['win_rate']}%  |  Expectancy: {s['expectancy_bps']} bps")
    print(f"  Avg Win: {s['avg_win_bps']} bps  |  Avg Loss: {s['avg_loss_bps']} bps  |  ERR: {s['elastic_recovery_ratio']}")
    print(f"  Exit: {s['exit_distribution']}")
    
    # Topology by exit type
    exits = {}
    for t in trades:
        e = t.get("exit_type", "?")
        if e not in exits:
            exits[e] = []
        exits[e].append(t)
    
    print(f"\n  {'Exit':<15} {'N':>4} {'Avg PnL':>10} {'Efficiency':>12} {'Comp':>8} {'Bias':>8} {'Age':>6}")
    print(f"  {'-'*65}")
    
    for exit_type in ["TakeProfit", "TrailingStop", "Mortality", "StopLoss"]:
        if exit_type not in exits:
            continue
        g = exits[exit_type]
        n = len(g)
        avg = lambda k: sum(t.get(k, 0) for t in g) / n
        print(f"  {exit_type:<15} {n:>4} {avg('pnl_bps'):>+10.1f} {avg('eff'):>12.4f} {avg('comp'):>8.3f} {avg('bias'):>+8.4f} {avg('age'):>6.1f}")
    
    return exits

print("🧪 OUT-OF-SAMPLE VALIDATION — Architecture FROZEN")
print("   Testing: Do ecological laws survive outside the training window?")

train = load_data("observatory/data.json")
oos = load_data("observatory/oos_data.json")

train_exits = analyze(train, "TRAINING: 1m, April 18 → May 18 (30 days)")
oos_exits = analyze(oos, "OOS: 5m, March 19 → April 18 (30 days, non-overlapping)")

# === STRUCTURAL COMPARISON ===
print(f"\n{'='*70}")
print(f"  STRUCTURAL PROPERTY COMPARISON")
print(f"{'='*70}")

# Test 1: Smoothness Trap (monotonic efficiency inversion)
print("\n  TEST 1: Smoothness Trap — Does efficiency increase from TP → SL?")
for label, exits in [("Training", train_exits), ("OOS", oos_exits)]:
    effs = {}
    for exit_type in ["TakeProfit", "TrailingStop", "Mortality", "StopLoss"]:
        if exit_type in exits:
            g = exits[exit_type]
            effs[exit_type] = sum(t.get("eff", 0) for t in g) / len(g)
    
    vals = [effs.get(e, 0) for e in ["TakeProfit", "TrailingStop", "Mortality", "StopLoss"] if e in effs]
    monotonic = all(vals[i] <= vals[i+1] for i in range(len(vals)-1))
    print(f"    {label}: {' → '.join(f'{e}={effs[e]:.3f}' for e in ['TakeProfit','TrailingStop','Mortality','StopLoss'] if e in effs)}")
    print(f"    Monotonic: {'✅ YES' if monotonic else '❌ NO'}")

# Test 2: Genesis Compression (TP enters from lower compression)
print("\n  TEST 2: Genesis Compression — Do winners enter from compression?")
for label, data in [("Training", train), ("OOS", oos)]:
    winners = [t for t in data["trades"] if t.get("pnl_bps", 0) > 0 and "comp" in t]
    losers = [t for t in data["trades"] if t.get("pnl_bps", 0) <= 0 and "comp" in t]
    if winners and losers:
        w_comp = sum(t["comp"] for t in winners) / len(winners)
        l_comp = sum(t["comp"] for t in losers) / len(losers)
        print(f"    {label}: Winner comp={w_comp:.3f}, Loser comp={l_comp:.3f}, Delta={w_comp-l_comp:+.3f}")

# Test 3: Pre-Bias Toxicity (higher bias = worse outcomes)
print("\n  TEST 3: Pre-Bias — Does established direction predict worse outcomes?")
for label, data in [("Training", train), ("OOS", oos)]:
    low_bias = [t for t in data["trades"] if abs(t.get("bias", 0)) < 0.2 and "bias" in t]
    high_bias = [t for t in data["trades"] if abs(t.get("bias", 0)) > 0.35 and "bias" in t]
    if low_bias and high_bias:
        lb_pnl = sum(t["pnl_bps"] for t in low_bias) / len(low_bias)
        hb_pnl = sum(t["pnl_bps"] for t in high_bias) / len(high_bias)
        lb_wr = len([t for t in low_bias if t["pnl_bps"] > 0]) / len(low_bias) * 100
        hb_wr = len([t for t in high_bias if t["pnl_bps"] > 0]) / len(high_bias) * 100
        survives = lb_pnl > hb_pnl
        print(f"    {label}: Low bias={lb_pnl:+.1f} bps ({lb_wr:.0f}% WR), High bias={hb_pnl:+.1f} bps ({hb_wr:.0f}% WR) {'✅' if survives else '❌'}")

# Test 4: Elasticity Age (older = more toxic)
print("\n  TEST 4: Elasticity Age — Are stale entries more toxic?")
for label, data in [("Training", train), ("OOS", oos)]:
    fresh = [t for t in data["trades"] if t.get("age", 0) <= 10 and "age" in t]
    stale = [t for t in data["trades"] if t.get("age", 0) > 15 and "age" in t]
    if fresh and stale:
        f_pnl = sum(t["pnl_bps"] for t in fresh) / len(fresh)
        s_pnl = sum(t["pnl_bps"] for t in stale) / len(stale)
        survives = f_pnl > s_pnl
        print(f"    {label}: Fresh={f_pnl:+.1f} bps (n={len(fresh)}), Stale={s_pnl:+.1f} bps (n={len(stale)}) {'✅' if survives else '❌'}")

print(f"\n{'='*70}")
print(f"  VERDICT")
print(f"{'='*70}")
