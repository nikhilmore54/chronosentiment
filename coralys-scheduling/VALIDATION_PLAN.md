# Coralys Scheduling — M6 Validation & Evidence Program

**Document Status:** Planning Baseline (v1.0)

**Purpose:**
Establish objective evidence that the Coralys Scheduling Platform is correct,
performant, competitive, and operationally useful before customer pilot
deployments.  M6 does **not** introduce major new scheduling functionality;
instead, it validates the capabilities implemented in Milestone 5 and
determines where further engineering is required.

---

## Guiding Principles

> **Evidence before claims.**

Implementation alone is insufficient.  Every significant capability should be
supported by measurable evidence before it is presented as a product
capability.

> **Reproducibility.**

All benchmark, scenario, and performance results produced during M6 must be
reproducible.  Every report records:

- Coralys commit SHA
- Benchmark or scenario version
- Random seed(s) where applicable
- Hardware and OS configuration
- Rust compiler version
- Configuration parameters

This ensures that any result can be regenerated and that the provenance of
every chart or table is traceable.

The validation ladder is:

```text
Implemented
      ↓
Unit Tested
      ↓
Scenario Validated
      ↓
Benchmark Validated
      ↓
Competitive Validated
      ↓
Pilot Validated
```

Only after progressing through this ladder should stronger external claims be
made.

---

## Position in the Coralys Roadmap

```text
M4  Evaluation Framework                 ✓
        │
M5  Scheduling Intelligence              ✓
        │
M6  Validation & Evidence                ◀ Next
        │
M7  UltraCrew Pilot Deployments
        │
Production Adoption
```

---

## M6.1 — Architectural Invariant Verification

**Objective:** Confirm that the architectural boundary between the legality
layer and the optimization/resilience layers is enforced continuously, not just
by convention.

**Claim to verify:** `optimization/` and `resilience/` never embed legality
logic; Layer 2 `LegalityChecker` is the sole feasibility oracle.

**Verification method:** CI-enforced dependency boundary check.

- Forbid direct imports from `legality/*` inside `optimization/*` except
  through the `LegalityChecker` API.
- Forbid legality constants (`max_duty_time`, `minimum_rest`, `qualification`,
  etc.) from appearing in optimization or resilience source files.
- Fail the build if violated.

This turns an architectural convention into a continuously enforced rule.

**Acceptance gate:** Build passes with boundary check enabled.

**Status:** Pending CI rule implementation.

---

## M6.2 — Functional Scenario Validation

**Objective:** Demonstrate that Coralys behaves correctly across representative
airline scheduling scenarios.  This validates operational behaviour rather than
implementation.

**Scope:** A comprehensive suite of synthetic planning scenarios covering:

- Normal schedule generation
- Crew shortages
- Reserve activation
- Cancelled pairings
- Crew unavailability
- Partial rosters
- Qualification failures
- Insufficient rest
- Excessive duty time
- Airport connectivity failures

Each scenario defines:

- Initial roster
- Operational event(s)
- Expected legality outcome
- Expected recovery outcome
- Expected planner summary

**Acceptance gates:**

- Results are deterministic across repeated runs.
- Legality engine produces expected violations for each scenario.
- Recovery behaviour matches specification.

**Measurements (reported, not gated):**

- Recovered pairing count per disruption scenario
- Unrecovered pairing count per disruption scenario
- Runtime per scenario

---

## M6.3 — Optimization Quality Baseline

**Objective:** Measure optimization quality rather than simply verifying
implementation.

### M6.3a — Comparative measurements

For each optimization strategy (greedy constructor, local search, future
metaheuristics), collect:

- Objective value (weighted-sum cost)
- Legality (violations after optimization)
- Runtime
- Iterations to convergence

**Acceptance gates:**

- Local search never regresses greedy on canonical scenarios (sanity check).
- Mean improvement of local search over greedy is reported.
- Runtime overhead of local search over greedy is reported.
- Convergence curves are reported.

Note: Only the first is a hard acceptance gate.  The remaining three are
measurements.  Thresholds for "good" improvement or "acceptable" runtime
should not be set until empirical data is available and, ideally, customer or
benchmark context exists.

### M6.3b — Optimization characterization

Measure the optimizer's internal behaviour:

- Convergence curves (objective vs. iteration)
- Sensitivity to initial solution (variance across multiple greedy seeds)
- Repeatability (same result given same seed)
- Runtime variance
- Neighborhood acceptance rate (fraction of moves accepted)
- Objective improvements by iteration

These measurements are valuable before any external comparison because they
characterize how the optimizer behaves.

---

## M6.4 — Scalability & Performance Validation

**Objective:** Understand computational limits.

**Problem size progression:**

| Flights | Crew |
|---------|------|
| 50      | 20   |
| 100     | 40   |
| 250     | 100  |
| 500     | 200  |
| 1000    | 400  |

**Measurements (all reported; no pre-specified targets):**

- Runtime (wall clock)
- Memory usage
- Legality evaluation time
- Incremental evaluation speed
- Optimization iterations per second

**Deliverables:**

- Scalability curves (runtime vs. problem size)
- Complexity analysis
- Identified bottlenecks

Note: Performance targets (e.g. "legal roster within N seconds") should be
derived from customer requirements, competitor performance, or published
literature — not invented in advance.  This milestone collects the data needed
to set meaningful targets.

---

## M6.5 — Robustness Validation

**Objective:** Determine whether the resilience layer produces useful
operational outcomes and whether `RobustnessScore` is a meaningful metric.

**Disruption scenarios:**

- 1 crew member unavailable
- 3 crew members unavailable simultaneously
- 20 cancelled pairings
- Reserve exhaustion (more disruptions than reserve crew)
- Cascading disruptions (disruption during recovery)

**Measurements:**

- Recovery success rate (recovered / total orphaned pairings)
- Legality preserved after recovery
- Runtime
- Planner intervention required

**On `RobustnessScore`:**

The current score is explicitly a heuristic — rest-buffer ratio and crew-slack
ratio combined with fixed weights.  This milestone should determine:

- Whether it correlates with recovery success rate.
- Whether the current weights require adjustment.
- Whether additional factors are needed.
- Whether it should be retained, recalibrated, or replaced.

The correlation coefficient will be measured and reported.  No minimum
threshold is pre-specified; the data will determine whether the metric is
predictive enough to retain.

---

## M6.6 — Competitive Validation

**Objective:** Position Coralys against existing airline scheduling systems.

**Distinction:** Feature parity and performance parity are separate questions.

**Feature matrix** (can Coralys perform the same categories of tasks?):

- Legality coverage (FAA/EASA/ICAO rules)
- Qualification modelling
- Reserve handling
- Disruption recovery
- Optimization objectives
- Interactive validation
- What-if analysis
- Incremental evaluation
- Explainable violations

**Performance matrix** (how well does it perform them?):

- Optimization quality vs. published benchmarks
- Recovery speed vs. published results
- Runtime vs. comparable systems

**Comparison targets:**

- Jeppesen Crew Management
- Lufthansa NetLine/Crew
- Sabre AirCentre Crew
- AIMS
- NAVBLUE
- RAIDO

The purpose is not to claim superiority, but to identify capability gaps and
guide the roadmap.

---

## M6.7 — Claims Review

**Objective:** Ensure that every external claim maps to evidence before any
brochure, website, or sales communication is published.

**Process:** For each claim, identify the supporting evidence artifact.  If no
evidence exists, the claim must be softened or removed.

**Example claims register:**

| Claim | Required evidence | Status |
|-------|-------------------|--------|
| Explainable legality violations | M5.2 unit tests + M6.2 scenario validation | Pending M6.2 |
| Incremental evaluation | M5.3 unit tests + M6.4 benchmarks | Pending M6.4 |
| Disruption recovery capability | M6.5 disruption study | Pending M6.5 |
| Robustness metric | M6.5 calibration study | Pending M6.5 |
| Optimization improvement | M6.3 benchmark report | Pending M6.3 |
| Scalable to 1000+ flights | M6.4 scalability curve | Pending M6.4 |
| Competitive with commercial schedulers | M6.6 feature + performance matrix | Pending M6.6 |
| Production-ready | M6.2–M6.6 + pilot readiness assessment | Pending all |

This gate keeps technical, marketing, and sales messaging aligned with
demonstrated capabilities.

---

## M6.8 — Pilot Readiness Assessment

**Objective:** Determine whether Coralys is ready for operational pilots.

**Assessment dimensions:**

Technical: stability, correctness, reproducibility.
Operational: planner workflow, explanation quality, recovery usability.
Performance: acceptable runtime, scalability, responsiveness.
Product: deployment readiness, configuration flexibility, observability,
documentation.

**Outputs:**

- Readiness scorecard
- Remaining gaps
- Recommended pilot scope

---

## Evidence Registry

Each validation artifact produced during M6 is recorded with the following
metadata to ensure discoverability, auditability, and reproducibility.

| Field | Example |
|-------|---------|
| Evidence ID | `EV-M6.3-001` |
| Related milestone | `M6.3` |
| Commit SHA | `cef81802` |
| Scenario or benchmark version | `S1 v1.0` |
| Random seed(s) | `42` |
| Hardware / compiler | `Apple M2, rustc 1.78.0` |
| Result artifact | Report, CSV, plots |
| Claims supported | Optimization improvement |

The registry is maintained as a living document alongside the validation
reports.  Every entry in the M6.7 claims register must reference at least one
Evidence Registry entry.

---

## Deliverables

M6 produces evidence, not just code.  Artifacts include:

- Scenario validation reports (M6.2)
- Optimization benchmark reports (M6.3)
- Performance and scalability reports (M6.4)
- Robustness evaluation report (M6.5)
- Competitive capability matrix (M6.6)
- Claims register (M6.7)
- Pilot readiness assessment (M6.8)

These documents become the factual basis for customer discussions, investor
presentations, and future product planning.

---

## Exit Criteria

Milestone 6 is complete when:

- Functional scenarios pass against expected outcomes (M6.2).
- Optimization quality, runtime, and convergence are measured and documented (M6.3).
- Scalability characteristics are understood across representative problem sizes (M6.4).
- Resilience behaviour has been empirically evaluated and `RobustnessScore` is
  either validated or revised (M6.5).
- Competitive strengths and gaps have been documented (M6.6).
- Every external claim maps to a documented evidence artifact (M6.7).
- A pilot readiness assessment identifies remaining work for limited customer
  deployments (M6.8).

---

## Honest current state (as of M5 completion)

| Claim | Status |
|-------|--------|
| 162 unit tests pass | ✅ Verified |
| LegalityChecker is sole feasibility oracle | ✅ Verified by architecture |
| Neighborhood operators are pure | ✅ Verified by type system |
| Greedy scheduler produces legal rosters | ⚠️ Verified on toy fixtures only |
| Local search improves on greedy | ⚠️ Not yet measured |
| RobustnessScore predicts recovery difficulty | ⚠️ Heuristic, not calibrated |
| System handles 1000-flight instances | ⚠️ Not yet measured |
| Competitive with commercial schedulers | ❌ Not yet assessed |
| Production-ready | ❌ Requires M6.2–M6.8 evidence |

---

## Recommended timeline

| Sub-milestone | Effort estimate |
|---------------|----------------|
| M6.1 Architectural CI gate | 1 day |
| M6.2 Scenario validation | 1 week |
| M6.3 Optimization baseline + characterization | 3 days |
| M6.4 Scalability benchmarks | 1 week |
| M6.5 Robustness validation | 1 week |
| M6.6 Competitive analysis | Ongoing |
| M6.7 Claims review | 1 day (per release) |
| M6.8 Pilot readiness | 3 days |
