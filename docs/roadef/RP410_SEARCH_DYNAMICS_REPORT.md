# RP-410 — Search Dynamics

**Status:** Open — instrumentation design phase  
**Execution priority:** FIRST (before RP-407, RP-408, RP-409)  
**Rationale:** Produces evidence without changing solver behaviour  
**Exit gate:** Evidence explaining why Coralys prefers certain improvements, expressed as a
move-type distribution table per instance and per operator, with collapsed-basin vs. normal
instance comparison

---

## 1. Objective

Every accepted move is an observation. The goal is to understand what kinds of improvements
evolution naturally accepts, rather than simply whether the scalar objective improved.

This milestone answers:

> What kinds of improvements does evolution naturally accept?

instead of:

> Did the objective improve?

That is a far richer understanding of evolutionary search, and it is the prerequisite for
understanding the collapsed-basin failure mode (RP-407) and designing shoulder-aware operators
(RP-409).

---

## 2. Per-Move Telemetry Schema

For every accepted move, record the following fields:

| Field          | Type    | Description                                                        |
| -------------- | ------- | ------------------------------------------------------------------ |
| `instance`     | string  | Instance identifier (e.g. `setA-06`)                              |
| `seed`         | u64     | Random seed for the run                                            |
| `generation`   | u32     | Generation index at which the move was accepted                    |
| `operator`     | string  | Operator type that produced the move                               |
| `delta_rank1`  | f64     | Change in load at Rank 1 (Peak zone)                               |
| `delta_2_20`   | f64     | Change in cumulative load at Ranks 2–20 (Shoulder zone)            |
| `delta_21_100` | f64     | Change in cumulative load at Ranks 21–100 (Transition zone)        |
| `delta_tail`   | f64     | Change in cumulative load at Ranks 101+ (Tail zone)                |
| `accepted`     | bool    | Always `true` for accepted-move records                            |
| `move_class`   | string  | Derived classification (see §3)                                    |

For rejected moves, record the same schema with `accepted = false` and `move_class = "rejected"`.
Rejected-move sampling may be throttled (e.g. 1-in-10) to control output volume.

---

## 3. Move Classification

Classify each accepted move into one of the following types based on which zone shows the
largest improvement:

| Class                  | Condition                                                        |
| ---------------------- | ---------------------------------------------------------------- |
| `peak_improvement`     | `delta_rank1 < 0` and `delta_rank1` is the dominant improvement |
| `shoulder_improvement` | `delta_2_20 < 0` and shoulder is the dominant improvement       |
| `transition_improvement` | `delta_21_100 < 0` and transition is the dominant improvement  |
| `tail_improvement`     | `delta_tail < 0` and tail is the dominant improvement           |
| `mixed_improvement`    | Multiple zones improve by comparable amounts                     |
| `neutral`              | No zone improves by more than a threshold ε                      |

The dominance threshold and ε are to be calibrated from the first telemetry run.

---

## 4. Per-Generation Telemetry Schema

In addition to per-move records, capture the following per generation for the population best:

| Field                | Type    | Description                                                    |
| -------------------- | ------- | -------------------------------------------------------------- |
| `instance`           | string  | Instance identifier                                            |
| `seed`               | u64     | Random seed                                                    |
| `generation`         | u32     | Generation index                                               |
| `mlu`                | f64     | Maximum Link Utilisation of the best individual                |
| `sdi`                | f64     | Shoulder Dominance Index of the best individual                |
| `top20_prefix`       | [f64]   | First 20 entries of the best individual's load vector          |
| `diversity`          | f64     | Population diversity metric                                    |
| `routing_entropy`    | f64     | Shannon entropy over routing family distribution               |
| `move_class_counts`  | map     | Count of each move class accepted this generation              |

---

## 5. Output Format

Telemetry is written as newline-delimited JSON (`.jsonl`) for streaming compatibility and
post-hoc analysis with standard tools (Python, jq, DuckDB).

One file per (instance, seed) pair:

```
rp410_moves_setA-06_seed42.jsonl      — per-move records
rp410_generations_setA-06_seed42.jsonl — per-generation records
```

---

## 6. Implementation Plan

### 6.1 Phase 1 — Instrumentation hooks (no solver changes)

Add telemetry collection points at:

1. The move-acceptance decision point — record delta values before and after.
2. The end-of-generation population evaluation — record per-generation summary.

The telemetry writer must be:
- Opt-in via a feature flag or environment variable (default off in production runs).
- Zero-overhead when disabled (no allocation, no branching in the hot path).
- Buffered writes (flush every N records or every generation boundary).

### 6.2 Phase 2 — Baseline telemetry run

Run all 20 Set A instances with telemetry enabled, 5 seeds each, standard generation budget.
This produces the baseline move-type distribution for comparison in RP-407.

### 6.3 Phase 3 — Analysis

Compute the following from the telemetry:

- Move-type distribution per instance (collapsed-basin vs. normal).
- Move-type distribution per operator.
- Generation-stratified distribution (generations 1–50, 51–200, 201+).
- Routing entropy trajectory per instance.
- SDI trajectory per instance.

---

## 7. Analysis Questions

| Question | Method |
| -------- | ------ |
| What fraction of accepted moves improve the shoulder vs. the tail? | Move-class histogram per instance |
| Do collapsed-basin instances show a different move-type distribution? | Compare collapsed vs. normal histograms |
| Which operators produce shoulder improvements? | Move-class histogram per operator |
| Does the move-type distribution shift over generations? | Generation-stratified histogram |
| At what generation does routing entropy collapse in collapsed-basin instances? | Entropy trajectory plot |

---

## 8. Exit Gate Criteria

The milestone is complete when:

1. Telemetry is collected for all 20 Set A instances, ≥ 5 seeds each.
2. Move-type distribution tables are produced per instance and per operator.
3. Collapsed-basin vs. normal instance comparison is documented with statistical summary.
4. Generation-stratified analysis is complete.
5. The routing entropy collapse generation is identified for all 6 collapsed-basin instances.

---

## 9. Findings

*This section will be populated as telemetry runs complete.*

### 9.1 Move-Type Distribution (All Instances)

| Move Class             | Count | Fraction |
| ---------------------- | ----- | -------- |
| `peak_improvement`     | TBD   | TBD      |
| `shoulder_improvement` | TBD   | TBD      |
| `transition_improvement` | TBD | TBD      |
| `tail_improvement`     | TBD   | TBD      |
| `mixed_improvement`    | TBD   | TBD      |
| `neutral`              | TBD   | TBD      |

### 9.2 Collapsed Basin vs. Normal Comparison

| Move Class             | Collapsed Basin | Normal Instances | Δ |
| ---------------------- | --------------- | ---------------- | - |
| `shoulder_improvement` | TBD             | TBD              | TBD |
| `tail_improvement`     | TBD             | TBD              | TBD |

### 9.3 Routing Entropy Collapse

| Instance  | Collapse generation (median) | Std dev |
| --------- | ---------------------------- | ------- |
| setA-06   | TBD                          | TBD     |
| setA-08   | TBD                          | TBD     |
| setA-10   | TBD                          | TBD     |
| setA-13   | TBD                          | TBD     |
| setA-16   | TBD                          | TBD     |
| setA-19   | TBD                          | TBD     |

---

## 10. Data Files

*To be populated as telemetry runs complete.*

Files will be written to `docs/roadef/rp410/` to avoid cluttering the main docs directory.

---

## 11. Document History

| Date       | Event                                                                          |
| ---------- | ------------------------------------------------------------------------------ |
| 2026-08-04 | Stub created. Telemetry schema defined. Implementation plan drafted.           |