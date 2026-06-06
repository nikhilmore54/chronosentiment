# SD-005 Resolution — Feasibility Discovery Failure

**Defect ID:** SD-005  
**Status:** CLOSED  
**Sprint:** 3.8  
**Classification:** Discovery Failure  
**Evidence artifact:** `services/ultracrew_server/feasible_lineage_report.md`  
**Canonical run:** seed=61, 5000 generations, instance n050w4  

---

## Defect Statement

The Coralys MOGA archive contains 0% feasible solutions after 5000 generations on the
INRC-II n050w4 instance. The archive is populated exclusively with infeasible schedules.
No HC constraint violations are ever reduced to zero.

---

## Research State at Sprint 3.8 Entry

```
SD-003: CLOSED — Proxy/External Misalignment (Pareto domination geometry)
SD-006: CLOSED — O3 (HC_Successions proxy) is the sole driving objective

Known:
    O3 pressure causes champion eviction.
    The dominator improves O3 while worsening HC_Coverage and HC_ForbiddenSuccessions externally.
    Confirmed across 4 independent seeds (gen2, gen29, gen158, gen283).

Unknown (entering Sprint 3.8):
    Does O3-driven domination also remove feasibility from the archive?
    Are feasible solutions ever discovered and then evicted (Retention Failure)?
    Or are they never discovered at all (Discovery Failure)?
```

SD-005 was a **causal dependency investigation**, not an independent feasibility census.
The question was whether the same O3 mechanism that evicts externally-good champions
also evicts feasible solutions, producing the observed 0% feasibility.

---

## Instrumentation

Sprint 3.8 added `FeasibleLifecycle` tracking to `inrc_archive_forensics.rs`:

```rust
struct FeasibleLifecycle {
    genome_hash: u64,
    discovered_at: u64,
    admitted_at: Option<u64>,
    evicted_at: Option<u64>,
    exit_reason: Option<ExitReason>,
    dominator_hash: Option<u64>,  // killer identity for Retention Failure causal chain
    hc_total: usize,
    official_total: f64,
    proxy: Vec<f64>,
}
```

Key design decisions:
- `HashMap<u64, FeasibleLifecycle>` keyed by genome hash — deduplication prevents
  multiple records for the same genome across rediscovery cycles
- `dominator_hash` field captures the killer's identity without requiring a second run
- Census sampling every 100 generations for feasibility distribution timeline
- Frozen definitions applied before the run:
  - **First feasible** = earliest `discovered_at`
  - **Best feasible** = lowest `official_total`

---

## Evidence

### Lineage Record

| Metric | Count |
|---|---|
| Total feasible genomes discovered | 0 |
| Admitted to archive | 0 |
| Evicted from archive | 0 |
| Evicted by Dominated | 0 |
| Still in archive at gen 5000 | 0 |

### Census Timeline (selected checkpoints)

All 51 census checkpoints (gen 100 through gen 5000) show:

| feasible_count | near_feasible_5 | near_feasible_10 | infeasible_count |
|---|---|---|---|
| 0 | 0 | 0 | 100 (archive size) |

No feasible genome was ever produced. No near-feasible genome (HC_Total ≤ 5 or ≤ 10)
was ever produced. All archive members are deeply infeasible at every census point.

### Proxy Objective Distribution

The `best_proxy` for all archive members is approximately −1.00 across all objectives.
This is consistent with a genome initialization or mutation problem producing
structurally identical or near-identical genomes that cannot satisfy HC constraints.

---

## Classification

Applying the frozen classification table from `sd005_sprint38_charter.md`:

| Observation | Classification |
|---|---|
| **No feasible solution ever observed** | **Discovery Failure** |
| Feasible solution observed but never admitted | Admission Failure |
| Feasible solution admitted then dominated | Retention Failure |
| Feasible solution survives but archive remains ~0% feasible | Representation Failure |
| Feasible solutions persist in archive | SD-005 Falsified |

**SD-005 Classification: Discovery Failure**

The evaluator never returned `feasible=true` in 5000 generations. The Pareto archive
is innocent — it never received a feasible genome to retain or evict. The O3 mechanism
identified in SD-006 is not implicated in SD-005.

---

## Causal Chain Assessment

The causal chain hypothesis entering Sprint 3.8 was:

```
O3 pressure
    ↓
Proxy domination
    ↓
Champion eviction (SD-006, CLOSED)
    ↓
Feasible genome eviction (SD-005)
    ↓
0% feasible archive
```

This hypothesis is **falsified**. The archive never received a feasible genome to evict.
SD-005 and SD-006 are **independent defects** with different root causes:

- **SD-006** (CLOSED): Archive retention mechanism — O3 proxy pressure evicts externally-good champions
- **SD-005** (CLOSED): Upstream evaluation failure — the evaluator/mutator never produces feasible schedules

---

## Root Cause

The root cause of SD-005 is upstream of the archive, in one or more of:

1. **Mutation operators** — `UltraCrewMutator` may not explore the feasible region of
   the constraint landscape. The SA neighbourhood search (20 iterations, α=0.95) may
   be insufficient to escape the infeasible basin.

2. **Constraint landscape geometry** — The INRC-II n050w4 instance may have a feasibility
   boundary that is difficult to reach by random mutation from the baseline schedule.
   HC constraints (coverage, skills, forbidden successions) may form a tight feasibility
   region that requires coordinated multi-constraint satisfaction.

3. **Proxy objective alignment** — The proxy objectives (O1–O5) may not guide the search
   toward feasibility. If minimising proxy objectives does not correlate with reducing
   HC violations, the archive will converge to a region that is proxy-optimal but
   externally infeasible.

4. **Baseline initialization** — The baseline schedule produced by `generate_baseline_schedule`
   may be deeply infeasible, and the mutation operators may not have sufficient reach to
   escape this region in 5000 generations.

---

## Scientific Debt Ledger

| ID | Description | Status |
|---|---|---|
| SD-003 | Champion Retention Error: best external champion not in final archive | CLOSED (Sprint 3.6) |
| SD-005 | 0% feasible solutions in archive after 5000 generations | CLOSED (Sprint 3.8) |
| SD-006 | O3 proxy pressure causes champion eviction | CLOSED (Sprint 3.7) |
| SD-007 | Discovery Failure: mutation operators cannot reach feasible region | OPEN |

SD-007 is the next investigation target. The question is whether the mutation operators
are structurally incapable of producing feasible schedules, or whether the constraint
landscape requires a different search strategy (e.g., repair operators, constraint-guided
mutation, or feasibility-directed initialization).

---

## Commits

| Commit | Description |
|---|---|
| `19233818` | Sprint 3.8 charter (`sd005_sprint38_charter.md`) |
| `6a16df0f` | `FeasibleLifecycle` tracking + `feasible_lineage_report.md` writer |
| `2d9919ba` | `feasible_lineage_report.md` (seed=61, 5000 gens) — Discovery Failure |