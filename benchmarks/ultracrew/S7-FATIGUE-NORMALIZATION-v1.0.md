# Sprint 7 — Fatigue Normalization and UB-002 First Measurement
## S7-FATIGUE-NORMALIZATION-v1.0

**Frozen:** 2026-07-13  
**Branch:** governance-hardening  
**Commits:** b0bf6fa2 → 1a2c5e7 (UB-002 first run)

---

## Sprint 7 Research Question

> Can Coralys optimize workload fairness (SC1) while simultaneously minimizing
> historical fatigue (SC2 > 0) without introducing hard constraint violations?

---

## H5: Fatigue Normalization

**Hypothesis:** Representing historical fatigue as a normalized ecological state
([0,1]) yields a numerically stable and interpretable multi-objective
optimization problem in which SC2 can influence scheduling decisions without
overwhelming feasibility.

**Problem identified:** `get_historical_fatigue()` returned raw moving average
of weekly hours (~20–40h). SC2 = fatigue × hours × 2.0 contributed ~43,000
fitness units per week against a base fitness of 10,000. UB-002 would have
deeply negative fitness.

**Fix implemented** (`adapters/ultracrew/src/ecology.rs`, commit `6ec2743f`):

```rust
const REFERENCE_HOURS: f64 = 40.0;
fatigue = mean(buffer) / REFERENCE_HOURS, clamped to [0.0, 1.0]
```

**Semantics:** 0.0 = no historical load, 1.0 = worker averaged full-time hours.
Absolute, workforce-wide comparable signal (not relative to own peak).

**SC2 magnitude with normalized fatigue on UB-002:**

| Group | Prior mean | Fatigue | SC2 @ 32h/week |
|---|---|---|---|
| HIGH | 39.75h | 0.994 | 63.6 |
| MEDIUM | 32.5h | 0.813 | 52.0 |
| LOW | 20.25h | 0.506 | 32.4 |

HIGH vs LOW difference: 31.2 fitness units/worker/week — sufficient to drive
meaningful assignment decisions.

**UB-001 regression:** HC1=HC2=HC3=Rest=0 all weeks, SC2=0.0 confirmed.
SC2=0 path unaffected (historical_workloads=null → fatigue returns 0.0).

---

## H5-R0: Week 3 SC Decomposition

**Observation:** Week 3 returned fitness=9854.4 after H5 change.

**Investigation:** 5-run SC decomposition probe on Week 3.

| Run | Fitness | SC1 | SC2 | Verdict |
|---|---|---|---|---|
| 1 | 9918.4 | 81.6 | 0.0 | Optimal |
| 2 | 9854.4 | 145.6 | 0.0 | Convergence failure |
| 3 | 9790.4 | 209.6 | 0.0 | Convergence failure |
| 4 | 9854.4 | 145.6 | 0.0 | Convergence failure |
| 5 | 9918.4 | 81.6 | 0.0 | Optimal |

**Conclusion:** SC2=0.0 on all runs. Week 3 deviation is stochastic optimizer
convergence failure (SC1 > 81.6), pre-existing since Sprint 4. H5 introduced
zero semantic regression.

**Revised UB-001 regression contract:**
- SC2 = 0.0 (all weeks, all seeds) — HARD REQUIREMENT
- HC1 = HC2 = HC3 = Rest = 0 (all weeks) — HARD REQUIREMENT
- SC1 = 81.6 when optimizer converges — SOFT (seed-dependent)
- Fitness = 9918.4 when SC1 = 81.6 — DERIVED

---

## UB-002 Design

**Benchmark:** `benchmarks/ultracrew/UB-002-v1.0.json`  
**Derived from:** UB-001-v1.0  
**Single change:** `historical_workloads` non-null (SC2 > 0)

**Workload groups (skill-balanced — one worker per skill category in each group):**

| Group | Workers | Prior hours | Fatigue |
|---|---|---|---|
| HIGH | 1, 2, 3, 5 | [40, 38, 42, 39] | 0.994 |
| MEDIUM | 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 18 | [32, 33, 31, 34] | 0.813 |
| LOW | 16, 17, 19, 20 | [20, 22, 18, 21] | 0.506 |

Skill balance: each of HIGH and LOW contains exactly one worker from each skill
category (Nurse+ICU, SeniorNurse+ICU, Nurse-only, SeniorNurse-only). Fatigue
is the only manipulated variable.

---

## UB-002 First Measurement

**Run:** 4 weeks, 50 gens each, seeds 42–45, `historical_workloads` active.

**Hard constraint results:**

| Week | HC1 | HC2 | HC3 | Rest | Valid |
|---|---|---|---|---|---|
| 1 | 0 | 0 | 0 | 0 | True |
| 2 | 0 | 0 | 0 | 0 | True |
| 3 | 0 | 0 | 0 | 0 | True |
| 4 | 0 | 0 | 0 | 0 | True |

**Feasibility preserved.** Introducing SC2 did not break hard constraints.

**SC decomposition:**

| Week | Fitness | SC1 | SC2 |
|---|---|---|---|
| 1 | ~8836 | ~81.6 | ~1034 |
| 2 | ~8820 | ~145.6 | ~1034 |
| 3 | ~8836 | ~81.6 | ~1034 |
| 4 | ~8836 | ~81.6 | ~1034 |

On UB-002, the current workload design produces SC2 penalties approximately
12.7× larger than SC1 penalties. This ratio is specific to this benchmark
instance and workload design; different instances may produce different ratios.

**Assignment influence (avg shifts/week):**

| Group | Avg shifts/week |
|---|---|
| HIGH (fatigue=0.994) | 3.94 |
| MEDIUM (fatigue=0.813) | 4.10 |
| LOW (fatigue=0.506) | 4.50 |

LOW workers receive 0.56 more shifts/week than HIGH workers (~18h over 4 weeks).
SC2 influence **confirmed**: optimizer preferentially loads low-fatigue workers.

**Week 2 observation:** Week 2 produced a higher SC1 penalty (145.6 vs 81.6
on other weeks) while maintaining a similar SC2 penalty (~1034). Whether this
reflects stochastic convergence or a structural trade-off remains an open
question.

---

## Sprint 7 Research Question — Answered

> Can Coralys optimize workload fairness (SC1) while simultaneously minimizing
> historical fatigue (SC2 > 0) without introducing hard constraint violations?

**YES.** Feasibility is preserved (HC1=HC2=HC3=Rest=0 all weeks). SC2 is
active and demonstrably influences assignments. The optimizer correctly
protects historically fatigued workers from further loading.

---

## What Sprint 7 Established

- **Sprint 6:** Proved optimality under a single-objective workload-balancing model.
- **Sprint 7:** Demonstrated that Coralys can optimize a richer ecological objective while preserving feasibility.

This is a meaningful progression for the platform. Coralys now optimizes
multiple competing workforce objectives, not merely balancing hours.

---

## Open Questions for Sprint 8

Sprint 8 should focus on **characterization**, not calibration.

**Research question:** How sensitive is optimizer behaviour to the current
SC1/SC2 objective formulation?

Specific questions to answer before any objective weighting changes:

1. **Week 2 reproducibility** — Is the SC1 spike stochastic or structural?
   Run Week 2 across 20 seeds to determine.
2. **SC2 influence stability** — Is the assignment distribution (HIGH < MEDIUM < LOW)
   stable across seeds, or does it vary significantly?
3. **Workload sensitivity** — How does assignment distribution change as
   historical workload values vary?
4. **Planner preference** — Do schedules with lower SC2 and comparable
   feasibility consistently result in fewer planner edits or higher
   Planner Acceptance Score?

Only after these questions are answered should objective weighting be reconsidered.

---

## Phase II — Adoption: Stream Priorities

Sprint 7 is one piece of Stream A. The programme continues across four streams.

**Stream A — Coralys Platform (~20–25%)**
- UB-002 frozen (this sprint)
- Sprint 8: characterize SC1/SC2 behaviour (Week 2 reproducibility, SC2 stability, assignment sensitivity)
- **Benchmark creation rule:** A new benchmark instance is introduced only when an existing benchmark cannot answer a product or platform engineering question

**Stream B — UltraCrew (~50–60%)**
- Resume execution of the UltraCrew product roadmap immediately after Sprint 7 freeze
- Planner-quality scheduling with realistic datasets
- Planner validation and explanation quality
- Publishable rosters and pilot demonstration

**Stream C — Research Station (~10–15%)**
- Publish Sprint 6/7 findings
- Update methodology documentation
- Record benchmark evidence

**Stream D — ROADEF Competition (independent)**
- Continue campaign independently of product streams

The direction of information is:

```
Products
    ↓
Research questions
    ↓
Benchmark
    ↓
Platform improvement
    ↓
Products
```

UB-003 through UB-005 exist to answer product questions. Those questions should
come from product evidence, not from the benchmark roadmap itself.

---

## Platform Governance Rules

Two categories of Coralys work are distinguished permanently.

**Platform Maintenance** — preserves correctness. May happen at any time.
Examples: regression fixes, performance improvements, bug fixes, observability,
documentation.

**Platform Research** — changes optimization behaviour. May only begin after:
1. A product question exists.
2. Existing benchmarks cannot answer it.
3. A benchmark is frozen.
4. A measurable hypothesis is written.

Examples: new objectives, new operators, new ecology models, new benchmarks.

This distinction prevents optimizer changes without a product reason and keeps
the evidence-driven discipline established in H1–H5.

---

## Programme Pillars

| Pillar | Purpose |
|---|---|
| Coralys Platform | Optimization engine and scientific foundation |
| UltraCrew | Commercial workforce scheduling product |
| Research Station | Evidence, methodology, and benchmark archive |
| ROADEF Campaign | Independent external validation of platform capability |

Each pillar has a clear role. The platform supports the product. The product
generates new questions. Research answers those questions. External competitions
provide independent validation.