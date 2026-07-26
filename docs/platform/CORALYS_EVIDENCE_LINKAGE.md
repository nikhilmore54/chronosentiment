# Coralys Platform — Evidence Linkage

**Document type:** Evidence Linkage
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Platform / Engineering / Commercial

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | New benchmark result; new pilot evidence; new customer interview; hypothesis confidence update |

**Relationship to other documents:**
- Informed by: `ULTRACREW_WORKFORCE_EVIDENCE.md` (INRC benchmark results)
- Informed by: `BENCHMARK_RESULTS.md` (benchmark results)
- Informed by: `BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` (benchmark specification)
- Informed by: `INRC_PRODUCT_EVIDENCE_PROGRAMME.md` (INRC evidence programme)
- Informed by: `EL-001_Phase1B_Evidence_Ledger.md` (ChronoSentiment commercial validation)
- Informed by: `P001_MILESTONE.md` (SunAir pilot milestone)
- Informed by: `P001_PILOT_RUNBOOK.md` (SunAir pilot runbook)
- Informed by: `docs/research/CS-R-015_Investment_Thesis.md` (investment thesis hypotheses H1–H7)
- Informs: `CORALYS_GAP_REGISTER.md` (gap prioritisation)
- Informs: Engineering sprint planning; commercial validation decisions

---

## Purpose

This document connects every piece of benchmark evidence, pilot evidence, and commercial validation evidence to the specific product capabilities it validates. It answers the question: "What do we know, and what does it prove?"

Evidence is organised by product and capability. Each evidence item records the source document, the claim it supports, and the confidence level it provides.

---

## Evidence Confidence Scale

| Level | Meaning |
|-------|---------|
| **A** | High confidence — multiple independent sources; reproducible results |
| **B** | Moderate confidence — consistent pattern; single strong source |
| **C** | Preliminary — limited sources; early-stage evidence |
| **D** | Unvalidated — secondary research only; no primary evidence |
| **X** | Contradicted — primary evidence contradicts the claim |

---

## UltraCrew Evidence

### Capability: Multi-Objective Optimisation Engine

**Claim:** The MOGA engine produces feasible, high-quality workforce schedules across multiple competing objectives simultaneously.

| Evidence ID | Source | Evidence | Confidence |
|-------------|--------|----------|------------|
| UC-E-001 | `ULTRACREW_WORKFORCE_EVIDENCE.md` §2 | INRC Sprint01: 0 hard coverage violations, 0 understaffed shifts, 0 weekend split violations, 0 consecutive-day violations. Objective score 9247.3. Runtime 1840ms. Seed 42 reproducible. | **A** |
| UC-E-002 | `adapters/ultracrew/inrc_m22_benchmark.csv` | M22 benchmark campaign — multi-seed ablation confirming consistent performance across random seeds | **A** |
| UC-E-003 | `adapters/ultracrew/ablation_matrix_30seed.csv` | 30-seed ablation matrix — confirms optimisation engine stability across seed variation | **A** |
| UC-E-004 | `adapters/ultracrew/horizon_test_n030w8.csv` | Horizon test (30 nurses, 8 weeks) — confirms engine scales to multi-week scheduling horizons | **B** |

**Validated capability:** Multi-objective optimisation engine — **Implemented and validated**

---

### Capability: INRC2 Nurse Rostering

**Claim:** UltraCrew implements the full INRC2 problem specification and produces schedules that satisfy all hard constraints.

| Evidence ID | Source | Evidence | Confidence |
|-------------|--------|----------|------------|
| UC-E-005 | `ULTRACREW_WORKFORCE_EVIDENCE.md` §2–§3 | INRC Sprint01: all four hard constraint categories satisfied (no double-shift, mandatory rest after night, minimum staffing, contract bounds). Soft penalty 147 pts. | **A** |
| UC-E-006 | `adapters/ultracrew/inrc_alpha_sweep_52w.csv` | 52-week alpha sweep — confirms constraint satisfaction across a full annual scheduling horizon | **A** |
| UC-E-007 | `adapters/ultracrew/history_test_n030w4.csv` | History-aware scheduling test (30 nurses, 4 weeks) — confirms multi-week continuity with history tracking | **B** |
| UC-E-008 | `adapters/ultracrew/inrc_m22f1_deep_attribution.csv` | Deep attribution analysis — confirms which constraints drive soft penalty; supports constraint tuning | **B** |

**Validated capability:** INRC2 nurse rostering — **Implemented and validated**

---

### Capability: Workforce Operations Learning Loop (Ecology)

**Claim:** The ecology-aware optimisation engine accumulates operational knowledge across generations and improves solution quality over time.

| Evidence ID | Source | Evidence | Confidence |
|-------------|--------|----------|------------|
| UC-E-009 | `adapters/ultracrew/m23a_survival_results.csv` | Survival analysis — confirms that high-quality solutions persist across generations (survival curves) | **B** |
| UC-E-010 | `adapters/ultracrew/m23a_extinction_curves.csv` | Extinction curves — confirms that low-quality solutions are eliminated; ecology is functioning | **B** |
| UC-E-011 | `adapters/ultracrew/m23a3_basin_curves.csv` | Basin curves — confirms convergence to high-quality solution basins | **B** |
| UC-E-012 | `adapters/ultracrew/m23a4_optionality_results.csv` | Optionality analysis — confirms that the engine maintains solution diversity (not premature convergence) | **B** |
| UC-E-013 | `adapters/ultracrew/memory_depth_ablation_30seed.csv` | Memory depth ablation — confirms that deeper memory improves solution quality; validates the learning loop | **A** |

**Validated capability:** Ecology-aware optimisation / Learning Loop foundation — **Implemented and validated**

---

### Capability: Pipeline Observability

**Claim:** The optimisation pipeline provides full observability — evolution metrics, processor metrics, convergence tracking.

| Evidence ID | Source | Evidence | Confidence |
|-------------|--------|----------|------------|
| UC-E-014 | `adapters/ultracrew/m30_0b_passive_telemetry.csv` | Passive telemetry — confirms that the pipeline collects and records operational metrics during optimisation runs | **B** |
| UC-E-015 | `adapters/ultracrew/mechanism_audit_seed12346.csv` | Mechanism audit — confirms that the pipeline correctly attributes performance to specific operators | **B** |

**Validated capability:** Pipeline observability — **Implemented and validated**

---

### Capability: SunAir Pilot (Airline Crew Scheduling)

**Claim:** UltraCrew can be deployed in a real airline crew scheduling context and produce legally compliant crew rosters.

| Evidence ID | Source | Evidence | Confidence |
|-------------|--------|----------|------------|
| UC-E-016 | `docs/P001_MILESTONE.md` | P001 milestone — SunAir pilot milestone document; records pilot scope, success criteria, and milestone status | **C** |
| UC-E-017 | `docs/P001_PILOT_RUNBOOK.md` | SunAir pilot runbook — operational guide for the SunAir pilot; confirms pilot is planned and structured | **C** |
| UC-E-018 | `docs/sunair_pilot_guide.md` | SunAir pilot guide — customer-facing guide for the SunAir pilot | **C** |
| UC-E-019 | `docs/sunair_sales_playbook.md` | SunAir sales playbook — commercial engagement guide for the SunAir pilot | **C** |

**Validated capability:** Airline crew scheduling pilot — **Planned; evidence is preparatory (C)**

---

## ChronoSentiment Enterprise Evidence

### Commercial Validation Hypotheses (H1–H7)

The ChronoSentiment Enterprise commercial validation is governed by `EL-001_Phase1B_Evidence_Ledger.md`, which tests seven hypotheses from `CS-R-015_Investment_Thesis.md`.

| Hypothesis | Claim | Current Confidence | Evidence source |
|------------|-------|-------------------|----------------|
| H1 | Investment teams have a documented decision provenance problem | D | EL-001 (no primary evidence yet) |
| H2 | The problem is acute enough to pay for a solution | D | EL-001 (no primary evidence yet) |
| H3 | AI documentation is a distinct, valued capability | D | EL-001 (no primary evidence yet) |
| H4 | The target segment (£500M–£5B AUM) is reachable | D | EL-001 (no primary evidence yet) |
| H5 | The Decision Archive creates switching costs | D | EL-001 (no primary evidence yet) |
| H6 | Regulatory pressure creates urgency | D | EL-001 (no primary evidence yet) |
| H7 | The product can be built on the Coralys platform | D | EL-001 (no primary evidence yet) |

**Note:** All ChronoSentiment Enterprise hypotheses are at confidence D (unvalidated — secondary research only). Phase 1B commercial validation (customer interviews, expert interviews, product demonstrations) is required to upgrade these to B or A.

---

### Research Evidence (Secondary)

The following secondary research documents provide the foundation for the ChronoSentiment Enterprise hypotheses. They are not primary evidence but inform the hypothesis design.

| Evidence ID | Source | Claim supported |
|-------------|--------|----------------|
| CS-R-001 | Research programme | Investment decision documentation problem exists |
| CS-R-009 | `docs/research/CS-R-009_AI_Adoption_Investment_Management.md` | AI adoption in investment management is accelerating |
| CS-R-015 | `docs/research/CS-R-015_Investment_Thesis.md` | Commercial rationale for ChronoSentiment Enterprise |
| CS-R-015A | `docs/research/CS-R-015A_Executive_Investment_Summary.md` | Executive summary of investment thesis |

---

## ChronoSentiment Personal Evidence

### Commercial Validation

ChronoSentiment Personal is in an earlier stage than Enterprise. No structured commercial validation programme has been initiated. The primary evidence source is founder self-use (documented in `ChronoSentiment_Personal_Blueprint_v1.md` governance section).

| Evidence ID | Source | Evidence | Confidence |
|-------------|--------|----------|------------|
| CS-P-E-001 | `ChronoSentiment_Personal_Blueprint_v1.md` | Blueprint governance: "Next Review: After first 30 days of founder self-use" — confirms founder self-use is the primary validation mechanism | **C** |

**Note:** ChronoSentiment Personal requires a structured validation programme before any hypothesis can be upgraded above C.

---

## Evidence Coverage Summary

| Product | Capability | Evidence level | Gap |
|---------|------------|---------------|-----|
| UltraCrew | Multi-objective optimisation | **A** | None |
| UltraCrew | INRC2 nurse rostering | **A** | None |
| UltraCrew | Ecology / Learning Loop | **A** | Full learning loop workflow not yet validated |
| UltraCrew | Pipeline observability | **B** | None |
| UltraCrew | Airline crew scheduling | **C** | Pilot not yet completed |
| UltraCrew | Disruption recovery | **D** | No evidence; capability not yet implemented |
| UltraCrew | Operational Knowledge Graph | **D** | No evidence; capability not yet implemented |
| CS Enterprise | Decision Workspace | **D** | No evidence; capability not yet implemented |
| CS Enterprise | Investment Thesis | **D** | No evidence; capability not yet implemented |
| CS Enterprise | Committee Review | **D** | No evidence; capability not yet implemented |
| CS Enterprise | Commercial hypotheses H1–H7 | **D** | Phase 1B validation required |
| CS Personal | Research Workspace | **D** | No evidence; capability not yet implemented |
| CS Personal | Investment Thesis | **D** | No evidence; capability not yet implemented |
| CS Personal | Commercial validation | **C** | Founder self-use only |

---

## Evidence Acquisition Priorities

Based on the evidence coverage summary, the following evidence acquisition activities are highest priority:

**Priority 1 — Complete the SunAir pilot (UltraCrew)**
- Upgrade UC-E-016 through UC-E-019 from C to A
- Validates airline crew scheduling in a real operational context
- Provides the first commercial reference for UltraCrew

**Priority 2 — Initiate Phase 1B commercial validation (ChronoSentiment Enterprise)**
- Execute the EL-001 evidence acquisition protocol (customer interviews, expert interviews)
- Target: upgrade H1 and H2 from D to B within 60 days
- Kill criteria: if H1 or H2 cannot be upgraded to B after 10 interviews, reconsider product direction

**Priority 3 — Validate disruption recovery (UltraCrew)**
- Implement disruption recovery workflow (see Gap Register)
- Run structured disruption recovery tests against INRC and airline scenarios
- Target: upgrade disruption recovery evidence from D to B

**Priority 4 — Founder self-use validation (ChronoSentiment Personal)**
- Complete 30 days of structured founder self-use
- Document findings against the blueprint capabilities
- Target: upgrade CS-P-E-001 from C to B

---

*Coralys Platform Evidence Linkage v1.0 | July 2026 | Status: Baseline*
*Connects benchmark reports and pilot evidence to the capabilities they validate.*
*Review trigger: New benchmark result; new pilot evidence; new customer interview; hypothesis confidence update.*