# Sprint 10 Plan — Benchmark Reproduction & Semantic Validation

**Document:** SPRINT10-PLAN-v1.0.md  
**Date:** 2026-07-16  
**Status:** FROZEN — Sprint-ready  
**Predecessor:** Sprint 9 (closed at b8b2a9c2)

---

## Mission

> **Determine whether Coralys can faithfully reproduce the CVD-001 benchmark semantics and document any irreducible differences with evidence.**

This is broader than "recover HC3 semantics." The ultimate goal is faithful benchmark reproduction — HC3 is one constraint among several that require semantic validation.

---

## Definition of Benchmark Reproduction

A benchmark is considered faithfully reproduced when Coralys either:

- produces evaluation results that are semantically equivalent to the benchmark under confirmed benchmark semantics, or
- documents every remaining difference with supporting evidence and explicit rationale.

Exact numerical equality is not always achievable or required. Semantic equivalence under confirmed semantics is the standard.

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

Those are different claims and must be kept separate. Sprint 11 can then pivot to Airline Solution Engine development — validated product evolution — with a clear, documented boundary between what was verified and what was designed.

---

## Evidence Hierarchy

Interpret benchmark semantics using the following precedence. When evidence sources conflict, higher authority takes precedence.

| Level | Source | Authority |
|---|---|---|
| E1 | Benchmark evaluator source code | Highest |
| E2 | Official benchmark documentation (README, technical reports) | High |
| E3 | Dataset generation code | Medium |
| E4 | Dataset artifacts (credit_constrains.csv, creditedHours, etc.) | Medium |
| E5 | Observed benchmark outputs | Low |
| E6 | Research hypotheses | Lowest |

Confidence reflects the strength of the conclusion, while Evidence Level reflects the authority of the supporting source. A high-confidence conclusion from E6 evidence is weaker than a low-confidence conclusion from E1 evidence.

Sprint 9 established that the available evidence is at level E3–E4 for HC3 and base caps. Milestone 1 seeks E1–E2 evidence.

---

## Research Integrity Principle

> Coralys will not modify platform behavior, product behavior, benchmark interpretations, or research conclusions solely to improve agreement with a benchmark unless the underlying benchmark semantics are supported by evidence at the appropriate level of the Evidence Hierarchy. When authoritative evidence is unavailable, assumptions shall be explicitly documented, assigned an evidence level and confidence, and clearly distinguished from verified benchmark behavior.

This principle prohibits "tweaking until it matches" without supporting evidence. It formalizes the methodology established during Sprint 9 and governs all benchmark reproduction work in Sprint 10.

---

## Stopping Rule

Sprint 10 will not spend unlimited effort searching for unavailable benchmark artifacts.

If Milestone 1 exhausts all reasonable public sources and author contact attempts without obtaining additional authoritative evidence, Sprint 10 proceeds under Option C (documented working hypothesis) rather than delaying product development indefinitely.

The stopping rule is triggered when all of the following have been attempted: GERAD archive search, authors' public repositories, ROADEF 2010 supplementary materials, and direct author contact.

---

## Milestone 1 — Benchmark Evidence Acquisition & Provenance

**Goal:** Recover every authoritative artifact for the CVD-001 benchmark and establish its provenance and trustworthiness.

**Tasks:**
- Obtain `README.pdf` from GERAD archive (`G1422-DataSets.zip`)
- Obtain benchmark evaluator source (Quesnel et al., Polytechnique Montréal / GERAD)
- Search GERAD technical report archives (G-2010-xx series)
- Search authors' public repositories (GitHub, institutional pages)
- Search ROADEF 2010 challenge supplementary materials
- Contact benchmark authors if necessary

**Deliverables:**
- Evidence catalogue: every artifact examined, provenance, confidence level, evidence hierarchy level (E1–E6)
- Source provenance document: where each artifact was obtained
- Confidence assessment: what can be determined from available evidence

**Exit criterion:** Every obtainable benchmark artifact has been examined and catalogued, or the stopping rule has been triggered.

---

## Milestone 2 — Benchmark Semantic Reconstruction

**Objective:** Reconstruct and classify every benchmark semantic that cannot already be established from the published dataset or implementation evidence. Reconstruction precedes classification: semantics are first inferred from the available evidence according to the Evidence Hierarchy, then classified by enforcement category with an associated evidence level and confidence.

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

| Rule | Current status | Evidence source | Evidence level | Confidence |
|---|---|---|---|---|
| HC1 | Hard constraint (confirmed) | Sprint 9 implementation | E3 | High |
| HC2 | Hard constraint (confirmed) | Sprint 9 implementation | E3 | High |
| HC3 | Hypothesis H1 (unconfirmed) | credit_constrains.csv (generator only) | E3–E4 | Low |
| Rest | Hard constraint (confirmed) | Sprint 9 implementation | E3 | High |
| Base caps | Unknown | credit_constrains.csv (generator only) | E3–E4 | Low |
| Credit accounting | Partial | creditedHours file | E4 | Medium |

**Exit criterion:** Every unresolved benchmark rule has been classified with interpretation, evidence source, evidence level, and confidence.

---

## Milestone 3A — Benchmark Reference Reconstruction

**Goal:** Reconstruct what the benchmark evaluator would do, independent of Coralys.

**Output:** `BENCHMARK-REFERENCE-v1.0.md`

This document answers: "What would the benchmark evaluator do for a given schedule?"

It is written from the benchmark's perspective, not Coralys's. It serves as the reference against which Coralys is compared in Milestone 3B, and as a standalone artifact for future publication and long-term validation. `BENCHMARK-REFERENCE-v1.0.md` is intended as a long-lived reference artifact — the canonical specification against which future Coralys releases are validated, not merely a sprint deliverable.

**Exit criterion:** A complete reference description of benchmark evaluation behavior, with evidence citations and evidence hierarchy levels for every claim.

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
- Constraint satisfaction behavior: does Coralys reject, penalize, or accept the same schedules as the benchmark under equivalent conditions?

**Output:** `REPRODUCTION-STUDY-v1.0.md`

Structure:
- For each dimension: Coralys behavior vs benchmark behavior vs gap
- Gap classification: exact match / semantic equivalent / known difference / unknown
- Confidence level and evidence level for each comparison

**Reproduction Package** (additional deliverable):
- Benchmark inputs (instance1/ dataset)
- Coralys configuration (adapter settings, scenario contract, generation parameters)
- Execution commands (cvd001_adapter.py invocation, server startup)
- Generated outputs (result JSON files)
- Comparison scripts (hc3_audit.py and any new scripts)

This package makes future verification straightforward and strengthens any publication based on the work.

**Exit criterion:** Every comparison dimension evaluated and documented; reproduction package committed.

---

## Milestone 4 — Benchmark Reproduction Decision

**Goal:** Based on the reproduction study, decide the forward path. This decision determines whether Coralys is benchmark-compatible, benchmark-equivalent, or deliberately diverging — and what the consequences are for product development.

**Decision Criteria**

| Condition | Decision |
|---|---|
| Benchmark semantics recovered and Coralys is semantically equivalent | Option A |
| Benchmark semantics recovered but Coralys differs due to missing airline capabilities | Option B |
| Benchmark semantics cannot be established with sufficient authority | Option C |

**Possible outcomes:**

**Option A — Coralys already matches benchmark**
- No further changes needed for benchmark reproduction
- Sprint 11 proceeds to Airline Solution Engine development (validated product evolution)

**Option B — Coralys reproduces the benchmark, but additional airline capabilities are required to reproduce commercial airline scheduling behavior**
- Specific gaps identified (e.g., base continuity, duty generation, pairings)
- These belong in the Solution Engine layer, not the platform
- Distinguishes benchmark fidelity from commercial product completeness
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
- Reproduction package committed
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

> **Scheduling Representation as a First-Class Design Decision: Decoupling Decision Representation from Optimization in Workforce Scheduling**

The research contribution is that representation is an independent architectural decision — not the evolutionary algorithm itself. This claim remains true regardless of whether the optimizer is evolutionary, CP, MILP, LNS, or another method. That is the stronger and more durable contribution.

The representation taxonomy and architectural contribution remain valid regardless of whether Sprint 10 concludes with benchmark equivalence (Option A), benchmark ambiguity (Option C), or identified implementation differences (Option B). Sprint 10 primarily determines the strength of the empirical evaluation supporting the paper.

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
| Sprint 10 | Benchmark semantics recovery + faithful reproduction | FROZEN — ready to execute |
| Sprint 11 | Airline Solution Engine (validated product evolution) | Blocked on Sprint 10 |
| Sprint 12 | Pairing Generator | Blocked on Sprint 11 |
| Sprint 13 | Commercial Airline Product (UltraCrew v1.0) | Blocked on Sprint 12 |