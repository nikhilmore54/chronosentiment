# Sprint 8 — Objective Characterization
## S8-OBJECTIVE-CHARACTERIZATION-PLAN-v1.0

**Status:** OPEN  
**Phase:** II — Adoption  
**Primary Stream:** A (Coralys Platform, ~20%)  
**Lead Product Stream:** B (UltraCrew, ~60%)  
**Frozen benchmark:** UB-002-v1.0 (no changes)

---

## Sprint 8 Research Question

> How stable and predictable is Coralys' behaviour under the current SC1/SC2
> objective model?

This is a characterization sprint, not an optimization sprint. No objective
weights, operators, or ecology models change during Sprint 8.

---

## H6 — Week 2 Reproducibility

**Research question:** Is the SC1 increase observed in Week 2 a stochastic
optimization artifact or an inherent characteristic of the benchmark instance?

**Method:** 20 seeds × Week 2 only × 50 generations. Collect fitness, SC1,
SC2, HC1, HC2, HC3, Rest, runtime per seed.

**Acceptance criteria:**

- **Outcome A:** Week 2 SC1 distribution is consistent with other weeks across
  seeds → Week 2 spike is stochastic. No optimizer change required.
- **Outcome B:** Week 2 consistently produces higher SC1 across seeds →
  structural characteristic of UB-002. Investigate the benchmark, not the
  optimizer.

**Script:** `scripts/ub002_seed_stability.py`

---

## H7 — SC2 Influence Stability

**Research question:** Is SC2 influence on assignment distribution stable
across optimizer seeds?

**Method:** 20 seeds × all 4 weeks × 50 generations. Measure avg shifts/week
for HIGH, MEDIUM, LOW groups per seed.

**Acceptance criterion:** HIGH < MEDIUM < LOW holds consistently across seeds
with low variance → H7 confirmed. Otherwise investigate.

**Script:** `scripts/ub002_assignment_analysis.py`

---

## H8 — Workload Sensitivity

**Research question:** How sensitive is Coralys to different historical
workload profiles?

**Method:** Vary `historical_workloads` only (all other benchmark fields
frozen). Test profiles: balanced, light spread, heavy spread, bimodal, uniform.
Measure SC1, SC2, assignment distribution per profile.

**Script:** `scripts/ub002_workload_sensitivity.py`

---

## Sprint 8 Explicit Exclusions

The following are explicitly out of scope for Sprint 8:

- SC1 weight changes
- SC2 weight changes
- New mutation operators
- New crossover operators
- Ecology redesign
- UB-003
- Objective redesign

---

## Expected Freeze Report

`benchmarks/ultracrew/S8-OBJECTIVE-CHARACTERIZATION-v1.0.md`

Questions to answer:

- ✓ Week 2 stochastic or structural?
- ✓ SC2 influence stable across seeds?
- ✓ Assignment sensitivity quantified?
- ✓ Product evidence required for objective change?

---

## Parallel Stream B (Primary Engineering)

While Stream A performs characterization, the majority of engineering effort
runs in Stream B (UltraCrew):

- Replace synthetic datasets with realistic customer-style datasets
- Improve planner explanations and assignment reasoning
- Begin measuring Planner Acceptance Score on realistic scheduling scenarios
- Continue toward planner-quality scheduling and pilot readiness

Stream B proceeds independently of Stream A characterization results.

---

## References

- [`GOV-001-PROGRAMME-GOVERNANCE-v1.0.md`](GOV-001-PROGRAMME-GOVERNANCE-v1.0.md) — programme governance
- [`benchmarks/ultracrew/UB-002-v1.0.json`](UB-002-v1.0.json) — frozen benchmark
- [`benchmarks/ultracrew/S7-FATIGUE-NORMALIZATION-v1.0.md`](S7-FATIGUE-NORMALIZATION-v1.0.md) — Sprint 7 freeze