# Scenario Domain Contract v1

**Version:** 1.0  
**Status:** Frozen (pre-implementation)  
**Last updated:** 2026-05-29  
**Authority:** Evaluation substrate specification  
**Companion:** `docs/contracts/UI_API_CONTRACT_v1.md`

---

## 1. Purpose & Authority

This document defines what constitutes a **valid scenario domain** for strategy evaluation, comparison, ranking, and inspect selection in ChronoSentiment.

It governs **evaluation authority** — not API shape, not UI projection, not narrative text.

| In scope | Out of scope |
|----------|--------------|
| Scenario identity and eligibility | UI payload fields |
| Substrate provenance | Narrative projection templates |
| Chronology / replay guarantees per domain | Internal Rust DTO names |
| Multi-scenario aggregation rules | Attestation mechanics → `REPLAY_ATTESTATION_CONTRACT_v1.md` |
| Inspect scenario selection | GA mutation operators |

### Why this contract exists (Phase C rationale)

Phase B established:

```text
Strategy → Simulation Harness → SimEvent → Inspect
```

within a **single substrate**. That substrate (`deterministic_demo_fixture`) currently functions as both:

1. **Simulation input** (legitimate)
2. **Evaluation authority** (hidden — must be separated)

Phase C is **authority correction**, not a robustness feature. Without a frozen scenario contract, a CSV file or fixture helper becomes a de facto scenario definition — the same failure mode `UI_API_CONTRACT_v1.md` eliminated for mock payloads.

Per `AUTHORITY_MAP.md` and `Service_Boundary_Definition.md`:

> Observational surfaces verify replay law; they do not redefine replay law.

Scenario domains declare **where** evaluation may occur. They do not define strategy quality by themselves.

### The recurring authority pattern (governance architecture)

Phase C applies the same loop that succeeded for UI API, transport, inspect, and evaluation:

```text
Hidden Authority
       ↓
Authority Identification
       ↓
Contract Freeze          ← this document
       ↓
Projection Boundary      ← ScenarioResult[] → aggregation → ranking
       ↓
Implementation
       ↓
Observability            ← every ranking traceable to ScenarioResult[]
```

| Surface | Hidden authority removed |
|---------|--------------------------|
| UI API | Mock payload shape |
| Transport | Hardcoded endpoint selection |
| Inspect | Narrative-first explanation (`SimEvent[]` became authoritative) |
| Evaluation | Single fixture authority (Phase C target) |

**Observational shift (Phase B → Phase C):**

| Before | After |
|--------|-------|
| Narrative authoritative | `SimEvent[]` authoritative |
| `fitness` authoritative | `ScenarioResult[]` authoritative |

Ranking, comparison, and aggregated fitness are **projections** — not primary observables.

---

## 2. Core Types

### 2.1 `ScenarioDomain`

A scenario domain is a **bounded, replay-eligible chronology window** with explicit provenance.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scenario_id` | string | **Yes** | Stable identifier (see §3) |
| `substrate_source` | `SubstrateSource` | **Yes** | Provenance descriptor (not a raw path) |
| `asset` | string | No | Primary instrument symbol |
| `chronology` | `ChronologyGuarantee` | **Yes** | Ordering and bounds |
| `replay` | `ReplayGuarantee` | **Yes** | Determinism expectations |
| `evaluation` | `EvaluationEligibility` | **Yes** | Whether domain may score strategies |
| `domain_class` | `DomainClass` | **Yes** | Authority classification |

#### `DomainClass`

| Value | Meaning |
|-------|---------|
| `CERTIFIED_FIXTURE` | Harness-defined, replay-tested substrate (e.g. `deterministic_demo`) |
| `HISTORICAL_SLICE` | Derived from ingested chronology with attested bounds |
| `SYNTHETIC_REGIME` | Generated substrate with declared seed and regime tag |
| `HOLDOUT` | Eligible for evaluation; excluded from GA training selection |

**Rule:** `HOLDOUT` domains MUST NOT contribute to GA fitness selection unless explicitly configured for holdout reporting only.

### 2.1.1 `ScenarioRegistry` — authority, not policy

Phase C v1 registry MUST be **declarative**, not intelligent.

**Target (v1):**

```rust
struct ScenarioDomain {
    id: String,
    substrate_source: SubstrateSource,
    chronology_guarantee: ChronologyGuarantee,
    replay_guarantee: ReplayGuarantee,
    evaluation_eligible: bool,
}
```

**Forbidden in Phase C v1:**

```rust
struct SmartScenarioEngine { /* policy, heuristics, auto-selection */ }
```

The registry records **which domains exist and their guarantees**. It does not:

- auto-pick "best" scenarios for ranking
- infer domain quality from CSV metadata
- embed aggregation policy

Policy belongs in explicit, documented aggregation reducers applied **after** `ScenarioResult[]` materializes.

#### Registry: declaration vs policy

| Allowed (authority) | Forbidden (policy) |
|---------------------|-------------------|
| `registry.register(domain)` | `registry.select_best_domains(...)` |
| `registry.get(id)` | `registry.auto_exclude_underperformers(...)` |
| `registry.list_eligible()` | Heuristic domain selection inside registry |

The registry **declares** which domains exist. It does **not** decide which domains matter for a given strategy.

---

### 2.2 `SubstrateSource`

Prevents `"CSV file"` from becoming implicit scenario authority.

| Field | Type | Required |
|-------|------|----------|
| `kind` | `"fixture"` \| `"historical"` \| `"synthetic"` | **Yes** |
| `reference` | string | **Yes** | Logical reference (not bare filesystem path in API responses) |
| `version` | string | No | Substrate version or ingest batch id |
| `ingest_hash` | string | No | Content hash when available (Phase D input) |

**Examples (valid):**

```json
{ "kind": "fixture", "reference": "deterministic_demo_v1" }
{ "kind": "historical", "reference": "BTCUSDT_csv_window_3", "version": "2026-05-29", "ingest_hash": "abc123..." }
```

**Forbidden as scenario definition:**

- Unlabeled filesystem paths in consumer-facing metadata
- Ad-hoc candle arrays constructed inside API handlers
- Mock-random market sequences

---

### 2.3 `ChronologyGuarantee`

| Field | Type | Required |
|-------|------|----------|
| `start_ts` | integer | **Yes** |
| `end_ts` | integer | **Yes** |
| `event_count` | integer | **Yes** |
| `monotonic` | boolean | **Yes** | Timestamps strictly non-decreasing |
| `gap_policy` | `"FAIL"` \| `"MARK_DEGRADED"` | **Yes** |

**Invariant:** If `monotonic == false` or gap policy triggers, domain MUST be marked ineligible or `replay_status = INVALID`.

---

### 2.4 `ReplayGuarantee`

| Field | Type | Required |
|-------|------|----------|
| `deterministic` | boolean | **Yes** |
| `engine_mode` | `"IDEAL"` \| `"REAL"` | **Yes** |
| `replay_id` | string | No | Logical replay session identifier |
| `expected_event_hash` | string | No | Phase D — populated after attestation |

**Phase B baseline:** `deterministic_demo` uses `engine_mode = REAL`, `deterministic = true`.

**Phase D extension:** `expected_event_hash` becomes certification input; not required in Phase C.

---

### 2.5 `EvaluationEligibility`

| Field | Type | Required |
|-------|------|----------|
| `eligible` | boolean | **Yes** |
| `reason` | string | No | Required when `eligible == false` |
| `min_events` | integer | No | Minimum market events for scoring |

---

## 3. Scenario Identifier Convention

Scenario IDs MUST be stable, parseable, and not derived at runtime from opaque paths.

### Canonical patterns

| Pattern | Example | Source |
|---------|---------|--------|
| `{fixture_name}` | `deterministic_demo` | Certified harness fixture |
| `{asset}_csv_window_{n}` | `BTCUSDT_csv_window_3` | `scenarios_from_candles()` |
| `{asset}_{regime}_{seed}` | `BTCUSDT_trend_42` | Synthetic regime generator |

**Rules:**

1. IDs are lowercase-safe strings; use `_` separators.
2. Window index `n` is zero-based and stable for a given ingest version.
3. Re-ingest that changes chronology bounds MUST bump `substrate_source.version` or issue new IDs.

---

## 4. Evaluation Flow (Phase C Target)

### 4.1 Multi-scenario evaluation

```text
Strategy
   ↓
ScenarioRegistry.get_eligible_domains()
   ↓
For each ScenarioDomain:
   run simulation / evaluation
   ↓
ScenarioResult[]
   ↓
AggregationLayer
   ↓
Ranking / Compare response
```

### 4.2 Inspect (unchanged authority)

Inspect remains **causal and local**:

```text
selected_scenario_id
   ↓
Simulation Harness (that domain's substrate)
   ↓
SimEvent[]
   ↓
Projection (EventWrapper, narrative)
   ↓
Inspect Response
```

**Rule:** Inspect MUST NOT aggregate across scenarios. Aggregation belongs to ranking/compare only.

### 4.3 Scenario selection for inspect

| Source | Precedence |
|--------|------------|
| Request `scenarios[0]` | Highest |
| Scenario embedded in `strategy_id` | Second |
| Explicit default (`deterministic_demo`) | Third |
| First eligible domain from registry | Fourth (implementation) |

Current Phase B default: `deterministic_demo` — valid `CERTIFIED_FIXTURE` domain.

### 4.4 Evaluation Domain vs Inspection Domain

These concepts sound similar but serve different purposes. They MUST remain separate.

#### Evaluation Domain

**Where a strategy is tested** — used for compare, ranking, GA robustness.

Examples:

```text
deterministic_demo
historical_slice_2025_01
historical_slice_2025_02
```

Materializes as entries in `ScenarioResult[]` across all eligible evaluation domains.

#### Inspection Domain

**Which trace is being viewed** — used for causal replay projection only.

Examples:

```text
primary_scenario      ← request scenarios[0] or default
worst_case_scenario   ← future: inspect selector over ScenarioResult[]
best_case_scenario    ← future: inspect selector over ScenarioResult[]
```

Inspect selects **one** inspection domain and projects its `SimEvent[]` trace. It does not aggregate across evaluation domains.

| Concept | Question answered | Aggregates? |
|---------|-------------------|-------------|
| Evaluation Domain | Where was the strategy tested? | Yes (via aggregation layer) |
| Inspection Domain | Which execution path am I viewing? | No |

---

## 5. `ScenarioResult` (Observational Substrate)

`Vec<ScenarioResult>` is the **primary Phase C artifact** — the new observational substrate.

Before Phase C, `fitness` is often treated as the observable. After Phase C:

```text
ScenarioResult[]   ← observable (authoritative per domain)
        ↓
Aggregation          ← projection
        ↓
Ranking              ← projection
```

**Forbidden flow:**

```text
Ranking → inference about scenarios
```

Everything except `ScenarioResult[]` is projection. Rankings MUST be derivable from per-domain results without reverse inference.

Before collapsing to a single score, evaluation MUST materialize per-domain results.

```rust
ScenarioResult {
    scenario_id: String,
    fitness: f64,
    execution_fitness: f64,
    avg_pnl: f64,
    std_dev: f64,
    max_drawdown: f64,
    trade_count: usize,
    replay_status: ReplayStatus,  // VALID | DEGRADED | INVALID | SKIPPED
    domain_class: DomainClass,
    // Phase D — see REPLAY_ATTESTATION_CONTRACT_v1.md
    attestation: AttestationRecord,
}
```

`AttestationRecord` carries `expected_event_hash`, `result_hash`, `event_count`, `substrate_reference`, and `attestation_timestamp` for divergence diagnosis.

### `ReplayStatus`

| Value | Meaning |
|-------|---------|
| `VALID` | Evaluation completed; replay guarantees satisfied |
| `DEGRADED` | Evaluation completed with chronology/replay warnings |
| `INVALID` | Domain ineligible or replay failed |
| `SKIPPED` | Domain excluded by policy (e.g. holdout-only) |

### Aggregation rule (observability first)

**Forbidden in Phase C v1:**

```text
aggregated_score = average(fitness)
```

(with no exposed per-domain breakdown)

**Required pattern:**

```text
scenario_results[]  →  exposed internally / in debug surfaces
aggregated_score    →  derived explicitly from scenario_results[]
```

Suggested derived fields (optional in API v1):

| Field | Derivation |
|-------|------------|
| `aggregated_score` | Configured reducer (mean, min, weighted) — MUST be documented |
| `worst_case_fitness` | `min(scenario_results.fitness)` |
| `domain_consistency` | `std(scenario_results.fitness)` |
| `domains_evaluated` | `count(replay_status == VALID)` |

**Robustness principle (cursorrules):** High fitness requires consistent performance across diverse domains — not peak performance on one fixture.

### 5.1 `ScenarioAggregator` — future contract boundary

Once `ScenarioResult[]` exists, a new hidden authority surface tends to emerge: **the aggregation rule**.

Initially harmless:

```text
average(fitness)
```

Over time it can accumulate weights, filters, exclusions, bonuses, and penalties that become more influential than the scenarios themselves.

**Phase C v1 requirement:** keep aggregation in an isolated module (`ScenarioAggregator` or equivalent) — not embedded in registry, handlers, or UI.

**Future (when authority concentrates):**

```text
Aggregator Contract
       ↓
Projection Boundary
       ↓
Implementation
```

Same governance loop as UI API and scenario domains. Do not pre-implement the contract now; **do** isolate the code path so the contract can be frozen later without archaeology.

---

## 6. Registered Domains (v1)

| scenario_id | domain_class | substrate_source | inspect_default | compare/rank |
|-------------|--------------|------------------|-----------------|--------------|
| `deterministic_demo` | `CERTIFIED_FIXTURE` | `fixture:deterministic_demo_v1` | **Yes** | **Yes** (Phase C) |

### Planned (Phase C implementation)

| scenario_id pattern | domain_class | Notes |
|---------------------|--------------|-------|
| `{asset}_csv_window_{n}` | `HISTORICAL_SLICE` | From `scenarios_from_candles()` |
| `{asset}_{regime}_{seed}` | `SYNTHETIC_REGIME` | Seed-declared synthetic domains |

New domains MUST be registered here or in `fixtures/contracts/scenario_registry.json` before use in ranking/compare.

---

## 7. Relationship to Other Contracts

```text
UI_API_CONTRACT_v1              ← HTTP shapes the UI consumes
SCENARIO_DOMAIN_CONTRACT_v1     ← What domains evaluation may use
REPLAY_ATTESTATION_CONTRACT_v1  ← Digests, levels, verification (Phase D)
AUTHORITY_MAP.md                ← Who may define meaning
```

```text
Substrate Layer          Scenario Contract        Attestation Contract   API / UI
────────────────         ─────────────────        ────────────────────   ────────
deterministic_demo  →    ScenarioDomain      →    event_hash/result  →   (internal)
CSV ingest          →    ScenarioDomain      →    AttestationRecord  →   Ranking (projection)
SimEvent log        →    ReplayGuarantee     →    Level 0–4          →   Inspect (observational)
```

---

## 8. Phase Roadmap Alignment

| Phase | Deliverable | Status |
|-------|-------------|--------|
| A | UI + evaluation spine | **Done** |
| B | Replay-backed inspect (single domain) | **Done** |
| C | Scenario authority separation | **Done** (v1 registry + ScenarioResult[] + isolated aggregator) |
| D | Replay hash attestation → certification | **Contract frozen** — see `REPLAY_ATTESTATION_CONTRACT_v1.md` |

### Why Phase C precedes Phase D

Attestation within one fixture proves:

> this strategy replayed correctly once.

Attestation across registered domains proves:

> this strategy was replay-valid across a defined scenario domain.

Phase D certification MUST reference `ScenarioDomain` entries, not an anonymous event log.

---

## 9. Implementation Checklist (Phase C)

| Step | Deliverable | Status |
|------|-------------|--------|
| 1 | This document (`SCENARIO_DOMAIN_CONTRACT_v1.md`) | Done |
| 2 | `ScenarioRegistry` (in-memory v1; file-backed registry later) | Done |
| 3 | Register `deterministic_demo` + first historical slice set | Done (v1: `deterministic_demo_execution`) |
| 4 | Evaluate compare/rank across `scenario_results[]` | Done |
| 5 | Expose aggregation metadata without breaking UI contract | Done (comparison reason + internal `last_scenario_results`) |
| 6 | Inspect: accept `scenarios[0]` as domain selector (already wired) | Done |
| 7 | Contract tests: eligible domain count, no orphan CSV authority | Done (`tests/test_scenario_domain_contract.py`) |

### Phase C success criterion

Phase C succeeds when **both** statements are true:

1. **Authority correction:** No individual substrate can determine a strategy's standing by itself.

2. **Observability:** Every ranking can be traced back to explicit `ScenarioResult[]` materialization — the Phase C equivalent of Phase B's inspect traceability.

Do **not** measure success by:

- better rankings
- more realistic PnL outcomes

Measure success by **governance**: authority is declared, materialized, projected, and auditable.

### Phase D strengthening (post Phase C)

Today, a replay hash certifies:

> this execution path occurred.

After Phase C, `ScenarioResult[]` + per-domain replay hashes can certify:

> this standing emerged from declared domains with replay-valid traces.

That is the certification surface Phase D should target — not single-run attestation alone.

---

## 10. Change Control

Changes to required fields, identifier conventions, or aggregation invariants require:

1. New contract version (`SCENARIO_DOMAIN_CONTRACT_v2.md`)
2. Registry migration note
3. `AUTHORITY_MAP.md` update if evaluation authority surfaces change

Adding registered domains with documented provenance is backward-compatible within v1.
