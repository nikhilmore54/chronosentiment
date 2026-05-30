# UI API Contract v1

**Version:** 1.0  
**Status:** Frozen  
**Last updated:** 2026-05-29  
**Consumer:** `my-chrono-sentiment-ui`  
**Reference implementations:** `mock_api_server.py`, `infrastructure/observatory/api` (target)

---

## 1. Authority & Scope

This document is the **UI-facing payload specification** for the ChronoSentiment MVP frontend.

It governs **only** what the React application may depend upon at the HTTP boundary.

| In scope | Out of scope |
|----------|--------------|
| Request bodies the UI sends | Internal Rust DTOs |
| Response shapes the UI reads | Domain model definitions |
| Required vs optional fields | Replay substrate semantics |
| Wire-format → UI normalization rules | `CanonicalInspectResponse` internal authority surfaces |

### Governance alignment

Per `Service_Boundary_Definition.md` and `AUTHORITY_MAP.md`:

- **Authority changes occur at explicit boundaries**, not by modifying consumers.
- Certification state, narrative blocks, and comparison metadata are **observational surfaces**. Backends emit them; the UI displays them. The UI must not synthesize authoritative meaning (Law One in the frontend codebase).
- This contract is a **projection layer**. Domain and replay models may evolve independently as long as they satisfy this boundary.

### What this document is not

- Not a freeze on React component implementation.
- Not a freeze on mock-server randomness or seed behavior.
- Not a substitute for replay certification schemas under `/docs` and `AUTHORITY_MAP.md`.

---

## 2. Compatibility Rules

### Version identifier

All conforming backends SHOULD expose a version marker via `GET /health` (informational only; not required for UI operation).

### Field rules

| Rule | Meaning |
|------|---------|
| **Required** | MUST be present with the declared type. Absence breaks UI workflow. |
| **Optional** | MAY be omitted. UI renders `—`, empty states, or hides sections. |
| **Deprecated** | Still tolerated during migration; MUST NOT be emitted by new implementations. |
| **Forbidden** | MUST NOT appear in v1 responses (e.g. legacy aliases listed under deprecations). |

### Envelope preference

Prefer **object envelopes** over bare arrays. Envelopes carry metadata (totals, provenance, certification) without breaking consumers.

### Type conventions

- Wire format: **snake_case** JSON (HTTP bodies).
- Inspect narrative blocks: normalized to **camelCase** inside the UI by `normalizeInspectResponse()` (see §6). Backends MUST emit snake_case.

---

## 3. Transport & Configuration

### Base URL

All UI fetch calls MUST route through a single configurable base URL:

```env
REACT_APP_API_BASE_URL=http://localhost:8000
```

Default when unset: `http://localhost:8000`.

> **Infrastructure status (v1):** env-based routing is implemented via `src/config/api.js` (base URL) and `src/services/api.js` (`apiUrl`). Copy `.env.example` to `.env` to override the default.

### CORS

Backends MUST allow the UI origin (`http://localhost:3000` in development).

### Error responses

On non-2xx responses, backends SHOULD return JSON:

```json
{ "message": "Human-readable error", "error": "MACHINE_CODE" }
```

The UI reads `message` or `error` when present.

---

## 4. Shared Types

### 4.1 `StrategyEvaluation`

Used in ranking tables, GA results, and comparison rows.

| Field | Type | Required | UI usage |
|-------|------|----------|----------|
| `strategy_id` | string | **Yes** | Row key, navigation to inspect |
| `execution_fitness` | number | **Yes** | Primary ranking metric |
| `ga_fitness` | number | No | Secondary column; `—` if absent |
| `avg` | number | No | PnL display |
| `std` | number | No | Volatility display |
| `sharpe` | number | No | Comparison metric aggregates |
| `max_drawdown` | number | No | Reserved |
| `fill_rate` | number | No | Comparison metric aggregates |
| `slippage` | number | No | Comparison metric aggregates |
| `certification_state` | string | No | Badge when present |
| `certification_reason` | string | No | Tooltip / subtitle |
| `classification` | string | No | Global ranking badge |

**`execution_fitness` invariant:** MUST be a finite number. The UI no longer applies fallback resolution (ARTIFACT-001 removed).

### 4.2 `GenerationHistoryEntry`

Extends `StrategyEvaluation` semantics for GA history rows.

| Field | Type | Required |
|-------|------|----------|
| `generation` | integer | No |
| `ga_fitness` | number | No |
| `execution_fitness` | number | No |
| `avg` | number | No |
| `strategy_id` | string | No |

### 4.3 `Signal`

| Field | Type | Required | UI usage |
|-------|------|----------|----------|
| `asset` | string | **Yes** | Grouping key |
| `action` | `"BUY"` \| `"SELL"` \| `"HOLD"` | **Yes** | Filter + badge |
| `confidence` | number | No | Sorting, strength |
| `composite_score` | number | No | Strong/weak classification |
| `strategy_id` | string | No | Row identity |
| `entry_zone` | `[number, number]` | No | Zone display |
| `target` | number | No | Target display |
| `stop_loss` | number | No | Stop display |
| `scenario_pnl` | number | No | Asset rollup |

---

## 5. Shell Endpoints

These support the application chrome (connectivity, system status). They are not workflow-critical but SHOULD be implemented for parity.

### 5.1 `GET /health`

**Response — all optional:**

```json
{
  "status": "online",
  "system_phase": "LIVE",
  "throttle_state": "OPEN",
  "cohort_id": "cohort-2026-A"
}
```

Used as fallback when `/observatory` is unavailable.

### 5.2 `GET /observatory`

**Response:**

| Field | Type | Required |
|-------|------|----------|
| `snapshot_sequence_id` | integer | No |
| `system_phase` | string | No |
| `governor_state.throttle_state` | string | No |
| `governor_state.cohort_id` | string | No |
| `governor_state.active_cohort_size` | integer | No |
| `governor_state.governor_version` | string | No |
| `kernel_state.queue_depth` | integer | No |
| `kernel_state.fill_latency_ns` | integer | No |
| `kernel_state.sync_ratio` | number | No |
| `kernel_state.events_per_second` | integer | No |
| `kernel_state.kernel_version` | string | No |

---

## 6. Workflow Endpoints

### 6.1 `GET /run_ga`

Triggers GA execution. UI currently uses GET; POST is also supported by the mock for parameterized runs.

#### Response

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `global_best` | `StrategyEvaluation` | **Yes** | Verdict hero metric |
| `global_best_generation` | integer | No | Peak generation index |
| `generation_found` | integer | No | Alias tolerated by `normalizeGaResult()` |
| `final_generation_best` | `GenerationHistoryEntry` | No | |
| `final_gen_best` | `GenerationHistoryEntry` | No | Deprecated alias |
| `generation_history` | `GenerationHistoryEntry[]` | No | |
| `results` | `StrategyEvaluation[]` | No | Legacy fallback for `global_best` |
| `total_generations` | integer | No | |
| `seed` | integer | No | |

#### UI normalization (`normalizeGaResult`)

The UI merges aliases before render. Backends SHOULD emit canonical names:

- Prefer `global_best_generation` over `generation_found`
- Prefer `final_generation_best` over `final_gen_best`
- Prefer `global_best` over bare `results[0]`

---

### 6.2 `GET /signals/latest`

Fetched immediately after GA completion.

#### Response

| Field | Type | Required |
|-------|------|----------|
| `signals` | `Signal[]` | **Yes** |
| `snapshot_ts` | string (ISO-8601) | No |
| `total` | integer | No |

---

### 6.3 `GET /ga/strategy-store`

Loaded on Run GA panel mount and refreshed after GA completion.

#### Response

| Field | Type | Required |
|-------|------|----------|
| `strategies` | `StrategyEvaluation[]` | **Yes** |
| `store_version` | string | No |
| `total` | integer | No |

---

### 6.4 `GET /ga/global-ranking`

#### Canonical response envelope

```json
{
  "rankings": [],
  "total": 0
}
```

| Field | Type | Required |
|-------|------|----------|
| `rankings` | `RankingRow[]` | **Yes** |
| `total` | integer | **Yes** |

#### `RankingRow`

| Field | Type | Required | UI usage |
|-------|------|----------|----------|
| `strategy_id` | string | **Yes** | Row key |
| `execution_fitness` | number | **Yes** | Sort + display |
| `ga_fitness` | number | No | |
| `avg` | number | No | |
| `std` | number | No | |
| `classification` | string | No | Badge |
| `rank` | integer | No | Informational; UI re-sorts by `execution_fitness` |

#### Deprecations

| Shape | Status |
|-------|--------|
| Bare `RankingRow[]` at root | **Deprecated** — UI tolerates during migration |
| Root key `ranking` (singular) | **Deprecated** — UI tolerates as fallback |

New implementations MUST use `{ "rankings": [...], "total": N }`.

---

### 6.5 `POST /compare_strategies`

#### Request

```json
{
  "strategies": [
    { "strategy_config": { "strategy_id": "strat_200_5_4_2" } }
  ],
  "scenarios": [],
  "seed": 42
}
```

| Field | Type | Required |
|-------|------|----------|
| `strategies` | `{ strategy_config: object }[]` | **Yes** (min 2 for UI validation) |
| `strategies[].strategy_config.strategy_id` | string | **Yes** |
| `scenarios` | array | No |
| `seed` | integer | No |

#### Response

| Field | Type | Required |
|-------|------|----------|
| `ranking` | `StrategyEvaluation[]` | **Yes** |
| `comparison_summary` | object | **Yes** |
| `comparison_summary.reason` | string | **Yes** |
| `seed` | integer | No |

#### `comparison_summary` optional fields

| Field | Type | UI behavior when present |
|-------|------|--------------------------|
| `replay_certified` | boolean | Certification header |
| `replay_integrity` | string | Certification row |
| `timestamp_cohesion` | string | Certification row |
| `sync_state` | string | Certification row |
| `governor_action` | string | Certification row |
| `metrics` | `ComparisonMetric[]` | Expected vs observed table |
| `best_strategy` | string | Reserved; not currently rendered |

#### `ComparisonMetric`

| Field | Type | Required |
|-------|------|----------|
| `key` | string | **Yes** |
| `expected` | number | **Yes** |
| `observed` | number | **Yes** |
| `diverged` | boolean | No |

#### Deprecations

| Field | Status |
|-------|--------|
| `results` (instead of `ranking`) | **Forbidden** |
| `analytical_conclusion` (instead of `comparison_summary.reason`) | **Forbidden** |

---

### 6.6 `POST /inspect_strategy`

#### Request

```json
{
  "strategy_id": "strat_200_5_4_2",
  "seed": 42
}
```

| Field | Type | Required |
|-------|------|----------|
| `strategy_id` | string | **Yes** |
| `seed` | integer | No (default: 42) |

---

#### Wire response (backend emission)

Backends emit **snake_case**. The UI applies `normalizeInspectResponse()` before consumption.

##### Required wire fields

| Field | Type |
|-------|------|
| `strategy_id` | string |
| `narrative_blocks` | `NarrativeBlockWire[]` |
| `execution_trace` | `TraceEventWire[]` |

##### Optional wire fields

| Field | Type | UI usage |
|-------|------|----------|
| `seed` | integer | Context display |
| `certification_state` | string | Badge (`CERTIFIED`, `DEGRADED`, `PARTIAL`, `INVALID`) |
| `certification_reason` | string | Badge subtitle |
| `decision_trace` | `TraceEventWire[]` | Raw trace panel |
| `event_sequence` | `TraceEventWire[]` | Raw trace panel |
| `execution_fitness` | number | Reserved |
| `ga_fitness` | number | Reserved |
| `verdict` | string | Reserved |
| `confidence` | string | Reserved |
| `avg`, `std`, `sharpe`, `max_drawdown`, `fill_rate`, `slippage` | number | Reserved |
| `metrics` | object | Optional; `metrics.total_trades` rendered when present |

##### `NarrativeBlockWire`

| Field | Type | Required |
|-------|------|----------|
| `sequence_id` | integer | **Yes** |
| `group` | string | **Yes** |
| `narrative` | string | **Yes** |
| `parent_sequence_id` | integer \| null | No |
| `block_type` | string | No |
| `timestamp_ns` | integer | No |
| `is_key_event` | boolean | No |
| `key_event_marker` | string | No |
| `divergence_score` | number \| null | No |

##### `TraceEventWire`

| Field | Type | Required |
|-------|------|----------|
| `sequence_id` | integer | **Yes** (replay slider bounds) |
| `type` | string | No |
| `timestamp_ns` | integer | No |
| `payload` | object | No (merged into event by normalizer) |

---

#### Canonical UI model (post-normalization)

This is the shape components consume after `normalizeInspectResponse()`. **Contract tests SHOULD validate that wire responses normalize into this model.**

##### Top-level UI model

| Field | Type | Required after normalization |
|-------|------|------------------------------|
| `strategy_id` | string | **Yes** |
| `narrative_blocks` | `NarrativeBlockUI[]` | **Yes** (may be empty) |
| `execution_trace` | `TraceEventUI[]` | **Yes** (may be empty) |
| `certification_state` | string | No |
| `certification_reason` | string | No |
| `decision_trace` | `TraceEventUI[]` | No |
| `event_sequence` | `TraceEventUI[]` | No |
| `metrics` | object | No |

##### `NarrativeBlockUI`

Produced by `normalizeNarrativeBlock()`:

| Field | Type | Required | Source wire field |
|-------|------|----------|-------------------|
| `id` | integer | **Yes** | `sequence_id` |
| `group` | string | **Yes** | `group` |
| `narrative` | string | **Yes** | `narrative` |
| `parentId` | integer \| null | No | `parent_sequence_id` |
| `blockType` | string | No | `block_type` |
| `timestamp_ns` | integer | No | `timestamp_ns` |
| `isKeyEvent` | boolean | No | `is_key_event` |
| `keyEventMarker` | string | No | `key_event_marker` |
| `divergenceScore` | number \| null | No | `divergence_score` |

##### `TraceEventUI`

Produced by `normalizeTraceEvent()`:

| Field | Type | Notes |
|-------|------|-------|
| `sequence_id` | integer | Required for slider |
| `type` | string | |
| `timestamp_ns` | integer | |
| `...payload fields` | any | Flattened from `payload` |

##### UI invariants

1. **`narrative_blocks` authority:** The UI MUST NOT synthesize narrative text. Blocks MUST originate from the backend.
2. **`execution_trace` bounds:** Replay slider uses `min(sequence_id)` / `max(sequence_id)` from `execution_trace`.
3. **Dual-mode comparison:** Divergence analysis is derived client-side from two normalized inspect responses (ARTIFACT-010). Backends are not required to emit `divergence_analysis[]` in v1.

---

## 7. Endpoint Summary

### UI endpoint inventory

All network calls route through `src/services/api.js` → `apiUrl(path)`.

| Component | Method | Endpoint |
|-----------|--------|----------|
| App | GET | `/observatory` |
| App | GET | `/health` |
| RunGA | GET | `/run_ga` |
| RunGA (signals panel) | GET | `/signals/latest` |
| RunGA (strategy store) | GET | `/ga/strategy-store` |
| StrategyInspector | POST | `/inspect_strategy` |
| CompareStrategies | POST | `/compare_strategies` |
| GlobalRanking | GET | `/ga/global-ranking` |

### Workflow panels

| Method | Path | Workflow panel |
|--------|------|----------------|
| GET | `/run_ga` | Run GA |
| GET | `/signals/latest` | Run GA (post-execution) |
| GET | `/ga/strategy-store` | Run GA |
| POST | `/inspect_strategy` | Inspect Strategy |
| POST | `/compare_strategies` | Compare Strategies |
| GET | `/ga/global-ranking` | Global Ranking |
| GET | `/health` | App shell |
| GET | `/observatory` | App shell |

### Contract fixtures & tests

**Fixtures** (`fixtures/contracts/`): canonical minimum payloads derived from this document, not from mock responses.

| Fixture | Purpose |
|---------|---------|
| `manifest.json` | Contract version + endpoint inventory |
| `run_ga.response.json` | Required GA fields |
| `inspect_strategy.response.json` | Wire-format inspect minimum |
| `compare_strategies.response.json` | Required compare fields only |
| `compare_strategies.response.extended.json` | Optional certification/metrics surfaces |
| `global_ranking.response.json` | Envelope `{ rankings, total }` |

**Tests** (`tests/test_ui_api_contract.py`):

```bash
# Offline: fixture + transport authority checks
pytest tests/test_ui_api_contract.py -q

# Live: validate mock or Rust API at the same URL
UI_CONTRACT_TEST_API_URL=http://localhost:8000 pytest tests/test_ui_api_contract.py -q
```

Backends MAY emit `X-Contract-Version: 1.0`; tests validate when present.

---

## 8. Implementation Checklist

Use this sequence for backend replacement (see project governance docs):

| Step | Deliverable | Status |
|------|-------------|--------|
| 1 | This document (`UI_API_CONTRACT_v1.md`) | **Done** |
| 2 | `REACT_APP_API_BASE_URL` in all UI fetch calls | **Done** |
| 3 | JSON fixtures under `fixtures/contracts/` | **Done** |
| 4 | Contract tests: Rust/mock response ≡ fixture shape | **Done** |
| 5 | First real endpoint: `GET /ga/global-ranking` | **Done** (transport E2E — all four workspaces) |
| 6 | UI verified against mock and Rust with no component changes | **Done** (browser pass — `artifacts/browser_pass6/`, 2026-05-30) |

---

## 9. Relationship to Internal Models

```text
Domain / Replay Layer          UI Contract Layer           Frontend
─────────────────────          ─────────────────           ────────
CanonicalInspectResponse  →    POST /inspect_strategy  →   normalizeInspectResponse()
CandidateEvaluationDto    →    StrategyEvaluation      →   direct field access
GaResult                  →    GET /run_ga             →   normalizeGaResult()
```

Internal Rust types MAY carry additional fields. They MUST project into this contract at the HTTP boundary. The UI MUST NOT depend on fields not listed here.

---

## 10. Change Control

Changes to required fields or forbidden deprecations require:

1. A new contract version (`UI_API_CONTRACT_v2.md`)
2. Updated fixtures and contract tests
3. Explicit note in `AUTHORITY_MAP.md` if observational authority surfaces change

Optional field additions are backward-compatible within v1 and do not require a version bump.
