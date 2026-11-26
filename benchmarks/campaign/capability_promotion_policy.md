# Coralys Capability Promotion Policy
## GOV-010 — Platform Normative Document

*Applies to: All Coralys optimization domains and platform capabilities.*
*Companion to GOV-008 (Benchmark Qualification) and GOV-009 (Feasibility & Execution Qualification).*
*Version: 1.1 — Frozen, 2026-07-08.*

---

## Normative Principle

> **New algorithms, diagnostics, and engineering capabilities SHALL first be implemented
> and validated within the designated proving-ground adapter. Only after demonstrating
> measurable value, domain independence, architectural stability, and governance
> compatibility SHALL they be promoted into the Coralys platform as reusable capabilities.**
>
> For Coralys v1.x, the designated proving-ground adapter is **CVRP**.
>
> Promotion decisions apply to **individual capabilities**, not entire adapters,
> frameworks, or products. Each capability progresses independently through the
> promotion pipeline.
>
> Every promotion SHALL reference traceable evidence in a Qualification Data Record (QDR).
> Promotion without traceable evidence is prohibited.

---

## §1 Governance Context

GOV-010 completes the three-document governance system:

| Document | Question answered |
|---|---|
| **GOV-008** | Can we trust the benchmark comparison? |
| **GOV-009** | Is the problem feasible, and how should execution proceed? |
| **GOV-010** | Has a capability earned promotion into the Coralys platform? |

These three documents address orthogonal concerns without overlapping responsibilities.

---

## §2 Development Pipeline

Every new capability follows this pipeline:

```
Research Idea
      │
      ▼
Stage 1 — Experiment (proving-ground adapter only)
      │
      ▼
Stage 2 — Validation (multiple campaigns)
      │
      ├── Promote  → Stage 3 — Coralys Platform
      ├── Retain   → Keep in proving-ground adapter permanently
      ├── Reject   → Discard; document reason in QDR
      └── Defer    → Collect additional evidence; revisit
```

---

## §3 Three Promotion Stages

### Stage 1 — Experiment (Proving-ground adapter only)

**Goal:** Does it work?

The capability is implemented only in the proving-ground adapter.
No abstraction. No generalization. Minimum viable implementation.

> **Stage 1 implementations prioritize rapid evidence collection over abstraction,
> reuse, or API stability. Ugly code is acceptable at Stage 1.**

**Measurements required:**
- Does it produce correct results?
- What is the runtime cost?
- What is the engineering effort?
- Does it improve at least one measurable dimension (see §4)?

**Exit criterion:** Measurable benefit demonstrated on at least one campaign run.

---

### Stage 2 — Validation (Multiple campaigns)

**Goal:** Is the benefit real and repeatable?

Run at least two campaigns with the capability enabled.
Collect before/after evidence.

**Example evidence format:**

```
Capability: FC-2 Capacity Validation
Before: 144 instances, 67s average runtime
After:  15 impossible instances skipped, 54s average runtime
Saving: 13s average, 1872s total campaign time
Evidence level: Observed (two campaigns)
```

Evidence must be `Observed` or `Verified` per GOV-008 evidence hierarchy.
Hypotheses do not qualify.

**Stage 2 outcomes — four paths:**

| Outcome | Condition | Action |
|---|---|---|
| **Promote** | Measurable benefit confirmed; domain-independent | Proceed to Stage 3 |
| **Retain** | Benefit confirmed; domain-specific | Keep in proving-ground adapter permanently |
| **Reject** | No measurable benefit, or cost exceeds benefit | Discard; document reason in QDR |
| **Defer** | Insufficient data; campaign interrupted; benchmark issue | Collect additional evidence; revisit |

Not every experiment deserves promotion. Not every experiment deserves retention.
Some experiments should be rejected after evidence shows they are not worthwhile.
Some experiments need more data before a decision can be made — these are deferred, not rejected.

---

### Stage 3 — Promotion (Coralys platform)

**Goal:** Make the capability available to all optimization domains.

Generalize. Abstract. Move into Coralys.

**Promotion checklist:**

| Question | Requirement |
|---|---|
| Is it domain-independent by design and evidence? | Yes |
| Has it demonstrated measurable benefit (net-positive)? | Yes |
| Does it simplify future adapters? | Yes |
| Does it reduce duplication? | Yes |
| Is the API stable (no breaking changes expected)? | Yes |
| Can existing adapters continue working without modification? | Yes (or migration documented) |
| Does it comply with GOV-008 and GOV-009 (governance compatibility)? | Yes |
| Is there a QDR entry with traceable evidence for this promotion? | Yes |

All eight must be Yes before promotion.

**Note on domain independence:** A capability may be promoted before a second adapter
exists if it has been shown to be architecturally domain-independent. The requirement
is domain independence by design and evidence — not the existence of a second consumer.

### §3.1 Promotion Stability

Once a capability is promoted into Coralys, subsequent experimentation SHALL
occur in the originating proving-ground adapter.

The platform implementation evolves only through evidence-backed revisions.

Platform capabilities SHALL NOT be used as experimental sandboxes.

### §3.2 Promotion Record

Every promoted capability SHALL include a Promotion Record with the following fields:

```
Promotion Record

Capability identifier:    <name>
Originating adapter:      <e.g. CVRP>
Originating QDR ref:      <QDR version and section>
Promotion decision date:  <YYYY-MM-DD>
Evidence summary:         <one paragraph>
Promoted API / version:   <e.g. coralys-moga v0.2, FeasibilityCertificate>
Governance compatibility: GOV-008 ✓  GOV-009 ✓  GOV-010 ✓
```

---

## §4 Measurable Benefit Definition

"Measurable benefit" is not subjective. A capability must improve at least one of:

| Dimension | Example measurement |
|---|---|
| **Execution time** | Average campaign runtime reduced by X% |
| **Solution quality** | Average gap reduced by X% |
| **Qualification confidence** | Confidence KPI increased from X% to Y% |
| **Observability** | New diagnostic data available per instance |
| **Engineering simplicity** | Lines of code reduced; complexity reduced |
| **Maintainability** | Fewer special cases; clearer invariants |

A capability must improve at least one dimension **without causing unacceptable regression
in another measured dimension.** For example: runtime ↓ 20% with quality ↓ 30% fails
promotion — the net effect is negative. Improvement must be net-positive across the system.

**Measured benefit shall be evaluated against the baseline implementation under
comparable benchmark conditions.** Comparisons against unrelated runs or different
datasets are not valid evidence.

If a capability improves none of these dimensions, it SHALL NOT be promoted.

---

## §5 What Belongs in the Proving-Ground Adapter

Domain-specific capabilities stay in the proving-ground adapter permanently.

Examples (CVRP):
- CVRPLIB parsing and `.vrp` file format
- TSPLIB distance metrics (EUC_2D, GEO, EXPLICIT)
- CMT / Augerat / Taillard / X-family registries
- BKS databases and provenance records
- Route split algorithms
- Vehicle capacity calculations
- Distance matrix handling
- CVRP-specific constraint validation

These are routing concepts. They do not transfer to other domains.

---

## §6 What Graduates into Coralys

Only capabilities that are demonstrably domain-independent.

### Qualification Framework (GOV-008, GOV-009)

FC-1, FC-2, FC-3 are not routing concepts. They are optimization concepts.

| Concept | CVRP form | General form |
|---|---|---|
| Structural validation | Graph connectivity, demand non-negativity | Instance schema validation |
| Capacity validation | `Σd_i ≤ K×Q` | Resource bound check |
| Bin packing relaxation | FFD on vehicle demands | FFD on resource demands |
| Feasibility confidence | F0–F5 ladder | F0–F5 ladder (domain-agnostic) |

A workforce scheduling instance also has structural validity, capacity validity,
and bin packing relaxation (can shifts be covered by available staff?).
The framework transfers directly.

### Telemetry and Observability

Generation statistics, operator statistics, convergence, diversity, execution timing —
these benefit every optimization domain. Already in `coralys-moga`.

### Runtime Prediction

Not CVRP-specific. A crew scheduling optimizer also needs runtime estimates.
Pending Stage 2 validation.

### Difficulty Estimation

A workforce scheduling instance also has easy, moderate, hard, and extreme cases.
The Difficulty Index concept transfers directly.
Pending Stage 2 validation.

### Benchmark Qualification (GOV-008)

Every optimization adapter eventually needs benchmark provenance, metadata validation,
and comparison certificates. Already platform-level in GOV-008.

---

## §7 Three-Layer Architecture

```
Coralys Platform
│
├── Generic capabilities (promoted from proving-ground adapter)
│   ├── Evolution Engine (coralys-moga)
│   ├── Qualification Framework (GOV-008, GOV-009)
│   ├── Feasibility Certification (FC-1 through FC-5)
│   ├── Observability and Telemetry
│   ├── Runtime Prediction (pending Stage 2 validation)
│   ├── Difficulty Estimation (pending Stage 2 validation)
│   └── Reporting
│
└── Domain Adapters
    ├── CVRP (proving ground — v1.x)
    ├── UltraCrew (future)
    ├── Workforce Scheduling (future)
    ├── Crew Scheduling (future)
    ├── Vehicle Routing (future)
    └── Future domains
```

---

## §8 Anti-Patterns

The following are explicitly prohibited:

| Anti-pattern | Why prohibited |
|---|---|
| Abstracting before measuring | Produces wrong abstractions |
| Promoting without evidence | Pollutes the platform with unvalidated code |
| Promoting without a QDR entry | Promotion without traceable evidence is prohibited |
| Keeping domain-agnostic code in the adapter | Creates duplication when new domains are added |
| Designing for hypothetical future domains | Premature generalization |
| Skipping Stage 2 validation | Evidence is required; hypotheses are not evidence |
| Treating "Retain" as failure | Domain-specific capabilities belong in the adapter |
| Treating "Reject" as failure | Discarding bad experiments is good engineering |
| Treating "Defer" as Reject | Insufficient data is not the same as negative evidence |
| Using the platform as an experimental sandbox | Platform capabilities evolve only through evidence-backed revisions |

---

## §9 Current Promotion Status

| Capability | Stage | Status | Evidence |
|---|---|---|---|
| Evolution Engine | Promoted | In Coralys (`coralys-moga`) | Campaign v1.1 144/144 |
| Observability / Telemetry | Promoted | In Coralys (`coralys-moga`) | Campaign v1.1 |
| Benchmark Qualification (GOV-008) | Promoted | Platform normative document | Campaign v1.1 |
| Feasibility Certification (GOV-009) | Stage 1 | CVRP only — FC-1/FC-2.5/FC-2/FC-3 implemented | Build clean; Stage 2 validation pending |
| Early Convergence Termination | Stage 1 | CVRP only — `NoImprovement(30)` wired | Build clean; Stage 2 validation pending |
| Difficulty Index (DI) | Stage 1 | Defined in GOV-009; not yet implemented | Stage 2 validation pending |
| Runtime Prediction | Stage 1 | Defined in GOV-009; not yet implemented | Stage 2 validation pending |
| Execution Profiles | Stage 1 | Defined in GOV-009; not yet implemented | Stage 2 validation pending |

---

## §10 Qualification History

| Version | Date | Author | Change |
|---|---|---|---|
| 1.0 | 2026-07-08 | Coralys Engineering | Initial specification — formalized from CVRP qualification campaign experience. Six refinements applied: domain independence criterion clarified; Retire→Reject; Defer outcome added; measurable benefit defined concretely with net-positive requirement; backward compatibility added; Stage 1 "ugly code" principle stated; QDR requirement added; §3.1 Promotion Stability added. |
| 1.1 | 2026-07-08 | Coralys Engineering | Frozen. Six timelessness and traceability refinements: "currently CVRP" → "designated proving-ground adapter"; capability-specific promotion statement added; "Phase 4" references replaced with "Stage 2 validation pending"; §3.2 Promotion Record subsection added; measurable benefit baseline comparison requirement added; status table decoupled from release plan. |

---

*This document is normative. All Coralys engineering decisions about capability scope
and promotion SHALL follow this policy. The proving-ground adapter remains the
innovation laboratory. Coralys remains the stable, reusable platform.*