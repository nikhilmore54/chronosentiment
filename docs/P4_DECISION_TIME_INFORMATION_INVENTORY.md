# P4 — Decision-Time Information Inventory
## Allowed / Forbidden / Derived Field Classification

**Date:** 2026-09-11  
**Status:** ACTIVE — Phase 1 complete  
**Source artifacts:** `live_capture/ledger/entries/*.json` + `time_machine/analysis/TIME009/observations/*.json`

---

## Purpose

Establish the legitimate search space for Optimal Trade Discovery.

Every candidate policy variable must be classifiable as ALLOWED or DERIVED. Any field that contains or derives from future information is FORBIDDEN and must never be used as a policy input.

---

## ALLOWED — Available at decision time

These fields are present in the ledger entry and/or observation artifact and reflect only information known at the moment the decision was generated.

| Field | Type | Notes |
|---|---|---|
| `ticker` | categorical | Identity of the instrument |
| `direction` | categorical | LONG / SHORT — engine's directional call |
| `action` | categorical | Watch / NoTrade — engine's action recommendation |
| `reference_price` | numeric | Price at snapshot time |
| `adaptive_target` | numeric | Engine-computed target price (null for NoTrade) |
| `adaptive_risk` | numeric | Engine-computed risk boundary (null for NoTrade) |
| `adaptive_horizon_sessions` | numeric | Engine-computed horizon in sessions (null for NoTrade) |
| `evidence_class` | categorical | Favourable / Mixed / Insufficient |
| `degradation_level` | categorical | Exact / Approximate / Insufficient |
| `sample_size` | numeric | Number of historical analogues |
| `target_rate` | numeric | Historical target-hit rate among analogues |
| `rank_score` | numeric | Engine's composite rank score (0–1) |
| `vol_regime` | categorical | present / absent |
| `volume_regime` | categorical | Normal / High / Low |
| `certification_status` | categorical | CERTIFIED / DEGRADED |
| `cohort_date` | date | Date of the decision cohort |
| `source_snapshot_unix` | timestamp | Unix timestamp of the source snapshot |
| `admitted_at` | timestamp | When the decision was admitted to the ledger |

---

## FORBIDDEN — Contains or derives from future information

These fields are present in the observation artifact but reflect information that was not available at decision time. They must NEVER be used as policy inputs.

| Field | Reason |
|---|---|
| `exit_reason` | Determined by future price movement |
| `exit_bar_index` | Determined by future price movement |
| `exit_bar_timestamp` | Determined by future price movement |
| `exit_price` | Future price |
| `sessions_to_outcome` | Determined by future price movement |
| `target_reached` | Future outcome |
| `risk_reached` | Future outcome |
| `horizon_reached` | Future outcome |
| `actual_mfe` | Maximum favourable excursion — future price |
| `actual_mae` | Maximum adverse excursion — future price |
| `realized_return` | Future outcome |
| `horizon_elapsed` | Future state |
| `ambiguous` | Future state |
| `n_bars_after_t0` | Computed from future bars |
| `n_bars_in_horizon` | Computed from future bars |
| `first_eligible_bar_timestamp` | Depends on future bar availability |
| `eligible_for_primary_comparison` | Eligibility flag set post-observation |
| `observation_status` | Set after outcome is known |
| `observed_at` | Timestamp of observation, not decision |

---

## DERIVED — Safely computable from ALLOWED fields only

These fields do not exist in the artifacts but can be computed exclusively from ALLOWED information.

| Derived field | Formula | Notes |
|---|---|---|
| `upside_pct` | `(adaptive_target - reference_price) / reference_price` | Percentage upside to target |
| `downside_pct` | `(reference_price - adaptive_risk) / reference_price` | Percentage downside to risk |
| `rr_ratio` | `upside_pct / downside_pct` | Reward-to-risk ratio |
| `target_distance_pct` | same as `upside_pct` | Alias |
| `risk_distance_pct` | same as `downside_pct` | Alias |
| `is_watch` | `action == "Watch"` | Boolean: tradeable decision |
| `is_no_trade` | `action == "NoTrade"` | Boolean: engine declined |
| `is_long` | `direction == "LONG"` | Boolean |
| `is_short` | `direction == "SHORT"` | Boolean |
| `is_favourable` | `evidence_class == "Favourable"` | Boolean |
| `is_mixed` | `evidence_class == "Mixed"` | Boolean |
| `is_exact` | `degradation_level == "Exact"` | Boolean |
| `is_certified` | `certification_status == "CERTIFIED"` | Boolean |
| `cohort_day_of_week` | `dayofweek(cohort_date)` | 0=Mon … 4=Fri |
| `cohort_month` | `month(cohort_date)` | 1–12 |
| `expected_return` | `target_rate * upside_pct - (1 - target_rate) * downside_pct` | Simple EV estimate |
| `analogue_density` | `sample_size / max_sample_size` | Relative analogue count (requires population max) |

---

## Outcome variables (targets for policy evaluation — FORBIDDEN as inputs)

| Field | Use |
|---|---|
| `realized_return` | Primary return metric |
| `target_reached` | Binary win (target hit) |
| `risk_reached` | Binary loss (risk hit) |
| `horizon_reached` | Horizon expiry |
| `actual_mfe` | Best case during horizon |
| `actual_mae` | Worst case during horizon |
| `sessions_to_outcome` | Speed of resolution |

---

## Population summary

- **Total decisions:** ~1,393 (all cohorts)
- **COMPLETE observations:** ~300 (have outcome fields populated)
- **PENDING observations:** ~1,093 (outcome fields null — horizon not yet elapsed)
- **Eligible for primary comparison:** subset of COMPLETE where `eligible_for_primary_comparison == true`
- **NoTrade decisions:** present in population — must be preserved, not filtered out

---

## Candidate policy search space

Based on the ALLOWED + DERIVED fields, the legitimate policy variables for Sprint 1 are:

**Selection filters (which decisions to act on):**
- `action` == Watch (exclude NoTrade)
- `direction` (LONG only / SHORT only / both)
- `evidence_class` threshold (Favourable only / Favourable+Mixed / all)
- `degradation_level` threshold (Exact only / Exact+Approximate / all)
- `certification_status` (CERTIFIED only / CERTIFIED+DEGRADED)
- `sample_size` >= N
- `target_rate` >= threshold
- `rank_score` >= threshold
- `rr_ratio` >= threshold
- `vol_regime` filter
- `volume_regime` filter

**Ranking variables (how to order selected decisions):**
- `rank_score`
- `target_rate`
- `rr_ratio`
- `expected_return`
- `sample_size`
- combinations of the above

**Outcome metric for policy evaluation (FORBIDDEN as inputs, used only for scoring):**
- Primary: `realized_return` (mean, median)
- Secondary: `target_reached` rate, `risk_reached` rate
- Composite: return + drawdown + win rate + downside + frequency + consistency

---

## Next step: Sprint 1 — Rank

Use the ALLOWED + DERIVED fields above to construct candidate ranking policies and evaluate them against the 300 COMPLETE decisions. Measure whether top-ranked decisions outperform bottom-ranked decisions on `realized_return`.

**Guiding principle: Explore aggressively. Validate ruthlessly.**