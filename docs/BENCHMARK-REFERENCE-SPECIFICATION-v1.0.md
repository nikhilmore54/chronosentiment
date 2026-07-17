# BENCHMARK-REFERENCE-SPECIFICATION-v1.0

**Document class:** Engineering Specification  
**Status:** DRAFT — Milestone 3A (revision 2, pre-freeze)  
**Predecessor:** `data/cvd001/BENCHMARK-SEMANTICS-v1.0.md` (frozen 781499c4)  
**Successor:** `adapters/cvd001/` Rust crate (Milestone 3B)  
**Branch:** `governance-hardening`  
**Date:** 2026-07-17

---

## 1. Purpose and Scope

This document is the engineering specification for the CVD-001 benchmark adapter. It translates the semantic definitions in BENCHMARK-SEMANTICS-v1.0 into a concrete Rust implementation plan: module structure, type definitions, function signatures, evaluator execution flow, and validation oracle test map.

**Architectural position:** `adapters/cvd001` is a **Coralys benchmark adapter**. It translates CVD-001 benchmark data into Coralys evaluation concepts (solution, objective, constraint violations). It does not own those concepts — Coralys does. This separation ensures that evaluation infrastructure is not duplicated across benchmarks.

**In scope:**
- Rust crate `adapters/cvd001` — the CVD-001 adapter
- All types, functions, and modules required to compute the benchmark objective and enforce HC3
- Validation oracle test cases (O1–O8, OI1–OI2) derived from BENCHMARK-SEMANTICS §7

**Out of scope:**
- Coralys scheduling engine internals
- Optimisation algorithms (solvers, metaheuristics)
- Input parsing / file format handling (separate concern)
- Any reconstruction methodology (that work is complete and frozen at M2-FROZEN)

---

## 2. Workspace Integration

The CVD-001 adapter is a new Rust crate added to the existing workspace:

```
adapters/cvd001/
├── Cargo.toml
└── src/
    ├── lib.rs          # crate root; re-exports public API
    ├── types.rs        # domain types: FlightLeg, Duty, CrewMember, Solution,
    │                   #   ConstraintViolation, EvaluationResult
    ├── credit.rs       # duty_credit() — R1 component
    ├── workload.rs     # credited_workload() — R1 aggregate
    ├── objective.rs    # objective() — R2
    ├── hc3.rs          # hc3_feasible() — R3
    └── evaluator.rs    # evaluate() — top-level entry point
```

The crate is added to `[workspace] members` in the root `Cargo.toml`.

**Module dependency graph (strictly acyclic):**

```
types
  └─► credit
        └─► workload
              ├─► objective
              ├─► hc3
              └─► evaluator (imports all of the above)
```

No module imports from a module that depends on it. `evaluator.rs` is the only module that imports from all others.

---

## 3. Cargo.toml

```toml
[package]
name = "cvd001"
version = "0.1.0"
edition = "2021"
description = "CVD-001 benchmark adapter — airline crew scheduling (GERAD G-2014-22)"

[dependencies]
# No external dependencies for the reference adapter.
# All arithmetic is pure Rust f64.

[dev-dependencies]
# No external test dependencies required.
```

---

## 4. Type Definitions (`types.rs`)

All numeric types are `f64`. Identifiers are `u32`.

The domain model preserves the three-level hierarchy from BENCHMARK-SEMANTICS §2:
`FlightLeg → Duty → CrewMember`. Credits are pre-computed at each level by the
instance data loader; the adapter treats them as authoritative inputs.

### 4.1 `FlightLeg`

```rust
/// A single flight leg within a duty.
///
/// # Fields
/// - `id`: unique flight leg identifier (f ∈ F)
/// - `credit`: credited minutes c_f for this leg (pre-computed by loader)
/// - `duration`: actual block time d_f in minutes
///
/// # Caller responsibility
/// `credit >= 0.0`, `duration >= 0.0`.
#[derive(Debug, Clone)]
pub struct FlightLeg {
    pub id: u32,
    pub credit: f64,    // c_f — credited minutes
    pub duration: f64,  // d_f — actual block minutes
}
```

### 4.2 `Duty`

```rust
/// A duty (work period) composed of one or more flight legs.
///
/// # Fields
/// - `id`: unique duty identifier (t ∈ T)
/// - `credit`: credited minutes c_t for this duty (pre-computed by loader;
///   may differ from the sum of leg credits due to qualification rules)
/// - `legs`: the flight legs comprising this duty
///
/// # Caller responsibility
/// `credit >= 0.0`.
#[derive(Debug, Clone)]
pub struct Duty {
    pub id: u32,
    pub credit: f64,        // c_t — duty-level credited minutes
    pub legs: Vec<FlightLeg>,
}
```

### 4.3 `CrewMember`

```rust
/// A crew member with their contract parameters and assigned duties.
///
/// # Fields
/// - `id`: unique crew member identifier (n ∈ N)
/// - `min_workload`: W^min_n — contractual minimum (soft, enforced via Δ_n)
/// - `max_workload`: W^max_n — hard cap; HC3-A requires W_n <= max_workload
/// - `target_workload`: t_n — target credited minutes; Δ_n = |W_n − t_n|
/// - `duties`: duties assigned to this crew member in this solution
///
/// # Caller responsibility
/// `min_workload <= max_workload`. `target_workload >= 0.0`.
#[derive(Debug, Clone)]
pub struct CrewMember {
    pub id: u32,
    pub min_workload: f64,     // W^min_n
    pub max_workload: f64,     // W^max_n — hard cap (HC3-A)
    pub target_workload: f64,  // t_n
    pub duties: Vec<Duty>,
}
```

### 4.4 `Solution`

```rust
/// A complete solution: the full crew roster for one scheduling period.
///
/// `crew` is indexed 0..N-1. The evaluator treats this as the complete input.
#[derive(Debug, Clone)]
pub struct Solution {
    pub crew: Vec<CrewMember>,
}
```

### 4.5 `ConstraintViolation`

```rust
/// A structured record of a single constraint violation.
///
/// Carrying violations as structured data (rather than only a boolean flag)
/// allows Coralys to consume constraint information uniformly across benchmarks
/// and to support richer diagnostics and multi-constraint evaluation in future.
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    /// Human-readable constraint identifier (e.g. "HC3").
    pub constraint: &'static str,
    /// Index into `EvaluationResult::workloads` of the violating crew member.
    pub crew_member_index: usize,
    /// The crew member's id field.
    pub crew_member_id: u32,
    /// The computed workload W_n that caused the violation.
    pub workload: f64,
    /// The threshold that was exceeded (W^max_n for HC3).
    pub threshold: f64,
}
```

### 4.6 `EvaluationResult`

```rust
/// The result of evaluating a solution against the CVD-001 benchmark.
///
/// # Fields
/// - `workloads`: W_n for each crew member, same order as `solution.crew`
/// - `violations`: structured list of constraint violations (empty if feasible)
/// - `feasible`: true iff `violations` is empty
/// - `objective`: Z = Σ_n |W_n − t_n|; `f64::INFINITY` when infeasible
///
/// # Design
/// `violations` is always populated with full diagnostic information even when
/// `feasible` is false. This allows Coralys to inspect which constraints were
/// violated and by how much, without re-evaluating the solution.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub workloads: Vec<f64>,
    pub violations: Vec<ConstraintViolation>,
    pub feasible: bool,
    pub objective: f64,
}
```

---

## 5. Module Specifications

### 5.1 `credit.rs` — Duty Credit (R1 component)

**Mathematical basis:** R1 (WP-M2.2), BENCHMARK-SEMANTICS §3.

The credited workload of a duty is its `credit` field (c_t), pre-computed by the
instance data loader. The `duty_credit` function is a named point of truth.

```rust
use crate::types::Duty;

/// Return the credited minutes for a single duty.
///
/// The credited value is `duty.credit` (c_t), pre-computed by the loader.
/// This function does not re-derive the credit formula; that formula is
/// encoded in the instance data loader. See BENCHMARK-SEMANTICS §3 (R1).
///
/// # Panics
/// Panics in debug builds if `duty.credit < 0.0`.
pub fn duty_credit(duty: &Duty) -> f64 {
    debug_assert!(duty.credit >= 0.0, "duty {} has negative credit: {}", duty.id, duty.credit);
    duty.credit
}
```

### 5.2 `workload.rs` — Credited Workload (R1 aggregate)

**Mathematical basis:** R1 (WP-M2.2), BENCHMARK-SEMANTICS §3.

W_n = Σ_{t ∈ T_n} c_t

```rust
use crate::types::CrewMember;
use crate::credit::duty_credit;

/// Compute the total credited workload W_n for one crew member.
///
/// W_n = Σ_{d ∈ duties_n} duty_credit(d)
///
/// Returns 0.0 for a crew member with no duties.
pub fn credited_workload(member: &CrewMember) -> f64 {
    member.duties.iter().map(|d| duty_credit(d)).sum()
}
```

### 5.3 `objective.rs` — Objective Function (R2)

**Mathematical basis:** R2 (WP-M2.3), BENCHMARK-SEMANTICS §4.

Z = Σ_{n ∈ N} Δ_n, where Δ_n = |W_n − t_n|. (α = 0, β = 1 for benchmark adapter.)

```rust
use crate::types::CrewMember;

/// Compute Δ_n = |W_n − t_n| for one crew member.
pub fn workload_deviation(member: &CrewMember, workload: f64) -> f64 {
    (workload - member.target_workload).abs()
}

/// Compute Z = Σ_n Δ_n.
///
/// `crew` and `workloads` must have equal length.
///
/// # Panics
/// Panics if `crew.len() != workloads.len()`.
pub fn objective(crew: &[CrewMember], workloads: &[f64]) -> f64 {
    assert_eq!(crew.len(), workloads.len(),
        "crew and workloads slices must have equal length: {} vs {}",
        crew.len(), workloads.len());
    crew.iter()
        .zip(workloads.iter())
        .map(|(m, &w)| workload_deviation(m, w))
        .sum()
}
```

### 5.4 `hc3.rs` — Hard Constraint HC3 (R3)

**Mathematical basis:** R3 (WP-M2.4), BENCHMARK-SEMANTICS §5.

HC3-A: W_n ≤ W^max_n for all n ∈ N.

```rust
use crate::types::{CrewMember, ConstraintViolation};

/// Check HC3 for a single crew member.
/// Returns `Some(ConstraintViolation)` if violated, `None` if satisfied.
pub fn hc3_check_member(
    member: &CrewMember,
    workload: f64,
    index: usize,
) -> Option<ConstraintViolation> {
    if workload > member.max_workload {
        Some(ConstraintViolation {
            constraint: "HC3",
            crew_member_index: index,
            crew_member_id: member.id,
            workload,
            threshold: member.max_workload,
        })
    } else {
        None
    }
}

/// Collect all HC3 violations across the crew roster.
///
/// Returns an empty Vec if all crew members satisfy W_n <= W^max_n.
/// Returns one ConstraintViolation per violating crew member.
///
/// # Panics
/// Panics if `crew.len() != workloads.len()`.
pub fn hc3_violations(crew: &[CrewMember], workloads: &[f64]) -> Vec<ConstraintViolation> {
    assert_eq!(crew.len(), workloads.len(),
        "crew and workloads slices must have equal length: {} vs {}",
        crew.len(), workloads.len());
    crew.iter()
        .zip(workloads.iter())
        .enumerate()
        .filter_map(|(i, (m, &w))| hc3_check_member(m, w, i))
        .collect()
}

/// Convenience predicate: true iff no HC3 violations exist.
pub fn hc3_feasible(crew: &[CrewMember], workloads: &[f64]) -> bool {
    hc3_violations(crew, workloads).is_empty()
}
```

### 5.5 `evaluator.rs` — Top-Level Entry Point

**Mathematical basis:** All of R1–R4; BENCHMARK-SEMANTICS §6 (Evaluator Flow).

```rust
use crate::types::{Solution, EvaluationResult};
use crate::workload::credited_workload;
use crate::hc3::hc3_violations;
use crate::objective::objective;

/// Evaluate a solution against the CVD-001 benchmark.
///
/// # Execution flow
/// 1. Compute W_n for each crew member.
/// 2. Collect HC3 violations (W_n > W^max_n).
///    If any: return infeasible result (violations populated, objective = INFINITY).
/// 3. Compute Z = Σ_n |W_n − t_n|.
/// 4. Return feasible result (violations empty, objective = Z).
pub fn evaluate(solution: &Solution) -> EvaluationResult {
    let workloads: Vec<f64> = solution.crew.iter().map(|m| credited_workload(m)).collect();
    let violations = hc3_violations(&solution.crew, &workloads);

    if !violations.is_empty() {
        return EvaluationResult {
            workloads,
            violations,
            feasible: false,
            objective: f64::INFINITY,
        };
    }

    let z = objective(&solution.crew, &workloads);
    EvaluationResult {
        workloads,
        violations: vec![],
        feasible: true,
        objective: z,
    }
}
```

### 5.6 `lib.rs` — Crate Root

```rust
//! CVD-001 benchmark adapter for Coralys.
//!
//! Translates CVD-001 benchmark data (GERAD G-2014-22, Kasirzadeh/Saddoune/Soumis)
//! into Coralys evaluation concepts. Mathematical reconstruction documented in
//! data/cvd001/WP-M2.1 through WP-M2.6 and data/cvd001/BENCHMARK-SEMANTICS-v1.0.md.
//!
//! # Public API
//! - [`types`]: domain types (FlightLeg, Duty, CrewMember, Solution,
//!              ConstraintViolation, EvaluationResult)
//! - [`evaluator::evaluate`]: top-level entry point
//! - [`credit::duty_credit`]: single-duty credit
//! - [`workload::credited_workload`]: per-crew-member workload
//! - [`objective::objective`]: benchmark objective Z
//! - [`hc3::hc3_violations`]: HC3 constraint violation collection
//! - [`hc3::hc3_feasible`]: HC3 feasibility predicate

pub mod types;
pub mod credit;
pub mod workload;
pub mod objective;
pub mod hc3;
pub mod evaluator;

pub use types::{FlightLeg, Duty, CrewMember, Solution, ConstraintViolation, EvaluationResult};
pub use evaluator::evaluate;
```

---

## 6. Evaluator Execution Flow (Normative)

```
Solution
  │
  ▼
[workload.rs] credited_workload(member) for each n
  │  → workloads: Vec<f64>
  │
  ▼
[hc3.rs] hc3_violations(crew, workloads)
  │
  ├─ non-empty → EvaluationResult {
  │                workloads,
  │                violations,        ← structured HC3 records
  │                feasible: false,
  │                objective: INFINITY
  │              }
  │
  └─ empty
       │
       ▼
     [objective.rs] objective(crew, workloads)
       │  → Z: f64
       │
       ▼
     EvaluationResult {
       workloads,
       violations: vec![],
       feasible: true,
       objective: Z
     }
```

---

## 7. Validation Oracle Test Map

The following test cases are derived from BENCHMARK-SEMANTICS-v1.0 §7 (Validation Oracles O1–O8) and two integration oracles (OI1, OI2). Each oracle maps to one or more unit tests in the respective module's `#[cfg(test)]` block.

### 7.1 Unit Oracles

| Oracle | Description | Module | Expected result |
|--------|-------------|--------|-----------------|
| O1 | Single duty, credit = 60.0 | `credit.rs` | `duty_credit(d) == 60.0` |
| O2 | Zero-duty crew member | `workload.rs` | `credited_workload(m) == 0.0` |
| O3 | Three duties, credits 30/45/60 | `workload.rs` | `credited_workload(m) == 135.0` |
| O4 | W_n == t_n exactly | `objective.rs` | `workload_deviation(m, w) == 0.0` |
| O5 | W_n > t_n by 10.0 | `objective.rs` | `workload_deviation(m, w) == 10.0` |
| O6 | W_n < t_n by 10.0 | `objective.rs` | `workload_deviation(m, w) == 10.0` |
| O7 | W_n == W^max_n (boundary) | `hc3.rs` | `hc3_check_member(m, w, 0) == None` |
| O8 | W_n > W^max_n by epsilon | `hc3.rs` | `hc3_check_member(m, w, 0) == Some(...)` |

### 7.2 Integration Oracles

| Oracle | Description | Module | Expected result |
|--------|-------------|--------|-----------------|
| OI1 | All crew feasible, Z = Σ Δ_n | `evaluator.rs` | `result.feasible == true`, `result.violations.is_empty()`, `result.objective == expected_z` |
| OI2 | One crew member violates HC3 | `evaluator.rs` | `result.feasible == false`, `result.violations.len() == 1`, `result.violations[0].constraint == "HC3"`, `result.objective.is_infinite()` |

### 7.3 Test Implementation Notes

- All oracles are implemented as `#[test]` functions in the respective module's `#[cfg(test)]` block.
- Oracle inputs are constructed inline (no file I/O required for unit tests).
- OI1 and OI2 are in `evaluator.rs` tests.
- Floating-point comparisons use `(a - b).abs() < 1e-9` for exact-representable values.

---

## 8. Numeric Conventions

| Convention | Value |
|------------|-------|
| Workload unit | credited minutes (f64) |
| Objective unit | credited minutes (f64) |
| Infeasible sentinel | `f64::INFINITY` |
| Boundary condition | W_n == W^max_n is **feasible** (≤, not <) |
| Empty crew | Z = 0.0, feasible = true, violations = [] |
| Empty duties | W_n = 0.0 |

---

## 9. Implementation Decisions

The following decisions translate benchmark semantics into concrete implementation choices. Traceability is to BENCHMARK-SEMANTICS-v1.0 (predecessor document); reconstruction confidence labels are not repeated here.

| Uncertainty | Implementation choice | Rationale |
|-------------|----------------------|-----------|
| Credit formula internals | Caller pre-computes; `duty.credit` is authoritative | Adapter boundary: credit formula is instance-data concern, not evaluator concern |
| Weighting coefficients α, β | α = 0, β = 1 (hardcoded) | Benchmark semantics default; Coralys may override for its own objective model |
| HC3 exact form | `workload <= max_workload` (≤, not <) | HC3-A preferred reconstruction; boundary is feasible |
| Base enforcement | Soft via Δ_n; no separate hard base check | B2 preferred reconstruction; Δ_n absorbs base deviation |
| Aggregation order | Per-crew `sum()`, no secondary transform | Preferred aggregation reconstruction |
| Constraint result | `Vec<ConstraintViolation>` (structured) | Enables Coralys to consume violations uniformly across benchmarks |

---

## 10. Configuration Control

| Field | Value |
|-------|-------|
| Document version | 1.0 |
| Status | DRAFT — awaiting freeze after Milestone 3B implementation |
| Predecessor | BENCHMARK-SEMANTICS-v1.0.md (781499c4) |
| Successor | `adapters/cvd001/` Rust crate (Milestone 3B) |
| Freeze condition | All 7 source files written, `cargo test -p cvd001` passes all oracles O1–O8 + OI1 + OI2 |
| Freeze tag | `M3-FROZEN` (applied after Milestone 3B complete) |

---

*End of BENCHMARK-REFERENCE-SPECIFICATION-v1.0*
