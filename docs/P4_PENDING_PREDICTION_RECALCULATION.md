# P4 Pending Prediction Recalculation

**Date:** 2026-09-11  
**Status:** ROOT CAUSE IDENTIFIED — recalculation pending  
**Controlling artifact:** `docs/P4_WALKFORWARD_POLICY_LOCK.md` (frozen, not modified)

---

## 1. Original Pending Population

| Metric | Value |
|---|---|
| Total Aug 24+ PENDING decisions | 1,089 |
| LONG × NoTrade | 588 |
| SHORT × NoTrade | 501 |
| Watch decisions | **0** |
| SHORT Watch decisions | **0** |

### Cohort breakdown

| Cohort date | N decisions |
|---|---|
| 2026-08-24 | 70 |
| 2026-08-25 | 70 |
| 2026-08-26 | 70 |
| 2026-08-27 | 140 |
| 2026-08-28 | 70 |
| 2026-08-31 | 56 |
| 2026-09-01 | 56 |
| 2026-09-02 | 70 |
| 2026-09-03 | 70 |
| 2026-09-04 | 101 |
| 2026-09-07 | 79 |
| 2026-09-08 | 79 |
| 2026-09-09 | 79 |
| 2026-09-10 | 79 |
| **Total** | **1,089** |

---

## 2. NoTrade Root Cause

### Finding: Empty evidence store at prediction time

Every single Aug 24+ decision has `evidence_store_n_files: 0`.

| Field | Value across all 1,089 decisions |
|---|---|
| `evidence_store_n_files` | 0 (100%) |
| `degradation_level` | Insufficient (100%) |
| `sample_size` | 0 (100%) |
| `target_rate` | 0.0 (100%) |
| `evidence_class` | Insufficient (100%) |
| `rank_score` | 0.0 (100%) |
| `adaptive_target` | null (100%) |
| `adaptive_risk` | null (100%) |

### Two producer populations

**Population A — Backfill (N=656, 60.2%)**
- `producer: "backfill_missing_cohorts.v1"`
- `certification_status: STALE`
- `_backfill_note: "Retrospective admission from cache replay. Snapshot replayed at..."`
- These are retrospective cache replays that ran without evidence data loaded

**Population B — Live pipeline (N=433, 39.8%)**
- `producer: "live005_ledger.v1"`
- `certification_status: DEGRADED`
- `_backfill_note: none`
- These are live pipeline entries that also had empty evidence stores

### Root cause summary — corrected causal chain

```
Backend server OFF
       ↓
Prediction engine cannot access/populate evidence store
       ↓
evidence_store_n_files = 0
       ↓
No analogue evidence available
       ↓
Evidence degrades to Insufficient
       ↓
sample_size = 0, target_rate = 0.0
       ↓
action = NoTrade (100%)
       ↓
1,089 Aug 24+ decisions become NoTrade
       ↓
P4 walk-forward appears blocked
```

**Proximate symptom:** `evidence_store_n_files: 0`
**Actual root cause:** Backend server was unavailable during prediction execution. The empty evidence store was a downstream consequence of the unavailable backend, not an intrinsic absence of historical evidence.

This is an **operational dependency failure**, not a data infrastructure failure and not a market signal failure. The underlying market data, decision timestamps, and historical analogue evidence all exist. The prediction engine ran but could not reach the backend to load evidence.

**Engineering lesson:** The cohort runner should fail loudly (not silently produce valid-looking NoTrade records) when the backend/evidence service is unavailable. A preflight gate is required:

```
BACKEND CONNECTIVITY → EVIDENCE STORE ACCESSIBLE → EVIDENCE STORE NON-EMPTY
→ COHORT SNAPSHOT VALID → INFORMATION-SET BOUNDARY VALID → RUN PREDICTION
```

### What this means for P4

The 1,089 NoTrade decisions are not evidence that the market offered no Watch opportunities. They are evidence that the prediction engine ran without its evidence store. The original records must be preserved as the historical record of what was produced, but they cannot be used as the walk-forward evaluation population.

---

## 3. Recalculation Plan

### Objective

Re-run the prediction engine for each Aug 24+ cohort using:
- The original decision timestamp (source_snapshot_timestamp)
- The original market snapshot data
- The evidence store **properly loaded** with historical analogue data available at that time
- No future data (strict information-set boundary)

### Information-set boundary (CRITICAL)

For each cohort date D, the recalculation may only use:
- Market data available at the snapshot time on date D
- Analogue evidence from dates strictly before D
- No outcomes from date D or later

This is identical to the constraint that would have applied to a live prediction on date D.

### Output schema (prediction-diff dataset)

For every decision in the 1,089 PENDING population:

| Field | Description |
|---|---|
| `decision_id` | Original decision ID |
| `cohort_date` | Original cohort date |
| `ticker` | Ticker |
| `direction` | Direction (LONG/SHORT) |
| `original_action` | NoTrade (all 1,089) |
| `recalculated_action` | Watch or NoTrade |
| `original_degradation_level` | Insufficient (all 1,089) |
| `recalculated_degradation_level` | Exact / Approximate / Insufficient |
| `original_sample_size` | 0 (all 1,089) |
| `recalculated_sample_size` | N analogues found |
| `original_target_rate` | 0.0 (all 1,089) |
| `recalculated_target_rate` | Computed value |
| `original_rank_score` | 0.0 (all 1,089) |
| `recalculated_rank_score` | Computed value |
| `recalculated_evidence_class` | Favourable / Mixed / Insufficient |
| `recalculated_certification_status` | CERTIFIED / DEGRADED / STALE |
| `reason_for_change` | Evidence store populated / No analogues found / etc. |

### Key statistic to produce

```
1,089 original NoTrade
        ↓
X recalculated Watch
Y recalculated NoTrade (genuine — no analogues even with evidence store)
```

Then break X by cohort date and direction.

### Separation from frozen P4 policies

The recalculated predictions form a **separate dataset: TIME-009 RECOMPUTED**.

- Original records: preserved as `TIME-009 ORIGINAL` — 1,089 NoTrade
- Recalculated records: `TIME-009 RECOMPUTED` — new predictions

If recalculation produces Watch decisions, those become a new prospective cohort evaluated against the **already frozen** Track A/B thresholds:
- Track A: target_rate >= 0.314607 AND < 0.327778
- Track B.1: target_rate >= 0.327778 AND < 0.345900
- Track B.2: degradation_level == Exact
- Track B.3: sample_size >= 51 AND <= 150

**The thresholds are not recalculated. Only the predictions are recalculated.**

### Next action

1. Locate the evidence store and confirm it contains historical analogue data
2. Identify the prediction engine entry point for a single cohort
3. Run a test recalculation on one cohort (e.g., Aug 24) to confirm the approach
4. If successful, run all 14 cohorts
5. Produce the prediction-diff dataset
6. Identify qualifying Watch decisions under the frozen thresholds
7. Evaluate against 5m intraday data (horizons H15/H60/H300 have already elapsed for all Aug 24+ cohorts)

---

## 4. Integrity constraints

1. Original NoTrade records are preserved — not overwritten
2. Recalculated predictions are a separate dataset
3. Frozen P4 thresholds are not modified
4. Information-set boundary is strictly enforced per cohort date
5. No future data used in recalculation
6. If recalculation also produces NoTrade for a decision, that is a valid result — it means no analogues existed even with the evidence store loaded