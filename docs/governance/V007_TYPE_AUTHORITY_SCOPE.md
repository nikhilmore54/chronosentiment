# V-007 — Type Authority Scope (Phase A Inspection)

**Status:** INSPECTION COMPLETE — Phase C migration complete  
**Governance class:** Lane 1 — authority convergence (bounded structural cleanup)  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-007  
**Cadence posture:** inspect before unify · classify before migrate

---

## Intent

Determine whether the `OrderState` / `PortfolioState` / `SystemState` naming collisions in `services/api` are merely lexical, or whether they encode divergent operational semantics — and therefore what governance burden consolidation carries.

This is **not** replay substrate constitutional governance (Lane 2). V-007 does not touch chronology bytes, manifest dialect law, or frozen cohort identity.

---

## Scope Declaration

| In scope | Out of scope |
|----------|--------------|
| Duplicate struct names in `services/api/src/replay.rs` vs `services/api/src/dto.rs` | `core/` simulation state (no matching type names) |
| Consumer graph for both lineages | Merging `CanonicalInspectResponse` into MVP `SystemState` |
| Serialization / API surfaces for MVP replay DTOs | Full `replay_response.schema.json` conformance program (Lane 3 adjacent) |
| Runtime vs persisted classification | Shared struct extraction or reducer unification |
| Replay / certification coupling check | Rename migration (`Internal*` checklist item) |

**Discipline preserved:**

```text
same name ≠ same authority
different representation ≠ governance breach
```

unless operational authority is ambiguous.

---

## Central Question — Answer

> Are the naming collisions merely lexical, or do they encode divergent operational semantics?

**Answer: both lexical collision and semantic divergence.**

The shared names suggest a single authority, but inspection finds:

1. **Two independent event reducers** implementing the same conceptual fold with different field types and different reduction rules.
2. **Only one reducer is wired to the live Axum route**; the other is library/test-only but exported via `lib.rs`.
3. **A third, separately named canonical family** (`CanonicalPortfolioState`, `CanonicalInspectResponse`) serves a different endpoint and schema target — not a name collision, but the same domain vocabulary.

Lexical cleanup alone (rename only) would reduce import confusion but would **not** resolve semantic authority until a single reducer doctrine is declared.

---

## Lineage Inventory

### Collision set (same struct names, two definition sites)

| Type | Authority A — library reducer | Authority B — API / runtime DTO |
|------|------------------------------|--------------------------------|
| `OrderState` | `services/api/src/replay.rs:16` | `services/api/src/dto.rs:466` |
| `PortfolioState` | `services/api/src/replay.rs:28` | `services/api/src/dto.rs:478` |
| `SystemState` | `services/api/src/replay.rs:34` | `services/api/src/dto.rs:484` |

**Additional related type (no name collision):**

| Type | Location | Role |
|------|----------|------|
| `OrderStatus` (enum) | `replay.rs:7` | Strongly typed status in library reducer only |
| `CanonicalPortfolioState` | `dto.rs:146` | Schema-oriented portfolio for `POST /inspect_strategy` |
| `CanonicalInspectResponse` | `dto.rs:107` | Certified inspect response shape (partial schema conformance) |

**Downstream TypeScript mirror (consumer of Authority B JSON):**

| Type | Location |
|------|----------|
| `OrderState`, `PortfolioState`, `SystemState` | `services/ui/src/types.ts:1–21` |

**Core module tree:** no definitions or imports of these three names under `core/`.

---

## Structural Comparison

| Field / aspect | `replay.rs` | `dto.rs` (MVP) |
|----------------|-------------|----------------|
| Order quantities | `i32` | `u64` |
| Order price | `i32` (scaled integer) | `f64` via `to_real()` |
| Order status | `OrderStatus` enum | `String` |
| Portfolio PnL | `i64` | `f64` |
| Portfolio position | `i64` | `i64` |
| Serde on MVP types | `Serialize, Deserialize` | `Serialize, Clone` only (Axum outbound) |
| Reduction entrypoint | `handle_replay()` / `apply_event()` | `EvaluationService::get_replay()` inline loop |

---

## Semantic Divergence (Reduction Logic)

Both paths fold `SimEvent` streams into order book + portfolio snapshots, but they are **not equivalent implementations**.

| Behavior | `replay.rs` | `evaluation_service::get_replay` |
|----------|-----------|-----------------------------------|
| PartialFill status when `remaining == 0` | Sets `FILLED` before any later `OrderFilled` | Always sets `"PARTIAL"`; relies on later `OrderFilled` for `"FILLED"` |
| PartialFill PnL | `pnl += (filled_qty * price) as i64` — integer product, no side multiplier | `pnl += multiplier * filled_qty * to_real(price)` — signed `f64` |
| PartialFill position | `position += filled_qty * multiplier` | Same pattern (aligned) |
| `last_sequence_id` | Last **applied** event's `sequence_id()` | Request parameter `seq_id` (may differ if stream ends early) |
| `OrderFilled` | Status only | Status + `quantity_remaining = 0` |
| `QueueProgression` | Updates `queue_ahead` | Updates `queue_ahead` (aligned) |
| Test contract | `test_replay_consistency` asserts `portfolio.pnl == sim.pnl` | No equivalent test on live path |

**Implication:** the library reducer optimizes for harness PnL parity with `SimulationResult.pnl`; the runtime reducer optimizes for UI-facing currency units and string statuses. Same name, different operational contracts.

---

## Consumer Graph

```text
chronosentiment_core::SimEvent stream (in-memory, post-harness)
        │
        ├─► replay.rs::handle_replay()
        │         └─► replay::SystemState
        │               consumers:
        │                 • lib.rs re-export (library tree)
        │                 • replay.rs unit test test_replay_consistency
        │               NOT wired to Axum routes
        │
        └─► EvaluationService::get_replay()  [OPERATIONAL AUTHORITY for live replay]
                  └─► dto::SystemState
                        consumers:
                          • GET /replay/:id → replay_handler (strategy_handlers.rs)
                          • services/ui App.tsx fetchSystemState()
                          • services/ui StateViewerPanel.tsx

POST /inspect_strategy  [separate authority path]
        └─► CanonicalInspectResponse + CanonicalPortfolioState
              (schema-oriented; not dto::SystemState)
```

| Consumer | Type used | Wired to production route |
|----------|-----------|---------------------------|
| `replay_handler` | `dto::SystemState` | yes — `GET /replay/:id` |
| `EvaluationService::get_replay` | `dto::SystemState` | yes — sole producer for route |
| `replay::handle_replay` | `replay::SystemState` | no — library + test only |
| `lib.rs` `pub use replay::*` | re-exports collision types | library consumers only |
| UI `types.ts` | TS mirror of `dto::SystemState` | yes — client deserialization |
| `inspect_strategy_handler` | `CanonicalInspectResponse` | yes — different schema family |

**Operational authority for live replay state:** `EvaluationService::get_replay()` → `dto::SystemState`.

**Library / test authority for reducer semantics:** `replay.rs` (currently disconnected from routes).

---

## Serialization Surfaces

| Surface | Shape | Schema target | Conformance |
|---------|-------|---------------|-------------|
| `GET /replay/:id` JSON | `{ orders, portfolio, last_sequence_id }` | `schemas/canonical/replay_response.schema.json` | **NON-CONFORMANT** — MVP subset only; documented in `schemas/canonical/README.md` |
| `POST /inspect_strategy` JSON | `CanonicalInspectResponse` | `replay_response.schema.json` + `decision_trace.schema.json` | **PARTIAL** — canonical fields present; legacy event arrays retained |
| Library `replay::SystemState` | Serde round-trip capable | none declared | internal / test only |
| UI client | `services/ui/src/types.ts` | informal mirror of MVP DTO | coupled to `dto::SystemState` JSON |

**Externally serialized (certification-sensitive):** `dto::SystemState` via Axum JSON and UI consumption.

**Not persisted:** neither lineage writes these structs to disk, chronology JSONL, frozen cohort artifacts, or observatory manifests.

---

## Runtime vs Persisted Classification

| Classification | `replay.rs` types | `dto::SystemState` | `CanonicalInspectResponse` |
|----------------|-------------------|--------------------|-----------------------------|
| Persisted replay substrate | no | no | no |
| In-memory runtime (API process) | yes (library/tests) | yes (live route) | yes (inspect route) |
| Cross-session reproducibility binding | no | no | partial (`replay_signature` field on inspect path only) |
| Client-visible contract | no | yes | yes |

---

## Replay / Certification Coupling Check

| Coupling vector | V-007 sensitivity | Notes |
|-----------------|-------------------|-------|
| `chronology_hash` / JSONL bytes | **NONE** | Types are not serialized into chronology |
| `cs-ingest` frozen cohort replay | **NONE** | No ingest pipeline references |
| `trace_replay` / observatory manifests | **NONE** | Manifest fields do not include MVP `SystemState` |
| Canonical schema certification (`replay_response`) | **INDIRECT** | MVP DTO is documented non-conformant; inspect path uses separate canonical structs |
| Determinism of API replay slider | **MODERATE** | Client reconstructs state from `GET /replay/:id`; reducer choice affects displayed PnL/status |
| Library test `sim.pnl` parity | **MODERATE** | Only enforced on `replay.rs` path, not operational path |

**Lane escalation:** V-007 remains **Lane 1**. Lane 2 replay substrate ceremony is **not required** for consolidation unless a future migration persists these shapes into certified artifacts.

Lane 3 adjacency: schema conformance for `/replay` and full certified replay responses is a separate hardening track — related vocabulary, not the same violation class as duplicate Rust definitions.

---

## Per-Type Classification

| Type family | Collision? | Classification | Governance intensity |
|-------------|------------|----------------|----------------------|
| `OrderState` | yes (`replay.rs` vs `dto.rs`) | **structurally divergent** + **externally serialized** (dto) + **semantically divergent** (reducer rules) | authority governance → then bounded cleanup |
| `PortfolioState` | yes | **structurally divergent** + **semantically divergent** (PnL arithmetic) + **externally serialized** (dto) | authority governance → then bounded cleanup |
| `SystemState` | yes | **structurally divergent** + **semantically divergent** (`last_sequence_id` semantics) + **externally serialized** (dto) | authority governance → then bounded cleanup |
| `CanonicalPortfolioState` | no (distinct name) | **externally serialized**; schema-target authority for inspect | track separately — schema convergence, not V-007 rename |
| `OrderStatus` enum | no collision | library-only adjunct | rename with library types if reducer stays |

**Summary matrix (collision set only):**

| Classification label | Applies? |
|---------------------|----------|
| alias-equivalent | **no** |
| structurally divergent | **yes** |
| semantically divergent | **yes** (duplicate reducers) |
| externally serialized | **yes** (`dto` lineage + UI) |

---

## Escalation Threshold

| Finding | Threshold crossed? | Required next artifact |
|---------|-------------------|------------------------|
| Lexical-only collision | no | — |
| Duplicate operational reducers | **yes** | Phase B: authority decision — which reducer is canonical for live replay |
| Persisted replay substrate coupling | no | Lane 2 not triggered |
| Externally serialized MVP DTO | yes | Phase B must declare JSON contract stability during any rename |
| Schema conformance gap (`replay_response`) | adjacent | Lane 3 — not blocking V-007 rename, but blocks claiming certification |

**Migration blocked until:**

1. Phase B authority decision document declares single reducer doctrine (likely: operational `get_replay` logic as authority, library path demoted or aligned).
2. Explicit policy on whether MVP `SystemState` remains transitional or is retired toward `CanonicalInspectResponse` / full schema.
3. Proof: unit test parity or documented intentional divergence between harness PnL and UI PnL.

**Prohibited without authorization:**

- Renaming only (`Internal*`) without reducer authority decision — reduces confusion but preserves semantic drift.
- Merging reducers without classification sign-off.
- Treating `CanonicalInspectResponse` consolidation as part of V-007 without separate scope.

---

## Recommended Phase B Posture (not authorized here)

Inspect findings suggest the proportional path:

1. **Declare operational authority:** `EvaluationService::get_replay` + `dto::SystemState` for `GET /replay/:id`.
2. **Demote or align library path:** either delete duplicate reducer in favor of shared function, or rename to `Internal*` and re-implement as thin wrapper calling canonical reducer.
3. **Preserve UI contract:** TypeScript types track `dto` JSON until schema migration.
4. **Defer schema certification:** full `replay_response` conformance remains Lane 3.

---

## Artifact Discipline

| Phase | Artifact | Status |
|-------|----------|--------|
| A — inspect | this document | **complete** |
| B — classify / declare policy | `V007_TYPE_AUTHORITY_DECISION.md` | **complete** |
| C — bounded migration | compile + focused tests + AUTHORITY_MAP update | **authorized — not started** |

---

## References

- `AUTHORITY_MAP.md` — V-007 ledger, Lane 1 cadence
- `services/api/src/replay.rs` — library reducer + `OrderStatus` enum
- `services/api/src/dto.rs:466+` — MVP API DTOs
- `services/api/src/services/evaluation_service.rs:712+` — operational reducer
- `services/api/src/handlers/strategy_handlers.rs:46+` — `replay_handler`
- `services/ui/src/types.ts` — client contract mirror
- `schemas/canonical/README.md` — endpoint conformance audit (`GET /replay/:id` NON-CONFORMANT)
- `docs/governance/V003_API_ERROR_CONSOLIDATION_SCOPE.md` — precedent for bounded Lane 1 scope doc
