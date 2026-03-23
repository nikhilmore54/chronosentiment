# 📘 ChronoSentiment

## Service Boundary Definition

### **v1.3 — Enforcement-Complete, Interpretation-Locked, Self-Defending System**

---

# 1. Purpose

This document defines:

* **Ownership** — who controls each domain
* **Authority** — who can create events
* **Visibility** — who can read what
* **Interaction** — how services communicate
* **Enforcement** — how violations are prevented
* **Interpretation** — how rules must be understood

---

## 🔒 Core Position

This is not documentation.

> It is the **constitutional layer of the system**, defining what is allowed, what is impossible, and what is invalid.

---

# 2. Foundational System Laws (NON-NEGOTIABLE)

---

## Law 1 — Causality is Centralized

> Only the Sequencer defines event order
> No service may reinterpret causality

---

## Law 2 — Reality is Event-Only

> If it is not an event, it did not happen

---

## Law 3 — State is Private, Events are Public

> Services expose events, never internal state

---

## Law 4 — Time is Injected, Never Queried

> There is no “current time”—only event time

---

## Law 5 — Authority is Enforced, Not Trusted

> Permissions are enforced via schema + broker ACL

---

## Law 6 — Decisions Depend Only on History

> Services depend only on past events—not other services’ current state

---

## 🆕 Law 7 — Intent Must Not Be Validated Against Reality

> All well-formed intent must be accepted regardless of feasibility

---

## 🔒 Law 8 — Optimization Must Not Violate Causal Isolation

> The simulation core must remain a closed deterministic system.
> Optimization processes (e.g., Genetic Algorithms) must operate strictly outside the simulation boundary.

---

### Allowed

* schema validation
* format validation
* basic constraints

---

### Forbidden

* liquidity checks
* balance checks
* execution feasibility checks

---

## 🔒 Interpretation

> Intent is free.
> Execution is conditional.
> Reality is constructed later.

---

---

# 3. Service Authority Contracts (WRITE CONTROL)

---

## 3.1 Authority Matrix

| Service          | Allowed Event Types                                                     | Forbidden            |
| ---------------- | ----------------------------------------------------------------------- | -------------------- |
| Ingestion        | OrderIntentCreated, MarketEventReceived                                 | Execution, Portfolio |
| Sequencer        | Metadata only (sequence_id, timestamp)                                  | Business events      |
| Latency          | LatencyScheduled, LatencyResolved                                       | Queue, Execution     |
| ESE              | OrderEnteredQueue, QueueUpdated, PartialFill, OrderFilled, OrderExpired | Portfolio            |
| Market Simulator | MarketTrade, MarketOrderAdded, MarketOrderCancelled                     | User execution       |
| Portfolio        | PortfolioUpdated                                                        | Execution, Queue     |
| Event Store      | NONE                                                                    | ALL                  |
| Replay           | NONE                                                                    | ALL                  |
| Snapshot/Hash    | SnapshotCreated, RealityCheck                                           | ALL others           |
| API/WebSocket    | NONE                                                                    | ALL                  |

---

## 🔒 Enforcement

* Schema Registry validates ownership
* Broker ACL enforces topic permissions

---

---

## 3.2 Sequencer Interpretation Model

> The Sequencer is part of the **causal fabric**, not business logic

---

### It DOES

* assign `sequence_id`
* assign `event_timestamp`
* enforce total ordering

---

### It DOES NOT

* inspect payload meaning
* branch logic
* filter events

---

## 🔒 Rule

> Sequencer defines **when**, never **what or why**

---

---

## 3.3 Ingestion Interpretation (Intent Purity Enforcement)

> Ingestion is a **syntax validator**, not a semantic validator

---

### It MAY

* validate schema
* normalize input

---

### It MUST NOT

* evaluate feasibility
* access execution state
* reject based on market conditions

---

---

# 4. Read Boundary Contracts (VISIBILITY CONTROL)

---

## 4.1 Read Matrix

| Service          | Allowed Reads           | Forbidden             |
| ---------------- | ----------------------- | --------------------- |
| Ingestion        | NONE                    | ALL                   |
| Sequencer        | Envelope only           | Business data         |
| Latency          | Order intent events     | Order book, portfolio |
| ESE              | Market + latency events | Portfolio             |
| Market Simulator | Market state            | User orders           |
| Portfolio        | Execution events        | Queue, latency        |
| Replay           | All events              | External input        |
| API              | Projections only        | Raw event store       |

---

## 🔒 Rule

> No direct cross-service state access

---

## Interpretation

> Services observe history—not each other

---

---

# 5. Interaction Model

---

## 5.1 Allowed Communication

```text
Producer → Event → Event Bus → Consumer
```

---

## Forbidden

* direct API mutation
* shared DB
* implicit coupling

---

---

## 5.2 Read Access Modes

---

### Mode 1 — Event-Driven (DEFAULT)

```text
Event → Projection → Consumer
```

---

### Mode 2 — Synchronous Read (STRICT)

Allowed ONLY if:

* read-only
* derived state
* idempotent
* stateless

---

### Allowed

```text
UI → Portfolio → getBalance()
```

---

### Forbidden

```text
ESE → Portfolio → decision influence
```

---

## 🔒 Rule

> Reads must NEVER influence event generation

---

---

## 5.3 UI Responsibility Principle

> UI must represent uncertainty—not eliminate it

---

### Required

* show pending states
* allow uncertain actions

---

### Forbidden

* pre-validating trades
* simulating execution

---

---

# 6. State Mutation Rules

---

## Rule 6.1 — Event Exclusivity

> No state change without an event

---

## Rule 6.2 — Ownership

| Domain    | Owner     |
| --------- | --------- |
| Queue     | ESE       |
| Execution | ESE       |
| Portfolio | Portfolio |
| Ordering  | Sequencer |

---

## Rule 6.3 — Isolation

> No cross-domain mutation

---

---

# 7. Determinism & Temporal Integrity

---

## Rule 7.1 — No Hidden Time

* no system clock

---

## Rule 7.2 — Event-Time Injection

```text
event.timestamp
```

---

## Rule 7.3 — Seeded Randomness

* event-derived

---

## Rule 7.4 — Replay Equivalence

```text
Same events → same state → same hash
```

---

---

# 8. Infrastructure Enforcement

---

## 8.1 Schema Registry

* enforces event ownership
* validates schema

---

## 8.2 Broker ACL

```text
Ingestion → PUB order.intent
ESE → SUB order.latency → PUB execution.events
Portfolio → SUB execution.events → PUB portfolio.events
```

---

### Enforcement

* unauthorized publish → rejected
* unauthorized subscribe → denied

---

---

## 8.3 Determinism Harness

```text
Run same input twice → hashes must match
```

---

---

# 9. Boundary Violation Policy

---

## 🚫 Violations

* unauthorized event emission
* cross-domain mutation
* direct state access
* non-determinism

---

## Enforcement

* schema validation
* ACL
* CI tests
* runtime guards

---

---

# 10. Anti-Patterns (STRICTLY FORBIDDEN)

---

❌ logic leakage
❌ read leakage
❌ shortcut DB writes
❌ event forgery
❌ time-based logic
❌ intent validation against reality

---

---

# 11. Final System Characterization

ChronoSentiment is:

> **A deterministic, event-sourced, causally-ordered state machine
> with enforced authority, temporal purity, and intent-reality separation**

