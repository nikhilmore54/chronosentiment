#!/usr/bin/env python3
"""
HDV-001-D Decision Path Metrics
=================================
Reads the frozen hdv001_price_paths_v1.json and computes, for every
decision, direction-normalized MAE/MFE at sessions 1, 2, 3, 5, 10
plus timing fields.

Direction normalization:
  LONG:  favorable = price rises above reference_price
         adverse   = price falls below reference_price
         MFE_n = max(close - reference_price) / reference_price  over sessions 1..n
         MAE_n = min(close - reference_price) / reference_price  over sessions 1..n
         (MFE >= 0 means price moved in Coralys direction)

  SHORT: favorable = price falls below reference_price
         adverse   = price rises above reference_price
         MFE_n = max(reference_price - close) / reference_price  over sessions 1..n
         MAE_n = min(reference_price - close) / reference_price  over sessions 1..n
         (MFE >= 0 means price moved in Coralys direction)

So positive MFE always means "price moved in the direction Coralys predicted."
Positive MAE means "price moved favorably at some point" (same sign convention).
Negative MAE means "price moved adversely."

Timing fields (session number, 1-indexed, or null if not reached):
  time_to_mae          -- session at which the running MAE was most adverse
  time_to_mfe          -- session at which the running MFE was most favorable
  time_to_target       -- first session where close crosses target_price in direction
  time_to_stop         -- first session where close crosses stop_price adversely
  time_to_entry_recovery -- for decisions that went adverse first, session where
                            close returns to reference_price

Output:
  datasets/hdv001/hdv001_decision_metrics_v1.json
  datasets/hdv001/HDV_001_D_METRICS_REPORT.md
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# ── paths ─────────────────────────────────────────────────────────────────────
WORKSPACE    = Path(__file__).resolve().parent.parent
PATHS_FILE   = WORKSPACE / "datasets" / "hdv001" / "hdv001_price_paths_v1.json"
DATASET_FILE = WORKSPACE / "datasets" / "stop_research_dataset_v01.json"
OUTPUT_FILE  = WORKSPACE / "datasets" / "hdv001" / "hdv001_decision_metrics_v1.json"
REPORT_FILE  = WORKSPACE / "datasets" / "hdv001" / "HDV_001_D_METRICS_REPORT.md"

# ── session checkpoints ───────────────────────────────────────────────────────
CHECKPOINTS = [1, 2, 3, 5, 10]

# ─────────────────────────────────────────────────────────────────────────────

def compute_metrics(path_record: dict, meta: dict) -> dict:
    """
    Compute direction-normalized MAE/MFE and timing fields for one decision.
    Returns a flat metrics dict.
    """
    direction     = path_record["direction"]
    ref_price     = path_record["reference_price"]
    target_price  = path_record["target_price"]
    stop_price    = path_record["stop_price"]
    sessions      = path_record["sessions"]
    obs_status    = path_record["observation_status"]
    n_sessions    = len(sessions)

    # direction multiplier: +1 for LONG, -1 for SHORT
    # favorable_return = multiplier * (close - ref_price) / ref_price
    mult = 1.0 if direction == "LONG" else -1.0

    # ── per-session returns (direction-normalized) ────────────────────────────
    returns = []
    for s in sessions:
        ret = mult * (s["close"] - ref_price) / ref_price
        returns.append(ret)

    # ── MAE/MFE at each checkpoint ────────────────────────────────────────────
    # MFE_n = max favorable return seen in sessions 1..n
    # MAE_n = min return seen in sessions 1..n (most adverse; negative = adverse)
    checkpoint_metrics = {}
    for cp in CHECKPOINTS:
        window = returns[:cp]
        if not window:
            checkpoint_metrics[f"mfe_{cp}"] = None
            checkpoint_metrics[f"mae_{cp}"] = None
        else:
            checkpoint_metrics[f"mfe_{cp}"] = round(max(window), 8)
            checkpoint_metrics[f"mae_{cp}"] = round(min(window), 8)

    # ── timing: session of worst adverse and best favorable ───────────────────
    if returns:
        mae_val = min(returns)
        mfe_val = max(returns)
        time_to_mae = returns.index(mae_val) + 1   # 1-indexed
        time_to_mfe = returns.index(mfe_val) + 1
    else:
        mae_val = None
        mfe_val = None
        time_to_mae = None
        time_to_mfe = None

    # ── time_to_target: first session where close crosses target ──────────────
    time_to_target = None
    for i, s in enumerate(sessions):
        if direction == "LONG" and s["close"] >= target_price:
            time_to_target = i + 1
            break
        elif direction == "SHORT" and s["close"] <= target_price:
            time_to_target = i + 1
            break

    # ── time_to_stop: first session where close crosses stop adversely ────────
    time_to_stop = None
    for i, s in enumerate(sessions):
        if direction == "LONG" and s["close"] <= stop_price:
            time_to_stop = i + 1
            break
        elif direction == "SHORT" and s["close"] >= stop_price:
            time_to_stop = i + 1
            break

    # ── time_to_entry_recovery ────────────────────────────────────────────────
    # Only meaningful if the decision went adverse at some point.
    # Find first session where close returns to >= ref_price (LONG) or
    # <= ref_price (SHORT) after having gone adverse.
    time_to_entry_recovery = None
    went_adverse = False
    for i, (s, r) in enumerate(zip(sessions, returns)):
        if r < 0:
            went_adverse = True
        if went_adverse:
            if direction == "LONG" and s["close"] >= ref_price:
                time_to_entry_recovery = i + 1
                break
            elif direction == "SHORT" and s["close"] <= ref_price:
                time_to_entry_recovery = i + 1
                break

    # ── coralys state variables from meta ─────────────────────────────────────
    trend      = meta.get("coralys_trend")
    momentum   = meta.get("coralys_momentum")
    volatility = meta.get("coralys_volatility")

    return {
        "decision_id":           path_record["decision_id"],
        "instrument":            path_record["instrument"],
        "direction":             direction,
        "decision_time":         path_record["decision_time"],
        "decision_date_ist":     path_record["decision_date_ist"],
        "reference_price":       ref_price,
        "target_price":          target_price,
        "stop_price":            stop_price,
        "coralys_trend":         trend,
        "coralys_momentum":      momentum,
        "coralys_volatility":    volatility,
        "observation_status":    obs_status,
        "sessions_available":    n_sessions,
        # MAE/MFE at checkpoints (direction-normalized, fractional)
        **checkpoint_metrics,
        # timing
        "time_to_mae":           time_to_mae,
        "time_to_mfe":           time_to_mfe,
        "time_to_target":        time_to_target,
        "time_to_stop":          time_to_stop,
        "time_to_entry_recovery": time_to_entry_recovery,
        # final session values (for convenience)
        "final_return":          round(returns[-1], 8) if returns else None,
        "final_mfe":             round(mfe_val, 8) if mfe_val is not None else None,
        "final_mae":             round(mae_val, 8) if mae_val is not None else None,
    }


def main():
    print("=" * 70)
    print("HDV-001-D DECISION PATH METRICS")
    print("=" * 70)

    # load price paths
    with open(PATHS_FILE) as f:
        paths_data = json.load(f)
    paths = {p["decision_id"]: p for p in paths_data["paths"]}
    print(f"Loaded {len(paths)} price paths")

    # load decision metadata (for Coralys state variables)
    with open(DATASET_FILE) as f:
        dataset = json.load(f)
    meta_map = {r["decision_id"]: r for r in dataset["records"]}
    print(f"Loaded {len(meta_map)} decision metadata records")

    # compute metrics
    metrics_list = []
    stats = {
        "total": 0,
        "complete": 0,
        "maturing": 0,
        "no_sessions": 0,
    }

    for decision_id, path_rec in paths.items():
        stats["total"] += 1
        meta = meta_map.get(decision_id, {})
        m = compute_metrics(path_rec, meta)
        metrics_list.append(m)

        if path_rec["observation_status"] == "COMPLETE":
            stats["complete"] += 1
        elif path_rec["observation_status"] == "MATURING":
            stats["maturing"] += 1
        if path_rec["sessions_available"] == 0:
            stats["no_sessions"] += 1

    # ── aggregate summary (COMPLETE decisions only) ───────────────────────────
    complete = [m for m in metrics_list if m["observation_status"] == "COMPLETE"]

    def median(vals):
        v = sorted(x for x in vals if x is not None)
        if not v:
            return None
        n = len(v)
        return round(v[n // 2] if n % 2 else (v[n // 2 - 1] + v[n // 2]) / 2, 6)

    def pct_positive(vals):
        v = [x for x in vals if x is not None]
        if not v:
            return None
        return round(sum(1 for x in v if x > 0) / len(v), 4)

    summary = {}
    for cp in CHECKPOINTS:
        mfe_vals = [m[f"mfe_{cp}"] for m in complete]
        mae_vals = [m[f"mae_{cp}"] for m in complete]
        summary[f"median_mfe_{cp}"] = median(mfe_vals)
        summary[f"median_mae_{cp}"] = median(mae_vals)
        summary[f"pct_positive_mfe_{cp}"] = pct_positive(mfe_vals)

    # target/stop hit rates
    n_complete = len(complete)
    n_target = sum(1 for m in complete if m["time_to_target"] is not None)
    n_stop   = sum(1 for m in complete if m["time_to_stop"] is not None)
    summary["n_complete"]       = n_complete
    summary["n_target_hit"]     = n_target
    summary["n_stop_hit"]       = n_stop
    summary["pct_target_hit"]   = round(n_target / n_complete, 4) if n_complete else None
    summary["pct_stop_hit"]     = round(n_stop   / n_complete, 4) if n_complete else None

    # ── write output ──────────────────────────────────────────────────────────
    output = {
        "version":    "hdv001_decision_metrics_v1",
        "built_at":   datetime.now(timezone.utc).isoformat(),
        "source":     "hdv001_price_paths_v1.json",
        "n_decisions": len(metrics_list),
        "stats":      stats,
        "summary_complete_decisions": summary,
        "metrics":    metrics_list,
    }
    with open(OUTPUT_FILE, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nWrote {len(metrics_list)} decision metrics to {OUTPUT_FILE.relative_to(WORKSPACE)}")

    # ── print summary ─────────────────────────────────────────────────────────
    print(f"\nStats:")
    print(f"  Total decisions : {stats['total']}")
    print(f"  COMPLETE        : {stats['complete']}")
    print(f"  MATURING        : {stats['maturing']}")
    print(f"  No sessions     : {stats['no_sessions']}")

    print(f"\nAggregate summary (COMPLETE decisions, N={n_complete}):")
    print(f"  {'Session':<10} {'Median MFE':>12} {'Median MAE':>12} {'% MFE>0':>10}")
    print(f"  {'-'*46}")
    for cp in CHECKPOINTS:
        mfe = summary.get(f'median_mfe_{cp}')
        mae = summary.get(f'median_mae_{cp}')
        pct = summary.get(f'pct_positive_mfe_{cp}')
        mfe_s = f"{mfe*100:+.3f}%" if mfe is not None else "N/A"
        mae_s = f"{mae*100:+.3f}%" if mae is not None else "N/A"
        pct_s = f"{pct*100:.1f}%" if pct is not None else "N/A"
        print(f"  Session {cp:<3}    {mfe_s:>12} {mae_s:>12} {pct_s:>10}")

    print(f"\n  Target hit rate : {summary['pct_target_hit']*100:.1f}%  ({n_target}/{n_complete})")
    print(f"  Stop hit rate   : {summary['pct_stop_hit']*100:.1f}%  ({n_stop}/{n_complete})")

    # ── write report ──────────────────────────────────────────────────────────
    report_lines = [
        "# HDV-001-D Decision Path Metrics Report",
        "",
        f"**Generated:** 2026-08-17",
        f"**Source:** `datasets/hdv001/hdv001_price_paths_v1.json`",
        f"**Output:** `datasets/hdv001/hdv001_decision_metrics_v1.json`",
        "",
        "## Direction Normalization",
        "",
        "Positive MFE/MAE = price moved in Coralys predicted direction.",
        "Negative MFE/MAE = price moved against Coralys predicted direction.",
        "",
        "LONG:  favorable_return = (close - reference_price) / reference_price",
        "SHORT: favorable_return = (reference_price - close) / reference_price",
        "",
        "## Statistics",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Total decisions | {stats['total']} |",
        f"| COMPLETE | {stats['complete']} |",
        f"| MATURING | {stats['maturing']} |",
        f"| No sessions | {stats['no_sessions']} |",
        "",
        "## Aggregate Summary (COMPLETE decisions only)",
        "",
        f"N = {n_complete}",
        "",
        "| Session | Median MFE | Median MAE | % MFE > 0 |",
        "|---------|-----------|-----------|-----------|",
    ]
    for cp in CHECKPOINTS:
        mfe = summary.get(f'median_mfe_{cp}')
        mae = summary.get(f'median_mae_{cp}')
        pct = summary.get(f'pct_positive_mfe_{cp}')
        mfe_s = f"{mfe*100:+.3f}%" if mfe is not None else "N/A"
        mae_s = f"{mae*100:+.3f}%" if mae is not None else "N/A"
        pct_s = f"{pct*100:.1f}%" if pct is not None else "N/A"
        report_lines.append(f"| {cp} | {mfe_s} | {mae_s} | {pct_s} |")

    report_lines += [
        "",
        "## Target and Stop Hit Rates (COMPLETE decisions)",
        "",
        f"| Metric | Count | Rate |",
        f"|--------|-------|------|",
        f"| Target hit within 10 sessions | {n_target} | {summary['pct_target_hit']*100:.1f}% |",
        f"| Stop hit within 10 sessions | {n_stop} | {summary['pct_stop_hit']*100:.1f}% |",
        "",
        "## Notes",
        "",
        "Primary analysis uses COMPLETE decisions (>= 10 sessions observed).",
        "MATURING decisions are included in the output file but excluded from",
        "aggregate summary statistics.",
        "",
        "Do not modify C3-002 based on these findings.",
        "HDV-001-G freeze gate must be passed before any implementation changes.",
    ]
    REPORT_FILE.write_text("\n".join(report_lines))
    print(f"\nReport written to: {REPORT_FILE.relative_to(WORKSPACE)}")
    print("\nHDV-001-D: COMPLETE.")
    sys.exit(0)


if __name__ == "__main__":
    main()