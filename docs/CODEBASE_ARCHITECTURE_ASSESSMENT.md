# Codebase Architecture Assessment

> **Date**: 2026-07-19  
> **Version**: 2.0 — revised after peer review  
> **Purpose**: Ground the product portfolio strategy in the actual implementation.  
> **Scope**: Full workspace inventory, domain model assessment, boundary analysis, refactoring recommendations.  
> **Status**: Supersedes v1.0. Corrections applied to three recommendations following architectural review.

---

## Architectural Principles

These principles govern all recommendations in this document. They are stated first because they constrain the migration plan.

**Principle 1 — Platform crates never depend on products.**  
The dependency direction is always: Product → Platform. Never the reverse.

**Principle 2 — Products depend on interfaces, not algorithms.**  
A product such as UltraCrew depends on a `WorkforceScenario` trait, not on a specific optimization algorithm. The MOGA engine is injected, not imported directly.

**Principle 3 — Benchmarks are permanent.**  
Research implementations are never discarded. The INRC2 benchmark suite is a regression suite, a validation dataset, a performance benchmark, and research evidence. It is preserved in full regardless of any refactoring.

**Principle 4 — Generalize after two concrete implementations, not one.**  
A shared abstraction extracted from a single implementation is shaped almost entirely by that implementation and is likely to be a poor fit for the second. The correct sequence is: implement concretely twice, compare, then extract the shared abstraction. This principle directly governs the timing of the generic workforce model.

**Principle 5 — Distinguish domain implementations from products.**  
A domain model (e.g. the airline scheduling domain) is not the same as a product (e.g. AirlineOps). The domain model is a library. The product is the application layer built on top of it. These are separate concerns and should be separate crates.

---

## 1. Current Architecture — What Actually Exists

The workspace is a Cargo workspace defined in [`Cargo.toml`](../Cargo.toml). The actual members are:

```
infrastructure/core
infrastructure/optimization
infrastructure/observatory/api
financial/ese
financial/strategies
financial/core
coralys-moga
coralys-simulation
coralys-ecology
coralys-decision
coralys-recommendation
coralys-infrastructure
adapters/ultracrew          ← primary UltraCrew library (INRC2 solver)
adapters/chronosentiment
adapters/cvrp
adapters/cvd001
adapters/roadef
coralys-v2
coralys-core
coralys-eval
coralys-matching
coralys-scheduling          ← airline domain model (name is ambiguous — see §3c)
services/ultracrew_server
services/cvrp_server
```

**What does not exist** (assumed in earlier discussions but absent from the workspace):
- `services/chrono_server` — not in workspace manifest
- `adapters/inrc` — not in workspace (INRC logic lives inside `adapters/ultracrew/src/inrc/`)
- `adapters/airline` — not in workspace
- `coralys-policy` — directory exists but not a workspace member

---

## 2. Platform Inventory

### 2a. Mature Platform Crates

| Crate | Status | Assessment |
|---|---|---|
| `coralys-moga` | ✅ Mature | Full MOGA engine: ecology, traits, benchmarks, falsification tests, observatory. Genuinely generic. |
| `coralys-core` | ✅ Mature | Generic traits: `Scenario`, `Solution`, `Outcome`, `State`, `Action`, `DecisionPlugin`, `ReasoningEngine`. No domain coupling. |

### 2b. Stub Platform Crates

These crates exist in the workspace but contain only 1–5 files, mostly trait definitions with no implementation:

| Crate | Files | Assessment |
|---|---|---|
| `coralys-ecology` | 6 | Traits and models only. Ecology logic lives in `coralys-moga/src/ecology/`. |
| `coralys-eval` | 5 | Adapter, pipeline, registry, types. Thin. |
| `coralys-decision` | 2 | `lib.rs` + `traits.rs`. Stub. |
| `coralys-simulation` | 2 | `lib.rs` + `traits.rs`. Stub. |
| `coralys-recommendation` | 3 | `lib.rs`, `recommender.rs`, `traits.rs`. Stub. |
| `coralys-matching` | Unknown | Not fully read; likely stub. |
| `coralys-infrastructure` | 1 | `lib.rs` only. Stub. |
| `coralys-v2` | 2 | `lib.rs` only. Stub. |

### 2c. Infrastructure Crates

| Crate | Assessment |
|---|---|
| `infrastructure/core` | Shared infrastructure primitives |
| `infrastructure/optimization` | Optimization infrastructure |
| `infrastructure/observatory/api` | Observatory API |

---

## 3. Product Inventory

### 3a. UltraCrew — `adapters/ultracrew`

**Current state**: A mature, INRC2-specific nurse rostering solver.

**Target state**: A generic workforce rostering decision platform. The generic layer does not yet exist as code.

**Evidence of current state**:
- [`src/inrc/`](../adapters/ultracrew/src/inrc/) contains: `models.rs`, `parser.rs`, `evaluator.rs`, `exporter.rs`, `history.rs`, `bipartite_matching.rs`, `audit.rs`, `optimization.rs`
- [`src/workforce/`](../adapters/ultracrew/src/workforce/) contains: `mod.rs`, `ecology_adapter.rs`, `workforce_metrics.rs` — thin stubs, not functional
- Test data: INRC2 benchmark instances for n030w4, n040w4, n050w4, n060w8, n080w8, n100w4, n120w8
- Benchmark CSVs: `inrc_m22_benchmark.csv`, `ablation_matrix_30seed.csv`, `ecology_response_curve_30seed.csv`
- Binary tools: `ultracrew-cli.rs`, `inrc_ecology_ablation_matrix.rs`, `inrc_m22_ancestry.rs`, etc.

**Domain model** (from [`src/inrc/models.rs`](../adapters/ultracrew/src/inrc/models.rs)): `InrcScenario`, nurses, shifts, contracts, skills, coverage requirements — all INRC2 types.

**Genericity level**: Low. The `src/workforce/` module is a thin wrapper that does not abstract the INRC model.

---

### 3b. UltraCrew Server — `services/ultracrew_server`

**Current state**: An HTTP server wrapping the INRC2 solver.

**Target state**: A generic workforce rostering service, decoupled from INRC2 types.

**Evidence of current state**:
- [`src/optimizer.rs`](../services/ultracrew_server/src/optimizer.rs) line 5: `use ultracrew::inrc::models::InrcScenario` — direct INRC import
- `UltraCrewEvaluator` holds an `InrcScenario` field
- Fitness function uses INRC constraint names: `min_assignments`, `max_working_weekends`, `complete_weekends`
- [`src/models.rs`](../services/ultracrew_server/src/models.rs) line 57 comment: `// nurse_id -> daily shifts`
- `src/inrc_observer.rs`: INRC-specific observer
- Benchmark output files: `benchmark_results_n030w4.csv`, `benchmark_results_n050w4.csv`, `benchmark_results_n080w8.csv`

---

### 3c. `coralys-scheduling` — Airline Domain Model (name is ambiguous)

**Current state**: Contains a fully airline-specific domain implementation.

**Open question**: Is this crate intended to be the permanent airline domain crate, or was it created as a generic scheduling framework with the airline domain as the first implementation?

The crate's own documentation says: *"Airline crew scheduling domain model for the Coralys platform."* The public API exports `FlightLeg`, `Duty`, `Pairing`, `Rotation`, `FDP`, `AircraftType`, `AirportCode`, `CrewMember`, `CrewRole`. The legality module contains `fdp.rs`, `duty_time.rs`, `base_return.rs`, `qualification.rs`.

**Two possible futures**:

Option A — Permanent airline scope: rename to `coralys-airline` or `adapters/airline`. Correct if the intent is for this crate to remain airline-specific forever.

Option B — Generic scheduling framework: preserve the name `coralys-scheduling` and eventually extract generic scheduling abstractions, with the airline types becoming one concrete implementation.

**Recommendation**: Defer this decision. The inventory tells us what the crate contains, not what role it is intended to play long-term. Renaming now would be a permanent architectural commitment made without sufficient evidence. The decision should be revisited after the generic workforce model has stabilized (Phase 1) and the comparison between INRC and airline domains has been completed (Phase 1 validation). See Principle 4.

---

### 3d. CVRP — `adapters/cvrp` + `services/cvrp_server`

A vehicle routing (CVRP) solver. Separate domain. Not relevant to workforce or airline products.

---

### 3e. ChronoSentiment — `financial/` + `adapters/chronosentiment`

Financial decision intelligence. `financial/ese`, `financial/strategies`, `financial/core`. Separate domain.

---

## 4. Domain Model Assessment

### The naming problem

The codebase has a significant naming inversion:

| Name | What it says | What it actually is |
|---|---|---|
| `coralys-scheduling` | Generic scheduling platform | Airline crew scheduling domain implementation (intent unclear) |
| `adapters/ultracrew` | An adapter | The primary UltraCrew product library |
| `src/workforce/` in ultracrew | Generic workforce layer | Thin stub over INRC types |

This inversion is the root cause of confusion in earlier product strategy discussions. The "generic workforce rostering platform" does not yet exist as code. What exists is a mature INRC2 nurse rostering solver.

### What the INRC2 model covers

The INRC2 model in [`adapters/ultracrew/src/inrc/`](../adapters/ultracrew/src/inrc/) covers:

- Nurses (id, name, skills, contract)
- Shifts (type, duration, forbidden successors)
- Contracts (min/max hours, max consecutive shifts, min rest)
- Coverage requirements (skill, min/preferred count per shift)
- History (previous week's assignments for multi-week planning)
- Soft constraints (S1–S7 in INRC2 scoring)
- Hard constraints (H1–H4 in INRC2 scoring)

This model is generalizable to other workforce domains. The concepts map cleanly:

| INRC2 concept | Generic concept |
|---|---|
| Nurse | Worker |
| Shift type | Shift |
| Skill | Skill |
| Contract | Contract |
| Coverage requirement | Demand |
| History | Prior state |

However, per Principle 4, the generalization should not be extracted from INRC alone. The airline domain (already implemented in `coralys-scheduling`) provides the second concrete implementation. Comparing the two will produce a stronger abstraction than extracting from INRC in isolation.

---

## 5. Genericity Assessment

### ✅ Already generic — keep as-is

- `coralys-moga`: engine is domain-agnostic
- `coralys-core`: traits are domain-agnostic
- `coralys-eval`: evaluation pipeline is domain-agnostic

### 🔄 Generalizable — but only after Phase 1 validation

- `adapters/ultracrew/src/inrc/`: the INRC model can be abstracted into a generic workforce model. However, the abstraction should be extracted into a **platform crate** (`coralys-workforce`), not into `adapters/ultracrew/src/workforce/`. See §6 for the dependency direction argument.
- `services/ultracrew_server/src/models.rs`: `DecisionCase`, `ScheduleVersion`, `Recommendation` are already close to generic — the INRC coupling is in the data, not the type names.

### 📦 Defer — do not rename yet

- `coralys-scheduling`: the name is ambiguous and the long-term intent is unclear. Defer the rename decision until Phase 1 validation is complete.

### 🚫 Replace — do not build on

- `services/ultracrew_server/src/optimizer.rs` line 5: `use ultracrew::inrc::models::InrcScenario` — this direct INRC import in the server is the coupling point that prevents the server from being a generic workforce server. This import should be replaced with a generic `WorkforceScenario` trait once the generic layer exists in the platform.

### ➕ Add — does not exist yet

- `coralys-workforce`: a new **platform crate** containing the generic workforce domain model (`Worker`, `Shift`, `Skill`, `Demand`, `Contract`, `Roster`, `WorkforceScenario`). This is a platform concern, not a product concern. It should not live inside `adapters/ultracrew`.
- INRC2 adapter implementing `WorkforceScenario` — so the existing INRC solver becomes one concrete implementation of the platform interface.
- AirlineOps product layer — built on top of `coralys-scheduling` (or `coralys-airline` if renamed): pairing optimizer, crew assignment via the generic workforce capability, recovery, crew control.

---

## 6. Boundary Assessment

### Why the generic workforce model belongs in the platform, not in `adapters/ultracrew`

The v1.0 assessment proposed placing the generic workforce model inside `adapters/ultracrew/src/workforce/`. This is architecturally incorrect.

If the generic model lives inside `adapters/ultracrew`, then every future domain that needs workforce rostering (manufacturing, retail, airline crew assignment) must depend on UltraCrew. That makes UltraCrew the de facto platform, violating Principle 1.

The correct dependency graph is:

```
Manufacturing adapter
Retail adapter
Airline crew assignment (AirlineOps)
INRC2 adapter (UltraCrew)
        │
        ▼
coralys-workforce   ← platform crate (generic Worker, Shift, Skill, Demand, Contract)
        │
        ▼
coralys-core        ← platform crate (Scenario, Solution, Outcome traits)
        │
        ▼
coralys-moga        ← platform crate (MOGA engine)
```

UltraCrew is a product that uses the platform. It is not the platform.

### Current actual boundaries

```
adapters/ultracrew
  └── src/inrc/          ← INRC2 domain + solver logic (mature)
  └── src/workforce/     ← stub (3 files, not functional)
  └── src/pipeline.rs    ← optimization pipeline
  └── src/decision_intelligence.rs
  └── src/recommendation.rs

services/ultracrew_server
  └── imports ultracrew::inrc::models::InrcScenario directly
  └── INRC-specific fitness function
  └── INRC-specific observer

coralys-scheduling
  └── 100% airline domain implementation
  └── No connection to ultracrew or INRC
```

### Target boundaries (after refactoring)

```
coralys-workforce   ← NEW platform crate
  └── Worker, Shift, Skill, Demand, Contract, WorkforceRoster
  └── WorkforceScenario trait

adapters/ultracrew
  └── src/inrc/          ← INRC2 adapter implementing WorkforceScenario
  └── src/pipeline.rs    ← optimization pipeline (generic)

services/ultracrew_server
  └── imports WorkforceScenario trait (not InrcScenario directly)
  └── generic fitness function parameterized by domain adapter
  └── INRC2 adapter injected at startup

coralys-scheduling  →  decision deferred (see §3c)
  └── airline domain implementation (FlightLeg, Duty, Pairing, Rotation, Roster)
  └── legality rules (FDP, duty time, base return, qualification)
  └── optimization (neighborhood search, local search)
  └── resilience (disruption, reserve, robustness)
  └── planner (incremental, whatif, summary)
```

---

## 7. Refactoring Recommendations

### Priority 1 — Phase 1 validation: compare INRC and Airline domains (before any extraction)

Before extracting a generic workforce model, compare the INRC2 domain model with the airline domain model in `coralys-scheduling`. Identify:
- What concepts are shared (worker, shift, assignment, constraint, objective)
- What concepts are domain-specific (FDP, pairing, duty period vs. INRC contract types)
- What the minimal shared interface looks like

This comparison is the evidence base for the generic model. Without it, the abstraction will be shaped by INRC alone and may be a poor fit for airline crew assignment or other industries.

**Risk**: Low. This is analysis work, not code change.

### Priority 2 — Create `coralys-workforce` platform crate

After Phase 1 validation, create a new platform crate `coralys-workforce` containing the generic workforce domain model derived from the comparison. Add it to the workspace manifest.

```rust
// coralys-workforce/src/lib.rs
pub struct Worker { pub id: WorkerId, pub skills: Vec<Skill>, pub contract: Contract }
pub struct Shift { pub id: ShiftId, pub kind: ShiftKind, pub duration: Duration }
pub struct Demand { pub shift_id: ShiftId, pub skill: Skill, pub min: u32, pub preferred: u32 }
pub struct WorkforceRoster { pub assignments: Vec<Assignment> }
pub trait WorkforceScenario { fn workers(&self) -> &[Worker]; fn demands(&self) -> &[Demand]; }
```

**Risk**: Medium. Requires touching the INRC evaluator and mutator. The INRC model is mature and well-tested — the abstraction must not break existing benchmark parity (Principle 3).

### Priority 3 — Make `InrcScenario` implement `WorkforceScenario`

Implement the `WorkforceScenario` trait for `InrcScenario`. All existing INRC2 benchmark parity tests must continue to pass.

**Risk**: Low once Priority 2 is done.

### Priority 4 — Decouple `services/ultracrew_server` from `InrcScenario`

Replace the direct `use ultracrew::inrc::models::InrcScenario` import in [`services/ultracrew_server/src/optimizer.rs`](../services/ultracrew_server/src/optimizer.rs) with a `WorkforceScenario` bound from `coralys-workforce`. Inject the INRC adapter at startup via configuration.

**Risk**: Medium. The server's fitness function has INRC constraint names hardcoded. This requires the generic workforce model to be stable first.

### Priority 5 — Decide on `coralys-scheduling` scope

After Phase 1 validation, the comparison between INRC and airline domains will clarify whether `coralys-scheduling` should:
- Remain airline-specific and be renamed to `coralys-airline`
- Evolve into a generic scheduling framework with airline as one implementation

Make this decision then, not now.

**Risk**: Low to defer. Moderate to execute (crate rename touches Cargo.toml, package name, dependency declarations, use paths, documentation, CI, examples).

---

## 8. Migration Plan

### Phase 0 — No code changes (current)

Freeze the product portfolio document. Agree on the three-product structure. Do not rename any crates yet.

### Phase 1 — Validate common abstractions (1–2 weeks)

1. Read and compare `adapters/ultracrew/src/inrc/models.rs` and `coralys-scheduling/src/domain/` side by side.
2. Identify shared concepts: worker/crew, shift/duty, assignment, constraint, objective.
3. Identify domain-specific concepts: FDP, pairing, duty period (airline) vs. INRC contract types, forbidden successors (INRC).
4. Draft the `WorkforceScenario` interface based on the intersection.
5. Decide on `coralys-scheduling` scope (Option A or Option B from §3c).
6. Commit: *"docs: Phase 1 validation — INRC vs Airline domain comparison"*

### Phase 2 — Create `coralys-workforce` platform crate (1 week)

1. Create `coralys-workforce` crate with generic workforce domain model.
2. Add to workspace manifest.
3. Implement `WorkforceScenario` for `InrcScenario` — all existing INRC tests must pass.
4. Commit: *"feat: coralys-workforce platform crate; InrcScenario implements WorkforceScenario"*

### Phase 3 — Decouple server (1 week)

1. Replace `InrcScenario` import in `ultracrew_server/src/optimizer.rs` with `WorkforceScenario` bound.
2. Make fitness function generic over constraint weights (pass via config).
3. Inject INRC adapter at server startup.
4. Commit: *"refactor: ultracrew_server decoupled from InrcScenario"*

### Phase 4 — Stream B (ongoing)

With the generic model in place, Stream B modules (B1 Planner Workspace, B2 Disruption Console, B3 Explanation Engine, B4 Operational Readiness) are built against `WorkforceScenario` — not against `InrcScenario`. The INRC2 benchmark suite validates correctness throughout.

### Phase 5 — AirlineOps product layer (future)

Build the AirlineOps product layer on top of `coralys-scheduling` (or `coralys-airline` if renamed in Phase 1):
- Pairing optimizer
- Crew assignment (calls the generic `WorkforceScenario` capability, not UltraCrew directly)
- Recovery
- Crew control

Note: AirlineOps uses the **generic workforce rostering capability** (`coralys-workforce`), not UltraCrew the product. Whether UltraCrew's server is reused as infrastructure is a deployment decision, not an architectural one.

---

## 9. Revised Target Architecture

```
Coralys Workspace
│
├── Platform (generic, domain-agnostic)
│   ├── coralys-core          Generic traits: Scenario, Solution, Outcome, DecisionPlugin
│   ├── coralys-moga          MOGA engine: ecology, traits, benchmarks
│   ├── coralys-workforce     ← NEW: generic Worker, Shift, Skill, Demand, WorkforceScenario
│   ├── coralys-eval          Evaluation pipeline and registry
│   ├── coralys-ecology       Population dynamics (currently stub)
│   ├── coralys-decision      Decision lineage (currently stub)
│   ├── coralys-simulation    Simulation framework (currently stub)
│   ├── coralys-recommendation Recommendation engine (currently stub)
│   └── coralys-matching      Matching primitives (currently stub)
│
├── Domain Implementations
│   ├── adapters/ultracrew    INRC2 adapter implementing WorkforceScenario
│   ├── coralys-scheduling    Airline domain implementation (rename decision deferred)
│   ├── adapters/cvrp         Vehicle routing domain
│   ├── adapters/cvd001       CVD001 domain
│   └── adapters/roadef       ROADEF challenge domain
│
├── Products
│   ├── UltraCrew             Workforce Rostering Decision Platform
│   │   └── services/ultracrew_server  (decoupled from InrcScenario in Phase 3)
│   ├── AirlineOps            Airline Crew Management Platform (product layer: Phase 5)
│   └── ChronoSentiment       Financial Decision Intelligence
│       ├── financial/core
│       ├── financial/ese
│       ├── financial/strategies
│       └── adapters/chronosentiment
│
└── Research / Other
    ├── services/cvrp_server  CVRP research server
    └── apps/cvrp-playground  CVRP playground
```

---

## 10. Impact on `PRODUCT_PORTFOLIO.md`

The [`PRODUCT_PORTFOLIO.md`](../PRODUCT_PORTFOLIO.md) written earlier is directionally correct but needs two corrections before being frozen:

**Correction 1**: The document states UltraCrew is already a "Generic Workforce Rostering Decision Platform." The code shows it is currently an INRC2 nurse rostering solver with a thin workforce stub. The portfolio document should distinguish current state from target state.

**Correction 2**: The document states "AirlineOps uses UltraCrew." More precisely: AirlineOps uses the **generic workforce rostering capability** (`coralys-workforce`). Whether that capability is deployed as part of UltraCrew's server infrastructure is a deployment decision. Architecturally, AirlineOps depends on a platform interface, not on a product.

**Everything else in the portfolio document holds**:
- The three-product structure (UltraCrew, AirlineOps, ChronoSentiment) is correct.
- The integration point (AirlineOps produces pairings → generic workforce capability assigns crew) is correct.
- The capability ownership table is correct.
- The Stream B module framing (B1–B4 as generic workforce capabilities) is correct.

---

*Assessment v2.0 complete. Next step: update `PRODUCT_PORTFOLIO.md` with current-state corrections, then proceed to Phase 1 (INRC vs Airline domain comparison).*