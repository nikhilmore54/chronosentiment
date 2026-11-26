# 📘 ChronoSentiment

## Product System Design (PSD v1.1 — Prose-First, Deterministic, Simulation Architecture)

---

# 1. System Intent

Before understanding how ChronoSentiment is built, it is important to understand **what kind of system it is trying to be**.

ChronoSentiment is not designed as a trading platform, nor as a traditional backtesting engine. It is designed as a **closed, deterministic simulation system** whose purpose is to model how trading strategies behave when they are exposed to the realities of execution.

In most existing systems, execution is either assumed or abstracted. A decision is made, and a result is recorded. The intermediate process—where time passes, other participants act, and opportunities change—is compressed or ignored.

ChronoSentiment expands this missing layer.

It introduces a system in which execution is not assumed, but **constructed internally through a sequence of interactions**. Every outcome is therefore not a direct consequence of a decision, but a result of how that decision behaves when it enters a simulated environment governed by time, competition, and liquidity.

This architectural choice has deep implications. It means that:

* the system must be internally consistent
* all behavior must be derived, not injected
* outcomes must be explainable through process

To support this, the system is designed as a **deterministic, event-driven machine**. It does not observe execution—it **computes execution** under defined rules.

---

# 2. System Boundary

To preserve this internal consistency, ChronoSentiment operates within a clearly defined boundary.

The system accepts inputs such as historical market data, strategy definitions, and simulation parameters. It processes these inputs within its own environment and produces outputs such as execution outcomes, event logs, replay sessions, and analytical summaries.

```text
Inputs:
- Market Data
- Strategy Config
- Simulation Parameters

System:
- Deterministic Simulation Engine

Outputs:
- Execution Results
- Event Streams
- Replay Sessions
- Analytics
```

What is equally important is what the system does *not* do.

It does not connect to live markets. It does not route real orders. It does not depend on external execution infrastructure. These are not missing features—they are deliberate exclusions.

By isolating itself from live systems, ChronoSentiment ensures that:

* all outcomes are controlled
* all behavior is reproducible
* all analysis remains internally consistent

This aligns with the core MVP philosophy that prioritizes **credibility of simulation over completeness of infrastructure** .

---

# 3. Core Architectural Principle

At the heart of ChronoSentiment lies a single principle that governs all system behavior:

> **Everything that happens is an event, and every event is processed deterministically.**

This principle is not a technical implementation detail—it is the foundation of the system’s reliability.

In many systems, state can change implicitly. Background processes, asynchronous operations, or external dependencies can alter behavior in ways that are difficult to trace. ChronoSentiment eliminates this ambiguity.

Every change in the system is explicitly represented as an event. These events are processed in a strict order, and each event produces a well-defined transition in system state.

```text
Event → State Transition → Updated State → Next Event
```

This creates a system where:

* nothing happens implicitly
* every outcome has a traceable cause
* identical inputs always produce identical outputs

This deterministic structure transforms the system into a **controlled experimental environment**, where behavior can be studied with precision rather than inferred from approximations .

---

# 4. High-Level Architecture

With the core principle established, the system can be understood as a set of interconnected components, each responsible for a specific aspect of the simulation.

At a high level, the architecture is organized around the flow of information and the transformation of intent into outcome.

```text
Market Data Layer
        ↓
Simulation Kernel (Event Engine)
        ↓
Core Simulation Subsystems
        ↓
Analytics + UX Layer
```

The system begins with market data, which defines the environment. This data is processed by the simulation kernel, which orchestrates all activity. Within this framework, multiple subsystems interact to produce behavior.

What is important is not just the presence of these components, but how they relate to each other.

The system is not a pipeline where data flows linearly. It is a **dynamic system of interactions**, where:

* strategies respond to market conditions
* orders interact with simulated participants
* execution outcomes feed back into the system

Each subsystem contributes to a larger process in which outcomes are continuously shaped by interaction.

---

# 5. Subsystem Design

To fully understand the system, each subsystem must be examined not only in terms of what it does, but how it contributes to the overall behavior.

---

## 5.1 Simulation Kernel (The Backbone of the System)

The simulation kernel is the central orchestrator of the system. It does not perform trading logic or execution modeling itself. Instead, it governs **how and when all other components operate**.

Its primary role is to manage the lifecycle of events.

Every event—whether it originates from market data, strategy evaluation, or execution processing—is passed through the kernel. The kernel ensures that these events are:

* processed in a deterministic order
* applied consistently to system state
* capable of producing subsequent events

```text
Fetch Event → Process → Update State → Emit New Events
```

The kernel also controls the concept of time within the system. Time does not flow continuously. It advances only when events are processed. This ensures that:

* no transitions occur without cause
* all changes are observable
* replay remains exact

Without the kernel, the system would devolve into a collection of loosely connected components. With it, the system behaves as a **coherent, ordered machine**.

---

## 5.2 Strategy Engine (From Observation to Intent)

The strategy engine is responsible for interpreting market conditions and producing decisions.

However, within ChronoSentiment, a decision is not equivalent to a trade. The strategy engine does not execute anything. It produces **intent in the form of orders**.

This distinction is critical.

The strategy observes the market, evaluates conditions, and decides what it would like to do. But once that decision is expressed, it leaves the control of the strategy and enters the simulation environment.

This ensures that:

* strategies do not assume execution
* decisions are separated from outcomes
* behavior remains realistic

The output of the strategy engine is therefore not a result, but a structured expression of intent.

---

## 5.3 Execution Simulation Engine (Where Intent Meets Reality)

The Execution Simulation Engine (ESE) is the core of the system’s differentiation.

It is here that the system transitions from intention to interaction.

When an order enters the ESE, it does not immediately execute. Instead, it becomes part of a competitive environment where multiple factors determine its outcome.

The engine models:

* queue positioning (based on arrival time)
* interaction with available liquidity
* incremental execution through partial fills

```text
Order → Queue Entry → Queue Evolution → Execution → Completion
```

Execution is therefore not a single step, but a process that unfolds over time.

A key insight within this system is that execution depends on a chain of dependencies:

```text
Latency → Queue Position → Liquidity → Execution
```

This chain ensures that outcomes are not arbitrary. They are the result of how an order navigates a structured environment .

---

## 5.4 Latency Model (Introducing Time as a Constraint)

The latency model introduces a delay between when a decision is made and when the corresponding order becomes active.

This delay is deterministic, but its impact is significant.

During this interval:

* the market continues to evolve
* other participants act
* queue conditions change

As a result, the order may arrive in a different context than the one in which it was created.

Latency therefore does not simply delay execution. It **alters the conditions under which execution occurs**.

This creates a separation between:

* decision context (what the strategy saw)
* execution context (what the order encounters)

And this separation is one of the primary drivers of divergence between expected and actual outcomes.

---

## 5.5 Market Model / Order Flow Simulator (Creating Competition)

A system without competition would be unrealistic.

The market model introduces this competition by simulating the behavior of other participants.

These participants generate:

* trades that consume liquidity
* cancellations that alter queue depth
* new orders that increase competition

The goal of this component is not to perfectly replicate market behavior, but to ensure that the user’s order does not exist in isolation.

Instead, it must operate within a **dynamic and evolving environment**.

This transforms execution from a deterministic calculation into a **conditional process influenced by external activity**.

---

## 5.6 Portfolio Engine (Capturing Consequences)

The portfolio engine is responsible for tracking the financial consequences of execution.

It updates:

* positions
* capital
* profit and loss

based on execution events.

Importantly, the portfolio is not influenced by intent. It responds only to what has actually been executed.

This ensures that the system reflects reality:

* unfilled orders have no financial impact
* partial fills produce partial exposure
* execution timing affects outcomes

The portfolio is therefore a **mirror of execution**, not of strategy decisions.

---

## 5.7 Replay Engine (Restoring the Path to Outcome)

The replay engine enables the system to reconstruct the full sequence of events that led to any outcome.

This is not a secondary feature—it is a fundamental requirement.

Because all behavior is event-driven and deterministic, the system can:

* store the complete event sequence
* reconstruct state at any point in time
* allow users to navigate through the simulation

This transforms the system from a result generator into an **explainable environment**, where every outcome can be traced back to its causes.

---

# 6. Data Flow Architecture

To understand how these subsystems interact, it is helpful to view the system as a continuous flow of transformations.

```text
Market Data
   ↓
Strategy Decision
   ↓
Order Creation
   ↓
Latency
   ↓
Queue Entry
   ↓
Market Interaction
   ↓
Execution
   ↓
Portfolio Update
   ↓
Replay + Analysis
```

Each stage in this flow represents a transformation.

What begins as a decision is gradually shaped by time, competition, and liquidity until it becomes an outcome.

No stage is optional. No transition is hidden.

This ensures that the system remains transparent and explainable at every level.

---

# 7. Event Architecture

Events are the fundamental building blocks of the system.

Every change—whether it is a market update, a strategy decision, or an execution outcome—is represented as an event.

Each event contains:

* a type (what happened)
* a timestamp (when it happened)
* a sequence identifier (ordering)
* a payload (context)

```json
{
  "event_id": "uuid",
  "timestamp": 123456,
  "sequence_id": 42,
  "type": "OrderCreated",
  "payload": {}
}
```

Events are immutable. Once created, they cannot be altered.

This immutability ensures that:

* the system has a reliable history
* replay is exact
* debugging is precise

The event model therefore acts as the **backbone of both behavior and explainability**.

---

# 8. State Architecture

If events represent change, state represents truth.

At any moment, the system maintains a representation of:

* market conditions
* active orders and queue positions
* portfolio status

This state is not stored independently. It is derived from the sequence of events that have occurred.

> State = function(previous state + event)

This ensures that:

* no hidden state exists
* all state transitions are traceable
* replay can reconstruct state exactly

State is therefore not an independent entity—it is a **consequence of history**.

---

# 9. Determinism Enforcement

Determinism is enforced through a combination of architectural constraints.

The system uses:

* seeded randomness (for controlled variability)
* strict event ordering
* a virtual clock

At the same time, it explicitly avoids:

* real-time dependencies
* uncontrolled asynchronous behavior
* external sources of randomness

These constraints ensure that the system behaves consistently across runs.

Determinism is not just about repeatability. It is about **trust**.

It allows users to:

* compare strategies reliably
* debug behavior precisely
* understand cause and effect without ambiguity

---

# 10. UX-System Mapping

The user interface is not an independent layer. It is a projection of the system’s internal structure.

Each major UI component corresponds directly to a part of the system.

The Timeline represents the sequence of events over time.
The Trade Inspector reconstructs individual trades across decision, execution, and outcome layers.
Analytics summarize the effects of execution on performance.

This mapping ensures that:

* the UI reflects reality
* insights are grounded in system behavior
* users can move from observation to understanding

In this sense, the UI becomes an **instrument for exploring the system**, rather than a layer for displaying results .

---

# 11. Scalability Considerations

While the MVP is designed as a single-node system, the architecture is intended to evolve.

In its initial form, the system prioritizes:

* simplicity
* determinism
* clarity of behavior

As the system grows, it can be extended to support:

* distributed event processing
* parallel simulations
* scalable data storage

However, these extensions must preserve the core principle of determinism.

Scalability is therefore not just a technical challenge—it is a **constraint-aware evolution of the system**.

---

# 12. Risks in System Design

The design introduces several risks that must be managed carefully.

Non-determinism may arise if event ordering is not strictly enforced.
Execution realism may degrade if the market model is too simplistic.
Performance issues may occur due to the volume of events generated.

Each of these risks is directly tied to the system’s core characteristics.

Mitigation therefore involves reinforcing those characteristics:

* enforcing strict ordering
* calibrating simulation models
* optimizing event processing

The goal is not to eliminate complexity, but to ensure that complexity remains **controlled and explainable**.

---

# 13. Traceability to Product Intent

Every architectural decision in this system can be traced back to a product requirement.

Execution realism is implemented through the ESE, latency model, and market simulation.
Determinism is enforced through the kernel and event model.
Replay and explainability are enabled through event logging and state reconstruction.

This traceability ensures that the system remains aligned with its purpose.

It is not a collection of components. It is a **coherent realization of a product philosophy** .

---

# 14. Final System Characterization

ChronoSentiment is best understood not as a tool, but as a system with a specific way of thinking.

It treats execution as a process.
It treats outcomes as emergent.
It treats strategies as intentions interacting with reality.

From an architectural perspective, it is:

> a **deterministic, event-driven simulation system that constructs execution through interaction**

---

# 🔚 Final Outcome

This version of the PSD is now:

* **self-explanatory to external readers**
* **aligned with PRD, SRS, and MVP**
* **narrative-first, not structure-first**
* **ready for stakeholder + engineering use**
