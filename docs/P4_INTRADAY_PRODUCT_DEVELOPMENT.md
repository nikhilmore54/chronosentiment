# P4 Track B — Intraday Product Development

**Date:** 2026-09-12  
**Status:** ACTIVE — product development stream  
**Mode:** Build → backtest → observe → improve

---

## 1. Product Objective

Turn the existing intraday data and infrastructure into a **useful decision capability** for the user.

The product question is:

> **Given a current decision, what should the user do over the next 15 minutes to 5 hours?**

The output is an actionable recommendation — ENTER / WAIT / AVOID — with expected path behaviour, not raw model scores.

This is **not** a continuation of P4 hypothesis testing. B.1/B.2/B.3 are permanently frozen as historical artifacts. This stream is unconstrained by their feature boundaries.

---

## 2. What P4 Already Told Us

| Observation | Implication |
|---|---|
| All Watch H300 median ≈ +0.014% | Watch selection alone has little intraday value |
| SHORT Watch H300 median ≈ +0.492% | Direction selection is doing substantial work |
| B.2 (Exact) H300 median +0.515% | Exact evidence adds very little beyond SHORT Watch |
| B.3 (sample 51–150) H300 median +0.489% | Sample-size band adds essentially nothing |
| B.3 H15 +0.174% → H300 +0.489% | The effect develops over time — not a pure H15 phenomenon |
| B.1 H60 median +0.036% | Q3 target-rate selection did not translate into H60 edge |
| Zero >5% extreme moves in walk-forward | Extreme-move prediction not demonstrated by these candidates |
| Winners MFE median +2.452% vs Losers +0.289% | Path behaviour (MFE) is a strong separator — more useful than terminal return alone |

The interesting question is **not** which existing filter to try next. It is:

> **What information available at decision time explains why some SHORT Watch decisions subsequently outperform other SHORT Watch decisions at H15/H60/H300?**

---

## 3. Available Intraday Data

| Resolution | Location | Coverage |
|---|---|---|
| 5m | `intraday_capture/yahoo_cache_5m/` | Jun 22 – Sep 11 2026, 102 tickers |
| 1m | (to be confirmed) | timing/path refinement |
| 15m | (to be confirmed) | robustness/slower confirmation |

**Primary development resolution: 5m**  
Rationale: best balance between granularity and coverage. 1m for timing refinement. 15m for robustness checks.

**Horizons:** H15 (3 bars), H30 (6), H60 (12), H120 (24), H180 (36), H300 (60)

**Directions:** LONG and SHORT — no constraint to SHORT only

---

## 4. Unified Intraday Measurement Layer

Before building recommendations, build reliable measurement.

For every decision, compute at each horizon (H15/H30/H60/H120/H180/H300):

### Terminal return
Direction-adjusted close-to-close return from entry bar to horizon bar.

### Path behaviour
- **MFE** — maximum favourable excursion over horizon bars
- **MAE** — maximum adverse excursion over horizon bars
- **Time to MFE** — bar index at which MFE was achieved
- **Time to MAE** — bar index at which MAE was achieved
- **Early positive movement** — return at H15 (first 15 min)
- **Recovery behaviour** — did price recover after early adverse move?

### Opportunity quality signals
- Does the decision move favourably within the first 3 bars (H15)?
- Does MFE exceed 1% within H60?
- Does the decision reach its adaptive_target within H300?

This measurement layer is the foundation. Everything else is built on top of it.

---

## 5. Four Development Dimensions

### Dimension 1 — Timing

Does the edge emerge at a specific horizon rather than uniformly?

Test all horizons independently:
- H15, H30, H60, H120, H180, H300
- For LONG and SHORT separately
- Do not assume the locked P4 horizons (H15/H60/H300) are optimal

### Dimension 2 — Decision-time information

Use the full allowed information set to explain intraday outperformance:

- `rank_score`
- `target_rate`
- `sample_size`
- `rr_ratio` (note: broken for SHORT in current schema — fix before using)
- `expected_return`
- `evidence_class`
- `degradation_level`
- `vol_regime`
- `volume_regime`
- `certification_status`
- `adaptive_horizon_sessions`
- `direction`

Test **joint relationships**, not just individual quartiles. A decision with high rank_score AND high target_rate may behave differently from one with high rank_score alone.

### Dimension 3 — Path behaviour

MFE/MAE are more informative than terminal return alone.

Key questions:
- Which decision-time features predict early favourable movement (H15 MFE)?
- Which features predict persistence through H300?
- Can we identify decisions that move favourably early and then reverse?
- Can we identify decisions that start adversely but recover?

This could reveal that a decision has product value even when the final H300 return is modest.

### Dimension 4 — SHORT vs LONG asymmetry

P4 showed a very strong directional asymmetry (SHORT Watch H300 median +0.492% vs All Watch +0.014%). Investigate whether the decision-time features that distinguish good SHORT trades are fundamentally different from those for LONG trades. Do not assume the same policy works in both directions.

---

## 6. Product Output Format

The target product output for each opportunity:

```
INTRADAY DECISION
────────────────────────────────────
Ticker:     ADANIPORTS_NS
Direction:  SHORT
Entry:      ₹1,247.50

Expected path:
  H15 (15 min):  -0.3% to -0.8%  [favourable]
  H60 (1 hr):    -0.8% to -1.5%  [favourable]
  H300 (5 hr):   -1.2% to -2.1%  [favourable]

Favourable excursion (MFE):  +1.8% expected within H60
Adverse excursion (MAE):     -0.4% expected maximum

Confidence:  MODERATE
Rank score:  0.47

Recommended action:  ENTER
Why:  Strong SHORT signal, rank_score above threshold,
      expected early favourable movement within H15
────────────────────────────────────
```

This is the product target. The development loop determines what actually works.

---

## 7. Development Loop

```
DISCOVER
    ↓
RANK (opportunity quality)
    ↓
RECOMMEND (ENTER / WAIT / AVOID)
    ↓
SIMULATE (historical backtest)
    ↓
MEASURE (return, MFE, MAE, path)
    ↓
IMPROVE
    ↓
repeat
```

Each cycle should be fast. Build something, test it historically, observe what it does, keep or kill.

**No lengthy approval cycles. No elaborate research gates.**

---

## 8. Backtesting Environment

Use the existing infrastructure:

- **Ledger entries:** `live_capture/ledger/entries/` — decision-time information
- **TIME-009 observations:** `time_machine/analysis/TIME009/observations/` — daily outcomes
- **5m intraday cache:** `intraday_capture/yahoo_cache_5m/` — intraday paths
- **Development set:** Aug 20-21 (300 COMPLETE decisions)
- **Walk-forward set:** Aug 24+ (870 entries, 656 COMPLETE)

The backtesting script [`scripts/p4_track_b_intraday.py`](scripts/p4_track_b_intraday.py) is the starting point. Extend it rather than rewriting from scratch.

---

## 9. Product Metrics

Success is measured by product usefulness, not statistical significance alone:

| Metric | Description |
|---|---|
| Terminal return at horizon | Direction-adjusted return at H15/H60/H300 |
| MFE | How far in-the-money did the decision go? |
| MAE | How far out-of-the-money did it go? |
| Win rate | % of decisions with positive terminal return |
| Recommendation accuracy | % of ENTER recommendations that were profitable |
| Opportunity ranking quality | Do top-ranked decisions outperform bottom-ranked? |
| Trade frequency | How many actionable opportunities per cohort? |
| Worst outcome | What is the tail risk? |

---

## 10. Current Best Behaviour (Baseline)

From P4 walk-forward (Aug 24+ cohort, 5m intraday):

| Population | N | H15 median | H60 median | H300 median | H300 win rate |
|---|---|---|---|---|---|
| SHORT Watch (all) | 126 | +0.187% | +0.245% | +0.492% | 69.0% |
| LONG Watch (all) | 177 | -0.080% | -0.014% | +0.014% | 50.5% |

**The baseline to beat:** SHORT Watch H300 median +0.492%, win rate 69.0%.

Any intraday product capability must demonstrate improvement over this baseline to be worth building.

---

## 11. Next Implementation Experiment

**Phase 1 — Unified measurement (immediate):**

1. Extend [`scripts/p4_track_b_intraday.py`](scripts/p4_track_b_intraday.py) to output per-decision intraday records (not just aggregate tables)
2. Join with ledger decision-time fields
3. Produce a flat dataset: one row per decision × horizon, with all decision-time fields and all intraday outcome fields
4. This dataset is the foundation for all subsequent product experiments

**Phase 2 — Opportunity ranking view:**

1. For each cohort, rank SHORT Watch decisions by expected intraday quality
2. Compare top-ranked vs bottom-ranked at H60 and H300
3. Identify which decision-time features drive the ranking
4. Produce first opportunity-ranking output

**Phase 3 — Recommendation layer:**

1. Define ENTER / WAIT / AVOID thresholds from Phase 2 findings
2. Backtest recommendation accuracy historically
3. Integrate into Decision Cockpit

---

## 12. Constraints

- **B.1/B.2/B.3 are permanently frozen historical artifacts.** Track B product development does not modify or extend them.
- **No new locked candidates until a demonstrated discovery is found.** Discovery → development test → lock → prospective validation.
- **No hindsight.** All features used in recommendations must be available at decision time.
- **P4 historical artifacts are immutable.** Track A PASS, B.1 RETIRE, B.2/B.3 PASS no edge — permanent record.
- **Operating principle:** Build → backtest → observe → improve. Not: hypothesize → validate → publish.