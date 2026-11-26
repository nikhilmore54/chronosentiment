# BENCHMARK-SEMANTICS-v1.0
## CVD-001 Benchmark Semantics — Implementation Reference

**Document ID:** BENCHMARK-SEMANTICS-v1.0  
**Status:** DRAFT  
**Role:** Final translation artifact of Milestone 2. This document answers: *"What does this benchmark mean, independent of how we reconstructed it?"* It is the first document in the implementation lineage and the last document in the reconstruction lineage.  
**Mathematical baseline:** WP-M2.6 §3 Validated Reconstruction Summary (frozen at `d3e3303a`, tag `M2-FROZEN`)  
**Oracle:** [`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs) — ground truth implementation  
**Created:** 2026-07-17  

---

## 1. Benchmark Identity

**Name:** CVD-001  
**Source:** GERAD Technical Report G-2014-22 (Kasirzadeh, Saddoune, Soumis)  
**Domain:** Monthly airline crew rostering  
**Problem type:** Assign crew members to flight legs over a monthly planning horizon to minimise workload imbalance subject to contractual constraints  
**Dataset:** G1422-DataSets.zip (distributed with G-2014-22)

---

## 2. Domain Concepts

### 2.1 Crew Members

A crew member n ∈ N is an individual with:
- A contractual workload base W^min_n (minimum credited hours per month)
- A contractual workload cap W^max_n (maximum credited hours per month)
- A workload balance target t_n = (W^min_n + W^max_n) / 2

### 2.2 Flight Legs

A flight leg f ∈ F is a single operated flight with:
- A credit coefficient c_f (contractual credit rate, dimensionless)
- A duration d_f (block hours)
- A credited workload contribution c_f · d_f (credited hours)

### 2.3 Duties

A duty t ∈ T is a sequence of flight legs operated by a single crew member on a single calendar day. A duty has:
- A set of constituent flight legs
- A duty credit c_t = Σ_{f∈t} c_f · d_f (sum of credited workload contributions)

### 2.4 Monthly Credited Workload

The monthly credited workload W_n for crew member n is the sum of duty credits for all duties assigned to n over the planning month:

> W_n = Σ_{t∈T_n} c_t · x_{n,t,k}

where T_n is the set of duties assigned to n and x_{n,t,k} is the assignment indicator (1 if crew member n is assigned to duty t in qualification class k, 0 otherwise).

**Evidence status:** [Hypothesized | Moderate] — aggregation structure [Recovered | High]; qualification indexing [Hypothesized | Moderate]

### 2.5 Assignments and Schedules

An assignment is a (crew member, duty) pair. A schedule is a complete set of assignments covering all flight legs for the planning month.

---

## 3. Objective Semantics

The benchmark minimises total workload imbalance across all crew members:

> minimise Z = Σ_{n∈N} [α · cost_n + β · Δ_n]

where:
- Δ_n = |W_n − t_n| is the workload deviation for crew member n
- t_n = (W^min_n + W^max_n) / 2 is the workload balance target
- cost_n is an additional cost term (pairing cost, deadhead cost, or similar — exact definition not recovered)
- α, β are weighting coefficients (not recovered from public artifacts)

**Workload deviation:** Δ_n penalises both over-target and under-target deviations symmetrically. A crew member working exactly at target contributes zero to the objective.

**Evidence status:** Structural form [Recovered | High]; Δ_n [Hypothesized | Moderate]; weighting coefficients [Hypothesized | Low]; cost_n [Hypothesized | Moderate]

**Implementation guidance:** When α and β are unknown, a reasonable starting point is α = 0, β = 1 (pure workload balance). This is the minimum-assumption implementation consistent with the evidence.

---

## 4. Constraint Semantics

### 4.1 HC3 — Workload Cap (Hard Constraint)

Every crew member's monthly credited workload must not exceed their contractual cap:

> HC3: W_n ≤ W^max_n for all n ∈ N

**Enforcement:** Hard constraint. A schedule that violates HC3 for any crew member is infeasible. Violations are counted as hard constraint violations, not penalised in the objective.

**Evidence status:** Constraint structure [Recovered | High]; complete constraint [Hypothesized | Moderate] (inherits from W_n)

### 4.2 Base Enforcement (Soft)

The contractual base W^min_n is enforced softly through the workload balance objective. Under-base deviation (W_n < t_n) is penalised via Δ_n = |W_n − t_n|. There is no separate hard constraint enforcing W_n ≥ W^min_n.

**Evidence status:** [Hypothesized | Moderate] — consistent with E1 (no hard base violation counter visible) and ER-009 (Montréal model soft base pattern)

### 4.3 Other Hard Constraints

HC1 and HC2 are referenced in the benchmark but their mathematical definitions have not been recovered from public artifacts. The oracle ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) is the authoritative implementation for all hard constraints.

---

## 5. Enforcement Semantics

### 5.1 Aggregation level

All constraint checks and objective computations operate at the **monthly level** — per crew member, over the full planning month. There is no duty-level cap enforcement visible in the oracle.

### 5.2 Enforcement pattern

| Quantity | Enforcement | Mechanism |
|----------|-------------|-----------|
| W_n ≤ W^max_n | Hard | HC3 violation counter |
| W_n ≥ W^min_n | Soft | Δ_n penalty in objective |
| W_n near t_n | Soft | Δ_n penalty in objective |

### 5.3 Accumulation model

Monthly workload is accumulated as a running sum of duty credits. No duty-level cap is applied before monthly aggregation (sum-then-cap preferred; cap-then-sum not excluded).

**Evidence status:** [Hypothesized | Moderate]

---

## 6. Known Uncertainties

The following aspects of the benchmark semantics are not definitively recovered from public artifacts. Implementations should treat these as preferred reconstructions, not confirmed specifications.

| Aspect | Preferred reconstruction | Confidence | Recoverability |
|--------|--------------------------|------------|----------------|
| Qualification indexing in W_n | Single qualification class per assignment | Moderate | Moderate |
| Weighting coefficients α, β | Not recovered; use α=0, β=1 as default | Low | Low |
| cost_n definition | Pairing/deadhead cost; exact form unknown | Moderate | Moderate |
| HC3 exact identity | W_n ≤ W^max_n (HC3-A preferred) | Moderate | Low |
| Base enforcement mechanism | Soft via Δ_n (B2 preferred) | Moderate | Low |
| Duty-level cap | Not present (sum-then-cap preferred) | Moderate | Low |
| HC1, HC2 definitions | Not recovered | — | Low |

**Governing principle:** Unknown benchmark behaviour shall remain explicitly documented as unknown rather than replaced by speculative implementations. The oracle ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) is the ground truth for any aspect not covered here.

---

## 7. Implementation Guidance

### Rule 1 — Use the oracle as ground truth
For any aspect of benchmark semantics not specified here, defer to [`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs). Do not invent semantics.

### Rule 2 — Implement HC3 as a hard filter
Any candidate schedule with W_n > W^max_n for any n must be rejected before objective evaluation. HC3 is not a penalty — it is a feasibility gate.

### Rule 3 — Implement Δ_n as the primary objective term for the benchmark reference evaluator
For the benchmark reference implementation, when α and β are unknown, implement Z = Σ_n Δ_n (pure workload balance) as the minimum-assumption reference evaluator. This scoping is intentional: the Coralys scheduling engine will optimise many additional objectives beyond workload balance. The benchmark reference evaluator is not a constraint on Coralys' objective model.

### Rule 4 — Accumulate workload at the monthly level
Sum duty credits over the full month before comparing against W^max_n. Do not apply intermediate caps at the duty level unless evidence emerges to support it.

### Rule 5 — Preserve uncertainty in the implementation
Do not hardcode α = 1, β = 1 or any other specific weighting without evidence. Use configurable parameters so that the implementation can be updated when better evidence is available.

### Rule 6 — Validate against the oracle
Any Coralys implementation of the CVD-001 evaluator must produce identical hard constraint violation counts and objective values to [`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs) on all test instances.

---

## 8. Relationship to Reconstruction Documents

This document is a translation of the Milestone 2 reconstruction. It does not replace the work packages — those remain the authoritative evidence trail. The relationship is:

| Layer | Documents | Purpose |
|-------|-----------|---------|
| Evidence | Sprint 10 artifacts (S0–WP3) | What we found |
| Reconstruction | WP-M2.1 through WP-M2.6 | How we interpreted it |
| **Semantics** | **This document** | **What it means for implementation** |
| Implementation | `engine/` | How we build it |
| Validation | Oracle comparison | Whether we built it correctly |

**Successor document:** `BENCHMARK-REFERENCE-SPECIFICATION-v1.0` — this semantics document defines *what* the benchmark means; the reference specification defines *how* those semantics are realised as software components, interfaces, algorithms, and validation procedures.

**Coralys is not defined by this benchmark.** CVD-001 is one benchmark used to validate Coralys. The Coralys scheduling engine is expected to support additional objectives, constraints, and operational models beyond those represented in CVD-001 — including disruption recovery, fatigue-aware scheduling, reserve crew optimisation, and multi-airline deployment. This document specifies the CVD-001 benchmark semantics only. It does not constrain the Coralys product architecture.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial BENCHMARK-SEMANTICS — translation of WP-M2.6 §3 validated reconstruction summary into implementation-oriented reference; oracle reference established; 6 implementation rules; known uncertainties table |