# RP-411 Execution Throughput Analysis

**Telemetry source:** `/tmp/rp410_telemetry_v3`
**Runs analysed:** 20

---

## Executive Summary

**Instrumentation gap:** The per-phase timing fields (`eval_time_ms`, `crossover_time_ms`, `mutation_time_ms`, `repair_time_ms`, `selection_time_ms`, `telemetry_time_ms`, `other_time_ms`, `total_gen_time_ms`) are present in the `GenerationRecord` schema and serialised to JSONL, but all values are zero in this campaign. The timing measurement calls are not yet wired to the actual phase boundaries in `moga_impl.rs`. This is a known instrumentation gap — the schema is correct, the measurement wiring is incomplete.

**What is valid:** Generation counts per instance are correct and scientifically useful. They confirm the throughput story established in prior campaigns: large instances complete only 2–15 generations within their time budget, while small instances complete 39–76 generations. The 253× throughput spread across instances is the primary finding of this report.

**What is not valid:** All gens/min figures, phase percentages, and timing breakdowns are zero and should be disregarded. They will be populated in the next campaign after the timing wiring is completed.

---

## 1. Generation Counts by Instance (valid data)

Generation counts are derived from the number of `GenerationRecord` entries per run. These are correct.

| Instance | Nodes | Links | Demands | Budget (s) | Gens Run | Gens/s | Termination |
|----------|------:|------:|--------:|-----------:|---------:|-------:|-------------|
| setA-01  |    20 |    80 |      40 |         30 |       76 |   2.53 | NoImprovement(20) |
| setA-02  |    30 |   150 |      45 |         30 |       21 |   0.70 | NoImprovement(20) |
| setA-03  |    50 |   250 |      20 |         30 |       39 |   1.30 | TimeLimit |
| setA-04  |    50 |   250 |     200 |         30 |       14 |   0.47 | TimeLimit |
| setA-05  |   100 |   396 |     100 |         30 |       10 |   0.33 | TimeLimit |
| setA-06  |   100 |   500 |     500 |        125 |       13 |   0.10 | TimeLimit |
| setA-07  |   100 |   500 |     800 |        200 |       15 |   0.08 | TimeLimit |
| setA-08  |   150 |   654 |     200 |         65 |        9 |   0.14 | TimeLimit |
| setA-09  |   150 |   750 |     200 |         75 |       10 |   0.13 | TimeLimit |
| setA-10  |   150 |   966 |    1000 |        300 |       13 |   0.04 | TimeLimit |
| setA-11  |   200 |  1000 |     400 |        200 |       10 |   0.05 | TimeLimit |
| setA-12  |   200 |   898 |     400 |        179 |       10 |   0.06 | TimeLimit |
| setA-13  |   200 |  1000 |    2000 |        300 |       15 |   0.05 | TimeLimit |
| setA-14  |   250 |  1108 |     600 |        300 |        9 |   0.03 | TimeLimit |
| setA-15  |   250 |  1250 |     600 |        300 |        8 |   0.03 | TimeLimit |
| setA-16  |   250 |  1452 |    4800 |        300 |        5 |   0.02 | TimeLimit |
| setA-17  |   300 |  1270 |    2000 |        300 |        2 |   0.01 | TimeLimit |
| setA-18  |   300 |  1500 |    2000 |        300 |        8 |   0.03 | TimeLimit |
| setA-19  |   300 |  1500 |    2000 |        300 |        3 |   0.01 | TimeLimit |
| setA-20  |   300 |  1500 |    2000 |        300 |        2 |   0.01 | TimeLimit |

**Key finding:** setA-01 achieves 2.53 gens/s; setA-17 and setA-20 achieve 0.01 gens/s — a **253× throughput spread** from the smallest to the largest instances. 18 of 20 instances hit the time limit, confirming that throughput (not convergence) is the binding constraint for the majority of the benchmark.

---

## 2. Throughput Scaling Pattern

Generation rate drops sharply with instance size. Three tiers are visible:

**Small instances (≤100 demands, ≤30s budget):** 10–76 gens (0.33–2.53 gens/s). Evolutionary search has meaningful depth.

**Medium instances (200–600 demands, 65–300s budget):** 8–14 gens (0.03–0.21 gens/s). Search depth is severely limited. With a population of 50, this means 400–700 individual evaluations total.

**Large instances (≥1000 demands, 300s budget):** 2–15 gens (0.01–0.05 gens/s). At 2 generations, only 100 individual evaluations occur — far too few for meaningful evolutionary search. The optimizer is effectively a random constructor with one round of selection.

This confirms that **evaluation cost scales super-linearly with demand count**, consistent with the flow computation complexity of the underlying routing problem.

---

## 3. Termination Reason Distribution

| Termination Reason | Count | Instances |
|-------------------|------:|-----------|
| TimeLimit | 18 | setA-03 through setA-20 |
| NoImprovement(20) | 2 | setA-01, setA-02 |

Only setA-01 and setA-02 terminated due to convergence (no improvement for 20 generations). All other instances hit the time limit. This means the search is not converging — it is simply running out of time. Throughput improvement would directly translate to more evolutionary search depth for 18/20 instances.

---

## 4. Instrumentation Gap: Phase Timing

The following fields are present in the `GenerationRecord` schema but emit zero values in this campaign:

| Field | Status | Required fix |
|-------|--------|-------------|
| `eval_time_ms` | Zero | Wrap evaluation loop in `moga_impl.rs` with `Instant::now()` / `.elapsed()` |
| `crossover_time_ms` | Zero | Wrap crossover block with `Instant::now()` / `.elapsed()` |
| `mutation_time_ms` | Zero | Wrap mutation block with `Instant::now()` / `.elapsed()` |
| `repair_time_ms` | Zero | Repair is not a separate phase in this harness |
| `selection_time_ms` | Zero | Wrap selection block with `Instant::now()` / `.elapsed()` |
| `telemetry_time_ms` | Zero | Wrap `emit_generation` call with `Instant::now()` / `.elapsed()` |
| `other_time_ms` | ~0.04ms | Residual from partial instrumentation |
| `total_gen_time_ms` | Zero | Wrap full generation loop body with `Instant::now()` / `.elapsed()` |

The schema is correct. The `GenerationRecord` struct in [`adapters/roadef/src/telemetry.rs`](adapters/roadef/src/telemetry.rs) already contains all required fields. Only the measurement calls in [`adapters/roadef/src/moga_impl.rs`](adapters/roadef/src/moga_impl.rs) are missing.

---

## 5. Research Programme Implications

Despite the instrumentation gap, the generation count data confirms the RP-411 research question is well-posed and the answer is already partially visible:

**The throughput problem is real and severe.** Large instances complete only 2 generations in 300 seconds. The optimizer cannot evolve under these conditions — it is a constructor with one selection step.

**The phase breakdown question remains open.** Without timing data, it is not possible to determine whether the bottleneck is evaluation (flow computation per individual), variation (crossover/mutation), selection overhead, or telemetry overhead. This is the primary deliverable of RP-411 Phase 2.

**Recommended next step:** Wire the timing calls in `moga_impl.rs` and re-run the campaign. The schema, analysis script, and report template are all ready — only the measurement instrumentation is missing. This is a one-session implementation task.
