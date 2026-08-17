# Historical Decision Validation v1 (HDV-001)
## Methodology — Frozen Before Implementation

**Status:** FROZEN  
**Date:** 2026-08-17  
**Depends on:** Coralys Decision Intelligence v0.1 (MVP-001 → MVP-010, 69/69 tests)  
**C3-002 artifact hash:** `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`  
**Coralys execution artifact hash:** `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

---

## 1. Research Contract

**Input:** Frozen C3-002 decisions materialized as `DecisionRecord`s via `DecisionRecordBuilder`.  
**Data:** Historically available market data (OHLCV, adjusted for corporate actions).  
**Unit of analysis:** One Coralys `DecisionRecord`.  
**Output:** Decision-level evidence — MAE, MFE, outcome — for every certified decision.

This is **not** a portfolio replay. There is no capital allocation, no position sizing,
no EqualWeight, no MaxPerLot, no user execution behaviour, no ranking, no confidence score.

The question being answered is:

> **Does a frozen Coralys decision have measurable value when evaluated against the
> subsequent price path, independent of how capital is allocated or how the user executes?**

---

## 2. Explicitly Excluded

The following must not appear anywhere in HDV-001:

- Portfolio allocation (EqualWeight, MaxPerLot, or any variant)
- Capital constraints or position sizing
- User execution behaviour or execution timing
- Ranking of decisions by any score
- Confidence or probability fields
- Stop-loss optimisation or modification
- Modification of C3-002 based on validation results
- Modification of the Coralys execution artifact based on validation results

If any of the above appear in the implementation, the validation is invalid.

---

## 3. Temporal Correctness

**Rule:** The price path used to evaluate a decision must begin **strictly after**
`decision_timestamp`. No bar that closes at or before `decision_timestamp` may be used
to compute MAE, MFE, or outcome.

**Rationale:** The temporal firewall that governs `DecisionRecord` certification must
also govern the evaluation. A decision evaluated against data it could have seen is not
a validation — it is a refit.

**Implementation requirement:** The first evaluation bar must have `bar_open_time > decision_timestamp`.

---

## 4. Corporate-Action Correctness

**Rule:** All price series must be adjusted for dividends and splits using the
**back-adjusted** method, applied consistently across the entire historical universe.

**Rationale:** Unadjusted prices produce spurious MAE/MFE spikes at corporate-action
dates that are not real adverse/favorable excursions.

**Implementation requirement:** The adjustment must be applied before any MAE/MFE
calculation. The adjustment factor must be recorded in the dataset metadata.

---

## 5. Survivorship Treatment

**Rule:** The historical universe must include instruments that were delisted, suspended,
or merged during the evaluation period. Decisions on such instruments must be evaluated
up to the last available bar and then closed with `OutcomeStatus::Horizon`.

**Rationale:** Excluding delisted instruments introduces survivorship bias that
systematically overstates favorable outcomes.

**Implementation requirement:** The dataset must record `survivorship_status` per
instrument: `ACTIVE`, `DELISTED`, `SUSPENDED`, `MERGED`.

---

## 6. Historical Universe Definition

The historical universe is defined as:

- All instruments for which C3-002 produced at least one certified decision in the
  development period.
- Instruments must have been listed and actively traded at `decision_timestamp`.
- Minimum liquidity threshold: average daily volume ≥ 100,000 shares over the 20
  sessions preceding `decision_timestamp`.

The universe definition is frozen at the start of each period (development, validation,
holdout) and must not be modified based on results.

---

## 7. Decision-Time Information Boundary

**Rule:** No information that post-dates `decision_timestamp` may be used to select,
filter, or weight decisions in the evaluation.

This means:

- No filtering decisions by subsequent outcome.
- No filtering decisions by subsequent volatility.
- No filtering decisions by subsequent corporate actions.
- No filtering decisions by subsequent liquidity.

All decisions that were certified at `decision_timestamp` are included in the evaluation,
regardless of what subsequently happened.

---

## 8. Development / Validation / Holdout Periods

Three strictly non-overlapping historical periods:

| Period      | Purpose                                      | Status at freeze |
|-------------|----------------------------------------------|------------------|
| Development | Methodology development and debugging        | Open             |
| Validation  | Frozen methodology applied to new data       | Sealed           |
| Holdout     | Final credibility check — untouched until v1 complete | Sealed   |

**The holdout period must not be examined until the development and validation
methodology is fully frozen and the validation results are recorded.**

Specific date ranges are to be defined in `HDV_001_PERIODS.md` before implementation
begins. They must not be changed after the first development run.

---

## 9. Decision-Level Outcome Definitions

For each `DecisionRecord`, the following outcomes are defined:

| Outcome          | Definition                                                                 |
|------------------|----------------------------------------------------------------------------|
| `TARGET`         | The price path reached `target_price` before `reference_risk_boundary_price` or horizon. |
| `REFERENCE_RISK` | The price path reached `reference_risk_boundary_price` before `target_price` or horizon. |
| `HORIZON`        | Neither target nor reference risk was reached within the observation window. |

**Observation window:** 10 trading sessions from `decision_timestamp` (inclusive of
session 1, exclusive of session 0). This is the default; it may be varied in sensitivity
analysis but the primary result uses 10 sessions.

**Direction convention:**
- `LONG`: target is above entry; reference risk is below entry.
- `SHORT`: target is below entry; reference risk is above entry.

**Price used:** Intraday high/low for MAE/MFE; close for session-level outcome snapshots.

---

## 10. MAE and MFE Definitions

**Maximum Adverse Excursion (MAE):**  
The worst intraday price move against the decision direction, measured from the
decision-time reference price (close of `decision_timestamp` session), expressed as
a percentage.

```
LONG  MAE = (min(low over observation window) - reference_price) / reference_price * 100
SHORT MAE = (reference_price - max(high over observation window)) / reference_price * 100
```

MAE is always ≤ 0 for a correctly directional decision.

**Maximum Favorable Excursion (MFE):**  
The best intraday price move in the decision direction, measured from the
decision-time reference price, expressed as a percentage.

```
LONG  MFE = (max(high over observation window) - reference_price) / reference_price * 100
SHORT MFE = (reference_price - min(low over observation window)) / reference_price * 100
```

MFE is always ≥ 0 for a correctly directional decision.

**Reference price:** Close of the session at `decision_timestamp`. If the decision is
intraday, the reference price is the last available close before `decision_timestamp`.

---

## 11. Regime Segmentation

Decisions are segmented by the Coralys-certified state at `decision_timestamp`:

| Dimension  | Values                        |
|------------|-------------------------------|
| Trend      | Bullish / Bearish / Neutral   |
| Momentum   | Positive / Negative / Neutral |
| Volatility | present / absent              |

Additionally, an **independent calendar/market regime** is defined based on:
- Nifty 50 trend (20-session EMA direction) at `decision_timestamp`
- VIX India level at `decision_timestamp` (if available)

The independent regime must be defined without reference to C3-002 output.

---

## 12. Independent Baseline Strategies

To establish whether Coralys decisions have value above a naive baseline, two
independent baselines are evaluated over the same historical universe and periods:

| Baseline   | Definition                                                              |
|------------|-------------------------------------------------------------------------|
| Random     | Random direction (50/50 LONG/SHORT) for every instrument on every session. |
| Momentum   | Direction determined by 20-session price momentum, independent of C3-002. |

Both baselines use the same target, reference risk, and observation window definitions
as the Coralys decisions. They are evaluated on the same instruments and sessions.

---

## 13. Predefined Success Criteria

The following criteria are defined before any data is examined. The validation is
considered **positive** if all primary criteria are met on the validation period.
The holdout is examined only if the validation is positive.

### Primary criteria (all must pass)

| Criterion                        | Threshold                        |
|----------------------------------|----------------------------------|
| Target rate (LONG decisions)     | > Random baseline target rate    |
| Target rate (SHORT decisions)    | > Random baseline target rate    |
| Median MFE (LONG)                | > Median MAE absolute value (LONG) |
| Median MFE (SHORT)               | > Median MAE absolute value (SHORT) |
| P90 MAE (LONG)                   | < reference_risk_boundary distance |
| P90 MAE (SHORT)                  | < reference_risk_boundary distance |

### Secondary criteria (informational, not pass/fail)

| Criterion                        | Measurement                      |
|----------------------------------|----------------------------------|
| Regime stability                 | Target rate variance across regimes |
| Holdout consistency              | Target rate within ±5pp of validation |
| Worst-regime performance         | Target rate in worst regime > 40% |

---

## 14. What This Validation Does Not Prove

Even if all primary criteria pass, HDV-001 does **not** prove:

- That any particular capital allocation strategy is optimal.
- That the reference risk boundary is the correct stop level.
- That the observation window of 10 sessions is optimal.
- That C3-002 should be modified.
- That the Coralys execution artifact should be modified.
- That the product is ready for autonomous execution.

Those are separate research questions, each requiring their own frozen methodology.

---

## 15. Artefact Integrity

The following artefacts must be frozen and hash-verified before HDV-001 begins:

| Artefact                        | Hash field in DecisionRecord          |
|---------------------------------|---------------------------------------|
| C3-002 policy artifact          | `certification.policy_artifact_hash`  |
| Coralys execution artifact      | `certification.execution_artifact_hash` |
| Market data snapshot            | `certification.data_snapshot_id`      |

The validation dataset must record the hash of the market data used for each decision's
price path evaluation. This allows independent reproduction of every MAE/MFE result.

---

## 16. Implementation Sequence

```
HDV-001-A  Freeze period definitions (HDV_001_PERIODS.md)
HDV-001-B  Build price path extractor (decision_timestamp → N sessions)
HDV-001-C  Implement MAE/MFE calculator (corporate-action adjusted)
HDV-001-D  Implement outcome classifier (TARGET / REFERENCE_RISK / HORIZON)
HDV-001-E  Build baseline strategies (Random, Momentum)
HDV-001-F  Run development period — debug only, no result recording
HDV-001-G  Freeze implementation — no changes after this point
HDV-001-H  Run validation period — record results
HDV-001-I  Evaluate against predefined success criteria
HDV-001-J  If positive: run holdout period
HDV-001-K  Write final report
```

**HDV-001-G is a hard gate.** No implementation changes are permitted after it.
Any bug found after HDV-001-G requires a new methodology version (HDV-002).

---

## 17. Relationship to v0.1 Product

HDV-001 is a **research programme**, not a product feature.

Its outputs feed the `EvidenceRecord` fields in `DecisionRecord`:

```rust
pub struct EvidenceRecord {
    pub similar_decisions_count: Option<u32>,
    pub historical_target_rate: Option<f64>,
    pub median_mae_pct: Option<f64>,
    pub p90_mae_pct: Option<f64>,
    pub median_mfe_pct: Option<f64>,
    pub median_time_to_target_sessions: Option<f64>,
}
```

These fields remain `None` in v0.1. They are populated only after HDV-001 produces
validated, holdout-confirmed evidence.

The product schema does not change. The research layer enriches it.

---

*This document is frozen. Modifications require a new version (HDV-002-METHODOLOGY.md).*