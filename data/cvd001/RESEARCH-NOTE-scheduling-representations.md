# Research Note: Scheduling Representation Taxonomy
**Date:** 2025-07-13
**Status:** DRAFT — not frozen. Leave as draft until CVD-001 execution is complete.
**Origin:** CVD-001 Milestone 1 architectural discussion
**Stream:** C (Research Station)

---

## Observation

The CVD-001 schema mapping forced an explicit choice between two scheduling representations:

- **Strategy A:** Flight Leg → Shift (chosen for Sprint 9)
- **Strategy B:** Duty → Shift (deferred to Phase II)

This choice is not unique to airline scheduling. It recurs across every scheduling domain, but most literature assumes one representation without justification.

---

## Core Principle

**Scheduling representation and optimization algorithm are orthogonal design decisions.**

The scheduling representation determines the decision variables and constraint model. The optimization algorithm determines how the resulting search space is explored. Commercial scheduling systems are typically designed around a specific scheduling representation (for example, pairings in airline systems or shifts in workforce systems). Coralys explores an alternative architectural approach in which the optimization engine is separated from the scheduling representation through domain-specific adapters, allowing the same optimization engine to operate on multiple scheduling representations.

This is the conceptual contribution of the proposed research. The paper is about **representation**, not about MOGA.

---

## Representation Taxonomy

| Strategy | Optimization unit | Commercial use | Typical industries |
|---|---|---|---|
| S1 | Activity / task | Rare | Research |
| S2 | Shift / flight leg | Yes | Hospitals, workforce, retail |
| S3 | Duty | Yes | Rail, bus, airlines |
| S4 | Pairing | Yes | Airlines |
| S5 | Rotation / Tour | Yes | Long-haul airlines, logistics |
| S6 | Monthly Crew Line | Yes | Airlines (rostering) |
| S7 | Hierarchical | Increasingly common | Large enterprise systems |

S5 (Rotation/Tour) is relevant for long-haul operations where pairings span multiple days and crew may operate international rotations. S7 (Hierarchical) covers systems that optimize at multiple levels simultaneously (e.g., duty + pairing + roster in a single pipeline).

### Representation Principle

Moving from S1 to S7 changes **the optimization object**, not the optimization algorithm.

The choice of representation primarily influences:
- the decision variables,
- the search-space size and structure,
- the constraints that can be expressed naturally,
- the amount of preprocessing required,
- the explainability of the resulting schedule to domain practitioners.

The optimization algorithm then determines how effectively that search space is explored.

The optimization engine itself may remain unchanged. This is the central claim of the proposed research.

---

## Commercial Airline Pipeline (typical)

```
Flights
  ↓
Duty Generation
  ↓
Pairing Generation
  ↓
Crew Assignment
  ↓
Crew Rostering
```

Large commercial airline crew scheduling systems typically optimize pairings or higher-level crew constructs rather than individual flight legs, because preprocessing substantially reduces the combinatorial search space while embedding regulatory constraints:
- 10,000 flight legs → 2,000 legal duties → 600 legal pairings

Examples: Lufthansa NetLine, Jeppesen Crew, Sabre Crew — all use pairing as the optimization unit.

---

## Hospital / Workforce Scheduling (typical)

```
Shift requirement
  ↓
Worker
```

No pairing concept. UltraCrew naturally belongs in this family (S2).

---

## Coralys Architectural Approach

Coralys separates the evolutionary algorithm from the scheduling representation. The optimization unit is not fixed — it is determined by the adapter. This means:

```
UltraCrew adapter  → Shift (S2)
Airline adapter    → Pairing (S4)
Rail adapter       → Duty (S3)
```

...without changing the MOGA core. This architectural separation enables Coralys to support multiple scheduling representations through domain-specific adapters. Whether this represents a practical differentiator relative to commercial systems is a question for future comparative research.

**Possible long-term architectural direction (not a committed design):** `Optimization Object<T>` where T ∈ {FlightLeg, Duty, Pairing, CrewLine, PatientVisit, TrainTrip, DeliveryStop}. This would formalize the adapter contract at the type level. It is recorded here as an architectural idea, not an intended implementation.

---

## Proposed Research Paper Structure

1. **Introduction** — Why representation matters; the algorithm/unit distinction
2. **Taxonomy** — S1–S7 with formal definitions
3. **Complexity Analysis** — Decision variables, constraint propagation, search-space size, memory, runtime
4. **Industrial Usage** — Healthcare, airlines, rail, public transport, logistics
5. **Experimental Comparison** — Same benchmark at different abstraction levels; compare solution quality, runtime, explainability, preprocessing effort, portability
6. **Platform Architecture Implications** — Why engines should separate algorithm from representation; Coralys as a case study

---

## Relationship to CVD-001

The CVD-001 pipeline provides the first empirical data point. The three-level progression keeps CVD-001 answering its intended question (can Coralys ingest and optimize a real industrial dataset?) without prematurely building an airline scheduling system.

**Level 1 — Platform validation (Sprint 9)**
- Import CVD-001; map legs to shifts (Strategy A / S2)
- Validate Coralys on a real industrial dataset
- Measure: feasibility (HC=0), credited-hour coverage (±2% target), runtime, PAS

**Level 2 — Airline capability layer (Phase II product work)**
- Duty Generator: construct legal duties from connected legs
- Pairing Generator: construct base-return pairings from duties
- Add: base continuity, deadheading, aircraft qualifications, regulatory rules
- These are adapter/product capabilities, not Coralys platform features

**Level 3 — Commercial airline product (future)**
- Monthly crew rostering, bid lines, preferential bidding
- Disruption recovery, real-time re-optimization
- Operations control integration

**Key architectural principle:** The Duty Generator and Pairing Generator belong to the airline solution engine layer, not inside Coralys. Coralys remains domain-independent. Only the adapter changes between S2 (shifts) and S4 (pairings).

If Strategy A produces acceptable credited-hour coverage (±2% target), it provides evidence that S2 is sufficient for initial airline validation. If it fails, it provides the complexity argument for S3/S4. Either outcome contributes useful empirical evidence toward the proposed comparative study and motivates the next stage of investigation.

---

## Open Research Questions

This note motivates several future questions:

- How does scheduling representation affect optimization quality?
- At what problem size does preprocessing become advantageous?
- Which representations produce the most explainable schedules?
- Can one optimization engine support multiple scheduling representations without sacrificing performance?

These questions remain open until empirical comparison has been performed.

The taxonomy is intended as a descriptive framework rather than a prescriptive hierarchy. Different industries, problem scales, and operational objectives may justify different scheduling representations. In many domains — such as hospital rostering — S2 is exactly the right representation.

---

## Governance Note

This research note does not trigger H9 or UB-003. It is a Stream C observation arising from product work. Any experimental comparison would require a new UB benchmark (UB-003) with explicit governance approval per GOV-001.

*DRAFT — not frozen. Revise after CVD-001 execution is complete.*