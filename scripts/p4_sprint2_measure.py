#!/usr/bin/env python3
"""
P4 Sprint 2 — Measure
=====================
Decompose the SHORT advantage discovered in Sprint 1.

Question: Within SHORT decisions, what observable characteristics
systematically separate stronger from weaker outcomes?

Also investigates:
- rank_score anomaly (bottom 20% beats top 20%)
- direction × quality variable interactions
- distributional statistics (median, percentiles) alongside mean

Inputs:  live_capture/ledger/entries/*.json
         time_machine/analysis/TIME009/observations/*.json
Outputs: docs/P4_SPRINT2_MEASURE_RESULTS.md

Guiding principle: Explore aggressively. Validate ruthlessly.
"""

import json
import sys
from pathlib import Path
import statistics

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
OUT_DOC = REPO_ROOT / "docs" / "P4_SPRINT2_MEASURE_RESULTS.md"

# ── Data loading (identical to Sprint 1) ──────────────────────────────────────

def load_json(path):
    with open(path) as f:
        return json.load(f)

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
                "ticker": e.get("ticker"),
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
                "_realized_return": obs.get("realized_return"),
                "_target_reached": obs.get("target_reached"),
                "_risk_reached": obs.get("risk_reached"),
                "_horizon_reached": obs.get("horizon_reached"),
                "_actual_mfe": obs.get("actual_mfe"),
                "_actual_mae": obs.get("actual_mae"),
                "_eligible": obs.get("eligible_for_primary_comparison", False),
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
            rec["is_approximate"] = rec["degradation_level"] == "Approximate"
            rec["is_certified"] = rec["certification_status"] == "CERTIFIED"

            records.append(rec)
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)

    return records

# ── Statistics ─────────────────────────────────────────────────────────────────

def pct(v, decimals=1):
    if v is None:
        return "—"
    return f"{v*100:+.{decimals}f}%"

def pct_plain(v, decimals=1):
    if v is None:
        return "—"
    return f"{v*100:.{decimals}f}%"

def stats(returns):
    if not returns:
        return {}
    n = len(returns)
    s = sorted(returns)
    wins = [r for r in returns if r > 0]
    losses = [r for r in returns if r < 0]
    flat = [r for r in returns if r == 0]

    def percentile(data, p):
        if not data:
            return None
        idx = (len(data) - 1) * p / 100
        lo, hi = int(idx), min(int(idx) + 1, len(data) - 1)
        return data[lo] + (data[hi] - data[lo]) * (idx - lo)

    return {
        "n": n,
        "mean": statistics.mean(returns),
        "median": statistics.median(returns),
        "p25": percentile(s, 25),
        "p75": percentile(s, 75),
        "p10": percentile(s, 10),
        "p90": percentile(s, 90),
        "win_rate": len(wins) / n,
        "loss_rate": len(losses) / n,
        "flat_rate": len(flat) / n,
        "mean_win": statistics.mean(wins) if wins else None,
        "mean_loss": statistics.mean(losses) if losses else None,
        "min": min(returns),
        "max": max(returns),
        "stdev": statistics.stdev(returns) if n > 1 else 0.0,
        "target_hit": None,  # filled separately if available
    }

def full_row(label, recs):
    returns = [r["_realized_return"] for r in recs]
    target_hits = [r["_target_reached"] for r in recs if r["_target_reached"] is not None]
    risk_hits = [r["_risk_reached"] for r in recs if r["_risk_reached"] is not None]
    s = stats(returns)
    if not s:
        return f"| {label} | 0 | — | — | — | — | — | — | — | — | — |"
    n = s["n"]
    th = f"{sum(target_hits)/len(target_hits)*100:.1f}%" if target_hits else "—"
    rh = f"{sum(risk_hits)/len(risk_hits)*100:.1f}%" if risk_hits else "—"
    return (f"| {label} | {n} | {pct(s['mean'])} | {pct(s['median'])} | "
            f"{pct(s['p25'])} | {pct(s['p75'])} | "
            f"{pct_plain(s['win_rate'])} | {pct_plain(s['loss_rate'])} | "
            f"{pct(s['stdev'])} | {th} | {rh} |")

def short_row(label, recs):
    returns = [r["_realized_return"] for r in recs]
    s = stats(returns)
    if not s:
        return f"| {label} | 0 | — | — | — | — | — |"
    return (f"| {label} | {s['n']} | {pct(s['mean'])} | {pct(s['median'])} | "
            f"{pct_plain(s['win_rate'])} | {pct_plain(s['loss_rate'])} | {pct(s['stdev'])} |")

# ── Rank score quartile analysis ───────────────────────────────────────────────

def quartile_analysis(recs, rank_field, label):
    valid = [r for r in recs if r.get(rank_field) is not None]
    if not valid:
        return []
    valid.sort(key=lambda r: r[rank_field])
    n = len(valid)
    q1 = valid[:n//4]
    q2 = valid[n//4:n//2]
    q3 = valid[n//2:3*n//4]
    q4 = valid[3*n//4:]
    rows = []
    for qname, qrecs in [("Q1 (lowest)", q1), ("Q2", q2), ("Q3", q3), ("Q4 (highest)", q4)]:
        rows.append(short_row(f"{label} {qname}", qrecs))
    return rows

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("Loading COMPLETE decisions...")
    all_records = load_complete_decisions()
    print(f"  Total: {len(all_records)}")

    watch = [r for r in all_records if r["is_watch"]]
    short_watch = [r for r in watch if r["is_short"]]
    long_watch = [r for r in watch if r["is_long"]]

    lines = []
    lines.append("# P4 Sprint 2 — Measure Results")
    lines.append("")
    lines.append("**Date:** 2026-09-11")
    lines.append(f"**Population:** {len(all_records)} COMPLETE decisions")
    lines.append(f"**Watch decisions:** {len(watch)} (LONG: {len(long_watch)}, SHORT: {len(short_watch)})")
    lines.append("")
    lines.append("**Sprint 2 question:** Within SHORT decisions, what observable characteristics")
    lines.append("systematically separate stronger from weaker outcomes?")
    lines.append("")
    lines.append("Column key: N | Mean | Median | P25 | P75 | Win% | Loss% | Stdev | Target hit% | Risk hit%")
    lines.append("")
    lines.append("---")
    lines.append("")

    # ── Section 1: Direction × action full breakdown ───────────────────────────
    lines.append("## 1. Direction × action breakdown (full distributional view)")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | P25 | P75 | Win% | Loss% | Stdev | Target hit% | Risk hit% |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|")
    lines.append(full_row("All COMPLETE", all_records))
    lines.append(full_row("Watch (all)", watch))
    lines.append(full_row("LONG Watch", long_watch))
    lines.append(full_row("SHORT Watch", short_watch))
    lines.append(full_row("NoTrade (all)", [r for r in all_records if not r["is_watch"]]))
    lines.append("")

    # ── Section 2: SHORT Watch decomposition ──────────────────────────────────
    lines.append("---")
    lines.append("")
    lines.append("## 2. SHORT Watch decomposition — degradation level")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for deg in ["Exact", "Approximate", "Insufficient"]:
        recs = [r for r in short_watch if r["degradation_level"] == deg]
        lines.append(short_row(f"SHORT Watch {deg}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 3. SHORT Watch decomposition — sample size bands")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for lo, hi, label in [(1, 25, "1–25"), (26, 50, "26–50"), (51, 100, "51–100"),
                           (101, 150, "101–150"), (151, 999, "151+")]:
        recs = [r for r in short_watch if lo <= r["sample_size"] <= hi]
        lines.append(short_row(f"SHORT Watch sample {label}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 4. SHORT Watch decomposition — target_rate quartiles")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for row in quartile_analysis(short_watch, "target_rate", "SHORT Watch target_rate"):
        lines.append(row)
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 5. SHORT Watch decomposition — rank_score quartiles")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for row in quartile_analysis(short_watch, "rank_score", "SHORT Watch rank_score"):
        lines.append(row)
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 6. SHORT Watch decomposition — vol_regime")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for regime in ["present", "absent"]:
        recs = [r for r in short_watch if r["vol_regime"] == regime]
        lines.append(short_row(f"SHORT Watch vol={regime}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 7. SHORT Watch decomposition — volume_regime")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for regime in ["Normal", "High", "Low"]:
        recs = [r for r in short_watch if r["volume_regime"] == regime]
        lines.append(short_row(f"SHORT Watch volume={regime}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 8. SHORT Watch decomposition — certification_status")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for status in ["CERTIFIED", "DEGRADED"]:
        recs = [r for r in short_watch if r["certification_status"] == status]
        lines.append(short_row(f"SHORT Watch cert={status}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 9. SHORT Watch — cross-tabulation: degradation × sample_size")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for deg in ["Exact", "Approximate"]:
        for lo, hi, slabel in [(1, 50, "1–50"), (51, 150, "51–150"), (151, 999, "151+")]:
            recs = [r for r in short_watch
                    if r["degradation_level"] == deg and lo <= r["sample_size"] <= hi]
            lines.append(short_row(f"SHORT Watch {deg} sample {slabel}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 10. rank_score anomaly investigation — direction interaction")
    lines.append("")
    lines.append("Sprint 1 found: bottom 20% by rank_score (+45.5%) beats top 20% (+4.4%).")
    lines.append("Hypothesis: the anomaly is driven by direction composition, not rank_score itself.")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")

    # rank_score quartiles for LONG Watch
    lines.append("")
    lines.append("**LONG Watch by rank_score quartile:**")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for row in quartile_analysis(long_watch, "rank_score", "LONG Watch rank_score"):
        lines.append(row)

    lines.append("")
    lines.append("**SHORT Watch by rank_score quartile:**")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for row in quartile_analysis(short_watch, "rank_score", "SHORT Watch rank_score"):
        lines.append(row)

    lines.append("")
    lines.append("**All Watch by rank_score quartile (mixed direction):**")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    # Show direction composition within each quartile
    valid = [r for r in watch if r.get("rank_score") is not None]
    valid.sort(key=lambda r: r["rank_score"])
    n = len(valid)
    quartiles = [
        ("Q1 (lowest)", valid[:n//4]),
        ("Q2", valid[n//4:n//2]),
        ("Q3", valid[n//2:3*n//4]),
        ("Q4 (highest)", valid[3*n//4:]),
    ]
    for qname, qrecs in quartiles:
        n_long = sum(1 for r in qrecs if r["is_long"])
        n_short = sum(1 for r in qrecs if r["is_short"])
        base = short_row(f"Watch rank_score {qname}", qrecs)
        # append direction composition
        lines.append(base.rstrip(" |") + f" L:{n_long}/S:{n_short} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 11. LONG Watch decomposition — where does the loss come from?")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for deg in ["Exact", "Approximate", "Insufficient"]:
        recs = [r for r in long_watch if r["degradation_level"] == deg]
        lines.append(short_row(f"LONG Watch {deg}", recs))
    lines.append("")
    for lo, hi, label in [(1, 50, "1–50"), (51, 100, "51–100"), (101, 150, "101–150"), (151, 999, "151+")]:
        recs = [r for r in long_watch if lo <= r["sample_size"] <= hi]
        lines.append(short_row(f"LONG Watch sample {label}", recs))
    lines.append("")
    for regime in ["present", "absent"]:
        recs = [r for r in long_watch if r["vol_regime"] == regime]
        lines.append(short_row(f"LONG Watch vol={regime}", recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 12. Outcome distribution — gap-through analysis")
    lines.append("")
    lines.append("Extreme ±100% values distort means. Identifying gap-through frequency.")
    lines.append("")
    lines.append("| Cohort | N | N with |return|>50% | Gap-through% | Median (excl gaps) | Mean (excl gaps) |")
    lines.append("|---|---|---|---|---|---|")

    for label, recs in [("All Watch", watch), ("LONG Watch", long_watch), ("SHORT Watch", short_watch)]:
        returns = [r["_realized_return"] for r in recs]
        gaps = [r for r in returns if abs(r) > 0.5]
        non_gaps = [r for r in returns if abs(r) <= 0.5]
        gap_pct = f"{len(gaps)/len(returns)*100:.1f}%" if returns else "—"
        med_ng = pct(statistics.median(non_gaps)) if non_gaps else "—"
        mean_ng = pct(statistics.mean(non_gaps)) if non_gaps else "—"
        lines.append(f"| {label} | {len(returns)} | {len(gaps)} | {gap_pct} | {med_ng} | {mean_ng} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 13. SHORT Watch — best observable combination candidates")
    lines.append("")
    lines.append("Combinations of decision-time variables showing strongest separation.")
    lines.append("These are finalist candidates for walk-forward evaluation.")
    lines.append("")
    lines.append("| Combination | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")

    combos = [
        ("SHORT Watch Exact", [r for r in short_watch if r["is_exact"]]),
        ("SHORT Watch Exact sample>=50", [r for r in short_watch if r["is_exact"] and r["sample_size"] >= 50]),
        ("SHORT Watch Exact sample>=100", [r for r in short_watch if r["is_exact"] and r["sample_size"] >= 100]),
        ("SHORT Watch sample>=50", [r for r in short_watch if r["sample_size"] >= 50]),
        ("SHORT Watch sample>=100", [r for r in short_watch if r["sample_size"] >= 100]),
        ("SHORT Watch vol=present", [r for r in short_watch if r["vol_regime"] == "present"]),
        ("SHORT Watch Exact+vol=present", [r for r in short_watch if r["is_exact"] and r["vol_regime"] == "present"]),
        ("SHORT Watch Exact+sample>=50+vol=present",
         [r for r in short_watch if r["is_exact"] and r["sample_size"] >= 50 and r["vol_regime"] == "present"]),
        ("SHORT Watch target_rate>=0.30", [r for r in short_watch if r["target_rate"] >= 0.30]),
        ("SHORT Watch target_rate>=0.35", [r for r in short_watch if r["target_rate"] >= 0.35]),
        ("SHORT Watch Exact+target_rate>=0.30",
         [r for r in short_watch if r["is_exact"] and r["target_rate"] >= 0.30]),
    ]

    for label, recs in combos:
        lines.append(short_row(label, recs))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## Interpretation notes")
    lines.append("")
    lines.append("- All results are in-sample on the 300 COMPLETE historical development set.")
    lines.append("- Median is the primary reliability indicator given gap-through outliers.")
    lines.append("- N < 15 should be treated as directional signals only, not stable estimates.")
    lines.append("- Any combination with N >= 15, positive median, and win rate > 60% is a walk-forward candidate.")
    lines.append("- TIME-009 observations used as outcome data only — not a tuning target.")
    lines.append("")
    lines.append("**Next step:** Sprint 3 — Failure analysis.")
    lines.append("For every losing SHORT Watch decision: what observable characteristics were present?")
    lines.append("Let the failure taxonomy emerge from the data.")

    report = "\n".join(lines)
    OUT_DOC.write_text(report)
    print(f"\nReport written to: {OUT_DOC}")
    print("\n" + "="*80)
    print(report)
    print("="*80)

if __name__ == "__main__":
    run()