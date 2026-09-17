# Phase 2: Backend Trace — ScheduleResult, Metrics, and Pareto Archive

**Date:** 2026-02-09  
**Baseline commit:** `fd53c6917` (Phase 1 — no synthetic schedule paths)  
**Status:** Read-only investigation. No code changes.

---

## 1. What `ScheduleResponse` currently serializes

```rust
struct ScheduleResponse {
    schedule: HashMap<u64, u64>,                          // shift_id → worker_id
    metrics: HashMap<String, f64>,                        // from analyze_solution()
    constraint_report: Option<ConstraintReport>,          // INRC constraint evaluation
    recommendations: Vec<SchedulingRecommendation>,       // human-readable recommendations
    telemetry: Option<OptimizationReport>,                // per-generation convergence data
}
```

All five fields are already populated and serialized on every `/api/schedule` and `/api/reschedule` response.

---

## 2. What `metrics` contains (`analyze_solution`)

`analyze_solution()` in `adapters/ultracrew/src/decision_intelligence.rs` returns a `HashMap<String, f64>` with exactly five keys:

| Key | Source field on `ScheduleSolution` | Meaning |
|---|---|---|
| `fitness` | `solution.fitness` | Overall optimizer fitness score (lower = better cost) |
| `hard_violations` | `solution.hard_violations` | Count of hard-constraint violations |
| `fairness_penalty` | `solution.fairness_penalty` | Variance of hours across workers |
| `fatigue_penalty` | `solution.fatigue_penalty` | Fatigue penalty from ecology data |
| `rest_violations` | `solution.rest_violations` | Rest-period violation count |

**Coverage is not in `metrics`.** There is no `coverage`, `filled_positions`, `required_positions`, or `utilization` key. These were hardcoded constants in the now-removed `buildSyntheticAlternatives()`.

---

## 3. What `ScheduleSolution` carries

```rust
pub struct ScheduleSolution {
    pub assignments: HashMap<u64, u64>,
    pub fitness: f64,
    pub hard_violations: usize,
    pub fairness_penalty: f64,
    pub fatigue_penalty: f64,
    pub rest_violations: usize,
    pub recommendations: Option<Vec<SchedulingRecommendation>>,
    pub telemetry: Option<OptimizationReport>,
}
```

`ScheduleSolution` is built from `ScheduleEvaluation::from_evaluation()`. `ScheduleEvaluation` also carries `hc1_violations`, `hc2_violations`, `hc3_violations` — but these are **not propagated** into `ScheduleSolution`. They are lost at the boundary.

---

## 4. What `OptimizationReport` (telemetry) contains

```rust
pub struct OptimizationReport {
    pub generations: Vec<GenerationTelemetry>,
}

pub struct GenerationTelemetry {
    pub generation: usize,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub hard_violations: usize,
    pub hc1_violations: usize,
    pub hc2_violations: usize,
    pub hc3_violations: usize,
    pub average_hc3_violations: f64,
    pub hc4_violations: usize,
    pub rest_violations: usize,
    pub population_valid_count: usize,
    pub soft_violations: usize,
    pub fairness_penalty: f64,
    pub workload_penalty: f64,
    pub elapsed_time_ms: u128,
    pub unique_genomes: usize,
}
```

This is rich per-generation convergence data. It is already serialized in `ScheduleResponse.telemetry`. The UI currently ignores it entirely.

---

## 5. The Pareto archive — critical finding

### Two different engine paths

The codebase has **two distinct evolution engine paths**:

**Path A — `run_inrc_startup_pipeline` (INRC benchmark path):**
- Uses `coralys_moga::engine_proof::EvolutionEngine`
- Has a `ParetoArchive<G>` with `solutions: Vec<ParetoSolution>`
- Each `ParetoSolution` has a `fitness: Vec<f64>` (multi-objective vector) and a `genome`
- Returns `InrcStartupResult { schedule, pareto_solutions: Vec<InrcParetoSolution> }`
- **This path is NOT used by `/api/schedule`**

**Path B — `run_pipeline_from_request` → `run_pipeline` → `run_optimization` (production path):**
- Uses `coralys_moga::engine::EvolutionEngine` (different engine)
- Returns `GaResult<ScheduleEvaluation>` with:
  - `global_best: ScheduleEvaluation` — single best solution
  - `generation_history: Vec<ScheduleEvaluation>` — one entry per generation
  - `average_fitness_history: Vec<f64>`
- **No Pareto archive is returned.** `run_pipeline` only accesses `ga_result.global_best`.

### What this means

The production scheduling path (`/api/schedule`, `/api/reschedule`) uses a **single-objective GA** (`coralys_moga::engine::EvolutionEngine`) that returns only the global best. There is no Pareto archive in this path. The multi-objective Pareto engine (`engine_proof`) is used only for the INRC benchmark pipeline.

---

## 6. Where demand/coverage is defined

Coverage is computed inside `UltraCrewEvaluator::evaluate()` in `adapters/ultracrew/src/inrc/schedule_optimizer.rs`. The evaluator computes `coverage_deficit` as `objective[5]` (the HC1 penalty). The raw demand comes from `week_data.requirements` (parsed from INRC scenario files).

**Coverage is computed during evaluation but is not surfaced in `ScheduleSolution` or `ScheduleResponse`.** The `ScheduleEvaluation` struct carries `hc1_violations` (which encodes coverage deficit) but `ScheduleSolution::from_evaluation()` only maps it to a binary `hard_violations: if eval.is_valid { 0 } else { 1 }`.

---

## 7. What the UI currently receives vs. what it uses

The frontend `ScheduleResult` type (in `WorkflowTypes.ts`) currently declares:

```typescript
export interface ScheduleResult {
  schedule: Record<string, number>;
  alternatives?: RosterAlternative[];
  recommended_alternative_id?: string;
}
```

It only reads `schedule`. The `metrics`, `constraint_report`, `recommendations`, and `telemetry` fields from the backend response are **received but ignored** — they are not in the TypeScript type at all.

---

## 8. Summary of findings

| Question | Finding |
|---|---|
| What does `ScheduleResponse` serialize? | `schedule`, `metrics` (5 keys), `constraint_report`, `recommendations`, `telemetry` (per-generation) |
| What does `metrics` contain? | `fitness`, `hard_violations`, `fairness_penalty`, `fatigue_penalty`, `rest_violations` |
| Is coverage in `metrics`? | **No.** Not computed or surfaced at the API boundary. |
| Where is demand defined? | `week_data.requirements` in INRC scenario files, consumed by `UltraCrewEvaluator` |
| Where is coverage evaluated? | `UltraCrewEvaluator::evaluate()` as `objective[5]` (coverage_deficit) |
| Does the Pareto archive exist in the production path? | **No.** Production path uses single-objective GA. Only `global_best` is returned. |
| Does each candidate have evaluation metrics? | N/A — there are no candidates in the production path. |
| Is there a recommendation/global-best designation? | Yes — `global_best` is the single recommendation. No alternatives. |
| What is lost between Coralys → backend → UI? | `hc1/hc2/hc3_violations` (lost in `from_evaluation`), coverage metrics (never surfaced), per-generation telemetry (serialized but ignored by UI) |
| What must be added to the API? | See Phase 3 proposal below. |

---

## 9. Phase 3 proposal (read-only — not yet authorized)

The missing `alternatives` field requires one of two approaches:

**Option A — Expose the existing `generation_history` as candidates.**  
`GaResult.generation_history` contains one `ScheduleEvaluation` per generation. The best N distinct evaluations could be surfaced as alternatives. This requires no new optimizer capability — only a serialization change. However, `generation_history` entries are not Pareto-diverse; they are sequential snapshots of the global best, so they may not represent meaningfully different trade-offs.

**Option B — Switch the production path to the multi-objective engine (`engine_proof`).**  
This would give a genuine Pareto archive with diverse candidates. It is a larger change: the production path would need to use `EvolutionEngine<ScheduleGenome>` from `engine_proof` instead of the current single-objective engine. The `InrcParetoSolution` type already exists and carries per-objective fitness vectors.

**Option C — Run N independent optimization runs and surface each as an alternative.**  
Each run produces a different `global_best` due to stochastic initialization. This is the simplest path to genuine diversity but multiplies runtime by N.

**Recommendation:** Option B is the architecturally correct path. The multi-objective engine already exists and is proven (used in INRC benchmarks). The production path should use it. This is a Phase 3 decision — no implementation in this document.

---

## 10. What can be done without new optimizer capability (Phase 2 scope)

The backend already serializes `metrics` (fitness, hard_violations, fairness_penalty, fatigue_penalty, rest_violations) and `telemetry` (per-generation convergence). These are genuine optimizer outputs.

The UI can be updated to:
1. Declare these fields in `ScheduleResult` (TypeScript type extension)
2. Display them in the Review step as genuine backend metrics

This does not require any backend changes. It is a pure frontend type + display change.

**Coverage metrics cannot be added without a backend change** — they are not currently surfaced in `ScheduleResponse`.

---

*Phase 2 trace complete. No code changes made. Awaiting authorization for Phase 3.*
