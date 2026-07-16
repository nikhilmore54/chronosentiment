# Sprint 10 Plan — Benchmark Reproduction & Semantic Validation

**Document:** SPRINT10-PLAN-v1.0.md  
**Date:** 2026-07-16  
**Status:** PLANNED — entry conditions met  
**Predecessor:** Sprint 9 (closed at b8b2a9c2)

---

## Mission

> **Determine whether Coralys can faithfully reproduce the CVD-001 benchmark semantics and document any irreducible differences with evidence.**

This is broader than "recover HC3 semantics." The ultimate goal is faithful benchmark reproduction — HC3 is one constraint among several that require semantic validation.

---

## Non-Goals

Sprint 10 does **not** include:

- Airline pairing generation
- Duty generation
- Base continuity implementation
- Aircraft qualification
- Monthly rostering
- Airline-specific optimization improvements

These remain out of scope until benchmark semantics have been resolved or explicitly documented as research assumptions. Implementing airline features before resolving benchmark semantics would conflate benchmark reproduction with product evolution. Sprint 10 keeps these explicitly separate.

---

## Why This Sprint Comes Before Airline Feature Development

Phase 1 validated the platform. Phase 2 validates the evaluation. Without Phase 2, it is impossible to distinguish between:

- "Coralys matches the benchmark's intended evaluation"
- "Coralys implements a good scheduler that differs from the benchmark"

Those are different claims and must be kept separate. Sprint 11 can then pivot to Airline Solution Engine development with a clear, documented boundary between what was verified and what was designed.

---

## Milestone 1 — Benchmark Evidence Recovery

**Goal:** Recover every authoritative artifact for the CVD-001 benchmark.

**Tasks:**
- Obtain `README.pdf` from GERAD archive (`G1422-DataSets.zip`)
- Obtain benchmark evaluator source (Quesnel et al., Polytechnique Montréal / GERAD)
- Search GERAD technical report archives (G-2010-xx series)
- Search authors' public repositories (GitHub, institutional pages)
- Search ROADEF 2010 challenge supplementary materials
- Contact benchmark authors if necessary

**Deliverables:**
- Evidence catalogue: every artifact examined, provenance, confidence level
- Source provenance document: where each artifact was obtained
- Confidence assessment: what can be determined from available evidence

**Exit criterion:** Every obtainable benchmark artifact has been examined and catalogued.

---

## Milestone 2 — Benchmark Semantic Reconstruction

**Objective:** Reconstruct every benchmark semantic that cannot already be established from the published dataset or implementation evidence.

This is evidence-driven, not exhaustive. HC1, HC2, rest handling, and most adapter semantics are already understood from the Sprint 9 implementation. The focus is on unresolved semantics only.

**Currently unresolved:**
- HC3: workload credit constraint — hard constraint, soft penalty, objective term, or reporting metric?
- Base caps: per-base aggregate credit hour caps — enforcement mechanism?
- Credit accounting: how flight hours accumulate across a bid period

**For each unresolved rule, determine:**
- Hard constraint (feasibility)?
- Soft constraint (penalty)?
- Objective term (optimization target)?
- Reporting metric (post-hoc only)?
- Preprocessing only (not evaluated at runtime)?

**Output:** `BENCHMARK-SEMANTICS-v1.0.md`

| Rule | Current status | Evidence source | Confidence |
|---|---|---|---|
| HC1 | Hard constraint (confirmed) | Sprint 9 implementation | High |
| HC2 | Hard constraint (confirmed) | Sprint 9 implementation | High |
| HC3 | Hypothesis H1 (unconfirmed) | credit_constrains.csv (generator only) | Low |
| Rest | Hard constraint (confirmed) | Sprint 9 implementation | High |
| Base caps | Unknown | credit_constrains.csv (generator only) | Low |
| Credit accounting | Partial | creditedHours file | Medium |

**Exit criterion:** Every unresolved benchmark rule has been classified with evidence and confidence level.

---

## Milestone 3A — Benchmark Reference Reconstruction

**Goal:** Reconstruct what the benchmark evaluator would do, independent of Coralys.

**Output:** `BENCHMARK-REFERENCE-v1.0.md`

This document answers: "What would the benchmark evaluator do for a given schedule?"

It is written from the benchmark's perspective, not Coralys's. It serves as the reference against which Coralys is compared in Milestone 3B, and as a standalone artifact for future publication.

**Exit criterion:** A complete reference description of benchmark evaluation behavior, with evidence citations for every claim.

---

## Milestone 3B — Coralys Reproduction Study

**Goal:** Compare Coralys against the benchmark reference across all evaluation dimensions.

**Comparison dimensions:**
- Credit accounting: how flight hours are accumulated per worker and per base
- Deadheads: positioning flight handling
- Base constraints: per-base aggregate credit caps
- Objective values: fitness function alignment
- Runtime: optimization time comparison
- Workload distributions: per-worker hour distributions
- HC3 specifically: does Coralys HC3 match benchmark HC3?

**Output:** `REPRODUCTION-STUDY-v1.0.md`

Structure:
- For each dimension: Coralys behavior vs benchmark behavior vs gap
- Gap classification: exact match / semantic equivalent / known difference / unknown
- Confidence level for each comparison

**Exit criterion:** Every comparison dimension evaluated and documented.

---

## Milestone 4 — Architectural Decision

**Goal:** Based on the reproduction study, decide the forward path.

**Possible outcomes:**

**Option A — Coralys already matches benchmark**
- No further changes needed for benchmark reproduction
- Sprint 11 proceeds to Airline Solution Engine development

**Option B — UltraCrew needs airline capabilities**
- Specific gaps identified (e.g., base continuity, duty generation, pairings)
- These belong in the Solution Engine layer, not the platform
- Sprint 11 scoped to implement identified capabilities

**Option C — Benchmark is irreducibly ambiguous**
- Freeze Working Hypothesis H1 (or best available hypothesis)
- Document assumptions explicitly
- Label all subsequent results as hypothesis-driven
- Proceed with documented uncertainty

**Output:** `SPRINT10-DECISION-v1.0.md`

**Exit criterion:** One of Option A, B, or C is formally adopted with documented rationale.

---

## Exit Conditions

Sprint 10 is complete when all of the following are met:

- `BENCHMARK-SEMANTICS-v1.0.md` is written and committed
- `BENCHMARK-REFERENCE-v1.0.md` is written and committed
- `REPRODUCTION-STUDY-v1.0.md` is written and committed
- `SPRINT10-DECISION-v1.0.md` is written and committed
- One of Option A, B, or C is formally adopted
- Every remaining discrepancy between Coralys and the benchmark has been classified as:
  - confirmed implementation difference,
  - benchmark ambiguity, or
  - deliberate product deviation

---

## Parallel Research Stream

While Sprint 10 runs, formally begin the representation research stream.

**Not implementation — research.**

Available foundation:
- Taxonomy of scheduling representations (Strategy A through D)
- Architecture: domain-independent platform + domain-specific adapters
- Motivation: representation choice affects constraint expressibility
- Benchmark: CVD-001 as experimental substrate

Sprint 10 produces the missing experimental evidence. The combination enables a paper:

> **Scheduling Representation and Optimization: Decoupling Decision Representation from Search in Evolutionary Workforce Scheduling**

The research contribution is the separation of representation from search — not the evolutionary algorithm itself. That is the stronger claim.

---

## Sprint 10 Entry Conditions

All conditions are met:
- Sprint 9 formally closed (✅ b8b2a9c2)
- `SPRINT9-EXIT-REPORT-v1.0.md` committed (✅ 3379561f)
- No open engineering work from Sprint 9 remains (✅)

---

## Sprint Progression Context

| Sprint | Focus | Status |
|---|---|---|
| Sprint 8 | Platform stabilization | Complete |
| Sprint 9 | Industrial benchmark integration + architectural validation | Complete (b8b2a9c2) |
| Sprint 10 | Benchmark semantics recovery + faithful reproduction | Planned |
| Sprint 11 | Airline Solution Engine (airport graph, base continuity, duty generator) | Blocked on Sprint 10 |
| Sprint 12 | Pairing Generator | Blocked on Sprint 11 |
| Sprint 13 | Commercial Airline Product (UltraCrew v1.0) | Blocked on Sprint 12 |