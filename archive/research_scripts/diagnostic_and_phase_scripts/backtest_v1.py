"""
Backtest v1 — Historical Intelligence Backtest
===============================================
Replays the 459-decision historical universe through the v0.3 entry-time rules
with strict information-timeline enforcement. No look-ahead bias.

Conforms to: docs/INTELLIGENCE_CONTRACT_V1.md

Information available at each checkpoint:
  ENTRY:  direction, OQS, h60_classification
          momentum_persistence is NOT available at entry.
          Entry-time classification uses v0.3 OQS-based rules only.
  H120:   h120_ret, mfe_h120 — used for LONG WAIT-MID reassessment only
  H300:   outcome measured

Entry-time rules (v0.3, information-safe):
  SHORT ENTER:     h60_class == ENTER
  SHORT WAIT-HIGH: h60_class == WAIT AND OQS >= 50
  SHORT WAIT-LOW:  h60_class == WAIT AND OQS < 50
  SHORT AVOID:     h60_class == AVOID
  LONG ENTER:      h60_class == ENTER
  LONG WAIT-HIGH:  h60_class == WAIT AND OQS >= 65
  LONG WAIT-MID:   h60_class == WAIT AND 40 <= OQS < 65
  LONG WAIT-LOW:   h60_class == WAIT AND OQS < 40
  LONG AVOID:      h60_class == AVOID

H120 reassessment (LONG WAIT-MID only):
  h120_ret > +0.2%  → ENTER-LATE (act)
  h120_ret < -0.2%  → AVOID-LATE (do not act)
  mfe_h120 < 0.1%   → AVOID-LATE (override)
  otherwise         → WAIT-LATE  (continue monitoring, still act)

ACT states:  ENTER, WAIT-HIGH, ENTER-LATE, WAIT-LATE
NO-ACT:      WAIT-LOW, AVOID, AVOID-LATE
"""

import json
import statistics
import random
from collections import defaultdict

# ── Load data ─────────────────────────────────────────────────────────────────
data = json.load(open("datasets/p4_opportunity_dataset.json"))

THRESHOLD   = 0.002   # 0.2%
MFE_FLOOR   = 0.001   # 0.1%
CAPITAL     = 10_000  # ₹ notional per trade

# ── Entry-time classification (v0.3, information-safe) ───────────────────────
def entry_state(r):
    cls  = r["path_5m"]["h60_classification"]
    oqs  = r["opportunity_dimensions"]["opportunity_quality_score"]
    dirn = r["direction"]
    if cls == "ENTER": return "ENTER"
    if cls == "AVOID": return "AVOID"
    if dirn == "SHORT":
        return "WAIT-HIGH" if oqs >= 50 else "WAIT-LOW"
    else:
        if oqs >= 65:  return "WAIT-HIGH"
        if oqs >= 40:  return "WAIT-MID"
        return "WAIT-LOW"

# ── H120 reassessment (LONG WAIT-MID only) ───────────────────────────────────
def final_state(r, es):
    if r["direction"] != "LONG" or es != "WAIT-MID":
        return es
    h120 = r["path_5m"].get("h120_ret")
    mfe  = r["path_5m"].get("mfe_h120")
    if h120 is None:
        return "WAIT-LATE"
    if mfe is not None and mfe < MFE_FLOOR:
        return "AVOID-LATE"
    if h120 > THRESHOLD:  return "ENTER-LATE"
    if h120 < -THRESHOLD: return "AVOID-LATE"
    return "WAIT-LATE"

ACT_STATES = {"ENTER", "WAIT-HIGH", "ENTER-LATE", "WAIT-LATE"}

# ── Build trade list ──────────────────────────────────────────────────────────
trades = []
for r in data:
    h300 = r["path_5m"].get("h300_ret")
    if h300 is None:
        continue
    es = entry_state(r)
    fs = final_state(r, es)
    act = fs in ACT_STATES
    trades.append({
        "date":        r.get("cohort_date", "unknown"),
        "instrument":  r.get("instrument", "?"),
        "direction":   r["direction"],
        "entry_state": es,
        "final_state": fs,
        "acted":       act,
        "h300_ret":    h300,
        "daily_ret":   r.get("daily_return"),
        "pnl":         h300 * CAPITAL if act else 0.0,
        "oqs":         r["opportunity_dimensions"]["opportunity_quality_score"],
    })

# ── Stats helper ──────────────────────────────────────────────────────────────
def stats(returns):
    if not returns: return None
    returns = sorted(returns)
    n = len(returns)
    winners = [v for v in returns if v > 0.002]
    losers  = [v for v in returns if v < -0.002]
    pf = (sum(winners) / abs(sum(losers))) if losers and sum(losers) != 0 else None
    return {
        "n":        n,
        "mean":     statistics.mean(returns),
        "median":   statistics.median(returns),
        "win_rate": len(winners) / n,
        "avg_win":  statistics.mean(winners) if winners else 0.0,
        "avg_loss": statistics.mean(losers) if losers else 0.0,
        "pf":       pf,
        "cum":      sum(returns),
    }

def pct(v, d=2):
    if v is None: return "—"
    return f"{v*100:+.{d}f}%"

def fmt_pf(v):
    if v is None: return "—"
    if v > 99: return ">99x"
    return f"{v:.2f}x"

# ─────────────────────────────────────────────────────────────────────────────
# SECTION 1 — Per-state performance
# ─────────────────────────────────────────────────────────────────────────────
print("=" * 80)
print("BACKTEST v1 — Per-state H300 performance")
print("Entry rules: v0.3 (information-safe). H120 reassessment for LONG WAIT-MID.")
print("=" * 80)
print()

STATE_ORDER = ["ENTER", "WAIT-HIGH", "WAIT-MID", "ENTER-LATE", "WAIT-LATE",
               "WAIT-LOW", "AVOID-LATE", "AVOID"]

for direction in ["LONG", "SHORT"]:
    print(f"{direction}")
    print(f"  {'State':<14} {'N':>4} {'Acted':>6} {'Mean':>8} {'Median':>8} "
          f"{'Win%':>7} {'AvgWin':>8} {'AvgLoss':>9} {'PF':>7} {'CumRet':>9}")
    print(f"  {'-'*80}")
    for state in STATE_ORDER:
        group = [t for t in trades if t["direction"] == direction
                 and t["final_state"] == state]
        if not group: continue
        acted = sum(1 for t in group if t["acted"])
        s = stats([t["h300_ret"] for t in group])
        if not s: continue
        print(f"  {state:<14} {s['n']:>4} {acted:>6} {pct(s['mean']):>8} "
              f"{pct(s['median']):>8} {s['win_rate']*100:>6.1f}% "
              f"{pct(s['avg_win']):>8} {pct(s['avg_loss']):>9} "
              f"{fmt_pf(s['pf']):>7} {pct(s['cum']):>9}")
    print()

# ─────────────────────────────────────────────────────────────────────────────
# SECTION 2 — Strategy vs baselines
# ─────────────────────────────────────────────────────────────────────────────
print("=" * 80)
print("STRATEGY vs BASELINES — Cumulative H300 return on acted trades")
print("=" * 80)
print()

strategy = [t for t in trades if t["acted"]]
random.seed(42)

baselines = [
    ("Strategy (ACT states)",  [t["h300_ret"] for t in strategy]),
    ("All decisions",          [t["h300_ret"] for t in trades]),
    ("ENTER only",             [t["h300_ret"] for t in trades if t["entry_state"] == "ENTER"]),
    ("All LONG",               [t["h300_ret"] for t in trades if t["direction"] == "LONG"]),
    ("All SHORT",              [t["h300_ret"] for t in trades if t["direction"] == "SHORT"]),
    ("Random 50%",             [t["h300_ret"] for t in trades if random.random() < 0.5]),
]

print(f"  {'Strategy':<28} {'N':>4} {'Mean':>8} {'Median':>8} {'Win%':>7} "
      f"{'PF':>7} {'CumRet':>9} {'CumPnL':>12}")
print(f"  {'-'*85}")
for label, rets in baselines:
    s = stats(rets)
    if not s: continue
    cum_pnl = s["cum"] * CAPITAL
    print(f"  {label:<28} {s['n']:>4} {pct(s['mean']):>8} {pct(s['median']):>8} "
          f"{s['win_rate']*100:>6.1f}% {fmt_pf(s['pf']):>7} {pct(s['cum']):>9} "
          f"₹{cum_pnl:>10,.0f}")

# ─────────────────────────────────────────────────────────────────────────────
# SECTION 3 — Attribution tree
# ─────────────────────────────────────────────────────────────────────────────
print()
print("=" * 80)
print("ATTRIBUTION — Where does strategy P&L come from?")
print("=" * 80)
print()

total_pnl = sum(t["pnl"] for t in strategy)
print(f"  Total strategy P&L: ₹{total_pnl:,.0f}  ({len(strategy)} trades)")
print()
print(f"  {'Direction + State':<30} {'N':>4} {'P&L (₹)':>12} {'Share':>8} {'Mean ret':>10}")
print(f"  {'-'*68}")
for direction in ["LONG", "SHORT"]:
    for state in STATE_ORDER:
        group = [t for t in strategy
                 if t["direction"] == direction and t["final_state"] == state]
        if not group: continue
        pnl = sum(t["pnl"] for t in group)
        share = pnl / total_pnl * 100 if total_pnl != 0 else 0
        mean_ret = statistics.mean(t["h300_ret"] for t in group)
        print(f"  {direction+' '+state:<30} {len(group):>4} "
              f"₹{pnl:>10,.0f} {share:>7.1f}% {pct(mean_ret):>10}")

# ─────────────────────────────────────────────────────────────────────────────
# SECTION 4 — Equity curve by cohort date
# ─────────────────────────────────────────────────────────────────────────────
print()
print("=" * 80)
print("EQUITY CURVE — Cumulative P&L by cohort date")
print("=" * 80)
print()

by_date = defaultdict(list)
for t in strategy:
    by_date[t["date"]].append(t["pnl"])

cum = 0.0
print(f"  {'Date':<12} {'Trades':>7} {'Day P&L':>12} {'Cum P&L':>12} {'Cum Ret':>10}")
print(f"  {'-'*57}")
for date in sorted(by_date.keys()):
    day_pnl = sum(by_date[date])
    cum += day_pnl
    n = len(by_date[date])
    # Cumulative return relative to total capital deployed across all strategy trades
    cum_ret = cum / (len(strategy) * CAPITAL) if strategy else 0
    print(f"  {date:<12} {n:>7} ₹{day_pnl:>10,.0f} ₹{cum:>10,.0f} {cum_ret*100:>+9.2f}%")

# ─────────────────────────────────────────────────────────────────────────────
# SECTION 5 — Summary card
# ─────────────────────────────────────────────────────────────────────────────
print()
print("=" * 80)
print("SUMMARY CARD")
print("=" * 80)

s_strat = stats([t["h300_ret"] for t in strategy])
s_all   = stats([t["h300_ret"] for t in trades])
s_enter = stats([t["h300_ret"] for t in trades if t["entry_state"] == "ENTER"])

print(f"""
  Capital per trade:    ₹{CAPITAL:,}
  Universe:             {len(trades)} decisions (Aug 20 – Sep 7)
  Strategy trades:      {len(strategy)} ({len(strategy)/len(trades)*100:.1f}% of universe)

  STRATEGY (ACT states)
    Mean trade return:  {pct(s_strat['mean'])}
    Median return:      {pct(s_strat['median'])}
    Win rate:           {s_strat['win_rate']*100:.1f}%
    Avg win:            {pct(s_strat['avg_win'])}
    Avg loss:           {pct(s_strat['avg_loss'])}
    Profit factor:      {fmt_pf(s_strat['pf'])}
    Cumulative return:  {pct(s_strat['cum'])}
    Total P&L:          ₹{s_strat['cum']*CAPITAL:,.0f}

  vs ALL DECISIONS
    Mean trade return:  {pct(s_all['mean'])}
    Win rate:           {s_all['win_rate']*100:.1f}%
    Profit factor:      {fmt_pf(s_all['pf'])}
    Cumulative return:  {pct(s_all['cum'])}

  vs ENTER ONLY
    Mean trade return:  {pct(s_enter['mean'])}
    Win rate:           {s_enter['win_rate']*100:.1f}%
    Profit factor:      {fmt_pf(s_enter['pf'])}
    Cumulative return:  {pct(s_enter['cum'])}
""")

# ─────────────────────────────────────────────────────────────────────────────
# SECTION 6 — Transaction cost sensitivity
# ─────────────────────────────────────────────────────────────────────────────
print("=" * 80)
print("TRANSACTION COST SENSITIVITY")
print("=" * 80)
print()
base_rets = [t["h300_ret"] for t in strategy]
print(f"  {'Cost (bps)':>12} {'Net mean':>10} {'Net win%':>10} {'Net PF':>8} {'Net cum':>10}")
print(f"  {'-'*54}")
for cost_bps in [0, 5, 10, 20, 30, 50]:
    cost = cost_bps / 10_000
    adj  = [r - cost for r in base_rets]
    s2   = stats(adj)
    if not s2: continue
    print(f"  {cost_bps:>12} {pct(s2['mean']):>10} {s2['win_rate']*100:>9.1f}% "
          f"{fmt_pf(s2['pf']):>8} {pct(s2['cum']):>10}")