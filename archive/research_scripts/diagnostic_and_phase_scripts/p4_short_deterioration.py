#!/usr/bin/env python3
"""
P4 — SHORT ENTER Continuous Deterioration Analysis
====================================================
Investigates whether rolling 1m bar monitoring can detect the ~240m failure
pattern for SHORT ENTER losers before the H300 outcome is known.

Key finding from loss analysis:
  SHORT ENTER losers:  first adverse bar median ~240m (4 hours into trade)
  SHORT ENTER winners: first adverse bar median ~0-2m (fail immediately or not at all)

Question: Is there a detectable deterioration signal in the 1m path between
entry and the ~240m failure point that distinguishes losers from winners?

Approach:
  1. For each SHORT ENTER decision, extract the full 1m path (300 bars = 5 hours)
  2. Compute rolling metrics at each bar: cumulative return, rolling MFE, rolling MAE,
     drawdown from peak, consecutive adverse bars
  3. Compare these rolling metrics between eventual winners and losers
  4. Identify if any metric shows a detectable divergence before ~240m

This is an OBSERVATION analysis only. No new rules are created.
"""

import json
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone

REPO_ROOT = Path(__file__).resolve().parent.parent
REC_DIR   = REPO_ROOT / "live_capture" / "recommendations"
CACHE_1M  = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"

HORIZONS_1M = {"H15": 15, "H60": 60, "H120": 120, "H180": 180, "H300": 300}
THRESHOLD   = 0.002
MFE_OVERRIDE = 0.001

# Checkpoints (in minutes = bar indices for 1m) to sample rolling state
CHECKPOINTS = [15, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300]

# ── Frozen Phase 4 rules ───────────────────────────────────────────────────────

def bucket(ret, t=THRESHOLD):
    if ret is None: return "FLAT"
    return "FAV" if ret > t else ("ADV" if ret < -t else "FLAT")

def classify_h60(h15_ret, h60_ret, mfe_h60):
    if h15_ret is None or h60_ret is None: return "UNKNOWN"
    if mfe_h60 is not None and mfe_h60 < MFE_OVERRIDE: return "AVOID"
    b15 = bucket(h15_ret); b60 = bucket(h60_ret)
    if b15 == "FAV" and b60 == "FAV": return "ENTER"
    if b15 == "ADV" and b60 == "ADV": return "AVOID"
    return "WAIT"

# ── Data loading ───────────────────────────────────────────────────────────────

def load_json(path):
    with open(path) as f: return json.load(f)

def parse_iso_to_unix(ts_str):
    if not ts_str: return None
    try:
        return datetime.fromisoformat(ts_str.replace("Z", "+00:00")).timestamp()
    except Exception: return None

def load_intraday_cache(cache_dir):
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

def load_live003_decisions(min_cohort_date="2026-09-03"):
    decisions = []
    for p in sorted(REC_DIR.glob("LIVE-003-*.json")):
        try:
            data = load_json(p)
            snap_ts = data.get("source_snapshot_timestamp")
            snap_unix = parse_iso_to_unix(snap_ts)
            fname = p.stem; parts = fname.split("-")
            date_str = parts[2] if len(parts) > 2 else ""
            cohort_date = f"{date_str[:4]}-{date_str[4:6]}-{date_str[6:8]}" if len(date_str) == 8 else date_str
            if cohort_date < min_cohort_date or snap_unix is None: continue
            for r in data.get("recommendations", []):
                if r.get("action") != "Watch": continue
                ticker = r.get("instrument") or r.get("ticker")
                if not ticker: continue
                decisions.append({
                    "ticker": ticker,
                    "ticker_ns": ticker.replace(".", "_"),
                    "direction": r.get("direction"),
                    "reference_price": r.get("reference_price"),
                    "cohort_date": cohort_date,
                    "snap_unix": snap_unix,
                })
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)
    return decisions

# ── Path extraction and rolling metrics ───────────────────────────────────────

def extract_full_path(decision, cache_idx, n_bars=300):
    """
    Extract the full 1m path for a decision (up to n_bars bars after entry).
    Returns list of direction-adjusted returns at each bar, or None if no data.
    """
    ticker_ns = decision["ticker_ns"]
    snap_unix = decision["snap_unix"]
    direction = decision["direction"]

    if not snap_unix or ticker_ns not in cache_idx: return None
    bars = cache_idx[ticker_ns]
    start_idx = next((i for i, b in enumerate(bars) if b["timestamp"] > snap_unix), None)
    if start_idx is None: return None

    entry_price = bars[start_idx].get("close")
    if not entry_price or entry_price == 0: return None

    is_short = (direction == "SHORT")
    path = []
    for i in range(n_bars):
        idx = start_idx + i
        if idx >= len(bars): break
        p = bars[idx].get("close")
        if p is None: break
        ret = (entry_price - p) / entry_price if is_short else (p - entry_price) / entry_price
        path.append(ret)

    return path if len(path) >= 60 else None  # need at least H60 worth of bars

def rolling_metrics(path):
    """
    Compute rolling metrics at each bar in the path.
    Returns dict of metric_name → list of values (one per bar).
    """
    n = len(path)
    cumret    = path[:]  # cumulative return at each bar (= path value itself)
    peak      = [None] * n
    drawdown  = [None] * n
    consec_adv = [0] * n  # consecutive adverse bars ending at this bar

    running_peak = path[0]
    consec = 0

    for i in range(n):
        r = path[i]
        if r > running_peak:
            running_peak = r
        peak[i] = running_peak
        drawdown[i] = r - running_peak  # negative = drawdown from peak

        if r < -THRESHOLD:
            consec += 1
        else:
            consec = 0
        consec_adv[i] = consec

    return {
        "cumret": cumret,
        "peak": peak,
        "drawdown": drawdown,
        "consec_adv": consec_adv,
    }

def first_adverse_bar(path):
    """Bar index where cumulative return first goes below -THRESHOLD."""
    for i, r in enumerate(path):
        if r < -THRESHOLD:
            return i
    return None

def max_drawdown_by(path, bar):
    """Maximum drawdown (from peak) up to bar index."""
    if bar >= len(path): bar = len(path) - 1
    peak = path[0]
    max_dd = 0.0
    for i in range(bar + 1):
        if path[i] > peak: peak = path[i]
        dd = path[i] - peak
        if dd < max_dd: max_dd = dd
    return max_dd

def cumret_at(path, bar):
    if bar < len(path): return path[bar]
    return None

# ── Analysis ───────────────────────────────────────────────────────────────────

def analyse_short_deterioration(decisions, cache_idx):
    print(f"\n  Extracting SHORT ENTER paths...")

    short_enters = []
    for dec in decisions:
        if dec["direction"] != "SHORT": continue
        path = extract_full_path(dec, cache_idx, n_bars=300)
        if path is None: continue

        # Classify using frozen H60 rules
        h15 = cumret_at(path, 14)   # bar 15 (0-indexed: bar 14)
        h60 = cumret_at(path, 59)   # bar 60
        mfe_h60 = max(path[:60]) if len(path) >= 60 else None
        classification = classify_h60(h15, h60, mfe_h60)
        if classification != "ENTER": continue

        h300 = cumret_at(path, 299)
        if h300 is None: continue

        winner = h300 > THRESHOLD
        fab = first_adverse_bar(path)

        # Rolling metrics at each checkpoint
        metrics = rolling_metrics(path)
        checkpoint_data = {}
        for cp in CHECKPOINTS:
            bar = cp - 1  # 0-indexed
            if bar < len(path):
                checkpoint_data[cp] = {
                    "cumret": path[bar],
                    "peak": metrics["peak"][bar],
                    "drawdown": metrics["drawdown"][bar],
                    "consec_adv": metrics["consec_adv"][bar],
                    "max_dd_by": max_drawdown_by(path, bar),
                }

        short_enters.append({
            **dec,
            "path": path,
            "H300": h300,
            "winner": winner,
            "first_adverse_bar": fab,
            "checkpoints": checkpoint_data,
        })

    winners = [m for m in short_enters if m["winner"]]
    losers  = [m for m in short_enters if not m["winner"]]
    print(f"  SHORT ENTER: {len(short_enters)} total  Winners: {len(winners)}  Losers: {len(losers)}")
    return short_enters, winners, losers

# ── Printing ───────────────────────────────────────────────────────────────────

def pct(v, decimals=2):
    if v is None: return "—"
    return f"{v*100:+.{decimals}f}%"

def med(group, key, cp):
    vals = [m["checkpoints"][cp][key] for m in group
            if cp in m["checkpoints"] and m["checkpoints"][cp].get(key) is not None]
    return statistics.median(vals) if vals else None

def print_checkpoint_comparison(winners, losers):
    print(f"\n{'='*80}")
    print(f"  SHORT ENTER — Rolling Path Comparison: Winners vs Losers")
    print(f"{'='*80}")
    print(f"  Winners N={len(winners)}  Losers N={len(losers)}")

    print(f"\n  ── Cumulative Return at Checkpoint ──────────────────────────────────────")
    print(f"  {'Min':>6} {'W med':>10} {'L med':>10} {'Gap':>10}  Note")
    print(f"  {'-'*60}")
    for cp in CHECKPOINTS:
        w_med = med(winners, "cumret", cp)
        l_med = med(losers,  "cumret", cp)
        gap = (w_med - l_med) if w_med is not None and l_med is not None else None
        note = ""
        if gap is not None and abs(gap) > 0.003:
            note = "← material gap"
        print(f"  {cp:>4}m {pct(w_med):>10} {pct(l_med):>10} {pct(gap):>10}  {note}")

    print(f"\n  ── Drawdown from Peak at Checkpoint ─────────────────────────────────────")
    print(f"  {'Min':>6} {'W med':>10} {'L med':>10} {'Gap':>10}  Note")
    print(f"  {'-'*60}")
    for cp in CHECKPOINTS:
        w_med = med(winners, "drawdown", cp)
        l_med = med(losers,  "drawdown", cp)
        gap = (w_med - l_med) if w_med is not None and l_med is not None else None
        note = ""
        if gap is not None and abs(gap) > 0.002:
            note = "← material gap"
        print(f"  {cp:>4}m {pct(w_med):>10} {pct(l_med):>10} {pct(gap):>10}  {note}")

    print(f"\n  ── Max Drawdown from Peak (cumulative to checkpoint) ────────────────────")
    print(f"  {'Min':>6} {'W med':>10} {'L med':>10} {'Gap':>10}  Note")
    print(f"  {'-'*60}")
    for cp in CHECKPOINTS:
        w_med = med(winners, "max_dd_by", cp)
        l_med = med(losers,  "max_dd_by", cp)
        gap = (w_med - l_med) if w_med is not None and l_med is not None else None
        note = ""
        if gap is not None and abs(gap) > 0.002:
            note = "← material gap"
        print(f"  {cp:>4}m {pct(w_med):>10} {pct(l_med):>10} {pct(gap):>10}  {note}")

    print(f"\n  ── Consecutive Adverse Bars at Checkpoint ───────────────────────────────")
    print(f"  {'Min':>6} {'W med':>10} {'L med':>10}  Note")
    print(f"  {'-'*50}")
    for cp in CHECKPOINTS:
        w_med = med(winners, "consec_adv", cp)
        l_med = med(losers,  "consec_adv", cp)
        note = ""
        if w_med is not None and l_med is not None and abs(w_med - l_med) > 1:
            note = "← material gap"
        w_str = f"{w_med:.1f}" if w_med is not None else "—"
        l_str = f"{l_med:.1f}" if l_med is not None else "—"
        print(f"  {cp:>4}m {w_str:>10} {l_str:>10}  {note}")

def print_first_adverse_bar(winners, losers):
    print(f"\n{'='*80}")
    print(f"  SHORT ENTER — First Adverse Bar Distribution")
    print(f"{'='*80}")

    def fab_stats(group, label):
        fabs = [m["first_adverse_bar"] for m in group if m["first_adverse_bar"] is not None]
        never = sum(1 for m in group if m["first_adverse_bar"] is None)
        if not fabs:
            print(f"  {label}: N={len(group)}  no adverse bars in any trade")
            return
        fabs_min = [f for f in fabs]
        fabs_min.sort()
        print(f"  {label} (N={len(group)}):")
        print(f"    Never adverse: {never}")
        print(f"    First adverse bar (bars): min={min(fabs_min)}  "
              f"p25={fabs_min[len(fabs_min)//4]}  "
              f"median={statistics.median(fabs_min):.0f}  "
              f"p75={fabs_min[3*len(fabs_min)//4]}  "
              f"max={max(fabs_min)}")
        print(f"    First adverse bar (min):  min={min(fabs_min)}  "
              f"p25={fabs_min[len(fabs_min)//4]}  "
              f"median={statistics.median(fabs_min):.0f}  "
              f"p75={fabs_min[3*len(fabs_min)//4]}  "
              f"max={max(fabs_min)}")
        # Distribution by time bucket
        buckets = [(0,15,"0–15m"), (15,60,"15–60m"), (60,120,"60–120m"),
                   (120,180,"120–180m"), (180,240,"180–240m"), (240,300,"240–300m")]
        print(f"    Distribution:")
        for lo, hi, lbl in buckets:
            n = sum(1 for f in fabs_min if lo <= f < hi)
            if n > 0:
                print(f"      {lbl}: {n}")

    fab_stats(winners, "Winners")
    fab_stats(losers,  "Losers")

def print_product_assessment(winners, losers):
    print(f"\n{'='*80}")
    print(f"  PRODUCT ASSESSMENT — Can continuous monitoring detect SHORT failures?")
    print(f"{'='*80}")
    print("""
  The question: Is there a detectable deterioration signal in the 1m path
  between entry and ~240m that distinguishes SHORT losers from winners?

  Evaluation criteria:
    A. Is there a material gap in any rolling metric between winners and losers
       at a checkpoint BEFORE 240m?
    B. Is that gap large enough to be actionable (>0.3% cumret, >0.2% drawdown)?
    C. Does the gap appear consistently (not just in 1-2 trades)?

  If A+B+C: continuous monitoring is a viable product capability.
  If not:   SHORT failures are not detectable before they occur — the ~240m
            timing is an outcome, not a signal.

  See checkpoint comparison above for the actual gaps.
  Look for rows marked '← material gap' before the 240m checkpoint.
""")

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("=" * 80)
    print("P4 — SHORT ENTER Continuous Deterioration Analysis")
    print("=" * 80)

    print("\nLoading LIVE-003 Watch decisions (Sep 3–11)...")
    decisions = load_live003_decisions()
    short_decisions = [d for d in decisions if d["direction"] == "SHORT"]
    print(f"  SHORT Watch decisions: {len(short_decisions)}")

    print("\nLoading 1m cache...")
    cache_1m = load_intraday_cache(CACHE_1M)
    print(f"  Tickers: {len(cache_1m)}")

    short_enters, winners, losers = analyse_short_deterioration(decisions, cache_1m)

    if not short_enters:
        print("\n  No SHORT ENTER decisions found. Check data.")
        return

    print_first_adverse_bar(winners, losers)
    print_checkpoint_comparison(winners, losers)
    print_product_assessment(winners, losers)

    # Summary table for documentation
    print(f"\n{'='*80}")
    print(f"  SUMMARY TABLE (for docs/P4_LOSS_ANALYSIS.md)")
    print(f"{'='*80}")
    print(f"  SHORT ENTER: N={len(short_enters)}  Winners={len(winners)}  Losers={len(losers)}")
    print(f"  Win rate: {len(winners)/len(short_enters)*100:.1f}%")

    # Find earliest checkpoint with material gap
    material_gaps = []
    for cp in CHECKPOINTS:
        w_med = med(winners, "cumret", cp)
        l_med = med(losers,  "cumret", cp)
        if w_med is not None and l_med is not None:
            gap = abs(w_med - l_med)
            if gap > 0.003:
                material_gaps.append((cp, gap, w_med, l_med))

    if material_gaps:
        earliest = material_gaps[0]
        print(f"\n  Earliest material cumret gap: {earliest[0]}m "
              f"(W={pct(earliest[2])} L={pct(earliest[3])} gap={pct(earliest[1])})")
        print(f"  → Continuous monitoring MAY be viable from {earliest[0]}m onward")
    else:
        print(f"\n  No material cumret gap found before 240m")
        print(f"  → Continuous monitoring does NOT reliably distinguish SHORT losers before failure")

if __name__ == "__main__":
    run()