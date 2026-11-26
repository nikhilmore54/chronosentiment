#!/usr/bin/env python3
"""
P4 — LONG ENTER MFE@H60 Distribution Analysis
================================================
Bins LONG ENTER decisions by MFE@H60 magnitude and compares H300 outcomes
per bin at both 1m and 5m resolution.

Question: At what level of favourable movement by H60 does a LONG ENTER
decision become materially better or worse?

Uses the same frozen Phase 4 state machine and same Sep 3-11 cohort as
the 1m vs 5m loss analysis.
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

HORIZONS_1M = {"H15": 15, "H60": 60, "H120": 120, "H180": 180, "H300": 300}
HORIZONS_5M = {"H15":  3, "H60": 12, "H120":  24, "H180":  36, "H300":  60}

THRESHOLD   = 0.002
MFE_OVERRIDE = 0.001

MFE_BINS = [0.0, 0.0025, 0.005, 0.0075, 0.010, 0.0125, 0.015, 0.020, 1.0]
BIN_LABELS = [
    "0–0.25%", "0.25–0.50%", "0.50–0.75%", "0.75–1.00%",
    "1.00–1.25%", "1.25–1.50%", "1.50–2.00%", ">2.00%"
]

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

def measure_path(decision, cache_idx, horizons):
    ticker_ns = decision["ticker_ns"]
    snap_unix = decision["snap_unix"]
    ref_price = decision["reference_price"]
    direction = decision["direction"]
    if not snap_unix or not ref_price or ticker_ns not in cache_idx: return {}
    bars = cache_idx[ticker_ns]
    start_idx = next((i for i, b in enumerate(bars) if b["timestamp"] > snap_unix), None)
    if start_idx is None: return {}
    entry_price = bars[start_idx].get("close")
    if not entry_price or entry_price == 0: return {}
    is_short = (direction == "SHORT")
    results = {}
    for label, n_bars in horizons.items():
        end_idx = start_idx + n_bars - 1
        if end_idx >= len(bars): continue
        exit_price = bars[end_idx].get("close")
        if not exit_price: continue
        ret = (entry_price - exit_price) / entry_price if is_short else (exit_price - entry_price) / entry_price
        path_prices = [bars[start_idx + i].get("close") for i in range(n_bars)
                       if start_idx + i < len(bars) and bars[start_idx + i].get("close")]
        mfe = mae = None
        if path_prices:
            excursions = [(entry_price - p) / entry_price if is_short else (p - entry_price) / entry_price
                          for p in path_prices]
            mfe = max(excursions); mae = min(excursions)
        results[label] = {"ret": ret, "mfe": mfe, "mae": mae,
                          "entry_price": entry_price, "exit_price": exit_price}
    return results

# ── Distribution analysis ──────────────────────────────────────────────────────

def bin_index(mfe_val):
    if mfe_val is None: return None
    for i in range(len(MFE_BINS) - 1):
        if MFE_BINS[i] <= mfe_val < MFE_BINS[i + 1]:
            return i
    return len(MFE_BINS) - 2  # last bin

def analyse_mfe_distribution(decisions, cache_idx, horizons, res_label):
    print(f"\n  [{res_label}] Measuring LONG ENTER paths...")
    long_enters = []
    for dec in decisions:
        if dec["direction"] != "LONG": continue
        path = measure_path(dec, cache_idx, horizons)
        if not path or "H15" not in path or "H60" not in path or "H300" not in path:
            continue
        h15 = path["H15"]["ret"]; h60 = path["H60"]["ret"]
        mfe_h60 = path["H60"]["mfe"]; h300 = path["H300"]["ret"]
        h120 = path.get("H120", {}).get("ret")
        classification = classify_h60(h15, h60, mfe_h60)
        if classification != "ENTER": continue
        long_enters.append({
            **dec,
            "H15": h15, "H60": h60, "H120": h120, "H300": h300,
            "mfe_H60": mfe_h60,
            "mfe_H300": path["H300"]["mfe"],
            "mae_H300": path["H300"]["mae"],
            "winner": h300 > THRESHOLD,
        })

    print(f"  [{res_label}] LONG ENTER decisions: {len(long_enters)}")

    # Bin by MFE@H60
    bins = [[] for _ in BIN_LABELS]
    for m in long_enters:
        idx = bin_index(m["mfe_H60"])
        if idx is not None:
            bins[idx].append(m)

    return long_enters, bins

# ── Printing ───────────────────────────────────────────────────────────────────

def pct(v, decimals=2):
    if v is None: return "—"
    return f"{v*100:+.{decimals}f}%"

def rate(v, decimals=1):
    if v is None: return "—"
    return f"{v*100:.{decimals}f}%"

def print_distribution(res_label, long_enters, bins):
    print(f"\n{'='*70}")
    print(f"  [{res_label}] LONG ENTER — MFE@H60 Distribution vs H300 Outcome")
    print(f"{'='*70}")

    total = len(long_enters)
    winners = [m for m in long_enters if m["winner"]]
    losers  = [m for m in long_enters if not m["winner"]]
    print(f"  Total LONG ENTER: {total}  Winners: {len(winners)}  Losers: {len(losers)}")
    if total > 0:
        print(f"  Overall win rate: {len(winners)/total*100:.1f}%")

    print(f"\n  {'MFE@H60 bin':<16} {'N':>4} {'Winners':>8} {'Win%':>7} "
          f"{'H300 med':>10} {'H300 MFE med':>13} {'H300 MAE med':>13}")
    print(f"  {'-'*72}")

    for i, (label, group) in enumerate(zip(BIN_LABELS, bins)):
        if not group:
            print(f"  {label:<16} {'0':>4}")
            continue
        n = len(group)
        w = sum(1 for m in group if m["winner"])
        h300_vals = [m["H300"] for m in group]
        mfe_vals  = [m["mfe_H300"] for m in group if m["mfe_H300"] is not None]
        mae_vals  = [m["mae_H300"] for m in group if m["mae_H300"] is not None]
        h300_med  = statistics.median(h300_vals) if h300_vals else None
        mfe_med   = statistics.median(mfe_vals) if mfe_vals else None
        mae_med   = statistics.median(mae_vals) if mae_vals else None
        win_rate  = w / n if n > 0 else None
        print(f"  {label:<16} {n:>4} {w:>8} {rate(win_rate):>7} "
              f"{pct(h300_med):>10} {pct(mfe_med):>13} {pct(mae_med):>13}")

    # Winner vs loser MFE@H60 distribution
    print(f"\n  Winner MFE@H60 distribution (N={len(winners)}):")
    if winners:
        mfe_w = sorted(m["mfe_H60"] for m in winners if m["mfe_H60"] is not None)
        if mfe_w:
            print(f"    min={pct(min(mfe_w))}  p25={pct(mfe_w[len(mfe_w)//4])}  "
                  f"median={pct(statistics.median(mfe_w))}  "
                  f"p75={pct(mfe_w[3*len(mfe_w)//4])}  max={pct(max(mfe_w))}")

    print(f"\n  Loser MFE@H60 distribution (N={len(losers)}):")
    if losers:
        mfe_l = sorted(m["mfe_H60"] for m in losers if m["mfe_H60"] is not None)
        if mfe_l:
            print(f"    min={pct(min(mfe_l))}  p25={pct(mfe_l[len(mfe_l)//4])}  "
                  f"median={pct(statistics.median(mfe_l))}  "
                  f"p75={pct(mfe_l[3*len(mfe_l)//4])}  max={pct(max(mfe_l))}")

def print_comparison(enters_1m, bins_1m, enters_5m, bins_5m):
    print(f"\n{'='*70}")
    print(f"  1m vs 5m — LONG ENTER MFE@H60 Bin Comparison")
    print(f"{'='*70}")
    print(f"  {'MFE@H60 bin':<16} {'1m N':>6} {'1m Win%':>8} {'1m H300':>9} "
          f"{'5m N':>6} {'5m Win%':>8} {'5m H300':>9}")
    print(f"  {'-'*65}")

    for i, label in enumerate(BIN_LABELS):
        g1 = bins_1m[i]; g5 = bins_5m[i]
        n1 = len(g1); n5 = len(g5)
        w1 = sum(1 for m in g1 if m["winner"])
        w5 = sum(1 for m in g5 if m["winner"])
        h300_1 = statistics.median([m["H300"] for m in g1]) if g1 else None
        h300_5 = statistics.median([m["H300"] for m in g5]) if g5 else None
        wr1 = w1/n1 if n1 else None
        wr5 = w5/n5 if n5 else None
        print(f"  {label:<16} {n1:>6} {rate(wr1):>8} {pct(h300_1):>9} "
              f"{n5:>6} {rate(wr5):>8} {pct(h300_5):>9}")

    print(f"\n  Overall:")
    total_1m = len(enters_1m); total_5m = len(enters_5m)
    w1 = sum(1 for m in enters_1m if m["winner"])
    w5 = sum(1 for m in enters_5m if m["winner"])
    print(f"    1m: N={total_1m}  win={rate(w1/total_1m if total_1m else None)}")
    print(f"    5m: N={total_5m}  win={rate(w5/total_5m if total_5m else None)}")

def print_threshold_candidates(enters_1m, enters_5m):
    print(f"\n{'='*70}")
    print(f"  THRESHOLD CANDIDATE ANALYSIS")
    print(f"{'='*70}")
    print(f"  Testing: if MFE@H60 >= threshold, classify as HIGH-QUALITY ENTER")
    print(f"  Metric: win rate above vs below threshold at both resolutions")
    print()

    thresholds = [0.003, 0.005, 0.007, 0.010, 0.012, 0.015]

    print(f"  {'Threshold':<12} {'1m above N':>11} {'1m above W%':>12} "
          f"{'1m below N':>11} {'1m below W%':>12} "
          f"{'5m above N':>11} {'5m above W%':>12} "
          f"{'5m below N':>11} {'5m below W%':>12}")
    print(f"  {'-'*105}")

    for t in thresholds:
        a1 = [m for m in enters_1m if m["mfe_H60"] is not None and m["mfe_H60"] >= t]
        b1 = [m for m in enters_1m if m["mfe_H60"] is not None and m["mfe_H60"] < t]
        a5 = [m for m in enters_5m if m["mfe_H60"] is not None and m["mfe_H60"] >= t]
        b5 = [m for m in enters_5m if m["mfe_H60"] is not None and m["mfe_H60"] < t]

        wa1 = sum(1 for m in a1 if m["winner"])
        wb1 = sum(1 for m in b1 if m["winner"])
        wa5 = sum(1 for m in a5 if m["winner"])
        wb5 = sum(1 for m in b5 if m["winner"])

        wr_a1 = wa1/len(a1) if a1 else None
        wr_b1 = wb1/len(b1) if b1 else None
        wr_a5 = wa5/len(a5) if a5 else None
        wr_b5 = wb5/len(b5) if b5 else None

        t_label = f">={t*100:.2f}%"
        print(f"  {t_label:<12} {len(a1):>11} {rate(wr_a1):>12} "
              f"{len(b1):>11} {rate(wr_b1):>12} "
              f"{len(a5):>11} {rate(wr_a5):>12} "
              f"{len(b5):>11} {rate(wr_b5):>12}")

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("=" * 70)
    print("P4 — LONG ENTER MFE@H60 Distribution Analysis")
    print("=" * 70)

    print("\nLoading LIVE-003 Watch decisions (Sep 3–11)...")
    decisions = load_live003_decisions()
    long_decisions = [d for d in decisions if d["direction"] == "LONG"]
    print(f"  LONG Watch decisions: {len(long_decisions)}")

    print("\nLoading 1m cache...")
    cache_1m = load_intraday_cache(CACHE_1M)
    print(f"  Tickers: {len(cache_1m)}")

    print("\nLoading 5m cache...")
    cache_5m = load_intraday_cache(CACHE_5M)
    print(f"  Tickers: {len(cache_5m)}")

    enters_1m, bins_1m = analyse_mfe_distribution(decisions, cache_1m, HORIZONS_1M, "1m")
    enters_5m, bins_5m = analyse_mfe_distribution(decisions, cache_5m, HORIZONS_5M, "5m")

    print_distribution("1m", enters_1m, bins_1m)
    print_distribution("5m", enters_5m, bins_5m)
    print_comparison(enters_1m, bins_1m, enters_5m, bins_5m)
    print_threshold_candidates(enters_1m, enters_5m)

    print(f"\n{'='*70}")
    print(f"  INTERPRETATION GUIDE")
    print(f"{'='*70}")
    print("""
  Look for:
    1. A bin boundary where win rate materially increases (>10pp jump)
    2. That boundary surviving at BOTH 1m and 5m
    3. Sufficient N above and below the boundary (N>=5 in each group)

  If found: candidate threshold for LONG ENTER quality filter.
  If not found: MFE@H60 does not support a threshold — observation only.

  Do NOT select a threshold based on this cohort alone.
  Any candidate must be validated prospectively (Sep 15+).
""")

if __name__ == "__main__":
    run()