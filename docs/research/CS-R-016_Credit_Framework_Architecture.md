# CS-R-016 — Credit Framework Architecture

**Status:** Draft  
**Date:** 2026-07-28  
**Author:** UltraCrew Architecture  
**Triggered by:** GERAD G-2014-22 benchmark analysis (`benchmarks/gerad-g2014-22/raw/instance1/instance1/creditedHours`)

---

## 1. Discovery

Analysis of the GERAD G-2014-22 benchmark dataset revealed that the benchmark does **not** optimise block hours. It optimises **credited hours** — a contractual construct distinct from any operational time metric.

From `creditedHours` (instance 1, 33 crew, 31-day horizon):

| Statistic | Value |
|---|---|
| Average credited hours | 68.75h |
| Minimum | 23.67h (Schedule 25) |
| Maximum | 84.9h (Schedule 9) |
| Spread | ~61h |

Two crew members can have identical duty counts and identical block hours yet differ by 61 credited hours. The optimiser must recognise that difference.

From `credit_constrains.csv` (instance 1):

| Base | Minimum credited hours | Share |
|---|---|---|
| BASE1 | 326.9h | 16.4% |
| BASE2 | 1,279.4h | 64.3% |
| BASE3 | 383.3h | 19.3% |
| Slack | 3% | — |

These are **base-level constraints** — not attached to a duty or a crew member, but to an organisational unit.

---

## 2. The Distinction That Matters

The existing [`DutyMetrics`](../../adapters/airline/src/domain/duty.rs) correctly captures operational facts:

| Field | Definition |
|---|---|
| `report_time` | First leg departure − briefing offset |
| `release_time` | Last leg arrival + debriefing offset |
| `duty_duration` | FDP = release − report |
| `block_time` | Sum of scheduled leg durations |
| `flight_time` | Block time of operated (non-deadhead) legs |
| `turnaround_time` | duty_duration − block_time |
| `sector_count` | Number of legs |
| `contains_deadhead` | Positioning leg present |
| `contains_layover` | Duty ends away from base |

**Credited hours are not an operational property of a duty.** They depend on:

- Collective bargaining agreements (CBA)
- Company pay rules
- Minimum duty pay guarantees (e.g. "minimum 3h credit per duty regardless of block time")
- Deadhead credit policies (some CBAs credit 50% of deadhead block time, others 100%)
- Layover credit (some agreements add a fixed credit per overnight away from base)
- Premium rates (night flying, international, holiday)
- Contractual overrides

Two airlines can assign different credited hours to exactly the same duty. Therefore `credited_hours` must **not** be added to `DutyMetrics`.

---

## 3. Proposed Architecture

```
Duty
  │
  ▼
DutyMetrics                    ← operational layer (already exists)
  ├── report_time
  ├── release_time
  ├── duty_duration (FDP)
  ├── block_time
  ├── flight_time
  ├── turnaround_time
  ├── sector_count
  ├── contains_deadhead
  └── contains_layover

          │
          ▼  (input to)

CreditPolicy                   ← contractual layer (new)
  ├── min_duty_credit_hours     (minimum pay guarantee per duty)
  ├── deadhead_credit_factor    (0.0–1.0 fraction of deadhead block credited)
  ├── layover_credit_hours      (fixed credit added per overnight layover)
  ├── night_premium_factor      (multiplier for legs between 22:00–06:00 local)
  └── compute(duty: &DutyMetrics, context: &CreditContext) -> DutyCredit

          │
          ▼

DutyCredit                     ← per-duty contractual output (new)
  ├── credited_hours            (contractual hours for this duty)
  ├── credit_cost               (monetary cost at applicable pay rate)
  └── credit_components         (breakdown: block, deadhead, layover, premium)

          │  (aggregated over roster)
          ▼

RosterMetrics                  ← roster-level aggregate (new)
  ├── total_block_hours
  ├── total_flight_hours
  ├── total_fdp_hours
  ├── total_credited_hours      ← lives here, not on DutyMetrics
  ├── credit_cost
  ├── vacations
  ├── deadheads
  └── layovers
```

---

## 4. Constraint Hierarchy

The GERAD formulation introduces three distinct constraint scopes:

```
Constraint scope    Example
─────────────────   ──────────────────────────────────────────────────
Duty                FDP ≤ 13h (HC1), sectors ≤ 4 (HC2)
Roster              Weekly hours ≤ 60h (HC3), credited hours ≥ min_pay
Base                Total credited hours ≥ credit_constrains floor
```

The compliance engine already evaluates duty-level and roster-level constraints. **Base-level constraints** are a new class that evaluates an entire crew base as a unit.

Proposed extension to the compliance framework:

```
ComplianceRegistry
  ├── RegulatoryPack          (DGCA, EASA, FAA Part 117)
  │     └── DutyConstraints   (FDP, rest, sectors)
  ├── CompanyPack             (airline-specific operational rules)
  │     └── RosterConstraints (weekly hours, consecutive days)
  ├── AgreementPack           (CBA / union agreement)        ← new
  │     ├── CreditPolicy      (how credited hours are computed)
  │     └── BaseConstraints   (credit floor per base)        ← new
  └── OptimizationObjective
        ├── FairnessObjective (balance credited hours, not block hours)
        └── CostObjective     (minimise total credit_cost)
```

---

## 5. Fairness Objective Correction

The current fairness objective likely balances duty counts or block hours. The GERAD benchmark shows the correct target is **credited hours**.

Example where block-hour fairness fails:

| Crew | Duties | Block hours | Credited hours |
|---|---|---|---|
| A | 10 | 40h | 68h |
| B | 10 | 40h | 82h |

Operationally identical. Contractually very different. An optimiser balancing block hours would see no imbalance. An optimiser balancing credited hours would correctly penalise the 14h gap.

The SC1 soft constraint ("High workload imbalance — Fairness penalty: 4845.07") observed in the portal Step 4 output is this credited-hours spread. The penalty magnitude (4845) is consistent with the GERAD cost function which weights fairness violations heavily.

**Corrected fairness objective:**

```
Fairness penalty = Σ (credited_hours_i − mean_credited_hours)²
                   over all crew i in the same base
```

---

## 6. Cost Function Interpretation

The GERAD objective function can now be read as:

```
Total cost = Σ schedule_cost_i
           = Σ f(credited_hours_i, pay_rate_i, violations_i)
```

This is **not** a function of block hours. It is a function of credited hours × contractual pay rate. This aligns with how airlines actually budget crew operations: the payroll line item is credited hours × hourly rate, not block hours × hourly rate.

Implication: UltraCrew's cost model should use `credit_cost` from `DutyCredit` as the primary cost signal, not `block_time` from `DutyMetrics`.

---

## 7. Implementation Plan

### Phase 1 — Credit Policy trait (no breaking changes)

```rust
/// Computes contractual credit for a single duty.
pub trait CreditPolicy: Send + Sync {
    fn compute(&self, metrics: &DutyMetrics, context: &CreditContext) -> DutyCredit;
}

pub struct CreditContext {
    pub crew_base: AirportCode,
    pub home_base: AirportCode,
    pub applicable_date: NaiveDate,
}

pub struct DutyCredit {
    pub credited_hours: f64,
    pub credit_cost: f64,
    pub components: CreditComponents,
}

pub struct CreditComponents {
    pub block_credit: f64,
    pub deadhead_credit: f64,
    pub layover_credit: f64,
    pub premium_credit: f64,
    pub minimum_guarantee_applied: bool,
}
```

### Phase 2 — GERAD reference implementation

```rust
pub struct GeradCreditPolicy {
    pub min_duty_credit_hours: f64,   // GERAD uses ~3h minimum
    pub deadhead_credit_factor: f64,  // 1.0 (full deadhead credit)
    pub layover_credit_hours: f64,    // 0.0 (GERAD does not add layover credit)
    pub pay_rate_per_hour: f64,       // derived from creditedHours cost data
}
```

### Phase 3 — RosterMetrics aggregation

Aggregate `DutyCredit` values across a crew member's full roster into `RosterMetrics`. Feed `total_credited_hours` into the fairness objective.

### Phase 4 — BaseConstraint evaluator

```rust
pub struct BaseCreditFloor {
    pub base: AirportCode,
    pub minimum_credited_hours: f64,
    pub slack_fraction: f64,  // 0.03 for GERAD instance 1
}
```

Evaluate after full schedule construction, not per-duty.

---

## 8. What Does Not Change

- [`DutyMetrics`](../../adapters/airline/src/domain/duty.rs:116) — no new fields. Remains purely operational.
- [`BriefingOffsets`](../../adapters/airline/src/domain/duty.rs:61) — unchanged.
- Existing regulatory packs (DGCA, EASA, FAA Part 117) — unchanged. They evaluate `DutyMetrics` directly.
- The compliance engine's duty-level and roster-level evaluators — unchanged interfaces.

The Credit Framework is an **additive layer** that reads `DutyMetrics` as input and produces `DutyCredit` as output. It does not modify the operational domain model.

---

## 9. Evidence from GERAD Instance 7

[`credit_constraints.csv` (instance 7)](../../benchmarks/gerad-g2014-22/raw/instance7/instance7/credit_constraints.csv) provides multiple credit distribution scenarios for 305 crew:

| Scenario | BASE1 | BASE2 | BASE3 |
|---|---|---|---|
| Proportional (initial) | 11,426h | 6,864h | 3,721h |
| 90%/5%/5% | 19,810h | 1,101h | 1,101h |
| Equal 33.3% | 7,337h | 7,337h | 7,337h |

The paper's key finding: forcing equal credit distribution across bases significantly increases total cost. This validates that base credit constraints are a real optimisation lever, not just a reporting artefact.

---

## 10. Summary

| Layer | Struct | Owns |
|---|---|---|
| Operational | `DutyMetrics` | FDP, block time, flight time, turnaround, sector count |
| Contractual | `CreditPolicy` → `DutyCredit` | Credited hours, credit cost, components |
| Roster | `RosterMetrics` | Aggregated credited hours, total cost, vacations |
| Base | `BaseCreditFloor` | Minimum credited hours per organisational unit |
| Fairness | `FairnessObjective` | Variance of credited hours across crew (not block hours) |

The separation preserves the clean architecture already established in the codebase and allows different airline agreements to be supported by swapping `CreditPolicy` implementations without touching the operational or regulatory layers.