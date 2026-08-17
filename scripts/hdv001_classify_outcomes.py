#!/usr/bin/env python3
"""
HDV-001-E Outcome Classifier
==============================
Reads hdv001_decision_metrics_v1.json and classifies each COMPLETE
decision into one of four outcome categories, preserving ordering.

Classification rules (applied in order):
  1. TARGET_BEFORE_RISK  -- target hit before stop (or target hit, stop never hit)
  2. RISK_BEFORE_TARGET  -- stop hit before target (or stop hit, target never hit)
  3. HORIZON             -- neither target nor stop hit within 10 sessions (COMPLETE)
  4. MATURING            -- observation window not yet complete
  5. NO_SESSIONS         -- no price data available

The ordering is critical:
  If time_to_target < time_to_stop  -> TARGET_BEFORE_RISK
  If time_to_stop < time_to_target  -> RISK_BEFORE_TARGET
  If time_to_target == time_to_stop -> TARGET_BEFORE_RISK (target takes precedence)
  If only target hit                -> TARGET_BEFORE_RISK
  If only stop hit                  -> RISK_BEFORE_TARGET
  If neither hit (COMPLETE)         -> HORIZON

Output:
  datasets/hdv001/hdv001_outcomes_v1.json
  datasets/hdv001/HDV_001_E_OUTCOMES_REPORT.md
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# ── paths ─────────────────────────────────────────────────────────────────────
WORKSPACE     = Path(__file__).resolve().parent.parent
METRICS_FILE  = WORKSPACE / "datasets" / "hdv001" / "hdv001_decision_metrics_v1.json"
OUTPUT_FILE   = WORKSPACE / "datasets" / "hdv001" / "hdv001_outcomes_v1.json"
REPORT_FILE   = WORKSPACE / "datasets" / "hdv001" / "HDV_001_E_OUTCOMES_REPORT.md"


def classify_outcome(m: dict) -> str:
    obs_status   = m["observation_status"]
    n_sessions   = m["sessions_available"]
    time_target  = m["time_to_target"]
    time_stop    = m["time_to_stop"]

    if n_sessions == 0:
        return "NO_SESSIONS"
    if obs_status == "MATURING":
        return "MATURING"

    # COMPLETE decision
    if time_target is not None and time_stop is not None:
        if time_target <= time_stop:
            return "TARGET_BEFORE_RISK"
        else:
            return "RISK_BEFORE_TARGET"
    elif time_target is not None:
        return "TARGET_BEFORE_RISK"
    elif time_stop is not None:
        return "RISK_BEFORE_TARGET"
    else:
        return "HORIZON"


def main():
    print("=" * 70)
    print("HDV-001-E OUTCOME CLASSIFIER")
    print("=" * 70)

    with open(METRICS_FILE) as f:
        metrics_data = json.load(f)
    metrics = metrics_data["metrics"]
    print(f"Loaded {len(metrics)} decision metrics")

    outcomes = []
    counts = {
        "TARGET_BEFORE_RISK": 0,
        "RISK_BEFORE_TARGET": 0,
        "HORIZON":            0,
        "MATURING":           0,
        "NO_SESSIONS":        0,
    }

    for m in metrics:
        outcome = classify_outcome(m)
        counts[outcome] += 1
        outcomes.append({
            "decision_id":       m["decision_id"],
            "instrument":        m["instrument"],
            "direction":         m["direction"],
            "decision_time":     m["decision_time"],
            "decision_date_ist": m["decision_date_ist"],
            "coralys_trend":     m["coralys_trend"],
            "coralys_momentum":  m["coralys_momentum"],
            "coralys_volatility":m["coralys_volatility"],
            "observation_status":m["observation_status"],
            "sessions_available":m["sessions_available"],
            "outcome":           outcome,
            "time_to_target":    m["time_to_target"],
            "time_to_stop":      m["time_to_stop"],
            "final_return":      m["final_return"],
            "mfe_10":            m.get("mfe_10"),
            "mae_10":            m.get("mae_10"),
        })

    n_complete = counts["TARGET_BEFORE_RISK"] + counts["RISK_BEFORE_TARGET"] + counts["HORIZON"]

    # ── segmentation by Coralys state (COMPLETE only) ─────────────────────────
    complete_outcomes = [o for o in outcomes if o["observation_status"] == "COMPLETE"]

    def segment_rates(subset):
        n = len(subset)
        if n == 0:
            return {"n": 0, "target_pct": None, "risk_pct": None, "horizon_pct": None}
        return {
            "n": n,
            "target_pct": round(sum(1 for o in subset if o["outcome"] == "TARGET_BEFORE_RISK") / n, 4),
            "risk_pct":   round(sum(1 for o in subset if o["outcome"] == "RISK_BEFORE_TARGET") / n, 4),
            "horizon_pct":round(sum(1 for o in subset if o["outcome"] == "HORIZON") / n, 4),
        }

    # by direction
    seg_long  = segment_rates([o for o in complete_outcomes if o["direction"] == "LONG"])
    seg_short = segment_rates([o for o in complete_outcomes if o["direction"] == "SHORT"])

    # by trend
    seg_bullish = segment_rates([o for o in complete_outcomes if o["coralys_trend"] == "Bullish"])
    seg_bearish = segment_rates([o for o in complete_outcomes if o["coralys_trend"] == "Bearish"])

    # by momentum
    seg_pos_mom = segment_rates([o for o in complete_outcomes if o["coralys_momentum"] == "Positive"])
    seg_neg_mom = segment_rates([o for o in complete_outcomes if o["coralys_momentum"] == "Negative"])

    # by trend+momentum combination
    combos = {}
    for trend in ["Bullish", "Bearish"]:
        for mom in ["Positive", "Negative"]:
            key = f"{trend}_{mom}"
            subset = [o for o in complete_outcomes
                      if o["coralys_trend"] == trend and o["coralys_momentum"] == mom]
            combos[key] = segment_rates(subset)

    segmentation = {
        "by_direction": {"LONG": seg_long, "SHORT": seg_short},
        "by_trend":     {"Bullish": seg_bullish, "Bearish": seg_bearish},
        "by_momentum":  {"Positive": seg_pos_mom, "Negative": seg_neg_mom},
        "by_trend_momentum": combos,
    }

    # ── write output ──────────────────────────────────────────────────────────
    output = {
        "version":    "hdv001_outcomes_v1",
        "built_at":   datetime.now(timezone.utc).isoformat(),
        "source":     "hdv001_decision_metrics_v1.json",
        "n_decisions": len(outcomes),
        "counts":     counts,
        "n_complete": n_complete,
        "segmentation": segmentation,
        "outcomes":   outcomes,
    }
    with open(OUTPUT_FILE, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nWrote {len(outcomes)} outcomes to {OUTPUT_FILE.relative_to(WORKSPACE)}")

    # ── print summary ─────────────────────────────────────────────────────────
    print(f"\nOutcome counts (all {len(outcomes)} decisions):")
    for k, v in counts.items():
        print(f"  {k:<22}: {v}")

    print(f"\nOutcome rates (COMPLETE decisions, N={n_complete}):")
    tbr = counts["TARGET_BEFORE_RISK"]
    rbr = counts["RISK_BEFORE_TARGET"]
    hor = counts["HORIZON"]
    print(f"  TARGET_BEFORE_RISK : {tbr:4d}  ({tbr/n_complete*100:.1f}%)")
    print(f"  RISK_BEFORE_TARGET : {rbr:4d}  ({rbr/n_complete*100:.1f}%)")
    print(f"  HORIZON            : {hor:4d}  ({hor/n_complete*100:.1f}%)")

    print(f"\nSegmentation by direction (COMPLETE):")
    for d, s in [("LONG", seg_long), ("SHORT", seg_short)]:
        if s["n"] > 0:
            print(f"  {d:<6} N={s['n']:4d}  TARGET={s['target_pct']*100:.1f}%  RISK={s['risk_pct']*100:.1f}%  HORIZON={s['horizon_pct']*100:.1f}%")

    print(f"\nSegmentation by trend (COMPLETE):")
    for t, s in [("Bullish", seg_bullish), ("Bearish", seg_bearish)]:
        if s["n"] > 0:
            print(f"  {t:<8} N={s['n']:4d}  TARGET={s['target_pct']*100:.1f}%  RISK={s['risk_pct']*100:.1f}%  HORIZON={s['horizon_pct']*100:.1f}%")

    print(f"\nSegmentation by trend+momentum (COMPLETE):")
    for key, s in combos.items():
        if s["n"] > 0:
            print(f"  {key:<20} N={s['n']:4d}  TARGET={s['target_pct']*100:.1f}%  RISK={s['risk_pct']*100:.1f}%  HORIZON={s['horizon_pct']*100:.1f}%")

    # ── write report ──────────────────────────────────────────────────────────
    report_lines = [
        "# HDV-001-E Outcome Classification Report",
        "",
        f"**Generated:** 2026-08-17",
        f"**Source:** `datasets/hdv001/hdv001_decision_metrics_v1.json`",
        f"**Output:** `datasets/hdv001/hdv001_outcomes_v1.json`",
        "",
        "## Classification Rules",
        "",
        "Applied in order:",
        "1. TARGET_BEFORE_RISK -- target hit before or at same session as stop",
        "2. RISK_BEFORE_TARGET -- stop hit before target",
        "3. HORIZON            -- neither hit within 10 sessions (COMPLETE only)",
        "4. MATURING           -- observation window not yet complete",
        "",
        "## Outcome Counts",
        "",
        "| Outcome | Count |",
        "|---------|-------|",
    ]
    for k, v in counts.items():
        report_lines.append(f"| {k} | {v} |")

    report_lines += [
        "",
        f"## Outcome Rates (COMPLETE decisions, N={n_complete})",
        "",
        "| Outcome | Count | Rate |",
        "|---------|-------|------|",
        f"| TARGET_BEFORE_RISK | {tbr} | {tbr/n_complete*100:.1f}% |",
        f"| RISK_BEFORE_TARGET | {rbr} | {rbr/n_complete*100:.1f}% |",
        f"| HORIZON | {hor} | {hor/n_complete*100:.1f}% |",
        "",
        "## Segmentation by Direction",
        "",
        "| Direction | N | TARGET | RISK | HORIZON |",
        "|-----------|---|--------|------|---------|",
    ]
    for d, s in [("LONG", seg_long), ("SHORT", seg_short)]:
        if s["n"] > 0:
            report_lines.append(
                f"| {d} | {s['n']} | {s['target_pct']*100:.1f}% | {s['risk_pct']*100:.1f}% | {s['horizon_pct']*100:.1f}% |"
            )

    report_lines += [
        "",
        "## Segmentation by Coralys Trend",
        "",
        "| Trend | N | TARGET | RISK | HORIZON |",
        "|-------|---|--------|------|---------|",
    ]
    for t, s in [("Bullish", seg_bullish), ("Bearish", seg_bearish)]:
        if s["n"] > 0:
            report_lines.append(
                f"| {t} | {s['n']} | {s['target_pct']*100:.1f}% | {s['risk_pct']*100:.1f}% | {s['horizon_pct']*100:.1f}% |"
            )

    report_lines += [
        "",
        "## Segmentation by Trend + Momentum",
        "",
        "| Trend + Momentum | N | TARGET | RISK | HORIZON |",
        "|------------------|---|--------|------|---------|",
    ]
    for key, s in combos.items():
        if s["n"] > 0:
            report_lines.append(
                f"| {key} | {s['n']} | {s['target_pct']*100:.1f}% | {s['risk_pct']*100:.1f}% | {s['horizon_pct']*100:.1f}% |"
            )

    report_lines += [
        "",
        "## Governance Note",
        "",
        "Do not modify C3-002 based on these findings.",
        "HDV-001-G freeze gate must be passed before any implementation changes.",
        "Primary analysis uses COMPLETE decisions only.",
        "MATURING decisions will be reclassified when their observation windows complete.",
    ]
    REPORT_FILE.write_text("\n".join(report_lines))
    print(f"\nReport written to: {REPORT_FILE.relative_to(WORKSPACE)}")
    print("\nHDV-001-E: COMPLETE.")
    sys.exit(0)


if __name__ == "__main__":
    main()