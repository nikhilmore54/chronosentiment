# Canonical Schema Registry — Law Zero Audit Surface

**Constitutional Authority**: Every UI element must point to a specific field in a specific backend schema.  
Any UI element that cannot be traced to a schema field is a Law One violation and invents reality.

---

## Schema Inventory

| Schema File | `$id` | Authority Layer | Replaces |
|-------------|-------|-----------------|---------|
| [`event.schema.json`](event.schema.json) | `chrono:schema:event:v1` | Kernel / Sequencer | Raw event objects in `execution_trace[]` |
| [`replay_response.schema.json`](replay_response.schema.json) | `chrono:schema:replay_response:v1` | Replay Engine | `POST /inspect_strategy` response shape |
| [`observatory_state.schema.json`](observatory_state.schema.json) | `chrono:schema:observatory_state:v1` | Governor | Hardcoded operational awareness strip in `App.js:97-117` |
| [`governor_telemetry.schema.json`](governor_telemetry.schema.json) | `chrono:schema:governor_telemetry:v1` | Governor | Hardcoded status labels in `CompareStrategies.js:202-214` |
| [`decision_trace.schema.json`](decision_trace.schema.json) | `chrono:schema:decision_trace:v1` | Strategy Evaluation Engine | `groupAndNarrateEvents()` + `compareNarrativeBlocks()` in `StrategyInspector.js` |

---

## Law Zero Audit Table — Complete UI Element Mapping

### App.js — Operational Awareness Strip (Lines 97–117)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| System State dot + "Nominal" label | Hardcoded string | `observatory_state` | `system_phase` | **LAW ONE** — fabricates kernel state |
| "Chronology Engine: Synchronized (1.00x)" | Hardcoded string | `observatory_state` | `kernel_state.sync_ratio` | **LAW ONE** — fabricates sync ratio |
| "Governor: Active" | Hardcoded string | `observatory_state` | `governor_state.throttle_state` | **LAW ONE** — fabricates governor state |
| "Cohort: NSE_ALPHA_01" | Hardcoded string | `observatory_state` | `governor_state.cohort_id` | **LAW ONE** — fabricates cohort identity |
| IST clock (`useClock()`) | Client-computed | `observatory_state` | `snapshot_at_ns` (display only) | Acceptable — wall clock display, not system state |
| Footer clock | Client-computed | — | — | Acceptable — cosmetic only |

### App.js — Navigation Shell (Lines 7–12, 120–149)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| Tab list (Run GA, Inspect Strategy, Compare Strategies, Global Ranking) | Hardcoded | — | — | Structural — acceptable as shell navigation |
| Cross-tab strategy handoff (`selectedStrategyId`, `selectedSeed`) | Client state | `replay_response` | `strategy_id`, `session_id` | **LAW THREE** — strategy identity should come from a certified replay session reference, not free-form string state |

---

### RunGA.js — Pre-Execution Environment Block (Lines 224–253)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| "Integrity: 100.0%" | Hardcoded | `observatory_state` | `kernel_state.sync_ratio` | **LAW ONE** — fabricates integrity metric |
| "Bounds: 0 → 12,400" | Hardcoded | `observatory_state` | `snapshot_sequence_id` (max bound) | **LAW ONE** — fabricates sequence bounds |
| "Dispersion: 1.00x" | Hardcoded | `observatory_state` | `kernel_state.sync_ratio` | **LAW ONE** — duplicate fabrication |
| "Governor state: NOMINAL" | Hardcoded | `observatory_state` | `governor_state.throttle_state` | **LAW ONE** — fabricates governor state |
| "Network: ACTIVE" | Hardcoded | `observatory_state` | `system_phase` | **LAW ONE** — fabricates system phase |
| "All subsystems nominal" | Hardcoded | `observatory_state` | `system_phase` | **LAW ONE** — fabricates subsystem state |

### RunGA.js — GA Parameters (Lines 117–120, 190–207)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| Population size input | Client state (50) | — | — | **LAW THREE** — collected but never sent to `GET /run_ga` |
| Generations input | Client state (20) | — | — | **LAW THREE** — collected but never sent |
| Mutation rate input | Client state (0.1) | — | — | **LAW THREE** — collected but never sent |
| Seed input | Client state (42) | — | — | **LAW THREE** — collected but never sent |

### RunGA.js — Execution Verdict Zone (Lines 259–277)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| Large fitness number display | `resolveExecutionFitness(gaResult.global_best)` — client-resolved from 4 fallback fields | `decision_trace` | `decision.confidence` (or a dedicated `ga_result` schema field) | **LAW THREE** — client resolves fitness from ambiguous field cascade |
| Divergence badge (Overfit / Hidden Gem / Aligned) | `divergenceBadge()` — client-computed from `ga_fitness` vs `execution_fitness` ratio | `governor_telemetry` | `payload.ga_generation.strategies_certified` + kernel classification | **LAW THREE** — client computes classification that kernel must certify |
| "Certified Execution Fitness" label | Hardcoded string | `decision_trace` | `decision.verdict` | **LAW ONE** — label claims certification the UI cannot grant |
| "Search Gen: N" | `resolvePeakGeneration()` — client-computed from history array | `governor_telemetry` | `payload.ga_generation.generation_number` | **LAW THREE** — client searches for peak; backend must emit it |
| "Avg PnL: N" | `gaResult.global_best?.avg` — raw field access | `decision_trace` | `ga_lineage.fitness_score` | Acceptable field access if schema-aligned |

### RunGA.js — Signals Topology Zone (Lines 280–333)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `buildAssetRollups()` — asset aggregation | Client-computed: participation rate, avgConf, avgPnl, score formula `0.5*maxConf + 0.3*participation + 0.2*...` | `decision_trace` | `signal_inputs[].contribution` | **LAW THREE** — client aggregates and scores signals; backend must emit pre-aggregated rollups |
| `topSignalsPerAsset()` — signal ranking | Client-computed: filters, sorts by confidence, slices top-K | `decision_trace` | `signal_inputs[]` (pre-ordered by backend) | **LAW THREE** — client ranks signals; backend must emit ranked signal list |
| `signalStrength()` — STRONG/WEAK badge | Client-computed: `composite_score > 1e-9` | `decision_trace` | `signal_inputs[].contribution` (threshold applied by backend) | **LAW THREE** — client applies threshold; backend must classify strength |
| Asset symbol display | `s.asset` | `decision_trace` | `signal_inputs[].signal_name` | Acceptable field access |
| BUY/SELL action badge | `s.action` | `decision_trace` | `decision.action` | Acceptable field access |
| Entry zone, Target, Stop Loss | `s.entry_zone`, `s.target`, `s.stop_loss` | `decision_trace` | `decision.price_target` | **LAW ZERO** — `entry_zone` and `stop_loss` have no field in any canonical schema |

### RunGA.js — Strategy Store (Lines 142–158)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `GET /ga/strategy-store` fetch | Calls non-existent endpoint | `observatory_state` | `active_strategies[]` | **LAW ZERO** — endpoint does not exist; data should come from `observatory_state.active_strategies` |

---

### GlobalRanking.js — Ranking Table (Lines 54–130)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `resolveExecutionFitness()` — 4-field cascade | Client-computed: tries `execution_fitness`, `fitness`, `score`, `final_fitness` | `observatory_state` | `active_strategies[].status` + a dedicated ranking schema field | **LAW THREE** — ambiguous field resolution; backend must emit a single canonical fitness field |
| `resolveGaFitness()` — single field | `row.ga_fitness` | `governor_telemetry` | `payload.ga_generation.best_fitness` | **LAW ZERO** — `ga_fitness` has no field in any canonical schema |
| Client-side sort by fitness | `[...ranking].sort(...)` | `observatory_state` | Backend must emit pre-sorted ranking | **LAW FOUR** — UI sorts/aggregates operational state |
| `classificationColor()` — badge color logic | Client-computed: `stable→grn`, `volatile→amb`, `fragile→red` | `observatory_state` | `active_strategies[].status` | **LAW THREE** — client maps classification to color; backend must emit classification; color mapping is acceptable in UI |
| Rank number `index + 1` | Client-computed from sort position | `observatory_state` | A `ranking_position` field | **LAW FOUR** — UI derives rank; backend must certify rank |
| Strategy ID display | `row.strategy_id` | `observatory_state` | `active_strategies[].strategy_id` | Acceptable field access |
| Avg PnL, Std Dev | `row.avg`, `row.std` | `observatory_state` | **LAW ZERO** — `avg` and `std` have no field in any canonical schema |
| Classification badge | `row.classification` | `observatory_state` | **LAW ZERO** — `classification` has no field in `observatory_state.active_strategies[]` |

---

### CompareStrategies.js — Structural Comparison Panel (Lines 192–243)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| "Replay Cert: VALID" label | Hardcoded | `replay_response` | `certification_state` | **LAW ONE** — fabricates replay certification |
| "Replay Integrity: CERTIFIED" | Hardcoded | `replay_response` | `certification_state` | **LAW ONE** — fabricates certification |
| "Timestamp Cohesion: VALID" | Hardcoded | `replay_response` | `certification_state` | **LAW ONE** — fabricates timestamp validity |
| "Synchronization State: DEGRADED" | Hardcoded | `observatory_state` | `system_phase` | **LAW ONE** — fabricates degraded state |
| "Governor Action: THROTTLED" | Hardcoded | `governor_telemetry` | `event_class` + `payload.next_state` | **LAW ONE** — fabricates governor action |
| `queue_depth=12` (Expected State) | Hardcoded | `observatory_state` | `kernel_state.queue_depth` | **LAW ONE** — fabricates expected queue depth |
| `fill_latency=42ms` (Expected State) | Hardcoded | `observatory_state` | `kernel_state.fill_latency_ns` | **LAW ONE** — fabricates expected latency |
| `sync_ratio=0.91` (Expected State) | Hardcoded | `observatory_state` | `kernel_state.sync_ratio` | **LAW ONE** — fabricates expected sync ratio |
| `queue_depth=17` (Observed State) | Hardcoded | `observatory_state` | `kernel_state.queue_depth` | **LAW ONE** — fabricates observed queue depth |
| `fill_latency=58ms` (Observed State) | Hardcoded | `observatory_state` | `kernel_state.fill_latency_ns` | **LAW ONE** — fabricates observed latency |
| `sync_ratio=0.67` (Observed State) | Hardcoded | `observatory_state` | `kernel_state.sync_ratio` | **LAW ONE** — fabricates observed sync ratio |
| `parseStrategyParamsFromId()` | Client-computed: parses strategy config from ID string | `replay_response` | `strategy_id` (opaque — backend resolves config) | **LAW THREE** — client decodes strategy config from ID; backend must resolve config from ID |

### CompareStrategies.js — Ranking Table (Lines 147–185)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `resolveExecutionFitness()` | Client-computed 4-field cascade | `decision_trace` | `decision.confidence` | **LAW THREE** — same cascade violation as GlobalRanking |
| `resolveGaFitness()` | `row.ga_fitness` | `governor_telemetry` | `payload.ga_generation.best_fitness` | **LAW ZERO** — no canonical field |
| Rank number `i + 1` | Client-computed from array position | — | Backend must emit rank | **LAW FOUR** — UI derives rank |

---

### StrategyInspector.js — Client-Side Computation Functions

| Function | Lines | Violation | Required Replacement |
|----------|-------|-----------|---------------------|
| `groupAndNarrateEvents()` | 27–97 | **LAW THREE** — client groups and narrates events | `decision_trace.narrative_blocks[]` — backend emits pre-grouped, pre-narrated blocks |
| `compareNarrativeBlocks()` | 99–114 | **LAW THREE** — client computes divergence between two strategies | `decision_trace.narrative_blocks[].block_type = DIVERGENCE_MARKER` + `divergence_score` |
| `getExecutionSummary()` | 116–122 | **LAW THREE** — client counts steps, fills, queue progressions | `decision_trace.narrative_blocks[]` counts are backend-emitted |
| `normalizeTraceEvent()` | 5–9 | **LAW THREE** — client normalizes/flattens event payload | `event.schema.json` — events arrive pre-normalized |
| `normalizeInspectResponse()` | 11–19 | **LAW THREE** — client normalizes API response shape | `replay_response.schema.json` — response arrives in canonical shape |
| `getCausalChain()` | 214–218 | **LAW THREE** — client traverses `parentId` links to build causal chain | `replay_response.causal_chain[]` — backend emits certified causal chain |
| Verdict computation block | 246–273 | **LAW THREE** — client computes `finalVerdict`, `confidenceLevel`, `confidenceColorClass`, `confidenceReason` | `decision_trace.decision.verdict` + `decision.confidence` |

### StrategyInspector.js — Replay Position Slider (Lines 283–300)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| Range slider `min={minSeqId}` | Client-computed: `Math.min(...execution_trace.map(e => e.sequence_id))` | `replay_response` | `event_window.first_sequence_id` | **LAW THREE** — client computes min; backend emits it |
| Range slider `max={maxAvailableSeqId}` | Client-computed: `Math.max(...execution_trace.map(e => e.sequence_id))` | `replay_response` | `event_window.last_sequence_id` | **LAW THREE** — client computes max; backend emits it |
| Slider `onChange` → `setSelectedMaxSeqId` | Client filters events locally by `sequence_id <= selectedMaxSeqId` | `replay_response` | `requested_sequence_id` — slider value becomes a backend request parameter | **LAW ONE** — client reconstructs state locally; must request certified state from Replay Engine |
| "Seq N" display | Client-computed current position | `replay_response` | `requested_sequence_id` | Acceptable display once schema-driven |

### StrategyInspector.js — Pre-Execution State Block (Lines 325–344)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| "Vector: ONLINE" | Hardcoded | `observatory_state` | `system_phase` | **LAW ONE** — fabricates vector state |
| "Tracking initialized" | Hardcoded | `observatory_state` | `system_phase` | **LAW ONE** — fabricates tracking state |
| "Latch: Locked" | Hardcoded | `observatory_state` | `governor_state.throttle_state` | **LAW ONE** — fabricates latch state |
| "Awaiting trace ID" | Hardcoded | — | — | Acceptable UX placeholder |

---

### StrategyColumn.js — Column Header (Lines 38–44)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `inspectionResult?.strategy_id` | Direct field access | `replay_response` | `strategy_id` | Acceptable — direct schema field |
| `seed` display | Passed from parent | `replay_response` | — | **LAW ZERO** — `seed` has no field in `replay_response`; must be added or sourced from a GA schema |
| `inspectionResult?.metrics?.total_trades` | Direct field access | `decision_trace` | **LAW ZERO** — `metrics.total_trades` has no field in any canonical schema |

### StrategyColumn.js — Causal Chain Panel (Lines 84–108)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `activeChain` set | Client-computed by `getCausalChain()` traversing `parentId` links | `replay_response` | `causal_chain[]` | **LAW THREE** — client derives ancestry; backend certifies it |
| `block.group` display | Client-computed by `groupAndNarrateEvents()` | `decision_trace` | `narrative_blocks[].group` | **LAW THREE** — group is client-synthesized |
| `block.narrative` display | Client-computed by `groupAndNarrateEvents()` | `decision_trace` | `narrative_blocks[].narrative` | **LAW THREE** — narrative is client-synthesized |
| `block.id` (seq display) | Client-assigned from `sequence_id` | `decision_trace` | `narrative_blocks[].sequence_id` | Acceptable once schema-driven |

### StrategyColumn.js — Divergence Markers (Lines 19–27)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `isBlockDivergent()` | Client-computed from `divergenceStatements` array | `decision_trace` | `narrative_blocks[].block_type = DIVERGENCE_MARKER` | **LAW THREE** — client marks divergence; backend certifies it |
| `getBlockDivergenceMessage()` | Client-computed divergence message string | `decision_trace` | `narrative_blocks[].divergence_score` + `narrative` | **LAW THREE** — client synthesizes divergence message |

---

### NarrativeBlock.js — Block Rendering (Lines 27–55)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `block.group` | Client-synthesized by `groupAndNarrateEvents()` | `decision_trace` | `narrative_blocks[].group` | **LAW THREE** — must be backend-emitted |
| `block.isKeyEvent` + `block.keyEventMarker` | Client-computed heuristic | `decision_trace` | `narrative_blocks[].block_type` | **LAW THREE** — key event classification must be backend-certified |
| `seq:{block.id}` display | Client-assigned | `decision_trace` | `narrative_blocks[].sequence_id` | Acceptable once schema-driven |
| `block.narrative` | Client-synthesized | `decision_trace` | `narrative_blocks[].narrative` | **LAW THREE** — must be backend-emitted |
| `↳ derived from seq:{block.parentId}` | Client-computed from `parentId` | `decision_trace` | `narrative_blocks[].parent_block_id` | **LAW THREE** — parent relationship must be backend-certified |
| Divergence message `⚑ {blockDivergenceMessage}` | Client-computed | `decision_trace` | `narrative_blocks[].divergence_score` | **LAW THREE** — must be backend-certified |
| `isDimmed` state | Client-computed from `activeChain.size > 0 && !isActive` | `replay_response` | `causal_chain[]` | **LAW THREE** — dimming logic depends on client-derived causal chain |

---

### ComparisonPanels.js — Execution Summary (Lines 24–49)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| Steps count | `executionSummary.totalSteps` — client-counted | `decision_trace` | `narrative_blocks.length` (backend-emitted count) | **LAW THREE** — client counts steps |
| Partial fills count | `executionSummary.partialFills` — client-counted via `narrative.includes('partially filled')` | `decision_trace` | `narrative_blocks[]` filtered by `block_type` | **LAW THREE** — client string-matches to count fills |
| Queue Progression Yes/No | `executionSummary.hasQueueProgression` — client-computed | `decision_trace` | `narrative_blocks[].group = QUEUE` | **LAW THREE** — client infers queue behavior |
| Full fills count | `executionSummary.totalFills` — client-counted via `narrative.includes('fully executed')` | `decision_trace` | `narrative_blocks[]` filtered by `block_type` | **LAW THREE** — client string-matches to count fills |
| Insights list (lines 24–49) | Client-computed comparative logic | `decision_trace` | `divergence_score` + `narrative_blocks[].block_type = DIVERGENCE_MARKER` | **LAW THREE** — client synthesizes comparative insights |

### ComparisonPanels.js — Final Verdict (Lines 114–121)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `finalVerdict` string | Client-computed in `StrategyInspector.js:246-273` | `decision_trace` | `decision.verdict` | **LAW THREE** — client synthesizes verdict |
| `confidenceLevel` | Client-computed | `decision_trace` | `decision.confidence` | **LAW THREE** — client computes confidence |
| `confidenceColorClass` | Client-computed from confidence level | `decision_trace` | `decision.confidence` (UI maps to color — acceptable) | **LAW THREE** — underlying value must be backend-certified |
| `confidenceReason` | Client-synthesized string | `decision_trace` | `narrative_blocks[].narrative` | **LAW THREE** — client synthesizes rationale |

### ComparisonPanels.js — Divergence Analysis (Lines 123–139)

| UI Element | Current Value | Required Schema | Required Field | Violation |
|------------|--------------|-----------------|----------------|-----------|
| `divergenceStatements[]` | Client-computed by `compareNarrativeBlocks()` | `decision_trace` | `narrative_blocks[].block_type = DIVERGENCE_MARKER` + `divergence_score` | **LAW THREE** — client computes all divergence; backend must certify it |
| "No significant execution divergences detected" | Client-computed empty-state | `decision_trace` | `narrative_blocks[]` with no `DIVERGENCE_MARKER` blocks | **LAW THREE** — client infers absence of divergence |

---

## Law Zero Violation Summary

| Violation Type | Count | Severity |
|---------------|-------|----------|
| **LAW ONE** — UI fabricates backend-certified values (hardcoded) | 22 | Critical |
| **LAW THREE** — UI computes values the backend must certify | 41 | Critical |
| **LAW FOUR** — UI aggregates or sorts operational state | 3 | High |
| **LAW ZERO** — UI element has no schema field at all | 7 | Blocking |

### LAW ZERO Gaps — Schema Fields That Must Be Added

These UI elements have no corresponding field in any canonical schema. The schemas must be extended before these surfaces can be rebuilt:

| UI Element | File | Missing Schema Field |
|------------|------|---------------------|
| `row.avg` (Avg PnL) | `GlobalRanking.js:109` | `observatory_state.active_strategies[].avg_pnl` |
| `row.std` (Std Dev) | `GlobalRanking.js:110` | `observatory_state.active_strategies[].pnl_std_dev` |
| `row.classification` | `GlobalRanking.js:116` | `observatory_state.active_strategies[].classification` |
| `row.ga_fitness` | `GlobalRanking.js:113`, `RunGA.js:19` | `observatory_state.active_strategies[].ga_fitness` |
| `inspectionResult?.metrics?.total_trades` | `StrategyColumn.js:42` | `replay_response.portfolio_state.total_trades` |
| `seed` parameter | `StrategyColumn.js:40` | `replay_response.seed` (or GA lineage schema) |
| `s.entry_zone`, `s.stop_loss` | `RunGA.js:312-320` | `decision_trace.signal_inputs[].entry_zone`, `stop_loss` |

---

## Build Sequence

The following sequence is the only valid order for rebuilding the observer UI. Skipping steps violates the constitutional build contract.

- [x] **Step 1** — Define 5 canonical schemas (`event`, `replay_response`, `observatory_state`, `governor_telemetry`, `decision_trace`)
- [x] **Step 2** — Identify all LAW ZERO gaps (schema fields that don't exist yet) — see table above
- [ ] **Step 3** — Extend schemas to cover all LAW ZERO gaps
- [ ] **Step 4** — Implement backend endpoints that emit these schemas
- [ ] **Step 5** — Build `services/ui` as a pure observer of those endpoints
- [ ] **Step 6** — Law One audit: trace every UI element in `services/ui` to a specific schema field using this README as the audit checklist
---

## Full-Stack Backend Audit

### services/api — Endpoint Inventory vs Canonical Schemas

The Rust API (`services/api/`) runs on port 8000 via Axum. The following table audits every endpoint against the canonical schemas.

| Endpoint | Handler | Response DTO | Schema Conformance | Violations |
|----------|---------|-------------|-------------------|------------|
| `GET /` | health | `{ "status": "ok" }` | — | Acceptable health check |
| `GET /timeline` | `timeline_handler` | `TimelineResponse { events: Vec<EventWrapper> }` | `event.schema.json` | **PARTIAL** — `EventWrapper` has `sequence_id`, `timestamp`, `type`, `parent_sequence_id`, `payload` but lacks `source_layer`, `kernel_signature`, `replay_session_id` |
| `GET /events` | `events_handler` | `Vec<EventWrapper>` | `event.schema.json` | **PARTIAL** — same gaps as `/timeline` |
| `GET /replay/:seq_id` | `replay_handler` | `SystemState { orders, portfolio, last_sequence_id }` | `replay_response.schema.json` | **NON-CONFORMANT** — `SystemState` has no `session_id`, `certification_state`, `event_window`, `causal_chain`, `replay_signature`; `PortfolioState` has only `pnl` and `position` (missing `positions[]`, `cash_balance`, `total_equity`, `unrealized_pnl`, `realized_pnl`) |
| `GET /order/:order_id` | `order_inspection_handler` | `TradeInspectorResponse` | `decision_trace.schema.json` | **NON-CONFORMANT** — response has `decision`, `execution[]`, `outcome`, `causal_chain[]` but no `narrative_blocks[]`, `signal_inputs[]`, `trace_signature`, `ga_lineage` |
| `POST /evaluate_strategy` | `evaluate_strategy_handler` | `EvaluateStrategyResponse { strategy_evaluation: StrategyEvaluationDto }` | `decision_trace.schema.json` | **NON-CONFORMANT** — `StrategyEvaluationDto` is `UnifiedStrategyEvaluation` from core; no `narrative_blocks[]`, `trace_signature`, `certification_state` |
| `POST /compare_strategies` | `compare_strategies_handler` | `CompareStrategiesResponse { ranking[], comparison_summary }` | `decision_trace.schema.json` | **NON-CONFORMANT** — `comparison_summary` has only `best_strategy` and `reason` string; no `divergence_score`, `narrative_blocks[DIVERGENCE_MARKER]`, `certification_state` |
| `POST /inspect_strategy` | `inspect_strategy_handler` | `InspectStrategyResponse { strategy_id, decision_trace[], execution_trace[], metrics, event_sequence[] }` | `replay_response.schema.json` + `decision_trace.schema.json` | **NON-CONFORMANT** — `decision_trace[]` and `execution_trace[]` are `Vec<EventWrapper>` (raw events), not `narrative_blocks[]`; `metrics` is `StrategyEvaluationDto` not a certified schema; no `certification_state`, `replay_signature`, `trace_signature` |
| `GET /run_ga` | `run_ga_handler` | `RunGaResponse { results[], generation_history[], best_per_regime, global_best, global_best_generation, generation_found, final_generation_best, final_gen_best }` | `governor_telemetry.schema.json` | **NON-CONFORMANT** — duplicates `global_best_generation` as `generation_found`, `final_generation_best` as `final_gen_best`; no `telemetry_signature`, `event_class`, `anchor_sequence_id`; GA result is not a telemetry event |
| `GET /ga/global-ranking` | `get_global_ranking_handler` | `Vec<StrategyEvaluationDto>` | `observatory_state.schema.json` | **NON-CONFORMANT** — returns flat array of evaluation DTOs; no `snapshot_id`, `snapshot_sequence_id`, `system_phase`, `governor_state`, `kernel_state`, `observatory_signature`; no `ranking_position` field |
| `GET /ga/strategy-store` | `get_strategy_store_handler` | `{ path, store }` | — | **NON-CONFORMANT** — raw JSON dump of on-disk store; no schema; not a canonical observatory surface |
| `GET /signals/latest` | `latest_signals_handler` | `SignalsSnapshotDto { timestamp, signals[] }` | `decision_trace.schema.json` | **PARTIAL** — `TradeSignalDto` has `entry_zone`, `stop_loss`, `target`, `confidence`, `action`, `strategy_id` (covers LAW ZERO gaps in RunGA.js); but no `trace_signature`, `narrative_blocks[]`, `signal_inputs[].contribution` |
| `GET /signals/trade-suggestions` | `trade_suggestions_handler` | `TradeSuggestionsResponse` | — | **NON-CONFORMANT** — no canonical schema coverage |
| `GET /signals/replay-suggestions` | `replay_suggestions_handler` | `ReplaySuggestionsResponse { asset, metrics, timeline[], pnl }` | `replay_response.schema.json` | **NON-CONFORMANT** — `ReplaySuggestionPoint` has no `certification_state`, `replay_signature`, `causal_chain[]` |
| `POST /test_determinism` | `test_determinism_handler` | `{ deterministic: bool }` | — | **NON-CONFORMANT** — determinism result should be part of `replay_response.certification_state`; standalone boolean is not a certified schema field |

### services/api — DTO Shape Violations

| DTO | File | Violation |
|-----|------|-----------|
| [`EventWrapper`](services/api/src/dto.rs:114) | `dto.rs:114` | Missing `source_layer`, `kernel_signature`, `replay_session_id` vs `event.schema.json` |
| [`SystemState`](services/api/src/dto.rs:296) | `dto.rs:296` | Replaces `replay_response.schema.json` but has no `session_id`, `certification_state`, `event_window`, `causal_chain[]`, `replay_signature` |
| [`PortfolioState`](services/api/src/dto.rs:290) | `dto.rs:290` | Has only `pnl: f64` and `position: i64`; missing `positions[]`, `cash_balance`, `total_equity`, `unrealized_pnl`, `realized_pnl` |
| [`InspectStrategyResponse`](services/api/src/dto.rs:79) | `dto.rs:79` | `decision_trace: Vec<EventWrapper>` — raw events, not `narrative_blocks[]`; no `trace_signature` |
| [`RunGaResponse`](services/api/src/dto.rs:88) | `dto.rs:88` | Duplicates `global_best_generation` / `generation_found` and `final_generation_best` / `final_gen_best`; no `telemetry_signature` |
| [`ComparisonSummary`](services/api/src/dto.rs:73) | `dto.rs:73` | Only `best_strategy: String` and `reason: String`; no `divergence_score`, `certification_state` |
| [`TradeInspectorResponse`](services/api/src/dto.rs:269) | `dto.rs:269` | Has `causal_chain: Option<Vec<EventWrapper>>` — correct concept, wrong shape; should be `Vec<u64>` (sequence_ids) per `replay_response.causal_chain[]` |
| [`CertificationResponse`](services/api/src/lib.rs:38) | `lib.rs:38` | `status: String` ("PASS"/"FAIL") — should be `certification_state` enum per `replay_response.schema.json`; `hash_1`/`hash_2` are not `replay_signature` SHA-256 fields |

### services/api — certify.rs Violations

| Issue | File | Violation |
|-------|------|-----------|
| [`hash_simulation_events()`](services/api/src/certify.rs:76) | `certify.rs:76` | Uses string length + first/last 16 chars as "hash" — not SHA-256; `kernel_signature` and `replay_signature` in canonical schemas require SHA-256 hex digest |
| `fill_probability: 0.5` hardcoded | `certify.rs:22` | Hardcoded fill probability for replay reconstruction — violates determinism guarantee; replay must use original parameters |
| `config_hash: "default-config-hash"` | `certify.rs:64` | Hardcoded config hash — not a real hash of simulation config |

### services/api — inspector.rs Violations

| Issue | File | Violation |
|-------|------|-----------|
| [`MinimalEvent`](services/api/src/inspector.rs:17) | `inspector.rs:17` | Missing `source_layer`, `kernel_signature` vs `event.schema.json` |
| [`EventType`](services/api/src/inspector.rs:7) | `inspector.rs:7` | Enum has 6 values; `event.schema.json` has 16 `event_type` values — backend event taxonomy is incomplete |
| Causal chain traversal | `inspector.rs:156-177` | Backend traverses `parent_sequence_id` links to build causal chain — this is correct authority (backend certifies causal lineage); but result is `Vec<MinimalEvent>` not `Vec<u64>` (sequence_ids) as specified in `replay_response.causal_chain[]` |

---

### observatory/ — Static Research App Audit

The `observatory/` directory is a standalone static HTML/JS research tool that reads from pre-exported `data.json` files. It is **not** connected to the Rust API and operates entirely from file-based data.

| Surface | Authority Classification | Schema Conformance | Notes |
|---------|------------------------|-------------------|-------|
| Ecology tab (`renderEcology`) | Experimental Tooling | None | Reads `data.summary` from `data.json` — no canonical schema; acceptable as research tool |
| Smoothness Trap tab (`renderSmoothness`) | Experimental Tooling | None | Client-computes efficiency averages, inversion table — acceptable for research |
| Edge Genesis tab (`renderGenesis`) | Experimental Tooling | None | Client-computes compression/bias stats — acceptable for research |
| Toxicity Atlas tab (`renderAtlas`) | Experimental Tooling | None | Client-computes age bins, freshness decay curve — acceptable for research |
| Trade Replay tab (`renderReplay`) | Experimental Tooling | None | `buildLifecycleEvents()` synthesizes lifecycle narrative client-side — **would be LAW THREE violation if promoted to canonical UI** |
| Comparative Ecology tab (`renderComparative`) | Experimental Tooling | None | `testBias()`, `testAge()`, `testTrap()` compute statistical tests client-side — **would be LAW THREE violations if promoted** |

**Observatory Classification**: The `observatory/` app is correctly classified as **Experimental Tooling** (Layer 5 in the authority hierarchy — non-authoritative exploration). It should not be promoted to canonical infrastructure. Its research findings (smoothness trap, pre-bias toxicity, elasticity age decay) are valuable discoveries that should be extracted and certified by the backend before being surfaced in `services/ui`.

---

### cs-ingest/ — Ingest Pipeline Audit

| Module | Authority Classification | Schema Conformance | Notes |
|--------|------------------------|-------------------|-------|
| [`observatory.rs`](cs-ingest/src/observatory.rs) | Kernel / Sequencer | None | Spawns observatory binary, reads `[TELEMETRY]` lines — telemetry format is not a canonical schema |
| [`telemetry.rs`](cs-ingest/src/telemetry.rs) | Kernel / Sequencer | None | `TelemetryProcessor.process_line()` computes `entropy`, `corridor`, `instability_type` client-side from raw telemetry — these computed fields should be certified by the kernel, not derived in the ingest pipeline |
| [`replay.rs`](cs-ingest/src/replay.rs) | Replay Engine | None | `run_replay_step()` orchestrates replay but produces no canonical `ReplayResponse` — output is raw archive files, not schema-conformant responses |

**cs-ingest Classification**: The ingest pipeline is a data preparation layer, not a canonical authority layer. Its outputs feed the observatory research tool. The `instability_type` classification in [`telemetry.rs:44-52`](cs-ingest/src/telemetry.rs:44) is a computed field that would need kernel certification before appearing in any canonical schema.

---

## Full-Stack Violation Summary

| Layer | Violation Type | Count | Severity |
|-------|---------------|-------|----------|
| **React UI** — LAW ONE (hardcoded values) | UI fabricates backend-certified values | 22 | Critical |
| **React UI** — LAW THREE (client computation) | UI computes values the backend must certify | 41 | Critical |
| **React UI** — LAW FOUR (client aggregation) | UI aggregates or sorts operational state | 3 | High |
| **React UI** — LAW ZERO (no schema field) | UI element has no corresponding schema field | 7 | Blocking |
| **Backend API** — Non-conformant response shapes | Endpoints return DTOs that don't match canonical schemas | 11 | Critical |
| **Backend API** — Missing signature fields | `kernel_signature`, `replay_signature`, `trace_signature` absent from all responses | 8 | Critical |
| **Backend API** — Fake hash implementation | `certify.rs` uses string length as "hash" instead of SHA-256 | 1 | Critical |
| **Backend API** — Hardcoded replay parameters | `fill_probability: 0.5` hardcoded in certify replay | 1 | Critical |
| **Backend API** — Duplicate/redundant fields | `RunGaResponse` duplicates `global_best_generation`/`generation_found` | 2 | Medium |
| **cs-ingest** — Client-computed classifications | `instability_type` computed in ingest, not certified by kernel | 1 | Medium |
| **observatory/** — Research tool promoted risk | `buildLifecycleEvents()` and statistical tests would be LAW THREE if promoted | 6 | Low (currently acceptable) |

**Total full-stack violations: 103+**

---

## Revised Build Sequence (Full-Stack)

- [x] **Step 1** — Define 5 canonical schemas
- [x] **Step 2** — Identify all LAW ZERO gaps
- [ ] **Step 3** — Extend schemas to cover LAW ZERO gaps (7 fields)
- [ ] **Step 4** — Add `kernel_signature`, `replay_signature`, `trace_signature` SHA-256 fields to all backend response DTOs
- [ ] **Step 5** — Implement real SHA-256 hashing in `certify.rs` (replace string-length pseudo-hash)
- [ ] **Step 6** — Reshape backend DTOs to match canonical schemas: `SystemState` → `ReplayResponse`, `InspectStrategyResponse` → `ReplayResponse + DecisionTrace`, `RunGaResponse` → `GovernorTelemetry`
- [ ] **Step 7** — Add `source_layer` to `EventWrapper` / `MinimalEvent`
- [ ] **Step 8** — Expand `EventType` enum in `inspector.rs` to match all 16 `event_type` values in `event.schema.json`
- [ ] **Step 9** — Add `narrative_blocks[]` emission to `inspect_strategy` endpoint (replaces client-side `groupAndNarrateEvents()`)
- [ ] **Step 10** — Add `certification_state` enum to all replay/inspect responses
- [ ] **Step 11** — Build `services/ui` as a pure observer of the now-conformant endpoints
- [ ] **Step 12** — Law One audit: trace every UI element in `services/ui` to a specific schema field