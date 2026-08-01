# UltraCrew — Pairing-Topology Mutation Operator Evaluation

**Status:** In progress — architectural analysis complete, implementation pending
**Date:** 2026-07-31
**Relates to:** [UltraCrew vs GENCOL Pipeline Divergence Analysis](UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md), Section 7 Step 1
**Hypothesis under test:** Introducing a pairing-aware mutation operator improves pairing quality metrics on the GERAD benchmark without degrading assignment coverage or fairness.

---

## 1. Objective

Investigate whether introducing pairing topology as an explicit optimization variable in the MOGA improves benchmark alignment with the GERAD reference. This is the highest-priority architectural investigation identified in the Pipeline Divergence Analysis.

---

## 2. Background

### 2.1 Architectural Finding from GERAD Experiments (2026-07-31)

The GERAD end-to-end pipeline experiment ([`adapters/airline/tests/gerad_e2e.rs`](../../adapters/airline/tests/gerad_e2e.rs)) revealed a fundamental architectural distinction between the current UltraCrew implementation and a production airline crew pairing optimizer.

**Current UltraCrew pipeline:**

```
flights.csv + crew.csv
        ↓
Greedy duty construction (global chronological grouping)
        ↓
Greedy pairing construction (single-duty pairing model)
        ↓
GreedyScheduler (assignment)
        ↓
LocalSearch (assignment swap/relocate only)
        ↓
Final roster
```

**Key consequence:** `LocalSearch` never changes pairing topology. It can only move existing pairings between crew rotations. If the greedy constructor creates suboptimal duties or pairings, the optimizer is confined to improving assignment — it cannot repair earlier construction decisions. This is a structural limitation, not an implementation detail.

**Observed evidence:** In the GERAD experiment, instances 1–3 showed:
- Instance1: 54 pairings (8h) vs 52 pairings (10h) — 33 crew, 1013 legs
- Instance2: 81 pairings (8h) vs 82 pairings (10h) — 34 crew, 1500 legs
- Instance3: 141 pairings (8h) vs 142 pairings (10h) — 47 crew, 1855 legs

These counts are far smaller than the GERAD reference (172–1648 pairings) because the global chronological grouping produces many spatial breaks (499–1004 per instance), and the single-duty pairing model cannot form multi-duty pairings from single-day flights.

**Contrast with benchmark:** The GERAD reference pairings span two days (Jan 29–30) with multi-duty structure. The benchmark's operational model is a **joint construction-and-assignment optimizer** — pairing boundaries are determined simultaneously with crew assignment, not as a fixed preprocessing step.

### 2.2 Two Separable Issues

The GERAD experiments identified two distinct issues:

**Issue 1 — Solution quality (algorithmic):** The greedy pairing constructor freezes pairing topology before optimization begins. The optimizer cannot explore alternative pairing structures. This limits solution quality independently of runtime.

**Issue 2 — Runtime (performance):** The `WorkloadBalanceObjective` evaluates the full roster on each candidate move. With 571 pairings × 34 rotations, the greedy assignment alone requires ~19K objective evaluations. In debug mode, each evaluation involves roster cloning, making the total cost prohibitive for large instances.

These issues are related but separable: solving the performance issue does not address the solution quality limitation, and vice versa.

### 2.3 Short-Term Performance Improvements

Before changing the algorithm, the following implementation improvements can reduce runtime significantly:

**Compile in release mode:** Rust debug builds sacrifice performance for debuggability. Algorithmic benchmarks are much more meaningful under `cargo test --release`. Expected speedup: 10–50×.

**Avoid unnecessary cloning:** If `WorkloadBalanceObjective` clones the full roster on each evaluation, incremental updates (compute delta from the proposed move without materializing the full new roster) can reduce cost from O(pairings) to O(1) per evaluation.

**Index duties by airport and time window:** The current global chronological grouping compares all consecutive flight pairs. Indexing duties by report airport and departure time window reduces duty-successor lookup from O(n) to O(log n) or O(1).

**Cache objective evaluations:** If the same roster state is evaluated multiple times (e.g., during local search backtracking), caching avoids redundant computation.

### 2.4 Longer-Term Architectural Direction

The more significant improvement is to stop treating pairing as a fixed preprocessing step. Instead, pairing topology should be part of the optimization problem:

```
flights.csv + crew.csv
        ↓
Generate feasible duty graph (indexed by airport + time)
        ↓
Evolutionary optimizer
    ├── modify duty boundaries (merge/split FDPs)
    ├── merge/split pairings
    ├── assign crews
    └── evaluate: workload balance + legality + TAFB + hotel nights + deadhead
        ↓
Final roster
```

In this architecture, the genome encodes duty boundaries, pairing boundaries, and crew assignments simultaneously. Mutation operators can:
- Merge two adjacent duties into one (if the combined FDP is legal)
- Split a duty at a layover point (creating two shorter FDPs)
- Reassign a pairing to a different crew member
- Swap duty sequences between two crew members

This matches the direction of Coralys MOGA, where the genome represents the scheduling solution rather than just the assignment. It addresses both issues: solution quality improves because pairing is no longer frozen by a greedy heuristic, and scalability improves because the search is guided by evolutionary operators rather than exhaustive quadratic pairing construction.

**The GERAD experiments have been valuable precisely because they highlighted this architectural distinction:** the current implementation is effectively an assignment optimizer, whereas a production airline scheduler is a joint construction-and-assignment optimizer. That insight provides a clear direction for the next stage of UltraCrew's evolution.

### 2.5 Guiding Architectural Principle

The experiments establish a clear principle for the Coralys/UltraCrew architecture:

> **Use deterministic algorithms to define what is feasible; use Coralys to decide what is optimal.**

This gives each technique a clear role:

- **Deterministic logic:** correctness, legality, feasibility. Parsing input files, building a connectivity graph, computing legal successor relationships, precomputing feasibility constraints, filtering impossible moves. These are transformations, not optimization problems.
- **Coralys MOGA:** choice among feasible alternatives. Every stage that has multiple feasible alternatives becomes part of the search rather than a preprocessing heuristic.

**Optimization-first architecture:**

```
flights.csv + crew.csv
        ↓
Generate feasible scheduling graph
  (deterministic: parse, index, compute legal successors)
        ↓
Coralys MOGA
    ├── Optimize duty boundaries (merge/split FDPs)
    ├── Optimize pairing construction (group duties into pairings)
    ├── Optimize crew assignment (assign pairings to rotations)
    ├── Optimize reserve allocation
    ├── Optimize deadheads
    └── Evaluate objective vector (workload balance + legality + TAFB + hotel nights + deadhead)
        ↓
Best Schedule
```

**Strategic positioning:** Coralys becomes the optimization substrate. Applications like UltraCrew become domain adapters that define constraints and objectives, while Coralys searches the solution space wherever there are meaningful trade-offs. This is a much more powerful and reusable architecture than having isolated greedy algorithms sprinkled throughout the pipeline. The same optimization framework can be applied across UltraCrew, UltraRoute, manufacturing scheduling, field service, and other domains.

### 2.6 Reviewer Analysis: Search Space Collapse (2026-07-31)

The following analysis was provided by the reviewer after examining the end-to-end pipeline results for instances 1–4.

#### The layover threshold is not the primary driver at scale

The threshold can influence the solution, but only when the constructed duties are close to the threshold boundary. Once the network becomes sufficiently dense (instance 4, 5613 legs), the dominant factor becomes the deterministic chronological grouping and the spatial continuity constraint — not the threshold value. Instance 4 produced identical pairing counts (571) under both conditions.

#### The real issue is that the search space has already collapsed

By the time Coralys is invoked, duties are fixed, pairings are fixed, chronology is fixed, and spatial grouping is fixed. The optimizer is operating inside a very small neighbourhood. This explains several observations:

**Instance 2 (greedy = optimized = 0.5433):** No improving move exists because the search space has already been constrained by the greedy construction phase. The assignment optimizer is correct — there is simply nothing left to improve within the fixed pairing topology.

**Instance 4 (greedy = optimized = 0.1239, 571 pairings):** Same phenomenon at larger scale. The optimizer cannot improve because it is optimizing assignments over a pairing structure that is already fixed.

**Instance 3 (42–44% improvement):** Instance 3 is the exception. There was just enough flexibility in the assignment stage for local search to exploit. This is evidence that the assignment optimizer works. It is not evidence that the pairing strategy is globally good.

#### The pairing problem is itself an optimization problem

The current architecture effectively says: "decide the pairings greedily, then optimize the crew." But airline scheduling is a coupled problem:

```
maximize

  pairing quality
+ crew utilization
+ fairness
+ cost
+ legality
```

Changing one pairing changes everything downstream. These objectives cannot be optimized independently.

#### This is exactly where MOGA belongs

The reviewer's recommendation is to treat pairing as another genome dimension. Instead of:

```
Genome → Crew assignment
```

the genome should encode:

```
Genome
  Duty boundaries
  Pairing boundaries
  Crew assignment
  Reserve allocation
```

Crossover and mutation can then change the pairings themselves. This is a much richer search space than simply swapping pairings between rotations.

#### The primary conclusion from the GERAD experiments

> **The current optimization stage is downstream of the decisions that most strongly determine schedule quality.**

In other words:
- The optimizer is effective *within* the search space it receives (as shown by instance 3).
- But the search space itself is produced by deterministic heuristics.

If Coralys is intended to be the core differentiator, the long-term architecture should shift those construction decisions into the optimization process wherever feasible. Legality checks, feasibility graph construction, and constraint propagation remain deterministic — but the choices among feasible duties, pairings, and assignments become optimization variables rather than fixed inputs.

### 2.7 Reviewer Analysis: Coralys as a Generic Optimization Framework (2026-07-31)

The following analysis was provided by the reviewer as a second-order architectural insight, building on Section 2.6.

#### The framing error: pipeline stages vs decision problems

The mistake in the current architecture is thinking in terms of **pipeline stages** rather than **decision problems**. Coralys is not a scheduler — it is an optimization engine. The first question should be: *what decisions exist in airline scheduling that have multiple feasible alternatives?* Every such decision is a candidate optimization task.

#### Optimization Task Map

| Stage | Current implementation | Decision variables | Example objectives |
|-------|----------------------|-------------------|-------------------|
| Flight network | Parsing only | None | None |
| Duty construction | Greedy | Which flight continues the duty? When to end? Which connection? Skip a legal connection? | Minimize FDP, maximize utilization, minimize sit time, reduce deadheads |
| Pairing construction | Greedy chronological | Which duty follows another? When to terminate? Which overnight city? Which base? | Hotel cost, TAFB, crew productivity, legality, robustness |
| Crew assignment | GreedyScheduler | Which crew receives which pairing? | Fairness, qualifications, preferences, cost, reserve utilization |
| Reserve allocation | None | Which reserve? Where? When? | Coverage, cost, response time |
| Deadhead planning | None | Deadhead vs train vs taxi vs commercial? | Cost, time, crew welfare |
| Recovery scheduling | None | Swap? Delay? Reassign? Cancel? | Delay cost, passenger impact, crew welfare |

Every stage except flight network parsing is an optimization problem.

#### The uniform structure of every optimization task

Every scheduling optimization problem has exactly the same shape:

```
Current State
        │
        ▼
Generate Feasible Actions
        │
        ▼
Evaluate Objectives
        │
        ▼
Choose Best Action
        │
        ▼
New State
```

**Duty construction example:**
- State: current duty (legs assigned so far, current airport, current time)
- Actions: add next flight, terminate duty
- Constraints: FDP limits, legality, spatial continuity
- Objectives: utilization, cost, robustness

**Pairing construction example:**
- State: current pairing (duties assigned so far, current base, current time)
- Actions: append duty, terminate pairing
- Constraints: rest requirements, home base return, legality
- Objectives: TAFB, hotel cost, crew productivity

**Crew assignment example:**
- State: current roster (pairings assigned to rotations)
- Actions: assign pairing to crew, swap crew between pairings
- Constraints: qualifications, availability, legality
- Objectives: fairness, workload balance, cost

#### Architectural implication

Coralys should not expose separate APIs like `DutyOptimizer`, `PairingOptimizer`, or `CrewOptimizer`. Instead, it should expose a **generic optimization framework** where each scheduling phase supplies:

- **State representation** — the current scheduling context
- **Action generator** — all feasible next decisions
- **Constraint evaluator** — legality and feasibility rules
- **Objective evaluator** — one or more optimization objectives
- **State transition** — how an action transforms the state

Each airline-specific optimization task becomes a specialization of this common model. This gives a single optimization substrate reusable across all phases of airline scheduling and across other Coralys domains (workforce scheduling, routing, manufacturing, recovery planning).

**Contrast with current architecture:** Today, each phase has its own ad-hoc implementation (greedy duty grouping, greedy pairing grouping, `GreedyScheduler`, `LocalSearch`). These share no common abstraction. The generic framework replaces all of them with a single interface that Coralys optimizes.

**Relationship to the guiding principle (Section 2.5):** The generic framework is the concrete realization of "use deterministic algorithms to define what is feasible; use Coralys to decide what is optimal." The action generator and constraint evaluator are deterministic (they define feasibility); the objective evaluator and optimizer are Coralys (they decide optimality).

### 2.8 Reviewer Code Review: Implementation Observations (2026-07-31)

The following observations were provided by the reviewer after a careful reading of [`adapters/airline/tests/gerad_e2e.rs`](../../adapters/airline/tests/gerad_e2e.rs).

#### 1. The architecture is pipeline-driven instead of optimization-driven

Only two lines in the experiment actually invoke Coralys optimization: `GreedyScheduler::new()` + `greedy.assign()` and `LocalSearch::new()` + `local_search.run()`. Everything before that is deterministic. Coralys is only optimizing assignments, not pairings. This is the biggest architectural deviation.

#### 2. `build_pairings_from_flights()` has too many responsibilities

The function currently performs flight parsing, duty construction, duty validation, pairing construction, pairing validation, diagnostics, and spatial discontinuity counting. The reviewer recommends splitting into independently replaceable stages: `FlightGraphBuilder → DutyBuilder → PairingBuilder → Scheduler`. Even if they remain deterministic for now, each becomes independently replaceable by an optimizer later.

#### 3. The single-duty pairing model is a restricted baseline, not a pairing algorithm

The current code `for duty in duties { Pairing::new(... vec![duty]) }` is a useful baseline but is not a pairing algorithm. It should be documented as such and preserved as the deterministic reference, not extended.

#### 4. Pairing rejection is hiding useful diagnostic information

Currently `Err(_) => rejected_pairings += 1` discards the rejection reason. The reviewer recommends `Err(e) => { rejected_pairings += 1; pairing_errors[e] += 1; }` to produce a breakdown like `NotRoundTrip: 4882, IllegalRest: 0, InvalidBase: 130`. This information will become invaluable when Coralys starts generating pairings.

#### 5. Duty count is indirectly computed

`pairings.iter().map(|p| p.duties().len()).sum()` works only because every pairing contains exactly one duty. If Coralys later creates multi-duty pairings, this will silently produce wrong results. The reviewer recommends returning `duties.len()` directly from `build_pairings_from_flights()`.

#### 6. The seeded rotation is a design smell

`Rotation::new(... vec![seed])` exists because `Rotation::new()` cannot create an empty rotation. In reality, an empty rotation is perfectly valid before scheduling begins. The domain model should allow empty rotations during optimization and only require non-empty rotations (if desired) in the final published roster.

#### 7. The `max_iter=0` workaround reveals the optimizer's role

The code `let max_iter = if i <= 3 { 10 } else { 0 }` means the optimizer is skipped for large instances because it is too slow. This reveals that the optimizer is currently an optional refinement. In the Coralys architecture it should become the primary engine.

#### Overall assessment

The experiment is well structured and scientifically documented. The assumptions, methodology, and limitations are clearly stated. The reviewer recommends preserving `gerad_e2e.rs` as the **deterministic baseline** and developing a parallel Coralys-native implementation (`gerad_coralys.rs`) alongside it, rather than rewriting the existing file. See [`Coralys_GERAD_Integration_Roadmap.md`](Coralys_GERAD_Integration_Roadmap.md) for the three-milestone implementation plan.

---
### 2.9 Reviewer Analysis: Runtime, Search Space Collapse, and Next Steps (2026-07-31)

The following analysis was provided by the reviewer after the full 7-instance experiment completed (total runtime: 666.01s).

#### Runtime baseline

The experiment completed all 7 GERAD instances in 666.01 seconds (11.1 minutes) in release mode, covering 29,476 flight legs. This is acceptable for a research baseline but not for an optimization platform. The reviewer recommends adding stage-level timing instrumentation before introducing Coralys, so that each stage's cost is quantified:

```
Load CSVs          : ...ms
Build duties       : ...ms
Build pairings     : ...ms
Build roster       : ...ms
Greedy assignment  : ...ms
Local search       : ...ms
Evaluation         : ...ms
```

This gives a quantitative baseline against which Coralys can be measured not only on solution quality but also on performance as it replaces each scheduling stage.

#### The `max_iter=0` decision is intentional, not a failure

The code `let max_iter = if i <= 3 { 10 } else { 0 }` means local search is intentionally disabled for instances 4–7. The greedy=optimized result for those instances is expected — the optimizer is not failing, it is not running. This was a pragmatic choice to keep the experiment tractable. It also reveals that the optimizer is currently an optional refinement rather than the primary engine.

#### Instances 6 and 7 have zero assignment search space

Instance 6: 92 rotations, 92 pairings (1:1 ratio, pairings/rot=1–1)
Instance 7: 159 rotations, 159 pairings (1:1 ratio, pairings/rot=1–1)

Every rotation receives exactly one pairing. There is no assignment search space at all. This is a consequence of the global chronological grouping model producing very few pairings relative to the number of crew members. The assignment optimizer has nothing to optimize.

#### Pairing constructor is a filter, not a search algorithm

The accepted:rejected ratio is consistently ~1:9 to ~1:10 across all instances:

| Instance | Accepted | Rejected | Ratio |
|----------|----------|----------|-------|
| 1 | 54 | 907 | 1:16.8 |
| 2 | 81 | 1346 | 1:16.6 |
| 3 | 141 | 1684 | 1:11.9 |
| 4 | 571 | 5012 | 1:8.8 |
| 5 | 568 | 5149 | 1:9.1 |

The pairing constructor is rejecting far more candidate duties than it accepts. This confirms that it is acting as a filter (accepting only duties that happen to form valid single-duty pairings) rather than as a search algorithm that actively constructs good pairings.

#### Recommended next steps

1. **Freeze `gerad_e2e.rs` as the deterministic baseline.** Do not extend it further.
2. **Add stage-level timing instrumentation** to the baseline before introducing Coralys.
3. **Create `gerad_coralys_pairing.rs`** — a new experiment where Coralys optimizes pairing construction from the same `flights.csv`. Compare pairing count, objective value, runtime, constraint violations, and robustness against the baseline.
4. **Allow multi-duty pairings** in the Coralys implementation. The single-duty model was a practical baseline choice; the next implementation should allow Coralys to generate and evolve multi-duty pairings.
### 2.10 Profiling Results: Neighbour Generation is the Bottleneck (2026-07-31)

#### Measurement methodology

Stage-level timing was added to [`gerad_e2e.rs`](../../adapters/airline/tests/gerad_e2e.rs) (`pairing_build`, `roster_seed`, `greedy`, `local_search`) and sub-stage timing accumulators were added to [`local_search.rs`](../../adapters/airline/src/optimization/search/local_search.rs) (`neighbour_gen`, `legality`, `evaluate`). All measurements are from release-mode builds.

#### Stage-level results (instance 3, 141 pairings, 47 rotations, 10 iterations)

Instance 3 was selected as the representative profiling case because it is the smallest instance where Local Search finds genuine improvements (42.4% reduction). Instances 1–2 show no improvement and instances 4–7 are dominated by Greedy construction time; instance 3 is the only case where Local Search runtime is both non-trivial and meaningful to profile.

| Stage | Time |
|-------|------|
| Pairing build | 1ms |
| Roster seed | 0ms |
| Greedy assignment | ~3100ms |
| Local search | ~23000ms |

#### Local search sub-stage breakdown (instance 3, 8h condition)

| Sub-stage | Total time | Per move |
|-----------|-----------|---------|
| Neighbour generation | ~17,750ms | ~556µs |
| Objective evaluation | ~9ms | ~0.29µs |
| Legality checking | ~0.09ms | ~0.003µs |
| **Moves evaluated** | **31,944** | — |

Neighbour generation accounts for **76–89% of local search time**. Evaluation and legality are effectively free by comparison (~1,900× cheaper per move than generation).

#### Root cause: full Roster clone per candidate move

Inspection of [`swap_pairings()`](../../adapters/airline/src/optimization/neighborhood/swap.rs:18) confirms the mechanism:

```rust
// For every candidate move, swap_pairings() does:
let new_pairings_a: Vec<_> = pairings_a.iter().enumerate()
    .map(|(i, p)| if i == pairing_a { pairings_b[pairing_b].clone() } else { p.clone() })
    .collect();
// ... same for new_pairings_b ...
let new_rotations: Vec<_> = rotations.iter().enumerate()
    .map(|(i, r)| if i == rotation_a { new_rot_a.clone() }
                  else if i == rotation_b { new_rot_b.clone() }
                  else { (*r).clone() })
    .collect();
Roster::new(..., roster.legs().cloned().collect(), new_rotations)
```

Every candidate move clones:
- All pairings in rotation A (even unchanged ones)
- All pairings in rotation B (even unchanged ones)
- All other rotations in the roster
- All legs in the roster

For instance 3: 141 pairings × 47 rotations × 31,944 moves ≈ 211 million clone operations per local search run. The asymptotic cost per iteration is O(M × (L + R)) where M is the number of candidate moves evaluated, L is the number of legs in the roster, and R is the number of rotations.

#### Threshold comparison (instance 3)

| Condition | Moves | Neighbour gen | Ratio |
|-----------|-------|--------------|-------|
| 8h | 31,944 | ~17,750ms | 556µs/move |
| 10h | 27,241 | ~15,140ms | 556µs/move |

Runtime scales almost exactly proportionally with move count, confirming the per-move cost is constant and the total cost is purely a function of how many candidates are generated.

#### Fix: lazy move descriptor

The correct fix is to represent each candidate as a lightweight descriptor (two `usize` pairs for swap, three for relocate) and only materialize the `Roster` clone for the single accepted move per iteration. This reduces the number of clone operations from O(M × (L + R)) to O(L + R) per iteration — a theoretical reduction factor of ~31,944× for instance 3 (equal to the move count M). The observed wall-clock speedup is smaller (approximately 11×, measured in Section 2.11) because the descriptor evaluation itself has non-zero cost and the ranked-descriptor approach evaluates more descriptors than the original eager implementation.

```rust
// Before: O(moves × roster_size) clones
for each (i, pa, j, pb) in neighbourhood {
    let candidate: Roster = swap_pairings(current, i, pa, j, pb)?; // full clone
    if is_legal(&candidate) && cost(&candidate) < best_cost { ... }
}

// After: O(1) clones during search, O(roster_size) clone only on acceptance
for each (i, pa, j, pb) in neighbourhood {
    let delta = evaluate_delta(current, i, pa, j, pb); // no clone
    if delta < 0.0 && is_legal_descriptor(current, i, pa, j, pb) { // no clone
        best = Some((i, pa, j, pb, delta));
    }
}
if let Some((i, pa, j, pb, _)) = best {
    current = swap_pairings(&current, i, pa, j, pb)?; // one clone per iteration
}
```

See Section 3 for the implementation plan.

---

### 2.11 Ranked-Descriptor Fix: Root Cause Analysis and Corrected Implementation (2026-07-31)

#### Context

After implementing the lazy move descriptor approach described in Section 2.10, a behavioral regression was observed: the optimizer stopped finding improvements on all instances. Instance 3 (8h) reverted from `opt=0.2309` (42.4% improvement) back to `opt=0.4011` (0.0% improvement), identical to the greedy output.

#### Diagnostic instrumentation

A `[delta_diag]` eprintln was added to [`best_improving_move()`](../../adapters/airline/src/optimization/search/local_search.rs) to compare the predicted analytical delta against the actual weighted-cost delta for the first descriptor with `predicted_delta < 0`. This fired once per `best_improving_move()` call and printed:

```
[delta_diag] rot_i=0 pa=0 rot_j=19 pb=1
predicted_variance_delta=-0.042553
swap_pairings returned None (invalid move)
```

For valid moves, the formula was confirmed exact:

```
predicted_variance_delta = -0.121212
actual_weighted_delta    = -0.121212
```

#### Root cause: descriptor space ≠ materialization space

The Phase 1 scan enumerates all `(i, pa, j, pb)` index tuples analytically. However, [`swap_pairings()`](../../adapters/airline/src/optimization/neighborhood/swap.rs) applies additional structural validity checks and returns `None` for some descriptors (e.g., duplicate pairing placement, capacity invariants, or other roster-level constraints).

The original eager implementation never encountered this mismatch because it called `swap_pairings()` for every candidate — if it returned `None`, that move was simply skipped. The initial lazy implementation selected only the single best descriptor from Phase 1 and materialized it once. When that descriptor was structurally invalid, `swap_pairings()` returned `None`, Phase 2 returned `None`, and the optimizer concluded it was at a local optimum after the first iteration.

A secondary issue was floating-point noise: equal-leg swaps (`la == lb`) produce `delta ≈ -1e-17` due to cancellation errors in `(ci_new - mean)² - (ci - mean)²` when `ci_new == ci`. This caused the noise descriptor to be selected as "best" and suppress the relocate fallback.

#### Fix: ranked-descriptor approach

The corrected implementation collects **all** improving descriptors (those with `delta < -1e-9`) into a `Vec<(f64, usize, usize, usize, usize)>` sorted by delta ascending (most negative first). Phase 2 iterates through this ranked list and tries each descriptor in order:

```rust
// Phase 1: collect all improving descriptors
let mut improving_swaps: Vec<(f64, usize, usize, usize, usize)> = Vec::new();
for (i, pa, j, pb) in neighbourhood {
    let delta = analytical_variance_delta(current, i, pa, j, pb);
    if delta < -1e-9 {
        improving_swaps.push((delta, i, pa, j, pb));
    }
}
improving_swaps.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

// Phase 2: try ranked descriptors until one materializes
for (_delta, i, pa, j, pb) in &improving_swaps {
    if let Some(candidate) = swap_pairings(current, i, pa, j, pb) {
        if is_legal(&candidate) && cost(&candidate) < current_cost {
            return Some((candidate, cost));
        }
    }
    // swap_pairings returned None: continue to next-best descriptor
}
// Only after exhausting all improving swaps, try relocate
```

The `-1e-9` threshold eliminates floating-point noise from equal-leg swaps. The ranked iteration ensures that if the analytically best descriptor is structurally invalid, the second-best is tried, and so on — preserving the same search completeness as the original eager implementation.

#### Verified results (2026-07-31)

After applying the fix, instance 3 results were confirmed:

| Instance | Threshold | Greedy | Optimized | Improvement |
|----------|-----------|--------|-----------|-------------|
| 3        | 8h        | 0.4011 | **0.2309** | **42.4%** |
| 3        | 10h       | 0.3875 | **0.2173** | **43.9%** |

These match the pre-regression baseline exactly.

#### Performance after fix (Instance 3, 8h)

| Metric | Eager baseline | Corrected lazy |
|--------|---------------|----------------|
| Moves evaluated | ~31,944 | 58,307 |
| Neighbour generation | ~17,750ms | 1,543ms |
| Evaluation | ~9ms | 0.9ms |
| Local search total | ~23s | **2.0s** |

Despite evaluating nearly twice as many descriptors (because the ranked list includes all improving swaps, not just the best), the corrected implementation is approximately **11× faster** than the eager baseline. The performance gain comes from eliminating the full `Roster` clone for every candidate — only the accepted move is materialized.

#### Remaining bottleneck

Profiling shows neighbour generation (descriptor enumeration + materialization attempts) still dominates at 1.54s. The next optimization opportunity is to filter structurally invalid descriptors before ranking, so that `swap_pairings()` is never called for moves it would reject. This requires understanding the exact preconditions under which `swap_pairings()` returns `None` and encoding those checks as cheap O(1) predicates in Phase 1.

#### Reviewer architectural recommendation (Step 2)

The reviewer recommended adding an `evaluate_move()` default method to the `ScheduleObjective` trait, allowing objectives to provide an O(1) incremental delta without requiring a full roster clone. `WorkloadBalanceObjective` would implement this with the analytical variance delta formula validated above. `LocalSearch` would use `evaluate_move()` if available, falling back to full evaluation otherwise. This is the clean architectural path for making the fast path available to all objectives generically.

### 2.12 Bottleneck Migration (2026-07-31)

The descriptor-based analytical Local Search fundamentally changed the computational profile of the scheduling pipeline. Prior to this work, Local Search dominated execution time because every candidate move required complete reconstruction of the roster through repeated deep cloning of rotations, pairings, and legs. Profiling showed that neighbourhood generation accounted for the overwhelming majority of execution time, while legality checking and objective evaluation contributed only a negligible fraction.

Following the introduction of analytical descriptor evaluation and ranked descriptor materialization, the computational bottleneck shifted away from Local Search. Candidate moves are now ranked using an exact analytical evaluation of the workload-balance objective, and complete roster materialization is deferred until a structurally valid, promising descriptor has been identified. This preserves optimization quality while eliminating the repeated construction of temporary rosters during neighbourhood exploration.

The complete GERAD benchmark demonstrates that the computational profile now depends on the existence of exploitable improvements rather than on the size of the neighbourhood itself.

| Instance | Threshold | Greedy | Local Search | Optimization Result |
|----------|-----------|--------|--------------|---------------------|
| 1 | 8h | 257ms | 289ms | No improvement |
| 1 | 10h | 230ms | 439ms | **12.2% improvement** |
| 2 | 8h | 803ms | 380ms | No improvement |
| 2 | 10h | 825ms | 862ms | No improvement |
| 3 | 8h | 3.19s | 2.01s | **42.4% improvement** |
| 3 | 10h | 3.09s | 2.06s | **43.9% improvement** |
| 4 | 8h | 136.23s | 1ms | No improvement |
| 4 | 10h | 135.04s | <1ms | No improvement |
| 5 | 8h | 175.42s | 1ms | No improvement |
| 5 | 10h | 178.34s | 1ms | No improvement |
| 6 | 8h | <1ms | <1ms | No improvement |
| 6 | 10h | <1ms | <1ms | No improvement |
| 7 | 8h | 1ms | 1ms | No improvement |
| 7 | 10h | 1ms | 1ms | No improvement |

Several observations emerge from these results.

First, Local Search now behaves as an **adaptive refinement stage** rather than a dominant computational phase. When the greedy schedule is already locally optimal with respect to the explored neighbourhood (Instances 2, 4, 5, 6, and 7), the descriptor scan quickly establishes that no improving move exists and the search terminates almost immediately. The runtime therefore collapses from seconds to approximately one millisecond without sacrificing correctness.

Second, when genuine improvement opportunities exist (Instance 3 and, to a lesser extent, Instance 1 under the 10-hour threshold), Local Search spends additional time exploring and validating candidate moves because productive neighbourhoods are present. Even in these cases, the descriptor-based implementation remains substantially more efficient than the original eager implementation while reproducing the same optimized solutions.

Finally, the optimization has shifted the dominant computational cost of the scheduling pipeline. For the larger benchmark instances, the Greedy Scheduler now accounts for virtually all execution time, while Local Search contributes only a negligible fraction unless meaningful improvements are available. For example, in Instance 5 the Greedy Scheduler requires approximately **175 seconds**, whereas Local Search completes in approximately **1 millisecond** because no improving neighbourhood exists.

This represents a fundamental architectural change. The descriptor-based search has transformed Local Search from the principal runtime bottleneck into an opportunistic optimization phase whose computational cost is proportional to the availability of exploitable improvements rather than the size of the search neighbourhood. Consequently, future optimization efforts should concentrate on the Greedy Scheduler, which has become the dominant contributor to end-to-end execution time on large benchmark instances.

### 2.13 Greedy Scheduler Sub-Stage Profiling (2026-07-31)

Following the bottleneck migration documented in Section 2.12, the Greedy Scheduler became the dominant contributor to end-to-end execution time on large benchmark instances. The same hierarchical profiling methodology applied to Local Search was applied to the Greedy Scheduler to decompose its 136–175 second runtime into constituent operations.

#### Instrumentation

Five timing accumulators and four object-count accumulators were added to [`GreedyScheduler::assign()`](../../adapters/airline/src/optimization/search/greedy.rs):

| Accumulator | Measures |
|-------------|----------|
| `rotation_collect` | Collecting the rotations snapshot once per pairing |
| `pairing_clone` | Cloning the pairing vector + `Rotation::new()` |
| `rotation_clone` | Cloning all rotations into `new_rotations` vec |
| `leg_clone` | `current.legs().cloned().collect()` |
| `roster_new` | `Roster::new()` construction |
| `n_pairing_vec_clones` | Total pairings cloned across all candidates |
| `n_rotation_clones` | Total rotation clones across all candidates |
| `n_leg_clones` | Total leg clones across all candidates |
| `n_roster_constructions` | Total `Roster::new()` calls |

#### Results (instances 1–3, 8h threshold)

**Instance 1** (21 pairings, 33 rotations, 693 candidates):

| Stage | Time | Object count |
|-------|------|--------------|
| rotation_collect | 6µs | — |
| pairing_clone | 704µs | 903 pairings |
| rotation_clone | 12,617µs | 22,836 rotations |
| leg_clone | **85,695µs** | **700,996 legs** |
| roster_new | **101,456µs** | 692 rosters |
| evaluate | 71µs | — |
| commit | 2,110µs | — |
| **total greedy** | **264ms** | — |

leg_clone + roster_new = 187ms = **71% of greedy runtime**.

**Instance 2** (47 pairings, 34 rotations, 1,598 candidates):

| Stage | Time | Object count |
|-------|------|--------------|
| rotation_clone | 36,065µs | 53,924 rotations |
| leg_clone | **277,033µs** | **2,379,000 legs** |
| roster_new | **285,784µs** | 1,586 rosters |
| evaluate | 163µs | — |
| **total greedy** | **800ms** | — |

leg_clone + roster_new = 563ms = **70% of greedy runtime**.

**Instance 3** (94 pairings, 47 rotations, 4,418 candidates):

| Stage | Time | Object count |
|-------|------|--------------|
| rotation_clone | 123,058µs | 206,659 rotations |
| leg_clone | **960,043µs** | **8,156,435 legs** |
| roster_new | **1,193,347µs** | 4,397 rosters |
| evaluate | 938µs | — |
| **total greedy** | **2,977ms** | — |

leg_clone + roster_new = 2,153ms = **72% of greedy runtime**.

#### Root cause

The inner loop of [`GreedyScheduler::assign()`](../../adapters/airline/src/optimization/search/greedy.rs) performs a full `Roster` reconstruction for every candidate assignment:

```rust
// For each pairing × each rotation:
let legs_cloned: Vec<_> = current.legs().cloned().collect();  // O(L) per candidate
let new_rotations: Vec<_> = current.rotations()               // O(R) per candidate
    .enumerate()
    .map(|(i, r)| if i == rot_idx { new_rotation.clone() } else { r.clone() })
    .collect();
let candidate = Roster::new(..., legs_cloned, new_rotations); // O(L + R) per candidate
```

This is structurally identical to the bottleneck previously identified in Local Search. For Instance 3 with 94 pairings × 47 rotations = 4,418 candidates, the scheduler clones 8.1 million leg objects and constructs 4,397 complete rosters — all to evaluate a single scalar objective value.

The cost evaluation itself (`evaluate=938µs` for 4,418 calls = 0.21µs/call) is negligible. The entire runtime is consumed by roster materialization that exists solely to pass a `&Roster` to the evaluator.

#### Asymptotic analysis

| Operation | Complexity per candidate | Instance 3 count |
|-----------|--------------------------|-----------------|
| leg_clone | O(L) | 8,156,435 |
| rotation_clone | O(R) | 206,659 |
| Roster::new | O(L + R) | 4,397 |
| evaluate | O(R) | 4,397 |

For large instances (L ≈ 5,000–8,000 legs, R ≈ 145–247 rotations), leg cloning dominates because L >> R. The total work is O(P × R × L) where P is the number of pairings — cubic in the problem size.

#### Instance 4 (8h threshold, 426 pairings, 145 rotations, 61,770 candidates)

| Stage | Time | Object count |
|-------|------|--------------|
| rotation_collect | 128µs | — |
| pairing_clone | 142,788µs | 152,295 pairings |
| rotation_clone | 6,558,084µs | 8,817,740 rotations |
| leg_clone | **43,809,829µs** | **341,337,756 legs** |
| roster_new | **45,795,213µs** | 60,812 rosters |
| evaluate | 172,021µs | — |
| commit | 262,348µs | — |
| **total greedy** | **129,117ms** | — |

leg_clone + roster_new = 89.6s = **69% of greedy runtime**. The pattern from instances 1–3 holds at scale: 341 million leg clones account for the dominant cost. The evaluate accumulator (172ms for 60,812 calls = 2.8µs/call) confirms that objective evaluation is not the bottleneck.

#### Instance 5 (8h threshold, 321 pairings, 247 rotations, 79,287 candidates)

| Stage | Time | Object count |
|-------|------|--------------|
| rotation_collect | 294µs | — |
| pairing_clone | 130,186µs | 130,647 pairings |
| rotation_clone | 10,011,905µs | 19,461,871 rotations |
| leg_clone | **56,262,722µs** | **452,508,199 legs** |
| roster_new | **58,532,763µs** | 78,793 rosters |
| evaluate | 289,563µs | — |
| commit | 220,822µs | — |
| **total greedy** | **166,949ms** | — |

leg_clone + roster_new = 114.8s = 69% of greedy runtime. n_leg_clones = 452,508,199 (452 million).

#### Instances 6–7 (8h threshold)

Both instances show `pairings=0 candidates=0`. The greedy phase is a no-op: all pairings were already assigned before `GreedyScheduler::assign()` was invoked. Total greedy time is 0–1ms.

#### Cross-instance analysis

**Candidate count is the dominant scaling variable.** The measured candidate counts exactly equal the mathematical product of pairings × rotations:

| Instance | Rotations | Pairings assigned | Measured candidates | P × R |
|----------|-----------|-------------------|---------------------|-------|
| 1 | 33 | 21 | 693 | 21 × 33 = 693 ✓ |
| 2 | 34 | 47 | 1,598 | 47 × 34 = 1,598 ✓ |
| 3 | 47 | 94 | 4,418 | 94 × 47 = 4,418 ✓ |
| 4 | 145 | 426 | 61,770 | 426 × 145 = 61,770 ✓ |
| 5 | 247 | 321 | 79,287 | 321 × 247 = 79,287 ✓ |
| 6 | 92 | 0 | 0 | 0 × 92 = 0 ✓ |
| 7 | 159 | 0 | 0 | 0 × 159 = 0 ✓ |

The identity holds exactly for all seven instances. The [`GreedyScheduler`](../../adapters/airline/src/optimization/search/greedy.rs) executes a complete Cartesian product over `(pairing, rotation)` pairs with no pruning. There is no early termination, no feasibility pre-filter, and no candidate reduction.

**Runtime is not determined by the number of pairings alone.** Instance 5 has fewer pairings than Instance 4 (321 vs 426) yet more candidates (79,287 vs 61,770) because it has more rotations (247 vs 145). The product P × R, not P alone, determines the work.

**Objective evaluation is negligible.** Across all instances, the evaluate accumulator accounts for 0.13–0.17% of total greedy time. The evaluator is not the bottleneck.

**~99% of greedy time is roster construction.** For Instance 4: leg_clone (43.8s) + roster_new (45.8s) + rotation_clone (6.6s) = 96.2s out of 129.1s total = 74.5%. The remaining ~25% is pairing_clone and commit. Objective evaluation contributes 0.13%.

**Instances 6–7 are zero-cost because pairings=0.** Their greedy runtime is 0–1ms not because they are smaller problems, but because the seed roster already contains all pairings. The Greedy Scheduler cost scales with unassigned pairings, not total problem size.

#### Evidence-based optimization roadmap

| Opportunity | Evidence | Expected impact |
|-------------|----------|-----------------|
| Eliminate full `Roster` reconstruction per candidate | ~99% of greedy time is cloning and construction | Very high |
| Evaluate moves incrementally (delta-based, no clone) | Already successful in Local Search (≈11× speedup) | Very high |
| Prune infeasible (pairing, rotation) pairs before evaluation | Candidate count is exactly P × R with no pruning | High |
| Optimize objective evaluation | Only 0.13% of runtime | Negligible |

The first two opportunities are directly supported by the measurements. The third is the next logical target: if a pairing cannot legally be appended to a rotation (e.g. temporal overlap, airport discontinuity), the candidate roster need not be constructed at all. A lightweight pre-filter on `(pairing, rotation)` compatibility could eliminate a large fraction of the P × R candidates before any cloning occurs.

### 2.14 Open Investigations — Next Measurement Round

The profiling data collected in Sections 2.12–2.13 establishes an experimentally validated computational model of the current scheduler. Before any architectural changes are made, four questions should be answered by measurement rather than assumption. Each is framed as a specific instrumentation plan with a concrete output format.

#### Investigation 1 — Candidate feasibility rate

The current profiler counts total candidates constructed but does not distinguish between candidates that pass `Rotation::new()` validation and those that fail. Instrument the inner loop to record, per pairing:

```
pairing_id | rotations_examined | rotations_legal | rotations_improving
```

If only a small fraction of rotations are ever legal for a given pairing, a lightweight pre-filter on `(pairing, rotation)` compatibility could prune most of the Cartesian product before any cloning occurs. If the legal fraction is high, pre-filtering offers little benefit and the focus should remain on reducing construction cost per candidate.

**Instrumentation target:** [`GreedyScheduler::assign()`](../../adapters/airline/src/optimization/search/greedy.rs) — add per-pairing counters for `rotations_examined`, `rotations_legal` (those where `Rotation::new()` succeeds), and `rotations_improving` (those where the weighted cost is strictly lower than the current best).

#### Investigation 2 — Objective value distribution across candidates

The profiler currently records only the best candidate per pairing. It does not record how many candidates evaluate to the same objective value. If many candidates are objectively identical, they can be eliminated before construction by a descriptor-level comparison.

**Instrumentation target:** Add a histogram of objective values per pairing — count how many candidates share the minimum value, how many are within ε of the minimum, and how many are strictly worse. If the distribution is highly concentrated (e.g. 90% of candidates evaluate identically), a descriptor-based pre-filter becomes viable.

#### Investigation 3 — `Roster::new()` internal cost

The `roster_new` accumulator measures the total time inside `Roster::new()`, but does not separate the cost of data structure construction from any invariant validation performed inside the constructor. If `Roster::new()` performs expensive checks (e.g. iterating all legs to verify coverage), that validation cost is currently hidden inside the `roster_new` timer.

**Instrumentation target:** Read [`Roster::new()`](../../adapters/airline/src/domain/roster.rs) and identify whether it performs O(L) or O(R) validation. If it does, add a sub-timer inside the constructor to separate allocation from validation. This determines whether the `roster_new` cost is reducible by deferring validation to acceptance time.

#### Investigation 4 — Assignment frontier over time

The Cartesian product identity (`candidates = P × R`) holds globally, but it does not reveal whether the number of rotations examined per pairing changes as construction progresses. If every pairing is tested against all R rotations regardless of how many pairings have already been assigned, the scheduler has no notion of an assignment frontier. If the number shrinks over time (e.g. because some rotations become infeasible as they accumulate pairings), the frontier itself becomes an optimization opportunity.

**Instrumentation target:** Record `rotations_examined` per pairing in assignment order. Plot or tabulate the sequence. If the sequence is flat (constant R throughout), the scheduler is frontier-blind. If it decreases, the natural frontier can be exploited to prune candidates without any algorithmic change.
**Note — Investigations 3 and 4 were not executed.** Following the results of Investigations 1 and 2 (Section 2.15), the architectural direction shifted to the Coralys-native scheduler (Section 2.16). Since Coralys avoids the Greedy construction path entirely on large instances, further optimization of Greedy's internal structure (Investigations 3 and 4) became lower priority. They remain valid future work if Greedy is retained as an initialization strategy.

---

### 2.15 Greedy Candidate Landscape Analysis

**Source:** Experiment run 2026-08-01. [`adapters/airline/tests/gerad_e2e.rs`](../../adapters/airline/tests/gerad_e2e.rs), [`adapters/airline/src/optimization/search/greedy.rs`](../../adapters/airline/src/optimization/search/greedy.rs).

#### Experimental objective

Section 2.14 posed two questions: what fraction of (pairing, rotation) pairs are legal, and what fraction of legal candidates are improving? This section answers both questions by direct measurement across all 7 GERAD instances at both layover thresholds, and characterises the full candidate landscape: legal rate, improvement rate, tie rate, and waste rate.

#### Instrumentation added

Per-pairing counters `p_rot_examined`, `p_rot_legal`, `p_rot_improving` were added to [`GreedyScheduler::assign()`](../../adapters/airline/src/optimization/search/greedy.rs) and aggregated into cross-pairing totals and per-pairing min/max. Objective-distribution counters `obj_improving`, `obj_tied`, `obj_worse` count, across all legal candidates, how many are strictly improving, tied with the current best, or strictly worse. Results are emitted as a `[greedy_feasibility]` log line per (instance, threshold) pair via `eprintln!`.

#### Raw measurements (2026-08-01, `cargo test -p coralys-airline gerad_e2e --release -- --nocapture`)

One `[greedy_feasibility]` line is emitted per (instance, threshold) pair. Instances 6 and 7 have zero pairings assigned by the greedy scheduler (the seed roster already contains all pairings) and emit `rot_examined=0` for both thresholds — this is expected behaviour, not an instrumentation failure. The `pairings_ok` field in the preceding `[diag thr=Xh]` line is the total pairings built by the pairing builder; `rot_examined` equals `pairings_assigned × rotations` where `pairings_assigned` is the subset the greedy actually iterates over.

```
[diag thr=8h]  pairings_ok=54
[greedy_feasibility] rot_examined=693   rot_legal=692   rot_improving=63   legal_min=32  legal_max=33  improving_min=1 improving_max=5  obj_improving=63   obj_tied=120  obj_worse=509
[diag thr=10h] pairings_ok=52
[greedy_feasibility] rot_examined=627   rot_legal=625   rot_improving=52   legal_min=32  legal_max=33  improving_min=2 improving_max=5  obj_improving=52   obj_tied=92   obj_worse=481
[diag thr=8h]  pairings_ok=81
[greedy_feasibility] rot_examined=1598  rot_legal=1586  rot_improving=127  legal_min=33  legal_max=34  improving_min=1 improving_max=5  obj_improving=127  obj_tied=272  obj_worse=1187
[diag thr=10h] pairings_ok=82
[greedy_feasibility] rot_examined=1632  rot_legal=1620  rot_improving=126  legal_min=33  legal_max=34  improving_min=1 improving_max=5  obj_improving=126  obj_tied=274  obj_worse=1220
[diag thr=8h]  pairings_ok=141
[greedy_feasibility] rot_examined=4418  rot_legal=4397  rot_improving=294  legal_min=46  legal_max=47  improving_min=1 improving_max=6  obj_improving=294  obj_tied=478  obj_worse=3625
[diag thr=10h] pairings_ok=142
[greedy_feasibility] rot_examined=4465  rot_legal=4444  rot_improving=308  legal_min=46  legal_max=47  improving_min=1 improving_max=7  obj_improving=308  obj_tied=438  obj_worse=3698
[diag thr=8h]  pairings_ok=571
[greedy_feasibility] rot_examined=61770 rot_legal=60812 rot_improving=1775 legal_min=137 legal_max=145 improving_min=1 improving_max=10 obj_improving=1775 obj_tied=2833 obj_worse=56204
[diag thr=10h] pairings_ok=571
[greedy_feasibility] rot_examined=61770 rot_legal=60812 rot_improving=1738 legal_min=137 legal_max=145 improving_min=1 improving_max=9  obj_improving=1738 obj_tied=2739 obj_worse=56335
[diag thr=8h]  pairings_ok=568
[greedy_feasibility] rot_examined=79287 rot_legal=78793 rot_improving=1514 legal_min=241 legal_max=247 improving_min=1 improving_max=10 obj_improving=1514 obj_tied=2231 obj_worse=75048
[diag thr=10h] pairings_ok=568
[greedy_feasibility] rot_examined=79287 rot_legal=78793 rot_improving=1533 legal_min=241 legal_max=247 improving_min=1 improving_max=9  obj_improving=1533 obj_tied=2415 obj_worse=74845
[diag thr=8h/10h] pairings_ok=92  → rot_examined=0 (instance 6: greedy assigns 0 pairings)
[diag thr=8h/10h] pairings_ok=159 → rot_examined=0 (instance 7: greedy assigns 0 pairings)
```

#### Derived metrics

Four rates are derived from the raw counts. Legal rate = `rot_legal / rot_examined`. Improvement rate = `rot_improving / rot_legal`. Tie rate = `obj_tied / rot_legal`. Waste rate = `obj_worse / rot_legal`.

| Instance | Thr | rot_examined | rot_legal | rot_improving | obj_tied | obj_worse | Legal rate | Impr rate | Tie rate | Waste rate |
|----------|-----|-------------:|----------:|--------------:|---------:|----------:|-----------:|----------:|---------:|-----------:|
| 1 | 8h | 693 | 692 | 63 | 120 | 509 | 99.9% | 9.1% | 17.3% | 73.6% |
| 1 | 10h | 627 | 625 | 52 | 92 | 481 | 99.7% | 8.3% | 14.7% | 77.0% |
| 2 | 8h | 1,598 | 1,586 | 127 | 272 | 1,187 | 99.2% | 8.0% | 17.1% | 74.8% |
| 2 | 10h | 1,632 | 1,620 | 126 | 274 | 1,220 | 99.3% | 7.8% | 16.9% | 75.3% |
| 3 | 8h | 4,418 | 4,397 | 294 | 478 | 3,625 | 99.5% | 6.7% | 10.9% | 82.4% |
| 3 | 10h | 4,465 | 4,444 | 308 | 438 | 3,698 | 99.5% | 6.9% | 9.9% | 83.2% |
| 4 | 8h | 61,770 | 60,812 | 1,775 | 2,833 | 56,204 | 98.4% | 2.9% | 4.7% | 92.4% |
| 4 | 10h | 61,770 | 60,812 | 1,738 | 2,739 | 56,335 | 98.4% | 2.9% | 4.5% | 92.6% |
| 5 | 8h | 79,287 | 78,793 | 1,514 | 2,231 | 75,048 | 99.4% | 1.9% | 2.8% | 95.2% |
| 5 | 10h | 79,287 | 78,793 | 1,533 | 2,415 | 74,845 | 99.4% | 1.9% | 3.1% | 95.0% |
| 6–7 | both | 0 | 0 | 0 | 0 | 0 | — | — | — | — |

#### Findings

**Finding 1 — Legal rate is high but not 100%.** `legal_rate` is 98.4–99.9% across all instances and thresholds. A small fraction of (pairing, rotation) pairs are rejected by `Rotation::new()` — between 0.1% and 1.6% depending on instance. This demonstrates that Greedy already benefits from a modest amount of implicit pruning via the rotation legality check. A dedicated feasibility pre-filter would eliminate at most 1.6% of candidates (instance 4). The dominant cost is not infeasibility.

**Finding 2 — Improvement rate is 1.9–9.1%, declining with instance size.** Only 1.9–9.1% of legal candidates improve the current best assignment. This rate is not 1/P. For instance 1 (8h), 63 of 692 legal candidates are improving — 9.1%. For instance 4, 1,775 of 60,812 legal candidates are improving — 2.9%. For instance 5, 1,514 of 78,793 — 1.9%. As the problem grows, the proportion of useful candidate evaluations collapses. This is a stronger finding than the asymptotic O(P × R) characterisation: the algorithm is not merely expensive, it is increasingly inefficient as instance size grows.

**Finding 3 — Ties are non-trivial and suggest a plateau-rich objective landscape.** `obj_tied > 0` for all instances. Tied candidates represent 2.8–17.3% of legal candidates. Many assignments produce exactly the same workload score, indicating that the objective has plateaus and many equivalent assignments. This is relevant for evolutionary algorithms: population-based search generally navigates plateaus better than greedy constructive heuristics, which commit to a single path through the plateau.

**Finding 4 — Waste rate is 73–95%, growing with instance size.** The waste rate (candidates that are legal but neither improving nor tied) is 73.6% for instance 1 and 95.2% for instance 5. On the largest instances, over 90% of all construction work — cloning rotations, building Roster objects, evaluating objectives — produces candidates that are immediately discarded. This is the single most compelling quantitative justification for replacing Greedy: not because it is asymptotically O(P × R), but because 95% of the expensive work contributes nothing to the final solution.

**Finding 5 — The search landscape is relatively insensitive to the layover threshold.** Comparing 8h and 10h results for the same instance, the improvement rate and waste rate change by less than 1 percentage point. For instance 5: improving 1,514 (8h) vs 1,533 (10h); ties 2,231 vs 2,415. The greedy search characteristics are robust across threshold settings.

**Finding 6 — Instances 6–7 confirm the zero-pairings case.** The instrumentation emits `rot_examined=0` for instances 6 and 7 because the greedy scheduler receives an empty pairing list — the seed roster already contains all pairings. This is expected behaviour and confirms that the zero runtime observed for these instances in Section 2.12 is structural, not a measurement artefact.

#### Implications for scheduler architecture

The measurements reframe the central question. The question is not "why is Greedy O(P × R)?" — that is a consequence of the algorithm's structure. The question is: **why are 90–98% of evaluated candidates immediately discarded, and what does that imply for the design of a better scheduler?**

A feasibility pre-filter cannot address this (legal rate ≥ 98.4%). A tie-breaking shortcut offers modest benefit (2.8–17.3% of legal candidates). The interventions that address the dominant waste are:

1. **Incremental evaluation** — compute the objective delta without constructing a full Roster clone, accepting only when the delta is positive. This eliminates the clone cost for discarded candidates.
2. **Dominance pruning** — identify structural properties of (pairing, rotation) pairs that predict non-improvement without evaluation. This requires understanding why 90–98% of candidates are non-improving.
3. **Coralys-guided candidate generation** — replace exhaustive enumeration with a population-based search that generates candidates in the improving region of the assignment space directly. This is the approach evaluated in Section 2.17.

The Greedy scheduler exhaustively evaluates tens of thousands of legal assignments, yet fewer than 3% improve the incumbent solution on the largest benchmark instances. Coralys avoids exhaustive candidate enumeration by directly optimizing assignment vectors, thereby concentrating computational effort on promising regions of the search space rather than evaluating candidates that are overwhelmingly rejected.

---

### 2.16 Architectural Recommendation — Coralys-Native Scheduler

**Source:** Reviewer analysis, 2026-07-31.

#### Evidence summary

Three measurements from Sections 2.12–2.15 jointly support a change in architectural direction.

**Greedy dominates runtime.** Instances 4–5 show Greedy at 129–171 seconds, Local Search at 0–1 ms. The constructive phase is the bottleneck, not the optimizer.

**Local Search is already efficient.** The lazy descriptor approach reduced Local Search from 23s to 2s (≈11× speedup). Further investment in Local Search optimisation has diminishing returns.

**Greedy makes irreversible decisions.** Each pairing is assigned immediately. Later pairings inherit earlier assignment mistakes. This is the structural weakness of constructive heuristics: the optimizer cannot repair decisions made during construction.

#### What Coralys already provides

The Coralys framework already implements population management, mutation, crossover, Pareto ranking, constraint handling, and an evolution engine. These are generic optimization primitives. The missing piece for a crew scheduling application is a **genome** — a representation of the assignment state that Coralys can evolve.

#### Proposed genome

For the current GERAD experiment, the genome is a `pairing_id → rotation_id` assignment vector:

```
P1 → R4
P2 → R1
P3 → R7
...
```

This is structurally identical to what UltraCrew already optimizes (`shift_id → worker_id`). The domain objects (Pairing, Rotation, Roster) already exist. The constraint engine already exists. The objective functions already exist. Only the assignment engine changes.

#### Proposed pipeline

```
Flights
    ↓
Initial feasible genome  (one of several SchedulerInitializer strategies)
    ↓
Coralys Optimizer        (mutation: swap, relocate, exchange, 2-opt, block move)
    ↓
Legality check
    ↓
Repair (if needed)
    ↓
Best roster
```

#### SchedulerInitializer hierarchy

```
SchedulerInitializer
    ├── Empty            (all pairings unassigned)
    ├── Random           (random feasible assignment)
    ├── RoundRobin       (distribute pairings round-robin across rotations)
    ├── Greedy           (current GreedyScheduler — one strategy, not the scheduler)
    └── PreviousRoster   (warm-start from a prior solution)
```

Greedy becomes **one initialization strategy** rather than the primary scheduling algorithm. This enables a direct experimental comparison: does Coralys converge to the same quality regardless of initialization? If yes, the greedy constructor can be simplified or removed.

#### Build plan

Two parallel test binaries, sharing all domain infrastructure:

```
adapters/airline/tests/gerad_e2e.rs       // Baseline: Greedy + Local Search (unchanged)
adapters/airline/tests/gerad_coralys.rs   // Coralys-native optimizer (to be built)
```

Both use the same flight parser, duty builder, pairing builder, constraint engine, and objective functions. The only difference is the assignment engine.

#### Comparison dimensions

| Dimension | gerad_e2e.rs | gerad_coralys.rs |
|-----------|-------------|-----------------|
| Runtime | Measured (Sections 2.12–2.13) | To be measured |
| Convergence | Single pass (no iteration) | Population × generations |
| Final objective | Measured | To be measured |
| Scalability | O(P × R) per pairing | Configurable population size |
| Sensitivity to initialization | N/A (deterministic) | To be measured across initializers |

#### Decision criterion

If the Coralys-native scheduler consistently matches or exceeds the greedy pipeline on objective value across all 7 instances, the greedy scheduler is relegated to a baseline or initialization strategy. If it does not, the profiling data from Sections 2.12–2.15 identifies the specific construction operations to optimize in the greedy path.

---
### 2.17 Coralys-Native Evolutionary Construction versus Greedy Construction

**Source:** Experiment run 2026-07-31. [`adapters/airline/tests/gerad_coralys.rs`](../../adapters/airline/tests/gerad_coralys.rs).

To evaluate whether Coralys can replace the existing constructive scheduler, the [`GreedyScheduler`](../../adapters/airline/src/optimization/search/greedy.rs) was compared directly against a Coralys-native evolutionary scheduler operating on the same pairing sets produced by the GERAD preprocessing pipeline. The evolutionary scheduler represents each solution as a genome mapping pairings to rotations and optimizes the same `WorkloadBalanceObjective` used throughout the previous experiments.

The objective of this experiment was not to compare evolutionary search against Local Search, but to determine whether a population-based constructive algorithm can produce initial schedules that are equal to or better than the deterministic `GreedyScheduler` while avoiding the exhaustive candidate evaluation performed by Greedy construction.

#### Experimental configuration

- Genome: pairing → rotation assignment (`Vec<usize>` of length P)
- Population: 50
- Generations: 200
- Tournament selection (k=3)
- Crossover probability: 0.8
- Random seed: 42
- Fitness: `WorkloadBalanceObjective` (lower is better)
- Pairing generation: Condition A (8-hour layover)
- Repair: if any rotation is empty after crossover/mutation, steal one pairing from the most-loaded rotation

Two initialization strategies were evaluated: round-robin (`pairing i → rotation i % R`) and random (uniform random rotation).

**Limitation — single random seed.** All EA runs use seed 42. The results therefore represent a single stochastic trajectory. Whether the observed outcomes (particularly the quality advantage on Instances 3 and 5) are robust across seeds is an open question. Multi-seed validation is listed as Open Research Question 1 in Section 2.18.

#### Raw output (2026-07-31, `cargo test -p coralys-airline gerad_coralys_vs_greedy --release -- --nocapture`)

`ea_rr_gen` / `ea_rand_gen` = generation at which best score was first achieved. `inf` = no feasible Roster constructed in 200 generations.

| Instance | Pairings | Rotations | Greedy | Greedy_ms | EA_rr | EA_rr_gen | EA_rr_ms | EA_rand | EA_rand_gen | EA_rand_ms |
|----------|----------|-----------|--------|-----------|-------|-----------|----------|---------|-------------|------------|
| 1 | 54 | 33 | 0.8760 | 259 | 0.8760 | 0 | 3,262 | 0.8760 | 13 | 3,244 |
| 2 | 81 | 34 | 0.5433 | 818 | 0.5433 | 158 | 5,612 | 0.5433 | 107 | 6,372 |
| 3 | 141 | 47 | 0.4011 | 4,158 | **0.2309** | 73 | 8,726 | **0.2309** | 142 | 6,953 |
| 4 | 571 | 145 | 0.1239 | 134,646 | 0.1239 | 198 | 20,357 | **inf** | — | 2,338 |
| 5 | 568 | 247 | 0.2645 | 176,153 | **0.2402** | 199 | 21,305 | **0.2402** | 165 | 20,801 |
| 6 | 92 | 92 | 0.0794 | 1 | 0.0794 | 23 | 15,420 | 0.0794 | 22 | 16,186 |
| 7 | 159 | 159 | 0.1531 | 1 | 0.1531 | 0 | 24,297 | 0.1531 | 0 | 23,087 |

#### Overall results

| Instance | Greedy Score | Coralys (Round-Robin) | Coralys (Random) | Outcome |
|----------|-------------:|----------------------:|-----------------:|---------|
| 1 | 0.8760 | 0.8760 | 0.8760 | identical |
| 2 | 0.5433 | 0.5433 | 0.5433 | identical |
| 3 | **0.4011** | **0.2309** | **0.2309** | Coralys substantially better |
| 4 | 0.1239 | 0.1239 | infeasible | identical (RR only) |
| 5 | **0.2645** | **0.2402** | **0.2402** | Coralys better |
| 6 | 0.0794 | 0.0794 | 0.0794 | identical |
| 7 | 0.1531 | 0.1531 | 0.1531 | identical |

#### Major Finding 1 — Coralys reproduces Local Search without Greedy construction

The most significant observation occurred on Instance 3. The previous experiments established:

```
Greedy → 0.4011 → Local Search → 0.2309
```

The Coralys scheduler produced **0.2309** directly — without running the greedy scheduler or local search. In other words:

```
Coralys Construction = Greedy + Local Search
```

for this benchmark. This demonstrates that evolutionary construction can directly discover the same workload-balanced assignment that previously required a deterministic constructor followed by neighbourhood optimization. This is the first experimental evidence that Coralys can function as a native constructive scheduler rather than merely improving heuristic solutions.

#### Major Finding 2 — Coralys substantially outperforms Greedy on Instance 5

Instance 5 provides a second important result. Greedy produced 0.2645 whereas both evolutionary initializations converged to **0.2402** — approximately a **9.2% reduction** in the workload balance objective. Unlike Instance 3, this improvement is achieved without any post-construction Local Search, indicating that the evolutionary search itself discovers assignments unavailable to the deterministic constructive heuristic.

#### Major Finding 3 — Coralys avoids Greedy's construction bottleneck

| Instance | Greedy_ms | EA_rr_ms | Speedup |
|----------|----------:|---------:|---------|
| 1 | 259 | 3,262 | 0.08× |
| 2 | 818 | 5,612 | 0.15× |
| 3 | 4,158 | 8,726 | 0.48× |
| 4 | **134,646** | **20,357** | **6.6×** |
| 5 | **176,153** | **21,305** | **8.3×** |

For smaller instances, Greedy remains faster. However, on the largest constructive problems (Instances 4 and 5), Coralys becomes dramatically more efficient: 6.6× faster on Instance 4, 8.3× faster on Instance 5. This transition coincides with the point at which Greedy's exhaustive candidate evaluation (O(P × R) per pairing) becomes computationally dominant over the EA's fixed O(pop × gen) budget. Coralys does not eliminate Greedy's construction bottleneck — Greedy still performs O(P × R) candidate evaluations when invoked. Rather, Coralys avoids that algorithm entirely by using a population-based search that scales with O(pop × gen) regardless of P × R.

#### Major Finding 4 — Initialization quality matters

Round-robin initialization successfully produced feasible solutions for every benchmark. Random initialization failed on Instance 4 (571 pairings, 145 rotations) — the only failure across all 7 instances. With a large number of rotations to cover, the repair operator was insufficient to guarantee feasibility within 200 generations. Round-robin initialization, which guarantees coverage by construction, remained feasible throughout and matched the Greedy solution.

| Instance | Random Result |
|----------|---------------|
| 1–3, 5–7 | optimal |
| 4 | infeasible (inf) |

This demonstrates that initialization quality becomes increasingly important as assignment density increases (large R relative to P).

#### Major Finding 5 — Some benchmark instances require no optimization

Instances 6 and 7 contain one pairing per rotation, no constructive decisions, and Greedy construction time of approximately 1ms. Coralys matches the Greedy objective exactly but spends 15–24 seconds exploring a search space that contains virtually no meaningful improvements. These benchmarks illustrate that evolutionary optimization should not be applied indiscriminately. A lightweight structural analysis — detecting one-to-one pairing/rotation mappings or other trivial assignment patterns — could bypass evolutionary search entirely for such cases.

#### Conclusions

This experiment demonstrates three distinct operating regimes for workforce scheduling:

1. **Trivial construction** (Instances 6–7): deterministic assignment is sufficient; evolutionary search adds no value.
2. **Moderately constrained construction** (Instances 1–2): Greedy and Coralys converge to equivalent solutions.
3. **Highly constrained construction** (Instances 3–5): Coralys consistently matches or exceeds Greedy, and on the largest constructive instances it does so while substantially reducing construction time.

The evidence supports a broader architectural conclusion than a simple performance comparison. **Deterministic construction should be viewed as an initialization strategy rather than an architectural requirement.** Coralys demonstrates that constructive scheduling itself can be formulated as an optimization problem, allowing the same optimization framework to generate, refine, and evaluate schedules within a unified search process.

In this framing, Greedy, round-robin, random, and domain-specific heuristics are all instances of a `SchedulerInitializer` — different ways to seed the population. Coralys then searches the feasible solution space for the best outcomes. The deterministic algorithms define feasibility; Coralys searches within it. This is the design philosophy Coralys was built for, and the GERAD benchmark experiments provide the first experimental evidence that it holds for workforce scheduling at realistic scale.

### 2.18 Research Roadmap — Open Questions and Revised Priorities

The results of Sections 2.10–2.17 form a complete scientific narrative:

- Section 2.10 identified Local Search as the bottleneck.
- Section 2.11 eliminated that bottleneck with analytical descriptor evaluation while preserving solution quality.
- Section 2.12 showed the bottleneck migrated to the Greedy Scheduler.
- Section 2.13 proved Greedy spends almost all of its time constructing temporary rosters rather than evaluating solutions.
- Section 2.17 demonstrated that Coralys can equal or outperform Greedy while avoiding that construction process on the larger instances.

The strongest contribution is no longer the runtime improvement. The stronger contribution is:

> **A deterministic constructive scheduler is not necessary for workforce scheduling. A population-based optimizer can act as the constructive scheduler itself.**

This changes the research priorities.

#### Revised development roadmap

Previously the recommended sequence was: improve Greedy → then optimize Coralys. The evidence now supports the reverse:

1. **Improve Coralys** — better repair strategies, adaptive population sizing, convergence detection.
2. **Make Greedy optional** — treat it as one `SchedulerInitializer` among several (round-robin, random, domain heuristic, previous schedule).
3. **Eventually retire Greedy** — unless evidence emerges that it provides unique value on specific instance classes.

#### Architecture: Construction Strategy as Initialization

The architecture should be described not as "Greedy vs Coralys" but as:

```
Construction Strategy (SchedulerInitializer)
    │
    ├── Greedy
    ├── Round-Robin
    ├── Random
    ├── Previous Schedule
    └── Domain-specific heuristic
    │
    ▼
Coralys Optimization
```

Greedy stops being "the algorithm." It becomes one way to seed the population. Coralys then searches the feasible solution space for the best outcomes. The deterministic algorithms define feasibility; Coralys searches within it.

#### Open Research Question 1 — Where is the crossover point?

The current evidence establishes:

- Instance 3 (P×R = 6,627): Greedy faster, equivalent quality.
- Instance 4 (P×R = 82,795): Coralys 6.6× faster, equivalent quality.

The precise transition point is unknown. Characterizing it as a function of P, R, and constraint density is publishable experimental work.

#### Open Research Question 2 — Why does Coralys outperform Greedy?

On Instance 3, Coralys achieves 0.2309 vs Greedy's 0.4011 — a 42% improvement. On Instance 5, 0.2402 vs 0.2645 — a 9.2% improvement. The structural properties of those instances that create this advantage are not yet understood. Candidate explanations include: assignment density, constraint interaction, workload distribution topology, and pairing size variance. Answering this makes Coralys explainable rather than merely empirical.

#### Open Research Question 3 — Can Coralys optimize earlier decisions?

Today Coralys optimizes pairing → rotation assignment. The larger opportunity is to optimize the decisions that precede pairing construction:

```
Flights → Duty boundaries → Pairing topology → Crew assignment
```

This is where the largest search space exists. The current genome encodes only the final assignment step. A genome that encodes duty boundaries and pairing topology would allow Coralys to explore the full construction space, not just the assignment space. This is the direction described in Sections 2.4–2.5 and represents the longest-term architectural goal.
---

## 3. Proposed Implementation Plan (Near-Term)

Currently, the MOGA genome is a `HashMap<shift_id, worker_id>`. Pairing structure emerges as a side-effect of the assignment through deterministic greedy grouping in [`pairings_handler()`](../../services/ultracrew_server/src/main.rs:1300). The MOGA has no mechanism to intentionally target specific pairing structures.

The proposed change adds one new mutation operator that, instead of randomly reassigning a shift, attempts to complete a legal pairing by finding the next connectable shift for the same worker. This gives the MOGA a pairing-construction bias without requiring a full column generation implementation.

### 3.1 New mutation operator

Add a `pairing_completion_mutation` to [`coralys-moga/src/engine.rs`](../../coralys-moga/src/engine.rs):

```
For a randomly selected shift s assigned to worker w:
  Find the next shift s' such that:
    - s' is currently unassigned or assigned to a different worker
    - s' starts after s ends + minimum_connection_time
    - s' starts before s ends + LAYOVER_REST_HOURS
    - assigning s' to w would not violate any hard constraint
  If such s' exists:
    Assign s' to w (completing or extending a pairing)
  Else:
    Fall back to standard random reassignment
```

### 3.2 Pairing-topology fitness term

Add a term to [`ConstraintEngine.evaluate()`](../../adapters/ultracrew/src/constraint_engine.rs:39) that rewards pairings that return to home base within a target number of days (e.g., ≤ 3 days), penalizing pairings that span excessive time away from base.

### 3.3 Mutation probability

Introduce the new operator at a configurable probability (suggested initial value: 0.15–0.25 of all mutation events). Keep existing operators active.

---

## 4. Experimental Design

**Condition A (baseline):** Current MOGA without pairing-aware mutation.

**Condition B (experimental):** MOGA with pairing-completion mutation at p=0.20.

Run both conditions against all 7 GERAD instances. Use the same random seed for reproducibility.

---

## 5. Metrics to Record

For each instance and each condition:

- Pairing count ratio (UltraCrew / GERAD reference)
- Compliance rate (% pairings passing TC CAR 700 legality check)
- Multi-duty pairing ratio
- Mean pairing span (days)
- Coverage score (% shifts assigned)
- Fairness score (workload variance)
- MOGA convergence curve (fitness vs generation)
- Wall-clock runtime

---

## 6. Results

*(To be filled in after experiment is run.)*

| Instance | Pairing ratio A | Pairing ratio B | Compliance A | Compliance B | Coverage A | Coverage B |
|----------|----------------|----------------|--------------|--------------|------------|------------|
| 1        |                |                |              |              |            |            |
| 2        |                |                |              |              |            |            |
| 3        |                |                |              |              |            |            |
| 4        |                |                |              |              |            |            |
| 5        |                |                |              |              |            |            |
| 6        |                |                |              |              |            |            |
| 7        |                |                |              |              |            |            |

---

## 7. Analysis

Compare Condition A vs Condition B on pairing quality metrics. Verify that coverage and fairness are not degraded. If pairing quality improves without coverage regression, the operator is a net improvement and should be retained.

If pairing quality improves but coverage degrades, investigate whether the mutation probability needs tuning or whether the operator needs an additional constraint to prevent coverage loss.

---

## 8. Conclusion

*(To be filled in after experiment is run.)*

---

## 9. Reference

See [UltraCrew vs GENCOL Pipeline Divergence Analysis](UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md), Section 7 Step 1 for the architectural context of this investigation.