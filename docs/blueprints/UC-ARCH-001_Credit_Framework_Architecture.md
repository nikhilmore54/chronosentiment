# UC-ARCH-001 — Credit Framework Architecture

**Component:** UltraCrew Airline Adapter  
**Status:** Design — v2 (final)  
**Date:** 2026-07-28  
**Triggered by:** GERAD G-2014-22 benchmark analysis  
**Primary source:** Quesnel, Kasirzadeh & Soumis — *Description of Data Sets and Generators*, GERAD G-2014-22, June 2014  
**Revision history:**
- v1 — initial proposal, credit nested inside Compliance, credit and cost merged
- v2 — Credit Engine elevated to standalone first-class layer; credit/cost separated; `CreditPolicyMetadata` added; `CreditContext` extensibility documented; two-notions-of-credit clarified; determinism contract added; per-duty scope limitation noted; provenance in optimisation artefacts noted

---

## 0. Official Data Hierarchy (from GERAD spec §1)

The benchmark defines a strict four-level hierarchy:

```
Airleg      — a direct flight between two airports
    |
Duty        — one work day: one or more airlegs and/or deadheads,
              ending when the crew member is granted a sleep rest (layover)
    |
Pairing     — a sequence of duties and layovers that starts and ends
              at the same base (for an unspecified crew member)
    |
Crew Schedule — a pairing assigned to a specific crew member
```

This hierarchy aligns with UltraCrew's existing domain model. The main capability gap is the **Credit Engine** — a standalone first-class layer between `DutyMetrics` and all downstream consumers.

The spec also distinguishes three categories of input data:

```
Operational Data          Policy / Optimisation Inputs
-----------------         ----------------------------
day_x.csv (airlegs)       credit_constraints.csv
listOfBases.csv           crew_avail_const.csv
initialSolution.in        preferredVacations (generated)
                          preferredAirlegs (generated)
```

Credit constraints and crew availability are **optimisation inputs**, not regulatory rules. They belong in an Optimisation Compliance Pack, not in DGCA/EASA/FAA regulatory packs.

---

## 1. Discovery

Analysis of the GERAD G-2014-22 benchmark dataset revealed that the benchmark does **not** optimise block hours. It optimises **credited hours** — a contractual construct that is distinct from any operational time metric.

From [`creditedHours`](../../benchmarks/gerad-g2014-22/raw/instance1/instance1/creditedHours) (instance 1, 33 crew, 31-day horizon):

| Statistic | Value |
|---|---|
| Average credited hours | 68.75h |
| Minimum | 23.67h (Schedule 25) |
| Maximum | 84.9h (Schedule 9) |
| Spread | ~61h |

Two crew members can have identical duty counts and identical block hours yet differ by 61 credited hours. The optimiser must recognise that difference.

From [`credit_constrains.csv`](../../benchmarks/gerad-g2014-22/raw/instance1/instance1/credit_constrains.csv) (instance 1):

| Base | Minimum credited hours | Share |
|---|---|---|
| BASE1 | 326.9h | 16.4% |
| BASE2 | 1,279.4h | 64.3% |
| BASE3 | 383.3h | 19.3% |
| Slack | 3% | — |

These are **base-level constraints** — not attached to a duty or a crew member, but to an organisational unit.

---

## 2. Two Distinct Notions of Credit in the GERAD Specification

The GERAD specification uses the word "credit" in two different contexts. Future contributors must not conflate them.

### 2.1 Conceptual credit (the contractual formula)

Defined in §1 (Terminology):

> Credit is the **scheduled flight duration plus half the duration of deadheads**.

This is the formula that `CreditPolicy.compute()` implements. Briefing, debriefing, and turnaround time are **excluded**.

### 2.2 Generator processing (the `creditedHours` file)

The `creditedHours` file (§4.2) contains historical schedule data from a previous solution. When the credit constraint generator (`credit_constraints.cpp`) reads this file to derive base credit floors, it **subtracts two hours per duty** to remove briefing/debriefing time that was included in the historical records.

This subtraction is a **data cleaning step in the generator**, not part of the contractual credit formula. It corrects for the fact that the historical `creditedHours` values were recorded with briefing/debriefing included, whereas the formal credit definition excludes them.

**Implication for UltraCrew:** `CreditPolicy.compute()` implements the conceptual formula (§2.1). The two-hour adjustment is only relevant if UltraCrew ever needs to reproduce the GERAD generator's base constraint derivation from raw historical data. It is not part of the credit computation for individual duties.

---

## 3. The Distinction That Matters

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
- Deadhead credit policies — GERAD uses **0.5x deadhead block time**; other CBAs may use 0% or 100%
- Layover credit (some agreements add a fixed credit per overnight away from base)
- Premium rates (night flying, international, holiday)
- Contractual overrides

Two airlines can assign different credited hours to exactly the same duty. Therefore `credited_hours` must **not** be added to `DutyMetrics`.

---

## 4. Architecture: Credit Engine as a First-Class Capability

The **Credit Engine** is a standalone architectural capability, not a sub-component of Compliance.

Compliance asks: *"Is this roster allowed?"*  
Credit Engine asks: *"How many contractual hours does this duty generate?"*

These are different responsibilities. Credit is not itself a constraint — it is a **derived contractual quantity**. Compliance merely consumes it. This mirrors how the rest of UltraCrew is already organised:

```
Duty -> DutyMetrics -> Compliance
```

The Credit Engine inserts cleanly between `DutyMetrics` and all downstream consumers:

```
Duty
  |
  v
DutyMetrics                    <- operational layer (already exists)
  |- report_time
  |- release_time
  |- duty_duration (FDP)
  |- block_time
  |- flight_time
  |- turnaround_time
  |- sector_count
  |- contains_deadhead
  +- contains_layover

          |
          v  (input to)

Credit Engine                  <- standalone first-class contractual layer (new)
  |- CreditPolicy              (trait — one impl per CBA)
  |- CreditPolicyMetadata      (provenance)
  |- DutyCredit                (output — credited hours + components)
  +- CreditComponents          (breakdown for explainability)

          |
          +--------------------------------------------------+
          v                                                  v
Compliance                     <- consumes credit  CostModel <- airline-specific
  |- AgreementPack                                   +- cost = credited_hours
  |    +- BaseConstraints                                     x pay_rate_per_hour
  |- RegulatoryPack
  +- CompanyPack

          |                                                  |
          v                                                  v
FairnessObjective  <- uses credited_hours       Payroll / Analytics
CostObjective      <- uses CostModel output
```

### Why credit and cost are separated

Credit is **airline-independent** — the same formula applies to all crew on the same agreement.  
Cost is **airline-specific** — a Captain and a First Officer earn different rates for the same credited hours.

GERAD merges them because it solves one optimisation problem with one pay rate. UltraCrew must keep them independent to support multi-rank, multi-fleet, multi-agreement rosters.

Example:

```
DutyCredit.credited_hours = 6.5h

Captain  (160/hr) -> DutyCost.credit_cost = 1,040
FO       (115/hr) -> DutyCost.credit_cost = 747.50
```

The credit computation is identical. The cost computation differs by rank. The `CostModel` abstraction also unlocks future crew types (Cabin Crew, Contractor, Reserve, Instructor) without touching the Credit Engine.

---

## 5. Constraint Hierarchy

The GERAD formulation introduces three distinct constraint scopes:

```
Constraint scope    Example
-----------------   --------------------------------------------------
Duty                FDP <= 13h (HC1), sectors <= 4 (HC2)
Roster              Weekly hours <= 60h (HC3), credited hours >= min_pay
Base                Total credited hours >= credit_constrains floor
```

The `AgreementPack` **validates** contractual compliance — it does not compute credit. Its rules include:

```
Minimum monthly credit guarantee
Maximum monthly credit cap
Reserve guarantee
Minimum duty pay guarantee
Vacation entitlement
```

These rules evaluate `DutyCredit.credited_hours` against thresholds. They do not call `CreditPolicy.compute()`.

Proposed compliance framework extension:

```
ComplianceRegistry
  |- RegulatoryPack          (DGCA, EASA, FAA Part 117)
  |    +- DutyConstraints    (FDP, rest, sectors)
  |- CompanyPack             (airline-specific operational rules)
  |    +- RosterConstraints  (weekly hours, consecutive days)
  |- AgreementPack           (CBA / union agreement)        <- new
  |    +- BaseConstraints    (credit floor per base)        <- new
  +- OptimizationObjective
       |- FairnessObjective  (balance credited hours, not block hours)
       +- CostObjective      (minimise total cost from CostModel)
```

The constraint hierarchy is intentionally extensible upward:

```
Duty -> Roster -> Base -> Network -> Fleet -> Company
```

Lower levels do not change when higher levels are added.

---

## 6. Fairness Objective Correction

The current fairness objective likely balances duty counts or block hours. The GERAD benchmark shows the correct target is **credited hours**.

Example where block-hour fairness fails:

| Crew | Duties | Block hours | Credited hours |
|---|---|---|---|
| A | 10 | 40h | 68h |
| B | 10 | 40h | 82h |

Operationally identical. Contractually very different. An optimiser balancing block hours would see no imbalance. An optimiser balancing credited hours would correctly penalise the 14h gap.

The SC1 soft constraint ("High workload imbalance — Fairness penalty: 4845.07") observed in the portal Step 4 output is this credited-hours spread.

**Corrected fairness objective:**

```
Fairness penalty = sum over all crew i: (credited_hours_i - mean_credited_hours)^2
```

---

## 7. Cost Function Interpretation

The GERAD objective function:

```
Total cost = sum of schedule_cost_i
           = sum of f(credited_hours_i, pay_rate_i, violations_i)
```

This is **not** a function of block hours. It is a function of credited hours x contractual pay rate. UltraCrew's cost model should use `CostModel` output as the primary cost signal, not `block_time` from `DutyMetrics`.

---

## 8. Proposed Rust Types

### 8.1 `CreditPolicyMetadata` — provenance

Every optimisation result records which credit policy was applied. This matches the provenance approach already adopted in the compliance framework (`ViolationExplanation` with `rule_id`, `authority`, `version`).

**Optimisation artefacts should record both compliance metadata (rule packs and versions) and credit policy metadata**, so that any result can be fully reproduced from both the regulatory and contractual perspectives.

```rust
pub struct CreditPolicyMetadata {
    /// Stable identifier, e.g. "GERAD-G2014-22".
    pub id: &'static str,
    /// Normative authority, e.g. "Quesnel, Kasirzadeh & Soumis (2014)".
    pub authority: &'static str,
    /// Semantic version of this policy implementation.
    pub version: &'static str,
    /// Human-readable description.
    pub description: &'static str,
}
```

### 8.2 `CreditPolicy` trait

**Determinism contract:** A `CreditPolicy` implementation must be deterministic. Given identical `DutyMetrics` and `CreditContext`, it must always produce identical `DutyCredit`. This is consistent with Coralys' emphasis on reproducibility and deterministic optimisation. Implementations must not read external state, clocks, or random sources.

**Per-duty scope:** `CreditPolicy.compute()` intentionally operates per duty. Roster-level contractual adjustments (monthly guarantees, cumulative premiums, sequence bonuses, consecutive duty bonuses) are evaluated separately by `AgreementPack` or future `RosterCreditPolicy` implementations. This limitation is intentional — it keeps the per-duty computation stateless and parallelisable.

```rust
/// Computes contractual credit for a single duty.
/// One implementation per CBA / airline agreement.
///
/// # Determinism contract
/// Implementations MUST be deterministic: identical inputs MUST produce
/// identical outputs. No external state, clocks, or random sources.
///
/// # Scope
/// This interface computes per-duty credit only. Roster-level adjustments
/// (monthly guarantees, sequence bonuses) are handled by AgreementPack
/// or a future RosterCreditPolicy.
pub trait CreditPolicy: Send + Sync {
    fn compute(&self, metrics: &DutyMetrics, context: &CreditContext) -> DutyCredit;
    fn metadata(&self) -> CreditPolicyMetadata;
}
```

### 8.3 `CreditContext`

Sufficient for GERAD. Intentionally extensible for production airlines.

```rust
pub struct CreditContext {
    /// Crew member's assigned base.
    pub crew_base: AirportCode,
    /// Crew member's home base (may differ from crew_base during displacement).
    pub home_base: AirportCode,
    /// Date the duty begins (for holiday calendar, agreement version lookup).
    pub applicable_date: NaiveDate,

    // --- Production extensions (not required for GERAD) ---
    // pub aircraft_type: Option<AircraftType>,
    // pub crew_rank: Option<CrewRank>,
    // pub employment_type: Option<EmploymentType>,
    // pub agreement_version: Option<AgreementVersion>,
    // pub time_zone: Option<chrono_tz::Tz>,
    // pub holiday_calendar: Option<HolidayCalendarRef>,
}
```

### 8.4 `DutyCredit` — credit only, no cost

```rust
pub struct DutyCredit {
    /// Contractual credited hours for this duty.
    /// Airline-independent: same formula for all crew on the same agreement.
    pub credited_hours: f64,
    /// Breakdown for explainability (dispatcher view, audit trail).
    pub components: CreditComponents,
}

pub struct CreditComponents {
    /// Operated (non-deadhead) flight block time credited.
    pub block_credit: f64,
    /// Deadhead block time x deadhead_credit_factor.
    pub deadhead_credit: f64,
    /// Fixed credit for overnight layover away from base (0.0 for GERAD).
    pub layover_credit: f64,
    /// Night/international/holiday premium credit (0.0 for GERAD).
    pub premium_credit: f64,
    /// True if a minimum guarantee floor was applied.
    pub minimum_guarantee_applied: bool,
}
```

The `CreditComponents` breakdown gives explainability for free. A dispatcher inspecting a duty sees:

```
Flight      6.0h
Deadhead    1.5h   (3.0h x 0.5)
Layover     0.0h
Premium     0.0h
-----------------
Total       7.5h   (no minimum guarantee applied)
```

### 8.5 `CostModel` — airline-specific, separate from credit

```rust
/// Converts credited hours into monetary cost.
/// Separate from CreditPolicy because cost depends on rank and fleet,
/// while credit depends only on the duty and the agreement.
pub trait CostModel: Send + Sync {
    fn compute_cost(&self, credit: &DutyCredit, context: &CostContext) -> DutyCost;
}

pub struct CostContext {
    pub crew_rank: CrewRank,
    pub fleet: Fleet,
    pub applicable_date: NaiveDate,
}

pub struct DutyCost {
    pub credit_cost: f64,   // credited_hours x pay_rate
    pub pay_rate: f64,      // for audit trail
}
```

### 8.6 `RosterMetrics`

```rust
pub struct RosterMetrics {
    pub total_block_hours: f64,
    pub total_flight_hours: f64,
    pub total_fdp_hours: f64,
    /// Aggregated from DutyCredit.credited_hours across all duties.
    /// Lives here, not on DutyMetrics.
    pub total_credited_hours: f64,
    /// Aggregated from DutyCost.credit_cost across all duties.
    pub total_credit_cost: f64,
    pub vacations: u32,
    pub deadheads: u32,
    pub layovers: u32,
}
```

### 8.7 `BaseCreditFloor`

```rust
/// Minimum total credited hours for all crew in one base.
/// Loaded from credit_constrains.csv.
pub struct BaseCreditFloor {
    pub base: AirportCode,
    pub minimum_credited_hours: f64,
    pub slack_fraction: f64,  // 0.03 for GERAD instance 1
}
```

### 8.8 GERAD reference implementation

The official credit formula (Quesnel et al., §1):

```
credit = scheduled_flight_time + 0.5 x deadhead_time
```

Briefing, debriefing, and turnaround time are **excluded** (see Section 2 for the two-notions-of-credit distinction).

```rust
/// GERAD G-2014-22 credit policy.
/// Official formula (Quesnel et al. §1):
///   credit = scheduled_flight_time + 0.5 x deadhead_time
/// Briefing/debriefing time is excluded from credit (it is in DutyMetrics
/// but not in the contractual credit measure).
///
/// Determinism: this implementation is a pure function of its inputs.
pub struct GeradCreditPolicy {
    /// Fraction of deadhead block time credited. GERAD official value: 0.5.
    pub deadhead_credit_factor: f64,
}

impl Default for GeradCreditPolicy {
    fn default() -> Self {
        Self { deadhead_credit_factor: 0.5 }
    }
}

impl CreditPolicy for GeradCreditPolicy {
    fn compute(&self, metrics: &DutyMetrics, _ctx: &CreditContext) -> DutyCredit {
        // flight_time = operated (non-deadhead) block time
        let flight_hours = metrics.flight_time.num_minutes() as f64 / 60.0;
        // deadhead block time = block_time - flight_time
        let deadhead_hours = (metrics.block_time - metrics.flight_time)
            .num_minutes() as f64 / 60.0;
        let deadhead_credit = deadhead_hours * self.deadhead_credit_factor;
        let credited = flight_hours + deadhead_credit;
        DutyCredit {
            credited_hours: credited,
            components: CreditComponents {
                block_credit: flight_hours,
                deadhead_credit,
                layover_credit: 0.0,   // GERAD does not add layover credit
                premium_credit: 0.0,
                minimum_guarantee_applied: false,
            },
        }
    }

    fn metadata(&self) -> CreditPolicyMetadata {
        CreditPolicyMetadata {
            id: "GERAD-G2014-22",
            authority: "Quesnel, Kasirzadeh & Soumis (2014), GERAD Technical Report G-2014-22",
            version: "1.0",
            description: "credit = scheduled_flight_time + 0.5 x deadhead_time",
        }
    }
}
```

---

## 9. Implementation Phases

All phases are **additive** — no breaking changes to existing structs.

| Phase | Deliverable | File |
|---|---|---|
| 1 | `CreditPolicyMetadata` + `CreditPolicy` trait + `DutyCredit` + `CreditComponents` | `adapters/airline/src/domain/credit.rs` |
| 2 | `GeradCreditPolicy` implementation | `adapters/airline/src/domain/credit.rs` |
| 3 | `CostModel` trait + `DutyCost` + `CostContext` | `adapters/airline/src/domain/cost.rs` |
| 4 | `RosterMetrics` + aggregation helper | `adapters/airline/src/domain/roster_metrics.rs` |
| 5 | `BaseCreditFloor` loader + constraint check | `adapters/airline/src/domain/roster_metrics.rs` |
| 6 | Wire `FairnessObjective` to use `total_credited_hours` | `services/ultracrew_server/src/` |
| 7 | Wire `CostObjective` to use `CostModel` output | `services/ultracrew_server/src/` |

---

## 10. What Does Not Change

- [`DutyMetrics`](../../adapters/airline/src/domain/duty.rs) — **no new fields**. Remains purely operational.
- [`BriefingOffsets`](../../adapters/airline/src/domain/duty.rs) — unchanged.
- Existing regulatory packs (DGCA, EASA, FAA Part 117) — unchanged. They evaluate `DutyMetrics` directly.
- The compliance engine's duty-level and roster-level evaluators — unchanged interfaces.

The Credit Engine is an **additive layer** that reads `DutyMetrics` as input and produces `DutyCredit` as output. It does not modify the operational domain model.

---

## 11. Evidence from GERAD Instance 7

[`credit_constraints.csv` (instance 7)](../../benchmarks/gerad-g2014-22/raw/instance7/instance7/credit_constraints.csv) provides multiple credit distribution scenarios for 305 crew:

| Scenario | BASE1 | BASE2 | BASE3 |
|---|---|---|---|
| Proportional (initial) | 11,426h | 6,864h | 3,721h |
| 90%/5%/5% | 19,810h | 1,101h | 1,101h |
| Equal 33.3% | 7,337h | 7,337h | 7,337h |

The paper's key finding: forcing equal credit distribution across bases significantly increases total cost. This validates that base credit constraints are a real optimisation lever, not just a reporting artefact.

---

## 12. Summary

| Layer | Struct / Trait | Owns | Consumers |
|---|---|---|---|
| Operational | `DutyMetrics` | FDP, block time, flight time, turnaround, sector count | Credit Engine, Compliance |
| Credit Engine | `CreditPolicy` -> `DutyCredit` | Credited hours, components | Compliance, Fairness, CostModel, Analytics |
| Cost | `CostModel` -> `DutyCost` | Monetary cost per duty | CostObjective, Payroll |
| Roster | `RosterMetrics` | Aggregated credited hours, total cost, vacations | Fairness, Reporting |
| Base | `BaseCreditFloor` | Minimum credited hours per organisational unit | AgreementPack |
| Fairness | `FairnessObjective` | Variance of credited hours across crew (not block hours) | Optimiser |
| Provenance | `CreditPolicyMetadata` | Policy identity, authority, version | Audit trail, result records |

The separation preserves the clean architecture already established in the codebase and allows different airline agreements to be supported by swapping `CreditPolicy` implementations without touching the operational, regulatory, or cost layers.

---

## 13. References

- [`benchmarks/gerad-g2014-22/raw/instance1/instance1/creditedHours`](../../benchmarks/gerad-g2014-22/raw/instance1/instance1/creditedHours)
- [`benchmarks/gerad-g2014-22/raw/instance1/instance1/credit_constrains.csv`](../../benchmarks/gerad-g2014-22/raw/instance1/instance1/credit_constrains.csv)
- [`benchmarks/gerad-g2014-22/raw/instance7/instance7/credit_constraints.csv`](../../benchmarks/gerad-g2014-22/raw/instance7/instance7/credit_constraints.csv)
- [`adapters/airline/src/domain/duty.rs`](../../adapters/airline/src/domain/duty.rs) — existing `DutyMetrics`
- [`docs/blueprints/UC-B-001_UltraCrew_Blueprint_v1.0.md`](UC-B-001_UltraCrew_Blueprint_v1.0.md) — UltraCrew product blueprint
- Quesnel, F., Kasirzadeh, A., Soumis, F. (2014). *Description of Data Sets and Generators*. GERAD Technical Report G-2014-22. HEC Montréal.