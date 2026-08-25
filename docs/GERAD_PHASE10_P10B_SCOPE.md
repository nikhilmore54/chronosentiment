# GERAD Phase 10 P10-B — Repair-Scaling Characterization Scope

**Status:** PENDING AUTHORIZATION  
**Date:** 2026-08-25  
**Reference baseline:** commit 469fcefec (P10-A characterization complete)  
**Production baseline:** commit 1919018aa (Phase 9 H6-revised, generation-scoped Dijkstra cache)

---

## Motivation

P10-A established that the large-instance performance regression observed in the Phase 9 campaign is not caused by Dijkstra-cache reuse collapse. Cache reuse remains 99.8–100.0% across 200–6000 demands. The dominant cost at large instances is `repair_ms`, which scales highly non-linearly with instance size.

The causal mechanism of repair scaling is unresolved. P10-B is a measurement-only investigation to decompose `repair_ms` and identify the dominant scaling factor.

---

## Research Question

> **What mechanism causes repair workload to become disproportionately expensive at large instances, and what instance characteristics predict that transition?**

The key distinction to establish:

> Does repair become expensive because there are **more things to repair** (infeasibility count), because **each repair becomes intrinsically harder** (per-attempt cost), or because the repair algorithm performs **excessive repeated work** (algorithmic inefficiency)?

---

## P10-B Scope: Measurement Only

P10-B is **measurement and characterization only**. No changes to the repair operator's behavior. No production-path optimization. Instrumentation must be confined to observational counters that do not alter repair logic.

### Measurements to collect (per generation, per instance)

1. Number of infeasible individuals entering repair
2. Repair attempts per individual
3. Repair iterations/rounds per attempt
4. Successful vs failed repair attempts
5. Time spent per repair attempt (average and distribution)
6. Dijkstra calls attributable specifically to repair (vs improve)
7. Population feasibility state before and after repair
8. Repair work per demand (repair_ms / num_demands)
9. Repair work per infeasible individual

### Instances to measure

Same ladder as P10-A: setA-04 (200d), setA-06 (500d), setA-10 (1000d), setA-13 (2000d), setA-14 (600d), setA-16 (4800d), setA-19 (6000d).

Fixed 5 generations, seed=42, same configuration as P10-A sweep.

### Deliverable

`docs/GERAD_PHASE10_P10B_CHARACTERIZATION.md` with:

A. Repair invocation profile (infeasible count, attempts, iterations per generation)
B. Per-attempt cost scaling (time per attempt vs demand count)
C. Feasibility state transition (before/after repair, per generation)
D. Dominant mechanism identification (workload vs per-attempt cost vs repeated work)
E. Hypothesis selection for P10-C

---

## P10-C Scope (not yet authorized)

P10-C will implement one concrete repair optimization hypothesis selected from P10-B evidence. Gate protocol:

- 5/5 trajectory invariants bit-exact vs production baseline (1919018aa)
- T_net > 0 on setA-14 (medium, must not regress)
- T_net > 0 on setA-16 or setA-19 (large, must improve)
- Corroboration on second large instance

P10-C is not authorized until P10-B characterization is reviewed and a hypothesis is explicitly selected.

---

## Constraints

- P10-B: measurement only — no changes to repair operator behavior
- Production baseline remains 1919018aa
- H10-A (demand-count cache gate) is disfavored and should not be revisited without new evidence
- P10-C requires explicit authorization after P10-B evidence review
- Implementation confined to `adapters/roadef` (no coralys-core changes)

---

## Authorization Required

P10-B is not authorized until this scope document is explicitly approved. The authorization decision should confirm:

1. P10-B is measurement-only (no repair behavior changes)
2. The repair decomposition measurements listed above are within scope
3. The same 7-instance ladder is used
4. P10-C remains locked until P10-B evidence is reviewed