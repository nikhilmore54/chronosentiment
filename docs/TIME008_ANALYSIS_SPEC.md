# TIME-008 — Discrimination Analysis Specification

**Status:** FROZEN PRE-ANALYSIS  
**Frozen at:** 2026-08-20 (before examining aggregate_evidence.csv results)  
**Input dataset:** `time_machine/cohorts/aggregate_evidence.csv` (612 rows, 6 cohorts × 102 decisions)  
**Analyst:** ChronoSentiment research pipeline  

---

## Governing rule

> TIME-008 is analysis-only. No changes to Coralys, decision replay, evidence classification, target-rate calculation, adaptive R:R, action policy, observation logic, cohort selection, or eligibility rules are permitted as a result of TIME-008 findings.

TIME-008 consumes the frozen 612-row dataset and produces analysis artifacts. It does not modify the experiment that generated the evidence.

---

## Input

```
time_machine/cohorts/aggregate_evidence.csv
```

Columns of interest:

**Cohort dimension**
- `cohort_label` — T1–T6
- `as_of_cohort` — ISO timestamp of the historical T

**T0 evidence fields (frozen at decision time)**
- `ticker`
- `direction` — LONG | SHORT
- `action` — Buy | Sell | Watch | NoTrade
- `evidence_class` — Favourable | Mixed | Unfavourable | Insufficient
- `target_rate`
- `sample_size`
- `degradation_level`
- `adaptive_rr`
- `adaptive_horizon_sessions`
- `reference_price`
- `adaptive_target`
- `adaptive_risk`

**T+h outcome fields (observed after T)**
- `exit_reason` — TARGET | RISK | HORIZON | NO_TRADE
- `sessions_to_outcome`
- `target_reached` — bool
- `risk_reached` — bool
- `horizon_reached` — bool
- `actual_mfe` — maximum favourable excursion (fractional)
- `actual_mae` — maximum adverse excursion (fractional)
- `realized_return` — fractional return at exit
- `eligible_for_primary_comparison` — bool

---

## Pre-specified outcome variables

The following outcome variables are pre-specified. No additional outcome variables may be introduced post-hoc.

1. `target_reached` (binary) — primary
2. `risk_reached` (binary) — primary
3. `realized_return` (continuous) — primary
4. `actual_mfe` (continuous) — secondary
5. `actual_mae` (continuous) — secondary
6. `exit_reason` (categorical) — secondary

---

## Pre-specified analysis questions

### Q1 — Evidence-class discrimination

**Question:** Does evidence_class (Favourable / Mixed / Unfavourable) correspond to materially different forward outcomes?

**Analysis:**
- For each evidence_class level: mean(target_reached), mean(risk_reached), mean(realized_return)
- Distribution of exit_reason by evidence_class
- Pooled across all cohorts
- Cohort-by-cohort breakdown (T1–T6 separately)
- Consistency check: does the ordering hold across cohorts?

**Eligibility filter:** `eligible_for_primary_comparison == true` for primary analysis; full population for secondary.

**Claim threshold:** A pattern is noted as "consistent" only if the ordering holds in ≥4 of 6 cohorts. A pattern is noted as "inconsistent" if it reverses in ≥3 cohorts.

---

### Q2 — Direction asymmetry

**Question:** Does discrimination differ between LONG and SHORT decisions?

**Analysis:**
- Repeat Q1 stratified by direction (LONG / SHORT)
- Compare: does evidence_class discriminate outcomes differently for LONG vs SHORT?
- Note any direction × evidence_class interaction

**Claim threshold:** Same as Q1 (≥4/6 cohorts for consistency).

---

### Q3 — Action vs underlying decision

**Question:** Does action (Buy / Sell / Watch / NoTrade) contain predictive information beyond evidence_class?

**Analysis:**
- Within each evidence_class level: compare outcomes by action
- Specifically: within Favourable, does Buy outperform Watch?
- Within Mixed, does Watch outperform NoTrade?
- Note: action and evidence_class are not identical variables; action incorporates direction + evidence + policy

**Claim threshold:** Same as Q1.

---

### Q4 — R:R interaction

**Question:** Does the relationship between T0 evidence and outcome depend on adaptive_rr?

**Analysis:**
- Bin adaptive_rr into low / medium / high terciles (pre-specified: terciles of the full 612-row distribution)
- Within each bin: repeat Q1 evidence-class discrimination
- Note whether discrimination is stronger or weaker at higher R:R configurations

**Claim threshold:** Same as Q1. This is exploratory; no threshold tuning is permitted.

---

## Cohort dimension

All four questions are analyzed:
1. Pooled (T1–T6 combined)
2. Cohort-by-cohort (T1, T2, T3, T4, T5, T6 separately)
3. Consistency summary: how many cohorts show the same directional pattern?

T1 is not treated as special. It is one member of {T1, T2, T3, T4, T5, T6}.

---

## What TIME-008 may conclude

TIME-008 may conclude:

- **Consistent pattern:** The ordering holds in ≥4/6 cohorts (descriptive finding)
- **Inconsistent pattern:** The ordering reverses in ≥3/6 cohorts (descriptive finding)
- **No detectable pattern:** Differences are small and inconsistent across cohorts

TIME-008 may NOT conclude:

- That Coralys has proven predictive power
- That the system is economically useful
- That any threshold should be changed
- That any algorithm should be modified
- That the evidence is sufficient for live deployment

---

## What TIME-008 does NOT do

- Does not modify any algorithm
- Does not select a better threshold retrospectively
- Does not remove poor-performing cohorts
- Does not add new outcome variables post-hoc
- Does not interpret a statistically significant relationship as proof of economic utility
- Does not use findings to change the TIME-002→TIME-006 pipeline

---

## Output artifacts

```
time_machine/analysis/TIME008/
├── q1_evidence_class_discrimination.json
├── q2_direction_asymmetry.json
├── q3_action_vs_decision.json
├── q4_rr_interaction.json
├── cohort_consistency_summary.json
├── analysis_report.md
└── latest_run.json
```

---

## Downstream

TIME-008 findings feed TIME-009 (prospective validation design) and TIME-010 (evidence package / research conclusion). Neither TIME-009 nor TIME-010 may begin until TIME-008 artifacts are frozen.

---

*This specification was written and committed before examining the aggregate_evidence.csv results. Any deviation from this specification must be documented as an amendment with justification.*