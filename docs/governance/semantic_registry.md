# ChronoSentiment — Canonical Semantic Registry

**Authority:** Constitutional Layer (supersedes all implementation-level naming)  
**Status:** Active — violations are governance failures  
**Last Updated:** 2026-05-25  

---

## Law of Canonical Field Singularity

> One concept. One field. One authority. No synonyms in canonical schemas.

Any field not listed here that attempts to carry a meaning already registered here is a **semantic drift violation** and must be removed or aliased to the canonical form.

---

## Section 1 — Fitness and Evaluation Terms

| Canonical Term | Meaning | Authority Layer | Schema Field | Forbidden Synonyms |
|---|---|---|---|---|
| `execution_fitness` | Replay-certified execution quality score. Measures how well a strategy performed under real microstructure conditions (queue, latency, fills). Range: [0.0, 1.0]. | Kernel → Replay Engine | `decision_trace.decision.confidence` | `final_fitness`, `score`, `execution_score`, `live_score`, `replay_score` |
| `ga_fitness` | Optimization-space fitness score produced by the GA engine during evolutionary search. Not a certified execution metric — it is a search heuristic. Range: [0.0, 1.0]. | Experimental Tooling → GA Optimizer | `governor_telemetry.payload.ga_generation.best_fitness` | `fitness`, `raw_fitness`, `optimization_score`, `ga_score` |
| `avg_pnl` | Mean profit/loss per trade across evaluated scenarios. Denominated in internal scaled units (divide by PRICE_SCALE=10000 for ₹). | Kernel | `decision_trace.decision.expected_edge` | `avg`, `mean_pnl`, `expected_pnl`, `pnl_avg` |
| `std_dev` | Standard deviation of per-trade PnL. Measures execution consistency. | Kernel | `governor_telemetry.payload.risk_metrics.volatility` | `std`, `pnl_std`, `deviation`, `variance` |
| `trade_count` | Total number of evaluated round-trip trades in the evaluation window. | Kernel | `decision_trace.decision.trade_count` | `total_trades`, `num_trades`, `trade_num` |
| `classification` | Human-readable strategy quality label derived from `execution_fitness`. Values: `ELITE`, `STRONG`, `MODERATE`, `WEAK`, `INVALID`. | Replay Engine | `decision_trace.decision.classification` | `label`, `grade`, `tier`, `quality`, `rating` |

---

## Section 2 — Replay and Certification Terms

| Canonical Term | Meaning | Authority Layer | Schema Field | Forbidden Synonyms |
|---|---|---|---|---|
| `certification_state` | Formal replay validity verdict. Values: `CERTIFIED`, `DEGRADED`, `PARTIAL`, `INVALID`. Only the Replay Engine may emit this. | Replay Engine | `replay_response.certification_state` | `status`, `validity`, `replay_status`, `cert_status`, `verified` |
| `replay_signature` | BLAKE3/SHA-256 hash of the canonical replay session. Encodes: session_id, strategy_id, last_sequence_id, certification_state, event_count. | Kernel | `replay_response.replay_signature` | `hash`, `checksum`, `replay_hash`, `session_hash` |
| `trace_signature` | BLAKE3/SHA-256 hash of the decision trace. Encodes: trace_id, last_sequence_id, strategy_id, entry_state, exit_state. | Kernel | `decision_trace.trace_signature` | `decision_hash`, `trace_hash`, `audit_hash` |
| `kernel_signature` | Per-event BLAKE3 hash encoding event identity. Encodes: sequence_id, timestamp_ns, event_type, source_layer, payload hash. | Kernel | `event.kernel_signature` | `event_hash`, `event_signature`, `sig`, `hash` |
| `session_id` | UUID v4 identifying a single replay session. Unique per invocation of the replay engine. | Replay Engine | `replay_response.session_id` | `replay_id`, `run_id`, `session` |
| `trace_id` | UUID v4 identifying a single decision trace within a replay session. | Replay Engine | `decision_trace.trace_id` | `decision_id`, `trace`, `audit_id` |

---

## Section 3 — Event Identity Terms

| Canonical Term | Meaning | Authority Layer | Schema Field | Forbidden Synonyms |
|---|---|---|---|---|
| `sequence_id` | Monotonically increasing integer identifying an event's position in the canonical chronology. Assigned by the Sequencer. Never reused. | Kernel → Sequencer | `event.sequence_id` | `seq`, `id`, `event_id`, `order`, `index` |
| `timestamp_ns` | Event timestamp in nanoseconds since Unix epoch. Assigned at event emission. | Kernel | `event.timestamp_ns` | `timestamp`, `ts`, `time`, `event_time`, `exchange_ts` |
| `source_layer` | The authority layer that emitted the event. Values: `KERNEL`, `SEQUENCER`, `LATENCY_LAYER`, `ESE`, `PORTFOLIO_ENGINE`, `GOVERNOR`, `GA_OPTIMIZER`. | Kernel | `event.source_layer` | `origin`, `emitter`, `layer`, `source` |
| `parent_sequence_id` | The `sequence_id` of the causal parent event. `null` for root events. | Kernel → Sequencer | `event.parent_sequence_id` | `parent_id`, `caused_by`, `parent`, `causal_parent` |
| `event_type` | Canonical event classification. Must be one of the 16 registered types in `event_taxonomy.md`. | Kernel | `event.event_type` | `type`, `kind`, `category`, `event_kind` |

---

## Section 4 — Narrative and Observability Terms

| Canonical Term | Meaning | Authority Layer | Schema Field | Forbidden Synonyms |
|---|---|---|---|---|
| `narrative_blocks` | Backend-certified array of grouped chronology segments. Each block carries: group, sequence_id, narrative, block_type, optional divergence_score. The UI must render these — never generate them. | Replay Engine | `decision_trace.narrative_blocks[]` | `events_narrative`, `grouped_events`, `story`, `timeline_groups` |
| `causal_ancestry` | Ordered array of `sequence_id` values forming the causal chain from root to current event. | Replay Engine | `decision_trace.causal_ancestry[]` | `causal_chain`, `ancestry`, `parent_chain`, `event_chain` |
| `divergence_score` | Numeric measure [0.0, 1.0] of how much a narrative block deviates from expected execution. Emitted by the Replay Engine. | Replay Engine | `decision_trace.narrative_blocks[].divergence_score` | `drift_score`, `deviation`, `anomaly_score` |
| `observatory_state` | Snapshot of the full system observability surface at a point in time. | Governor | `observatory_state.*` | `system_state`, `state`, `snapshot`, `system_snapshot` |

---

## Section 5 — Portfolio and Position Terms

| Canonical Term | Meaning | Authority Layer | Schema Field | Forbidden Synonyms |
|---|---|---|---|---|
| `total_equity` | Total portfolio value (cash + unrealized positions) at the end of the evaluation window. Denominated in ₹ (already scaled). | Portfolio Engine | `replay_response.portfolio_state.total_equity` | `portfolio_value`, `equity`, `total_value`, `nav` |
| `realized_pnl` | Cumulative closed-trade profit/loss over the evaluation window. Denominated in ₹. | Portfolio Engine | `replay_response.portfolio_state.realized_pnl` | `closed_pnl`, `pnl`, `profit`, `gain` |
| `unrealized_pnl` | Mark-to-market value of open positions. Denominated in ₹. | Portfolio Engine | `replay_response.portfolio_state.unrealized_pnl` | `open_pnl`, `mtm`, `floating_pnl` |

---

## Section 6 — Forbidden Patterns

The following patterns are **semantic drift violations** regardless of context:

### 6.1 Multi-field truth aliasing
```
// VIOLATION: two fields carrying the same truth
fitness: f64,
execution_fitness: f64,  // same value, different name
```

### 6.2 Fallback resolution cascades
```typescript
// VIOLATION: client resolving ambiguous truth
function resolveExecutionFitness(strategy) {
  return strategy.execution_fitness 
    ?? strategy.fitness 
    ?? strategy.score 
    ?? 0;
}
```

### 6.3 Client-side verdict computation
```typescript
// VIOLATION: UI computing certified state
const isCertified = strategy.fitness > 0.7 && strategy.trade_count > 10;
```

### 6.4 Duplicate generation fields
```rust
// VIOLATION: same concept, two fields
global_best_generation: usize,
generation_found: usize,  // identical value
```

---

## Section 7 — Enforcement Rules

1. **New fields** must be registered here before being added to any canonical schema or DTO.
2. **Renamed fields** must update this registry and all downstream schemas atomically.
3. **Deprecated fields** must be listed in `transitional_artifacts.md` with a sunset milestone.
4. **UI elements** must reference a `schema.field.path` from this registry — never a computed or derived value.
5. **CI** must fail if a DTO field name matches a forbidden synonym listed in this registry.

---

## Section 8 — Registry Maintenance

This document is owned by the **Constitutional Layer**.  
Changes require explicit governance review.  
No implementation-layer change may introduce a new canonical term without updating this registry first.