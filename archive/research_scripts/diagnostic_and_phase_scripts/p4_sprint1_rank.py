#!/usr/bin/env python3
"""
P4 Sprint 1 — Rank
==================
Evaluate candidate ranking policies against the 300 COMPLETE decisions.

Question: Can ranking decisions using only decision-time information produce
a materially better outcome than the unfiltered universe?

Inputs:  live_capture/ledger/entries/*.json  (decision-time fields)
         time_machine/analysis/TIME009/observations/*.json  (outcomes)
Outputs: printed report + docs/P4_SPRINT1_RANK_RESULTS.md

Guiding principle: Explore aggressively. Validate ruthlessly.
"""

import json
import os
import sys
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional
import statistics

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
OUT_DOC = REPO_ROOT / "docs" / "P4_SPRINT1_RANK_RESULTS.md"

# ── Data loading ───────────────────────────────────────────────────────────────

def load_json(path: Path) -> dict:
    with open(path) as f:
        return json.load(f)

def load_complete_decisions() -> list[dict]:
    """
    Join ledger entries with COMPLETE TIME-009 observations.
    Returns only decisions with a realized_return (COMPLETE, eligible).
    FORBIDDEN fields are stripped before any policy logic runs.
    """
    # Load observations indexed by decision_id
    obs_by_id = {}
    for p in OBS_DIR.glob("TIME009-OBS-*.json"):
        try:
            o = load_json(p)
            if o.get("observation_status") == "COMPLETE" and o.get("realized_return") is not None:
                obs_by_id[o["decision_id"]] = o
        except Exception:
            pass

    # Load ledger entries and join
    records = []
    for p in LEDGER_DIR.glob("*.json"):
        try:
            e = load_json(p)
            did = e.get("decision_id")
            if did not in obs_by_id:
                continue
            obs = obs_by_id[did]

            # ALLOWED fields from ledger entry
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
                # OUTCOME (used only for evaluation, never as policy input)
                "_realized_return": obs.get("realized_return"),
                "_target_reached": obs.get("target_reached"),
                "_risk_reached": obs.get("risk_reached"),
                "_horizon_reached": obs.get("horizon_reached"),
                "_actual_mfe": obs.get("actual_mfe"),
                "_actual_mae": obs.get("actual_mae"),
                "_eligible": obs.get("eligible_for_primary_comparison", False),
            }

            # DERIVED fields (computed from ALLOWED only)
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
            rec["is_favourable"] = rec["evidence_class"] == "Favourable"
            rec["is_mixed"] = rec["evidence_class"] == "Mixed"
            rec["is_exact"] = rec["degradation_level"] == "Exact"
            rec["is_certified"] = rec["certification_status"] == "CERTIFIED"

            records.append(rec)
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)

    return records

# ── Statistics ─────────────────────────────────────────────────────────────────

def stats(returns: list[float]) -> dict:
    if not returns:
        return {"n": 0, "mean": None, "median": None, "win_rate": None,
                "loss_rate": None, "mean_win": None, "mean_loss": None,
                "min": None, "max": None, "stdev": None}
    n = len(returns)
    wins = [r for r in returns if r > 0]
    losses = [r for r in returns if r < 0]
    return {
        "n": n,
        "mean": statistics.mean(returns),
        "median": statistics.median(returns),
        "win_rate": len(wins) / n,
        "loss_rate": len(losses) / n,
        "mean_win": statistics.mean(wins) if wins else 0.0,
        "mean_loss": statistics.mean(losses) if losses else 0.0,
        "min": min(returns),
        "max": max(returns),
        "stdev": statistics.stdev(returns) if n > 1 else 0.0,
    }

def fmt_pct(v) -> str:
    if v is None:
        return "—"
    return f"{v*100:+.3f}%"

def fmt_n(v) -> str:
    if v is None:
        return "—"
    return f"{v:.1%}"

# ── Policy definitions ─────────────────────────────────────────────────────────

def apply_selection(records: list[dict], sel: dict) -> list[dict]:
    """Apply a selection filter dict to records."""
    out = records
    if sel.get("action_watch"):
        out = [r for r in out if r["is_watch"]]
    if sel.get("direction") == "LONG":
        out = [r for r in out if r["is_long"]]
    if sel.get("direction") == "SHORT":
        out = [r for r in out if r["is_short"]]
    if sel.get("evidence") == "Favourable":
        out = [r for r in out if r["is_favourable"]]
    if sel.get("evidence") == "Favourable+Mixed":
        out = [r for r in out if r["is_favourable"] or r["is_mixed"]]
    if sel.get("exact_only"):
        out = [r for r in out if r["is_exact"]]
    if sel.get("certified_only"):
        out = [r for r in out if r["is_certified"]]
    min_ss = sel.get("min_sample_size")
    if min_ss is not None:
        out = [r for r in out if r["sample_size"] >= min_ss]
    min_tr = sel.get("min_target_rate")
    if min_tr is not None:
        out = [r for r in out if r["target_rate"] >= min_tr]
    min_rs = sel.get("min_rank_score")
    if min_rs is not None:
        out = [r for r in out if r["rank_score"] >= min_rs]
    min_rr = sel.get("min_rr_ratio")
    if min_rr is not None:
        out = [r for r in out if r["rr_ratio"] is not None and r["rr_ratio"] >= min_rr]
    return out

def rank_and_slice(records: list[dict], rank_by: str, top_pct: float) -> list[dict]:
    """Rank records by a field (descending) and return top_pct fraction."""
    valid = [r for r in records if r.get(rank_by) is not None]
    if not valid:
        return []
    valid.sort(key=lambda r: r[rank_by], reverse=True)
    n = max(1, int(len(valid) * top_pct))
    return valid[:n]

# ── Main experiment ────────────────────────────────────────────────────────────

def run():
    print("Loading COMPLETE decisions...")
    all_records = load_complete_decisions()
    print(f"  Total COMPLETE decisions loaded: {len(all_records)}")

    # Baseline: all records
    all_returns = [r["_realized_return"] for r in all_records]
    baseline = stats(all_returns)

    # Watch-only baseline (action == Watch)
    watch = [r for r in all_records if r["is_watch"]]
    watch_returns = [r["_realized_return"] for r in watch]
    watch_baseline = stats(watch_returns)

    lines = []
    lines.append("# P4 Sprint 1 — Rank Results")
    lines.append("")
    lines.append("**Date:** 2026-09-11")
    lines.append(f"**Population:** {len(all_records)} COMPLETE decisions")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Baseline")
    lines.append("")
    lines.append("| Cohort | N | Mean return | Median | Win rate | Loss rate | Mean win | Mean loss | Stdev |")
    lines.append("|---|---|---|---|---|---|---|---|---|")

    def row(label, s):
        return (f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} | "
                f"{fmt_pct(s['mean_win'])} | {fmt_pct(s['mean_loss'])} | {fmt_pct(s['stdev'])} |")

    lines.append(row("All COMPLETE", baseline))
    lines.append(row("Watch only", watch_baseline))

    # Direction split
    long_r = [r["_realized_return"] for r in all_records if r["is_long"]]
    short_r = [r["_realized_return"] for r in all_records if r["is_short"]]
    lines.append(row("LONG (all)", stats(long_r)))
    lines.append(row("SHORT (all)", stats(short_r)))

    long_watch_r = [r["_realized_return"] for r in watch if r["is_long"]]
    short_watch_r = [r["_realized_return"] for r in watch if r["is_short"]]
    lines.append(row("LONG Watch", stats(long_watch_r)))
    lines.append(row("SHORT Watch", stats(short_watch_r)))

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Evidence class breakdown (Watch decisions only)")
    lines.append("")
    lines.append("| Evidence | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    for ev in ["Favourable", "Mixed", "Insufficient"]:
        ev_r = [r["_realized_return"] for r in watch if r["evidence_class"] == ev]
        s = stats(ev_r)
        lines.append(f"| {ev} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Ranking experiment — does rank_score separate outcomes?")
    lines.append("")
    lines.append("Universe: Watch decisions only, ranked by `rank_score` descending.")
    lines.append("")
    lines.append("| Slice | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    for pct, label in [(0.05, "Top 5%"), (0.10, "Top 10%"), (0.20, "Top 20%"),
                        (0.50, "Top 50%"), (1.00, "Full universe"),
                        (None, "Bottom 20%"), (None, "Bottom 10%"), (None, "Bottom 5%")]:
        if pct is not None and label.startswith("Top"):
            sliced = rank_and_slice(watch, "rank_score", pct)
        elif label == "Full universe":
            sliced = [r for r in watch if r.get("rank_score") is not None]
        else:
            # Bottom slices
            valid = [r for r in watch if r.get("rank_score") is not None]
            valid.sort(key=lambda r: r["rank_score"])
            p = {"Bottom 20%": 0.20, "Bottom 10%": 0.10, "Bottom 5%": 0.05}[label]
            n = max(1, int(len(valid) * p))
            sliced = valid[:n]
        sr = [r["_realized_return"] for r in sliced]
        s = stats(sr)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Ranking experiment — target_rate")
    lines.append("")
    lines.append("Universe: Watch decisions only, ranked by `target_rate` descending.")
    lines.append("")
    lines.append("| Slice | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    for pct, label in [(0.05, "Top 5%"), (0.10, "Top 10%"), (0.20, "Top 20%"),
                        (1.00, "Full universe"),
                        (None, "Bottom 20%"), (None, "Bottom 10%"), (None, "Bottom 5%")]:
        if pct is not None and label.startswith("Top"):
            sliced = rank_and_slice(watch, "target_rate", pct)
        elif label == "Full universe":
            sliced = [r for r in watch if r.get("target_rate") is not None]
        else:
            valid = [r for r in watch if r.get("target_rate") is not None]
            valid.sort(key=lambda r: r["target_rate"])
            p = {"Bottom 20%": 0.20, "Bottom 10%": 0.10, "Bottom 5%": 0.05}[label]
            n = max(1, int(len(valid) * p))
            sliced = valid[:n]
        sr = [r["_realized_return"] for r in sliced]
        s = stats(sr)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Ranking experiment — expected_return (EV)")
    lines.append("")
    lines.append("Universe: Watch decisions with valid target/risk, ranked by `expected_return` descending.")
    lines.append("")
    lines.append("| Slice | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    ev_watch = [r for r in watch if r.get("expected_return") is not None]
    for pct, label in [(0.05, "Top 5%"), (0.10, "Top 10%"), (0.20, "Top 20%"),
                        (1.00, "Full universe"),
                        (None, "Bottom 20%"), (None, "Bottom 10%"), (None, "Bottom 5%")]:
        if pct is not None and label.startswith("Top"):
            sliced = rank_and_slice(ev_watch, "expected_return", pct)
        elif label == "Full universe":
            sliced = ev_watch
        else:
            valid = sorted(ev_watch, key=lambda r: r["expected_return"])
            p = {"Bottom 20%": 0.20, "Bottom 10%": 0.10, "Bottom 5%": 0.05}[label]
            n = max(1, int(len(valid) * p))
            sliced = valid[:n]
        sr = [r["_realized_return"] for r in sliced]
        s = stats(sr)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Ranking experiment — rr_ratio")
    lines.append("")
    lines.append("Universe: Watch decisions with valid rr_ratio, ranked by `rr_ratio` descending.")
    lines.append("")
    lines.append("| Slice | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    rr_watch = [r for r in watch if r.get("rr_ratio") is not None and r["rr_ratio"] > 0]
    for pct, label in [(0.05, "Top 5%"), (0.10, "Top 10%"), (0.20, "Top 20%"),
                        (1.00, "Full universe"),
                        (None, "Bottom 20%"), (None, "Bottom 10%"), (None, "Bottom 5%")]:
        if pct is not None and label.startswith("Top"):
            sliced = rank_and_slice(rr_watch, "rr_ratio", pct)
        elif label == "Full universe":
            sliced = rr_watch
        else:
            valid = sorted(rr_watch, key=lambda r: r["rr_ratio"])
            p = {"Bottom 20%": 0.20, "Bottom 10%": 0.10, "Bottom 5%": 0.05}[label]
            n = max(1, int(len(valid) * p))
            sliced = valid[:n]
        sr = [r["_realized_return"] for r in sliced]
        s = stats(sr)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Selection filter experiment — LONG Watch only")
    lines.append("")
    lines.append("| Filter | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    filter_experiments = [
        ("LONG Watch (all)", {"action_watch": True, "direction": "LONG"}),
        ("LONG Watch Favourable", {"action_watch": True, "direction": "LONG", "evidence": "Favourable"}),
        ("LONG Watch Fav+Mixed", {"action_watch": True, "direction": "LONG", "evidence": "Favourable+Mixed"}),
        ("LONG Watch Exact", {"action_watch": True, "direction": "LONG", "exact_only": True}),
        ("LONG Watch sample>=50", {"action_watch": True, "direction": "LONG", "min_sample_size": 50}),
        ("LONG Watch sample>=100", {"action_watch": True, "direction": "LONG", "min_sample_size": 100}),
        ("LONG Watch target_rate>=0.35", {"action_watch": True, "direction": "LONG", "min_target_rate": 0.35}),
        ("LONG Watch target_rate>=0.40", {"action_watch": True, "direction": "LONG", "min_target_rate": 0.40}),
        ("LONG Watch rank_score>=0.5", {"action_watch": True, "direction": "LONG", "min_rank_score": 0.5}),
        ("LONG Watch rank_score>=0.6", {"action_watch": True, "direction": "LONG", "min_rank_score": 0.6}),
        ("LONG Watch rr>=1.0", {"action_watch": True, "direction": "LONG", "min_rr_ratio": 1.0}),
        ("LONG Watch Exact+sample>=50", {"action_watch": True, "direction": "LONG", "exact_only": True, "min_sample_size": 50}),
        ("LONG Watch Fav+sample>=50", {"action_watch": True, "direction": "LONG", "evidence": "Favourable", "min_sample_size": 50}),
        ("LONG Watch Fav+tr>=0.35", {"action_watch": True, "direction": "LONG", "evidence": "Favourable", "min_target_rate": 0.35}),
    ]

    for label, sel in filter_experiments:
        selected = apply_selection(all_records, sel)
        sr = [r["_realized_return"] for r in selected]
        s = stats(sr)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Selection filter experiment — SHORT Watch only")
    lines.append("")
    lines.append("| Filter | N | Mean return | Median | Win rate | Loss rate |")
    lines.append("|---|---|---|---|---|---|")

    short_filter_experiments = [
        ("SHORT Watch (all)", {"action_watch": True, "direction": "SHORT"}),
        ("SHORT Watch Favourable", {"action_watch": True, "direction": "SHORT", "evidence": "Favourable"}),
        ("SHORT Watch Fav+Mixed", {"action_watch": True, "direction": "SHORT", "evidence": "Favourable+Mixed"}),
        ("SHORT Watch Exact", {"action_watch": True, "direction": "SHORT", "exact_only": True}),
        ("SHORT Watch sample>=50", {"action_watch": True, "direction": "SHORT", "min_sample_size": 50}),
        ("SHORT Watch target_rate>=0.35", {"action_watch": True, "direction": "SHORT", "min_target_rate": 0.35}),
        ("SHORT Watch rank_score>=0.5", {"action_watch": True, "direction": "SHORT", "min_rank_score": 0.5}),
    ]

    for label, sel in short_filter_experiments:
        selected = apply_selection(all_records, sel)
        sr = [r["_realized_return"] for r in selected]
        s = stats(sr)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} | {fmt_pct(s['median'])} | "
                     f"{fmt_n(s['win_rate'])} | {fmt_n(s['loss_rate'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Sample size distribution (Watch decisions)")
    lines.append("")
    lines.append("| Sample size bucket | N decisions | Mean return |")
    lines.append("|---|---|---|")

    buckets = [(0, 0), (1, 25), (26, 50), (51, 100), (101, 150), (151, 999)]
    for lo, hi in buckets:
        if lo == 0 and hi == 0:
            bucket_r = [r["_realized_return"] for r in watch if r["sample_size"] == 0]
            label = "0 (NoTrade/Insufficient)"
        else:
            bucket_r = [r["_realized_return"] for r in watch if lo <= r["sample_size"] <= hi]
            label = f"{lo}–{hi}"
        s = stats(bucket_r)
        lines.append(f"| {label} | {s['n']} | {fmt_pct(s['mean'])} |")

    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Interpretation notes")
    lines.append("")
    lines.append("- All policies evaluated on the same 300 COMPLETE decisions (historical development set).")
    lines.append("- These results are in-sample observations, not walk-forward validated.")
    lines.append("- Any policy showing material improvement becomes a finalist for walk-forward evaluation.")
    lines.append("- A policy that fails to improve is an equally valid result — it eliminates a hypothesis.")
    lines.append("- TIME-009 observations are used as outcome data only; they are not a tuning target.")
    lines.append("")
    lines.append("**Next step:** Identify the strongest-performing selection/ranking combination,")
    lines.append("then proceed to Sprint 2 (Measure) and Sprint 3 (Failure analysis).")

    report = "\n".join(lines)

    # Write to file
    OUT_DOC.write_text(report)
    print(f"\nReport written to: {OUT_DOC}")

    # Also print to stdout
    print("\n" + "="*80)
    print(report)
    print("="*80)

if __name__ == "__main__":
    run()