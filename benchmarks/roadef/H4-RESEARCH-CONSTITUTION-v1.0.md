# Horizon 4 Research Constitution — v1.0

**Status:** ACTIVE
**Issued:** 2026-07-10
**Supersedes:** N/A
**Baseline:** BASELINE-v1.0.json (M19.5, frozen 2026-07-10)

---

## Programme Objective

> **Increase the optimization capability of Coralys while preserving architectural
> generality and maintaining measurable improvement against the frozen M19.5 baseline.**

Horizon 3 answered: *Can Coralys solve new optimization domains correctly?*
Horizon 4 answers: *How do we make Coralys competitive?*

Every research priority in Horizon 4 must contribute to the programme objective.
No feature is accepted on the basis that it sounds good. Evidence is required.

---

## Governing Principles

### P-001 — M19.5 baseline is immutable

BASELINE-v1.0.json, BASELINE-v1.0-schema.json,
campaign_engine_v1.0_verify.json, and M19-FOUNDATION-REPORT-v1.0.md
shall not be modified. Corrections are issued through versioned successors
(e.g., M19-FOUNDATION-REPORT-v1.1.md) or errata documents.

### P-002 — Every RP begins with a written hypothesis

Before implementation begins, the research priority must state:

- What is the observed bottleneck or limitation?
- What change is proposed?
- What measurable outcome is predicted?
- What is the success criterion?

### P-003 — No optimization is accepted without benchmark evidence

Every RP must produce a delta report comparing the experiment against the
frozen baseline. The delta report must include: objective quality, feasibility
rate, runtime, and ms/gen. Improvements that cannot be measured are not accepted.

### P-004 — Generic improvements belong in coralys-moga

Capabilities that are domain-agnostic (evaluation caching, diversity
management, adaptive mutation rates, parallel evolution) belong in the
coralys-moga crate. They must not be entangled with ROADEF-specific logic.

### P-005 — Domain heuristics belong in adapters

ROADEF-specific topology heuristics, repair operators, constructive
initialization, and tuning belong in adapters/roadef (and eventually
coralys-roadef). They must not pollute the generic engine.

### P-006 — All performance comparisons reference BASELINE-v1.0.json

Unless explicitly stated otherwise, all performance claims in Horizon 4
are relative to the frozen M19.5 baseline. Claims without a stated baseline
are inadmissible.

### P-007 — Research claims require reproducible evidence

Every accepted result must be reproducible from a recorded experiment
manifest (commit, parameters, benchmark, seed policy, report). Results
that cannot be reproduced are not accepted.

### P-008 — Constitutional amendments require versioned successors

Amendments to this constitution shall be issued as versioned successors
(e.g., H4-RESEARCH-CONSTITUTION-v1.1.md). Historical versions remain
archived and are not modified in place. This keeps governance consistent
with the M19 artifact immutability policy.

---

## Research Tracks

Horizon 4 research is organized into two orthogonal tracks.

### Engineering Track

**Goal:** Increase optimization throughput — more generations per wall-clock second.

**Success metric:** Same optimizer, more generations, same wall clock.

| RP     | Description                        |
|--------|------------------------------------|
| RP-310 | Incremental evaluation             |
| RP-308 | Parallel evolution                 |
| RP-309 | Incremental routing structures     |

### Algorithmic Track

**Goal:** Increase solution quality — better objective and higher feasibility per generation.

**Success metric:** Same wall clock, better objective, higher feasibility.

| RP     | Description                        |
|--------|------------------------------------|
| RP-301 | Topology-aware initialization      |
| RP-302 | Constructive initialization        |
| RP-303 | Repair operators                   |
| RP-304 | Local search                       |
| RP-305 | Adaptive mutation                  |
| RP-306 | Diversity preservation             |
| RP-307 | Hyper-heuristics                   |

Engineering track improvements compound with algorithmic track improvements.
A faster evaluator (RP-310) makes every subsequent algorithmic improvement
more valuable. Engineering track is therefore prioritized first.

---

## Milestone Sequence and Exit Criteria

| Milestone | RP(s)      | Track       | Exit Criterion                                                                  |
|-----------|------------|-------------|---------------------------------------------------------------------------------|
| M20       | RP-310     | Engineering | >=2x ms/gen reduction on setA-10; no feasibility or objective regression        |
| M21       | RP-301     | Algorithmic | Infeasible instance count reduced from 3 to <=1 under frozen benchmark protocol |
| M22       | RP-302+303 | Algorithmic | Measurable improvement in feasibility rate or objective on >=1 target instance  |
| M23       | RP-304+305 | Algorithmic | Measurable objective improvement on >=1 evaluation-limited instance             |
| M24       | RP-306+307 | Algorithmic | Measurable improvement in search-sensitive instance stability                   |
| M25       | RP-308+309 | Engineering | >=1.5x ms/gen improvement on >=1 large instance; no regression                 |
| M26       | --         | Integration | Competition-grade coralys-roadef crate; full setA campaign submitted            |

Exit criteria are objective. A milestone is complete when its criterion is met
and verified by a delta report against BASELINE-v1.0.json.

---

## Experiment Cycle

Every RP must follow this cycle without exception:

  1. Hypothesis     -- Written statement of bottleneck, proposed change, predicted outcome
  2. Implementation -- Code change, scoped to the appropriate crate
  3. Benchmark      -- Campaign run against setA (20 instances), full measurement model
  4. Evidence       -- Delta report: Baseline -> Experiment -> Delta
  5. Decision       -- Accept (merge) or Reject (discard), with written rationale

No feature is merged without completing all five steps.

Rejected experiments are retained in the research record with their rationale.
Negative results are part of the evidence base: they prevent rediscovering
ineffective approaches and inform future hypotheses.

---

## Experiment Manifest

Every experiment must be recorded in an experiment manifest with the following fields:

- rp_id        -- Research priority identifier (e.g., RP-310)
- milestone    -- Milestone identifier (e.g., M20)
- hypothesis   -- Written hypothesis statement
- commit       -- Git commit hash of the implementation
- parameters   -- Campaign configuration (population, budget, seed policy, etc.)
- benchmark    -- Benchmark suite used (e.g., setA, 20 instances)
- seed_policy  -- Seed policy (e.g., Random unseeded)
- baseline_ref -- Reference baseline artifact (e.g., BASELINE-v1.0.json)
- report       -- Path to delta report
- decision     -- Accept or Reject
- rationale    -- Written rationale for decision

---

## Research Impact Ledger

Every accepted RP is recorded in the Research Impact Ledger. The ledger provides
an auditable history of why Coralys evolved the way it did and which research
directions delivered value.

| RP  | Milestone | Baseline     | Objective Delta | Feasibility Delta | ms/gen Delta | Decision |
|-----|-----------|--------------|-----------------|-------------------|--------------|----------|
| --  | M19.5     | (established)| --              | 17/20             | 131-116k     | --       |

This table is updated after each accepted RP. Rejected RPs are recorded in the
experiment manifest but not in this ledger.

---

## Architectural Boundary

The following boundary is a constitutional constraint, not a preference:

  coralys-moga/     Generic, domain-agnostic optimization platform
                    EvolutionEngine, Qualification, measurement model
                    Reusable across all Coralys domains

  adapters/roadef/  ROADEF-specific adapter
                    Topology heuristics, repair, constructive initialization
                    Domain-specific tuning

  coralys-roadef/   (M26) Competition-grade ROADEF optimizer
                    Topology heuristics, repair, local search, tuning
                    Does not pollute coralys-moga

Improvements with broad applicability (e.g., incremental evaluation, diversity
management) must be implemented in coralys-moga so they transfer to UltraCrew,
UltraRoute, ChronoSentiment, and future Coralys domains.

---

## Platform Maturity Model

| Stage                   | Status        | Evidence                                    |
|-------------------------|---------------|---------------------------------------------|
| Architecture            | Established   | EvolutionEngine: 0 modifications for ROADEF |
| Correctness             | Proven        | A-001: 0 violations across 3 campaigns      |
| Benchmark Governance    | Established   | BASELINE-v1.0.json + schema + reports       |
| Reproducibility         | Established   | Three independent campaign runs             |
| Measurement             | Established   | Full measurement model (ms/gen, SearchMode) |
| Competitiveness         | Beginning     | M20 is the first competitiveness milestone  |
| Publication             | Ready         | Architecture/methodology paper viable now   |
| Commercial Optimization | Future        | Post-M26                                    |

---

## Publication Opportunity

The M19 benchmark governance methodology is sufficient for a technical paper:

  A Benchmark-Driven Validation Methodology for a Generic Evolutionary
  Optimization Platform

Scope: architecture, benchmark governance, measurement model, validation
methodology. Does not require claims about state-of-the-art optimization
performance. Viable now, independent of Horizon 4 outcomes.

---

## Strategic Principle

ROADEF is a research laboratory, not the destination.

The purpose of ROADEF is to sharpen the platform capabilities under a
demanding benchmark. Success is measured not only by better ROADEF results,
but by improvements that transfer cleanly to other Coralys applications:
UltraCrew, UltraRoute, ChronoSentiment, and future optimization domains.

Every Horizon 4 improvement should be evaluated against two questions:

1. Does it improve ROADEF performance against the frozen M19.5 baseline?
2. Does it strengthen the generic Coralys platform for other domains?

Improvements that answer yes to both questions are the highest-value outcomes
of the Horizon 4 research programme.

---

*Horizon 4 Research Constitution v1.0 -- Issued 2026-07-10*
*Coralys Platform -- Horizon 4 Research Programme*
*Governing document for M20 through M26*
