# Coralys Product Portfolio

> **Status**: v1.1 — frozen (Architecture Baseline v1.0)
> **Date**: 2026-07-19
> **Relationship**: This document defines business strategy. It is consistent with [`docs/CODEBASE_ASSESSMENT.md`](docs/CODEBASE_ASSESSMENT.md) (what exists) and [`docs/ARCHITECTURE_EVOLUTION.md`](docs/ARCHITECTURE_EVOLUTION.md) (how it evolves). Where this document describes a vision, it is marked as such.

---

## 1. What is Coralys?

Coralys is a **Decision Optimization Platform** — a collection of reusable engines and libraries that power workforce and financial decision products.

Coralys does not ship to end users directly. It is the shared foundation that product teams build on.

### Platform Libraries

| Component | Purpose | Status |
|---|---|---|
| `coralys-moga` | Multi-objective genetic algorithm engine | Mature |
| `coralys-core` | Shared traits: Scenario, Solution, Outcome, DecisionPlugin | Mature |
| `coralys-eval` | Evaluation pipeline and registry | Stub |
| `coralys-ecology` | Population dynamics and diversity management | Stub |
| `coralys-decision` | Decision lineage and audit trail | Stub |
| `coralys-simulation` | Discrete-event simulation framework | Stub |
| `coralys-recommendation` | Recommendation and ranking engine | Stub |
| `coralys-matching` | Constraint-based matching primitives | Stub |
| `coralys-scheduling` | Scheduling domain library (currently airline-focused; long-term scope under architectural review — see [`docs/ARCHITECTURE_EVOLUTION.md`](docs/ARCHITECTURE_EVOLUTION.md) OQ-1) | Partial |

---

## 2. What Products Exist?

```
                    Coralys
          Decision Optimization Platform
                        │
        ┌───────────────┼────────────────┐
        │               │                │
   UltraCrew       AirlineOps     ChronoSentiment
   Workforce         Airline        Financial
   Rostering       Crew Mgmt     Decision Intelligence
```

### UltraCrew
**Vision**: Workforce Rostering Decision Platform — solves the general problem of assigning workers to shifts across any industry where the core model is: workers, shifts, skills, coverage, availability, contracts, and preferences.

**Current implementation**: The initial implementation is based on the mature INRC2 nurse rostering engine (`adapters/ultracrew/src/inrc/`). The generic workforce layer (`src/workforce/`) is a stub. UltraCrew will evolve toward the generic workforce model defined in the architecture roadmap.

### AirlineOps
**Vision**: Airline Crew Management Platform — solves the full airline crew management pipeline: flight schedule ingestion, crew pairing, crew assignment, crew recovery, crew control, and fatigue analysis.

**Current implementation**: The airline domain model exists in `coralys-scheduling` (FlightLeg, Duty, Pairing, Rotation, FDP, legality rules, optimization, resilience, planner). The AirlineOps product layer (pairing optimizer, crew assignment, recovery, crew control) does not yet exist.

### ChronoSentiment
**Vision and current implementation**: Financial Decision Intelligence Platform — applies temporal sentiment analysis and multi-objective optimization to financial decision problems. Active.

---

## 3. Product Relationship

```
        Coralys Platform
               │
               │  provides reusable capabilities
               │
    ┌──────────┼──────────┐
    ▼          ▼          ▼
UltraCrew  AirlineOps  ChronoSentiment
```

- Coralys is not sold directly. It is the shared platform.
- Products are independently marketable.
- Products use platform capabilities through interfaces. They do not depend on one another.
- Shared capabilities are owned by the platform, not by any product.
- AirlineOps uses the **generic planning capability** provided by the Coralys platform for crew assignment. Whether AirlineOps shares UltraCrew's deployment infrastructure is an operational decision, not an architectural one.

---

## 4. What Does Each Product Own?

### UltraCrew Owns

**Domain model** (vision — generic layer not yet implemented)
- Worker (identity, skills, contracts, availability, preferences)
- Shift (timing, skill requirements, coverage demand)
- Roster (assignment of workers to shifts over a planning horizon)
- Constraint (legal, contractual, operational)
- Objective (workload balance, preference satisfaction, cost)

**Current implementation**
- INRC2 nurse rostering engine (mature)
- INRC2 benchmark suite: n030–n120 instances (permanent — see Architectural Invariant AI-2)

**Application capabilities** (Stream B — to be built against generic interface)
- B1 — Planner Workspace: interactive roster construction and editing
- B2 — Disruption Console: real-time response to worker unavailability
- B3 — Explanation Engine: why a worker was or was not assigned
- B4 — Operational Readiness: worker qualification, certification, and availability status

**Applicable industries** (vision)
- Healthcare (nurse rostering, hospital staff scheduling)
- Manufacturing (shift planning, certification tracking)
- Retail (weekend and seasonal staffing)
- Contact Centres (intraday and weekly rostering)
- Hospitality (hotel and restaurant staff scheduling)
- Logistics and Warehousing (shift planning)
- Security (guard rostering)

**Does not own**
- Flight legs, pairings, duties, deadheads
- Aircraft qualifications, base assignments
- Flight Duty Period (FDP) limits
- Fatigue risk management systems (FRMS)
- Real-time crew control operations

---

### AirlineOps Owns

**Domain model** (exists in `coralys-scheduling`)
- Flight leg (origin, destination, departure, arrival, aircraft type)
- Duty (sequence of flight legs within a duty period)
- Pairing (sequence of duties forming a legal crew trip)
- Deadhead (positioning flight)
- Base (crew home base)
- Aircraft qualification (type rating)
- FDP (Flight Duty Period) and rest rules
- Fatigue score

**Application capabilities** (to be built — Phase 5)
- Flight schedule import and validation
- Crew pairing optimizer
- Crew assignment (via Coralys generic planning capability)
- Crew recovery (disruption re-optimization)
- Crew control (real-time replacement decisions)
- Fatigue analysis and FRMS integration
- Operations dashboard

**Does not own**
- The generic shift assignment engine (that is a Coralys platform capability)
- Generic worker/shift/skill/coverage model

---

### ChronoSentiment Owns

**Domain model**
- Temporal event stream
- Sentiment signal
- Asset class
- Decision horizon
- Continuity claim

**Application capabilities**
- Historical capture and replay
- Claim registry and validation
- Scenario comparison
- Decision intelligence reporting

**Does not own**
- Workforce scheduling
- Airline operations

---

## 5. How Do Products Interact?

### AirlineOps → Coralys Platform (for crew assignment)

AirlineOps uses the Coralys generic planning capability for crew assignment. It does not depend on UltraCrew the product.

```
AirlineOps

  Flight Schedule
        │
        ▼
  Crew Pairing Optimizer
        │
        ▼  (produces legal pairings)
        │
  Coralys Generic Planning Capability
        │
        ▼  (assigns crew members to pairings as atomic planning units)
        │
  Published Crew Roster
        │
        ▼
  Crew Recovery
        │
        ▼
  Crew Control
```

The integration point is conceptually defined:

> AirlineOps produces a set of **legal pairings**.
> For airline crew assignment, the current working assumption is that pairings will act as the Atomic Planning Units consumed by the Coralys generic planning capability. This will be validated during Phase 1.
> The capability assigns qualified crew members to those units.
> The resulting roster is returned to AirlineOps for publication and downstream operations.

The precise interface type and the exact mapping of pairings to the generic planning interface will be determined in Phase 1 of the architecture roadmap (see [`docs/ARCHITECTURE_EVOLUTION.md`](docs/ARCHITECTURE_EVOLUTION.md) OQ-3).

### Coralys → All Products

All products consume Coralys platform components as libraries. The dependency direction is always:

```
Product → Coralys Platform Component
```

Never the reverse.

---

## 6. Capability Ownership Table

| Capability | Business Owner | Platform Support |
|---|---|---|
| Workforce assignment | UltraCrew | Coralys planning capability |
| Worker availability | UltraCrew | Generic |
| Skill matching | UltraCrew | Generic |
| Coverage optimization | UltraCrew | Coralys planning capability |
| Contract enforcement | UltraCrew | Generic |
| Preference satisfaction | UltraCrew | Generic |
| Disruption response (generic) | UltraCrew | — |
| Disruption response (airline) | AirlineOps | — |
| Explanation engine | UltraCrew | — |
| Operational readiness (generic) | UltraCrew | — |
| Operational readiness (qualifications, FDP) | AirlineOps | — |
| Flight pairing | AirlineOps | — |
| Flight recovery | AirlineOps | — |
| Crew control | AirlineOps | — |
| Fatigue modelling | AirlineOps | Policy support |
| Multi-objective optimization | Shared | Coralys MOGA |
| Simulation | Shared | Coralys Simulation |
| Decision lineage | Shared | Coralys Decision |

---

## 7. What This Means for Stream B

Stream B implements the four UltraCrew application modules. All four modules are **generic workforce rostering capabilities** — not industry-specific.

| Module | Generic framing | Applicable to |
|---|---|---|
| B1 — Planner Workspace | Interactive roster construction and editing | All industries |
| B2 — Disruption Console | Response to worker unavailability | All industries |
| B3 — Explanation Engine | Assignment rationale and constraint audit | All industries |
| B4 — Operational Readiness | Worker qualification, certification, availability | All industries |

B4 examples by industry: hospital (skill gaps, mandatory training), manufacturing (certification expiry), retail (weekend availability), airline (type ratings, medical validity, visa status).

---

## 8. Product Implementation Status

| Product | Current implementation status |
|---|---|
| UltraCrew | Initial implementation exists — backed by the mature INRC2 nurse rostering engine and supporting libraries. Generic workforce layer is a stub. |
| AirlineOps | Product defined. Airline domain model exists in `coralys-scheduling`. Product layer (pairing optimizer, crew assignment, recovery, crew control) not yet implemented. |
| ChronoSentiment | Active. |
| Coralys Platform | Active (`coralys-moga`, `coralys-core` mature). Seven platform crates are stubs. Generic planning capability crate does not yet exist. |

---

## 9. Naming

| Product | Working name | Notes |
|---|---|---|
| Workforce Rostering Platform | UltraCrew | Active |
| Airline Crew Management | AirlineOps | Working name — may change before implementation begins |
| Financial Decision Intelligence | ChronoSentiment | Active |
| Decision Optimization Platform | Coralys | Active |

---

*This document will be frozen once the corrections in §1 (`coralys-scheduling` description) and §5 (AirlineOps dependency wording) have been reviewed and accepted. Changes after freezing require a new version number and date.*