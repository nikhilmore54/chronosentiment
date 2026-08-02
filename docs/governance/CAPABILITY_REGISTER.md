# Coralys Platform Capability Register

**Document:** GOV-CR-001
**Status:** Active
**Version:** 1.1
**Date:** 2026-08-02
**Owner:** Platform Engineering

---

## Purpose

This register is the authoritative record of every platform capability in the Coralys system, its current maturity level, its owner, and the evidence that supports its current classification.

It is a **platform-wide governance artefact**. Every research programme — CVRP, UltraCrew, ROADEF, or any future domain — updates this register as capabilities advance. It is not specific to any single programme or domain.

---

## Capability Maturity Scale

| Level | Name | Exit Criteria |
|-------|------|---------------|
| C0 | Concept proven | Mathematical formulation documented; theoretical basis established |
| C1 | Unit tested | Implementation exists; unit tests pass; no benchmark validation yet |
| C2 | Benchmark validated | Measurable improvement demonstrated on a recognised benchmark instance set with reproducible evidence |
| C3 | Cross-domain validated | Same capability succeeds in ≥ 2 independent problem domains with separate evidence records |
| C4 | Production validated | Deployed in a production or near-production context; performance documented |
| C5 | Industry-proven | Externally validated through competition result, peer-reviewed publication, or customer deployment |

Promotion between levels is evidence-driven. A capability cannot advance without a filed evidence record that satisfies the exit criteria for the target level. Promotion decisions are recorded in the Amendment Log at the end of this document.

---

## Capability Register

### Core Optimisation Engine

| Capability | Owner | Current | Target | Evidence | Next Milestone |
|------------|-------|---------|--------|---------|----------------|
| Evolution Engine | `coralys-moga` | C4 | C5 | CVRP benchmark, UltraCrew production pilot, ROADEF baseline | ROADEF competition result |
| Multi-objective optimisation (MOGA) | `coralys-moga` | C4 | C5 | CVRP, UltraCrew | ROADEF RP-406 |
| Observability / telemetry | `coralys-infrastructure` | C4 | C4 | Production deployment | — |
| Ecology / adaptive search | `coralys-ecology` | C3 | C4 | CVRP, UltraCrew | Production deployment |

### Planning and Search

| Capability | Owner | Current | Target | Evidence | Next Milestone |
|------------|-------|---------|--------|---------|----------------|
| Large neighbourhood search (LNS) | `coralys-planning` | C0 | C3 | — | RP-404 (ROADEF) |
| Multi-path candidate generation | `coralys-planning` | C0 | C3 | — | RP-403 (ROADEF) |
| Budget-aware transition planning | `coralys-planning` | C1 | C3 | RP-000 (budget semantics, ROADEF) | RP-402 (ROADEF) |
| Hyper-heuristic operator selection | `coralys-ecology` | C1 | C4 | CVRP operator ablation | RP-405 (ROADEF cross-domain) |

### Routing and Network Optimisation

| Capability | Owner | Current | Target | Evidence | Next Milestone |
|------------|-------|---------|--------|---------|----------------|
| Network routing (SR paths) | `adapters/roadef` | C2 | C3 | ROADEF Baseline v1.0 — Dataset A, 20 instances, commit `ec4d3821` | RP-401D |
| ECMP-aware routing | `coralys-core` | **C2** | C3 | RP-401C — 13/20 improved, 0 regressed, total improvement 2,512,099.84 (BASELINE_HISTORY v1.2) | RP-401 cross-domain |
| Budget-constrained re-routing | `coralys-planning` | C1 | C2 | RP-000 (shared-path strategy) | RP-402 (ROADEF) |

### Domain Adapters

| Capability | Owner | Current | Target | Evidence | Next Milestone |
|------------|-------|---------|--------|---------|----------------|
| Vehicle routing (CVRP) | `adapters/cvrp` | C3 | C4 | CVRP benchmark series, 64 experiment binaries | Production deployment |
| Workforce scheduling (INRC) | `adapters/ultracrew` | C3 | C4 | UltraCrew INRC benchmark, production pilot | Production deployment |
| Segment routing (ROADEF) | `adapters/roadef` | C2 | C5 | ROADEF Baseline v1.0 | RP-401 through RP-407, competition submission |

---

## Promotion Protocol

When a capability is ready for promotion:

1. File an evidence record (using the standard RP evidence schema or equivalent).
2. Confirm exit criteria for the target level are satisfied.
3. Update this register with the new level, evidence reference, and date.
4. Record the promotion in the Amendment Log below.

Promotion decisions require sign-off from the Platform Owner.

---

## Capability Dependency Map

Some capabilities depend on others reaching a minimum maturity level before they can advance:

```
Evolution Engine (C4)
        │
        ├── Multi-objective optimisation (C4)
        │
        └── Ecology / adaptive search (C3)
                │
                └── Hyper-heuristic operator selection (C1)
                        │
                        └── RP-405 → C3 (cross-domain: CVRP + ROADEF)

Budget-aware transition planning (C1)
        │
        └── RP-402 → C2 (ROADEF benchmark)
                │
                └── RP-402 + CVRP validation → C3

ECMP-aware routing (C2) ← promoted 2026-08-02 via RP-401C
        │
        └── RP-401 cross-domain validation → C3

LNS for routing (C0)
        │
        └── RP-404 → C2 (ROADEF benchmark)
                │
                └── RP-404 + CVRP validation → C3
```

---

## Amendment Log

| Version | Date | Change | Authorised By |
|---------|------|--------|---------------|
| 1.0 | 2026-08-02 | Initial register. Baseline established from RR1–RR4 governance work and ROADEF Baseline v1.0 (commit `ec4d3821`). Network routing promoted to C2 on strength of Dataset A results. Budget-aware transition planning promoted to C1 on strength of RP-000 finding. | Programme Owner |
| 1.1 | 2026-08-02 | ECMP-aware routing promoted C1→C2. Evidence: RP-401C — 13/20 Dataset A instances improved, 0 regressions, total improvement 2,512,099.84. Full evidence in BASELINE_HISTORY.md v1.2 and RP401_FINAL_REPORT.md v1.1. | RP-401 |