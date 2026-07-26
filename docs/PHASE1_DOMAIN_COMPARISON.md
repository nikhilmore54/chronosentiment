# Phase 1 — Cross-Domain Comparison: INRC2 vs Airline Crew Scheduling

> **Date**: 2026-07-21
> **Version**: 1.0 — Phase 1 Complete
> **Status**: FROZEN — feeds Phase 2 implementation
> **Purpose**: Resolve OQ-1, OQ-2, OQ-3, OQ-4 with evidence from the code.
> **Sources**:
> - [`adapters/ultracrew/src/inrc/models.rs`](../adapters/ultracrew/src/inrc/models.rs) — INRC2 domain model
> - [`coralys-scheduling/src/domain/`](../coralys-scheduling/src/domain/) — Airline domain model
> - [`coralys-scheduling/src/legality/mod.rs`](../coralys-scheduling/src/legality/mod.rs) — Airline legality layer
> - [`coralys-scheduling/src/optimization/objective.rs`](../coralys-scheduling/src/optimization/objective.rs) — Airline objectives

---

## 1. Domain Hierarchy Maps

### 1.1 INRC2 (Nurse Scheduling)

```
InrcScenario
    ├── nurses: Vec<InrcNurse>          (worker pool)
    │       └── id, contract, skills
    ├── contracts: Vec<InrcContract>    (work-rule templates)
    │       └── min/max assignments, consecutive work/off, weekends
    ├── shift_types: Vec<InrcShiftType> (work unit definitions)
    │       └── id, min/max consecutive assignments
    └── forbidden_shift_type_successions

InrcWeekData
    ├── requirements: Vec<InrcRequirement>   (coverage demand)
    │       └── (shift_type, skill) × 7 days × {minimum, optimal}
    └── shift_off_requests: Vec<InrcShiftOffRequest>

InrcHistory
    └── nurse_history: Vec<InrcNurseHistory>
            └── last_assigned_shift_type, consecutive counts, weekend count

Assignment (genome):
    (nurse_id: usize, shift_type_id: usize, day: usize)  →  AssignmentSlot
```

**Atomic Planning Unit**: `(nurse, shift_type, day)` — a single shift assignment.
The shift type is the work unit definition; the assignment is the act of placing a nurse into that shift on a specific day.

### 1.2 Airline Crew Rostering

```
Roster
    ├── crew_members: HashMap<CrewId, CrewMember>   (worker pool)
    │       └── id, name, role, qualifications, base
    ├── legs: HashMap<FlightLegId, FlightLeg>        (work to be covered)
    │       └── id, flight_number, origin, dest, departure, arrival, aircraft_type
    └── rotations: HashMap<CrewId, Rotation>         (assignments)
            └── crew_id + Vec<Pairing>
                    └── Pairing: base-to-base trip
                            └── Vec<Duty>: single work block
                                    └── Vec<FlightLeg>: atomic flight segment

PlanningPeriod: [start, end)  — months-long schedule period
```

**Atomic Planning Unit**: `Pairing` — a base-to-base trip assigned to a crew member.
`FlightLeg` and `Duty` are sub-components of a pairing; they are not the unit of assignment. The optimizer assigns pairings to crew members, not individual legs.

---

## 2. Domain Comparison Matrix

| Concept | INRC2 | Airline | Shared? |
|---|---|---|---|
| **Worker** | `InrcNurse` (id, contract, skills) | `CrewMember` (id, name, role, qualifications, base) | Yes — worker identity + capability set |
| **Worker capability** | `skills: Vec<String>` | `qualifications: Vec<Qualification>` (aircraft type ratings) | Yes — capability tags that constrain assignment legality |
| **Worker home** | implicit (hospital unit, not modeled) | `base: AirportCode` | Partial — airline has explicit geographic home; INRC2 does not |
| **Work unit definition** | `InrcShiftType` (id, min/max consecutive) | `FlightLeg` (id, origin, dest, departure, arrival, aircraft) | No — fundamentally different. Shift is a time-block template; FlightLeg is a concrete physical event |
| **Atomic Planning Unit** | `(nurse, shift_type, day)` — shift assignment | `Pairing` — base-to-base trip | **Yes at the interface level** — both are the unit assigned to a worker by the optimizer |
| **Sub-structure below APU** | None — shift is atomic | `Duty → FlightLeg` within a Pairing | No — airline has rich internal structure; INRC2 does not |
| **Coverage demand** | `InrcRequirement`: (shift_type, skill, day) → {minimum, optimal} | `FlightLeg` set in `Roster` — every leg must appear in exactly one rotation | Partial — both express "work that must be covered"; structure differs |
| **Coverage metric** | minimum + optimal levels per slot | binary: covered or not (one rotation per leg) | No — INRC2 has graded coverage; airline has binary coverage |
| **Planning horizon** | Multi-week (4–8 weeks), week-by-week | Schedule period (months), single solve | No — INRC2 is rolling/incremental; airline is batch |
| **Carry-over state** | `InrcNurseHistory` — consecutive counts, last shift, weekend count | None — each solve is independent | No — INRC2 has explicit history; airline does not |
| **Hard constraints** | HC1 min coverage, HC2 skill match, HC3 forbidden succession, HC4 single assignment/day | Duty connectivity, max duty time, min rest, FDP, qualification, base return, coverage | Partial — both have hard legality rules; specific rules are domain-specific |
| **Soft constraints / objectives** | Weighted penalty sum (S1–S8: totals, consecutive, preferences, weekends, optimal coverage) | Minimisation objectives: workload balance, coverage cost, rest quality | Partial — both optimize quality beyond legality; cost model structure differs |
| **Constraint architecture** | `InrcConstraintId` enum + `ObjectiveWeights` — single weighted sum | `LegalityRule` trait (hard) + `SchedulingObjective` trait (soft) — separated layers | No — INRC2 collapses hard+soft into one weighted score; airline separates them |
| **Forbidden transitions** | `InrcForbiddenSuccession` — shift type A cannot follow shift type B | Implicit in `DutyConnectivityRule` — legs within a duty must connect geographically | No — INRC2 has explicit succession rules; airline has geographic connectivity |
| **Worker contract / work rules** | `InrcContract` — min/max assignments, consecutive work/off, max weekends | Implicit in legality rules (max duty time, min rest, FDP limits) | Partial — both have work-rule limits; INRC2 makes them explicit per-worker; airline embeds them in rules |
| **Qualification check** | Skill match at assignment time (HC2) | `QualificationRule` — type rating per leg | Yes — both check worker capability against work unit requirements |
| **Geographic constraint** | None | `BaseReturnRule` — pairing must start and end at crew base | No — airline-specific |
| **Temporal structure of APU** | Day index (integer) — no wall-clock time | `DateTime<Utc>` — precise timestamps throughout | No — INRC2 is day-indexed; airline is timestamp-precise |

---

## 3. Shared Concepts at the Interface Level

The following concepts are genuinely shared and can be expressed in a common planning interface without semantic compromise:

**3.1 Worker**
Both domains have a worker entity with an identity and a capability set. The capability set constrains which work units the worker may be assigned to. The interface concept is:

```
Worker {
    id: WorkerId,
    capabilities: Vec<Capability>,   // skills (INRC2) or qualifications (airline)
}
```

**3.2 Atomic Planning Unit (APU)**
Both domains have a unit that the optimizer assigns to a worker. In INRC2 this is a shift assignment `(shift_type, day)`; in airline this is a `Pairing`. The interface concept is:

```
PlanningUnit {
    id: PlanningUnitId,
    required_capabilities: Vec<Capability>,   // skill (INRC2) or aircraft type (airline)
}
```

The optimizer's job is to produce a mapping `Worker → Vec<PlanningUnit>` that satisfies legality and optimizes quality.

**3.3 Coverage Demand**
Both domains express work that must be covered. The interface concept is:

```
CoverageDemand {
    unit: PlanningUnitId,
    minimum: usize,
    // optimal: usize  — present in INRC2, absent in airline (binary coverage)
}
```

The `optimal` field is INRC2-specific. The interface can carry it as `Option<usize>` without forcing the airline domain to use it.

**3.4 Planning Scenario**
The top-level input to the optimizer in both domains is a scenario that bundles workers, work units, and coverage demand:

```
PlanningScenario {
    id: ScenarioId,
    workers: Vec<Worker>,
    units: Vec<PlanningUnit>,
    demand: Vec<CoverageDemand>,
}
```

**3.5 Planning Solution**
The output is an assignment of workers to units:

```
PlanningSolution {
    scenario_id: ScenarioId,
    assignments: Vec<(WorkerId, PlanningUnitId)>,
}
```

---

## 4. Domain-Specific Concepts (Not Shared)

The following concepts are domain-specific and must remain in the domain library. They must not be forced into the platform interface.

**INRC2-specific:**
- `InrcShiftType.min_consecutive` / `max_consecutive` — shift-type-level run constraints
- `InrcForbiddenSuccession` — explicit shift-type transition rules
- `InrcContract` — per-worker work-rule templates (min/max assignments, consecutive work/off, max weekends, complete weekends)
- `InrcNurseHistory` — carry-over state from previous planning weeks
- `InrcRequirementLevel.optimal` — graded coverage (minimum vs. optimal staffing)
- `InrcConstraintId` enum — INRC2-specific constraint taxonomy (HC1–HC4, S1–S8)
- `ObjectiveWeights` — weighted penalty aggregation (INRC2 scoring convention)
- Day-indexed temporal model (no wall-clock timestamps)

**Airline-specific:**
- `FlightLeg` — concrete physical flight event with origin, destination, timestamps, aircraft type
- `Duty` — ordered sequence of legs forming a single work block
- `Pairing` — base-to-base trip with rest periods between duties
- `Rotation` — crew member's full sequence of pairings
- `BaseReturnRule` — geographic home-base constraint
- `DutyConnectivityRule` — geographic leg-to-leg connectivity within a duty
- `FlightDutyPeriodRule` (FDP) — aviation-specific fatigue regulation
- `QualificationRule` — aircraft type rating per leg
- `AirportCode`, `AircraftType`, `FlightNumber` — aviation identifiers
- `PlanningPeriod` with `DateTime<Utc>` — timestamp-precise temporal model
- `LegalityRule` / `LegalityChecker` architecture — separated hard-constraint layer
- `SchedulingObjective` trait — separated soft-objective layer

---

## 5. OQ Resolutions

### OQ-3 Resolution: Atomic Planning Unit in each domain

**INRC2**: The Atomic Planning Unit is a **shift assignment** — the triple `(nurse_id, shift_type_id, day_index)`. The `InrcShiftType` is the work unit definition; the assignment is the act of placing a nurse into that shift on a specific day. The shift is atomic: there is no sub-structure below it.

**Airline**: The Atomic Planning Unit is a **Pairing** — a base-to-base trip. `FlightLeg` and `Duty` are sub-components of a pairing; they are not the unit of assignment. The optimizer assigns pairings to crew members.

**Decision**: OQ-3 is resolved. The comparison must be made at the `(Worker, AssignableUnit)` level, where `AssignableUnit` is `ShiftAssignment` in INRC2 and `Pairing` in airline. This confirms the framing in Principle 7 and D-4.

---

### OQ-1 Resolution: Long-term role of `coralys-scheduling`

**Evidence**: `coralys-scheduling` currently contains the full airline domain model — `FlightLeg`, `Duty`, `Pairing`, `Rotation`, `Roster`, `CrewMember`, the legality layer, and the optimization layer. Its own module documentation says "Airline crew scheduling domain model." It has no dependency on INRC2 or generic workforce types.

**Finding**: The airline domain model is rich, well-structured, and airline-specific. The concepts in `coralys-scheduling` (`FlightLeg`, `Duty`, `Pairing`, `BaseReturnRule`, `FDP`) are not generic scheduling abstractions — they are airline domain types. Renaming the crate to `coralys-scheduling` and treating it as a generic framework would require either (a) forcing airline-specific types into a generic interface, or (b) leaving the crate name misleading.

**Decision**: **Option A — Permanently airline-specific.** `coralys-scheduling` should be renamed to `adapters/airline` (or `coralys-airline`) to accurately reflect its role as the airline domain library. The generic planning capability belongs in a new platform crate (see OQ-2). This decision is consistent with Principle 5 (distinguish domain libraries from products) and Principle 6 (name things for their long-term role).

**Rationale**: The generic planning interface identified in Section 3 (`PlanningScenario`, `Worker`, `PlanningUnit`) is not currently in `coralys-scheduling`. It should be created in a new platform crate. `coralys-scheduling` should become the airline domain library, parallel to `adapters/ultracrew` as the INRC2 domain library.

---

### OQ-2 Resolution: Home of the generic planning capability

**Evidence**: The shared interface identified in Section 3 (`PlanningScenario`, `Worker`, `PlanningUnit`, `CoverageDemand`, `PlanningSolution`) is genuinely shared across both domains. It does not belong in either domain library. Placing it in `adapters/ultracrew` would make UltraCrew a de facto platform (Principle 1 violation). Placing it in `coralys-scheduling` would conflate the airline domain library with the platform.

**Decision**: **Candidate A — New platform crate.** The generic planning capability belongs in a new platform crate. The dependency direction is:

```
adapters/ultracrew  →  coralys-planning  →  coralys-core
adapters/airline    →  coralys-planning
```

This satisfies Principle 1 (platform crates never depend upward), AI-3 (two independent implementations before promotion — both INRC2 and airline now provide the second implementation), and D-5 (planning capability is about Resources broadly, not workforce specifically).

---

### OQ-4 Resolution: Name of the generic interface

**Evidence**: The shared concepts are `PlanningScenario`, `Worker`, `PlanningUnit`, `CoverageDemand`, `PlanningSolution`. The word "scheduling" is already used by `coralys-scheduling` (airline domain). The word "workforce" is too narrow (D-5: the platform must support non-human resources). The word "assignment" is accurate but describes only one aspect of the problem.

**Decision**: **`coralys-planning`** — the new platform crate name. The core trait is `PlanningScenario`. This name:
- Is neutral with respect to domain (not workforce-specific, not airline-specific)
- Accurately describes the capability (allocating resources to work over a horizon)
- Is consistent with D-7 (planning answers *who/what performs the work?*)
- Does not conflict with `coralys-scheduling` (which will be renamed to `adapters/airline`)
- Leaves room for `coralys-scheduling` to become a sub-concern of planning if needed in future

---

## 6. Candidate Interface Sketch

This is a sketch only. It is not a proposal for immediate implementation. Phase 2 will refine and implement it.

```rust
// coralys-planning/src/lib.rs  (proposed)

/// Opaque identifier for a worker.
pub struct WorkerId(String);

/// A capability tag — skill name (INRC2) or aircraft type rating (airline).
pub struct Capability(String);

/// A worker with an identity and a capability set.
pub struct Worker {
    pub id: WorkerId,
    pub capabilities: Vec<Capability>,
}

/// Opaque identifier for a planning unit.
pub struct PlanningUnitId(String);

/// The atomic unit assigned to a worker by the optimizer.
///
/// Domain libraries define the concrete type (ShiftAssignment, Pairing, etc.)
/// and implement this trait.
pub trait PlanningUnit {
    fn id(&self) -> &PlanningUnitId;
    fn required_capabilities(&self) -> &[Capability];
}

/// Coverage demand for a planning unit.
pub struct CoverageDemand {
    pub unit_id: PlanningUnitId,
    pub minimum: usize,
    pub optimal: Option<usize>,   // None = binary coverage (airline); Some = graded (INRC2)
}

/// Opaque identifier for a planning scenario.
pub struct ScenarioId(String);

/// The top-level input to the optimizer.
pub trait PlanningScenario {
    fn id(&self) -> &ScenarioId;
    fn workers(&self) -> &[Worker];
    fn demand(&self) -> &[CoverageDemand];
}

/// A single assignment: worker W is assigned planning unit U.
pub struct Assignment {
    pub worker_id: WorkerId,
    pub unit_id: PlanningUnitId,
}

/// The output of the optimizer.
pub struct PlanningSolution {
    pub scenario_id: ScenarioId,
    pub assignments: Vec<Assignment>,
}
```

**What this interface does not include** (intentionally):
- Temporal model — day-indexed (INRC2) vs. timestamp-precise (airline) is domain-specific
- Constraint architecture — `InrcConstraintId` / `ObjectiveWeights` vs. `LegalityRule` / `SchedulingObjective` is domain-specific
- History / carry-over state — INRC2-specific
- Geographic constraints — airline-specific
- Sub-structure within the APU — airline-specific (`Duty`, `FlightLeg`)

The platform interface is intentionally thin. Domain libraries implement `PlanningScenario` and `PlanningUnit` and add all domain-specific richness below the interface boundary.

---

## 7. Exit Evidence Checklist

- [x] Domain comparison matrix completed (INRC2 vs airline, at Atomic Planning Unit level) — Section 2
- [x] OQ-1 resolved: `coralys-scheduling` long-term scope decided — Section 5, OQ-1: Option A (rename to `adapters/airline`)
- [x] OQ-2 resolved: home of the generic planning capability decided — Section 5, OQ-2: new `coralys-planning` platform crate
- [x] OQ-3 resolved: Atomic Planning Unit identified in each domain — Section 5, OQ-3: shift assignment (INRC2), Pairing (airline)
- [x] OQ-4 resolved: candidate name for the generic interface agreed — Section 5, OQ-4: `coralys-planning`
- [x] Candidate interface sketch reviewed and accepted — Section 6

**Phase 1 is complete. Phase 2 may begin.**

---

## 8. Phase 2 Preconditions Satisfied

All exit criteria from [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md) Phase 1 are met:

| Criterion | Status | Evidence |
|---|---|---|
| Domain comparison matrix | Complete | Section 2 |
| OQ-1 resolved | Complete | Section 5 — Option A |
| OQ-2 resolved | Complete | Section 5 — `coralys-planning` |
| OQ-3 resolved | Complete | Section 5 — shift assignment / Pairing |
| OQ-4 resolved | Complete | Section 5 — `coralys-planning` |
| Candidate interface sketch | Complete | Section 6 |

Phase 2 work: create `coralys-planning` crate, define the `PlanningScenario` and `PlanningUnit` traits, implement for `InrcScenario`, verify all INRC2 benchmark parity tests pass (AI-2), rename `coralys-scheduling` to `adapters/airline` if OQ-1 Option A is accepted by the team.

---

*Document frozen 2026-07-21. No further edits without a new architectural decision record.*