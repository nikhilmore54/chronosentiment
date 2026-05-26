# V-007 — Type Authority Decision (Phase B)

**Status:** DECIDED — policy binding; Phase C migration not yet authorized  
**Governance class:** Lane 1 — authority convergence (operationally authoritative, not replay constitutional)  
**Prerequisites:** `docs/governance/V007_TYPE_AUTHORITY_SCOPE.md` (Phase A inspection complete)  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-007

---

## Question

```text
Which reducer owns operational replay truth?
```

Phase A established that the fracture is **reducer authority**, not naming aesthetics. Phase B declares singular operational authority before any rename, shared extraction, adapter introduction, or DTO/internal convergence.

---

## Governance Profile (confirmed)

| Class | Burden |
|-------|--------|
| not replay constitutional | no ratification archive ceremony |
| not cosmetic | migration doctrine required |
| operationally authoritative | reducer authority must be singular |

**Lane 2:** not triggered — externally serialized, not replay-substrate-bound.

**Core asymmetry (binding):**

```text
runtime truth exists
library truth simulates
```

Operational replay truth is whatever the live Axum route emits. Library reducers may simulate or test, but may not compete as operational authority.

---

## Evidence Summary (from Phase A)

| Finding | Implication |
|---------|-------------|
| Two independent folds over `SimEvent` | Divergent replay semantics, UI truth, status transitions |
| `GET /replay/:id` wired to `EvaluationService::get_replay` | Runtime lineage already exists |
| `replay.rs::handle_replay` library-only | Simulates; not operational |
| Type shape drift (`i32` vs `u64`/`f64`, enum vs `String`) | Symptom of dual authority |
| Semantic rule drift (PnL, PartialFill, `last_sequence_id`) | Root cause — authority governance required |
| `services/ui/src/types.ts` mirrors MVP DTO JSON | External contract bound to runtime lineage |
| No chronology / frozen cohort persistence | Lane 1 proportional cleanup sufficient |

---

## Decisions

### D-1 — Canonical runtime replay reducer

**Decision:** **`EvaluationService::get_replay()`** in `services/api/src/services/evaluation_service.rs` is the **sole lawful operational replay reducer** for live API replay state.

**Output authority:** **`dto::SystemState`**, **`dto::OrderState`**, **`dto::PortfolioState`** in `services/api/src/dto.rs`.

**Route binding:** `GET /replay/:id` → `replay_handler` → `service.get_replay()` → JSON serialization of `dto::SystemState`.

**Rationale:** This is the only reducer on the production consumer graph (`services/ui` replay slider). Same-input operational truth must be attributable to a single fold.

---

### D-2 — Library reducer lineage status

**Decision:** **`replay.rs::handle_replay()` / `apply_event()`** and associated types are **DEPRECATED — test-legacy only**.

| Property | Ruling |
|----------|--------|
| Operational authority | **none** |
| New route wiring | **prohibited** |
| `lib.rs` re-export of collision types | **must be removed or renamed in Phase C** |
| Continued existence pre-Phase C | permitted only as legacy library/test surface |

**Rationale:** The library path simulates an alternate replay contract (`sim.pnl` integer parity) that is not what the UI consumes. It must not remain name-equivalent to operational types.

---

### D-3 — Semantic source of truth (reduction rules)

**Decision:** For operational replay, the following semantics from `get_replay()` are **binding**:

| Rule domain | Lawful runtime semantics |
|-------------|-------------------------|
| Order quantities | `u64` — native event quantities, no `i32` cast |
| Order price (JSON) | `f64` via `chronosentiment_core::to_real()` |
| Order status | `String` literals: `"NEW"`, `"ACTIVE"`, `"PARTIAL"`, `"FILLED"` |
| PartialFill status | Always `"PARTIAL"` on fill event; `"FILLED"` only via subsequent `OrderFilled` (or equivalent terminal transition) |
| PartialFill PnL | Signed `f64`: `multiplier * filled_qty * to_real(price)` where `multiplier` is `+1` Buy / `-1` Sell |
| Portfolio position | Signed `i64` accumulation (unchanged pattern) |
| `last_sequence_id` | **Request parameter `seq_id`**, not last-applied event sequence |
| Market events | No direct state mutation in MVP replay fold |

**Deprecated (non-operational) semantics — `replay.rs` only:**

| Rule | Deprecated behavior |
|------|---------------------|
| PartialFill PnL | `(filled_qty * price) as i64` without side multiplier |
| PartialFill terminal status | Sets `FILLED` when `quantity_remaining == 0` inline |
| `last_sequence_id` | Last applied event's `sequence_id()` |
| Order price storage | `i32` scaled integer in reducer state |

**Rationale:** UI and API clients already consume runtime semantics. Declaring library semantics as co-equal would preserve unattributable drift.

---

### D-4 — External JSON contract authority

**Decision:** External contract authority is **endpoint-scoped**, not name-unified across all replay surfaces.

| Endpoint | Authoritative JSON contract | Rust authority |
|----------|---------------------------|----------------|
| `GET /replay/:id` | MVP shape `{ orders, portfolio, last_sequence_id }` | `dto::SystemState` |
| Client mirror | `services/ui/src/types.ts` | Must track `dto::SystemState` field semantics |
| `POST /inspect_strategy` | `CanonicalInspectResponse` (+ embedded canonical portfolio) | `dto::CanonicalInspectResponse` — **separate scope** |

**Transitional classification:** MVP `dto::SystemState` for `GET /replay/:id` is **`TRANSITIONAL`** — lawful operational contract, not full `replay_response.schema.json` certification.

**Non-decision (explicit):** V-007 Phase C does **not** merge MVP `SystemState` into `CanonicalInspectResponse`. Full schema conformance remains **Lane 3**.

**Rationale:** Different endpoints serve different certification postures. Conflating them would expand V-007 beyond reducer authority into schema migration.

---

### D-5 — Allowed divergence policy

**Decision:**

| Divergence class | Permitted? |
|------------------|------------|
| Operational vs deprecated library reducer | **yes — temporarily**, until Phase C alignment |
| Operational vs `CanonicalInspectResponse` portfolio | **yes — by design** (separate endpoints) |
| Operational vs `SimulationResult.pnl` harness total | **yes — documented**; MVP fold is partial state snapshot, not full accounting authority |
| Intentional test-only alternate fold after Phase C | **no** — at most one canonical reducer module |

**Test contract migration (Phase C obligation):**

- `replay.rs::test_replay_consistency` asserting `portfolio.pnl == sim.pnl` against deprecated semantics **must be rewritten or retired** when library path is aligned or removed.
- Permitted Phase C outcome: test calls canonical reducer and asserts `dto` semantics, or test is moved to document harness-only parity explicitly outside operational authority.

---

### D-6 — Rename-only prohibition (binding)

**Decision:** **Rename-only migration is prohibited.**

Introducing `InternalOrderState`, `InternalPortfolioState`, or `InternalSystemState` **without** reducer unification or canonical delegation **does not** satisfy V-007.

**Authorized Phase C patterns (in preference order):**

1. **Extract canonical reducer** — single pure fold producing `dto::SystemState`; `get_replay()` and any remaining tests call it.
2. **Delegate deprecated path** — `handle_replay()` becomes thin wrapper over canonical reducer (optional type mapping only during transition).
3. **Remove deprecated path** — delete duplicate fold if no library consumer requires it after test migration.

**Prohibited Phase C pattern:**

- Lexical rename of `replay.rs` types while preserving independent fold logic.

**Rationale:** Phase A correctly prevented false cleanup classification. Names are secondary to reducer singularity.

---

### D-7 — Migration burden classification

**Decision:** Phase C is **bounded cleanup**, not replay constitutional migration.

| Activity | Class | Ceremony |
|----------|-------|----------|
| Extract shared reducer | authority convergence | compile + focused unit tests |
| Remove `lib.rs` collision re-exports | bounded cleanup | compile + API smoke |
| Align or retire library test | bounded cleanup | test update |
| Preserve MVP JSON shape for UI | compatibility preservation | no client break in Phase C |
| Full `replay_response.schema.json` conformance | Lane 3 | separate scope — not V-007 Phase C gate |

**No requirement for:** chronology byte fixtures, frozen cohort re-adjudication, or ratification archive.

---

### D-8 — Type name authority (post-decision)

**Decision:** Until Phase C completes:

| Name | Operational meaning |
|------|---------------------|
| `dto::SystemState` / `OrderState` / `PortfolioState` | **operational replay state** (runtime + JSON) |
| `replay.rs`同名 types | **deprecated — must not be imported as operational authority** |
| `CanonicalPortfolioState` | **inspect route canonical portfolio** — not MVP replay slider state |

After Phase C, collision names in `replay.rs` **must not** remain as public exports equating to operational types.

---

## Phase C Authorization Threshold

Phase C bounded migration may begin when this document is recorded in `AUTHORITY_MAP.md`.

**Phase C must deliver:**

1. Single canonical reducer module (location TBD in implementation — likely `services/api/src/replay.rs` rewritten as canonical fold, or new `replay_state.rs` with deprecated code removed).
2. `EvaluationService::get_replay()` calls canonical reducer — no inline duplicate fold.
3. Deprecated library fold removed or reduced to wrapper over canonical reducer.
4. `lib.rs` stops re-exporting name-colliding operational types.
5. Tests assert operational semantics or explicit non-operational harness scope.
6. `AUTHORITY_MAP.md` V-007 status updated to migration complete.

**Phase C must not deliver (without new scope doc):**

- `replay_response.schema.json` full conformance for `GET /replay/:id`
- Merge of MVP and canonical inspect response shapes
- Persistence of replay state into chronology or certification artifacts

---

## Non-Claims

This decision does **not**:

- Certify MVP `GET /replay/:id` as `replay_response.schema.json` conformant
- Resolve V-008 / V-009 / V-010
- Authorize Lane 2 capture schema migration
- Declare `SimulationResult.pnl` subordinate to MVP portfolio PnL for all accounting — only that MVP replay fold is not the harness accounting authority

---

## Artifact Discipline

| Phase | Artifact | Status |
|-------|----------|--------|
| A — inspect | `V007_TYPE_AUTHORITY_SCOPE.md` | **complete** |
| B — authority decision | this document | **complete** |
| C — bounded migration | code + tests + `AUTHORITY_MAP.md` | **complete** |

**Cadence preserved:**

```text
inspect before unify      ✓
classify before migrate   ✓
authority before extraction → Phase C
```

---

## References

- `docs/governance/V007_TYPE_AUTHORITY_SCOPE.md`
- `services/api/src/services/evaluation_service.rs:712+` — canonical reducer (operational)
- `services/api/src/replay.rs` — deprecated library fold
- `services/api/src/dto.rs:466+` — operational DTO authority
- `services/ui/src/types.ts` — client contract mirror
- `schemas/canonical/README.md` — Lane 3 conformance audit
- `docs/governance/V006_LIVE_CAPTURE_AUTHORITY_DECISION.md` — precedent for Phase B decision structure
