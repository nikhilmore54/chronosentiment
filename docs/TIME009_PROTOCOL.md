# TIME-009 — Prospective Validation Protocol

**Document ID:** TIME-009-PROTOCOL-v1.0
**Status:** FROZEN
**Frozen at:** 2026-08-20
**Frozen before first observation:** YES
**Supersedes:** nothing (first prospective protocol)
**Governed by:** governance-hardening branch, INDEX.md v1.97

---

## 1. Purpose

TIME-009 is the prospective validation phase of the ChronoSentiment research programme.

It answers the question:

> **What outcomes does the frozen Coralys/ChronoSentiment system produce when operating prospectively on genuinely unseen market data?**

TIME-009 does NOT:

- Reinterpret or reopen TIME-008.
- Modify Coralys, the recommendation engine, or any upstream pipeline component.
- Optimize thresholds, R:R parameters, or evidence classification rules.
- Cherry-pick cohorts or decisions.
- Extend or shorten the observation phase based on observed outcomes.

---

## 2. Governing constraints

These constraints are frozen before the first observation and cannot be changed during the TIME-009 observation phase.

1. **Algorithm freeze:** The Coralys C3-002 artifact hash `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121` and engine version `v1` are the frozen configuration. No changes to these artifacts are permitted during TIME-009.

2. **No retrospective modification:** Once a `TIME009-OBS-*.json` artifact is written, it is immutable. No field may be updated, corrected, or overwritten.

3. **No eligibility changes:** The eligibility rule (AC-T9-05) is frozen. It cannot be changed after the first observation is generated.

4. **No stopping rule changes:** The stopping rule (Section 4) is frozen. It cannot be changed after the first observation is generated.

5. **No cohort exclusions:** All LIVE-005 run dates within the observation window are included. No cohort may be excluded based on outcomes, eligibility counts, or operational failures.

6. **TIME-008 result does not modify this protocol:** The negative TIME-008 result is part of the research record. It does not change any TIME-009 rule.

---

## 3. Observation source and temporal integrity

### 3.1 T0 definition

T0 for each decision is the `admitted_at` timestamp in the LIVE-005 ledger entry.

The T0 artifact is the LIVE-005 ledger entry file at:
```
live_capture/ledger/entries/LIVE-005-{decision_id}.json
```

### 3.2 Observation source

TIME-009 observations are computed from **newly available OHLCV bars with timestamps strictly after `source_snapshot_timestamp`** from the T0 ledger entry.

The observation source is the Yahoo Finance API (same provider as LIVE-001), queried at observation time for bars in the range `(source_snapshot_timestamp, T0 + horizon]`.

**Temporal integrity invariant:** No bar with a timestamp ≤ `source_snapshot_timestamp` may be used to compute any outcome field. This invariant must be enforced by the `time009_observe` binary and verified in the artifact.

### 3.3 Distinction from historical cache

The historical Yahoo Finance cache used in TIME-002→TIME-007 (`live_capture/yahoo_cache/`) is NOT the observation source for TIME-009. TIME-009 observations must use bars fetched after T0, not bars from the historical snapshot that produced the T0 decision.

---

## 4. Stopping rule

**TIME-009 observation phase terminates at the earlier of:**

**(a)** 20 prospective cohort dates have been accumulated, OR

**(b)** 6 calendar weeks have elapsed from the first prospective cohort date.

**First cohort date:** The `admitted_at` date of the earliest LIVE-005 ledger entry included in TIME-009 (2026-08-20).

**Derived deadline:** 6 calendar weeks from 2026-08-20 = **2026-10-01**.

**Stopping criterion is evaluated on cohort/date existence, not on:**
- Number of favourable, actionable, eligible, or complete observations
- Statistical significance of any interim result
- Observed outcomes (positive or negative)
- Missing observation counts
- Operational failures

Once the stopping criterion is reached, the TIME-009 dataset is frozen and TIME-010 begins.

---

## 5. Acceptance criteria

### AC-T9-01 T0 immutability

The following fields are read verbatim from the LIVE-005 ledger entry and written into the observation artifact without modification:

- `decision_id`
- `certification_id`
- `recommendation_id`
- `source_snapshot_id`
- `source_snapshot_timestamp`
- `c3_002_artifact_hash`
- `engine_version`
- `ticker`
- `direction`
- `action`
- `evidence_class`
- `certification_status`
- `reference_price`
- `adaptive_target`
- `adaptive_risk`
- `adaptive_horizon_sessions`
- `admitted_at`

None of these fields may be recomputed, rounded, or altered by `time009_observe`.

### AC-T9-02 Temporal integrity

`observed_at` must be strictly after `admitted_at`.

No OHLCV bar with timestamp ≤ `source_snapshot_timestamp` may contribute to any outcome field (`target_reached`, `risk_reached`, `horizon_reached`, `exit_price`, `actual_mfe`, `actual_mae`, `realized_return`).

The artifact must record `first_eligible_bar_timestamp` (the timestamp of the first bar strictly after `source_snapshot_timestamp`) to make this verifiable.

### AC-T9-03 Horizon definition

The observation horizon is `adaptive_horizon_sessions` NSE trading sessions from the first eligible bar after T0.

- One NSE session = one trading day (09:15–15:30 IST, Monday–Friday, excluding NSE holidays).
- `adaptive_horizon_sessions` is read verbatim from T0. It is not recomputed, rounded, or altered.
- Partial sessions do not count toward the horizon.
- The horizon window is `[first_eligible_bar, first_eligible_bar + adaptive_horizon_sessions sessions]` inclusive.

### AC-T9-04 Outcome computation

Within the horizon window, the outcome is determined by whichever condition occurs first (scanning bars in chronological order):

**LONG decisions:**
- `target_reached = true` if any bar's `high` ≥ `adaptive_target`
- `risk_reached = true` if any bar's `low` ≤ `adaptive_risk`

**SHORT decisions:**
- `target_reached = true` if any bar's `low` ≤ `adaptive_target`
- `risk_reached = true` if any bar's `high` ≥ `adaptive_risk`

**Horizon:**
- `horizon_reached = true` if neither target nor risk is reached within `adaptive_horizon_sessions` sessions

If both target and risk are reached in the same bar, `target_reached` takes precedence (consistent with TIME-005 upstream rule).

`exit_price` is the bar's close price at the exit bar.
`realized_return` = `(exit_price - reference_price) / reference_price` for LONG; `(reference_price - exit_price) / reference_price` for SHORT.
`actual_mfe` = maximum favourable excursion from `reference_price` within the horizon.
`actual_mae` = maximum adverse excursion from `reference_price` within the horizon.

### AC-T9-05 Eligibility rule

`eligible_for_primary_comparison = true` if and only if ALL of:
- `certification_status` is CERTIFIED or DEGRADED
- `evidence_class` is Favourable or Mixed
- `observation_status` is COMPLETE

Unfavourable and Insufficient evidence classes are ineligible for primary comparison. This rule is consistent with the upstream TIME-008 eligibility rule and cannot be changed.

### AC-T9-06 Idempotency

Re-running `time009_observe` for a `decision_id` that already has a COMPLETE observation artifact must not overwrite it. The artifact is immutable once `observation_status = COMPLETE`.

PENDING artifacts (horizon not yet elapsed, or data unavailable) may be updated on subsequent runs.

### AC-T9-07 No algorithm changes

`time009_observe` performs:
- No C3-002 evaluation
- No recommendation recomputation
- No evidence reclassification
- No ranking or scoring

It reads T0 fields from the LIVE-005 ledger entry and observes market outcomes only.

### AC-T9-08 Provenance completeness

Every observation artifact must contain a `provenance_chain` field linking:
```
decision_id → certification_id → recommendation_id → source_snapshot_id → c3_002_artifact_hash
```

### AC-T9-09 Missing data handling

If OHLCV data for the required horizon is unavailable (market holiday, data gap, API failure, insufficient bars):
- `observation_status = PENDING`
- The artifact is written with available fields populated and missing fields null
- PENDING observations are NOT included in the TIME-009 dataset for primary analysis
- PENDING observations are NOT silently discarded — they are retained in the artifact directory with their status

A PENDING observation may transition to COMPLETE on a subsequent run if data becomes available, provided the horizon has not been exceeded.

### AC-T9-10 Cohort definition

Each LIVE-005 run date (the date portion of `admitted_at`, e.g. `20260820`) constitutes one prospective cohort.

Cohort membership is determined by `admitted_at` date, not observation date.

Multiple LIVE-005 runs on the same calendar date belong to the same cohort.

### AC-T9-11 DEGRADED inclusion

DEGRADED decisions (where `certification_status = DEGRADED`) are included in the TIME-009 dataset as a stratified secondary cohort, consistent with the LIVE-005 admission policy. They are not excluded from observation.

---

## 6. Artifact schema

### 6.1 Per-decision observation artifact

**Path:** `time_machine/analysis/TIME009/observations/TIME009-OBS-{decision_id}.json`

```json
{
  "observation_id": "TIME009-OBS-{decision_id}",
  "decision_id": "{from T0}",
  "certification_id": "{from T0}",
  "recommendation_id": "{from T0}",
  "source_snapshot_id": "{from T0}",
  "source_snapshot_timestamp": "{from T0}",
  "c3_002_artifact_hash": "{from T0}",
  "engine_version": "{from T0}",
  "producer": "time009_observe.v1",
  "observed_at": "<ISO8601 UTC>",
  "admitted_at": "{from T0}",
  "cohort_date": "YYYYMMDD",
  "ticker": "{from T0}",
  "direction": "{from T0}",
  "action": "{from T0}",
  "evidence_class": "{from T0}",
  "certification_status": "{from T0}",
  "reference_price": 0.0,
  "adaptive_target": 0.0,
  "adaptive_risk": 0.0,
  "adaptive_horizon_sessions": 0.0,
  "observation_status": "COMPLETE|PENDING",
  "first_eligible_bar_timestamp": "<unix or null>",
  "n_bars_in_horizon": 0,
  "exit_reason": "HORIZON|TARGET|RISK|null",
  "exit_bar_timestamp": "<unix or null>",
  "exit_price": 0.0,
  "sessions_to_outcome": 0,
  "target_reached": false,
  "risk_reached": false,
  "horizon_reached": false,
  "actual_mfe": 0.0,
  "actual_mae": 0.0,
  "realized_return": 0.0,
  "eligible_for_primary_comparison": false,
  "provenance_chain": {
    "t0_decision": "decision_id={} admitted_at={}",
    "t0_certification": "certification_id={} status={}",
    "t0_recommendation": "recommendation_id={}",
    "t0_snapshot": "source_snapshot_id={} source_snapshot_timestamp={}",
    "t0_algorithm": "c3_002_artifact_hash={} engine_version={}"
  }
}
```

### 6.2 Aggregate dataset

**Path:** `time_machine/analysis/TIME009/prospective_evidence.csv`

One row per COMPLETE observation. Columns mirror `time_machine/cohorts/aggregate_evidence.csv` (used in TIME-008) with additional columns:

| Additional column | Description |
|---|---|
| `cohort_date` | LIVE-005 run date (YYYYMMDD) |
| `certification_status` | CERTIFIED or DEGRADED |
| `observation_status` | COMPLETE (only COMPLETE rows in dataset) |
| `first_eligible_bar_timestamp` | Unix timestamp of first bar after T0 |

### 6.3 Run metadata

**Path:** `time_machine/analysis/TIME009/latest_run.json`

Records: `run_at`, `producer`, `n_decisions_processed`, `n_complete`, `n_pending`, `n_cohort_dates`, `first_cohort_date`, `stopping_rule`, `stopping_criterion_met`.

---

## 7. Implementation components

### 7.1 `time009_observe` binary

**Location:** `adapters/chronosentiment/src/bin/time009_observe.rs`

**CLI:**
```
time009_observe \
  --ledger      live_capture/ledger/ \
  --output      time_machine/analysis/TIME009/observations/ \
  --cache       live_capture/yahoo_cache \
  --run-meta    time_machine/analysis/TIME009/latest_run.json
```

**Behaviour:**
1. Scan all `live_capture/ledger/entries/LIVE-005-*.json` files.
2. For each entry, check if a COMPLETE observation artifact already exists (AC-T9-06).
3. If not, check if the horizon has elapsed (current time > `admitted_at` + `adaptive_horizon_sessions` sessions).
4. If horizon elapsed, fetch OHLCV bars from Yahoo Finance for the required window.
5. Enforce temporal integrity (AC-T9-02): discard any bar with timestamp ≤ `source_snapshot_timestamp`.
6. Compute outcomes (AC-T9-04).
7. Write artifact (AC-T9-01, AC-T9-08).

### 7.2 `time009_dataset` script

**Location:** `scripts/time009_dataset.py`

**CLI:**
```
python scripts/time009_dataset.py \
  --observations time_machine/analysis/TIME009/observations/ \
  --output       time_machine/analysis/TIME009/prospective_evidence.csv
```

**Behaviour:**
1. Read all `TIME009-OBS-*.json` files.
2. Filter to `observation_status = COMPLETE`.
3. Enforce schema invariants (no duplicate `observation_id`, no missing required fields).
4. Write CSV.

### 7.3 `start_backend.sh` integration

Add step 6 after LIVE-005:
```bash
echo "[backend] TIME-009: observing elapsed horizons..."
cargo run -p chronosentiment_adapter --bin time009_observe -- \
  --ledger   "$LIVE_LEDGER_DIR" \
  --output   "time_machine/analysis/TIME009/observations" \
  --cache    "$LIVE_YAHOO_CACHE" \
  --run-meta "time_machine/analysis/TIME009/latest_run.json"
echo "[backend] TIME-009 complete."
```

---

## 8. Primary endpoints for TIME-010

These endpoints are pre-specified. TIME-010 must evaluate them without modification.

**Primary (eligible rows only):**
- `target_reached` rate: Favourable vs Mixed
- `realized_return` mean: Favourable vs Mixed
- Cohort consistency: ≥4/N cohort dates show Favourable > Mixed ordering

**Secondary (full population):**
- `actual_mfe` mean by evidence_class
- `actual_mae` mean by evidence_class
- Direction stratification (LONG vs SHORT)

**Consistency threshold:** ≥4/N cohort dates (where N = number of cohort dates accumulated), consistent with TIME-008 pre-specified criterion.

**What TIME-010 may NOT do:**
- Change the primary endpoints after seeing the data.
- Exclude cohort dates based on outcomes.
- Apply retrospective R:R threshold selection.
- Claim three-class discrimination (Unfavourable is ineligible for primary comparison).
- Claim predictive or economic utility from a single prospective cohort.

---

## 9. Relationship to TIME-008

TIME-008 established:

> No consistent cross-cohort discrimination of forward outcomes by the frozen evidence classification was demonstrated in the historical experiment.

TIME-009 does not attempt to repair or reinterpret that result. It asks:

> Does the frozen system produce any observable prospective pattern when operating on genuinely unseen data?

The answer — whatever it is — will be reported in TIME-010 with the same discipline applied in TIME-008: pooled descriptive separation is distinguished from pre-specified cross-cohort consistency.

---

## 10. Version history

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-08-20 | Initial freeze — before first prospective observation |