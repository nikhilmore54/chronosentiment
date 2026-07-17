# Sprint 10 Closure Report

**Document:** SPRINT10-CLOSURE-REPORT-v1.0.md  
**Date:** 2026-07-17  
**Status:** FINAL  
**Branch:** governance-hardening  
**Final commit:** ea6cc00b

---

## Sprint 10 Completion Statement

**Sprint 10 — Benchmark Evidence Acquisition and Semantic Reconstruction**

**Status: COMPLETE**

Sprint 10 reconstructed the benchmark's scientific meaning without claiming mathematical knowledge that was not supported by evidence.

---

## Objectives

Sprint 10 was initiated to answer one question:

> Can we establish what the CVD-001 benchmark actually measures, and can we determine whether Coralys reproduces those semantics?

The sprint was governed by the Research Integrity Principle:

> No implementation changes until benchmark semantics are established from evidence.

---

## Deliverables

| Artifact | Status | Commit |
|---|---|---|
| [`SPRINT10-M1-EVIDENCE-ACQUISITION.md`](SPRINT10-M1-EVIDENCE-ACQUISITION.md) | Frozen | `0fd97d23` |
| [`SPRINT10-PLAN-v1.1.md`](SPRINT10-PLAN-v1.1.md) | Frozen | `b995e59b` |
| [`CVD-001-MILESTONE4-EVALUATION-v1.0.md`](CVD-001-MILESTONE4-EVALUATION-v1.0.md) | Frozen | `6a83fc2f` |
| [`BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md`](BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md) | Frozen | `ea6cc00b` |
| [`SPRINT10-CLOSURE-REPORT-v1.0.md`](SPRINT10-CLOSURE-REPORT-v1.0.md) | This document | — |

---

## Evidence Acquired

### Established Facts (E2 — peer-reviewed)

| Fact | Evidence |
|---|---|
| CVD-001 is distributed by GERAD Technical Report G-2014-22 | F17 |
| CVD-001 belongs to the monthly crew rostering family | ER-005 |
| Credited hours are contractual paid workload used to balance monthly bidlines | ER-006 |
| No publicly distributed evaluator has been located after exhaustive search | F18 |
| The benchmark objective emphasizes legality, contractual feasibility, and workload equity | ER-008 |
| The CVD-001 dataset is consistent with the standard Montréal monthly crew rostering resource model | ER-009 |

### Reconstructed Semantics (E3/E4 — generator and dataset)

| Semantic | Evidence |
|---|---|
| creditedHours file format reconstructed from parser | F12 |
| Duty boundary: new calendar day in flight number chars 4–5 | F11 |
| Deadheads (TDH prefix) excluded from duty counting | F10 |
| Briefing/debriefing: generator increments accumulator by 1 per duty | F13 |
| SC4 preference data generated from initial solution | F14 |
| Base credit caps generated with 3% slack from reference solution | F5 |
| Credit accumulation semantic pipeline reconstructed | ER-007 |

### Negative Evidence (E2 — confirmed absence)

| Finding | Evidence |
|---|---|
| CVD-001 is not a ROADEF challenge benchmark | F18 (S2) |
| No evaluator found in GERAD archive, institutional repositories, or general web | F18 (S1–S4) |
| GENCOL is proprietary and not publicly distributed | F16 |

---

## Evidence Not Recovered

| Item | Recoverability | Reason |
|---|---|---|
| Exact credited workload accumulation equation | Low | Not published in any located source |
| HC3 mathematical definition | Low | Not defined in any located paper |
| Objective aggregation function | Low | Not published; likely embedded in GENCOL |
| Evaluator source code | Very Low | GENCOL proprietary; no public distribution found |

These remain explicitly documented as bounded unknowns. They have not been replaced by unsupported assumptions.

---

## Scientific Contributions

Sprint 10 produced the following contributions beyond the immediate engineering objective:

1. **Evidence Hierarchy applied to benchmark reconstruction** — a six-level evidence classification (E1–E6) applied systematically to every claim about the benchmark.

2. **Two-axis knowledge model** — separating Semantic Understanding from Mathematical Reconstruction, making explicit that Sprint 10 substantially completed semantic reconstruction while leaving mathematical reconstruction partially open.

3. **Benchmark Reconstruction Principle** — a governance principle applicable beyond CVD-001: Coralys shall reproduce benchmark semantics only when supported by sufficient evidence. Unknown benchmark behavior shall remain explicitly documented as unknown rather than replaced by speculative implementations.

4. **Scientific Stopping Rule** — a defensible criterion for ending evidence acquisition: Sprint 10 stops when public evidence has been exhausted and remaining unknowns are explicitly documented. Author correspondence (S5) is optional and does not block closure.

5. **Benchmark Knowledge Matrix v1.0** — a frozen, versioned reference document recording the state of knowledge for every benchmark concept, with recoverability assessment and configuration control.

6. **Milestone 4 scientific reconciliation** — the engineering evaluation was upgraded to a research-grade report distinguishing implementation findings, empirical observations, and benchmark findings, with explicit evidence classification for every major claim.

---

## Known Unknowns

| Topic | Status | Candidates |
|---|---|---|
| HC3 semantics | Bounded Unknown | Contractual credit upper bound; bidline legality; monthly workload legality; collective agreement limit |
| Credit accumulation formula | Partially Characterized | Contractual, monthly, paid workload; exact equation not recovered |
| Objective aggregation | Partially Characterized | Legality + equity + credited hours; weighting unknown |
| Evaluator source | Not recovered (public artifacts) | May be embedded in proprietary GENCOL |

---

## Search Execution Summary

| Search | Outcome |
|---|---|
| S0 — Local artifact analysis | Complete — four .cpp generator files analyzed; F9–F15 established |
| S1 — GERAD archive | Complete — G-2014-22 provenance confirmed (F17); evaluator not found |
| S2 — ROADEF challenge pages | Complete — no ROADEF connection found |
| S3 — Institutional repositories | Complete — Quesnel GERAD page confirmed; no evaluator |
| S4 — General web search | Complete — no evaluator found; F18 established |
| S4b — Systematic Semantic Evidence Review | Complete — ER-005 through ER-009 established |
| WP3 — Mathematical recovery | Complete — ER-007/008/009 established; equations not recovered |
| S5 — Author correspondence | Optional post-sprint administrative validation |

---

## Transition to Milestone 2

Before Sprint 10, Milestone 2 would have been:

> Discover what the benchmark is.

After Sprint 10, Milestone 2 is:

> Recover the missing mathematics of an already-understood benchmark.

That is a much narrower, more tractable research problem.

**Milestone 2 — Mathematical Benchmark Reconstruction** opens with four research questions:

| ID | Question | Nature | Semantic Understanding | Mathematical Reconstruction |
|---|---|---|---|---|
| R1 | Recover exact credited workload equation | Mathematical Recovery | High | Not recovered |
| R2 | Recover objective aggregation and weighting | Mathematical Recovery | High | Partial |
| R3 | Recover HC3 mathematical definition | Mathematical Recovery | Partial (Bounded Unknown) | Not recovered |
| R4 | Validate base-cap enforcement semantics | Semantic Validation | High | Partial |

---

## Research Maturity Assessment

| Area | Completion |
|---|---|
| Dataset provenance | 100% |
| Scientific lineage | 100% |
| Planning semantics | 100% |
| Resource semantics | 95% |
| Workload semantics | 95% |
| Objective characterization | 85% |
| Mathematical reconstruction | 55–60% |
| Evaluator recovery | ~10% |

**Overall benchmark reconstruction: approximately 90% complete.**

The overall completion estimate reflects the fact that semantic reconstruction, provenance, and scientific lineage — the largest components of the benchmark reconstruction effort — are substantially complete. The remaining work is concentrated in mathematical reconstruction.

This completion estimate encompasses evidence acquisition, provenance reconstruction, semantic reconstruction, and mathematical reconstruction. It does not include Coralys implementation work or algorithmic optimization.

---

## Post-Sprint Administrative Notes

- S5 (author correspondence) may be sent at any time. If successful, record new evidence as ER-010+ in [`BENCHMARK-KNOWLEDGE-MATRIX-v1.1.md`](BENCHMARK-KNOWLEDGE-MATRIX-v1.1.md) without reopening Sprint 10.
- The Benchmark Reconstruction Principle is a candidate for elevation to the Research Operating System (ROS) as ROS-004, applicable to all future benchmark work.
- The two-axis knowledge model (Semantic Understanding / Mathematical Reconstruction) is a reusable methodology applicable to any benchmark reconstruction effort.

---

## Sprint 10 — CLOSED

**Milestone 1 (Evidence Acquisition and Semantic Reconstruction): COMPLETE**  
**Proceed to: Milestone 2 — Mathematical Benchmark Reconstruction**