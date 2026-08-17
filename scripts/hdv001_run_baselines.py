#!/usr/bin/env python3
"""
HDV-001-F Baseline Comparison Runner (CORRECTED)
=================================================
Runs three mechanical baselines against the 728 COMPLETE decisions.

Corrections applied vs. invalid first run (2026-08-17):
  Rule A — Directional geometry: target/stop distances are preserved from
            reference_price but reconstructed for the baseline direction.
            Absolute prices from Coralys are NEVER reused as-is for a
            direction-flipped baseline.
  Rule B — Momentum history: Baseline C uses hdv001_baseline_history_v1
            (2026-06-01 to 2026-07-13) for pre-decision lookback.
            Zero random fallback is permitted. Decisions with insufficient
            history are SKIPPED (not contaminated with random).

Baselines:
  A — Random:    direction sampled uniformly at random (LONG/SHORT)
  B — Inverse:   direction is always opposite to Coralys
  C — Momentum:  direction follows 20-session MA crossover on close prices

Success criterion (frozen in HDV-001-G Gate 6):
  Coralys TARGET_HIT rate on COMPLETE decisions must exceed all three
  baselines by >= 5 percentage points.

Outputs:
  datasets/hdv001/hdv001_baseline_results_v2.json
  datasets/hdv001/HDV_001_F_BASELINE_REPORT_v2.md
"""

import json
import random
import sys
from datetime import datetime, timezone
from pathlib import Path

# ── paths ─────────────────────────────────────────────────────────────────────
WORKSPACE    = Path(__file__).resolve().parent.parent
OUTCOMES     = WORKSPACE / "datasets" / "hdv001" / "hdv001_outcomes_v1.json"
METRICS      = WORKSPACE / "datasets" / "hdv001" / "hdv001_decision_metrics_v1.json"
PRIMARY_CACHE = WORKSPACE / "datasets" / "hdv001" / "hdv001_price_cache_v1"
HIST_CACHE   = WORKSPACE / "datasets" / "hdv001" / "hdv001_baseline_history_v1"
OUT_JSON     = WORKSPACE / "datasets" / "hdv001" / "hdv001_baseline_results_v2.json"
OUT_REPORT   = WORKSPACE / "datasets" / "hdv001" / "HDV_001_F_BASELINE_REPORT_v2.md"

RANDOM_SEED  = 42
MA_PERIOD    = 20

# ── helpers ───────────────────────────────────────────────────────────────────

def directional_boundaries(baseline_direction: str,
                           reference_price: float,
                           target_price: float,
                           stop_price: float) -> tuple[float, float]:
    """
    Rule A fix: reconstruct target/stop for baseline_direction by preserving
    the absolute distances from reference_price that Coralys declared.
    """
    target_distance = abs(target_price - reference_price)
    stop_distance   = abs(stop_price   - reference_price)
    if baseline_direction == "LONG":
        return reference_price + target_distance, reference_price - stop_distance
    else:
        return reference_price - target_distance, reference_price + stop_distance


def evaluate_outcome(direction: str,
                     entry_price: float,
                     target_price: float,
                     stop_price: float,
                     price_path: list[dict]) -> str:
    """
    Walk the price path bar-by-bar (high/low) and return TARGET_HIT or STOP_HIT.
    Uses the same logic as HDV-001-E outcome classifier.
    """
    for bar in price_path:
        high = bar["high"]
        low  = bar["low"]
        if direction == "LONG":
            if high >= target_price:
                return "TARGET_HIT"
            if low  <= stop_price:
                return "STOP_HIT"
        else:  # SHORT
            if low  <= target_price:
                return "TARGET_HIT"
            if high >= stop_price:
                return "STOP_HIT"
    return "STOP_HIT"   # conservative: unresolved = stop


def load_price_path(instrument: str, decision_date: str) -> list[dict]:
    """
    Load bars from the primary cache starting from decision_date.
    Returns list of bar dicts (date, open, high, low, close).
    """
    fname = instrument.replace("&M", "ANDM").replace(".", "_") + ".json"
    fpath = PRIMARY_CACHE / fname
    if not fpath.exists():
        return []
    data = json.loads(fpath.read_text())
    bars = data.get("bars", [])
    return [b for b in bars if b["date"] >= decision_date]


def load_history_closes(instrument: str, before_date: str, n: int) -> list[float]:
    """
    Rule B fix: load up to n closing prices strictly before before_date.
    Combines hdv001_baseline_history_v1 (pre-development) and
    hdv001_price_cache_v1 (development period) so decisions later in the
    study window can use primary-cache bars as lookback.
    Returns list of closes in chronological order (oldest first).
    Returns [] if fewer than n bars are available.
    """
    fname = instrument.replace("&M", "ANDM").replace(".", "_") + ".json"

    closes = []

    # 1. history cache (2026-06-01 to 2026-07-13)
    hist_path = HIST_CACHE / fname
    if hist_path.exists():
        data = json.loads(hist_path.read_text())
        for b in data.get("bars", []):
            if b["date"] < before_date:
                closes.append((b["date"], b["close"]))

    # 2. primary cache bars that are also before before_date
    prim_path = PRIMARY_CACHE / fname
    if prim_path.exists():
        data = json.loads(prim_path.read_text())
        for b in data.get("bars", []):
            if b["date"] < before_date:
                closes.append((b["date"], b["close"]))

    # sort and deduplicate by date
    seen = set()
    unique = []
    for d, c in sorted(closes):
        if d not in seen:
            seen.add(d)
            unique.append(c)

    if len(unique) < n:
        return []   # insufficient history — caller must skip
    return unique[-n:]   # most recent n closes before decision


def momentum_direction(closes: list[float]) -> str:
    """
    20-session MA crossover: if last close > MA(20) → LONG, else SHORT.
    """
    ma = sum(closes) / len(closes)
    return "LONG" if closes[-1] > ma else "SHORT"


# ── main ──────────────────────────────────────────────────────────────────────

def main():
    print("=" * 70)
    print("HDV-001-F BASELINE COMPARISON RUNNER (CORRECTED)")
    print("=" * 70)

    # load outcomes
    outcomes_raw = json.loads(OUTCOMES.read_text())
    metrics_raw  = json.loads(METRICS.read_text())

    # index metrics by decision_id for price-path access
    metrics_by_id = {m["decision_id"]: m for m in metrics_raw["metrics"]}

    # filter to COMPLETE decisions only
    complete = [r for r in outcomes_raw["outcomes"]
                if r["observation_status"] == "COMPLETE"]
    print(f"COMPLETE decisions: {len(complete)}")

    # Coralys baseline
    coralys_target = sum(1 for r in complete if r["outcome"] == "TARGET_BEFORE_RISK")
    coralys_rate   = coralys_target / len(complete) * 100
    print(f"Coralys TARGET_BEFORE_RISK: {coralys_target}/{len(complete)} = {coralys_rate:.1f}%")
    print()

    rng = random.Random(RANDOM_SEED)

    # ── Baseline A: Random ────────────────────────────────────────────────────
    print("Running Baseline A (Random)...")
    a_target = a_stop = 0
    a_records = []
    for rec in complete:
        m = metrics_by_id[rec["decision_id"]]
        direction = rng.choice(["LONG", "SHORT"])
        ref   = m["reference_price"]
        tgt_p, stp_p = directional_boundaries(
            direction, ref, m["target_price"], m["stop_price"]
        )
        path = load_price_path(rec["instrument"], rec["decision_date_ist"])
        result = evaluate_outcome(direction, ref, tgt_p, stp_p, path)
        if result == "TARGET_HIT":
            a_target += 1
        else:
            a_stop += 1
        a_records.append({"decision_id": rec["decision_id"],
                          "direction": direction, "outcome": result})
    a_rate = a_target / len(complete) * 100
    print(f"  Baseline A TARGET_HIT: {a_target}/{len(complete)} = {a_rate:.1f}%")

    # ── Baseline B: Inverse ───────────────────────────────────────────────────
    print("Running Baseline B (Inverse)...")
    b_target = b_stop = 0
    b_records = []
    for rec in complete:
        m = metrics_by_id[rec["decision_id"]]
        direction = "SHORT" if rec["direction"] == "LONG" else "LONG"
        ref   = m["reference_price"]
        tgt_p, stp_p = directional_boundaries(
            direction, ref, m["target_price"], m["stop_price"]
        )
        path = load_price_path(rec["instrument"], rec["decision_date_ist"])
        result = evaluate_outcome(direction, ref, tgt_p, stp_p, path)
        if result == "TARGET_HIT":
            b_target += 1
        else:
            b_stop += 1
        b_records.append({"decision_id": rec["decision_id"],
                          "direction": direction, "outcome": result})
    b_rate = b_target / len(complete) * 100
    print(f"  Baseline B TARGET_HIT: {b_target}/{len(complete)} = {b_rate:.1f}%")

    # ── Baseline C: Momentum ──────────────────────────────────────────────────
    print("Running Baseline C (Momentum, MA-20)...")
    c_target = c_stop = c_skip = 0
    c_records = []
    for rec in complete:
        m = metrics_by_id[rec["decision_id"]]
        closes = load_history_closes(rec["instrument"], rec["decision_date_ist"], MA_PERIOD)
        if not closes:
            c_skip += 1
            c_records.append({"decision_id": rec["decision_id"],
                              "direction": None, "outcome": "SKIPPED"})
            continue
        direction = momentum_direction(closes)
        ref   = m["reference_price"]
        tgt_p, stp_p = directional_boundaries(
            direction, ref, m["target_price"], m["stop_price"]
        )
        path = load_price_path(rec["instrument"], rec["decision_date_ist"])
        result = evaluate_outcome(direction, ref, tgt_p, stp_p, path)
        if result == "TARGET_HIT":
            c_target += 1
        else:
            c_stop += 1
        c_records.append({"decision_id": rec["decision_id"],
                          "direction": direction, "outcome": result})

    c_eligible = len(complete) - c_skip
    c_rate = (c_target / c_eligible * 100) if c_eligible > 0 else 0.0
    print(f"  Baseline C TARGET_HIT: {c_target}/{c_eligible} eligible = {c_rate:.1f}%  (skipped: {c_skip})")

    # ── Success criterion ─────────────────────────────────────────────────────
    print()
    print("=" * 70)
    print("SUCCESS CRITERION (HDV-001-G Gate 6)")
    print("Coralys must exceed all three baselines by >= 5 pp")
    print("=" * 70)
    margins = {
        "A_Random":   coralys_rate - a_rate,
        "B_Inverse":  coralys_rate - b_rate,
        "C_Momentum": coralys_rate - c_rate,
    }
    all_pass = True
    for name, margin in margins.items():
        status = "PASS" if margin >= 5.0 else "FAIL"
        if status == "FAIL":
            all_pass = False
        print(f"  vs {name}: margin = {margin:+.1f} pp  [{status}]")

    overall = "PASS" if all_pass else "FAIL"
    print(f"\nOverall HDV-001-F: {overall}")

    # ── persist results ───────────────────────────────────────────────────────
    result_doc = {
        "version":    "hdv001_baseline_results_v2",
        "run_at":     datetime.now(timezone.utc).isoformat(),
        "random_seed": RANDOM_SEED,
        "ma_period":  MA_PERIOD,
        "n_complete": len(complete),
        "coralys": {
            "target_hit": coralys_target,
            "rate_pct":   round(coralys_rate, 2),
        },
        "baseline_A_random": {
            "target_hit": a_target,
            "n":          len(complete),
            "rate_pct":   round(a_rate, 2),
            "margin_vs_coralys_pp": round(margins["A_Random"], 2),
            "criterion_pass": margins["A_Random"] >= 5.0,
        },
        "baseline_B_inverse": {
            "target_hit": b_target,
            "n":          len(complete),
            "rate_pct":   round(b_rate, 2),
            "margin_vs_coralys_pp": round(margins["B_Inverse"], 2),
            "criterion_pass": margins["B_Inverse"] >= 5.0,
        },
        "baseline_C_momentum": {
            "target_hit": c_target,
            "n_eligible": c_eligible,
            "n_skipped":  c_skip,
            "rate_pct":   round(c_rate, 2),
            "margin_vs_coralys_pp": round(margins["C_Momentum"], 2),
            "criterion_pass": margins["C_Momentum"] >= 5.0,
        },
        "overall_pass": all_pass,
        "records_A": a_records,
        "records_B": b_records,
        "records_C": c_records,
    }
    OUT_JSON.write_text(json.dumps(result_doc, indent=2))
    print(f"\nResults written: {OUT_JSON.relative_to(WORKSPACE)}")

    # ── report ────────────────────────────────────────────────────────────────
    report = f"""# HDV-001-F Baseline Comparison Report (v2 — Corrected Run)

**Run date:** {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}
**Status:** {'PASS' if all_pass else 'FAIL'}
**Corrections applied:** Rule A (directional geometry) + Rule B (momentum history)

---

## Study Population

| Metric | Value |
|--------|-------|
| Total decisions | 1144 |
| COMPLETE (resolved) | {len(complete)} |
| MATURING (excluded) | {1144 - len(complete)} |

---

## Results

| Baseline | TARGET_HIT | N | Rate | Margin vs Coralys | Criterion (>=5 pp) |
|----------|-----------|---|------|-------------------|-------------------|
| **Coralys** | {coralys_target} | {len(complete)} | **{coralys_rate:.1f}%** | — | — |
| A — Random | {a_target} | {len(complete)} | {a_rate:.1f}% | {margins['A_Random']:+.1f} pp | {'PASS' if margins['A_Random'] >= 5.0 else 'FAIL'} |
| B — Inverse | {b_target} | {len(complete)} | {b_rate:.1f}% | {margins['B_Inverse']:+.1f} pp | {'PASS' if margins['B_Inverse'] >= 5.0 else 'FAIL'} |
| C — Momentum | {c_target} | {c_eligible} eligible | {c_rate:.1f}% | {margins['C_Momentum']:+.1f} pp | {'PASS' if margins['C_Momentum'] >= 5.0 else 'FAIL'} |

*Baseline C: {c_skip} decisions skipped (insufficient 20-session history — Rule B, zero random fallback)*

---

## Methodology Notes

**Rule A — Directional geometry (corrected):**
For each baseline decision, target and stop prices are reconstructed from
`reference_price +/- |original_distance|` in the baseline direction. Coralys's
absolute target/stop prices are never reused for a direction-flipped baseline.

**Rule B — Momentum history (corrected):**
Baseline C uses `hdv001_baseline_history_v1` (2026-06-01 to 2026-07-13) combined
with primary cache bars as pre-decision lookback. Decisions with fewer than 20
sessions of history are skipped entirely. Zero random fallback.

**Price path evaluation:**
Bar-by-bar walk using high/low (not close-only) to detect target/stop crossing.

---

## Conclusion

{'Coralys exceeds all three mechanical baselines by >= 5 percentage points. HDV-001-F criterion: PASS.' if all_pass else 'Coralys does not exceed all three mechanical baselines by >= 5 percentage points. HDV-001-F criterion: FAIL.'}
"""
    OUT_REPORT.write_text(report)
    print(f"Report written:  {OUT_REPORT.relative_to(WORKSPACE)}")

    sys.exit(0 if all_pass else 1)


if __name__ == "__main__":
    main()