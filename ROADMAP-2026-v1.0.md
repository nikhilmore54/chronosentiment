# Coralys Research & Development Roadmap — 2026

**Document:** ROADMAP-2026-v1.0.md  
**Date:** 2026-07-16  
**Status:** ACTIVE  
**Version:** 1.0

---

## Vision

Coralys is a domain-independent evolutionary optimization platform. Its architecture separates the optimization engine (MOGA core, constraint engine, scenario contract) from domain-specific adapters and solution engines. This separation enables the same platform to power scheduling across airlines, rail, logistics, and healthcare — while maintaining a rigorous, evidence-driven research program.

---

## Phase 1 — Platform Validation ✅ COMPLETE

**Objective:** Demonstrate that the Coralys platform can optimize real scheduling problems across multiple domains without domain-specific changes to the core.

**Benchmarks validated:**
- INRC (nurse rostering) — constraint engine correctness
- CVRP (vehicle routing) — multi-objective optimization
- CVD-001 Strategy A (airline crew scheduling) — industrial dataset ingestion, 1013/1013 shifts, 33/33 workers

**Platform capabilities confirmed:**
- Domain-independent MOGA core
- Scenario contract (externalized constraint semantics)
- Backward-compatible adapter interface
- Reproducible execution pipeline

**Key artifacts:**
- `adapters/ultracrew/src/public_contracts.rs` — Scenario struct
- `adapters/ultracrew/src/constraint_engine.rs` — scenario-aware HC3
- `scripts/cvd001_adapter.py` — Strategy A pipeline
- `data/cvd001/SPRINT9-EXIT-REPORT-v1.0.md` — Phase 1 closure

**Status:** Closed at commit b8b2a9c2 (Sprint 9).

---

## Phase 2 — Benchmark Reproduction & Semantic Validation

**Objective:** Determine whether Coralys can faithfully reproduce the CVD-001 benchmark semantics and document any irreducible differences with evidence.

**Why this phase comes before domain feature development:**

Phase 1 validated the platform. Phase 2 validates the evaluation. Without Phase 2, it is impossible to distinguish between:
- "Coralys matches the benchmark's intended evaluation"
- "Coralys implements a good scheduler that differs from the benchmark"

Those are different claims and must be kept separate.

**Sprints:**

### Sprint 10 — Benchmark Reproduction & Semantic Validation

Mission: Recover authoritative benchmark semantics and compare Coralys against them.

Milestones:
- M1: Benchmark Evidence Recovery — obtain README.pdf, evaluator source, GERAD archives
- M2: Semantic Reconstruction — classify every benchmark rule (hard / soft / objective / reporting)
- M3: Reproduction Study — compare Coralys vs benchmark across all dimensions
- M4: Architectural Decision — Option A (match), Option B (gaps identified), or Option C (ambiguous, freeze hypothesis)

Deliverables: `BENCHMARK-SEMANTICS-v1.0.md`, `REPRODUCTION-STUDY-v1.0.md`, `SPRINT10-DECISION-v1.0.md`

**Parallel research stream:** Representation taxonomy paper — representation-independent evolutionary scheduling evaluated on ROADEF 2010.

**Phase 2 exit criterion:** Every benchmark rule classified with evidence; Coralys reproduction gap documented; forward path formally adopted.

---

## Phase 3 — Domain Solution Engines

**Objective:** Build domain-specific solution engines on top of the validated Coralys platform.

**Entry condition:** Phase 2 complete. Benchmark semantics resolved or documented as irreducibly ambiguous.

**Why this ordering matters:** Phase 3 features are product evolution, not benchmark reproduction. The distinction must be explicit in every sprint plan and research artifact.

### Sprint 11 — Airline Solution Engine

Capabilities:
- Airport graph (connectivity, distances, timezone handling)
- Base continuity (crew must return to home base)
- Duty generator (legal duty construction from flight legs)

### Sprint 12 — Pairing Generator

Capabilities:
- Legal duty sequences
- Pairing optimization (minimize deadheads, balance workload)
- Pairing feasibility constraints (rest, duty time limits)

### Sprint 13 — Commercial Airline Product (UltraCrew v1.0)

Capabilities:
- Monthly roster generation
- Preferential bidding system (PBS) integration
- Recovery scheduling (disruption handling)
- OCC (Operations Control Center) integration

**Other domain solution engines (parallel tracks, post-Sprint 11):**
- UltraRoute — logistics / vehicle routing
- Rail scheduling engine
- Healthcare rostering engine

---

## Phase 4 — Commercial Products

**Objective:** Package domain solution engines as commercial products with full operational capability.

**Products:**
- **UltraCrew** — airline crew scheduling (monthly rosters, PBS, recovery, OCC)
- **UltraRoute** — logistics route optimization
- **ChronoSentiment** — market timing and portfolio scheduling
- Future Coralys applications (rail, healthcare, energy)

**Entry condition:** Phase 3 solution engines validated against real operational datasets.

---

## Research Program

Running in parallel with all phases:

### Active investigations
- CVD-001 benchmark reproducibility (Sprint 10)
- Representation taxonomy (Strategy A through D)

### Target publications
- "Representation-independent evolutionary scheduling: a domain-agnostic MOGA framework evaluated on the ROADEF 2010 airline crew scheduling benchmark"
- "Reproducibility in combinatorial optimization benchmarks: a case study of CVD-001"

### Research governance
- All results labeled by evidence level: verified / hypothesis-driven / exploratory
- No implementation proceeds ahead of evidence
- Benchmark reproduction and product evolution kept explicitly separate

---

## Current State (as of 2026-07-16)

| Phase | Status | Last commit |
|---|---|---|
| Phase 1 — Platform Validation | ✅ Complete | b8b2a9c2 |
| Phase 2 — Benchmark Reproduction | 🔄 Sprint 10 planned | — |
| Phase 3 — Domain Solution Engines | ⏸ Blocked on Phase 2 | — |
| Phase 4 — Commercial Products | ⏸ Blocked on Phase 3 | — |

**Active sprint:** Sprint 10 (entry conditions met, not yet started)  
**Blocked:** Experiment 3 (HC3 implementation) — pending benchmark semantics confirmation  
**Open research question:** CVD-001 HC3 semantics (H1 hypothesis, not yet confirmed)