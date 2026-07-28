# CR-001 — Constitutional Operationalisation Review

> **Date**: 2026-07-28
> **Status**: Accepted — Pending Ratification
> **Type**: Constitutional Precision Review
> **Constitution**: [`docs/ARCHITECTURE_EVOLUTION.md`](../ARCHITECTURE_EVOLUTION.md) (frozen 2026-07-22)
> **Reviewer**: Programme Architecture
> **Trigger**: UC-ARCH-001 Credit Framework implementation + M-001–M-004 validation chain

---

## Purpose

This document records the first constitutional precision review conducted after successful architectural validation of the Coralys Platform. It does not propose changes to constitutional direction. It identifies four areas where existing constitutional principles can be made more precise and mechanically applicable.

This review follows the Constitution's own amendment standard:

> *Future architectural changes shall originate from implementation evidence, benchmark evidence, pilot evidence, or repeated product evidence.*
> — `ARCHITECTURE_EVOLUTION.md`, Architecture Program Closure

---

## Constitutional Maturity Assessment

The review did not discover architectural inconsistency. It discovered constitutional maturity.

| Stage | Evidence | Constitutional effect |
|---|---|---|
| Architecture definition | Principles, Invariants, Layer Model | Constitutional baseline established |
| Repository convergence | M-001 – M-004 | Constitutional baseline validated |
| New domain (UC-ARCH-001) | Credit Framework, 217 tests | Semantic boundary independently validated |
| CR-001 | This review | Operationalises validated principles |

None of those stages changes the architecture. Each increases confidence in it.

**Finding:** `ARCHITECTURE_EVOLUTION.md` already constitutes the Coralys Platform Constitution. It establishes a coherent and internally consistent constitutional architecture validated through M-001–M-004, benchmark execution, and UC-ARCH-001. No change in architectural direction is warranted.

---

## Evidence Supporting This Review

The Constitution requires implementation evidence, benchmark evidence, or repeated product evidence before architectural changes are justified. All three are present.

| Evidence type | Source |
|---|---|
| Implementation evidence | M-001 (Solution Adapter boundary), M-002 (`coralys-planning` execution contract), M-003 (repository conformance), M-004 (production runtime conformance) |
| Benchmark evidence | INRC2 parity preservation (182 tests, 0 regressions), CVRP benchmark suite, GERAD G-2014-22 benchmark execution (385 duties, 724h horizon) |
| Repeated product evidence | UC-ARCH-001 Credit Framework: 217 tests demonstrating domain/platform separation under realistic airline domain complexity |

---

## UC-ARCH-001 as Constitutional Evidence

UC-ARCH-001 has a dual status following this review.

**As an implementation milestone**: the Credit Framework (phases 1–7) introduced `CreditPolicy`, `CostModel`, `RosterMetrics`, `BaseCreditFloor`, `CreditedHoursBalanceObjective`, and `CreditCostObjective` entirely within `adapters/airline`. The platform (`coralys-core`, `coralys-moga`, `coralys-planning`) has no knowledge of `credited_hours`, `deadhead`, `pairing`, or any airline domain concept.

**As constitutional evidence**: UC-ARCH-001 is the first demonstration under realistic domain complexity that Principle 10 and AI-5 are operationally achievable, not merely aspirational. The airline Credit Framework introduced concepts including `Pairing`, `Deadhead`, `CreditedHours`, and `AgreementRules` without requiring any corresponding concepts in the Platform layer. Future domain implementations can point to UC-ARCH-001 as the established pattern.

---

## Gap Analysis

### Methodology

For each proposed amendment, this review answers four questions:

1. What does the Constitution already require? (cited from constitutional text)
2. What implementation evidence exists?
3. Is there a genuine constitutional gap?
4. If yes, what is the minimal amendment?

---

### Amendment 1 — Intra-Platform Placement Rule

**Constitutional question:** Does the Constitution provide a decision procedure for classifying a concept as Platform Runtime vs Platform Capability vs outside the platform?

**Existing constitutional position:**

Principle 1 and AI-1 establish the inter-layer dependency direction:
```
Application → Solution Adapter → Platform
```
Platform crates must never import Solution Adapters, Products, or Applications.

The Layer Model names the Platform layer crates (`coralys-core`, `coralys-moga`, `coralys-planning`) but treats them as peers without defining their constitutional roles relative to one another.

**Gap type:** Genuine gap.

The Constitution governs *inter-layer placement* (what belongs in Platform vs Solution Adapter vs Application) but not *intra-platform placement* (what belongs in `coralys-core` vs `coralys-moga` vs a future capability crate). A reviewer today cannot apply a constitutional test to determine whether a new concept belongs in the runtime substrate or in a reusable capability crate.

**Proposed amendment:**

Add a constitutional decision procedure for intra-platform placement:

> **Intra-Platform Placement Rule**
>
> Every concept proposed for the Platform layer SHALL be classified as one of:
>
> - **Platform Runtime** — indispensable to platform execution; Coralys cannot boot without it. Belongs in `coralys-core`.
> - **Platform Capability** — reusable by multiple unrelated products without modification. Belongs in a dedicated capability crate (`coralys-moga`, `coralys-planning`, etc.).
> - **Domain Capability** — encodes knowledge about a specific business domain. Does not belong in the Platform layer; belongs in a Domain Library.
>
> A concept may exist in only one classification. Cross-layer duplication is prohibited.

**Evidence:** The existing crate structure already reflects this classification implicitly. `coralys-core` holds execution context and common traits. `coralys-moga` holds the MOGA engine. `coralys-planning` holds the planning execution contract. UC-ARCH-001 demonstrates that airline domain concepts (`CreditPolicy`, `DutyCredit`, `RosterMetrics`) belong in the Domain Library, not the Platform. The amendment makes the implicit classification explicit and testable.

---

### Amendment 2 — Semantic Admissibility Categories for AI-5

**Constitutional question:** Does the Constitution provide an enforceable review rule for determining whether a concept is a domain semantic that must not enter the Platform?

**Existing constitutional position:**

AI-5 states:
> *Domain libraries own semantics. Products own user experience.*

Principle 10 states:
> *Coralys does not ask "How do we fit this domain into Coralys?" It asks "Is Coralys generic enough that this domain can express itself without compromise?"*

**Gap type:** Precision gap.

The constitutional principle exists and is strong. The weakness is operational: reviewers must apply judgment to determine whether a proposed platform concept is "domain semantic" without a constitutional test. The principle is correct but not mechanically enforceable.

**Proposed amendment:**

Add semantic admissibility categories to AI-5:

> **AI-5 Semantic Admissibility**
>
> The following semantic categories are constitutionally prohibited from all Platform crates (`coralys-*`):
>
> - Business entities (e.g. `Crew`, `Nurse`, `Driver`, `Vehicle`, `Patient`)
> - Business workflows (e.g. `PairingOptimization`, `ShiftBidding`, `RoutePlanning`)
> - Operational policies (e.g. `FatigueRule`, `UnionAgreement`, `RegulatoryLimit`)
> - Industry-specific metrics (e.g. `CreditedHours`, `BlockTime`, `FDP`, `ACMENA`)
> - Customer-specific rules (e.g. `AirlineXContractClause`, `HospitalYAwardRate`)
>
> A concept that falls into any of these categories belongs in a Domain Library, regardless of how generically it is named.

**Evidence:** UC-ARCH-001 demonstrates that all five categories can be implemented entirely within `adapters/airline` without Platform involvement. The amendment converts the existing principle into a checklist applicable during code review.

---

### Amendment 3 — Platform Extension Obligation (completing the AI-4 bilateral contract)

**Constitutional question:** Does the Constitution state what the Platform Runtime is constitutionally obliged to provide to enable Solution Adapters to compose Platform Capabilities?

**Existing constitutional position:**

AI-4 states:
> *Solution adapters define which platform capabilities are needed and in what order. The platform executes those capabilities through the common execution contract (`coralys-core`).*

This establishes two constitutional facts:
1. Solution Adapters are responsible for capability composition.
2. The Platform is responsible for capability execution.

**Gap type:** Genuine gap.

AI-4 constitutionalises the obligation on Solution Adapters but not the reciprocal obligation on the Platform Runtime. The Constitution does not state what the Platform must provide to enable that composition. The bilateral contract is half-written.

**Proposed amendment:**

Complete the AI-4 bilateral contract:

> **AI-4 Platform Extension Obligation**
>
> The Platform Runtime SHALL provide a stable mechanism through which Solution Adapters integrate Platform Capabilities. This mechanism must:
>
> - be stable across Platform Runtime versions (subject to AI-7),
> - not require Solution Adapters to depend on Platform implementation details,
> - enable capability composition without coupling Solution Adapters to one another.
>
> The specific form of this mechanism (trait, registry, injection, configuration) is an architectural decision, not a constitutional one. The constitutional obligation is that such a mechanism must exist and be stable.

**Evidence:** M-001 through M-004 demonstrate the pattern in production. The UltraCrew Solution Adapter composes `coralys-moga` and `coralys-planning` through the common execution contract without depending on platform implementation details. The amendment elevates the demonstrated pattern into a constitutional obligation.

---

### Amendment 4 — Differentiated Stability Tiers extending AI-7

**Constitutional question:** Does the Constitution define different stability guarantees for Platform Runtime, Platform Capabilities, and Domain Libraries?

**Existing constitutional position:**

AI-7 states:
> *Breaking changes to a platform interface require an explicit architectural decision recorded in this document. Benchmark adapters must continue to compile after any platform interface change. Domain libraries must not be required to rewrite large portions of code due to arbitrary platform API churn. Stability of the platform interface is a first-class concern.*

**Gap type:** Precision gap.

AI-7 establishes the stability principle and requires explicit decisions for breaking changes. It does not differentiate stability guarantees by layer. Platform Runtime, Platform Capabilities, and Domain Libraries are treated uniformly. As the platform matures and multiple products depend on it, the absence of differentiated stability tiers creates ambiguity about what stability is actually promised to whom.

**Proposed amendment:**

Add differentiated stability tiers to AI-7:

> **AI-7 Stability Tiers**
>
> | Layer | Stability guarantee |
> |---|---|
> | Platform Runtime (`coralys-core`) | Stable. Backwards compatible. Breaking changes require constitutional amendment. |
> | Platform Capabilities (`coralys-moga`, `coralys-planning`, etc.) | Stable public contracts. Internal evolution permitted. Breaking changes require an explicit architectural decision recorded in `ARCHITECTURE_EVOLUTION.md`. |
> | Domain Libraries (`adapters/airline`, `adapters/ultracrew`, etc.) | Product-owned. No platform stability guarantees apply. Evolve freely within the dependency direction rules. |
>
> A "breaking change" is any change that requires a Solution Adapter or Application to modify its code to continue compiling and passing tests.

**Evidence:** The existing benchmark preservation rule (AI-2) already enforces a form of this for INRC2. The amendment generalises the principle across all layers and makes the stability contract explicit for each.

---

## Constitutional Operationalisation

CR-001 introduces a governance activity not previously named in the Constitution's own taxonomy.

The Constitution distinguishes:
- **Principles** — architectural philosophy
- **Invariants** — prohibited relationships
- **Decisions** — recorded resolutions of open questions
- **Milestones** — implementation evidence
- **Program Closure** — freeze declaration

CR-001 belongs in a sixth category:

> **Constitutional Operationalisation** — the refinement of an existing constitutional principle into objective, repeatable review rules without altering the constitutional direction of the platform.

All four amendments share this character: they reduce reviewer discretion, improve repeatability, and make constitutional compliance more mechanically testable. That is an indicator of constitutional maturity rather than constitutional expansion.

---

## Disposition

**Status:** Accepted — Pending Ratification

The four amendments are accepted as analysis. They are not immediately applied to `ARCHITECTURE_EVOLUTION.md`.

**Ratification trigger:** The amendments should be incorporated into `ARCHITECTURE_EVOLUTION.md` when one of the following product-driven events occurs:

- A second Domain Pack is introduced beyond `adapters/airline`
- A second Platform Capability requires extension registration
- A platform versioning or API compatibility issue arises in a pilot
- Pilot feedback exposes a constitutional ambiguity that one of these amendments would resolve

Until a ratification trigger occurs, CR-001 remains archived as accepted analysis. This keeps constitutional amendments exceptional, consistent with the Constitution's own governance model.

**What does not change:** The constitutional architecture, dependency rules, layer model, and all existing Principles, Invariants, and Decisions remain unchanged. CR-001 does not alter any existing constitutional text.

---

## Evidence Chain

```
Architecture Baseline (ARCHITECTURE_EVOLUTION.md v1.0)
        │
        ▼
Repository Convergence (M-001 → M-004, 2026-07-22)
        │
        ▼
UC-ARCH-001 Credit Framework (2026-07-28)
217 tests, 0 failures
Semantic boundary validated under realistic domain complexity
        │
        ▼
CR-001 Constitutional Operationalisation Review (this document)
Accepted — Pending Ratification
        │
        ▼
P-001 Pilot (next governing milestone)
Customer evidence
        │
        ▼
CR-002 (only if future evidence requires it)
```

There is no reason for CR-002 to exist today. A mature constitution accumulates amendments slowly.

---

## Commercial Implication

The constitutional review strengthens the commercial narrative.

The platform architecture has been validated through multiple independent implementations (INRC2, CVRP, airline Credit Framework) without requiring architectural redesign. That is a factual statement that can be made to investors and pilot customers.

> *"The platform architecture has been validated through multiple independent domain implementations without requiring architectural redesign. UC-ARCH-001 demonstrates that a substantial business capability — airline crew credit calculation — was introduced entirely within the Domain Library layer without modifying the platform."*

---

*This document is frozen as of 2026-07-28. It is updated only if the ratification trigger is met or if the constitutional review process itself requires amendment.*