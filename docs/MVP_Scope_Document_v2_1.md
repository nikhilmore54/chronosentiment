# 📘 ChronoSentiment

## MVP Scope Document (v2.1 — Simulation-First, Execution-Realism Focused)

---

## 1. MVP Philosophy

The ChronoSentiment MVP is not designed as a reduced version of a full trading platform. It is designed as a **focused validation of a single, critical capability**:

> the ability to simulate—and explain—how a trading strategy behaves under realistic execution constraints.

This distinction is fundamental.

Most systems in the current ecosystem either:

* evaluate strategies under simplified assumptions, or
* execute trades in real markets without prior validation of execution behavior

ChronoSentiment deliberately operates in the space between these two.

The MVP does not attempt to:

* connect to brokers
* enable live trading
* support a broad strategy ecosystem

Instead, it establishes a **credible execution simulation layer**, grounded in three principles:

* deterministic behavior
* observable execution mechanics
* fully replayable outcomes

The success of the MVP is therefore not measured by feature coverage, but by:

> how convincingly it reproduces execution behavior, and how clearly it explains that behavior to the user.

---

## 2. MVP Objectives

The MVP is expected to demonstrate three tightly coupled capabilities. These are not independent features, but interdependent qualities that together define product credibility.

---

### 2.1 Execution Realism

At the core of the MVP is the requirement that execution behaves differently from traditional simulation systems.

In most backtesting environments, execution is assumed. In ChronoSentiment, execution is **earned through interaction with the market model**.

This requires the system to simulate:

* **latency**, representing the delay between decision and market entry
* **queue-based execution**, where orders must wait behind others at the same price
* **partial fills**, where available liquidity may not satisfy the entire order
* **slippage**, where the final execution price differs from the intended price

These are not enhancements layered on top of simulation—they are the mechanism through which the system produces value.

---

### 2.2 Deterministic Simulation

To ensure that outcomes can be trusted and analyzed, the system must behave deterministically.

This means that:

> given the same inputs, the system must always produce the same outputs.

This requirement extends across the entire system:

| Aspect             | Deterministic Requirement |
| ------------------ | ------------------------- |
| Event ordering     | Fixed and reproducible    |
| Execution outcomes | Identical across runs     |
| Replay behavior    | Exact reconstruction      |

Determinism transforms the system from a probabilistic simulator into a **controlled experimental environment**, where outcomes can be studied and compared rigorously.

---

### 2.3 Explainability

Execution realism without explainability creates confusion rather than insight.

The MVP must therefore ensure that every outcome can be traced and understood.

A user should be able to:

* inspect any trade
* understand the sequence of events leading to it
* identify the specific factors that influenced execution

This includes visibility into:

* when an order was delayed
* how it moved through the queue
* what caused partial or missed fills

The goal is not only to simulate behavior, but to make that behavior **intelligible**.

---

## 3. MVP Scope Definition

The MVP is intentionally constrained to a **closed simulation loop**, where all inputs, events, and outcomes are controlled within the system.

---

### 3.1 In-Scope Capabilities

The following capabilities are required to establish a credible simulation environment.

---

#### 3.1.1 Simulation Kernel

The simulation kernel acts as the backbone of the system. It is responsible for orchestrating all activity within the simulation.

It must provide:

* an event-driven execution model, where all system changes are triggered by discrete events
* a virtual clock, allowing time to progress independently of real-world constraints
* a deterministic event queue, ensuring consistent ordering and processing

Without this layer, the system cannot guarantee reproducibility or coherence.

---

#### 3.1.2 Execution Simulation Engine (ESE)

The Execution Simulation Engine is the core of the product’s differentiation.

It is responsible for modeling how orders are processed within a market.

For the MVP, this includes:

* an **L2-level order book approximation**, sufficient to model available liquidity at each price level
* **FIFO queue modeling**, ensuring that orders are executed in the order they arrive
* **deterministic latency injection**, introducing delay between order creation and market entry
* **partial fill logic**, allowing orders to be executed incrementally based on available liquidity

The ESE is where the system moves from theoretical execution to **behavioral realism**.

---

#### 3.1.3 Market Model

The market model provides the environment in which execution occurs.

At a minimum, it must:

* replay historical price and volume data
* introduce a basic level of participant activity

This participant activity does not need to be highly sophisticated. Its purpose is to ensure that:

> the user’s order interacts with a dynamic environment, rather than a static dataset.

---

#### 3.1.4 Strategy Engine (Basic)

The MVP includes a minimal strategy engine, sufficient to generate trading decisions.

Strategies are defined using structured configuration (e.g., JSON), rather than code.

This ensures:

* controlled behavior
* deterministic signal generation
* reduced complexity

The goal is not to provide flexibility, but to enable **repeatable and interpretable decision-making**.

---

#### 3.1.5 Replay Engine

Replay is a foundational capability, not an auxiliary feature.

The system must allow:

* full reconstruction of any simulation
* step-by-step navigation through events
* accelerated playback (10x–50x, with higher speeds optional)

Replay ensures that outcomes are not only observable, but **revisitable and analyzable**.

---

#### 3.1.6 Trade Inspector (Core UX)

The Trade Inspector is the primary interface through which users understand system behavior.

It must present each trade across three layers:

| Layer           | Purpose                     |
| --------------- | --------------------------- |
| Decision Layer  | Why the trade was triggered |
| Execution Layer | How the order behaved       |
| Outcome Layer   | What result was produced    |

This layered view bridges the gap between strategy logic and execution reality.

---

#### 3.1.7 Timeline

The timeline provides a structured view of simulation events over time.

It enables users to:

* navigate across the simulation
* identify key moments
* correlate decisions with outcomes

This transforms the simulation from a static result into a **navigable sequence of events**.

---

#### 3.1.8 Basic Analytics

The MVP includes a minimal set of analytics, sufficient to evaluate performance without introducing unnecessary complexity.

These include:

* profit and loss (PnL)
* drawdown
* a basic measure of execution efficiency

The purpose of analytics at this stage is not depth, but **context**.

---

### 3.2 Explicitly Out of Scope

To maintain focus and ensure timely delivery, several areas are intentionally excluded.

These exclusions are not limitations—they are design decisions.

| Category               | Excluded Capability               | Rationale                                |
| ---------------------- | --------------------------------- | ---------------------------------------- |
| Trading Infrastructure | Broker integration, live routing  | Outside simulation scope                 |
| Strategy Ecosystem     | Code-based strategies, portfolios | Adds complexity without validation value |
| Market Modeling        | Full L3 reconstruction            | Not required for MVP realism             |
| Enterprise Features    | Collaboration, permissions        | Not needed for early validation          |

These exclusions protect the MVP from scope expansion and ensure alignment with its core objective.

---

## 4. Simulation Constraints (Non-Negotiable)

The credibility of the MVP depends on strict adherence to a set of constraints.

---

### 4.1 No Real Market Interaction

All behavior must be simulated.
The system must not connect to or depend on live market infrastructure.

---

### 4.2 Deterministic Execution

The system must not introduce uncontrolled randomness.

Where variability is required, it must be:

* seeded
* reproducible

---

### 4.3 Reproducibility

Every simulation must be replayable with exact fidelity.

This includes:

* event sequences
* execution outcomes
* system state transitions

---

### 4.4 Optimization Isolation

The MVP may include a constrained optimization loop (e.g., Genetic Algorithm) for strategy evolution. However, this loop must operate strictly outside the simulation execution.

Each simulation run must:

- execute independently
- remain fully deterministic
- be unaffected by prior or parallel runs

Optimization may use simulation outputs to generate new strategies, but must not influence execution behavior within a running simulation.

---

## 5. UX Scope (Aligned to MVP)

The user experience is designed to support understanding, not aesthetics.

---

### 5.1 Visibility of Execution

Users must be able to clearly observe:

* when orders are delayed
* how they enter and move through queues
* how they are filled over time

---

### 5.2 Traceability

Every trade must be traceable through:

* the sequence of events that produced it
* the execution path it followed

---

### 5.3 Controlled Interaction

The system must allow users to:

* replay simulations
* inspect individual events
* iterate on strategies

The UX is therefore a **tool for reasoning**, not just presentation.

---

## 6. Data Scope

The MVP relies on a constrained but sufficient data model.

---

### 6.1 Required Data

* historical price and volume data
* basic order book approximation (L2-level)

---

### 6.2 Optional Enhancements

If time permits, the system may include:

* synthetic participant modeling

This is considered an enhancement rather than a dependency.

---

## 7. MVP Success Criteria

The success of the MVP is evaluated at both the user level and the system level.

---

### 7.1 User-Level Success

A user should be able to:

* identify why execution differed from expectation
* explain the role of latency, queue position, and liquidity
* refine strategies based on these insights

---

### 7.2 System-Level Success

The system must demonstrate:

* deterministic outputs
* complete event traceability
* accurate and consistent replay

---

## 8. MVP Trade-offs

The MVP deliberately sacrifices breadth in order to achieve depth.

It prioritizes:

> fidelity of execution simulation over completeness of feature set.

This means:

* limited UI sophistication
* minimal analytics
* constrained strategy definition

These trade-offs are necessary to ensure that the core capability is delivered convincingly.

---

## 9. MVP Roadmap (8–12 Weeks)

Development proceeds in structured phases, each building on the previous.

| Phase       | Focus                                 | Outcome                   |
| ----------- | ------------------------------------- | ------------------------- |
| Weeks 1–3   | Simulation kernel + event model       | Deterministic foundation  |
| Weeks 4–6   | Execution engine + strategy engine    | Realistic execution       |
| Weeks 7–9   | UX (timeline + inspector) + analytics | User visibility           |
| Weeks 10–12 | Refinement + validation               | Stability and credibility |

---

## 10. Final Positioning

The ChronoSentiment MVP is not a simplified version of a larger system. It is a **focused validation of a foundational capability**.

It answers a single, critical question:

> *How would my strategy behave under realistic execution conditions?*

By answering this question with clarity, consistency, and transparency, the MVP establishes the basis for all future expansion.