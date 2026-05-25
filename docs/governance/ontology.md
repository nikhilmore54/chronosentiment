# ChronoSentiment — System Ontology

**Authority:** Constitutional Layer (highest authority — supersedes all other documents)  
**Status:** Frozen — changes require explicit governance review  
**Last Updated:** 2026-05-25  

---

## Section 1 — System Identity

ChronoSentiment is **institutional replay observability infrastructure**.

It is a system for certifying that a given strategy, applied to a market chronology, produces deterministic, auditable, cryptographically-signed results.

This identity statement is the single source of truth for all architectural decisions. Any feature, component, or behavior that cannot be justified by this identity statement is out of scope.

---

## Section 2 — What ChronoSentiment Is

| Identity Claim | Meaning |
|---|---|
| **Replay infrastructure** | The system replays a fixed chronology of market events against a strategy. The chronology is immutable. The replay is deterministic. |
| **Observability infrastructure** | The system makes the internal state of a replay visible to authorized observers. Observers cannot alter the replay. |
| **Certification infrastructure** | The system produces a cryptographically-signed verdict (`certification_state`) attesting that a strategy produced a specific outcome under a specific chronology. |
| **Deterministic** | Given the same strategy and the same chronology, the system must produce byte-identical output on every invocation. Non-determinism is a system failure. |
| **Auditable** | Every state transition is recorded as a canonical event with a `kernel_signature`. The full event log is sufficient to reconstruct any replay outcome. |

---

## Section 3 — What ChronoSentiment Is Not

These are **ontological prohibitions**. Any implementation that crosses these boundaries is a constitutional violation.

| Prohibited Identity | Why It Is Prohibited |
|---|---|
| **Not a trading platform** | ChronoSentiment does not place orders, manage live positions, or connect to live exchange feeds. It replays historical chronologies. |
| **Not an analytics workstation** | ChronoSentiment does not perform exploratory data analysis, generate charts, or support ad-hoc queries. It certifies replay outcomes. |
| **Not a general backtesting framework** | ChronoSentiment does not support arbitrary strategy languages, pluggable data sources, or configurable simulation parameters at runtime. It applies a fixed kernel to a fixed chronology. |
| **Not a prediction engine** | ChronoSentiment does not forecast future prices, generate trading signals for live use, or optimize strategies for deployment. The GA optimizer is a search tool for finding certifiable strategies, not a prediction system. |
| **Not a real-time system** | ChronoSentiment operates on recorded chronologies. It has no real-time data ingestion path in its certified layer. `cs-ingest` is a chronology recorder, not a live feed processor. |

---

## Section 4 — Authority Hierarchy

The following layers are ordered by authority. Higher layers constrain lower layers. Lower layers cannot override higher layers.

```
Layer 0: Constitutional Layer (this document + governance docs)
  ↓ constrains
Layer 1: Kernel (core/src/ — deterministic simulation engine)
  ↓ constrains
Layer 2: Schemas (schemas/canonical/ — canonical data contracts)
  ↓ constrains
Layer 3: Replay Engine (services/api/ — certified replay execution)
  ↓ constrains
Layer 4: Observer UI (my-chrono-sentiment-ui/ — schema renderer only)
  ↓ constrained by all above
Layer 5: Experimental Tooling (GA optimizer, synthetic data generators)
  ↓ constrained by all above, results not certified
```

**Invariant:** No layer may perform operations reserved for a higher layer.

---

## Section 5 — Forbidden Operations by Layer

### Layer 4 (Observer UI) — Forbidden Operations

The UI is a schema renderer. It must not:

- Compute `certification_state` from fitness thresholds or trade counts
- Generate `narrative_blocks[]` or any replay narrative text
- Resolve ambiguous field names via fallback cascades (ARTIFACT-001)
- Classify strategies as "certified", "verified", or equivalent without reading `certification_state` from the backend
- Perform causal traversal of event chains
- Compute divergence scores
- Rank strategies by any metric not provided by the backend
- Modify, filter, or transform canonical event data before display

### Layer 5 (Experimental Tooling) — Forbidden Operations

The GA optimizer and synthetic generators must not:

- Emit events into the canonical event stream
- Produce `certification_state` verdicts
- Write to the strategy store directly
- Claim that GA-optimized strategies are certified without a replay session

### Layer 3 (Replay Engine) — Forbidden Operations

The Replay Engine must not:

- Use non-deterministic random number generation
- Use `HashMap` where iteration order affects output
- Produce different output for the same `(strategy_id, chronology_hash)` pair
- Emit `certification_state: CERTIFIED` when any `INVALID`-impact event type is missing

### Layer 1 (Kernel) — Forbidden Operations

The Kernel must not:

- Accept runtime configuration that changes simulation behavior after a session starts
- Use wall-clock time as an input to any computation
- Produce floating-point results without fixed-precision arithmetic rules

---

## Section 6 — Ontological Invariants

These invariants must hold at all times. A violation is a system failure, not a bug.

| # | Invariant | Enforcement |
|---|---|---|
| OI-001 | `execution_fitness` is the only certified fitness metric. `ga_fitness` is never certified. | `semantic_registry.md` Section 1 |
| OI-002 | `certification_state` is emitted only by the Replay Engine. No other layer may compute or assign it. | `event_taxonomy.md` Section 1, Layer 4 |
| OI-003 | Every canonical event carries a `kernel_signature`. Events without signatures are not canonical. | `signatures.rs` — `compute_event_signature()` |
| OI-004 | `PRICE_SCALE = 10000`. All prices are stored as integers. Division by `PRICE_SCALE` is the only permitted conversion to ₹. | `core/src/lib.rs` |
| OI-005 | `BTreeMap` is used wherever map iteration order affects output. `HashMap` is forbidden in deterministic paths. | `core/src/ga.rs` |
| OI-006 | Seed derivation in the GA optimizer uses `config.seed ^ generation * 1_000_003 ^ strategy_index * 7_919`. No entropy sources. | `core/src/ga.rs` |
| OI-007 | `holdout_scenarios` is the canonical field name for evaluation isolation. `execution_scenarios` is a retired synonym. | `semantic_registry.md` |
| OI-008 | Port assignments are fixed: API=8000, cs-ingest=8001, observatory=8002, UI=3000. Port 8501 is retired. | `runtime_contract.md` |
| OI-009 | `narrative_blocks[]` is generated by the Replay Engine. The UI renders it. No other arrangement is permitted. | `event_taxonomy.md`, ARTIFACT-005 |
| OI-010 | `RawEventType` in `market_adapter.rs` is an internal parsing type. It must not appear in any canonical event stream or DTO. | `event_taxonomy.md` Section 4 |

---

## Section 7 — Canonical Source of Truth Map

For any question about the system, the authoritative answer is found in:

| Question | Authoritative Source |
|---|---|
| What does a field mean? | `docs/governance/semantic_registry.md` |
| What event types exist? | `docs/governance/event_taxonomy.md` |
| What is a stub/shim? | `docs/governance/transitional_artifacts.md` |
| What are the infrastructure invariants? | `docs/governance/runtime_contract.md` |
| What is the system? | This document (`ontology.md`) |
| What is the canonical data shape? | `schemas/canonical/*.schema.json` |
| What violations exist? | `schemas/canonical/README.md` |
| What are the constitutional laws? | `DISCREPANCY_REPORT.md` Sections 1–4 |

---

## Section 8 — Governance Change Protocol

This document may only be changed by:

1. Identifying a specific ontological claim that is factually incorrect
2. Documenting the correction with evidence from the codebase
3. Updating all downstream documents that reference the changed claim
4. Verifying that no implementation violates the updated claim

Changes that expand the system identity (adding new "what it is" claims) require evidence that the new capability is already implemented and tested.

Changes that contract the system identity (adding new "what it is not" claims) require evidence that the prohibited behavior has been removed from the codebase.