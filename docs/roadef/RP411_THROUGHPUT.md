# RP-411 — Evolution Throughput Characterisation

**Status:** STUB — not yet executed  
**Depends on:** RP-406C (frozen), RP-407 (frozen), RP-410A (frozen)  
**Priority:** Highest — prerequisite for all large-instance evolutionary conclusions  

---

## 1. Motivation

The v2 campaign (RP-410A telemetry) reveals that generation rate varies by approximately three orders of magnitude across the setA benchmark suite:

| Instance | Nodes | Demands | Gens completed | Runtime (s) | Gens/min |
|----------|------:|--------:|---------------:|------------:|---------:|
| setA-01 | 50 | 100 | 91 | 15 | 364 |
| setA-03 | 75 | 200 | 81 | 20 | 243 |
| setA-13 | 200 | 2000 | 6 | 321 | 1.1 |
| setA-17 | 300 | 2000 | 2 | 339 | 0.35 |
| setA-20 | 400 | 6000 | 2 | 351 | 0.34 |

At 2 generations, the evolutionary search is effectively random: the population has been initialised and evaluated once, and a single generation of selection and variation has occurred. No meaningful evolutionary dynamics can emerge. Conclusions about search behaviour drawn from 2-generation runs cannot be compared with conclusions drawn from 90-generation runs.

Until throughput is characterised and improved, evolutionary conclusions on large instances remain limited. RP-411 is therefore a prerequisite for RP-408 (native lexicographic evaluation) and RP-409 (shoulder optimisation) on large instances.

---

## 2. Research Questions

1. Where is per-generation time spent? Which component dominates: evaluation, crossover, mutation, repair, selection, sorting, or telemetry?
2. How does per-generation cost scale with instance size (nodes, links, demands, time slots)?
3. How much time is spent computing load vectors for telemetry? Is telemetry overhead significant?
4. Which data structures dominate memory traffic? Are there cache-unfriendly access patterns?
5. What is the achievable generation rate after incremental evaluation (delta load updates)?
6. Is the bottleneck in the Rust evaluation code, or in the genome representation / repair operator?

---

## 3. Proposed Instrumentation

Add per-generation timing breakdowns to the telemetry schema. Each `GenerationRecord` should include:

```
eval_time_ms       — total time spent in evaluate() across all individuals
crossover_time_ms  — total time spent in crossover operator
mutation_time_ms   — total time spent in mutation operator
repair_time_ms     — total time spent in repair operator
selection_time_ms  — time spent in selection / sorting
telemetry_time_ms  — time spent emitting telemetry records
other_time_ms      — remainder (bookkeeping, cloning, etc.)
total_gen_time_ms  — wall-clock time for the full generation
```

This produces a per-generation cost breakdown for every instance, enabling:
- Identification of the dominant cost component
- Scaling analysis (cost vs instance size)
- Before/after comparison for any optimisation

---

## 4. Proposed Analysis

### 4.1 Cost Breakdown Table

For each instance, compute the mean per-generation cost breakdown across all generations:

| Instance | Eval % | Crossover % | Mutation % | Repair % | Selection % | Telemetry % | Other % |
|----------|-------:|------------:|-----------:|---------:|------------:|------------:|--------:|
| setA-01 | — | — | — | — | — | — | — |
| ... | | | | | | | |

### 4.2 Scaling Analysis

Plot per-generation cost vs instance size metrics (nodes, links, demands) to identify the dominant scaling factor. If evaluation cost scales as O(demands × links), incremental evaluation (recomputing only affected arcs after a move) becomes the highest-ROI optimisation.

### 4.3 Incremental Evaluation Estimate

Estimate the achievable speedup from incremental evaluation:
- Current: full re-evaluation of all arcs for all time slots per individual
- Incremental: recompute only arcs affected by the changed path segments

If evaluation is 80% of per-generation cost and incremental evaluation reduces it by 10×, the net speedup is approximately 8×, yielding ~80 generations for setA-17 instead of 2.

### 4.4 Telemetry Overhead

Measure the cost of `JsonlTelemetrySink` vs `NullTelemetrySink` to determine whether telemetry instrumentation materially affects campaign results. If overhead is >5%, consider buffered or async telemetry emission.

---

## 5. Success Metrics

| Metric | Target |
|--------|--------|
| Per-generation cost breakdown produced for all 20 instances | ✓ |
| Dominant cost component identified | ✓ |
| Scaling law characterised (O(?) with instance size) | ✓ |
| Incremental evaluation speedup estimated | ✓ |
| Telemetry overhead measured | ✓ |
| Achievable generation rate after optimisation estimated | ✓ |

The primary success criterion is: **can setA-17 and setA-20 achieve ≥50 generations within the 300s budget?** If yes, evolutionary conclusions become generalisable to large instances. If no, the bottleneck requires architectural changes (e.g., parallel evaluation, SIMD, reduced genome representation).

---

## 6. Relationship to Other Research Programmes

| Programme | Dependency on RP-411 |
|-----------|---------------------|
| RP-407 (Feasibility) | Needed to determine whether setA-16/19/20 invalidity is structural or throughput-limited |
| RP-408 (Lexicographic evaluation) | Only meaningful once large instances complete ≥50 generations |
| RP-409 (Shoulder optimisation) | Requires RP-408; therefore also requires RP-411 |
| RP-412 (Construction diagnostics) | Independent of throughput; can proceed in parallel |

---

## 7. Prior Evidence

- **RP-406C** (frozen): Benchmark characterisation. Establishes what the solver produces; does not address why.
- **RP-407** (frozen): Feasibility analysis. Establishes that 5/20 instances never produce valid solutions; throughput may be masking additional failures for large instances.
- **RP-410A** (frozen): Search dynamics. Establishes operator fingerprints and zone distributions for small/medium instances; explicitly notes that conclusions are not generalisable to large instances where throughput is the binding constraint.

*End of RP-411 stub*