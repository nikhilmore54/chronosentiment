# ChronoSentiment MVP System Guarantees

This document outlines the formal guarantees and architectural invariants of the ChronoSentiment execution simulation engine.

## 1. Determinism Guarantee
The ChronoSentiment engine is **100% deterministic**. Given the same set of input events (Market events and Order intents) and the same execution mode, the engine will produce **byte-for-byte identical output** across all runs, regardless of the environment, system time, or hardware.

*   **Fingerprint**: Every simulation run can be certified via a hash of the event sequence.
*   **No Randomness**: The engine does not use any source of entropy.
*   **No System Clock**: The engine uses discrete simulation time (event-time).

## 2. Replay Fidelity
Any past simulation state can be **perfectly reconstructed** by replaying the event stream. 
*   **Fact of Record**: The event log is the only source of truth.
*   **Identity Check**: `Simulate(Inputs) == Replay(Events_from_Simulate)`.

## 3. Causal Integrity
Every execution event in the system has a **provable causal link** to its origin.
*   **Parent Linking**: Every non-root event (Fills, Queue movements) includes a `parent_sequence_id`.
*   **DAG Structure**: The event stream forms a Directed Acyclic Graph (DAG) ensuring no circular dependencies or paradoxes.

## 4. Execution Physics
The engine enforces strict Market Microstructure rules:
*   **FIFO Correctness**: Orders at the same price level are processed in the strict order of their arrival.
*   **Latency Enforcement**: No order can be executed before its arrival time plus the fixed system latency.
*   **Inventory Conservation**: Total filled quantity across all events must exactly match the trade volume recorded in outcomes.

## 5. API Isolation
The API layer is a **stateless projection layer**.
*   **No Business Logic**: The API cannot modify the core simulation behavior.
*   **Read-Only Purity**: Calling inspection or timeline endpoints does not mutate the engine state.
*   **Independent Runs**: Successive API calls do not share state.

## 6. GA Stability
The Genetic Algorithm (GA) evolution is **strictly stable**.
*   **Stable Search**: Given the same fitness function and input, the GA will converge to the identical `best_config`.
*   **Temporal Purity**: The GA never leaks future information into past simulation steps.

## ⚡ Non-Guarantees (Explicit Exclusions)
*   **Performance Scaling**: This MVP is optimized for correctness, not high-frequency throughput.
*   **External Synchronization**: The engine is isolated; it does not synchronize with live market clocks.
*   **Strategy Alpha**: The GA provides a framework; actual strategy profitability depends on user-defined inputs.

---
**Certified by: v1.0-deterministic-core**
**Date: 2026-03-22**
