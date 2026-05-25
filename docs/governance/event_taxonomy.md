# ChronoSentiment — Canonical Event Taxonomy

**Authority:** Constitutional Layer → Kernel  
**Status:** Active — all event emissions must use canonical types  
**Last Updated:** 2026-05-25  

---

## Law of Chronological Completeness

> Every state transition in the system must be representable as a canonical event type.  
> No event may be emitted without a registered type.  
> No registered type may be emitted by an unauthorized layer.

---

## Current Implementation vs. Canonical Target

| Layer | Current Types | Canonical Types Required |
|---|---|---|
| Market Microstructure | `MarketEventType`: 3 values | 3 canonical types |
| Order Lifecycle | `EventType` (inspector): 6 values | 6 canonical types |
| Portfolio/Position | None | 3 canonical types |
| Replay/Certification | None | 3 canonical types |
| Governor/Telemetry | None | 1 canonical type |
| **Total** | **9 distinct concepts** | **16 canonical types** |

---

## Section 1 — Canonical Event Type Registry

### Layer 1: Market Microstructure Events
*Emitted by: `SEQUENCER` (from raw exchange feed)*  
*Current Rust type: `MarketEventType` in [`core/src/lib.rs`](core/src/lib.rs:149)*

| # | Canonical Type | Rust Variant | Meaning | Certification Impact |
|---|---|---|---|---|
| 1 | `MARKET_NEW_ORDER` | `MarketEventType::NewOrder` | New order added to the order book at a price level. Increases available liquidity. | Required for queue depth calculation. Missing = `DEGRADED`. |
| 2 | `MARKET_TRADE` | `MarketEventType::Trade` | Trade executed between two parties. Consumes liquidity. Drives queue progression. | Required for fill simulation. Missing = `INVALID`. |
| 3 | `MARKET_CANCEL` | `MarketEventType::Cancel` | Order removed from book without execution. Reduces available liquidity. | Required for accurate queue depth. Missing = `DEGRADED`. |

### Layer 2: Order Lifecycle Events
*Emitted by: `ESE` (Execution Simulation Engine)*  
*Current Rust type: `EventType` enum in [`services/api/src/inspector.rs`](services/api/src/inspector.rs:7)*  
*Also: `SimEvent` variants in [`core/src/lib.rs`](core/src/lib.rs:206)*

| # | Canonical Type | Rust Variant | Meaning | Certification Impact |
|---|---|---|---|---|
| 4 | `ORDER_INTENT` | `SimEvent::OrderIntent` / `EventType::OrderIntent` | Strategy signals intent to place an order at a price. Root event — no `parent_sequence_id`. | Required. Missing = `INVALID`. |
| 5 | `ORDER_ENTERED_QUEUE` | `SimEvent::OrderEnteredQueue` / `EventType::OrderEnteredQueue` | Order accepted into the exchange queue. `queue_ahead` field populated. Parent: `ORDER_INTENT`. | Required. Missing = `PARTIAL`. |
| 6 | `QUEUE_PROGRESSION` | `SimEvent::QueueProgression` / `EventType::QueueProgression` | Queue position advanced as trades consume ahead-of-queue liquidity. `queue_ahead` decremented. Parent: `ORDER_ENTERED_QUEUE`. | Optional but expected. Missing = `PARTIAL`. |
| 7 | `PARTIAL_FILL` | `SimEvent::PartialFill` / `EventType::PartialFill` | Order partially executed. `filled_qty` and `price` populated. Parent: `ORDER_ENTERED_QUEUE` or `QUEUE_PROGRESSION`. | Required if partial fills occurred. Missing = `DEGRADED`. |
| 8 | `ORDER_FILLED` | `SimEvent::OrderFilled` / `EventType::OrderFilled` | Order fully executed. Terminal event for the order lifecycle. Parent: `PARTIAL_FILL` or `ORDER_ENTERED_QUEUE`. | Required for certified execution. Missing = `INVALID`. |
| 9 | `ORDER_CANCELLED` | *(not yet implemented)* | Order removed from queue by strategy decision or system timeout. Terminal event. Parent: `ORDER_ENTERED_QUEUE` or `QUEUE_PROGRESSION`. | Required when cancellation occurs. Missing = `DEGRADED`. |

### Layer 3: Portfolio and Position Events
*Emitted by: `PORTFOLIO_ENGINE`*  
*Current Rust type: None — not yet implemented*

| # | Canonical Type | Rust Variant | Meaning | Certification Impact |
|---|---|---|---|---|
| 10 | `POSITION_OPENED` | *(not yet implemented)* | New position established following a fill event. Carries: `entry_price`, `quantity`, `side`. Parent: `ORDER_FILLED`. | Required for PnL certification. Missing = `PARTIAL`. |
| 11 | `POSITION_CLOSED` | *(not yet implemented)* | Position closed. `realized_pnl` computed and locked. Parent: `ORDER_FILLED` (exit leg). | Required for PnL certification. Missing = `PARTIAL`. |
| 12 | `EQUITY_SNAPSHOT` | *(not yet implemented)* | Portfolio equity snapshot at evaluation boundary. Carries: `total_equity`, `realized_pnl`, `unrealized_pnl`. | Required for `CERTIFIED` state. Missing = `DEGRADED`. |

### Layer 4: Replay and Certification Events
*Emitted by: `KERNEL` (Replay Engine)*  
*Current Rust type: None — not yet implemented*

| # | Canonical Type | Rust Variant | Meaning | Certification Impact |
|---|---|---|---|---|
| 13 | `REPLAY_SESSION_START` | *(not yet implemented)* | Replay session initialized. Carries: `session_id`, `strategy_id`, `chronology_hash`. Root event for the session. | Required. Missing = `INVALID`. |
| 14 | `REPLAY_SESSION_END` | *(not yet implemented)* | Replay session completed. `replay_signature` computed and locked. Carries: `event_count`, `certification_state`. | Required. Missing = `INVALID`. |
| 15 | `CERTIFICATION_VERDICT` | *(not yet implemented)* | Formal `certification_state` verdict emitted by the Replay Engine. Only this event type may carry a `certification_state` value. | This IS the certification. |

### Layer 5: Governor and Telemetry Events
*Emitted by: `GOVERNOR`*  
*Current Rust type: None — not yet implemented*

| # | Canonical Type | Rust Variant | Meaning | Certification Impact |
|---|---|---|---|---|
| 16 | `GOVERNOR_TELEMETRY` | *(not yet implemented)* | Governor health and metrics snapshot. Carries: `cpu_usage`, `memory_mb`, `active_replays`, `queue_depth`. | Informational. Does not affect `certification_state`. |

---

## Section 2 — Causality Rules

Every event except root events must carry a `parent_sequence_id`. The following causality chains are canonical:

```
REPLAY_SESSION_START (root)
  └── ORDER_INTENT (root within session)
        └── ORDER_ENTERED_QUEUE
              ├── QUEUE_PROGRESSION
              │     ├── QUEUE_PROGRESSION (repeated)
              │     ├── PARTIAL_FILL
              │     │     └── ORDER_FILLED (terminal)
              │     └── ORDER_CANCELLED (terminal)
              ├── PARTIAL_FILL
              │     └── ORDER_FILLED (terminal)
              └── ORDER_CANCELLED (terminal)
  └── POSITION_OPENED (parent: ORDER_FILLED)
        └── POSITION_CLOSED (parent: ORDER_FILLED exit leg)
              └── EQUITY_SNAPSHOT
  └── REPLAY_SESSION_END (terminal, root-level)
        └── CERTIFICATION_VERDICT (terminal)
```

**Market events are parallel, not causal children of order events:**
```
MARKET_NEW_ORDER (independent, no parent)
MARKET_TRADE (independent, no parent)
MARKET_CANCEL (independent, no parent)
```

Market events drive queue state changes that cause `QUEUE_PROGRESSION` events, but they are not direct parents in the causal chain.

---

## Section 3 — Certification Impact Matrix

| Missing Event Type | Effect on `certification_state` |
|---|---|
| `MARKET_TRADE` | `INVALID` — fill simulation impossible |
| `ORDER_INTENT` | `INVALID` — no order lifecycle to certify |
| `ORDER_FILLED` | `INVALID` — execution not confirmed |
| `REPLAY_SESSION_START` | `INVALID` — session boundary undefined |
| `REPLAY_SESSION_END` | `INVALID` — signature cannot be computed |
| `ORDER_ENTERED_QUEUE` | `PARTIAL` — queue position unknown |
| `PARTIAL_FILL` (when fills occurred) | `DEGRADED` — fill sequence incomplete |
| `ORDER_CANCELLED` (when cancellation occurred) | `DEGRADED` — lifecycle incomplete |
| `POSITION_OPENED` | `PARTIAL` — PnL chain broken |
| `POSITION_CLOSED` | `PARTIAL` — realized PnL unverifiable |
| `EQUITY_SNAPSHOT` | `DEGRADED` — portfolio state unverifiable |
| `MARKET_NEW_ORDER` | `DEGRADED` — queue depth inaccurate |
| `MARKET_CANCEL` | `DEGRADED` — queue depth inaccurate |
| `QUEUE_PROGRESSION` | `PARTIAL` — queue advancement untracked |
| `CERTIFICATION_VERDICT` | `INVALID` — no formal verdict emitted |
| `GOVERNOR_TELEMETRY` | No impact |

---

## Section 4 — Implementation Gap Analysis

### Implemented (9 types, partial conformance)
- `MARKET_NEW_ORDER` → `MarketEventType::NewOrder` ✓
- `MARKET_TRADE` → `MarketEventType::Trade` ✓
- `MARKET_CANCEL` → `MarketEventType::Cancel` ✓
- `ORDER_INTENT` → `EventType::OrderIntent` ✓
- `ORDER_ENTERED_QUEUE` → `EventType::OrderEnteredQueue` ✓
- `QUEUE_PROGRESSION` → `EventType::QueueProgression` ✓
- `PARTIAL_FILL` → `EventType::PartialFill` ✓
- `ORDER_FILLED` → `EventType::OrderFilled` ✓
- `MARKET_EVENT` → **NON-CANONICAL** — this is a wrapper, not a type. Must be decomposed into `MARKET_NEW_ORDER`, `MARKET_TRADE`, or `MARKET_CANCEL`.

### Not Implemented (7 types)
- `ORDER_CANCELLED` — ARTIFACT-009 (pending)
- `POSITION_OPENED` — ARTIFACT-010 (pending)
- `POSITION_CLOSED` — ARTIFACT-011 (pending)
- `EQUITY_SNAPSHOT` — ARTIFACT-012 (pending)
- `REPLAY_SESSION_START` — ARTIFACT-013 (pending)
- `REPLAY_SESSION_END` — ARTIFACT-014 (pending)
- `CERTIFICATION_VERDICT` — ARTIFACT-015 (pending)
- `GOVERNOR_TELEMETRY` — ARTIFACT-016 (pending)

### Non-Canonical Types to Retire
- `EventType::MarketEvent` in [`inspector.rs`](services/api/src/inspector.rs:7) — this is a wrapper type, not a canonical event type. It must be replaced by the three `MARKET_*` types.
- `RawEventType` in [`services/api/src/market_adapter.rs`](services/api/src/market_adapter.rs:6) — internal parsing type, must not leak into canonical event streams.

---

## Section 5 — Enforcement Rules

1. The `EventType` enum in [`services/api/src/inspector.rs`](services/api/src/inspector.rs:7) must be expanded to all 16 canonical types before any new replay endpoint is added.
2. `EventType::MarketEvent` must be removed and replaced with `MARKET_NEW_ORDER`, `MARKET_TRADE`, `MARKET_CANCEL`.
3. No event may be emitted by a layer other than its registered authority layer.
4. `CERTIFICATION_VERDICT` is the only event type permitted to carry a `certification_state` field.
5. Root events (`REPLAY_SESSION_START`, `ORDER_INTENT`, market events) must have `parent_sequence_id: null`.
6. Terminal events (`ORDER_FILLED`, `ORDER_CANCELLED`, `REPLAY_SESSION_END`, `CERTIFICATION_VERDICT`) must not have child events.