#!/usr/bin/env python3
"""
v0.4.1 Regime-by-Regime Stop-Loss Analysis
Primary research question: How does Coralys stop-loss behaviour change across
capital/allocation regimes, while the decision framework remains frozen?

Regimes:
  A: Rs.5K  EqualWeight  (capital-constrained -- zero lots, no stops)
  B: Rs.1M  EqualWeight  (unconstrained baseline)
  C: Rs.1M  MaxPerLot    (allocation-constrained)

Key question: Is the reduction in Genuine Adverse stops (B: 20.7% -> C: 4.8%)
a real change in stop behaviour, or a selection effect from MaxPerLot
preventing certain decisions from becoming trades?
"""

import json
import os
import statistics
from collections import defaultdict

BASE = "historical_runs/portfolio_v04_1_capital_allocation_experiment"
CONFIGS = [
    ("A", "v04_1_A_50_5k_equal",   "Rs.5K  EqualWeight"),
    ("B", "v04_1_B_50_1m_equal",   "Rs.1M  EqualWeight"),
    ("C", "v04_1_C_50_1m_maxlot",  "Rs.1M  MaxPerLot"),
]

CATEGORIES = [
    "GapThrough",
    "PrematureStop",
    "TemporaryExcursion",
    "StopTooTight",
    "DirectionFailure",
    "GenuineAdverse",
]

def load(config_dir):
    path = os.path.join(BASE, config_dir, "stop_loss_analysis.json")
    with open(path) as f:
        return json.load(f)

def pct(n, d):
    return 0.0 if d == 0 else 100.0 * n / d

def mean_or_na(vals):
    clean = [v for v in vals if v is not None]
    return statistics.mean(clean) if clean else float("nan")

def median_or_na(vals):
    clean = [v for v in vals if v is not None]
    return statistics.median(clean) if clean else float("nan")

def fmt(v, decimals=2, suffix=""):
    if v != v:  # nan
        return "N/A"
    return f"{v:.{decimals}f}{suffix}"

def analyze_regime(label, config_dir, desc):
    data = load(config_dir)
    diags = data.get("diagnostics", [])
    n_stops = data["n_coralys_stops"]

    # --- 1. Stop incidence ---
    # Get total lots from continuous_ledger.json -> coralys_summary.n_lots_opened
    ledger_path = os.path.join(BASE, config_dir, "continuous_ledger.json")
    with open(ledger_path) as f:
        ledger = json.load(f)
    coralys_lots = ledger.get("coralys_summary", {}).get("n_lots_opened", 0)

    # --- 2. Category counts ---
    cat_counts = defaultdict(int)
    cat_pnl = defaultdict(list)
    cat_counterfactual = defaultdict(list)
    cat_opp_cost = defaultdict(list)
    cat_holding = defaultdict(list)
    cat_tightness = defaultdict(list)
    cat_post_mfe = defaultdict(list)
    cat_gap_mag = defaultdict(list)
    cat_alloc = defaultdict(list)

    def fv(d, key, scale=1.0):
        """Safely get a float field, returning None if null."""
        v = d.get(key)
        return v * scale if v is not None else None

    for d in diags:
        cat = d["category"]
        cat_counts[cat] += 1
        cat_pnl[cat].append(d["realized_pnl_inr"])
        cat_counterfactual[cat].append(d["counterfactual_pnl_inr"])
        cat_opp_cost[cat].append(d["opportunity_cost_inr"])
        cat_holding[cat].append(d["holding_sessions"])
        v = fv(d, "stop_tightness_pct", 100)
        if v is not None: cat_tightness[cat].append(v)
        v = fv(d, "post_stop_max_favorable_pct", 100)
        if v is not None: cat_post_mfe[cat].append(v)
        v = fv(d, "gap_magnitude_pct", 100)
        if v is not None: cat_gap_mag[cat].append(v)
        cat_alloc[cat].append(d["allocation_inr"])

    # --- 3. Aggregate financials ---
    all_pnl = [d["realized_pnl_inr"] for d in diags if d.get("realized_pnl_inr") is not None]
    all_cf = [d["counterfactual_pnl_inr"] for d in diags if d.get("counterfactual_pnl_inr") is not None]
    all_opp = [d["opportunity_cost_inr"] for d in diags if d.get("opportunity_cost_inr") is not None]
    all_hold = [d["holding_sessions"] for d in diags if d.get("holding_sessions") is not None]
    all_tight = [fv(d, "stop_tightness_pct", 100) for d in diags if fv(d, "stop_tightness_pct") is not None]
    all_mfe = [fv(d, "post_stop_max_favorable_pct", 100) for d in diags if fv(d, "post_stop_max_favorable_pct") is not None]
    all_alloc = [d["allocation_inr"] for d in diags if d.get("allocation_inr") is not None]

    # target_reached / recovered / continued_adverse
    n_target = sum(1 for d in diags if d["target_reached_after_stop"])
    n_recovered = sum(1 for d in diags if d["recovered_after_stop_within_5"])
    n_continued = sum(1 for d in diags if d["continued_adverse_5_sessions"])

    return {
        "label": label,
        "desc": desc,
        "config_dir": config_dir,
        "n_stops": n_stops,
        "coralys_lots": coralys_lots,
        "stop_rate_of_lots": pct(n_stops, coralys_lots) if coralys_lots else 0.0,
        "cat_counts": dict(cat_counts),
        "cat_pnl": dict(cat_pnl),
        "cat_counterfactual": dict(cat_counterfactual),
        "cat_opp_cost": dict(cat_opp_cost),
        "cat_holding": dict(cat_holding),
        "cat_tightness": dict(cat_tightness),
        "cat_post_mfe": dict(cat_post_mfe),
        "cat_gap_mag": dict(cat_gap_mag),
        "cat_alloc": dict(cat_alloc),
        "all_pnl": all_pnl,
        "all_cf": all_cf,
        "all_opp": all_opp,
        "all_hold": all_hold,
        "all_tight": all_tight,
        "all_mfe": all_mfe,
        "all_alloc": all_alloc,
        "n_target": n_target,
        "n_recovered": n_recovered,
        "n_continued": n_continued,
        "total_stop_pnl": data["total_stop_realized_pnl_inr"],
        "total_opp_cost": data["total_opportunity_cost_inr"],
        "net_stop_benefit": data["net_stop_benefit_inr"],
        "raw": data,
    }

def print_section(title):
    print()
    print("=" * 72)
    print(f"  {title}")
    print("=" * 72)

def print_subsection(title):
    print()
    print(f"--- {title} ---")

def main():
    regimes = []
    for label, config_dir, desc in CONFIGS:
        r = analyze_regime(label, config_dir, desc)
        regimes.append(r)

    A, B, C = regimes

    print_section("v0.4.1 REGIME-BY-REGIME STOP-LOSS ANALYSIS")
    print(f"Primary question: Is the Genuine Adverse drop (B: 20.7% -> C: 4.8%)")
    print(f"real stop behaviour change, or a selection effect from MaxPerLot?")

    # =========================================================================
    print_section("1. STOP INCIDENCE BY REGIME")
    # =========================================================================
    print(f"{'Regime':<6} {'Desc':<22} {'Lots':>6} {'Stops':>6} {'Stop%':>7} {'Realized PnL':>14} {'Net Benefit':>12}")
    print("-" * 75)
    for r in regimes:
        print(f"{r['label']:<6} {r['desc']:<22} {r['coralys_lots'] or 0:>6} "
              f"{r['n_stops']:>6} {r['stop_rate_of_lots']:>6.1f}% "
              f"{r['total_stop_pnl']:>13,.0f} {r['net_stop_benefit']:>11,.0f}")

    # =========================================================================
    print_section("2. STOP TAXONOMY — COUNTS AND % OF STOPS")
    # =========================================================================
    print(f"{'Category':<22} {'B count':>8} {'B %stop':>8} {'C count':>8} {'C %stop':>8} {'Delta':>8}")
    print("-" * 65)
    for cat in CATEGORIES:
        b_n = B["cat_counts"].get(cat, 0)
        c_n = C["cat_counts"].get(cat, 0)
        b_pct = pct(b_n, B["n_stops"])
        c_pct = pct(c_n, C["n_stops"])
        delta = c_pct - b_pct
        print(f"{cat:<22} {b_n:>8} {b_pct:>7.1f}% {c_n:>8} {c_pct:>7.1f}% {delta:>+7.1f}%")

    # =========================================================================
    print_section("3. STOP TAXONOMY — % OF ALL REALIZED LOTS")
    # =========================================================================
    print("(Normalizes by total lots, not just stops — removes stop-rate confound)")
    print(f"{'Category':<22} {'B %lots':>8} {'C %lots':>8} {'Delta':>8}")
    print("-" * 50)
    for cat in CATEGORIES:
        b_n = B["cat_counts"].get(cat, 0)
        c_n = C["cat_counts"].get(cat, 0)
        b_pct = pct(b_n, B["coralys_lots"] or 1)
        c_pct = pct(c_n, C["coralys_lots"] or 1)
        delta = c_pct - b_pct
        print(f"{cat:<22} {b_pct:>7.1f}% {c_pct:>7.1f}% {delta:>+7.1f}%")

    # =========================================================================
    print_section("4. STOP SEVERITY — FINANCIAL METRICS BY CATEGORY")
    # =========================================================================
    for regime in [B, C]:
        print_subsection(f"Regime {regime['label']}: {regime['desc']}")
        print(f"{'Category':<22} {'N':>4} {'MeanPnL':>10} {'MedPnL':>10} {'MeanCF':>10} {'MeanOpp':>10} {'MeanAlloc':>10}")
        print("-" * 80)
        for cat in CATEGORIES:
            pnls = regime["cat_pnl"].get(cat, [])
            cfs = regime["cat_counterfactual"].get(cat, [])
            opps = regime["cat_opp_cost"].get(cat, [])
            allocs = regime["cat_alloc"].get(cat, [])
            n = len(pnls)
            if n == 0:
                print(f"{cat:<22} {0:>4} {'N/A':>10} {'N/A':>10} {'N/A':>10} {'N/A':>10} {'N/A':>10}")
            else:
                print(f"{cat:<22} {n:>4} {mean_or_na(pnls):>10,.0f} {median_or_na(pnls):>10,.0f} "
                      f"{mean_or_na(cfs):>10,.0f} {mean_or_na(opps):>10,.0f} {mean_or_na(allocs):>10,.0f}")

    # =========================================================================
    print_section("5. STOP GEOMETRY — TIGHTNESS, MFE, HOLDING PERIOD")
    # =========================================================================
    for regime in [B, C]:
        print_subsection(f"Regime {regime['label']}: {regime['desc']}")
        print(f"{'Category':<22} {'N':>4} {'MeanTight%':>11} {'MedMFE%':>9} {'MeanHold':>9}")
        print("-" * 60)
        for cat in CATEGORIES:
            tights = regime["cat_tightness"].get(cat, [])
            mfes = regime["cat_post_mfe"].get(cat, [])
            holds = regime["cat_holding"].get(cat, [])
            n = len(tights)
            if n == 0:
                print(f"{cat:<22} {0:>4} {'N/A':>11} {'N/A':>9} {'N/A':>9}")
            else:
                print(f"{cat:<22} {n:>4} {mean_or_na(tights):>10.2f}% {median_or_na(mfes):>8.2f}% {mean_or_na(holds):>8.1f}")

    # =========================================================================
    print_section("6. POST-STOP BEHAVIOUR — TARGET/RECOVERY/CONTINUATION")
    # =========================================================================
    print(f"{'Metric':<35} {'B':>10} {'C':>10}")
    print("-" * 57)
    for r in [B, C]:
        pass
    rows = [
        ("Target reached after stop", B["n_target"], C["n_target"]),
        ("Recovered within 5 sessions", B["n_recovered"], C["n_recovered"]),
        ("Continued adverse 5 sessions", B["n_continued"], C["n_continued"]),
    ]
    for label, b_n, c_n in rows:
        b_pct = pct(b_n, B["n_stops"])
        c_pct = pct(c_n, C["n_stops"])
        print(f"{label:<35} {b_n:>4} ({b_pct:.1f}%) {c_n:>4} ({c_pct:.1f}%)")

    # =========================================================================
    print_section("7. THE SELECTION EFFECT QUESTION")
    # =========================================================================
    print("""
Key question: The 416 decisions NOT realized under MaxPerLot (C) — are they
the same decisions that would have produced GenuineAdverse stops under B?

Approach: Compare the GenuineAdverse stops in B vs C by instrument.
If the same instruments appear as GenuineAdverse in B but are absent from C
(because they were never realized), that is a selection effect.
If the same instruments appear in both B and C but with different outcomes,
that is a genuine stop behaviour change.
""")

    # Build instrument -> category map for B and C
    b_instrument_cat = {}
    for d in B["raw"]["diagnostics"]:
        inst = d["instrument"]
        cat = d["category"]
        seq = d["trade_id"].split("-seq")[-1]
        key = (inst, seq)
        b_instrument_cat[key] = (cat, d)

    c_instrument_cat = {}
    for d in C["raw"]["diagnostics"]:
        inst = d["instrument"]
        cat = d["category"]
        seq = d["trade_id"].split("-seq")[-1]
        key = (inst, seq)
        c_instrument_cat[key] = (cat, d)

    # GenuineAdverse in B
    b_genuine = {k: v for k, v in b_instrument_cat.items() if v[0] == "GenuineAdverse"}
    c_genuine = {k: v for k, v in c_instrument_cat.items() if v[0] == "GenuineAdverse"}

    b_genuine_keys = set(b_genuine.keys())
    c_genuine_keys = set(c_genuine.keys())
    c_all_keys = set(c_instrument_cat.keys())

    # B GenuineAdverse trades that are also in C (same instrument+seq)
    in_both = b_genuine_keys & c_all_keys
    # B GenuineAdverse trades that are NOT in C at all (selection effect)
    not_in_c = b_genuine_keys - c_all_keys
    # B GenuineAdverse trades that ARE in C but with different category
    in_c_different_cat = {k for k in in_both if c_instrument_cat[k][0] != "GenuineAdverse"}
    # B GenuineAdverse trades that ARE in C and still GenuineAdverse
    in_c_still_genuine = b_genuine_keys & c_genuine_keys

    print(f"GenuineAdverse stops in B: {len(b_genuine_keys)}")
    print(f"GenuineAdverse stops in C: {len(c_genuine_keys)}")
    print()
    print(f"Of B's {len(b_genuine_keys)} GenuineAdverse stops:")
    print(f"  {len(not_in_c):>4} ({pct(len(not_in_c), len(b_genuine_keys)):.1f}%) NOT realized in C at all  --> SELECTION EFFECT")
    print(f"  {len(in_both):>4} ({pct(len(in_both), len(b_genuine_keys)):.1f}%) also realized in C")
    print(f"    of which {len(in_c_still_genuine)} ({pct(len(in_c_still_genuine), len(in_both)):.1f}%) still GenuineAdverse in C")
    print(f"    of which {len(in_c_different_cat)} ({pct(len(in_c_different_cat), len(in_both)):.1f}%) changed category in C")

    if in_c_different_cat:
        print()
        print("  Category changes for trades realized in both B and C:")
        print(f"  {'Instrument+Seq':<30} {'B category':<22} {'C category':<22}")
        print("  " + "-" * 74)
        for k in sorted(in_c_different_cat):
            b_cat = b_instrument_cat[k][0]
            c_cat = c_instrument_cat[k][0]
            print(f"  {str(k):<30} {b_cat:<22} {c_cat:<22}")

    if in_c_still_genuine:
        print()
        print("  Trades GenuineAdverse in BOTH B and C (genuine stop behaviour):")
        print(f"  {'Instrument+Seq':<30} {'B PnL':>10} {'C PnL':>10} {'B alloc':>10} {'C alloc':>10}")
        print("  " + "-" * 74)
        for k in sorted(in_c_still_genuine):
            b_d = b_instrument_cat[k][1]
            c_d = c_instrument_cat[k][1]
            print(f"  {str(k):<30} {b_d['realized_pnl_inr']:>10,.0f} {c_d['realized_pnl_inr']:>10,.0f} "
                  f"{b_d['allocation_inr']:>10,.0f} {c_d['allocation_inr']:>10,.0f}")

    # =========================================================================
    print_section("8. COUNTERFACTUAL ANALYSIS — STOP BENEFIT/COST")
    # =========================================================================
    print("opportunity_cost = counterfactual_pnl - realized_pnl")
    print("Positive opp_cost = stop was costly (price recovered after stop)")
    print("Negative opp_cost = stop was beneficial (price continued adverse)")
    print()
    for regime in [B, C]:
        total_opp = sum(regime["all_opp"])
        n_beneficial = sum(1 for x in regime["all_opp"] if x < 0)
        n_costly = sum(1 for x in regime["all_opp"] if x > 0)
        pnl_beneficial = sum(x for x in regime["all_opp"] if x < 0)
        pnl_costly = sum(x for x in regime["all_opp"] if x > 0)
        print(f"Regime {regime['label']}: {regime['desc']}")
        print(f"  Total opportunity cost: Rs.{total_opp:,.0f}")
        print(f"  Stops that were BENEFICIAL (price continued adverse): {n_beneficial} ({pct(n_beneficial, regime['n_stops']):.1f}%)")
        print(f"    Total benefit: Rs.{pnl_beneficial:,.0f}")
        print(f"  Stops that were COSTLY (price recovered after stop):  {n_costly} ({pct(n_costly, regime['n_stops']):.1f}%)")
        print(f"    Total cost:    Rs.{pnl_costly:,.0f}")
        print()

    # =========================================================================
    print_section("9. ALLOCATION SIZE EFFECT ON STOP OUTCOMES")
    # =========================================================================
    print("MaxPerLot uses smaller per-lot allocations. Does smaller allocation")
    print("change which stops are triggered or their severity?")
    print()
    print(f"{'Regime':<6} {'Mean alloc/lot':>15} {'Mean stop PnL':>14} {'Mean stop PnL %':>16}")
    print("-" * 55)
    for regime in [B, C]:
        if regime["all_alloc"]:
            mean_alloc = mean_or_na(regime["all_alloc"])
            mean_pnl = mean_or_na(regime["all_pnl"])
            mean_pnl_pct = mean_or_na([p / a * 100 for p, a in zip(regime["all_pnl"], regime["all_alloc"])])
            print(f"{regime['label']:<6} {mean_alloc:>14,.0f} {mean_pnl:>13,.0f} {mean_pnl_pct:>15.2f}%")
        else:
            print(f"{regime['label']:<6} {'N/A':>15} {'N/A':>14} {'N/A':>16}")

    # =========================================================================
    print_section("10. SUMMARY VERDICT")
    # =========================================================================
    b_genuine_n = len(b_genuine_keys)
    selection_n = len(not_in_c)
    behaviour_n = len(in_c_still_genuine)
    changed_n = len(in_c_different_cat)

    selection_pct = pct(selection_n, b_genuine_n)
    behaviour_pct = pct(behaviour_n, b_genuine_n)
    changed_pct = pct(changed_n, b_genuine_n)

    print(f"""
B has {b_genuine_n} GenuineAdverse stops. C has {len(c_genuine_keys)}.
The apparent reduction of {b_genuine_n - len(c_genuine_keys)} GenuineAdverse stops decomposes as:

  Selection effect (not realized in C):  {selection_n:>3} ({selection_pct:.1f}% of B's genuine adverse)
  Behaviour change (still genuine in C): {behaviour_n:>3} ({behaviour_pct:.1f}% of B's genuine adverse)
  Category change (realized, different): {changed_n:>3} ({changed_pct:.1f}% of B's genuine adverse)

VERDICT:
""")
    if selection_pct > 70:
        print("  The reduction in GenuineAdverse stops is PRIMARILY a SELECTION EFFECT.")
        print("  MaxPerLot prevents certain decisions from becoming trades, and those")
        print("  decisions happen to be the ones that would have produced genuine adverse")
        print("  stops. This is NOT evidence that MaxPerLot improves stop behaviour.")
        print("  It is evidence that MaxPerLot selects a different (better) population")
        print("  of trades from the same Coralys decision set.")
    elif selection_pct > 40:
        print("  The reduction is a MIXED EFFECT: partly selection, partly behaviour.")
        print(f"  {selection_pct:.0f}% is selection (trades not realized in C).")
        print(f"  {behaviour_pct:.0f}% reflects genuine stop behaviour differences.")
    else:
        print("  The reduction is primarily a GENUINE STOP BEHAVIOUR CHANGE.")
        print("  MaxPerLot's smaller per-lot sizing changes how stops are triggered.")

    print(f"""
DEFENSIBLE CONCLUSION:
  MaxPerLot changes the POPULATION of realized Coralys trades.
  The realized population under MaxPerLot exhibits substantially different
  stop-loss outcomes. Whether this represents:
    A. Better trade selection (selection effect)
    B. Better stop behaviour (mechanism effect)
    C. A combination
  ...is answered above by the decomposition of B's GenuineAdverse stops.
""")

if __name__ == "__main__":
    main()