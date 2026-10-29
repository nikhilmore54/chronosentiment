#!/usr/bin/env python3
"""
P4 Historical Intraday Opportunity Engine
==========================================
Replays all 417 Watch COMPLETE decisions (Aug 20 – Sep 7) against the 5m
Yahoo cache to produce a larger intraday opportunity dataset.

Output: datasets/p4_opportunity_dataset.json

This script does NOT modify any frozen P4 evidence.
See: docs/P4_HISTORICAL_INTRADAY_OPPORTUNITY_ENGINE.md
"""

import json
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone

REPO_ROOT  = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR    = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
CACHE_5M   = REPO_ROOT / "intraday_capture" / "yahoo_cache_5m"
OUTPUT_DIR = REPO_ROOT / "datasets"

HORIZONS_5M = {"H15": 3, "H30": 6, "H60": 12, "H120": 24, "H180": 36, "H300": 60}
THRESHOLD   = 0.002   # ±0.2% for FAV/ADV classification
MFE_OVERRIDE = 0.001  # MFE@H60 < 0.1% → AVOID override

# ── Frozen Phase 4 classification rules ───────────────────────────────────────

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

def classify_h120(h120_ret, mfe_h120=None):
    if h120_ret is None: return "UNKNOWN"
    if mfe_h120 is not None and mfe_h120 < MFE_OVERRIDE: return "AVOID-LATE"
    b = bucket(h120_ret)
    if b == "FAV": return "ENTER-LATE"
    if b == "ADV": return "AVOID-LATE"
    return "WAIT-LATE"

# ── Data loading ───────────────────────────────────────────────────────────────

def load_json(path):
    with open(path) as f: return json.load(f)

def load_intraday_cache(cache_dir):
    idx = {}
    for p in cache_dir.glob("*.json"):
        ticker_ns = p.stem.replace(".", "_")
        try:
            bars = load_json(p)
            bars.sort(key=lambda b: b["timestamp"])
            idx[ticker_ns] = bars
        except Exception as ex:
            print(f"  [warn] cache {p.name}: {ex}", file=sys.stderr)
    return idx

def load_complete_watch_decisions():
    """
    Load all Watch COMPLETE decisions by joining ledger entries with
    TIME009 observations. Returns list of enriched decision dicts.
    """
    # Index observations by decision_id
    obs_by_id = {}
    for p in OBS_DIR.glob("TIME009-OBS-*.json"):
        try:
            o = load_json(p)
            if (o.get("observation_status") == "COMPLETE"
                    and o.get("realized_return") is not None
                    and o.get("action") == "Watch"):
                obs_by_id[o["decision_id"]] = o
        except Exception as ex:
            print(f"  [warn] obs {p.name}: {ex}", file=sys.stderr)

    print(f"  Watch COMPLETE observations indexed: {len(obs_by_id)}")

    # Join with ledger entries for additional fields
    ledger_by_id = {}
    for p in LEDGER_DIR.glob("*.json"):
        try:
            e = load_json(p)
            did = e.get("decision_id")
            if did and did in obs_by_id:
                ledger_by_id[did] = e
        except Exception as ex:
            print(f"  [warn] ledger {p.name}: {ex}", file=sys.stderr)

    # Build decision list
    decisions = []
    for did, obs in obs_by_id.items():
        ledger = ledger_by_id.get(did, {})
        snap_unix = obs.get("source_snapshot_unix")
        if not snap_unix:
            continue
        ticker = obs.get("ticker") or ledger.get("ticker")
        if not ticker:
            continue
        ticker_ns = ticker.replace(".", "_")
        decisions.append({
            "decision_id": did,
            "ticker": ticker,
            "ticker_ns": ticker_ns,
            "direction": obs.get("direction"),
            "cohort_date": obs.get("cohort_date"),
            "snap_unix": snap_unix,
            "reference_price": obs.get("reference_price"),
            "target_rate": obs.get("target_rate"),
            "rank_score": obs.get("rank_score"),
            "evidence_class": obs.get("evidence_class"),
            "degradation_level": obs.get("degradation_level"),
            "vol_regime": obs.get("vol_regime"),
            "volume_regime": obs.get("volume_regime"),
            "sample_size": obs.get("sample_size"),
            "daily_outcome": "WIN" if (obs.get("realized_return") or 0) > 0 else "LOSS",
            "daily_return": obs.get("realized_return"),
            "actual_mfe": obs.get("actual_mfe"),
            "actual_mae": obs.get("actual_mae"),
        })

    return decisions

# ── Path reconstruction ────────────────────────────────────────────────────────

def reconstruct_path(decision, cache_idx):
    """
    Reconstruct the 5m intraday path for a decision.
    Returns path_5m dict or None if insufficient data.
    """
    ticker_ns = decision["ticker_ns"]
    snap_unix = decision["snap_unix"]
    direction = decision["direction"]

    if not snap_unix or ticker_ns not in cache_idx:
        return None

    bars = cache_idx[ticker_ns]
    # Find first bar strictly after snap_unix
    start_idx = next((i for i, b in enumerate(bars) if b["timestamp"] > snap_unix), None)
    if start_idx is None:
        return None

    entry_price = bars[start_idx].get("close")
    if not entry_price or entry_price == 0:
        return None

    is_short = (direction == "SHORT")
    max_bars = HORIZONS_5M["H300"]  # 60 bars

    # Extract path returns (direction-adjusted)
    path = []
    for i in range(max_bars):
        idx = start_idx + i
        if idx >= len(bars): break
        p = bars[idx].get("close")
        if p is None: break
        ret = (entry_price - p) / entry_price if is_short else (p - entry_price) / entry_price
        path.append(ret)

    if len(path) < HORIZONS_5M["H60"]:  # need at least H60
        return None

    n = len(path)

    # Horizon returns
    def ret_at(h_bars):
        idx = h_bars - 1
        return path[idx] if idx < n else None

    h15  = ret_at(HORIZONS_5M["H15"])
    h30  = ret_at(HORIZONS_5M["H30"])
    h60  = ret_at(HORIZONS_5M["H60"])
    h120 = ret_at(HORIZONS_5M["H120"])
    h180 = ret_at(HORIZONS_5M["H180"])
    h300 = ret_at(HORIZONS_5M["H300"])

    # MFE / MAE at H60 and H300
    def mfe_mae(end_bar):
        window = path[:end_bar]
        if not window: return None, None
        return max(window), min(window)

    mfe_h60, mae_h60   = mfe_mae(HORIZONS_5M["H60"])
    mfe_h300, mae_h300 = mfe_mae(min(HORIZONS_5M["H300"], n))

    # Time to MFE and MAE (bar index within H300 window)
    h300_window = path[:min(HORIZONS_5M["H300"], n)]
    time_to_mfe = h300_window.index(max(h300_window)) if h300_window else None
    time_to_mae = h300_window.index(min(h300_window)) if h300_window else None

    # First adverse bar (direction-adjusted return < -THRESHOLD)
    first_adverse = next((i for i, r in enumerate(path) if r < -THRESHOLD), None)

    # Max consecutive adverse bars
    max_consec = 0
    consec = 0
    for r in path:
        if r < -THRESHOLD:
            consec += 1
            max_consec = max(max_consec, consec)
        else:
            consec = 0

    # Peak-to-final drawdown
    peak = max(path) if path else 0
    final = path[-1] if path else 0
    peak_to_final_dd = (final - peak) if peak > 0 else 0

    # Path reversal: H300 < 0 after being > target_rate/2 at any point
    target_rate = decision.get("target_rate") or 0
    half_target = target_rate / 2
    ever_above_half = any(r > half_target for r in path)
    path_reversal = ever_above_half and (h300 is not None and h300 < 0)

    # Momentum persistence: fraction of bars where cumret > 0
    momentum_persistence = sum(1 for r in path if r > 0) / len(path) if path else 0

    # Phase 4 classifications
    h60_cls  = classify_h60(h15, h60, mfe_h60)
    h120_cls = classify_h120(h120, mfe_h60)  # use mfe_h60 as proxy for mfe_h120

    # MFE@H60 tier
    if mfe_h60 is None:
        mfe_h60_tier = "UNKNOWN"
    elif mfe_h60 < 0.005:
        mfe_h60_tier = "LOW"
    elif mfe_h60 < 0.015:
        mfe_h60_tier = "MID"
    else:
        mfe_h60_tier = "HIGH"

    intraday_winner = (h300 is not None and h300 > THRESHOLD)

    return {
        "bars_available": n,
        "entry_price": entry_price,
        "h15_ret": h15,
        "h30_ret": h30,
        "h60_ret": h60,
        "h120_ret": h120,
        "h180_ret": h180,
        "h300_ret": h300,
        "mfe_h60": mfe_h60,
        "mae_h60": mae_h60,
        "mfe_h300": mfe_h300,
        "mae_h300": mae_h300,
        "time_to_mfe_bars": time_to_mfe,
        "time_to_mae_bars": time_to_mae,
        "first_adverse_bar": first_adverse,
        "max_consec_adverse": max_consec,
        "peak_to_final_drawdown": peak_to_final_dd,
        "path_reversal": path_reversal,
        "momentum_persistence": momentum_persistence,
        "h60_classification": h60_cls,
        "h120_classification": h120_cls,
        "mfe_h60_tier": mfe_h60_tier,
        "intraday_winner": intraday_winner,
    }

# ── Opportunity quality scoring ────────────────────────────────────────────────

def percentile_rank(value, all_values):
    """Rank value within all_values, return 0–1."""
    if not all_values or value is None: return 0.5
    below = sum(1 for v in all_values if v < value)
    return below / len(all_values)

def compute_opportunity_scores(records):
    """
    Compute opportunity quality scores for all records.
    Each component is percentile-ranked within the full dataset.
    """
    # Collect component values
    early_momentum  = [r["path_5m"]["h15_ret"] for r in records if r["path_5m"]["h15_ret"] is not None]
    mfe_h60_vals    = [r["path_5m"]["mfe_h60"] for r in records if r["path_5m"]["mfe_h60"] is not None]
    adverse_vals    = [abs(r["path_5m"]["mae_h300"]) for r in records if r["path_5m"]["mae_h300"] is not None]
    timing_vals     = [r["path_5m"]["time_to_mfe_bars"] for r in records if r["path_5m"]["time_to_mfe_bars"] is not None]
    persist_vals    = [r["path_5m"]["momentum_persistence"] for r in records]

    for rec in records:
        p = rec["path_5m"]
        em  = p.get("h15_ret")
        mfe = p.get("mfe_h60")
        adv = abs(p["mae_h300"]) if p.get("mae_h300") is not None else None
        ttm = p.get("time_to_mfe_bars")
        per = p.get("momentum_persistence", 0)

        # Higher early momentum = better
        mc = percentile_rank(em, early_momentum) if em is not None else 0.5
        # Higher MFE@H60 = better
        fc = percentile_rank(mfe, mfe_h60_vals) if mfe is not None else 0.5
        # Lower adverse exposure = better (invert rank)
        ac = 1 - percentile_rank(adv, adverse_vals) if adv is not None else 0.5
        # Lower time-to-MFE = better (invert rank)
        tc = 1 - percentile_rank(ttm, timing_vals) if ttm is not None else 0.5
        # Higher persistence = better
        pc = percentile_rank(per, persist_vals)

        score = round(100 * (0.30 * mc + 0.25 * fc + 0.20 * ac + 0.15 * tc + 0.10 * pc))

        rec["opportunity_dimensions"] = {
            "early_momentum": em,
            "momentum_persistence": per,
            "adverse_exposure": adv,
            "mfe_h60_tier": p.get("mfe_h60_tier"),
            "opportunity_quality_score": score,
        }

# ── Reporting ──────────────────────────────────────────────────────────────────

def pct(v, d=2):
    if v is None: return "—"
    return f"{v*100:+.{d}f}%"

def rate(v, d=1):
    if v is None: return "—"
    return f"{v*100:.{d}f}%"

def print_summary(records):
    print(f"\n{'='*70}")
    print(f"  HISTORICAL INTRADAY OPPORTUNITY ENGINE — SUMMARY")
    print(f"{'='*70}")
    print(f"  Total records: {len(records)}")

    for direction in ["LONG", "SHORT"]:
        recs = [r for r in records if r["direction"] == direction]
        if not recs: continue
        winners = [r for r in recs if r["path_5m"]["intraday_winner"]]
        daily_w = [r for r in recs if r["daily_outcome"] == "WIN"]
        print(f"\n  {direction} (N={len(recs)})")
        print(f"    Daily WIN rate:    {rate(len(daily_w)/len(recs))}")
        print(f"    Intraday WIN rate: {rate(len(winners)/len(recs))}")

        # H60 classification breakdown
        for cls in ["ENTER", "AVOID", "WAIT", "UNKNOWN"]:
            cls_recs = [r for r in recs if r["path_5m"]["h60_classification"] == cls]
            if not cls_recs: continue
            cls_w = [r for r in cls_recs if r["path_5m"]["intraday_winner"]]
            print(f"    H60 {cls:10}: N={len(cls_recs):3}  intraday win={rate(len(cls_w)/len(cls_recs))}")

    # MFE@H60 distribution for LONG ENTER
    print(f"\n  ── LONG ENTER MFE@H60 Distribution (extended) ──────────────────────")
    long_enters = [r for r in records
                   if r["direction"] == "LONG"
                   and r["path_5m"]["h60_classification"] == "ENTER"]
    print(f"  LONG ENTER N={len(long_enters)}")

    bins = [(0, 0.005, "0–0.50%"), (0.005, 0.010, "0.50–1.00%"),
            (0.010, 0.015, "1.00–1.50%"), (0.015, 0.020, "1.50–2.00%"),
            (0.020, 1.0, ">2.00%")]
    print(f"  {'Bin':<14} {'N':>4} {'Win%':>7} {'H300 med':>10}")
    print(f"  {'-'*38}")
    for lo, hi, label in bins:
        group = [r for r in long_enters
                 if r["path_5m"]["mfe_h60"] is not None
                 and lo <= r["path_5m"]["mfe_h60"] < hi]
        if not group:
            print(f"  {label:<14} {'0':>4}")
            continue
        w = sum(1 for r in group if r["path_5m"]["intraday_winner"])
        h300s = [r["path_5m"]["h300_ret"] for r in group if r["path_5m"]["h300_ret"] is not None]
        h300_med = statistics.median(h300s) if h300s else None
        print(f"  {label:<14} {len(group):>4} {rate(w/len(group)):>7} {pct(h300_med):>10}")

    # SHORT ENTER cumret gap at checkpoints
    print(f"\n  ── SHORT ENTER Cumret Gap (extended) ────────────────────────────────")
    short_enters = [r for r in records
                    if r["direction"] == "SHORT"
                    and r["path_5m"]["h60_classification"] == "ENTER"]
    short_w = [r for r in short_enters if r["path_5m"]["intraday_winner"]]
    short_l = [r for r in short_enters if not r["path_5m"]["intraday_winner"]]
    print(f"  SHORT ENTER N={len(short_enters)}  Winners={len(short_w)}  Losers={len(short_l)}")

    checkpoint_bars = {"15m": 2, "30m": 5, "60m": 11, "120m": 23}
    print(f"  {'CP':>6} {'W med':>10} {'L med':>10} {'Gap':>10}")
    print(f"  {'-'*40}")
    for label, bar_idx in checkpoint_bars.items():
        def cp_ret(group, bar):
            vals = []
            for r in group:
                p = r["path_5m"]
                # Reconstruct from horizon returns
                if bar == 2 and p.get("h15_ret") is not None:
                    vals.append(p["h15_ret"])
                elif bar == 5 and p.get("h30_ret") is not None:
                    vals.append(p["h30_ret"])
                elif bar == 11 and p.get("h60_ret") is not None:
                    vals.append(p["h60_ret"])
                elif bar == 23 and p.get("h120_ret") is not None:
                    vals.append(p["h120_ret"])
            return statistics.median(vals) if vals else None

        w_med = cp_ret(short_w, bar_idx)
        l_med = cp_ret(short_l, bar_idx)
        gap = (w_med - l_med) if w_med is not None and l_med is not None else None
        note = "← material" if gap is not None and abs(gap) > 0.003 else ""
        print(f"  {label:>6} {pct(w_med):>10} {pct(l_med):>10} {pct(gap):>10}  {note}")

    # Opportunity quality score vs intraday win rate
    print(f"\n  ── Opportunity Quality Score vs Intraday Win Rate ───────────────────")
    score_bins = [(0, 25, "Q1 0–25"), (25, 50, "Q2 25–50"),
                  (50, 75, "Q3 50–75"), (75, 101, "Q4 75–100")]
    print(f"  {'Quartile':<14} {'N':>4} {'Intraday Win%':>14} {'Daily Win%':>12}")
    print(f"  {'-'*46}")
    for lo, hi, label in score_bins:
        group = [r for r in records
                 if lo <= r["opportunity_dimensions"]["opportunity_quality_score"] < hi]
        if not group: continue
        iw = sum(1 for r in group if r["path_5m"]["intraday_winner"])
        dw = sum(1 for r in group if r["daily_outcome"] == "WIN")
        print(f"  {label:<14} {len(group):>4} {rate(iw/len(group)):>14} {rate(dw/len(group)):>12}")

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("=" * 70)
    print("P4 Historical Intraday Opportunity Engine")
    print("=" * 70)

    print("\nLoading Watch COMPLETE decisions...")
    decisions = load_complete_watch_decisions()
    print(f"  Total decisions: {len(decisions)}")
    from collections import Counter
    dir_counts = Counter(d["direction"] for d in decisions)
    print(f"  LONG: {dir_counts.get('LONG', 0)}  SHORT: {dir_counts.get('SHORT', 0)}")
    cohort_counts = Counter(d["cohort_date"] for d in decisions)
    print(f"  Cohorts: {sorted(cohort_counts.keys())}")

    print("\nLoading 5m cache...")
    cache_5m = load_intraday_cache(CACHE_5M)
    print(f"  Tickers: {len(cache_5m)}")

    print("\nReconstructing paths...")
    records = []
    skipped = 0
    for dec in decisions:
        path = reconstruct_path(dec, cache_5m)
        if path is None:
            skipped += 1
            continue
        records.append({**dec, "path_5m": path})

    print(f"  Reconstructed: {len(records)}  Skipped: {skipped}")

    print("\nComputing opportunity quality scores...")
    compute_opportunity_scores(records)

    print_summary(records)

    # Write output
    OUTPUT_DIR.mkdir(exist_ok=True)
    out_path = OUTPUT_DIR / "p4_opportunity_dataset.json"
    # Strip raw path arrays from output (keep only computed metrics)
    with open(out_path, "w") as f:
        json.dump(records, f, indent=2)
    print(f"\n  Output written: {out_path}")
    print(f"  Records: {len(records)}")

if __name__ == "__main__":
    run()
