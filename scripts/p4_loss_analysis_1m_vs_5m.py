#!/usr/bin/env python3
"""
P4 Loss Analysis — 1m vs 5m Resolution Comparison
====================================================
Runs the same frozen Phase 4 state machine on both 1m and 5m intraday paths
for the same decision cohorts (Sep 3–11 LIVE-003 Watch decisions).

NOTE on cohort scope:
  1m cache covers Sep 3–11 only.
  5m cache covers Aug 24–Sep 11.
  For apples-to-apples comparison we restrict BOTH resolutions to Sep 3–11
  (the dates where 1m data exists), so the same decisions feed both analyses.

Produces:
  1. ENTER failure mode breakdown (SHORT and LONG) at both resolutions
  2. AVOID false-negative analysis at both resolutions
  3. Side-by-side 1m vs 5m failure matrix

Frozen Phase 4 rules (immutable):
  - threshold ±0.002 (0.2%)
  - MFE_H60 < 0.1% override → AVOID
  - H15/H60 → ENTER/AVOID/WAIT
  - H120 sub-classification → ENTER-LATE/AVOID-LATE/WAIT-LATE
  - H180 confirmation layer

Resolution mapping:
  1m: H15=15 bars, H60=60 bars, H120=120 bars, H180=180 bars, H300=300 bars
  5m: H15=3 bars,  H60=12 bars, H120=24 bars,  H180=36 bars,  H300=60 bars

Methodology:
  Same decisions → same entry → same H15/H60/H120/H180/H300 windows
  → 1m path vs 5m path (apples-to-apples comparison)
"""

import json
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone

REPO_ROOT = Path(__file__).resolve().parent.parent
REC_DIR   = REPO_ROOT / "live_capture" / "recommendations"
CACHE_1M  = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
CACHE_5M  = REPO_ROOT / "intraday_capture" / "yahoo_cache_5m"

# Horizons in bars for each resolution
HORIZONS_1M = {"H15": 15, "H60": 60, "H120": 120, "H180": 180, "H300": 300}
HORIZONS_5M = {"H15":  3, "H60": 12, "H120":  24, "H180":  36, "H300":  60}

THRESHOLD = 0.002   # ±0.2%
MFE_OVERRIDE = 0.001  # MFE < 0.1% → AVOID

# ── Frozen Phase 4 classification functions ────────────────────────────────────

def bucket(ret, t=THRESHOLD):
    if ret is None:
        return "FLAT"
    return "FAV" if ret > t else ("ADV" if ret < -t else "FLAT")

def classify_h60(h15_ret, h60_ret, mfe_h60):
    """H60 classification → ENTER / AVOID / WAIT"""
    if h15_ret is None or h60_ret is None:
        return "UNKNOWN"
    if mfe_h60 is not None and mfe_h60 < MFE_OVERRIDE:
        return "AVOID"
    b15 = bucket(h15_ret)
    b60 = bucket(h60_ret)
    if b15 == "FAV" and b60 == "FAV":
        return "ENTER"
    if b15 == "ADV" and b60 == "ADV":
        return "AVOID"
    return "WAIT"

def classify_h120(h120_ret, mfe_h120):
    """H120 sub-classification → ENTER-LATE / AVOID-LATE / WAIT-LATE"""
    if h120_ret is None:
        return "UNKNOWN"
    if mfe_h120 is not None and mfe_h120 < MFE_OVERRIDE:
        return "AVOID-LATE"
    b120 = bucket(h120_ret)
    if b120 == "FAV":
        return "ENTER-LATE"
    if b120 == "ADV":
        return "AVOID-LATE"
    return "WAIT-LATE"

# ── Data loading ───────────────────────────────────────────────────────────────

def load_json(path):
    with open(path) as f:
        return json.load(f)

def load_intraday_cache(cache_dir):
    """Load all ticker bar data from a cache directory."""
    idx = {}
    for p in cache_dir.glob("*.json"):
        ticker_ns = p.stem.replace(".", "_")
        try:
            bars = load_json(p)
            bars.sort(key=lambda b: b["timestamp"])
            idx[ticker_ns] = bars
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)
    return idx

def parse_iso_to_unix(ts_str):
    """Parse ISO 8601 string (with or without Z suffix) to unix timestamp."""
    if not ts_str:
        return None
    try:
        # Replace Z with +00:00 for fromisoformat compatibility
        ts_str = ts_str.replace("Z", "+00:00")
        dt = datetime.fromisoformat(ts_str)
        return dt.timestamp()
    except Exception:
        return None

def load_live003_decisions(min_cohort_date="2026-09-03"):
    """
    Load LIVE-003 Watch decisions from recommendation artifacts.
    Filters to cohorts >= min_cohort_date (default Sep 3, first date with 1m cache).

    Returns list of dicts with: ticker, ticker_ns, direction, action,
    reference_price, rank_score, cohort_date, snap_unix, source_file
    """
    decisions = []
    for p in sorted(REC_DIR.glob("LIVE-003-*.json")):
        try:
            data = load_json(p)

            # Parse snapshot timestamp → unix
            snap_ts = data.get("source_snapshot_timestamp")
            snap_unix = parse_iso_to_unix(snap_ts)

            # Derive cohort_date from filename e.g. LIVE-003-20260903-1000.json
            fname = p.stem
            parts = fname.split("-")
            date_str = parts[2] if len(parts) > 2 else ""
            if len(date_str) == 8:
                cohort_date = f"{date_str[:4]}-{date_str[4:6]}-{date_str[6:8]}"
            else:
                cohort_date = date_str

            # Filter to dates where 1m cache exists
            if cohort_date < min_cohort_date:
                continue

            if snap_unix is None:
                print(f"  [warn] {p.name}: could not parse snap timestamp '{snap_ts}'",
                      file=sys.stderr)
                continue

            recs = data.get("recommendations", [])
            for r in recs:
                if r.get("action") != "Watch":
                    continue
                ticker = r.get("instrument") or r.get("ticker")
                if not ticker:
                    continue
                # Normalise: HCLTECH_NS → HCLTECH_NS (already ok)
                #            HCLTECH.NS → HCLTECH_NS
                ticker_ns = ticker.replace(".", "_")
                decisions.append({
                    "ticker": ticker,
                    "ticker_ns": ticker_ns,
                    "direction": r.get("direction"),
                    "action": r.get("action"),
                    "reference_price": r.get("reference_price"),
                    "rank_score": r.get("rank_score"),
                    "cohort_date": cohort_date,
                    "snap_unix": snap_unix,
                    "source_file": p.name,
                })
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)
    return decisions

# ── Intraday path measurement ──────────────────────────────────────────────────

def measure_path(decision, cache_idx, horizons):
    """
    Measure intraday path for a decision at given horizons.
    Returns dict: horizon_label → {ret, mfe, mae, time_to_mfe, time_to_mae}
    or empty dict if no data.
    """
    ticker_ns = decision["ticker_ns"]
    snap_unix = decision["snap_unix"]
    ref_price = decision["reference_price"]
    direction = decision["direction"]

    if not snap_unix or not ref_price or ticker_ns not in cache_idx:
        return {}

    bars = cache_idx[ticker_ns]

    # First bar strictly after snapshot
    start_idx = None
    for i, b in enumerate(bars):
        if b["timestamp"] > snap_unix:
            start_idx = i
            break

    if start_idx is None:
        return {}

    entry_price = bars[start_idx].get("close")
    if not entry_price or entry_price == 0:
        return {}

    is_short = (direction == "SHORT")
    results = {}

    for label, n_bars in horizons.items():
        end_idx = start_idx + n_bars - 1
        if end_idx >= len(bars):
            continue

        exit_price = bars[end_idx].get("close")
        if not exit_price:
            continue

        # Direction-adjusted terminal return
        if is_short:
            ret = (entry_price - exit_price) / entry_price
        else:
            ret = (exit_price - entry_price) / entry_price

        # Path prices
        path_prices = []
        for i in range(n_bars):
            idx = start_idx + i
            if idx < len(bars):
                p = bars[idx].get("close")
                if p:
                    path_prices.append(p)

        mfe = mae = time_to_mfe = time_to_mae = None
        if path_prices:
            if is_short:
                excursions = [(entry_price - p) / entry_price for p in path_prices]
            else:
                excursions = [(p - entry_price) / entry_price for p in path_prices]
            mfe = max(excursions)
            mae = min(excursions)
            time_to_mfe = excursions.index(mfe)
            time_to_mae = excursions.index(mae)

        results[label] = {
            "ret": ret,
            "mfe": mfe,
            "mae": mae,
            "time_to_mfe": time_to_mfe,
            "time_to_mae": time_to_mae,
            "entry_price": entry_price,
            "exit_price": exit_price,
            "n_bars": n_bars,
        }

    return results

# ── Failure mode taxonomy ──────────────────────────────────────────────────────

def failure_mode(h15_ret, h60_ret, h120_ret, h180_ret, h300_ret):
    """
    Classify the timing of failure for a losing ENTER trade.
    Returns one of: IMMEDIATE, EARLY, MID-REVERSAL, LATE-REVERSAL,
                    VERY-LATE-REVERSAL, GRADUAL/OTHER
    """
    def fav(r): return r is not None and r > THRESHOLD
    def adv(r): return r is not None and r < -THRESHOLD

    if adv(h15_ret):
        return "IMMEDIATE"
    if fav(h15_ret) and adv(h60_ret):
        return "EARLY"
    if fav(h60_ret) and adv(h120_ret):
        return "MID-REVERSAL"
    if fav(h120_ret) and adv(h180_ret):
        return "LATE-REVERSAL"
    if fav(h180_ret) and adv(h300_ret):
        return "VERY-LATE-REVERSAL"
    return "GRADUAL/OTHER"

# ── Analysis functions ─────────────────────────────────────────────────────────

def analyse_resolution(decisions, cache_idx, horizons, res_label):
    """
    Run full loss analysis for one resolution.
    Returns dict with all computed metrics.
    """
    print(f"\n  [{res_label}] Measuring paths for {len(decisions)} decisions...")

    # Measure all paths
    measured = []
    for dec in decisions:
        path = measure_path(dec, cache_idx, horizons)
        if not path:
            continue
        # Need at least H15, H60, H300
        if "H15" not in path or "H60" not in path or "H300" not in path:
            continue

        h15  = path["H15"]["ret"]
        h60  = path["H60"]["ret"]
        h120 = path.get("H120", {}).get("ret")
        h180 = path.get("H180", {}).get("ret")
        h300 = path["H300"]["ret"]
        mfe_h60  = path["H60"]["mfe"]
        mfe_h120 = path.get("H120", {}).get("mfe")

        classification = classify_h60(h15, h60, mfe_h60)
        late_class     = classify_h120(h120, mfe_h120) if h120 is not None else "UNKNOWN"

        # First adverse bar (bars into trade when return first goes negative)
        # Use H300 path bars for timing
        h300_data = path["H300"]
        first_adv_bar = None
        # We don't have bar-by-bar data here; approximate from time_to_mae
        # time_to_mae is the bar index within H300 window where MAE occurs
        # For losers (H300 ret < 0), time_to_mae approximates first adverse excursion
        # Convert bar index to minutes
        bars_per_min = 1 if res_label == "1m" else 5
        if h300_data.get("time_to_mae") is not None:
            first_adv_bar = h300_data["time_to_mae"] * bars_per_min

        measured.append({
            **dec,
            "H15": h15, "H60": h60, "H120": h120, "H180": h180, "H300": h300,
            "mfe_H60": mfe_h60, "mfe_H120": mfe_h120,
            "mfe_H300": h300_data.get("mfe"),
            "mae_H300": h300_data.get("mae"),
            "classification": classification,
            "late_classification": late_class,
            "first_adv_min": first_adv_bar,
        })

    print(f"  [{res_label}] Measured: {len(measured)} decisions")

    results = {}
    for direction in ["SHORT", "LONG"]:
        dir_data = [m for m in measured if m["direction"] == direction]

        # ── ENTER analysis ──────────────────────────────────────────────────
        enters = [m for m in dir_data if m["classification"] == "ENTER"]
        winners = [m for m in enters if m["H300"] > THRESHOLD]
        losers  = [m for m in enters if m["H300"] <= THRESHOLD]

        # Failure modes for losers
        failure_counts = {}
        for m in losers:
            fm = failure_mode(m["H15"], m["H60"], m["H120"], m["H180"], m["H300"])
            failure_counts[fm] = failure_counts.get(fm, 0) + 1

        # H60/H120 FAV rates
        def h60_fav_rate(group):
            if not group: return None
            return sum(1 for m in group if bucket(m["H60"]) == "FAV") / len(group)

        def h120_fav_rate(group):
            if not group: return None
            eligible = [m for m in group if m["H120"] is not None]
            if not eligible: return None
            return sum(1 for m in eligible if bucket(m["H120"]) == "FAV") / len(eligible)

        # MFE@H60 median
        def mfe_h60_median(group):
            vals = [m["mfe_H60"] for m in group if m["mfe_H60"] is not None]
            return statistics.median(vals) if vals else None

        # First adverse bar median (for losers vs winners)
        def first_adv_median(group):
            vals = [m["first_adv_min"] for m in group if m["first_adv_min"] is not None]
            return statistics.median(vals) if vals else None

        # ── AVOID analysis ──────────────────────────────────────────────────
        avoids = [m for m in dir_data if m["classification"] == "AVOID"]
        avoid_winners = [m for m in avoids if m["H300"] > THRESHOLD]  # false negatives
        avoid_losers  = [m for m in avoids if m["H300"] <= THRESHOLD]

        # H60/H120 state for false negatives
        def h60_adv_rate(group):
            if not group: return None
            return sum(1 for m in group if bucket(m["H60"]) == "ADV") / len(group)

        def h120_fav_rate_avoid(group):
            if not group: return None
            eligible = [m for m in group if m["H120"] is not None]
            if not eligible: return None
            return sum(1 for m in eligible if bucket(m["H120"]) == "FAV") / len(eligible)

        # H300 median return for false negatives
        def h300_median(group):
            vals = [m["H300"] for m in group if m["H300"] is not None]
            return statistics.median(vals) if vals else None

        results[direction] = {
            # ENTER
            "enter_n": len(enters),
            "enter_winners": len(winners),
            "enter_losers": len(losers),
            "enter_win_rate": len(winners) / len(enters) if enters else None,
            "failure_modes": failure_counts,
            # Loser diagnostics
            "losers_h60_fav": h60_fav_rate(losers),
            "losers_h120_fav": h120_fav_rate(losers),
            "losers_mfe_h60_median": mfe_h60_median(losers),
            "losers_first_adv_median": first_adv_median(losers),
            # Winner diagnostics
            "winners_h60_fav": h60_fav_rate(winners),
            "winners_h120_fav": h120_fav_rate(winners),
            "winners_mfe_h60_median": mfe_h60_median(winners),
            "winners_first_adv_median": first_adv_median(winners),
            # AVOID
            "avoid_n": len(avoids),
            "avoid_false_neg": len(avoid_winners),
            "avoid_true_neg": len(avoid_losers),
            "avoid_fn_rate": len(avoid_winners) / len(avoids) if avoids else None,
            # False negative diagnostics
            "fn_h60_adv": h60_adv_rate(avoid_winners),
            "fn_h120_fav": h120_fav_rate_avoid(avoid_winners),
            "fn_h300_median": h300_median(avoid_winners),
            # WAIT
            "wait_n": len([m for m in dir_data if m["classification"] == "WAIT"]),
            # Total
            "total_n": len(dir_data),
        }

    return results

# ── Printing ───────────────────────────────────────────────────────────────────

def pct(v, decimals=1):
    if v is None: return "—"
    return f"{v*100:+.{decimals}f}%"

def rate(v, decimals=1):
    if v is None: return "—"
    return f"{v*100:.{decimals}f}%"

def num(v):
    if v is None: return "—"
    return f"{v:.0f}"

def print_resolution_detail(res_label, results):
    print(f"\n{'='*60}")
    print(f"  {res_label} RESOLUTION — DETAILED RESULTS")
    print(f"{'='*60}")

    for direction in ["SHORT", "LONG"]:
        r = results[direction]
        print(f"\n  ── {direction} ──────────────────────────────────────────")
        print(f"  Total Watch decisions measured: {r['total_n']}")
        print(f"  ENTER: {r['enter_n']}  |  AVOID: {r['avoid_n']}  |  WAIT: {r['wait_n']}")

        print(f"\n  ENTER ANALYSIS (N={r['enter_n']})")
        print(f"    Winners: {r['enter_winners']}  Losers: {r['enter_losers']}  "
              f"Win rate: {rate(r['enter_win_rate'])}")

        if r['enter_losers'] > 0:
            print(f"\n    Failure modes (losers N={r['enter_losers']}):")
            fm = r['failure_modes']
            for mode in ["IMMEDIATE", "EARLY", "MID-REVERSAL", "LATE-REVERSAL",
                         "VERY-LATE-REVERSAL", "GRADUAL/OTHER"]:
                n = fm.get(mode, 0)
                if n > 0:
                    print(f"      {mode:<22}: {n}")

            print(f"\n    Loser diagnostics:")
            print(f"      H60 FAV rate:          {rate(r['losers_h60_fav'])}")
            print(f"      H120 FAV rate:         {rate(r['losers_h120_fav'])}")
            print(f"      MFE@H60 median:        {pct(r['losers_mfe_h60_median'])}")
            print(f"      First adverse bar:     {num(r['losers_first_adv_median'])} min")

            print(f"\n    Winner diagnostics (N={r['enter_winners']}):")
            print(f"      H60 FAV rate:          {rate(r['winners_h60_fav'])}")
            print(f"      H120 FAV rate:         {rate(r['winners_h120_fav'])}")
            print(f"      MFE@H60 median:        {pct(r['winners_mfe_h60_median'])}")
            print(f"      First adverse bar:     {num(r['winners_first_adv_median'])} min")

        print(f"\n  AVOID ANALYSIS (N={r['avoid_n']})")
        print(f"    True negatives: {r['avoid_true_neg']}  "
              f"False negatives: {r['avoid_false_neg']}  "
              f"FN rate: {rate(r['avoid_fn_rate'])}")

        if r['avoid_false_neg'] > 0:
            print(f"\n    False negative diagnostics (N={r['avoid_false_neg']}):")
            print(f"      H60 ADV rate:          {rate(r['fn_h60_adv'])}")
            print(f"      H120 FAV rate:         {rate(r['fn_h120_fav'])}")
            print(f"      H300 median return:    {pct(r['fn_h300_median'])}")

def print_comparison_matrix(res_1m, res_5m):
    print(f"\n{'='*72}")
    print(f"  1m vs 5m FAILURE MATRIX")
    print(f"{'='*72}")

    def row(label, fn_1m_short, fn_5m_short, fn_1m_long, fn_5m_long):
        print(f"  {label:<30} {fn_1m_short:>10} {fn_5m_short:>10} {fn_1m_long:>10} {fn_5m_long:>10}")

    print(f"  {'Failure characteristic':<30} {'SHORT 1m':>10} {'SHORT 5m':>10} {'LONG 1m':>10} {'LONG 5m':>10}")
    print(f"  {'-'*70}")

    def frac(r, key_n, key_d):
        n = r.get(key_n, 0)
        d = r.get(key_d, 0)
        return f"{n}/{d}" if d else "—"

    def fm_n(r, mode):
        return str(r['failure_modes'].get(mode, 0)) if r['failure_modes'] else "—"

    s1 = res_1m["SHORT"]; s5 = res_5m["SHORT"]
    l1 = res_1m["LONG"];  l5 = res_5m["LONG"]

    row("ENTER losses",
        frac(s1, "enter_losers", "enter_n"),
        frac(s5, "enter_losers", "enter_n"),
        frac(l1, "enter_losers", "enter_n"),
        frac(l5, "enter_losers", "enter_n"))

    row("Win rate",
        rate(s1["enter_win_rate"]),
        rate(s5["enter_win_rate"]),
        rate(l1["enter_win_rate"]),
        rate(l5["enter_win_rate"]))

    print(f"  {'-'*70}")
    print(f"  {'  Failure modes:':<30}")

    for mode in ["IMMEDIATE", "EARLY", "MID-REVERSAL", "LATE-REVERSAL",
                 "VERY-LATE-REVERSAL", "GRADUAL/OTHER"]:
        row(f"    {mode}",
            fm_n(s1, mode), fm_n(s5, mode),
            fm_n(l1, mode), fm_n(l5, mode))

    print(f"  {'-'*70}")
    print(f"  {'  Loser diagnostics:':<30}")

    row("H60 FAV (losers)",
        rate(s1["losers_h60_fav"]),
        rate(s5["losers_h60_fav"]),
        rate(l1["losers_h60_fav"]),
        rate(l5["losers_h60_fav"]))

    row("H120 FAV (losers)",
        rate(s1["losers_h120_fav"]),
        rate(s5["losers_h120_fav"]),
        rate(l1["losers_h120_fav"]),
        rate(l5["losers_h120_fav"]))

    row("MFE@H60 median (losers)",
        pct(s1["losers_mfe_h60_median"]),
        pct(s5["losers_mfe_h60_median"]),
        pct(l1["losers_mfe_h60_median"]),
        pct(l5["losers_mfe_h60_median"]))

    row("First adverse bar (losers)",
        f"{num(s1['losers_first_adv_median'])}m",
        f"{num(s5['losers_first_adv_median'])}m",
        f"{num(l1['losers_first_adv_median'])}m",
        f"{num(l5['losers_first_adv_median'])}m")

    print(f"  {'-'*70}")
    print(f"  {'  Winner diagnostics:':<30}")

    row("H60 FAV (winners)",
        rate(s1["winners_h60_fav"]),
        rate(s5["winners_h60_fav"]),
        rate(l1["winners_h60_fav"]),
        rate(l5["winners_h60_fav"]))

    row("H120 FAV (winners)",
        rate(s1["winners_h120_fav"]),
        rate(s5["winners_h120_fav"]),
        rate(l1["winners_h120_fav"]),
        rate(l5["winners_h120_fav"]))

    row("MFE@H60 median (winners)",
        pct(s1["winners_mfe_h60_median"]),
        pct(s5["winners_mfe_h60_median"]),
        pct(l1["winners_mfe_h60_median"]),
        pct(l5["winners_mfe_h60_median"]))

    row("First adverse bar (winners)",
        f"{num(s1['winners_first_adv_median'])}m",
        f"{num(s5['winners_first_adv_median'])}m",
        f"{num(l1['winners_first_adv_median'])}m",
        f"{num(l5['winners_first_adv_median'])}m")

    print(f"  {'-'*70}")
    print(f"  {'AVOID false negatives':<30}")

    row("False negatives",
        frac(s1, "avoid_false_neg", "avoid_n"),
        frac(s5, "avoid_false_neg", "avoid_n"),
        frac(l1, "avoid_false_neg", "avoid_n"),
        frac(l5, "avoid_false_neg", "avoid_n"))

    row("FN rate",
        rate(s1["avoid_fn_rate"]),
        rate(s5["avoid_fn_rate"]),
        rate(l1["avoid_fn_rate"]),
        rate(l5["avoid_fn_rate"]))

    row("FN H60 ADV rate",
        rate(s1["fn_h60_adv"]),
        rate(s5["fn_h60_adv"]),
        rate(l1["fn_h60_adv"]),
        rate(l5["fn_h60_adv"]))

    row("FN H120 FAV rate",
        rate(s1["fn_h120_fav"]),
        rate(s5["fn_h120_fav"]),
        rate(l1["fn_h120_fav"]),
        rate(l5["fn_h120_fav"]))

    row("FN H300 median return",
        pct(s1["fn_h300_median"]),
        pct(s5["fn_h300_median"]),
        pct(l1["fn_h300_median"]),
        pct(l5["fn_h300_median"]))

def print_product_conclusions(res_1m, res_5m):
    print(f"\n{'='*60}")
    print(f"  PRODUCT CONCLUSIONS")
    print(f"{'='*60}")

    print("""
  Four conditions for a pattern to become a product candidate:
    1. Present at BOTH resolutions
    2. Visible BEFORE the eventual outcome
    3. Materially DIFFERENT between winners and losers
    4. ACTIONABLE at the time

  ── SHORT ENTER ──────────────────────────────────────────────""")

    s1 = res_1m["SHORT"]; s5 = res_5m["SHORT"]
    h120_diff_1m = None
    h120_diff_5m = None
    if s1["losers_h120_fav"] is not None and s1["winners_h120_fav"] is not None:
        h120_diff_1m = s1["winners_h120_fav"] - s1["losers_h120_fav"]
    if s5["losers_h120_fav"] is not None and s5["winners_h120_fav"] is not None:
        h120_diff_5m = s5["winners_h120_fav"] - s5["losers_h120_fav"]

    print(f"  H120 FAV gap (winners - losers): 1m={pct(h120_diff_1m)}  5m={pct(h120_diff_5m)}")
    print(f"  → If gap is small at both resolutions: H120 does NOT protect SHORT")

    print(f"\n  ── LONG ENTER ───────────────────────────────────────────────")
    l1 = res_1m["LONG"]; l5 = res_5m["LONG"]
    h120_diff_1m_l = None
    h120_diff_5m_l = None
    if l1["losers_h120_fav"] is not None and l1["winners_h120_fav"] is not None:
        h120_diff_1m_l = l1["winners_h120_fav"] - l1["losers_h120_fav"]
    if l5["losers_h120_fav"] is not None and l5["winners_h120_fav"] is not None:
        h120_diff_5m_l = l5["winners_h120_fav"] - l5["losers_h120_fav"]

    print(f"  H120 FAV gap (winners - losers): 1m={pct(h120_diff_1m_l)}  5m={pct(h120_diff_5m_l)}")
    print(f"  → If gap is large at both resolutions: H120 IS a protective signal for LONG")

    print(f"\n  ── AVOID FALSE NEGATIVES ────────────────────────────────────")
    print(f"  SHORT FN H300 median: 1m={pct(s1['fn_h300_median'])}  5m={pct(s5['fn_h300_median'])}")
    print(f"  LONG  FN H300 median: 1m={pct(l1['fn_h300_median'])}  5m={pct(l5['fn_h300_median'])}")
    print(f"  → If FN H300 median is small (<+0.5%) at both resolutions:")
    print(f"    AVOID false negatives are economically immaterial — no override justified")

    print(f"""
  ── SUMMARY ──────────────────────────────────────────────────
  Patterns satisfying all four conditions become product candidates.
  Patterns failing any condition are REJECTED.

  Candidate evaluation:
    SHORT H120 confirmation: check gap above — if <10pp, REJECT
    LONG  H120 confirmation: check gap above — if >10pp, CANDIDATE
    AVOID override:          check FN H300 median — if <+0.5%, REJECT

  These conclusions feed directly into Decision Cockpit v0.4 design.
  No rule changes are made here — this is observation only.
""")

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("=" * 60)
    print("P4 Loss Analysis — 1m vs 5m Resolution Comparison")
    print("=" * 60)

    print("\nLoading LIVE-003 Watch decisions...")
    decisions = load_live003_decisions()
    print(f"  Total Watch decisions loaded: {len(decisions)}")

    cohorts = sorted(set(d["cohort_date"] for d in decisions))
    print(f"  Cohorts: {cohorts}")

    for direction in ["SHORT", "LONG"]:
        n = sum(1 for d in decisions if d["direction"] == direction)
        print(f"  {direction}: {n}")

    print("\nLoading 1m cache...")
    cache_1m = load_intraday_cache(CACHE_1M)
    print(f"  Tickers loaded: {len(cache_1m)}")

    print("\nLoading 5m cache...")
    cache_5m = load_intraday_cache(CACHE_5M)
    print(f"  Tickers loaded: {len(cache_5m)}")

    # Run analysis at both resolutions
    res_1m = analyse_resolution(decisions, cache_1m, HORIZONS_1M, "1m")
    res_5m = analyse_resolution(decisions, cache_5m, HORIZONS_5M, "5m")

    # Print detailed results for each resolution
    print_resolution_detail("1m", res_1m)
    print_resolution_detail("5m", res_5m)

    # Print side-by-side comparison matrix
    print_comparison_matrix(res_1m, res_5m)

    # Print product conclusions
    print_product_conclusions(res_1m, res_5m)

if __name__ == "__main__":
    run()