"""
Scale vs Regime Disentanglement
Four conditions testing whether Smoothness Trap failure is resolution or regime:

  A: Training regime, 1m  (original finding)
  B: Training regime, 5m  (same regime, different scale)
  C: OOS regime, 5m       (different regime, same scale as B)
  D: Equities, 5m         (different asset, same scale)

If Smoothness Trap fails in B but holds in A → scale caused failure
If Smoothness Trap fails in B AND C but holds in D → crypto regime caused failure
"""
import json

CONDITIONS = {
    "A: Crypto 1m (training)":   "observatory/data.json",
    "B: Crypto 5m (same regime)": "observatory/training_5m_data.json",
    "C: Crypto 5m (OOS regime)": "observatory/oos_data.json",
    "D: Equities 5m":            "observatory/xasset_equities.json",
    "E: Commodities 5m":         "observatory/xasset_commodities.json",
}

def avg(lst, k): return sum(t.get(k, 0) for t in lst) / len(lst) if lst else 0
def wr(lst): return len([t for t in lst if t.get("pnl_bps", 0) > 0]) / len(lst) * 100 if lst else 0

print("=" * 80)
print("  SCALE vs REGIME DISENTANGLEMENT")
print("  Question: Did the Smoothness Trap fail due to 5m resolution or OOS regime?")
print("=" * 80)

# ── Table 1: Smoothness Trap survival ──────────────────────────────────────
print("\n  SMOOTHNESS TRAP: Efficiency ordering TP → TrailingStop → Mortality → SL")
print(f"\n  {'Condition':<35} {'TP':>7} {'TS':>7} {'Mort':>7} {'SL':>7} {'Mono?':>7} {'N':>5}")
print(f"  {'─'*75}")

trap_results = {}
for label, path in CONDITIONS.items():
    with open(path) as f: data = json.load(f)
    trades = data["trades"]
    if not trades: continue
    by_exit = {}
    for t in trades:
        e = t.get("exit_type", "?")
        by_exit.setdefault(e, []).append(t)
    effs = {e: avg(by_exit[e], "eff") for e in by_exit if e in ["TakeProfit","TrailingStop","Mortality","StopLoss"]}
    order = [effs.get(e, None) for e in ["TakeProfit","TrailingStop","Mortality","StopLoss"]]
    valid = [v for v in order if v is not None]
    monotonic = all(valid[i] <= valid[i+1] for i in range(len(valid)-1)) if len(valid) >= 2 else False
    trap_results[label] = monotonic
    cells = [f"{effs[e]:.3f}" if e in effs else "  —  " for e in ["TakeProfit","TrailingStop","Mortality","StopLoss"]]
    sym = "✅" if monotonic else "❌"
    n = sum(len(v) for v in by_exit.values())
    print(f"  {label:<35} {cells[0]:>7} {cells[1]:>7} {cells[2]:>7} {cells[3]:>7} {sym:>7} {n:>5}")

# ── Table 2: Pre-Bias Toxicity ─────────────────────────────────────────────
print(f"\n  PRE-BIAS TOXICITY: Low bias (<0.2) outperforms high bias (>0.35)?")
print(f"\n  {'Condition':<35} {'Low PnL':>9} {'Low WR':>8} {'High PnL':>9} {'High WR':>8} {'Holds?':>7}")
print(f"  {'─'*80}")

bias_results = {}
for label, path in CONDITIONS.items():
    with open(path) as f: data = json.load(f)
    trades = data["trades"]
    if not trades: continue
    low  = [t for t in trades if "bias" in t and abs(t["bias"]) < 0.20]
    high = [t for t in trades if "bias" in t and abs(t["bias"]) > 0.35]
    if not low or not high:
        bias_results[label] = None
        print(f"  {label:<35} {'— (n<3)':>9}")
        continue
    lp, hw = avg(low, "pnl_bps"), avg(high, "pnl_bps")
    holds = lp > hw
    bias_results[label] = holds
    sym = "✅" if holds else "❌"
    print(f"  {label:<35} {lp:>+9.1f} {wr(low):>7.0f}%  {hw:>+9.1f} {wr(high):>7.0f}%  {sym:>7}")

# ── Table 3: Elasticity Age ────────────────────────────────────────────────
print(f"\n  ELASTICITY AGE: Fresh (≤10 bars) outperforms stale (>15 bars)?")
print(f"\n  {'Condition':<35} {'Fresh PnL':>10} {'Fresh n':>8} {'Stale PnL':>10} {'Stale n':>8} {'Holds?':>7}")
print(f"  {'─'*80}")

age_results = {}
for label, path in CONDITIONS.items():
    with open(path) as f: data = json.load(f)
    trades = data["trades"]
    if not trades: continue
    fresh = [t for t in trades if "age" in t and t["age"] <= 10]
    stale = [t for t in trades if "age" in t and t["age"] >  15]
    if not fresh or not stale:
        age_results[label] = None
        print(f"  {label:<35} {'— (insufficient)':>28}")
        continue
    fp, sp = avg(fresh, "pnl_bps"), avg(stale, "pnl_bps")
    holds = fp > sp
    age_results[label] = holds
    sym = "✅" if holds else "❌"
    print(f"  {label:<35} {fp:>+10.1f} {len(fresh):>8} {sp:>+10.1f} {len(stale):>8} {sym:>7}")

# ── Disentanglement Verdict ────────────────────────────────────────────────
print(f"\n{'='*80}")
print(f"  DISENTANGLEMENT VERDICT")
print(f"{'='*80}")

# Smoothness Trap
trap_A = trap_results.get("A: Crypto 1m (training)")
trap_B = trap_results.get("B: Crypto 5m (same regime)")
trap_C = trap_results.get("C: Crypto 5m (OOS regime)")
trap_D = trap_results.get("D: Equities 5m")

print(f"\n  SMOOTHNESS TRAP:")
print(f"    A (Crypto 1m):  {'✅' if trap_A else '❌'}  B (Crypto 5m same regime): {'✅' if trap_B else '❌'}")
print(f"    C (Crypto 5m OOS): {'✅' if trap_C else '❌'}  D (Equities 5m): {'✅' if trap_D else '❌'}")

if trap_A and not trap_B:
    print(f"    → RESOLUTION EFFECT confirmed: same regime, same assets, 5m fails")
    if trap_D:
        print(f"    → But Equities 5m holds → topology signal is NOT purely crypto-specific")
        print(f"    → HYPOTHESIS: Smoothness Trap is resolution-sensitive in crypto but preserved in equities")
elif trap_A and trap_B and not trap_C:
    print(f"    → REGIME EFFECT confirmed: same scale (5m) fails only on OOS regime → regime-local")
elif trap_A and trap_B and trap_C:
    print(f"    → STRONG: Smoothness Trap survives BOTH resolution and regime change → robust")

print(f"\n  PRE-BIAS TOXICITY:")
b_A = bias_results.get("A: Crypto 1m (training)")
b_B = bias_results.get("B: Crypto 5m (same regime)")
b_C = bias_results.get("C: Crypto 5m (OOS regime)")
b_D = bias_results.get("D: Equities 5m")
b_E = bias_results.get("E: Commodities 5m")
survived = sum(1 for v in [b_A, b_B, b_C, b_D, b_E] if v is True)
tested   = sum(1 for v in [b_A, b_B, b_C, b_D, b_E] if v is not None)
print(f"    Survived {survived}/{tested} conditions")
if all(v for v in [b_A, b_B, b_C, b_D] if v is not None):
    print(f"    → STRONG candidate for liquidity-flow universal law")
if b_E is False:
    print(f"    → Commodities INVERT → event-driven ecology has different physics")

print(f"\n  SUMMARY TABLE:")
print(f"  {'Condition':<35} {'S.Trap':>8} {'Pre-Bias':>9} {'Age':>6}")
print(f"  {'─'*60}")
for label in CONDITIONS:
    t = "✅" if trap_results.get(label) else ("❌" if label in trap_results else "—")
    b = "✅" if bias_results.get(label) else ("❌" if bias_results.get(label) is False else "⚠️")
    a = "✅" if age_results.get(label) else ("❌" if age_results.get(label) is False else "⚠️")
    print(f"  {label:<35} {t:>8} {b:>9} {a:>6}")
