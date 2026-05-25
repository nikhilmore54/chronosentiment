# ChronoSentiment — Transitional Artifacts Registry

**Authority:** Constitutional Layer  
**Status:** Active — all stubs, shims, and temporary DTOs must be registered here  
**Last Updated:** 2026-05-25  

---

## Purpose

A transitional artifact is any code construct that exists because the canonical implementation is not yet complete. Every artifact here represents a governance debt. Artifacts must have:

1. A unique identifier (`ARTIFACT-NNN`)
2. A precise location in the codebase
3. A description of what canonical behavior it is substituting
4. A sunset criterion — the exact condition under which it must be removed
5. A blocking dependency — what must be built before it can be removed

Artifacts without sunset criteria are **permanent violations**, not transitional artifacts.

---

## Section 1 — Active Artifacts

### ARTIFACT-001: `resolveExecutionFitness()` — Frontend Fallback Cascade

| Field | Value |
|---|---|
| **File** | [`my-chrono-sentiment-ui/src/components/CompareStrategies.js`](my-chrono-sentiment-ui/src/components/CompareStrategies.js) |
| **Type** | Semantic shim — client-side truth resolution |
| **Severity** | CRITICAL — LAW ONE violation (client computing certified state) |
| **Description** | A JavaScript function that resolves `execution_fitness` by cascading through `execution_fitness ?? fitness ?? score ?? 0`. This exists because the backend does not yet guarantee `execution_fitness` is always present in the response. |
| **Canonical Replacement** | Backend must always emit `execution_fitness` in `StrategyEvaluationDto`. The field must never be `null` or absent. |
| **Sunset Criterion** | Remove when `POST /inspect_strategy` and `POST /run_ga` both guarantee `execution_fitness` is non-null in all response paths. |
| **Blocking Dependency** | ARTIFACT-003 (StrategyEvaluationDto completeness), canonical replay DTO emission |

---

### ARTIFACT-002: `SystemState` TEMP_DTO

| Field | Value |
|---|---|
| **File** | [`services/api/src/dto.rs`](services/api/src/dto.rs) |
| **Type** | Stub DTO — placeholder for observatory state |
| **Severity** | HIGH — emits fake observatory data |
| **Description** | `SystemState` struct marked `// TEMP_DTO` that returns hardcoded or partially-populated observatory state. Does not conform to `observatory_state.schema.json`. |
| **Canonical Replacement** | A fully schema-conformant `ObservatoryStateDto` populated from the Governor service at port 8002. |
| **Sunset Criterion** | Remove when the Governor service emits canonical `observatory_state` events and the API layer proxies them without transformation. |
| **Blocking Dependency** | Governor service implementation, `GOVERNOR_TELEMETRY` event type (ARTIFACT-016) |

---

### ARTIFACT-003: `SignalsSnapshotDto` TEMP_DTO

| Field | Value |
|---|---|
| **File** | [`services/api/src/dto.rs`](services/api/src/dto.rs) |
| **Type** | Stub DTO — placeholder for signals snapshot |
| **Severity** | MEDIUM — emits incomplete signals data |
| **Description** | `SignalsSnapshotDto` struct marked `// TEMP_DTO` that does not carry the full canonical signals surface. Missing: `source_layer`, `kernel_signature`, `sequence_id` on individual signals. |
| **Canonical Replacement** | A fully schema-conformant `SignalsSnapshotDto` where each signal carries its canonical event identity fields. |
| **Sunset Criterion** | Remove when `cs-ingest` emits signals with canonical event identity and the API layer passes them through without stripping fields. |
| **Blocking Dependency** | `cs-ingest` canonical event emission, `source_layer` propagation |

---

### ARTIFACT-004: `get_strategy_store_handler` Stub

| Field | Value |
|---|---|
| **File** | [`services/api/src/handlers/strategy_handlers.rs`](services/api/src/handlers/strategy_handlers.rs) |
| **Type** | Stub handler — returns empty or hardcoded response |
| **Severity** | MEDIUM — GET /strategy_store returns no real data |
| **Description** | The `get_strategy_store_handler` function is stubbed and does not read from a real persistent strategy store. The `STRATEGY_STORE_PATH` constant is defined but the store is not populated by any write path. |
| **Canonical Replacement** | A handler that reads from a BLAKE3-signed, append-only strategy store file at `STRATEGY_STORE_PATH`. Each entry must carry `session_id`, `strategy_id`, `certification_state`, `replay_signature`. |
| **Sunset Criterion** | Remove when `POST /inspect_strategy` writes certified results to the strategy store and `GET /strategy_store` reads and returns them. |
| **Blocking Dependency** | Strategy store write path in `inspect_strategy_handler`, persistent storage format definition |

---

### ARTIFACT-005: `narrative_blocks` Client-Side Generation

| Field | Value |
|---|---|
| **File** | [`my-chrono-sentiment-ui/src/components/StrategyInspector.js`](my-chrono-sentiment-ui/src/components/StrategyInspector.js) (and related components) |
| **Type** | Logic shim — UI generating certified narrative |
| **Severity** | CRITICAL — LAW THREE violation (UI synthesizing replay narrative) |
| **Description** | The frontend contains `groupAndNarrateEvents()` or equivalent logic that groups raw events and generates human-readable narrative text. This is a Replay Engine responsibility. The UI must only render `narrative_blocks[]` emitted by the backend. |
| **Canonical Replacement** | Backend `narrative_blocks[]` array in `CanonicalInspectResponse`. The UI renders these blocks without transformation. |
| **Sunset Criterion** | Remove when `POST /inspect_strategy` always returns a non-empty `narrative_blocks[]` array and the UI renders it directly. |
| **Blocking Dependency** | Backend `narrative_blocks[]` emission (reminder #26) |

---

### ARTIFACT-006: Fake Replay Labels in UI Components

| Field | Value |
|---|---|
| **Files** | [`my-chrono-sentiment-ui/src/components/GlobalRanking.js`](my-chrono-sentiment-ui/src/components/GlobalRanking.js), [`my-chrono-sentiment-ui/src/components/CompareStrategies.js`](my-chrono-sentiment-ui/src/components/CompareStrategies.js), [`my-chrono-sentiment-ui/src/components/RunGA.js`](my-chrono-sentiment-ui/src/components/RunGA.js) |
| **Type** | Display shim — UI fabricating certification labels |
| **Severity** | HIGH — LAW TWO violation (UI claiming certified state it cannot verify) |
| **Description** | Multiple UI components display labels like "Certified", "Replay Verified", or similar without reading `certification_state` from the backend response. These labels are computed from `execution_fitness` thresholds or hardcoded. |
| **Canonical Replacement** | All certification labels must be derived exclusively from `certification_state` field in the backend response. No threshold-based label computation in the UI. |
| **Sunset Criterion** | Remove when all UI components read `certification_state` from the canonical response and render it without transformation. |
| **Blocking Dependency** | `certification_state` propagation end-to-end (reminder #27) |

---

### ARTIFACT-007: Phantom Types `UnifiedStrategyEvaluation` / `UnifiedGaResponse`

| Field | Value |
|---|---|
| **File** | [`services/api/src/dto.rs`](services/api/src/dto.rs) (formerly referenced, now removed from compilation) |
| **Type** | Phantom type — referenced in UI but not emitted by backend |
| **Severity** | HIGH — causes frontend to expect fields that don't exist |
| **Description** | The frontend JavaScript components reference a `UnifiedStrategyEvaluation` or `UnifiedGaResponse` shape that was never formally defined in the backend. The backend emits `StrategyEvaluationDto` and `RunGaResponse` respectively. The UI must be updated to consume the actual emitted shapes. |
| **Canonical Replacement** | UI components must reference `StrategyEvaluationDto` field paths as documented in `semantic_registry.md`. |
| **Sunset Criterion** | Remove when all UI components are updated to consume the canonical DTO shapes and no frontend code references the phantom type names. |
| **Blocking Dependency** | Frontend DTO alignment audit |

---

### ARTIFACT-008: Port Reference Inconsistency

| Field | Value |
|---|---|
| **Files** | Multiple — UI config, service configs, documentation |
| **Type** | Infrastructure shim — inconsistent port references |
| **Severity** | MEDIUM — causes connection failures in multi-service deployments |
| **Description** | Port 8501 appears in some configuration files and documentation as the API port. The canonical port assignment is: `services/api` → 8000, `cs-ingest` → 8001, `observatory` → 8002, UI dev → 3000. Port 8501 is retired. |
| **Canonical Replacement** | All port references must use the canonical assignments defined in `runtime_contract.md`. |
| **Sunset Criterion** | Remove when all configuration files, environment variables, and documentation reference only canonical ports and port 8501 is absent from the entire codebase. |
| **Blocking Dependency** | `runtime_contract.md` port section (reminder #23) |

---

## Section 2 — Retired Artifacts

The following artifacts have been resolved and removed:

| ID | Description | Resolved In | Resolution |
|---|---|---|---|
| RETIRED-001 | `StdRng::from_entropy()` in GA optimizer | `core/src/ga.rs` | Replaced with deterministic seed derivation |
| RETIRED-002 | `rand::thread_rng()` in centroid calculation | `core/src/ga.rs` | Replaced with `StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_1234)` |
| RETIRED-003 | `HashMap` in `asset_regime_scenarios` | `core/src/ga.rs` | Replaced with `BTreeMap` for deterministic iteration |
| RETIRED-004 | `HashMap` in `best_per_bucket` | `core/src/ga.rs` | Replaced with `BTreeMap` for deterministic iteration |
| RETIRED-005 | Hard `assert!` on fitness bounds | `services/api/src/services/evaluation_service.rs` | Replaced with `eprintln! + clamp(0.0, 1.0)` |
| RETIRED-006 | `UnifiedGaResponse` phantom type in compilation | `services/api/src/dto.rs` | Removed from compilation; frontend alignment pending |

---

## Section 3 — Pending Artifact Registration

The following artifact IDs are reserved for types not yet fully implemented (referenced in `event_taxonomy.md`):

| ID | Description | Status |
|---|---|---|
| ARTIFACT-009 | `ORDER_CANCELLED` event type — not yet implemented | Pending |
| ARTIFACT-010 | `POSITION_OPENED` event type — not yet implemented | Pending |
| ARTIFACT-011 | `POSITION_CLOSED` event type — not yet implemented | Pending |
| ARTIFACT-012 | `EQUITY_SNAPSHOT` event type — not yet implemented | Pending |
| ARTIFACT-013 | `REPLAY_SESSION_START` event type — not yet implemented | Pending |
| ARTIFACT-014 | `REPLAY_SESSION_END` event type — not yet implemented | Pending |
| ARTIFACT-015 | `CERTIFICATION_VERDICT` event type — not yet implemented | Pending |
| ARTIFACT-016 | `GOVERNOR_TELEMETRY` event type — not yet implemented | Pending |

---

## Section 4 — Artifact Lifecycle Rules

1. **Registration is mandatory.** Any stub, shim, or temporary construct added to the codebase must be registered here within the same commit.
2. **Sunset criteria are mandatory.** An artifact without a sunset criterion is a permanent violation and must be escalated to governance review.
3. **Blocking dependencies must be resolved first.** An artifact cannot be removed until all its blocking dependencies are resolved.
4. **Retirement requires verification.** An artifact is only moved to Section 2 after the canonical replacement has been implemented and tested.
5. **No artifact may be re-introduced** once retired without a new registration and governance review.