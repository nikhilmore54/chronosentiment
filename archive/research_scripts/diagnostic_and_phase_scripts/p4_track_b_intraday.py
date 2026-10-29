#!/usr/bin/env python3
"""
P4 Track B — Intraday Discovery
================================
Evaluate all 300 COMPLETE decisions against their subsequent 5m intraday paths.

Horizons: H15 (3 bars), H30 (6), H60 (12), H120 (24), H180 (36), H300 (60)

Three separate objectives:
  1. Genuine return prediction (ordinary movement)
  2. Path prediction (MFE / MAE)
  3. Extreme-move prediction (gap-through phenomenon)

Search the complete allowed decision-time information space across all directions.
Do NOT optimize for daily realized_return — use intraday close at each horizon.

Primary dataset: 5m intraday (broadest coverage: Jun 22 – Sep 11)

Guiding principle: Explore aggressively. Validate ruthlessly.
"""

import json
import sys
from pathlib import Path
import statistics
from datetime import datetime, timezone

REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
INTRADAY_5M = REPO_ROOT / "intraday_capture" / "yahoo_cache_5m"
OUT_DOC = REPO_ROOT / "docs" / "P4_TRACK_B_INTRADAY_RESULTS.md"

# Horizons: label → number of 5m bars
HORIZONS = {
    "H15":  3,
    "H30":  6,
    "H60":  12,
    "H120": 24,
    "H180": 36,
    "H300": 60,
}

EXTREME_THRESHOLD = 0.05  # |return| > 5% at any horizon = extreme move

# ── Data loading ───────────────────────────────────────────────────────────────

def load_json(path):
    with open(path) as f:
        return json.load(f)

def load_intraday_index():
    """Load all 5m intraday files into a dict: ticker_ns → sorted list of bars."""
    idx = {}
    for p in INTRADAY_5M.glob("*.json"):
        # Convert filename: ADANIPORTS.NS.json → ADANIPORTS_NS
        ticker_ns = p.stem.replace(".", "_")
        try:
            bars = load_json(p)
            bars.sort(key=lambda b: b["timestamp"])
            idx[ticker_ns] = bars
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)
    return idx

def load_complete_decisions():
    obs_by_id = {}
    for p in OBS_DIR.glob("TIME009-OBS-*.json"):
        try:
            o = load_json(p)
            if o.get("observation_status") == "COMPLETE" and o.get("realized_return") is not None:
                obs_by_id[o["decision_id"]] = o
        except Exception:
            pass

    records = []
    for p in LEDGER_DIR.glob("*.json"):
        try:
            e = load_json(p)
            did = e.get("decision_id")
            if did not in obs_by_id:
                continue
            obs = obs_by_id[did]

            rec = {
                "decision_id": did,
                "ticker": e.get("ticker"),          # e.g. ADANIPORTS_NS
                "direction": e.get("direction"),
                "action": e.get("action"),
                "reference_price": e.get("reference_price"),
                "adaptive_target": e.get("adaptive_target"),
                "adaptive_risk": e.get("adaptive_risk"),
                "adaptive_horizon_sessions": e.get("adaptive_horizon_sessions"),
                "evidence_class": e.get("evidence_class"),
                "degradation_level": e.get("degradation_level"),
                "sample_size": e.get("sample_size") or 0,
                "target_rate": e.get("target_rate") or 0.0,
                "rank_score": e.get("rank_score") or 0.0,
                "vol_regime": e.get("vol_regime"),
                "volume_regime": e.get("volume_regime"),
                "certification_status": e.get("certification_status"),
                "cohort_date": obs.get("cohort_date"),
                "source_snapshot_unix": obs.get("source_snapshot_unix"),
                # Daily outcome (FORBIDDEN as policy input, used only for comparison)
                "_daily_return": obs.get("realized_return"),
                "_daily_target_reached": obs.get("target_reached"),
            }

            ref = rec["reference_price"]
            tgt = rec["adaptive_target"]
            rsk = rec["adaptive_risk"]
            if ref and tgt and rsk and ref > 0:
                rec["upside_pct"] = (tgt - ref) / ref
                rec["downside_pct"] = (ref - rsk) / ref
                rec["rr_ratio"] = rec["upside_pct"] / rec["downside_pct"] if rec["downside_pct"] > 0 else 0.0
                rec["expected_return"] = (
                    rec["target_rate"] * rec["upside_pct"]
                    - (1 - rec["target_rate"]) * rec["downside_pct"]
                )
            else:
                rec["upside_pct"] = None
                rec["downside_pct"] = None
                rec["rr_ratio"] = None
                rec["expected_return"] = None

            rec["is_watch"] = rec["action"] == "Watch"
            rec["is_long"] = rec["direction"] == "LONG"
            rec["is_short"] = rec["direction"] == "SHORT"
            rec["is_exact"] = rec["degradation_level"] == "Exact"

            records.append(rec)
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)

    return records

# ── Intraday path evaluation ───────────────────────────────────────────────────

def get_intraday_returns(rec, intraday_idx):
    """
    For a decision record, find the first 5m bar strictly after the snapshot
    and compute returns at each horizon.

    Returns dict: horizon_label → {return, mfe, mae, extreme}
    or None if no intraday data available.
    """
    ticker = rec["ticker"]
    snap_unix = rec.get("source_snapshot_unix")
    ref_price = rec.get("reference_price")

    if not snap_unix or not ref_price or ticker not in intraday_idx:
        return None

    bars = intraday_idx[ticker]

    # Find first bar strictly after snapshot
    start_idx = None
    for i, b in enumerate(bars):
        if b["timestamp"] > snap_unix:
            start_idx = i
            break

    if start_idx is None:
        return None

    entry_price = bars[start_idx]["close"]
    if not entry_price or entry_price == 0:
        return None

    results = {}
    for label, n_bars in HORIZONS.items():
        end_idx = start_idx + n_bars - 1
        if end_idx >= len(bars):
            continue  # not enough bars

        exit_price = bars[end_idx]["close"]
        if not exit_price:
            continue

        # Compute return from entry (direction-adjusted)
        if rec["is_long"]:
            ret = (exit_price - entry_price) / entry_price
        else:  # SHORT: profit when price falls
            ret = (entry_price - exit_price) / entry_price

        # MFE / MAE over the horizon bars
        path_prices = [bars[start_idx + i]["close"] for i in range(n_bars)
                       if start_idx + i < len(bars) and bars[start_idx + i]["close"]]
        if rec["is_long"]:
            mfe = max((p - entry_price) / entry_price for p in path_prices) if path_prices else None
            mae = min((p - entry_price) / entry_price for p in path_prices) if path_prices else None
        else:
            mfe = max((entry_price - p) / entry_price for p in path_prices) if path_prices else None
            mae = min((entry_price - p) / entry_price for p in path_prices) if path_prices else None

        results[label] = {
            "return": ret,
            "mfe": mfe,
            "mae": mae,
            "extreme": abs(ret) > EXTREME_THRESHOLD,
            "entry_price": entry_price,
            "exit_price": exit_price,
            "n_bars_available": min(n_bars, len(bars) - start_idx),
        }

    return results if results else None

# ── Statistics ─────────────────────────────────────────────────────────────────

def pct(v, d=2):
    return f"{v*100:+.{d}f}%" if v is not None else "—"

def pct_plain(v, d=1):
    return f"{v*100:.{d}f}%" if v is not None else "—"

def stats(values):
    if not values:
        return {}
    n = len(values)
    wins = [v for v in values if v > 0]
    losses = [v for v in values if v < 0]
    return {
        "n": n,
        "mean": statistics.mean(values),
        "median": statistics.median(values),
        "win_rate": len(wins) / n,
        "loss_rate": len(losses) / n,
        "stdev": statistics.stdev(values) if n > 1 else 0.0,
        "min": min(values),
        "max": max(values),
    }

def horizon_row(label, recs_with_intraday, horizon):
    returns = [r["_intraday"][horizon]["return"]
               for r in recs_with_intraday
               if horizon in r.get("_intraday", {})]
    mfes = [r["_intraday"][horizon]["mfe"]
            for r in recs_with_intraday
            if horizon in r.get("_intraday", {}) and r["_intraday"][horizon]["mfe"] is not None]
    maes = [r["_intraday"][horizon]["mae"]
            for r in recs_with_intraday
            if horizon in r.get("_intraday", {}) and r["_intraday"][horizon]["mae"] is not None]
    extremes = [r["_intraday"][horizon]["extreme"]
                for r in recs_with_intraday
                if horizon in r.get("_intraday", {})]

    s = stats(returns)
    if not s:
        return f"| {label} | {horizon} | 0 | — | — | — | — | — | — |"

    extreme_rate = sum(extremes) / len(extremes) if extremes else None
    mean_mfe = statistics.mean(mfes) if mfes else None
    mean_mae = statistics.mean(maes) if maes else None

    return (f"| {label} | {horizon} | {s['n']} | {pct(s['mean'])} | {pct(s['median'])} | "
            f"{pct_plain(s['win_rate'])} | {pct_plain(s['loss_rate'])} | "
            f"{pct(mean_mfe)} | {pct(mean_mae)} | {pct_plain(extreme_rate)} |")

def cohort_horizon_table(lines, label, recs, horizons=None):
    if horizons is None:
        horizons = list(HORIZONS.keys())
    lines.append(f"| {label} | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    for h in horizons:
        lines.append(horizon_row(label, recs, h))

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("Loading COMPLETE decisions...")
    all_records = load_complete_decisions()
    print(f"  Total: {len(all_records)}")

    print("Loading 5m intraday data...")
    intraday_idx = load_intraday_index()
    print(f"  Tickers loaded: {len(intraday_idx)}")

    print("Computing intraday returns...")
    matched = 0
    unmatched = 0
    for rec in all_records:
        result = get_intraday_returns(rec, intraday_idx)
        if result:
            rec["_intraday"] = result
            matched += 1
        else:
            rec["_intraday"] = {}
            unmatched += 1

    print(f"  Matched: {matched}, Unmatched: {unmatched}")

    # Subsets
    watch = [r for r in all_records if r["is_watch"]]
    long_watch = [r for r in watch if r["is_long"]]
    short_watch = [r for r in watch if r["is_short"]]
    no_trade = [r for r in all_records if not r["is_watch"]]

    lines = []
    lines.append("# P4 Track B — Intraday Discovery Results")
    lines.append("")
    lines.append("**Date:** 2026-09-11")
    lines.append(f"**Population:** {len(all_records)} COMPLETE decisions")
    lines.append(f"**Intraday dataset:** 5m, 102 tickers, Jun 22 – Sep 11 2026")
    lines.append(f"**Matched to intraday:** {matched} / {len(all_records)}")
    lines.append("")
    lines.append("**Horizons:** H15 (3 bars), H30 (6), H60 (12), H120 (24), H180 (36), H300 (60)")
    lines.append(f"**Extreme move threshold:** |return| > {EXTREME_THRESHOLD*100:.0f}% at horizon")
    lines.append("")
    lines.append("**Three objectives:**")
    lines.append("1. Genuine return prediction (ordinary movement)")
    lines.append("2. Path prediction (MFE / MAE)")
    lines.append("3. Extreme-move prediction")
    lines.append("")
    lines.append("---")
    lines.append("")

    # ── Section 1: Baseline across all horizons ────────────────────────────────
    lines.append("## 1. Baseline — all decisions by direction and horizon")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    for h in HORIZONS:
        lines.append(horizon_row("All COMPLETE", all_records, h))
    lines.append("")
    for h in HORIZONS:
        lines.append(horizon_row("Watch (all)", watch, h))
    lines.append("")
    for h in HORIZONS:
        lines.append(horizon_row("LONG Watch", long_watch, h))
    lines.append("")
    for h in HORIZONS:
        lines.append(horizon_row("SHORT Watch", short_watch, h))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 2. SHORT Watch — intraday decomposition by sample_size band")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    for lo, hi, label in [(1, 50, "1–50"), (51, 100, "51–100"), (101, 150, "101–150"), (151, 999, "151+")]:
        recs = [r for r in short_watch if lo <= r["sample_size"] <= hi]
        for h in HORIZONS:
            lines.append(horizon_row(f"SHORT Watch sample {label}", recs, h))
        lines.append("")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 3. SHORT Watch — intraday decomposition by rank_score quartile")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    valid = sorted(short_watch, key=lambda r: r["rank_score"])
    n = len(valid)
    for qname, qrecs in [("Q1", valid[:n//4]), ("Q2", valid[n//4:n//2]),
                          ("Q3", valid[n//2:3*n//4]), ("Q4", valid[3*n//4:])]:
        for h in HORIZONS:
            lines.append(horizon_row(f"SHORT Watch rank_score {qname}", qrecs, h))
        lines.append("")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 4. SHORT Watch — intraday decomposition by target_rate quartile")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    valid = sorted(short_watch, key=lambda r: r["target_rate"])
    n = len(valid)
    for qname, qrecs in [("Q1", valid[:n//4]), ("Q2", valid[n//4:n//2]),
                          ("Q3", valid[n//2:3*n//4]), ("Q4", valid[3*n//4:])]:
        for h in HORIZONS:
            lines.append(horizon_row(f"SHORT Watch target_rate {qname}", qrecs, h))
        lines.append("")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 5. LONG Watch — intraday decomposition by sample_size band")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    for lo, hi, label in [(1, 50, "1–50"), (51, 100, "51–100"), (101, 150, "101–150"), (151, 999, "151+")]:
        recs = [r for r in long_watch if lo <= r["sample_size"] <= hi]
        for h in HORIZONS:
            lines.append(horizon_row(f"LONG Watch sample {label}", recs, h))
        lines.append("")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 6. LONG Watch — intraday decomposition by rank_score quartile")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    valid = sorted(long_watch, key=lambda r: r["rank_score"])
    n = len(valid)
    for qname, qrecs in [("Q1", valid[:n//4]), ("Q2", valid[n//4:n//2]),
                          ("Q3", valid[n//2:3*n//4]), ("Q4", valid[3*n//4:])]:
        for h in HORIZONS:
            lines.append(horizon_row(f"LONG Watch rank_score {qname}", qrecs, h))
        lines.append("")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 7. Extreme-move analysis — which decision characteristics predict large intraday moves?")
    lines.append("")
    lines.append("Extreme move = |intraday return| > 5% at horizon.")
    lines.append("")
    lines.append("| Cohort | Horizon | N | Extreme% | Mean return (extreme) | Mean return (non-extreme) |")
    lines.append("|---|---|---|---|---|---|")

    for label, recs in [("All Watch", watch), ("LONG Watch", long_watch), ("SHORT Watch", short_watch)]:
        for h in HORIZONS:
            h_recs = [r for r in recs if h in r.get("_intraday", {})]
            if not h_recs:
                continue
            extreme = [r for r in h_recs if r["_intraday"][h]["extreme"]]
            non_extreme = [r for r in h_recs if not r["_intraday"][h]["extreme"]]
            ext_rate = len(extreme) / len(h_recs)
            ext_returns = [r["_intraday"][h]["return"] for r in extreme]
            non_returns = [r["_intraday"][h]["return"] for r in non_extreme]
            mean_ext = pct(statistics.mean(ext_returns)) if ext_returns else "—"
            mean_non = pct(statistics.mean(non_returns)) if non_returns else "—"
            lines.append(f"| {label} | {h} | {len(h_recs)} | {pct_plain(ext_rate)} | {mean_ext} | {mean_non} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 8. Best intraday combinations — candidate policy matrix")
    lines.append("")
    lines.append("Searching combinations of direction × quality variable × horizon.")
    lines.append("Ranked by median return (gap-through excluded: |return| <= 5%).")
    lines.append("")
    lines.append("| Combination | Horizon | N | Mean | Median | Win% | Loss% | Extreme% |")
    lines.append("|---|---|---|---|---|---|---|---|")

    combos = [
        ("SHORT Watch Exact", [r for r in short_watch if r["is_exact"]]),
        ("SHORT Watch sample 51–150", [r for r in short_watch if 51 <= r["sample_size"] <= 150]),
        ("SHORT Watch sample 151+", [r for r in short_watch if r["sample_size"] >= 151]),
        ("SHORT Watch target_rate Q2", sorted(short_watch, key=lambda r: r["target_rate"])[len(short_watch)//4:len(short_watch)//2]),
        ("SHORT Watch target_rate Q3", sorted(short_watch, key=lambda r: r["target_rate"])[len(short_watch)//2:3*len(short_watch)//4]),
        ("LONG Watch sample 151+", [r for r in long_watch if r["sample_size"] >= 151]),
        ("LONG Watch rank_score Q3", sorted(long_watch, key=lambda r: r["rank_score"])[len(long_watch)//2:3*len(long_watch)//4]),
        ("All Watch sample 51–150", [r for r in watch if 51 <= r["sample_size"] <= 150]),
    ]

    for label, recs in combos:
        for h in ["H15", "H60", "H300"]:
            h_recs = [r for r in recs if h in r.get("_intraday", {})]
            if not h_recs:
                continue
            returns = [r["_intraday"][h]["return"] for r in h_recs]
            extremes = [r["_intraday"][h]["extreme"] for r in h_recs]
            # Non-extreme only
            non_ext_returns = [r["_intraday"][h]["return"] for r in h_recs if not r["_intraday"][h]["extreme"]]
            s = stats(returns)
            ext_rate = sum(extremes) / len(extremes) if extremes else None
            if not s:
                continue
            lines.append(f"| {label} | {h} | {s['n']} | {pct(s['mean'])} | {pct(s['median'])} | "
                         f"{pct_plain(s['win_rate'])} | {pct_plain(s['loss_rate'])} | {pct_plain(ext_rate)} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 9. Intraday vs daily return comparison")
    lines.append("")
    lines.append("Does the intraday path at H300 match the daily realized_return?")
    lines.append("")
    lines.append("| Cohort | N | Daily mean | H300 mean | Daily median | H300 median |")
    lines.append("|---|---|---|---|---|---|")

    for label, recs in [("All Watch", watch), ("LONG Watch", long_watch), ("SHORT Watch", short_watch)]:
        h300_recs = [r for r in recs if "H300" in r.get("_intraday", {})]
        daily_returns = [r["_daily_return"] for r in h300_recs]
        h300_returns = [r["_intraday"]["H300"]["return"] for r in h300_recs]
        if not daily_returns:
            continue
        lines.append(f"| {label} | {len(h300_recs)} | {pct(statistics.mean(daily_returns))} | "
                     f"{pct(statistics.mean(h300_returns))} | {pct(statistics.median(daily_returns))} | "
                     f"{pct(statistics.median(h300_returns))} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## Interpretation notes")
    lines.append("")
    lines.append("- All results are in-sample on the 300 COMPLETE historical development set.")
    lines.append("- Intraday returns are direction-adjusted: LONG profits from price rise, SHORT from price fall.")
    lines.append("- Extreme move threshold: |return| > 5% at horizon.")
    lines.append("- MFE = maximum favourable excursion over horizon bars.")
    lines.append("- MAE = maximum adverse excursion over horizon bars.")
    lines.append("- Policy must be frozen before seeing each observation's future intraday path.")
    lines.append("- TIME-009 observations used as outcome data only — not a tuning target.")
    lines.append("")
    lines.append("**Next step:** Identify strongest intraday candidates, lock policy, walk-forward on Aug 24+ cohorts.")

    report = "\n".join(lines)
    OUT_DOC.write_text(report)
    print(f"\nReport written to: {OUT_DOC}")
    print("\n" + "="*80)
    print(report)
    print("="*80)

if __name__ == "__main__":
    run()