#!/usr/bin/env python3
"""
P4 Track B — Unified Intraday Measurement Dataset
===================================================
Produces one row per decision × horizon (flat records), joining all
decision-time fields with intraday path outcomes at H15/H30/H60/H120/H180/H300.

This is the measurement substrate for opportunity ranking and
ENTER/WAIT/AVOID recommendation logic. It does NOT encode any
ranking or recommendation rules — those are derived separately.

Output: datasets/p4_intraday_flat.csv (and .json)

Columns per row:
  decision_id, ticker, direction, action, cohort_date,
  source_snapshot_unix, certification_status,
  target_rate, rank_score, sample_size, evidence_class,
  degradation_level, vol_regime, volume_regime,
  reference_price, adaptive_target, adaptive_risk,
  adaptive_horizon_sessions,
  upside_pct, downside_pct, rr_ratio, expected_return,
  horizon, n_bars,
  intraday_return, mfe, mae,
  time_to_mfe, time_to_mae,
  early_return_h15,   # return at first 3 bars (H15) regardless of horizon
  target_attained,    # did price reach adaptive_target within horizon?
  risk_attained,      # did price reach adaptive_risk within horizon?
  entry_price, exit_price,
  n_bars_available,
  daily_return, daily_target_reached, daily_exit_reason
"""

import csv
import json
import sys
from pathlib import Path
import statistics

REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
INTRADAY_5M = REPO_ROOT / "intraday_capture" / "yahoo_cache_5m"
OUT_CSV = REPO_ROOT / "datasets" / "p4_intraday_flat.csv"
OUT_JSON = REPO_ROOT / "datasets" / "p4_intraday_flat.json"

HORIZONS = {
    "H15":  3,
    "H30":  6,
    "H60":  12,
    "H120": 24,
    "H180": 36,
    "H300": 60,
}

# ── Data loading ───────────────────────────────────────────────────────────────

def load_json(path):
    with open(path) as f:
        return json.load(f)

def load_intraday_index():
    idx = {}
    for p in INTRADAY_5M.glob("*.json"):
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

            ref = e.get("reference_price")
            tgt = e.get("adaptive_target")
            rsk = e.get("adaptive_risk")

            # Direction-aware R:R (SHORT: target < ref, risk > ref)
            upside_pct = None
            downside_pct = None
            rr_ratio = None
            expected_return = None
            tr = e.get("target_rate") or 0.0

            if ref and tgt and rsk and ref > 0:
                direction = e.get("direction")
                if direction == "SHORT":
                    upside_pct = (ref - tgt) / ref      # positive when tgt < ref
                    downside_pct = (rsk - ref) / ref    # positive when rsk > ref
                else:  # LONG
                    upside_pct = (tgt - ref) / ref
                    downside_pct = (ref - rsk) / ref
                if downside_pct and downside_pct > 0:
                    rr_ratio = upside_pct / downside_pct
                if tr and upside_pct is not None and downside_pct is not None:
                    expected_return = tr * upside_pct - (1 - tr) * downside_pct

            rec = {
                "decision_id": did,
                "ticker": e.get("ticker"),
                "direction": e.get("direction"),
                "action": e.get("action"),
                "cohort_date": obs.get("cohort_date"),
                "source_snapshot_unix": obs.get("source_snapshot_unix"),
                "certification_status": e.get("certification_status"),
                "target_rate": tr,
                "rank_score": e.get("rank_score") or 0.0,
                "sample_size": e.get("sample_size") or 0,
                "evidence_class": e.get("evidence_class"),
                "degradation_level": e.get("degradation_level"),
                "vol_regime": e.get("vol_regime"),
                "volume_regime": e.get("volume_regime"),
                "reference_price": ref,
                "adaptive_target": tgt,
                "adaptive_risk": rsk,
                "adaptive_horizon_sessions": e.get("adaptive_horizon_sessions"),
                "upside_pct": upside_pct,
                "downside_pct": downside_pct,
                "rr_ratio": rr_ratio,
                "expected_return": expected_return,
                # Daily outcome (for comparison only — not a policy input)
                "_daily_return": obs.get("realized_return"),
                "_daily_target_reached": obs.get("target_reached"),
                "_daily_exit_reason": obs.get("exit_reason"),
            }
            records.append(rec)
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)

    return records

# ── Intraday path measurement ──────────────────────────────────────────────────

def measure_intraday(rec, intraday_idx):
    """
    For a decision, compute per-horizon measurements.
    Returns dict: horizon_label → measurement dict, or empty dict if no data.
    """
    ticker = rec["ticker"]
    snap_unix = rec.get("source_snapshot_unix")
    ref_price = rec.get("reference_price")
    direction = rec.get("direction")
    tgt = rec.get("adaptive_target")
    rsk = rec.get("adaptive_risk")

    if not snap_unix or not ref_price or ticker not in intraday_idx:
        return {}

    bars = intraday_idx[ticker]

    # First bar strictly after snapshot
    start_idx = None
    for i, b in enumerate(bars):
        if b["timestamp"] > snap_unix:
            start_idx = i
            break

    if start_idx is None:
        return {}

    entry_price = bars[start_idx]["close"]
    if not entry_price or entry_price == 0:
        return {}

    is_short = (direction == "SHORT")

    # Early return at H15 (first 3 bars) — computed once, attached to all horizons
    h15_end = start_idx + 2  # 3 bars: start_idx, start_idx+1, start_idx+2
    early_return_h15 = None
    if h15_end < len(bars) and bars[h15_end]["close"]:
        h15_price = bars[h15_end]["close"]
        if is_short:
            early_return_h15 = (entry_price - h15_price) / entry_price
        else:
            early_return_h15 = (h15_price - entry_price) / entry_price

    results = {}
    for label, n_bars in HORIZONS.items():
        end_idx = start_idx + n_bars - 1
        if end_idx >= len(bars):
            continue

        exit_price = bars[end_idx]["close"]
        if not exit_price:
            continue

        # Direction-adjusted terminal return
        if is_short:
            ret = (entry_price - exit_price) / entry_price
        else:
            ret = (exit_price - entry_price) / entry_price

        # Path prices over horizon
        path_prices = []
        for i in range(n_bars):
            idx = start_idx + i
            if idx < len(bars) and bars[idx]["close"]:
                path_prices.append(bars[idx]["close"])

        # MFE / MAE (direction-adjusted)
        mfe = None
        mae = None
        time_to_mfe = None
        time_to_mae = None
        if path_prices:
            if is_short:
                excursions = [(entry_price - p) / entry_price for p in path_prices]
            else:
                excursions = [(p - entry_price) / entry_price for p in path_prices]
            mfe = max(excursions)
            mae = min(excursions)
            time_to_mfe = excursions.index(mfe)
            time_to_mae = excursions.index(mae)

        # Target / risk attainment within horizon
        target_attained = False
        risk_attained = False
        if tgt and rsk and path_prices:
            for p in path_prices:
                if is_short:
                    if p <= tgt:
                        target_attained = True
                    if p >= rsk:
                        risk_attained = True
                else:
                    if p >= tgt:
                        target_attained = True
                    if p <= rsk:
                        risk_attained = True

        results[label] = {
            "intraday_return": ret,
            "mfe": mfe,
            "mae": mae,
            "time_to_mfe": time_to_mfe,
            "time_to_mae": time_to_mae,
            "early_return_h15": early_return_h15,
            "target_attained": target_attained,
            "risk_attained": risk_attained,
            "entry_price": entry_price,
            "exit_price": exit_price,
            "n_bars_available": min(n_bars, len(bars) - start_idx),
        }

    return results

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("Loading COMPLETE decisions...")
    all_records = load_complete_decisions()
    print(f"  Total: {len(all_records)}")

    print("Loading 5m intraday data...")
    intraday_idx = load_intraday_index()
    print(f"  Tickers loaded: {len(intraday_idx)}")

    print("Computing intraday measurements...")
    flat_rows = []
    matched = 0
    unmatched = 0

    for rec in all_records:
        horizon_data = measure_intraday(rec, intraday_idx)
        if not horizon_data:
            unmatched += 1
            continue
        matched += 1

        for horizon, hdata in horizon_data.items():
            row = {
                # Decision-time fields
                "decision_id": rec["decision_id"],
                "ticker": rec["ticker"],
                "direction": rec["direction"],
                "action": rec["action"],
                "cohort_date": rec["cohort_date"],
                "source_snapshot_unix": rec["source_snapshot_unix"],
                "certification_status": rec["certification_status"],
                "target_rate": rec["target_rate"],
                "rank_score": rec["rank_score"],
                "sample_size": rec["sample_size"],
                "evidence_class": rec["evidence_class"],
                "degradation_level": rec["degradation_level"],
                "vol_regime": rec["vol_regime"],
                "volume_regime": rec["volume_regime"],
                "reference_price": rec["reference_price"],
                "adaptive_target": rec["adaptive_target"],
                "adaptive_risk": rec["adaptive_risk"],
                "adaptive_horizon_sessions": rec["adaptive_horizon_sessions"],
                "upside_pct": rec["upside_pct"],
                "downside_pct": rec["downside_pct"],
                "rr_ratio": rec["rr_ratio"],
                "expected_return": rec["expected_return"],
                # Horizon
                "horizon": horizon,
                "n_bars": HORIZONS[horizon],
                # Intraday outcomes
                "intraday_return": hdata["intraday_return"],
                "mfe": hdata["mfe"],
                "mae": hdata["mae"],
                "time_to_mfe": hdata["time_to_mfe"],
                "time_to_mae": hdata["time_to_mae"],
                "early_return_h15": hdata["early_return_h15"],
                "target_attained": hdata["target_attained"],
                "risk_attained": hdata["risk_attained"],
                "entry_price": hdata["entry_price"],
                "exit_price": hdata["exit_price"],
                "n_bars_available": hdata["n_bars_available"],
                # Daily outcome (comparison only)
                "daily_return": rec["_daily_return"],
                "daily_target_reached": rec["_daily_target_reached"],
                "daily_exit_reason": rec["_daily_exit_reason"],
            }
            flat_rows.append(row)

    print(f"  Decisions matched to intraday: {matched}, unmatched: {unmatched}")
    print(f"  Total flat rows: {len(flat_rows)}")

    # Write CSV
    OUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    if flat_rows:
        fieldnames = list(flat_rows[0].keys())
        with open(OUT_CSV, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(flat_rows)
        print(f"  CSV written: {OUT_CSV}")

    # Write JSON
    with open(OUT_JSON, "w") as f:
        json.dump(flat_rows, f, indent=2, default=str)
    print(f"  JSON written: {OUT_JSON}")

    # Quick summary
    print()
    print("=== QUICK SUMMARY ===")
    for direction in ["SHORT", "LONG"]:
        for action in ["Watch"]:
            subset = [r for r in flat_rows
                      if r["direction"] == direction and r["action"] == action
                      and r["horizon"] == "H300"]
            if not subset:
                continue
            rets = [r["intraday_return"] for r in subset]
            mfes = [r["mfe"] for r in subset if r["mfe"] is not None]
            maes = [r["mae"] for r in subset if r["mae"] is not None]
            wins = [v for v in rets if v > 0]
            print(f"{direction} {action} H300: N={len(subset)}, "
                  f"median={statistics.median(rets)*100:+.3f}%, "
                  f"win={len(wins)/len(subset):.1%}, "
                  f"MFE_med={statistics.median(mfes)*100:+.3f}%, "
                  f"MAE_med={statistics.median(maes)*100:+.3f}%")

if __name__ == "__main__":
    run()