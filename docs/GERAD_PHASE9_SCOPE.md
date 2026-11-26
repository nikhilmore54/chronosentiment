# GERAD Phase 9 — Optimization Scope

**Status: OPEN — scoping only, no implementation**
**Unblocked by:** Phase 8 closure (`31fe1d86e`)

---

## Governance position

Phase 8 has established the following evidence-backed attribution:

| Component | setA-01 % wall | setA-14 % wall |
|---|---|---|
| `constraint_model.is_feasible()` | **38.9%** | **41.7%** |
| `process_offspring` (improve path) | **35.7%** | **40.4%** |
| `process_offspring` (repair path) | 6.7% | 2.4% |
| Eval (Phase B parallel) | 17.0% | 13.9% |
| Rayon residual | 0.1% | 0.0% |

The ranking is consistent across both instances. No optimization has been implemented. Phase 9 must define its own gate criteria before any code changes are made.

**Governance rule:** Do not optimize `is_feasible()` and `process_offspring` simultaneously. Causal attribution requires one intervention at a time.

---

## Phase 9 structure

Phase 9 is split into two independent sub-phases. P9-B does not start until P9-A is promoted or rejected.

```
Phase 9
├── P9-A: Feasibility Optimization  ← primary candidate
│   ├── characterize is_feasible()
│   ├── formulate ONE optimization hypothesis
│   ├── implement one intervention
│   ├── verify trajectory invariants (5/5 bit-exact)
│   ├── measure wall-clock impact (T_net > 0)
│   ├── verify solution quality (no regression)
│   └── PROMOTE / REJECT
│
└── P9-B: Improve Optimization  ← secondary candidate
    └── blocked until P9-A disposition
```

---

## P9-A: Feasibility Optimization

### What Phase 8 established

`constraint_model.is_feasible()` is called once per offspring per generation in the sequential Phase A loop. It accounts for 38.9% of wall-clock on setA-01 and 41.7% on setA-14. Its result is used solely to classify the offspring as needing repair (infeasible) or improvement (feasible) before calling `process_offspring`.

### Questions that must be answered before implementation

1. **What does `is_feasible()` compute?**
   Read `constraint_model.rs` and identify the exact constraint checks performed. Are they all necessary for the repair/improve classification?

2. **Why is it called once per offspring?**
   Is the feasibility result available from a prior computation (e.g., from the parent, from crossover output), or must it always be recomputed from scratch?

3. **Which parts of its computation are repeated across offspring in the same generation?**
   Does `is_feasible()` depend on instance data (fixed), genome data (variable), or both? Which components are genome-independent?

4. **Can its result be derived incrementally?**
   If the child genome differs from the parent by a small mutation, can feasibility be checked incrementally rather than from scratch?

5. **What invariants must an optimization preserve?**
   - The feasibility classification must be identical (bit-exact) for every offspring
   - The trajectory (best_obj, n_eval, cache_hits, valid) must remain bit-exact
   - No change to the constraint semantics

6. **What constitutes a meaningful speedup?**
   - Minimum threshold: T_net > 0 on setA-01 (paired contemporaneous control)
   - Corroboration: T_net > 0 on setA-14
   - Attribution: `feasibility_ms` reduction ≥ 50% of the measured saving

7. **What solution-quality regression is unacceptable?**
   - `best_obj` must not increase (worse solution)
   - `valid` must remain true
   - Any regression on setA-14 (larger instance) is disqualifying

### Candidate hypotheses (to be evaluated, not implemented)

**H1: Memoization / caching of feasibility results**
If `is_feasible()` is deterministic given the genome, its result could be cached keyed by genome hash. Cost: hash computation + cache lookup. Benefit: avoids recomputation for repeated genomes (e.g., cache hits that were already evaluated).

**H2: Incremental feasibility check**
If the child differs from the parent by a bounded mutation, feasibility could be checked only for the changed components. Requires understanding which constraints are affected by which genome elements.

**H3: Lazy feasibility — defer to process_offspring**
If `process_offspring` already determines feasibility internally as part of repair/improve, the explicit `is_feasible()` call may be redundant. Requires reading `process_offspring` to confirm.

**H4: Cheaper feasibility proxy**
A cheaper necessary condition for infeasibility could be checked first (fast path), falling back to full `is_feasible()` only when the proxy is inconclusive.

### Gate criteria for P9-A

| Criterion | Requirement |
|---|---|
| Trajectory invariants | 5/5 bit-exact on setA-01 vs Phase 8 baseline |
| Trajectory invariants | 5/5 bit-exact on setA-14 vs Phase 8 baseline |
| T_net (setA-01) | > 0 (paired contemporaneous control) |
| T_net (setA-14) | > 0 (paired contemporaneous control) |
| `feasibility_ms` reduction | ≥ 50% of measured T_net |
| Solution quality | `best_obj` not worse on either instance |
| Attribution | ≥ 80% of T_net explained by `feasibility_ms` reduction |

---

## P9-B: Improve Optimization

**Blocked until P9-A is promoted or rejected.**

### What Phase 8 established

`process_offspring` (improve path) accounts for 35.7% of wall-clock on setA-01 and 40.4% on setA-14. It is called for every feasible offspring. It includes local search.

### Questions to answer before P9-B starts

1. What does the improve path in `process_offspring` compute?
2. What is the local search strategy (neighbourhood, termination criterion)?
3. Is the improve path called with the same genome multiple times across generations?
4. Can the improve path be bounded (early termination) without degrading solution quality?
5. What invariants must be preserved?

### Gate criteria for P9-B

Same structure as P9-A, with `improve_ms` replacing `feasibility_ms` as the primary attribution metric.

---

## What Phase 9 must NOT do

- Do not optimize `is_feasible()` and `process_offspring` in the same experiment
- Do not change the constraint semantics
- Do not change the evolutionary algorithm structure (selection, crossover, mutation rates)
- Do not start P9-B before P9-A disposition
- Do not promote based on setA-01 alone — setA-14 corroboration is required

---

## First action for Phase 9

Read [`adapters/roadef/src/constraint_model.rs`](../adapters/roadef/src/constraint_model.rs) and answer the seven questions in the P9-A characterization section above. The output is a written characterization document, not code.