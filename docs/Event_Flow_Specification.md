# 📘 ChronoSentiment

## Event Flow Specification (v2.0 — Canonical, Deterministic, Execution-Complete)

---

# 1. System Premise

ChronoSentiment operates as a **closed, deterministic, event-driven system**.

Every outcome is produced through a causal sequence:

```text
Intent → Delay → Competition → Position → Interaction → Execution → Outcome
```

No execution is assumed.
All execution is **constructed internally** through event interaction.

---

# 2. Unified System Flow (Authoritative)

```text
Market Data ─┐
             ├──→ Ingestion Layer → Kernel → Sequencer
User Intent ─┘

→ Latency Layer
→ Execution Simulation Engine (Queue + Interaction)
→ Execution Events
→ Portfolio Engine
→ Event Store (append-only)

→ Snapshot + State Hash (validation layer)

→ WebSocket Stream
→ UI Projection
```

---

# 3. Core Architectural Principles

---

## 3.1 Event Exclusivity

> No state change occurs without an event

---

## 3.2 Deterministic Ordering

```text
(timestamp, sequence_id) → total order
```

---

## 3.3 State Derivation

```text
State = reduce(events)
```

---

## 3.4 Replay Guarantee

```text
Same input → same event stream → same state → same outcome
```

---

## 3.5 Closed System

* Market is internalized
* Time is virtual
* Execution is computed

---

# 4. Dual Ingress Model (User + Market)

---

## 4.1 User Intent Ingress

### API Input

```json
{
  "type": "CreateOrder",
  "session_id": "S1",
  "instrument": "NSE:INFY",
  "side": "BUY",
  "quantity": 100,
  "price": 1500
}
```

---

### Event

```json
{
  "type": "OrderIntentCreated",
  "order_id": "O123",
  "timestamp": T0
}
```

---

## 4.2 Market Data Ingress (World Flow)

### Input

```json
{
  "type": "MarketEvent",
  "subtype": "TRADE | NEW_ORDER | CANCEL",
  "instrument": "NSE:INFY",
  "price": 1500,
  "quantity": 200,
  "exchange_timestamp": T_ex
}
```

---

## 4.3 Kernel Merge Rule

```text
User Events + Market Events → Single Ordered Stream
```

---

## 4.4 Ordering Priority

```text
1. exchange_timestamp
2. ingest_timestamp
3. sequence_id
```

---

## 4.5 Critical Guarantee

> Market events can pre-empt user orders
> → defines real queue competition

---

# 5. Kernel & Sequencing Layer

---

## 5.1 Responsibilities

* Assign sequence IDs
* Maintain virtual clock
* Enforce ordering
* Dispatch events

---

## 5.2 Event Structure

```json
{
  "event_id": "uuid",
  "sequence_id": 1024,
  "timestamp": 123456,
  "type": "EventType",
  "payload": {}
}
```

---

## 5.3 Guarantee

* Immutable events
* Strict ordering
* Single source of causality

---

# 6. Latency Layer (Temporal Divergence)

---

## 6.1 Purpose

Introduce separation between:

* decision time (T0)
* activation time (T1)

---

## 6.2 Events

```json
{
  "type": "LatencyScheduled",
  "order_id": "O123",
  "activation_time": T1
}
```

```json
{
  "type": "LatencyResolved",
  "order_id": "O123"
}
```

---

## 6.3 Principle

> Orders act in a **different reality than where they were decided**

---

# 7. Execution Simulation Engine (ESE)

---

## 7.1 Entry

```json
{
  "type": "OrderEnteredQueue",
  "order_id": "O123",
  "price": 1500,
  "queue_position": 7,
  "quantity_ahead": 1200
}
```

---

## 7.2 Queue Dynamics

Driven by:

* Market trades
* Cancellations
* New orders

---

## 7.3 Execution Condition

```text
IF:
queue_ahead == 0
AND liquidity_available > 0
```

---

## 7.4 Execution Events

### Partial Fill

```json
{
  "type": "PartialFill",
  "order_id": "O123",
  "filled_qty": 30,
  "remaining_qty": 70
}
```

---

### Full Fill

```json
{
  "type": "OrderFilled",
  "order_id": "O123",
  "avg_price": 1500
}
```

---

## 7.5 Core Principle

> Execution is a **multi-step process**, not a single event 

---

# 8. Portfolio Engine

---

## 8.1 Trigger

Execution events only

---

## 8.2 Event

```json
{
  "type": "PortfolioUpdated",
  "position": 100,
  "cash": -150000
}
```

---

## 8.3 Guarantee

> Portfolio reflects **executed reality only**

---

# 9. Event Store (Source of Truth)

---

## 9.1 Model

```text
Append-only event log
```

---

## 9.2 Stored Data

* sequence_id
* timestamp
* type
* payload

---

## 9.3 Guarantee

* immutable
* replayable
* ordered

---

## 9.4 Principle

> State is never stored—only reconstructed 

---

# 10. Snapshot System (Performance Layer)

---

## 10.1 Snapshot Event

```json
{
  "type": "SnapshotCreated",
  "sequence_id": 100000,
  "state_ref": "snapshot_100000",
  "state_hash": "abc123"
}
```

---

## 10.2 Contents

* order book
* queue state
* portfolio
* clock

---

## 10.3 Boot Flow

```text
Load snapshot → Apply remaining events → Reach current state
```

---

## 10.4 Constraint

> Snapshot is optimization, not truth

---

# 11. State Hash (Determinism Layer)

---

## 11.1 Event

```json
{
  "type": "RealityCheck",
  "sequence_id": 5000,
  "state_hash": "xyz789"
}
```

---

## 11.2 Trigger

* every N events
* critical transitions

---

## 11.3 Replay Validation

```text
Recompute hash → compare

IF mismatch:
→ HALT SYSTEM
```

---

## 11.4 Guarantee

> Determinism is **enforced, not assumed**

---

# 12. WebSocket Stream (Projection Layer)

---

## 12.1 Purpose

Real-time event streaming to UI

---

## 12.2 Example

```json
{
  "type": "OrderUpdated",
  "status": "PARTIALLY_FILLED"
}
```

---

## 12.3 Guarantee

* ordered
* session-scoped
* identical to kernel stream

---

# 13. UI Layer (Observer Only)

---

## 13.1 Role

Projection of:

* event stream
* state reconstruction

---

## 13.2 Components Mapping

| UI Component    | Source           |
| --------------- | ---------------- |
| Timeline        | event stream     |
| Trade Inspector | order events     |
| Portfolio View  | portfolio events |

---

## 13.3 Principle

> UI does not compute reality
> It **observes reality** 

---

# 14. Full Event Lifecycle

---

```text
1. Market/User Event Created
2. Kernel Sequencing
3. Latency Injection (if user order)
4. Queue Entry (ESE)
5. Market Interaction Loop
6. Execution (partial/full)
7. Portfolio Update
8. Event Persisted
9. Snapshot / Hash (periodic)
10. WebSocket Broadcast
11. UI Update
```

---

# 15. Failure & Edge Cases

---

## 15.1 No Liquidity

```json
{
  "type": "OrderUnfilled"
}
```

---

## 15.2 Latency Drift

```json
{
  "type": "LatencyExtended"
}
```

---

## 15.3 Queue Never Clears

```json
{
  "type": "OrderExpired"
}
```

---

## 15.4 Partial Execution Persistence

```text
PARTIALLY_FILLED → ACTIVE
```

---

# 16. System Guarantees (Final)

---

## 16.1 Causality

Every outcome = function of prior events

---

## 16.2 Determinism

No randomness without seed

---

## 16.3 Replay Fidelity

Exact reconstruction always possible

---

## 16.4 Isolation

Session-level independence

---

## 16.5 Explainability

Every trade = traceable event chain

---

# 🔚 Final Characterization

ChronoSentiment is:

> **A deterministic, event-sourced execution system where intent competes with market reality inside a unified causal timeline**

---

# 🚀 What This Enables

With this spec, an engineer (or GenAI agent) can now directly build:

* Kernel loop
* Sequencer
* Event store
* ESE
* Replay engine
* Streaming layer
