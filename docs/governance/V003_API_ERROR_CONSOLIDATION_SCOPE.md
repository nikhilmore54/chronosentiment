# V-003 — API Error Authority Consolidation Scope

**Status:** ACTIVE — bounded runtime-governance cleanup  
**Governance class:** runtime authority consolidation (broader than V-002, narrower than V-001)  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-003

---

## Intent

Remove the duplicate `ApiError` definition in `services/api/src/lib.rs:55` and route all API error construction through the Axum-aware authority in `services/api/src/errors.rs`.

This is **not** replay constitutional governance. V-003 does not touch identity law, replay substrate, certification semantics, or canonical chronology.

---

## Fracture Inventory

| Location | Type | Variants | HTTP mapping | IntoResponse |
|----------|------|----------|--------------|--------------|
| `services/api/src/lib.rs:55` | duplicate authority | `InvalidInput`, `InternalError` | none | no |
| `services/api/src/errors.rs:5` | **canonical target** | `ValidationError`, `EngineError`, `InternalError` | 400 / 422 / 500 | yes |

### Variant semantic mapping (consolidation doctrine)

| Legacy (`lib.rs`) | Canonical (`errors.rs`) | HTTP status | Notes |
|-------------------|-------------------------|-------------|-------|
| `InvalidInput` | `ValidationError` | 400 BAD_REQUEST | client / input constraint failures |
| _(absent)_ | `EngineError` | 422 UNPROCESSABLE_ENTITY | engine / scenario resolution failures |
| `InternalError` | `InternalError` | 500 INTERNAL_SERVER_ERROR | invariant / determinism / aggregation failures |

No new variants are required. `InvalidInput` is retired as a name, not preserved as a parallel variant.

---

## Consumer Classification

### Runtime (Axum) — already on `errors.rs`

These modules are compiled under `main.rs` and already import `crate::errors::ApiError`:

- `handlers/strategy_handlers.rs` — route handlers; uses `ValidationError`, `EngineError`
- `services/evaluation_service.rs` — service layer backing live routes; uses all three canonical variants

Live routes (`routes/strategy_routes.rs`) never reference `lib.rs` handlers directly.

### Library / certification-adjacent — on legacy `lib.rs::ApiError`

These modules compile under `lib.rs` and currently import `crate::ApiError`:

| Module | `InvalidInput` | `InternalError` | Wired to Axum |
|--------|----------------|-----------------|---------------|
| `simulate.rs` | yes | yes | no — library handler |
| `market_data_simulate.rs` | yes | yes | no |
| `events.rs` | yes | no | no |
| `inspector.rs` | yes | no | no |
| `timeline.rs` | no | yes | no |
| `replay.rs` | no | no (signature only) | no |
| `certify.rs` | no | no (signature only) | no — used by `examples/certify.rs` |

**Risk note:** `simulate.rs` uses a non-Axum error type today, which is why it cannot be wired as an Axum handler without consolidation. That is an integration boundary issue, not a replay-law issue.

---

## Canonical Authority

**`services/api/src/errors.rs::ApiError`** is the sole operational error authority because it owns:

- structured JSON error payloads (`ErrorMessage { message }`)
- HTTP status mapping
- Axum `IntoResponse` integration

---

## Migration Plan (bounded)

1. Register `errors` in the library root (`lib.rs`) and re-export `ApiError`.
2. Remove the duplicate enum from `lib.rs`.
3. Re-export `api::errors` from `main.rs` so binary modules keep `crate::errors::ApiError` paths stable.
4. Migrate library handler call sites: `InvalidInput` → `ValidationError`; keep `InternalError` unchanged.
5. Run `cargo check -p api`.
6. Update `AUTHORITY_MAP.md` and close V-003.

---

## Non-Goals

- Global error taxonomy reform across `chronosentiment_core` or other crates
- General API redesign or DTO changes
- Replay cohort comparison or constitutional ratification artifacts
- Adding `InvalidInput` as a permanent fourth variant (name consolidation only)
- Wiring legacy `lib.rs` handlers into Axum routes (out of scope unless explicitly requested later)

---

## Verification

| Check | Expected |
|-------|----------|
| `cargo check -p api` | pass (lib + bin) |
| Duplicate `enum ApiError` | exactly one definition in `errors.rs` |
| Library handlers | import canonical `ApiError` via `crate::errors` or re-export |
| Runtime handlers | unchanged HTTP semantics for existing variant usage |
