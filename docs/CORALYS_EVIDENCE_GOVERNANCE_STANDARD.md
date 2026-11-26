# Coralys Evidence Governance Standard

**Document type:** Meta-Governance Standard
**Version:** 1.0
**Status:** Active
**Date:** 2026-07-23
**Scope:** All Coralys-based products

---

## Purpose

This document defines the shared evidence governance lifecycle for all products built on the Coralys platform. It makes the methodology itself a governed asset rather than something implicitly embedded in individual product programmes.

Each product supplies its own domain-specific evidence programme. This standard defines the common structure, sequencing rules, and governance principles that all programmes must follow.

---

## The Core Principle

Every significant claim about a Coralys product must be supported by evidence that is:

- **Deterministic** — the same inputs produce the same outputs, every time.
- **Reproducible** — any evaluator can verify the results independently.
- **Falsifiable** — the claim can be disproved if the evidence does not hold.
- **Auditable** — the reasoning behind each claim is traceable and documented.

Evidence programmes are not marketing materials. They are governance artefacts.

---

## Standard Evidence Lifecycle

Every Coralys product evidence portfolio follows the same four-stage lifecycle:

```
Stage 1: Architecture Evidence (M-series)
        │
        ▼
Stage 2: Production Readiness (P-series)
        │
        ▼
Stage 3: Independent Domain Evidence (domain-specific series)
        │
        ▼
Stage 4: Domain Expansion (additional evidence programmes)
```

Each stage must be frozen before the next stage begins. This sequencing rule is non-negotiable.

---

## Stage Definitions

### Stage 1 — Architecture Evidence (M-series)

**Purpose:** Establish that the product's architecture is coherent, governed, and stable before any production demonstration.

**Standard deliverables:**
- M-x.1: Architecture overview and component map
- M-x.2: API stability report
- M-x.3: Core algorithm validation (robustness)
- M-x.4: Benchmark or qualification report
- M-x.5: Explainability and auditability design review

**Exit criteria:** All architectural claims have a corresponding validation report. No open architectural questions. All reports frozen.

**Audience:** Engineering leadership, technical due diligence reviewers.

---

### Stage 2 — Production Readiness (P-series)

**Purpose:** Demonstrate that the product is deployable, operable, and capable of delivering a complete user experience in a realistic scenario.

**Standard deliverables:**
- P-x.S1: Sales readiness (pitch deck, one-pager, demo script)
- P-x.S2: Production hardening (validation, configuration, logging, error handling, health checks, runbook)
- P-x.S3: Product completeness (planner workspace, disruption/event console, explanation engine, scenario comparison)
- P-x.S4: Platform evolution (API, SDK, integration patterns)

**Canonical demonstration entity:** A named, realistic customer or organisation (e.g. SunAir for UltraCrew, Atlas Capital for ChronoSentiment).

**Exit criteria:** All Stream 3 deliverables browser-verified. Canonical dataset deterministic. Hard violations = 0.

**Audience:** Prospective customers, sales engineers, operators.

---

### Stage 3 — Independent Domain Evidence (domain-specific series)

**Purpose:** Provide independent, technically rigorous evidence that the product performs well on a recognised or well-defined benchmark or canonical dataset — separate from the production demonstration.

**Standard deliverables:**
- E1: Canonical dataset (JSON/CSV, versioned and immutable)
- E2: Deterministic run report (same input → same output)
- E3: Domain-specific KPI report (constraint breakdown, objective decomposition, or equivalent)
- E4: Interactive evidence dashboard (HTML, self-contained, browser-verified)
- E5: Planner/analyst experience reports (domain-adapted versions of P-series UIs)
- E6: Executive evidence document (product story for customers, investors, partners)
- E7: Technical evidence document (regression history, KPI definitions, constraint analysis)
- E8: Demo guide (operator walkthrough, sign-off checklist)

**Canonical dataset requirements:**
- Fixed input universe (securities, workers, routes, etc.)
- Fixed historical or synthetic period
- Fixed data sources (public or versioned)
- Fixed evaluation protocol
- Versioned and immutable snapshot

**Exit criteria:** Deterministic replay confirmed. All E1–E8 deliverables complete and verified. Regression baseline established.

**Audience:** Technical evaluators, engineering due diligence teams, independent reviewers.

---

### Stage 4 — Domain Expansion

**Purpose:** Extend the evidence portfolio into additional operational domains using the same governance pattern.

Each expansion programme is a new Stage 3 programme for a different domain. It inherits the governance standard without redesign.

**Audience:** Domain-specific customers, partners, and evaluators.

---

## Governance Rules

| Rule | Description |
|------|-------------|
| Sequential stages | Each stage must be frozen before the next begins. |
| Evidence first | Deliverables accepted only after working demonstrations. |
| Canonical dataset | All demonstrations use the versioned canonical dataset unless otherwise noted. |
| Determinism | All runs must be reproducible from the same input. Seed or data snapshot must be fixed. |
| Regression | The first canonical run establishes the KPI baseline; all subsequent runs must meet or exceed it. |
| Domain separation | Each programme uses domain-appropriate language. No cross-domain terminology leakage. |
| Completion | Exit criteria govern completion, not percentage complete. |
| Immutability | Frozen programmes are not reopened for editorial refinement. Changes occur only when new capabilities materially change the evidence. |

---

## Product Registry

| Product | Stage 1 | Stage 2 | Stage 3 | Stage 4 |
|---------|---------|---------|---------|---------|
| UltraCrew | M-series ✅ frozen | P-001 / SunAir ✅ frozen | WS-001 / INRC ✅ frozen | RD-001, HC-001, FS-001 (future) |
| ChronoSentiment | M-series ⬜ | P-001 / Atlas Capital ⬜ | DS-001 / Canonical Dataset ⬜ | ME-001, EX-001, PF-001, RK-001 (future) |
| Future Product | M-series ⬜ | P-001 ⬜ | Domain Evidence ⬜ | — |

---

## Document Architecture (per product)

Each product evidence portfolio produces four standard documents at Stage 3:

| Document | Audience | Primary question answered |
|---------|---------|--------------------------|
| `{PRODUCT}_EVIDENCE_PROGRAMME.md` | Engineering governance | What evidence was produced, why, and under what governance? |
| `{PRODUCT}_DECISION_EVIDENCE.md` (or equivalent) | Customers, investors, partners | Why does this evidence matter, and what does it say about the product? |
| `{PRODUCT}_BENCHMARK_RESULTS.md` (or equivalent) | Technical evaluators | What exactly were the results and how are they measured? |
| `{PRODUCT}_DEMO_GUIDE.md` | Operators, sales engineers | How do I demonstrate the evidence package consistently? |

These four documents serve distinct audiences without overlapping.

---

## Relationship Diagram

```
Coralys Evidence Governance Standard (this document)
        │
        ├── UltraCrew
        │      ├── M-series  Architecture          ✅ frozen
        │      ├── P-001     Production Readiness  ✅ frozen
        │      ├── WS-001    Workforce Evidence     ✅ frozen
        │      └── Future    RD-001, HC-001, FS-001
        │
        ├── ChronoSentiment
        │      ├── M-series  Architecture          ⬜ not started
        │      ├── P-001     Production Readiness  ⬜ not started
        │      ├── DS-001    Decision Evidence      ⬜ not started
        │      └── Future    ME-001, EX-001, PF-001, RK-001
        │
        └── Future Products
               ├── M-series  Architecture
               ├── P-001     Production Readiness
               └── Domain Evidence
```

---

## Change History

| Date | Change |
|------|--------|
| 2026-07-23 | v1.0 created. Standard derived from UltraCrew M-series, P-001, and WS-001 governance. ChronoSentiment registered as second product. |