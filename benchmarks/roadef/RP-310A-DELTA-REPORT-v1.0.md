# RP-310A Delta Report — Dijkstra Cache Speedup (M20 Phase 4)

**Document:** RP-310A-DELTA-REPORT-v1.0.md
**Status:** PENDING — Campaign v4 running. TBD fields will be filled in upon completion.
**Governance:** G-001 — All claims benchmarked against BASELINE-v1.0.json under same protocol.

---

## M20 Phase 4 Acceptance Checklist

### Step 1 — Campaign integrity (verify before reading performance numbers)

- [ ] JSON timestamp newer than M19 baseline (2026-07-10T17:01:23)
- [ ] All 20 instances present in results array
- [ ] No `LoadError` or `EvolutionError` quality_class entries
- [ ] Fields present: `budget_secs`, `termination_reason`, `ms_per_generation`, `search_mode`

### Step 2 — Constitutional contracts

- [ ] A-001 preserved: valid == true ⇒ obj.is_finite() for all valid instances
- [ ] E-001 preserved: evaluate_solution_cached() semantically equivalent (demonstrated Phase 2)
- [ ] G-001 preserved: same protocol, same budget formula, same population size as baseline
- [ ] Feasibility rate unchanged: valid_count == 17, invalid_count == 3

### Step 3 — Performance delta (fill Section 3 table)

For each valid instance compute: ms/gen speedup = baseline_ms_per_gen / v4_ms_per_gen

Summary statistics to compute:
- [ ] Average ms/gen speedup across 17 valid instances
- [ ] Median ms/gen speedup
- [ ] Best speedup (instance name)
- [ ] Worst speedup (instance name)
- [ ] Count of instances with speedup ≥ 2×
- [ ] Count of instances with speedup < 1× (regressions)
- [ ] Search mode transitions: EvaluationLimited → SearchLimited count

### Step 4 — Constitutional exit criterion

Binary decision:

> **M20 Complete** — RP-310A satisfies ≥2× ms/gen on setA-10 with no regressions.

or

> **M20 Continued** — RP-310A accepted as successful engineering improvement; M20 remains open pending RP-310B.

### Step 5 — Research Impact Ledger entry

| Field | Value |
|---|---|
| RP | RP-310A |
| Hypothesis | Eliminate redundant shortest-path computations via per-evaluation destination caching |
| Result | TBD (Accepted / Partially accepted) |
| Evidence | Campaign v4 (`campaign_engine_v1.0_verify.json`) |
| Key Finding | TBD |
| Next Action | TBD (RP-310B or M21) |

---

## 1. Context

RP-310 targets a ≥2× reduction in `ms/gen` on setA-10 via elimination of redundant
Dijkstra computations. This report covers RP-310A: the Dijkstra reuse work package.

**Baseline:** `campaign_engine_v1.0_verify_BASELINE_PRE_M20.json`  
(timestamp: 2026-07-10T17:01:23, 20 instances, valid=17, infeasible=3)

**Campaign v4:** `campaign_engine_v1.0_verify.json`  
(evaluator: `evaluate_solution_cached()`, E-001 validated 2/2 PASS)

---

## 2. Evaluator-Level Evidence (Phase 2 profiler, empty solution)

| Metric | setA-04 | setA-10 |
|---|---|---|
| routing_µs (baseline, timed) | 14,454 | 223,060 |
| routing_µs (cached) | 7,473 | 106,034 |
| routing speedup | **1.93×** | **2.10×** |
| cache_hit_rate | 75.5% | 85.0% |
| dijkstra_fraction of routing | 28.6% | 19.4% |
| ecmp_fraction of routing | 69.7% | 79.9% |

**Note:** Evaluator-level speedup is measured on the empty solution (worst-case
Dijkstra load, maximum cache benefit). Campaign-level speedup reflects the
optimizer's actual access pattern with non-empty solutions.

---

## 3. Campaign-Level Delta (ms/gen, all 20 instances)

| Instance | demands | links | baseline ms/gen | v4 ms/gen | speedup | Δgens |
|---|---|---|---|---|---|---|
| setA-01 | 40 | 80 | 131 | TBD | TBD | TBD |
| setA-02 | 45 | 150 | 264 | TBD | TBD | TBD |
| setA-03 | 20 | 250 | 192 | TBD | TBD | TBD |
| setA-04 | 200 | 250 | 1,742 | TBD | TBD | TBD |
| setA-05 | 100 | 396 | 1,777 | TBD | TBD | TBD |
| setA-06 | 500 | 500 | 8,306 | TBD | TBD | TBD |
| setA-07 | 800 | 500 | 15,140 | TBD | TBD | TBD |
| setA-08 | 200 | 654 | 5,900 | TBD | TBD | TBD |
| setA-09 | 200 | 750 | 5,630 | TBD | TBD | TBD |
| setA-10 | 1,000 | 966 | 27,609 | TBD | TBD | TBD |
| setA-11 | 400 | 1,000 | 15,397 | TBD | TBD | TBD |
| setA-12 | 400 | 898 | 17,913 | TBD | TBD | TBD |
| setA-13 | 2,000 | 1,000 | 102,873 | TBD | TBD | TBD |
| setA-14 | 600 | 1,108 | 30,963 | TBD | TBD | TBD |
| setA-15 | 600 | 1,250 | 29,169 | TBD | TBD | TBD |
| setA-16 | 4,800 | 1,452 | null (Infeasible) | TBD | — | — |
| setA-17 | 2,000 | 1,270 | 116,687 | TBD | TBD | TBD |
| setA-18 | 2,000 | 1,500 | 104,613 | TBD | TBD | TBD |
| setA-19 | 6,000 | 1,998 | null (Infeasible) | TBD | — | — |
| setA-20 | 6,000 | 2,000 | null (Infeasible) | TBD | — | — |

---

## 4. Feasibility Rate Comparison

| Metric | Baseline | Campaign v4 |
|---|---|---|
| valid_count | 17 | TBD |
| invalid_count | 3 | TBD |
| Feasibility preserved | — | TBD |

---

## 5. Constitutional Exit Criterion Assessment

**Target:** ≥2× reduction in ms/gen on setA-10 with no degradation in objective
quality or feasibility rate.

| Criterion | Evaluator-level | Campaign-level |
|---|---|---|
| setA-10 routing speedup | **2.10×** ✓ | TBD |
| setA-10 ms/gen speedup | N/A | TBD |
| Feasibility rate unchanged | N/A | TBD |

---

## 6. O-010 Formal Observation

> **O-010 — ECMP propagation is the dominant routing cost after Dijkstra caching.**
>
> After eliminating 75–85% of Dijkstra computations via per-time-slot caching,
> `route_ecmp()` consumes 70–80% of routing time on the representative benchmark
> instances. This identifies `route_ecmp()` as the principal remaining hotspot
> and the next engineering target (RP-310B).

---

## 7. RP-310 Work Package Status

| Work Package | Scope | Status |
|---|---|---|
| RP-310A | Dijkstra reuse via per-time-slot cache | ✅ Implemented, E-001 PASS, Campaign v4 pending |
| RP-310B | ECMP propagation optimisation | 🔲 Pending (O-010 establishes motivation) |

---

*Generated: 2026-07-11. TBD fields to be filled upon Campaign v4 completion.*