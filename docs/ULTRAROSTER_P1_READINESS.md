# P1 Production Hardening — Readiness Assessment

**Status:** ASSESSMENT (not implementation)  
**Date:** 2026-09-02  
**Baseline:** P0-A through P0-E CLOSED at commit 5890e967b  
**Method:** Static trace of production code paths exposed by the frozen P0 architecture

---

## Scope

This document identifies concrete production risks in the UltraCrew system as it
stands after P0 closure. It does not propose Pareto algorithm changes, alternative
semantics changes, or UI presentation changes — those are frozen. It identifies
only risks that could cause production failures, data loss, incorrect behavior, or
operational blindness.

Each risk is rated:

- **CRITICAL** — can cause server crash, data corruption, or silent wrong output
- **HIGH** — can cause request failure, user-visible error, or incorrect behavior
- **MEDIUM** — degrades reliability or observability without immediate failure
- **LOW** — technical debt with no immediate production impact

---

## Risk Inventory

### R1 — Mutex Poison on Panicking Request (CRITICAL)

**Location:** `services/ultracrew_server/src/main.rs`, multiple sites  
**Pattern:** `state.lock().unwrap()`

Every request handler acquires the `AppState` mutex with `.unwrap()`. If any
request panics while holding the lock, the mutex becomes poisoned. All subsequent
requests will also panic at `.lock().unwrap()`, taking the server down permanently
until restart.

**Affected handlers:** `schedule_handler`, `reschedule_handler`, `sick_leave_handler`,
`swap_handler`, `get_schedule_handler`, `get_solution_handler`, and others.

**Trigger:** Any panic inside a handler that holds the lock — including panics in
`run_pipeline_from_request()`, `run_pareto_pipeline()`, or any downstream code.

**Fix direction:** Replace `.unwrap()` with `.unwrap_or_else(|e| e.into_inner())`
(poison recovery) or restructure to hold the lock only for state reads/writes, not
for the full computation.

---

### R2 — Pareto Pipeline Runs Synchronously in Request Handler (HIGH)

**Location:** `services/ultracrew_server/src/main.rs:1249`  
**Pattern:**
```rust
let alternatives = ultracrew::pareto_pipeline::run_pareto_pipeline(
    context.clone(),
    seed_genome,
    pareto_steps,
);
```

`run_pareto_pipeline()` runs synchronously in the Axum request handler. For large
datasets or high `generation_limit`, this blocks the Tokio async runtime thread for
the full duration of the Pareto search (up to 200 steps × 20 diverse seeds).

**Risk:** Under concurrent load, this can exhaust the Tokio thread pool, causing
all other requests to queue indefinitely. There is no timeout on the Pareto pipeline.

**Observed runtime:** 38ms for the heterogeneous test scenario (20 workers, 200 steps).
This is acceptable for a single request but not under concurrent load.

**Fix direction:** Spawn the Pareto pipeline in a `tokio::task::spawn_blocking()`
call to avoid blocking the async runtime. Add a configurable timeout.

---

### R3 — No Error Propagation from Pareto Pipeline (HIGH)

**Location:** `adapters/ultracrew/src/pareto_pipeline.rs:214`  
**Signature:** `pub fn run_pareto_pipeline(...) -> Vec<ProductionParetoSolution>`

`run_pareto_pipeline()` returns `Vec<>`, not `Result<>`. If the pipeline encounters
an internal error (e.g., `ScheduleOptimizer::create()` fails, constraint evaluator
panics), the error is either silently swallowed or causes a panic that propagates
to the request handler.

**Risk:** Silent empty result (`alternatives=[]`) when the pipeline fails internally
is indistinguishable from a legitimate "no alternatives found" outcome. The scheduler
sees the hard stop but has no way to know whether it was a genuine Pareto result or
a pipeline failure.

**Fix direction:** Change `run_pareto_pipeline()` to return `Result<Vec<ProductionParetoSolution>, ParetoError>`. Propagate errors to the response as a structured field (e.g., `pareto_error: Option<String>`) rather than silently returning empty.

---

### R4 — Hardcoded Pareto Constants Not Configurable (MEDIUM)

**Location:** `adapters/ultracrew/src/pareto_pipeline.rs:121,205`
```rust
const PARETO_MUTATION_PERTURBATIONS: usize = 3;
const PARETO_DIVERSE_SEED_COUNT: usize = 20;
```

These constants are compile-time fixed. There is no way to tune them per-request
or per-deployment without recompiling. For production datasets that differ
significantly from the test scenario (20 workers, 5 shifts/skill), these values
may be too low (insufficient diversity) or too high (excessive latency).

**Risk:** Suboptimal alternative generation for production dataset sizes without
a code change + redeploy cycle.

**Fix direction:** Expose as optional `ScheduleRequest` fields with the current
values as defaults. This preserves backward compatibility while allowing per-request
tuning.

---

### R5 — Silent Zero-Fill on Missing Metric Keys in mapParetoAlternatives (MEDIUM)

**Location:** `ui/ultracrew/src/workflow/WorkflowUtils.ts`, `mapParetoAlternatives()`
```typescript
coverage:           alt.metrics['coverage']           ?? 0,
filled_positions:   alt.metrics['filled_positions']   ?? 0,
required_positions: alt.metrics['required_positions'] ?? 0,
fairness_penalty:   alt.metrics['fairness_penalty']   ?? 0,
utilization:        alt.metrics['utilization']        ?? 0,
cost:               alt.metrics['cost']               ?? 0,
```

If the backend `analyze_solution()` function changes its metric key names, or if
a metric is absent for a particular candidate, the UI silently displays 0 for that
metric. The scheduler sees incorrect coverage/utilization data with no warning.

**Risk:** Incorrect metric display leading to a scheduler decision based on wrong
data. The `diff_from_recommended` field is always 0 in the mapping (it is computed
by `compareAlternatives()` separately, but `compareAlternatives()` is not called
in the current workflow path for Pareto alternatives).

**Fix direction:** Add a validation step in `mapParetoAlternatives()` that logs a
warning when expected keys are absent. Document the canonical key names as a
contract between `analyze_solution()` and `mapParetoAlternatives()`.

---

### R6 — `diff_from_recommended` Always 0 for Pareto Alternatives (MEDIUM)

**Location:** `ui/ultracrew/src/workflow/WorkflowUtils.ts`, `mapParetoAlternatives()`
```typescript
diff_from_recommended: 0, // computed by compareAlternatives() if needed
```

`RosterAlternativeMetrics.diff_from_recommended` is always 0 for Pareto alternatives.
`compareAlternatives()` computes this correctly but is not called in the Pareto
workflow path. The `ComparisonTable` in `SelectDecision.tsx` displays this field —
it will always show 0 for all Pareto alternatives, which is misleading.

**Risk:** Scheduler sees "0 assignments differ" for all alternatives, which is
factually wrong. This could affect the scheduler's ability to assess how different
each alternative is from the recommended option.

**Fix direction:** Call `compareAlternatives()` after `mapParetoAlternatives()` in
`PlannerWorkflow.tsx` and update `diff_from_recommended` from the comparison result.
Alternatively, compute it inside `mapParetoAlternatives()` relative to `raw[0]`.

---

### R7 — No Request-Level Observability on Pareto Pipeline (MEDIUM)

**Location:** `services/ultracrew_server/src/main.rs`, `schedule_handler`

The Pareto pipeline runs with no structured logging, no timing instrumentation,
and no per-request trace ID. When a production request produces 0 alternatives,
there is no way to determine from logs whether:
- The dataset was homogeneous (correct behavior)
- The pipeline failed internally (R3)
- The diverse seeding produced no feasible genomes
- The Pareto dominance check eliminated all candidates

**Risk:** Operational blindness. Production incidents cannot be diagnosed without
attaching a debugger or adding ad-hoc logging.

**Fix direction:** Add structured log entries at key pipeline stages:
`pareto_seed_count`, `pareto_archive_size_before_filter`, `pareto_archive_size_after_filter`,
`pareto_wall_ms`. These are already computed in the diagnostic harness
(`p0b1_candidate_generation_harness.rs`) — they need to be surfaced in production.

---

### R8 — AppState Holds Last Solution In-Memory Only (MEDIUM)

**Location:** `services/ultracrew_server/src/main.rs`, `AppState`

`AppState.last_solution` and `AppState.last_request` are in-memory only. A server
restart loses all state. The `DecisionRepository` in the frontend is also in-memory
(browser session). There is no durable persistence of:
- The selected alternative's identity
- The `SchedulerDecision` record
- The `RedistributionLog` provenance

**Risk:** A browser refresh or server restart between decision and export loses the
complete provenance chain. The P0-E acceptance record notes that `DecisionRepository`
records candidate identity — but this record does not survive a page reload.

**Fix direction:** This is a known architectural gap (P3.3 persistence gate was
previously identified). P1 should define the minimum durable persistence contract:
at minimum, the `SchedulerDecision` and selected alternative id should survive a
page reload.

---

### R9 — CSRF Token Endpoint Has No Rate Limiting (LOW)

**Location:** `services/ultracrew_server/src/main.rs`, `/api/csrf-token` handler

The CSRF token endpoint is unauthenticated and has no rate limiting. In a
production deployment, this could be abused to enumerate tokens or exhaust server
resources.

**Risk:** Low in a single-tenant deployment; higher in a multi-tenant or
internet-facing deployment.

**Fix direction:** Add rate limiting middleware (e.g., `tower_governor`) to the
CSRF token endpoint. This is a deployment concern, not a code change.

---

### R10 — `pareto-{idx}` IDs Are Not Stable Across Requests (LOW)

**Location:** `ui/ultracrew/src/workflow/WorkflowUtils.ts`, `mapParetoAlternatives()`
```typescript
const id = `pareto-${idx}`;
```

Alternative IDs are generated as `pareto-0`, `pareto-1`, etc. based on the index
in the backend response array. If the backend returns alternatives in a different
order on a subsequent request (e.g., after a reschedule), the same `pareto-0` id
refers to a different candidate. The `DecisionRepository` records `selected_id` —
if this id is used to look up the alternative later, it may resolve to the wrong
candidate.

**Risk:** Low within a single session (the id is only used within one
`ScheduleResult`). Higher if `DecisionRepository` records are persisted and
replayed across sessions (R8).

**Fix direction:** Use a content-derived id (e.g., hash of `objectives` vector)
rather than an index-based id. This makes the id stable across requests for the
same candidate.

---

## Risk Summary

| ID | Description | Severity |
|----|-------------|----------|
| R1 | Mutex poison on panicking request | CRITICAL |
| R2 | Pareto pipeline blocks async runtime | HIGH |
| R3 | No error propagation from Pareto pipeline | HIGH |
| R4 | Hardcoded Pareto constants not configurable | MEDIUM |
| R5 | Silent zero-fill on missing metric keys | MEDIUM |
| R6 | `diff_from_recommended` always 0 for Pareto alternatives | MEDIUM |
| R7 | No request-level observability on Pareto pipeline | MEDIUM |
| R8 | AppState and DecisionRepository in-memory only | MEDIUM |
| R9 | CSRF token endpoint has no rate limiting | LOW |
| R10 | `pareto-{idx}` IDs not stable across requests | LOW |

---

## Recommended P1 Sequencing

P1 work should be sequenced by severity and independence:

**P1-A (CRITICAL):** Fix mutex poison risk — replace `.unwrap()` with poison
recovery in all `state.lock()` sites. This is a pure correctness fix with no
architectural implications.

**P1-B (HIGH):** Spawn Pareto pipeline in `spawn_blocking()` + add configurable
timeout. This prevents async runtime starvation under concurrent load.

**P1-C (HIGH):** Add `Result<>` return type to `run_pareto_pipeline()` and
propagate errors to the API response. This closes the silent-failure gap.

**P1-D (MEDIUM):** Add structured logging to the Pareto pipeline path (seed count,
archive size, wall time). This is low-risk and high-value for operational diagnosis.

**P1-E (MEDIUM):** Fix `diff_from_recommended` for Pareto alternatives. This is
a UI correctness fix with no backend changes.

**P1-F (MEDIUM):** Define minimum persistence contract for `SchedulerDecision` and
selected alternative id. This requires a product decision on storage mechanism
before implementation.

**P1-G (LOW):** Content-derived alternative ids. Defer until R8 (persistence) is
resolved, since the id stability problem only matters if records are persisted.

---

## What P1 Does NOT Include

The following are explicitly out of scope for P1:

- Pareto algorithm changes (frozen at P0)
- Alternative semantics changes (frozen at P0)
- UI ranking or recommendation logic (frozen at P0)
- New objective functions (frozen at P0)
- Candidate filtering changes (frozen at P0)
- Diversity improvements beyond PARETO_DIVERSE_SEED_COUNT (frozen at P0)

P1 is production hardening of the frozen P0 architecture, not a continuation of
P0 research.

---

## Authorization Request

This document is a readiness assessment only. No implementation has been performed.

P1-A (mutex poison fix) is the highest-priority item and is self-contained.
Authorization to proceed with P1-A implementation is requested before any other
P1 work begins.