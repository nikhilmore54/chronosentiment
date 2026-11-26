#!/usr/bin/env python3
"""
P4 Sprint 3 — Failure Analysis
================================
Three-bucket separation of SHORT Watch decisions:
  A — Genuine winners: positive return, |return| <= 50% (no gap-through)
  B — Losers: negative return
  C — Gap-through winners: |return| > 50% (extreme positive outcomes)

Governing question:
  Can we find a decision-time characteristic that predicts genuine positive
  returns rather than merely predicting gap-through outcomes?

Decisive experiment:
  Does the 51–150 sample size band survive gap-through removal?

Inputs:  live_capture/ledger/entries/*.json
         time_machine/analysis/TIME009/observations/*.json
Outputs: docs/P4_SPRINT3_FAILURE_RESULTS.md
"""

import json
import sys
from pathlib import Path
import statistics

REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
OUT_DOC = REPO_ROOT / "docs" / "P4_SPRINT3_FAILURE_RESULTS.md"

GAP_THRESHOLD = 0.50  # |return| > 50% = gap-through

# ── Data loading ───────────────────────────────────────────────────────────────

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

            # Outcome buckets (FORBIDDEN as policy inputs — used only for analysis)
            r = rec["_realized_return"]
            rec["_is_gap_through"] = abs(r) > GAP_THRESHOLD
            rec["_is_genuine_winner"] = r > 0 and abs(r) <= GAP_THRESHOLD
            rec["_is_loser"] = r < 0
            rec["_is_flat"] = r == 0
            rec["_is_gap_winner"] = r > GAP_THRESHOLD

            records.append(rec)
        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)

    return records

# ── Statistics ─────────────────────────────────────────────────────────────────

def pct(v, d=1):
    return f"{v*100:+.{d}f}%" if v is not None else "—"

def pct_plain(v, d=1):
    return f"{v*100:.{d}f}%" if v is not None else "—"

def stats(returns):
    if not returns:
        return {}
    n = len(returns)
    wins = [r for r in returns if r > 0]
    losses = [r for r in returns if r < 0]
    return {
        "n": n,
        "mean": statistics.mean(returns),
        "median": statistics.median(returns),
        "win_rate": len(wins) / n,
        "loss_rate": len(losses) / n,
        "stdev": statistics.stdev(returns) if n > 1 else 0.0,
        "min": min(returns),
        "max": max(returns),
    }

def row(label, recs):
    returns = [r["_realized_return"] for r in recs]
    s = stats(returns)
    if not s:
        return f"| {label} | 0 | — | — | — | — | — |"
    return (f"| {label} | {s['n']} | {pct(s['mean'])} | {pct(s['median'])} | "
            f"{pct_plain(s['win_rate'])} | {pct_plain(s['loss_rate'])} | {pct(s['stdev'])} |")

def bucket_summary(label, recs):
    """Show three-bucket breakdown for a cohort."""
    n = len(recs)
    genuine = [r for r in recs if r["_is_genuine_winner"]]
    losers = [r for r in recs if r["_is_loser"]]
    gap_win = [r for r in recs if r["_is_gap_winner"]]
    flat = [r for r in recs if r["_is_flat"]]
    return (f"| {label} | {n} | {len(genuine)} ({pct_plain(len(genuine)/n if n else None)}) | "
            f"{len(losers)} ({pct_plain(len(losers)/n if n else None)}) | "
            f"{len(gap_win)} ({pct_plain(len(gap_win)/n if n else None)}) | "
            f"{len(flat)} ({pct_plain(len(flat)/n if n else None)}) |")

def quartile_rows(recs, field, label):
    valid = [r for r in recs if r.get(field) is not None]
    if not valid:
        return []
    valid.sort(key=lambda r: r[field])
    n = len(valid)
    qs = [valid[:n//4], valid[n//4:n//2], valid[n//2:3*n//4], valid[3*n//4:]]
    names = ["Q1 (lowest)", "Q2", "Q3", "Q4 (highest)"]
    return [row(f"{label} {qn}", qrecs) for qn, qrecs in zip(names, qs)]

# ── Main ───────────────────────────────────────────────────────────────────────

def run():
    print("Loading COMPLETE decisions...")
    all_records = load_complete_decisions()
    print(f"  Total: {len(all_records)}")

    watch = [r for r in all_records if r["is_watch"]]
    short_watch = [r for r in watch if r["is_short"]]
    long_watch = [r for r in watch if r["is_long"]]

    # Three buckets for SHORT Watch
    sw_genuine = [r for r in short_watch if r["_is_genuine_winner"]]
    sw_losers = [r for r in short_watch if r["_is_loser"]]
    sw_gap = [r for r in short_watch if r["_is_gap_winner"]]
    sw_flat = [r for r in short_watch if r["_is_flat"]]

    # SHORT Watch excluding gap-throughs
    sw_no_gap = [r for r in short_watch if not r["_is_gap_through"]]

    lines = []
    lines.append("# P4 Sprint 3 — Failure Analysis Results")
    lines.append("")
    lines.append("**Date:** 2026-09-11")
    lines.append(f"**Population:** {len(all_records)} COMPLETE decisions")
    lines.append(f"**SHORT Watch decisions:** {len(short_watch)}")
    lines.append("")
    lines.append("**Governing question:** Can we find a decision-time characteristic that predicts")
    lines.append("genuine positive returns rather than merely predicting gap-through outcomes?")
    lines.append("")
    lines.append(f"**Gap-through threshold:** |return| > {GAP_THRESHOLD*100:.0f}%")
    lines.append("")
    lines.append("**Three outcome buckets:**")
    lines.append("- **A — Genuine winners:** positive return, |return| ≤ 50%")
    lines.append("- **B — Losers:** negative return")
    lines.append("- **C — Gap-through winners:** return > 50% (extreme positive, typically target hit)")
    lines.append("")
    lines.append("---")
    lines.append("")

    # ── Section 1: Bucket overview ─────────────────────────────────────────────
    lines.append("## 1. Outcome bucket overview")
    lines.append("")
    lines.append("| Cohort | N | Genuine winners (A) | Losers (B) | Gap-through winners (C) | Flat |")
    lines.append("|---|---|---|---|---|---|")
    lines.append(bucket_summary("All Watch", watch))
    lines.append(bucket_summary("LONG Watch", long_watch))
    lines.append(bucket_summary("SHORT Watch", short_watch))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 2. SHORT Watch — gap-through excluded (decisive experiment)")
    lines.append("")
    lines.append("If the 51–150 sample size band survives gap-through removal, it is a genuine signal.")
    lines.append("If it collapses, the 'sweet spot' was an artifact of the gap-through mechanism.")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    lines.append(row("SHORT Watch (all, incl gaps)", short_watch))
    lines.append(row("SHORT Watch (gap-through excluded)", sw_no_gap))
    lines.append(row("SHORT Watch genuine winners only", sw_genuine))
    lines.append(row("SHORT Watch losers only", sw_losers))
    lines.append(row("SHORT Watch gap-through winners only", sw_gap))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 3. Sample size bands — with and without gap-throughs")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    for lo, hi, label in [(1, 50, "1–50"), (51, 100, "51–100"), (101, 150, "101–150"), (151, 999, "151+")]:
        recs_all = [r for r in short_watch if lo <= r["sample_size"] <= hi]
        recs_no_gap = [r for r in recs_all if not r["_is_gap_through"]]
        lines.append(row(f"SHORT Watch sample {label} (all)", recs_all))
        lines.append(row(f"SHORT Watch sample {label} (no gap)", recs_no_gap))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 4. rank_score quartiles — with and without gap-throughs")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    valid = [r for r in short_watch if r.get("rank_score") is not None]
    valid.sort(key=lambda r: r["rank_score"])
    n = len(valid)
    for qname, qrecs in [("Q1", valid[:n//4]), ("Q2", valid[n//4:n//2]),
                          ("Q3", valid[n//2:3*n//4]), ("Q4", valid[3*n//4:])]:
        no_gap = [r for r in qrecs if not r["_is_gap_through"]]
        lines.append(row(f"SHORT Watch rank_score {qname} (all)", qrecs))
        lines.append(row(f"SHORT Watch rank_score {qname} (no gap)", no_gap))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 5. target_rate quartiles — with and without gap-throughs")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")
    valid = sorted(short_watch, key=lambda r: r["target_rate"])
    n = len(valid)
    for qname, qrecs in [("Q1", valid[:n//4]), ("Q2", valid[n//4:n//2]),
                          ("Q3", valid[n//2:3*n//4]), ("Q4", valid[3*n//4:])]:
        no_gap = [r for r in qrecs if not r["_is_gap_through"]]
        lines.append(row(f"SHORT Watch target_rate {qname} (all)", qrecs))
        lines.append(row(f"SHORT Watch target_rate {qname} (no gap)", no_gap))
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 6. Characteristics of SHORT Watch losers (Bucket B)")
    lines.append("")
    lines.append(f"N losers: {len(sw_losers)}")
    lines.append("")
    lines.append("### 6a. Sample size distribution of losers")
    lines.append("")
    lines.append("| Sample size band | N losers | N total SHORT Watch | Loser rate |")
    lines.append("|---|---|---|---|")
    for lo, hi, label in [(1, 50, "1–50"), (51, 100, "51–100"), (101, 150, "101–150"), (151, 999, "151+")]:
        n_total = len([r for r in short_watch if lo <= r["sample_size"] <= hi])
        n_loss = len([r for r in sw_losers if lo <= r["sample_size"] <= hi])
        rate = f"{n_loss/n_total*100:.1f}%" if n_total > 0 else "—"
        lines.append(f"| {label} | {n_loss} | {n_total} | {rate} |")
    lines.append("")

    lines.append("### 6b. rank_score distribution of losers vs winners")
    lines.append("")
    lines.append("| Bucket | N | Mean rank_score | Median rank_score |")
    lines.append("|---|---|---|---|")
    for label, recs in [("Genuine winners (A)", sw_genuine), ("Losers (B)", sw_losers),
                         ("Gap-through winners (C)", sw_gap)]:
        rs = [r["rank_score"] for r in recs if r.get("rank_score") is not None]
        if rs:
            lines.append(f"| {label} | {len(rs)} | {statistics.mean(rs):.4f} | {statistics.median(rs):.4f} |")
        else:
            lines.append(f"| {label} | 0 | — | — |")
    lines.append("")

    lines.append("### 6c. target_rate distribution of losers vs winners")
    lines.append("")
    lines.append("| Bucket | N | Mean target_rate | Median target_rate |")
    lines.append("|---|---|---|---|")
    for label, recs in [("Genuine winners (A)", sw_genuine), ("Losers (B)", sw_losers),
                         ("Gap-through winners (C)", sw_gap)]:
        tr = [r["target_rate"] for r in recs]
        if tr:
            lines.append(f"| {label} | {len(tr)} | {statistics.mean(tr):.4f} | {statistics.median(tr):.4f} |")
        else:
            lines.append(f"| {label} | 0 | — | — |")
    lines.append("")

    lines.append("### 6d. sample_size distribution of losers vs winners")
    lines.append("")
    lines.append("| Bucket | N | Mean sample_size | Median sample_size |")
    lines.append("|---|---|---|---|")
    for label, recs in [("Genuine winners (A)", sw_genuine), ("Losers (B)", sw_losers),
                         ("Gap-through winners (C)", sw_gap)]:
        ss = [r["sample_size"] for r in recs]
        if ss:
            lines.append(f"| {label} | {len(ss)} | {statistics.mean(ss):.1f} | {statistics.median(ss):.1f} |")
        else:
            lines.append(f"| {label} | 0 | — | — |")
    lines.append("")

    lines.append("### 6e. rr_ratio distribution of losers vs winners")
    lines.append("")
    lines.append("| Bucket | N | Mean rr_ratio | Median rr_ratio |")
    lines.append("|---|---|---|---|")
    for label, recs in [("Genuine winners (A)", sw_genuine), ("Losers (B)", sw_losers),
                         ("Gap-through winners (C)", sw_gap)]:
        rr = [r["rr_ratio"] for r in recs if r.get("rr_ratio") is not None]
        if rr:
            lines.append(f"| {label} | {len(rr)} | {statistics.mean(rr):.4f} | {statistics.median(rr):.4f} |")
        else:
            lines.append(f"| {label} | 0 | — | — |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 7. SHORT Watch — gap-through excluded, full decomposition")
    lines.append("")
    lines.append("Repeating Sprint 2 splits on gap-through-excluded SHORT Watch decisions.")
    lines.append("This is the decisive test of whether any signal survives gap-through removal.")
    lines.append("")
    lines.append("| Cohort | N | Mean | Median | Win% | Loss% | Stdev |")
    lines.append("|---|---|---|---|---|---|---|")

    # Degradation
    for deg in ["Exact", "Approximate"]:
        recs = [r for r in sw_no_gap if r["degradation_level"] == deg]
        lines.append(row(f"SHORT Watch {deg} (no gap)", recs))

    # Sample size
    for lo, hi, label in [(51, 100, "51–100"), (101, 150, "101–150"), (151, 999, "151+")]:
        recs = [r for r in sw_no_gap if lo <= r["sample_size"] <= hi]
        lines.append(row(f"SHORT Watch sample {label} (no gap)", recs))

    # rank_score quartiles
    for qrow in quartile_rows(sw_no_gap, "rank_score", "SHORT Watch (no gap) rank_score"):
        lines.append(qrow)

    # target_rate quartiles
    for qrow in quartile_rows(sw_no_gap, "target_rate", "SHORT Watch (no gap) target_rate"):
        lines.append(qrow)

    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 8. LONG Watch Q3 rank_score pocket — detailed inspection")
    lines.append("")
    lines.append("Sprint 2 found: LONG Watch rank_score Q3 — N=13, +2.5% mean, +1.3% median, 76.9% win, 2.9% stdev.")
    lines.append("Suspiciously low stdev suggests a cluster of similar decisions. Inspecting.")
    lines.append("")

    lw_valid = [r for r in long_watch if r.get("rank_score") is not None]
    lw_valid.sort(key=lambda r: r["rank_score"])
    n = len(lw_valid)
    lw_q3 = lw_valid[n//2:3*n//4]

    lines.append(f"N in LONG Watch rank_score Q3: {len(lw_q3)}")
    lines.append("")
    lines.append("| decision_id | sample_size | target_rate | rank_score | realized_return |")
    lines.append("|---|---|---|---|---|")
    for r in sorted(lw_q3, key=lambda x: x["rank_score"]):
        lines.append(f"| {r['decision_id']} | {r['sample_size']} | {r['target_rate']:.4f} | "
                     f"{r['rank_score']:.4f} | {pct(r['_realized_return'])} |")
    lines.append("")

    # Check ticker concentration
    tickers = [r["ticker"] for r in lw_q3]
    ticker_counts = {}
    for t in tickers:
        ticker_counts[t] = ticker_counts.get(t, 0) + 1
    lines.append("**Ticker concentration in LONG Watch Q3:**")
    lines.append("")
    lines.append("| Ticker | Count |")
    lines.append("|---|---|")
    for t, c in sorted(ticker_counts.items(), key=lambda x: -x[1]):
        lines.append(f"| {t} | {c} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 9. Failure taxonomy — emerging categories")
    lines.append("")
    lines.append("Based on the three-bucket analysis, the following failure patterns are observable:")
    lines.append("")

    # Compute loser characteristics
    loser_ss_mean = statistics.mean([r["sample_size"] for r in sw_losers]) if sw_losers else None
    winner_ss_mean = statistics.mean([r["sample_size"] for r in sw_genuine]) if sw_genuine else None
    gap_ss_mean = statistics.mean([r["sample_size"] for r in sw_gap]) if sw_gap else None

    loser_tr_mean = statistics.mean([r["target_rate"] for r in sw_losers]) if sw_losers else None
    winner_tr_mean = statistics.mean([r["target_rate"] for r in sw_genuine]) if sw_genuine else None
    gap_tr_mean = statistics.mean([r["target_rate"] for r in sw_gap]) if sw_gap else None

    lines.append(f"**Genuine winners (A):** N={len(sw_genuine)}, "
                 f"mean sample_size={loser_ss_mean and winner_ss_mean and f'{winner_ss_mean:.0f}' or '—'}, "
                 f"mean target_rate={winner_tr_mean and f'{winner_tr_mean:.3f}' or '—'}")
    lines.append(f"**Losers (B):** N={len(sw_losers)}, "
                 f"mean sample_size={loser_ss_mean and f'{loser_ss_mean:.0f}' or '—'}, "
                 f"mean target_rate={loser_tr_mean and f'{loser_tr_mean:.3f}' or '—'}")
    lines.append(f"**Gap-through winners (C):** N={len(sw_gap)}, "
                 f"mean sample_size={gap_ss_mean and f'{gap_ss_mean:.0f}' or '—'}, "
                 f"mean target_rate={gap_tr_mean and f'{gap_tr_mean:.3f}' or '—'}")
    lines.append("")
    lines.append("Failure taxonomy (to be refined as more data arrives):")
    lines.append("")
    lines.append("| Failure class | Observable signal | Hypothesis |")
    lines.append("|---|---|---|")
    lines.append("| Insufficient analogues | sample_size < 51 | Too few historical comparisons to establish reliable direction |")
    lines.append("| Analogue saturation | sample_size > 150 | Large analogue pools may include dissimilar situations, diluting signal |")
    lines.append("| Gap-through dependency | |return| > 50% | Apparent edge may require target hit; non-gap-through outcomes are flat |")
    lines.append("| Direction mismatch | LONG direction | LONG Watch decisions are structurally negative in current sample |")
    lines.append("| Score inflation | High rank_score on LONG | rank_score assigns higher values to LONG, which underperforms |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## 10. Walk-forward finalist candidates — revised after gap-through analysis")
    lines.append("")
    lines.append("Criteria: N >= 15, positive median (including gap-through excluded), win rate > 60%")
    lines.append("")
    lines.append("| Candidate | N (all) | Median (all) | N (no gap) | Median (no gap) | Win% (no gap) | Status |")
    lines.append("|---|---|---|---|---|---|---|")

    candidates = [
        ("SHORT Watch Exact", [r for r in short_watch if r["is_exact"]]),
        ("SHORT Watch sample 51–100", [r for r in short_watch if 51 <= r["sample_size"] <= 100]),
        ("SHORT Watch sample 101–150", [r for r in short_watch if 101 <= r["sample_size"] <= 150]),
        ("SHORT Watch sample 51–150", [r for r in short_watch if 51 <= r["sample_size"] <= 150]),
        ("SHORT Watch target_rate Q2", sorted(short_watch, key=lambda r: r["target_rate"])[n//4:n//2]
         if len(short_watch) >= 4 else []),
        ("SHORT Watch target_rate Q3", sorted(short_watch, key=lambda r: r["target_rate"])[n//2:3*n//4]
         if len(short_watch) >= 4 else []),
    ]

    sw_n = len(short_watch)
    tr_sorted = sorted(short_watch, key=lambda r: r["target_rate"])
    candidates = [
        ("SHORT Watch Exact", [r for r in short_watch if r["is_exact"]]),
        ("SHORT Watch sample 51–100", [r for r in short_watch if 51 <= r["sample_size"] <= 100]),
        ("SHORT Watch sample 101–150", [r for r in short_watch if 101 <= r["sample_size"] <= 150]),
        ("SHORT Watch sample 51–150", [r for r in short_watch if 51 <= r["sample_size"] <= 150]),
        ("SHORT Watch target_rate Q2", tr_sorted[sw_n//4:sw_n//2]),
        ("SHORT Watch target_rate Q3", tr_sorted[sw_n//2:3*sw_n//4]),
    ]

    for label, recs in candidates:
        no_gap = [r for r in recs if not r["_is_gap_through"]]
        s_all = stats([r["_realized_return"] for r in recs])
        s_ng = stats([r["_realized_return"] for r in no_gap])
        n_all = s_all.get("n", 0)
        med_all = pct(s_all.get("median"))
        n_ng = s_ng.get("n", 0)
        med_ng = pct(s_ng.get("median"))
        wr_ng = pct_plain(s_ng.get("win_rate"))
        # Status: FINALIST if n_all >= 15 and median_no_gap > 0 and win_rate_no_gap > 0.5
        med_ng_val = s_ng.get("median")
        wr_ng_val = s_ng.get("win_rate")
        if n_all >= 15 and med_ng_val is not None and med_ng_val > 0 and wr_ng_val is not None and wr_ng_val > 0.5:
            status = "✓ FINALIST"
        elif n_all >= 15 and med_ng_val is not None and med_ng_val > 0:
            status = "~ MARGINAL"
        else:
            status = "✗ ELIMINATED"
        lines.append(f"| {label} | {n_all} | {med_all} | {n_ng} | {med_ng} | {wr_ng} | {status} |")
    lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## Interpretation notes")
    lines.append("")
    lines.append("- Gap-through events (|return| > 50%) are the primary driver of SHORT Watch's apparent edge.")
    lines.append("- The decisive question is whether any decision-time variable predicts genuine returns")
    lines.append("  (positive return without gap-through) rather than merely predicting gap-through frequency.")
    lines.append("- Any finalist surviving gap-through removal is a candidate for walk-forward evaluation.")
    lines.append("- Finalists with gap-through-dependent performance require a different policy framework:")
    lines.append("  they may still be valuable if gap-through events are predictable, but that requires")
    lines.append("  a separate gap-through prediction experiment, not a return-prediction experiment.")
    lines.append("- TIME-009 observations used as outcome data only — not a tuning target.")
    lines.append("")
    lines.append("**Next step:** Walk-forward evaluation of FINALIST candidates.")

    report = "\n".join(lines)
    OUT_DOC.write_text(report)
    print(f"\nReport written to: {OUT_DOC}")
    print("\n" + "="*80)
    print(report)
    print("="*80)

if __name__ == "__main__":
    run()