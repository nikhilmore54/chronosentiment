# UltraCrew / Coralys Platform Boundary Assessment

**Status:** WORKING DOCUMENT — Phase C-B Engineering Task 1  
**Date:** 2026-07-21  
**Purpose:** Map the current Coralys/UltraCrew boundary, identify what belongs where, and define the extraction plan before any code changes.

---

## 1. Current State — What Exists Today

### Layer 1: Coralys Platform Crates (pure platform, no domain)

| Crate | Responsibility |
|-------|---------------|
| `coralys-moga` | MOGA evolution engine — `EvolutionEngine`, `Genome`, `Evaluator`, `FitnessVector` |
| `coralys-core` | Core primitives |
| `coralys-decision` | Decision intelligence primitives |
| `coralys-recommendation` | Recommendation engine primitives |
| `coralys-simulation` | Simulation primitives |
| `coralys-ecology` | Workforce ecology (fatigue, historical hours) |
| `coralys-policy` | Policy engine |
| `coralys-scheduling` | Scheduling primitives |
| `coralys-infrastructure` | Infrastructure utilities |
| `coralys-matching` | Matching engine |

**Assessment:** These crates are correctly positioned. They contain no HTTP, no JSON serialization, no domain-specific concepts (nurses, shifts, INRC). They are the platform.

---

### Layer 2: UltraCrew Adapter Crate (`adapters/ultracrew` → `ultracrew` lib)

| Module | Responsibility | Layer Assessment |
|--------|---------------|-----------------|
| `optimization.rs` | `ScheduleContext`, `ScheduleOptimizer`, `ScheduleEvaluation` — wires Coralys MOGA to workforce domain | ✅ Correct layer — domain adapter |
| `ecology.rs` | `WorkforceEcology` — wraps `coralys-ecology` for workforce | ✅ Correct layer |
| `models.rs` | `Worker`, `Shift`, `Skill` — domain models | ✅ Correct layer |
| `constraint_engine.rs` | `ConstraintEngine`, `ConstraintReport` — workforce constraint evaluation | ✅ Correct layer |
| `recommendation.rs` | `RecommendationEngine`, `SchedulingRecommendation` — workforce recommendations | ✅ Correct layer |
| `decision_intelligence.rs` | `analyze_solution`, `generate_insights` — workforce decision analytics | ✅ Correct layer |
| `pipeline.rs` | `run_pipeline` — orchestrates MOGA + constraint evaluation | ✅ Correct layer |
| `schedule_solution.rs` | `ScheduleSolution` — output contract | ✅ Correct layer |
| `public_contracts.rs` | `ScheduleRequest`, `RescheduleRequest`, `ValidateRequest` — API input contracts | ✅ Correct layer |
| `generic_import.rs` | Generic CSV/JSON import → `ScheduleRequest` | ✅ Correct layer |
| `generic_export.rs` | `GenericExporter` — CSV/JSON export from `ScheduleSolution` | ✅ Correct layer |
| `inrc/` | INRC parser, models, optimizer — INRC-specific domain | ✅ Correct layer — domain-specific adapter |
| `helpers.rs` | Scenario generation helpers | ✅ Correct layer |
| `config/` | Configuration | ✅ Correct layer |
| `workforce/` | Workforce domain utilities | ✅ Correct layer |

**Assessment:** The `adapters/ultracrew` crate is correctly positioned. It is the UltraCrew domain adapter — it knows about workers, shifts, INRC, and Coralys, but not about HTTP or persistence.

---

### Layer 3: UltraCrew Server (`services/ultracrew_server`)

#### 3a. Modules that belong in the application layer (correct position)

| Module | Responsibility | Assessment |
|--------|---------------|------------|
| `main.rs` | Axum router, HTTP handlers, AppState, CORS | ✅ Application layer — stays here |
| `models.rs` | `DecisionCase`, `ScheduleVersion`, `DecisionLog` — application-level entities | ✅ Application layer — stays here |
| `persistence.rs` | File-based JSON persistence for decision cases | ✅ Application layer — stays here |

#### 3b. Modules that are INRC-domain logic (currently in server, should be in adapter)

| Module | Responsibility | Current Problem | Recommended Move |
|--------|---------------|-----------------|-----------------|
| `simulation.rs` | INRC sick-leave simulation, recovery planning, dashboard generation, roster health | Contains domain logic (sick leave, recovery, balance tracking) mixed with HTTP-facing DTOs | Domain logic → `adapters/ultracrew/src/inrc/simulation.rs`; DTOs stay in server or move to `public_contracts` |
| `optimizer.rs` | `ScheduleGenome`, `UltraCrewEvaluator`, `UltraCrewMutator` — INRC-specific MOGA genome | INRC-specific optimization genome belongs in the INRC adapter | → `adapters/ultracrew/src/inrc/optimizer.rs` |
| `validator.rs` | `validate_schedule` — INRC constraint validation | INRC-specific validation logic | → `adapters/ultracrew/src/inrc/validator.rs` |
| `builder.rs` | Schedule builder utilities | Likely INRC-specific | Assess and move to adapter |
| `tracker.rs` | Tracking utilities | Assess — may be application-level | Keep or move based on content |
| `inrc_observer.rs` | INRC-specific observer | INRC domain — belongs in adapter | → `adapters/ultracrew/src/inrc/observer.rs` |

---

## 2. The Real Boundary Problem

The architecture is **mostly correct already**. The `adapters/ultracrew` crate correctly separates domain logic from the platform. The `coralys-*` crates correctly contain pure platform primitives.

The actual boundary violation is narrower than it appears:

**`ultracrew_server` contains INRC domain logic** (sick-leave simulation, INRC-specific genome, INRC validator) that belongs in `adapters/ultracrew/src/inrc/`. This is a **domain logic leak into the application layer**, not a platform/application confusion.

The `main.rs` handlers themselves are correctly thin — they call into `ultracrew::*` and `ultracrew_server::simulation::*`. The problem is that `simulation.rs`, `optimizer.rs`, `validator.rs` are in the server crate instead of the adapter crate.

---

## 3. What the Extraction Actually Entails

### Move 1: `optimizer.rs` → `adapters/ultracrew/src/inrc/optimizer.rs`
- `ScheduleGenome`, `UltraCrewEvaluator`, `UltraCrewMutator`
- These implement `coralys_moga` traits for the INRC domain
- Zero HTTP dependency — clean move
- **Risk:** Low. Pure Rust struct/trait implementations.

### Move 2: `validator.rs` → `adapters/ultracrew/src/inrc/validator.rs`
- `validate_schedule` function
- INRC constraint validation logic
- Zero HTTP dependency — clean move
- **Risk:** Low.

### Move 3: `simulation.rs` — Partial split
- **Domain logic** (sick-leave algorithm, recovery planning, balance tracking) → `adapters/ultracrew/src/inrc/simulation.rs`
- **HTTP-facing DTOs** (`Dashboard`, `RosterHealth`, `Alert`, `SimulationState`, etc.) → stay in server or move to a dedicated `ultracrew_server::dto` module
- **Risk:** Medium. The DTOs and domain logic are currently interleaved. Requires careful separation.

### Move 4: `inrc_observer.rs` → `adapters/ultracrew/src/inrc/observer.rs`
- INRC-specific observer
- **Risk:** Low.

### What stays in `ultracrew_server`
- `main.rs` — HTTP handlers, router, AppState
- `models.rs` — `DecisionCase`, `ScheduleVersion` (application entities)
- `persistence.rs` — file persistence
- `tracker.rs` — assess content first
- All DTOs used in HTTP responses

---

## 4. API Surface Impact

**Zero.** The HTTP API surface (`/api/health`, `/api/nurses`, `/api/schedule`, `/api/export/*`, `/api/dashboard`, etc.) does not change. The extraction is purely internal — moving Rust modules between crates within the same Cargo workspace. The `main.rs` handlers call the same functions; they just import from `ultracrew::inrc::*` instead of `ultracrew_server::*`.

Demo Assessment v1.2 remains valid. All Phase B acceptance criteria remain satisfied.

---

## 5. Recommended Execution Order

Given the risk profile and the constraint that the API surface must not change:

| Step | Action | Risk | Effort |
|------|--------|------|--------|
| 1 | Move `optimizer.rs` → `adapters/ultracrew/src/inrc/optimizer.rs` | Low | 1–2h |
| 2 | Move `validator.rs` → `adapters/ultracrew/src/inrc/validator.rs` | Low | 1h |
| 3 | Move `inrc_observer.rs` → `adapters/ultracrew/src/inrc/observer.rs` | Low | 1h |
| 4 | Split `simulation.rs` — domain logic to adapter, DTOs stay in server | Medium | 3–4h |
| 5 | Verify build clean + re-run Phase B workflow exercise | Low | 30min |

Total estimated effort: ~6–8 hours engineering time.

---

## 6. What This Does NOT Require

- No new crates need to be created. The `adapters/ultracrew` crate already exists and is the right home.
- No `coralys-*` crates need to change. They are already correctly positioned.
- No API changes. No frontend changes. No Dockerfile changes.
- No governance document updates (the architecture baseline already describes this separation correctly).

---

## 7. Decision

**Proceed with extraction as Phase C-B Engineering Task 2**, in the order defined in Section 5.

The extraction is low-risk, improves the codebase's conformance to the already-frozen architecture baseline, and has zero external impact. It is the right first engineering task of Phase C-B.

---

*This document is a working assessment. It will be superseded by the completion record once extraction is done.*