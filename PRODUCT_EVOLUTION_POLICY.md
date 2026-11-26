# UltraCrew Product Evolution Policy

> **Status**: v1.0 — frozen 2026-07-20
> **Applies from**: Phase C (Evidence Generation) onward
> **Supersedes**: Internal assessment documents as the primary driver of product evolution

---

## Purpose

This document defines how UltraCrew's product roadmap is governed after the Internal Governance Phase closes.

Up to and including Demo Assessment v1.2, product evolution was driven by internal evidence: architecture validation, conformance reports, and readiness assessments. Those mechanisms were appropriate while proving that UltraCrew was technically and architecturally sound.

From Phase C onward, the primary source of truth is **operational evidence from pilot customers**. This policy defines how that evidence is collected, classified, synthesised, and translated into roadmap decisions.

---

## Evidence Hierarchy

```
Technical Evidence          (architecture, benchmarks, conformance)
        │
        ▼
Governance Evidence         (frozen baselines, conformance reports, demo assessments)
        │
        ▼
Operational Readiness       (end-to-end workflow validation, Demo Assessment v1.2)
        │
        ▼
Pilot Evidence              (Pilot Evidence Reports, PER-00X series)
        │
        ▼
Pattern Detection           (recurring findings across multiple pilots)
        │
        ▼
Roadmap Candidates          (evidence-qualified feature requests)
        │
        ▼
Product Roadmap
```

Internal evidence (top three layers) is now frozen. It evolves only when a major architectural change occurs. Pilot evidence (bottom four layers) evolves continuously.

---

## Phase Structure

### Phase A — Engineering Completion
Fix backend connectivity, add Backend Status indicator, add error states, complete operational UX improvements.

### Phase B — Operational Readiness
Exercise the complete 5-step workflow end-to-end with a healthy backend. Validate with realistic customer data. Produce and freeze Demo Assessment v1.2. Stop internal readiness assessments.

### Phase C — Evidence Generation (ongoing)
Three parallel workstreams:

| Workstream | Allocation | Focus |
|---|---|---|
| A | 60% | Customer discovery and pilot engagements |
| B | 25% | Engineering improvements driven by pilot evidence |
| C | 15% | Evidence synthesis, pattern detection, roadmap governance |

---

## Pilot Evidence Reports (PER-00X)

Every pilot engagement produces one Pilot Evidence Report. These replace internal assessment documents as the primary governance artifact.

### Fixed structure

| Section | Content |
|---|---|
| Customer Profile | Industry, organisation size, planning team size |
| Operational Context | Current scheduling process, tools, cycle length |
| Existing Workflow | How schedules are currently produced and distributed |
| Data Characteristics | Workers, shifts, skills, constraints, historical data quality |
| Observed Workflow | How planners actually used UltraCrew step by step |
| Observed Bottlenecks | Where planners slowed down, got confused, or needed assistance |
| Observed Decisions | Manual overrides, edits, rejections of generated schedules |
| Observed Exceptions | Edge cases, data quality issues, unexpected behaviours |
| Customer Outcomes | Time saved, schedule quality, planner acceptance, complaints |
| Customer Requests | Capabilities the customer asked for, verbatim where possible |
| Engineering Actions | Prioritised backlog items linked to this evidence |
| Evidence Classification | Customer-specific / Industry-specific / Generalizable |
| Confidence | High / Medium / Low — how certain are we of the finding |
| Repeatability | Observed once / Observed twice / Observed across multiple customers |

### Naming convention

```
PER-001_[customer_code]_[YYYY-MM].md
PER-002_[customer_code]_[YYYY-MM].md
```

---

## Evidence Classification

Every finding in a Pilot Evidence Report is classified as one of:

**Customer-specific** — relevant only to this customer's operational context. Does not influence the roadmap unless the same pattern appears in other pilots.

**Industry-specific** — relevant to a class of customers (e.g. healthcare, aviation ground handling, manufacturing). Influences roadmap when confirmed across two or more customers in the same industry.

**Generalizable** — relevant across industries and customer types. Influences roadmap when confirmed across two or more pilots regardless of industry.

---

## Roadmap Governance

### The rule

No roadmap item is accepted because one customer requested it.

### The process

```
Customer Request
        │
        ▼
Pilot Evidence Report (PER-00X)
        │
        ▼
Evidence Classification (Customer-specific / Industry-specific / Generalizable)
        │
        ▼
Pattern Detection (does this appear in other pilots?)
        │
        ▼
Roadmap Candidate (if threshold met)
        │
        ▼
Implementation
```

### Evidence thresholds

| Classification | Threshold to become a Roadmap Candidate |
|---|---|
| Customer-specific | Not eligible unless reclassified |
| Industry-specific | Observed in ≥ 2 pilots in the same industry |
| Generalizable | Observed in ≥ 2 pilots regardless of industry |

Exceptions may be made for safety-critical issues or contractual commitments, documented explicitly.

---

## Document Lifecycle

### Frozen (evolve only on major architectural change)

- Architecture Baseline v1.0
- ULTRACREW_GTM.md v1.0
- ULTRACREW_PILOT_CHECKLIST.md v1.0
- ULTRACREW_ARCHITECTURE_CONFORMANCE_REPORT_v1.1.md
- ULTRACREW_DEMO_ASSESSMENT_v1.1.md
- ULTRACREW_DEMO_ASSESSMENT_v1.2.md (once produced)
- PRODUCT_EVOLUTION_POLICY.md v1.0

### Living (evolve continuously)

- Pilot Evidence Reports (PER-00X series)
- Pattern Register (recurring findings across pilots)
- Product Backlog (evidence-qualified items)
- Product Roadmap

---

## Success Measures

**Engineering success** is measured by completion of the remaining operational tasks and successful end-to-end workflow validation (Phase A and B).

**Product success** is measured by pilot outcomes, customer adoption, workflow fit, and evidence gathered through Pilot Evidence Reports (Phase C).

**Roadmap quality** is measured by the proportion of shipped features that were driven by recurring patterns across multiple pilots rather than single-customer requests.

---

## Pattern Register

A Pattern Register should be maintained as a living document once three or more Pilot Evidence Reports exist. It records:

- Finding description
- Evidence classification
- Pilots where observed (PER references)
- Repeatability count
- Roadmap candidate status (Yes / No / Pending)

The Pattern Register is the primary input to quarterly roadmap reviews.

---

*This policy takes effect from Phase C onward. It is the final governance document produced during the Internal Governance Phase. Future governance artifacts are operational (Pilot Evidence Reports) rather than internal (readiness assessments).*