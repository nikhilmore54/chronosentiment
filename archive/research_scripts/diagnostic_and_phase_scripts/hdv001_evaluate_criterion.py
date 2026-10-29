#!/usr/bin/env python3
"""
HDV-001-F Official Criterion Evaluation
=========================================
Evaluates the FROZEN HDV-001-G Gate 6 success criterion against the
corrected v2 baseline results.

Frozen criterion (HDV-001-G Gate 6, 2026-08-17):
  1. Coralys TARGET_BEFORE_RISK rate > Baseline A (Random) by >= 5 pp
  2. Coralys TARGET_BEFORE_RISK rate > Baseline B (Inverse) by >= 5 pp
  3. The difference is consistent across at least 2 of the 4 Coralys
     state segments (Bullish_Positive, Bullish_Negative, Bearish_Positive,
     Bearish_Negative)

Baseline C (Momentum) is a CONTEXTUAL baseline only.
It is NOT part of the frozen pass/fail criterion.
The v2 runner incorrectly added it to the gate — this script corrects that.

Same-bar ambiguity rule (from HDV-001-G Gate 4):
  When both target and stop are hit in the same session, TARGET takes
  precedence. This is preserved in the evaluate_outcome() function below.

Outputs:
  datasets/hdv001/HDV_001_F_DETERMINATION.md  (official determination)
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

WORKSPACE   = Path(__file__).resolve().parent.parent
OUTCOMES    = WORKSPACE / "datasets" / "hdv001" / "hdv001_outcomes_v1.json"
METRICS     = WORKSPACE / "datasets" / "hdv001" / "hdv001_decision_metrics_v1.json"
PRIMARY_CACHE = WORKSPACE / "datasets" / "hdv001" / "hdv001_price_cache_v1"
HIST_CACHE  = WORKSPACE / "datasets" / "hdv001" / "hdv001_baseline_history_v1"
DETERMINATION = WORKSPACE / "datasets" / "hdv001" / "HDV_001_F_DETERMINATION.md"

RANDOM_SEED = 42
MA_PERIOD   = 20
STATES      = ["Bullish_Positive", "Bullish_Negative", "Bearish_Positive", "Bearish_Negative"]

import random

def directional_boundaries(baseline_dir, ref, tgt, stp):
    td = abs(tgt - ref)
    sd = abs(stp - ref)
    if baseline_dir == "LONG":
        return ref + td, ref - sd
    else:
        return ref - td, ref + sd

def evaluate_outcome(direction, tgt_p, stp_p, path):
    """
    Bar-by-bar walk using high/low.
    Same-bar rule: if both hit in same bar, TARGET takes precedence.
    """
    same_bar_cases = 0
    for bar in path:
        h, l = bar["high"], bar["low"]
        if direction == "LONG":
            target_hit = h >= tgt_p
            stop_hit   = l <= stp_p
        else:
            target_hit = l <= tgt_p
            stop_hit   = h >= stp_p
        if target_hit and stop_hit:
            same_bar_cases += 1
            return "TARGET_HIT", same_bar_cases
        if target_hit:
            return "TARGET_HIT", same_bar_cases
        if stop_hit:
            return "STOP_HIT", same_bar_cases
    return "STOP_HIT", same_bar_cases   # unresolved = stop

def load_path(instrument, decision_date):
    fname = instrument.replace("&M", "ANDM").replace(".", "_") + ".json"
    p = PRIMARY_CACHE / fname
    if not p.exists():
        return []
    data = json.loads(p.read_text())
    return [b for b in data.get("bars", []) if b["date"] >= decision_date]

def load_history_closes(instrument, before_date, n):
    fname = instrument.replace("&M", "ANDM").replace(".", "_") + ".json"
    closes = []
    for cache in [HIST_CACHE, PRIMARY_CACHE]:
        fp = cache / fname
        if fp.exists():
            data = json.loads(fp.read_text())
            for b in data.get("bars", []):
                if b["date"] < before_date:
                    closes.append((b["date"], b["close"]))
    seen = set()
    unique = []
    for d, c in sorted(closes):
        if d not in seen:
            seen.add(d)
            unique.append(c)
    if len(unique) < n:
        return []
    return unique[-n:]

def main():
    print("=" * 70)
    print("HDV-001-F OFFICIAL CRITERION EVALUATION")
    print("Frozen criterion: HDV-001-G Gate 6")
    print("=" * 70)

    outcomes_raw = json.loads(OUTCOMES.read_text())
    metrics_raw  = json.loads(METRICS.read_text())
    metrics_by_id = {m["decision_id"]: m for m in metrics_raw["metrics"]}

    complete = [r for r in outcomes_raw["outcomes"]
                if r["observation_status"] == "COMPLETE"]
    print(f"COMPLETE decisions: {len(complete)}")

    rng = random.Random(RANDOM_SEED)

    # ── per-decision baseline evaluation ─────────────────────────────────────
    records = []
    total_same_bar = 0

    for rec in complete:
        m   = metrics_by_id[rec["decision_id"]]
        ref = m["reference_price"]
        tgt = m["target_price"]
        stp = m["stop_price"]
        path = load_path(rec["instrument"], rec["decision_date_ist"])
        state = f"{rec['coralys_trend']}_{rec['coralys_momentum']}"

        # Coralys (original direction, original boundaries)
        c_out, c_sb = evaluate_outcome(rec["direction"], tgt, stp, path)
        total_same_bar += c_sb

        # Baseline A: random direction, reconstructed boundaries
        a_dir = rng.choice(["LONG", "SHORT"])
        a_tgt, a_stp = directional_boundaries(a_dir, ref, tgt, stp)
        a_out, _ = evaluate_outcome(a_dir, a_tgt, a_stp, path)

        # Baseline B: inverse direction, reconstructed boundaries
        b_dir = "SHORT" if rec["direction"] == "LONG" else "LONG"
        b_tgt, b_stp = directional_boundaries(b_dir, ref, tgt, stp)
        b_out, _ = evaluate_outcome(b_dir, b_tgt, b_stp, path)

        # Baseline C: momentum (contextual only, not in criterion)
        closes = load_history_closes(rec["instrument"], rec["decision_date_ist"], MA_PERIOD)
        if closes:
            ma = sum(closes) / len(closes)
            c_mom_dir = "LONG" if closes[-1] > ma else "SHORT"
            c_tgt, c_stp = directional_boundaries(c_mom_dir, ref, tgt, stp)
            c_out_mom, _ = evaluate_outcome(c_mom_dir, c_tgt, c_stp, path)
        else:
            c_mom_dir = None
            c_out_mom = "SKIPPED"

        records.append({
            "decision_id":  rec["decision_id"],
            "state":        state,
            "coralys_out":  c_out,
            "rand_out":     a_out,
            "inv_out":      b_out,
            "mom_out":      c_out_mom,
        })

    # ── aggregate rates ───────────────────────────────────────────────────────
    n = len(records)
    c_rate  = sum(1 for r in records if r["coralys_out"] == "TARGET_HIT") / n * 100
    a_rate  = sum(1 for r in records if r["rand_out"]    == "TARGET_HIT") / n * 100
    b_rate  = sum(1 for r in records if r["inv_out"]     == "TARGET_HIT") / n * 100
    mom_elig = [r for r in records if r["mom_out"] != "SKIPPED"]
    m_rate  = (sum(1 for r in mom_elig if r["mom_out"] == "TARGET_HIT") / len(mom_elig) * 100
               if mom_elig else 0.0)

    margin_a = c_rate - a_rate
    margin_b = c_rate - b_rate

    print(f"\nAggregate rates (N={n}):")
    print(f"  Coralys  : {c_rate:.1f}%")
    print(f"  Random A : {a_rate:.1f}%  margin {margin_a:+.1f} pp")
    print(f"  Inverse B: {b_rate:.1f}%  margin {margin_b:+.1f} pp")
    print(f"  Momentum C (contextual): {m_rate:.1f}%  (N={len(mom_elig)} eligible)")
    print(f"\n  Same-bar target+stop cases (Coralys): {total_same_bar}")

    # ── state-segment evaluation ──────────────────────────────────────────────
    print(f"\nState-segment evaluation (frozen criterion: >= 2 of 4 segments):")
    print(f"  {'State':<22} {'Coralys':>9} {'Random':>9} {'Inverse':>9} {'Beats both?':>12}")
    print(f"  {'-'*65}")

    segments_pass = 0
    seg_detail = {}
    for st in STATES:
        sub = [r for r in records if r["state"] == st]
        ns = len(sub)
        if ns == 0:
            seg_detail[st] = {"n": 0, "coralys": None, "random": None, "inverse": None, "pass": False}
            continue
        sc = sum(1 for r in sub if r["coralys_out"] == "TARGET_HIT") / ns * 100
        sa = sum(1 for r in sub if r["rand_out"]    == "TARGET_HIT") / ns * 100
        sb = sum(1 for r in sub if r["inv_out"]     == "TARGET_HIT") / ns * 100
        beats = (sc - sa >= 5.0) and (sc - sb >= 5.0)
        if beats:
            segments_pass += 1
        seg_detail[st] = {"n": ns, "coralys": sc, "random": sa, "inverse": sb, "pass": beats}
        flag = "YES" if beats else "no"
        print(f"  {st:<22} {sc:>8.1f}% {sa:>8.1f}% {sb:>8.1f}% {flag:>12}")

    # ── frozen criterion evaluation ───────────────────────────────────────────
    gate1 = margin_a >= 5.0
    gate2 = margin_b >= 5.0
    gate3 = segments_pass >= 2

    print(f"\nFrozen criterion (HDV-001-G Gate 6):")
    print(f"  Gate 1 — Coralys > Random by >= 5 pp  : {margin_a:+.1f} pp  {'PASS' if gate1 else 'FAIL'}")
    print(f"  Gate 2 — Coralys > Inverse by >= 5 pp : {margin_b:+.1f} pp  {'PASS' if gate2 else 'FAIL'}")
    print(f"  Gate 3 — >= 2 segments beat both       : {segments_pass}/4  {'PASS' if gate3 else 'FAIL'}")

    overall = gate1 and gate2 and gate3
    print(f"\n  OFFICIAL HDV-001-F DETERMINATION: {'PASS' if overall else 'FAIL'}")

    # ── write determination document ──────────────────────────────────────────
    seg_rows = ""
    for st in STATES:
        d = seg_detail[st]
        if d["n"] == 0:
            seg_rows += f"| {st} | 0 | N/A | N/A | N/A | no |\n"
        else:
            flag = "YES" if d["pass"] else "no"
            seg_rows += (f"| {st} | {d['n']} | {d['coralys']:.1f}% | "
                         f"{d['random']:.1f}% | {d['inverse']:.1f}% | {flag} |\n")

    doc = f"""# HDV-001-F Official Determination

**Date:** 2026-08-17
**Determination:** {'PASS' if overall else 'FAIL'}
**Frozen criterion source:** datasets/hdv001/HDV_001_G_FREEZE_GATE.md (Gate 6)

---

## Governance Note

The v2 baseline runner (`hdv001_run_baselines.py`) incorrectly added Baseline C
(Momentum) to the pass/fail gate and declared "Overall FAIL". This was a
governance error. Baseline C is a contextual baseline only and is not part of
the frozen HDV-001-G success criterion.

This document contains the official determination against the frozen criterion.

---

## Frozen Success Criterion (HDV-001-G Gate 6)

1. Coralys TARGET_BEFORE_RISK rate > Baseline A (Random) by >= 5 pp
2. Coralys TARGET_BEFORE_RISK rate > Baseline B (Inverse) by >= 5 pp
3. The difference is consistent across at least 2 of the 4 Coralys state segments

Baseline C (Momentum) is reported for context but is NOT part of the criterion.

---

## Aggregate Results (N={n} COMPLETE decisions)

| Model | TARGET_HIT | Rate | Margin vs Coralys | Criterion |
|-------|-----------|------|-------------------|-----------|
| **Coralys** | {sum(1 for r in records if r['coralys_out']=='TARGET_HIT')} | **{c_rate:.1f}%** | — | — |
| Baseline A — Random | {sum(1 for r in records if r['rand_out']=='TARGET_HIT')} | {a_rate:.1f}% | {margin_a:+.1f} pp | {'PASS (>= 5 pp)' if gate1 else 'FAIL (< 5 pp)'} |
| Baseline B — Inverse | {sum(1 for r in records if r['inv_out']=='TARGET_HIT')} | {b_rate:.1f}% | {margin_b:+.1f} pp | {'PASS (>= 5 pp)' if gate2 else 'FAIL (< 5 pp)'} |
| Baseline C — Momentum (contextual) | {sum(1 for r in mom_elig if r['mom_out']=='TARGET_HIT')} | {m_rate:.1f}% | {c_rate-m_rate:+.1f} pp | not in criterion |

---

## State-Segment Evaluation

| State | N | Coralys | Random | Inverse | Beats both by 5pp? |
|-------|---|---------|--------|---------|-------------------|
{seg_rows}
Segments beating both baselines by >= 5 pp: **{segments_pass} / 4**

---

## Same-Bar Ambiguity

Same-bar target+stop cases (Coralys evaluation): **{total_same_bar}**
Resolution rule: TARGET takes precedence (per HDV-001-G Gate 4).

---

## Official Criterion Evaluation

| Gate | Check | Result | Status |
|------|-------|--------|--------|
| 1 | Coralys > Random by >= 5 pp | {margin_a:+.1f} pp | {'PASS' if gate1 else 'FAIL'} |
| 2 | Coralys > Inverse by >= 5 pp | {margin_b:+.1f} pp | {'PASS' if gate2 else 'FAIL'} |
| 3 | >= 2 of 4 segments beat both | {segments_pass}/4 | {'PASS' if gate3 else 'FAIL'} |
| **OVERALL** | | | **{'PASS' if overall else 'FAIL'}** |

---

## Governance Constraints

{'Criterion PASS: risk-boundary research (HDV-002) may proceed.' if overall else 'Criterion FAIL: C3-002 must not be modified. Stop-loss research must not resume. Risk-boundary research is not justified by this evidence.'}

Do not modify C3-002 or reference-risk boundaries based on these findings.
"""
    DETERMINATION.write_text(doc)
    print(f"\nDetermination written: {DETERMINATION.relative_to(WORKSPACE)}")
    sys.exit(0)

if __name__ == "__main__":
    main()