# Optimization Boundary Contract

## Purpose
This document defines the strict constitutional boundary for the `chronosentiment_optimization` crate. 
The optimization crate is a mathematically pure, domain-blind evolutionary search engine. 

## What Optimization MAY Know
* Candidate ranking and tournament selection mechanics.
* Mutation and crossover primitives.
* Generation iteration logic.
* Diverse selection strategies (e.g., orthogonal behavioral grouping).
* The concept of an opaque `FitnessEvaluator<T>`.

## What Optimization MUST NEVER Know
* **Financial Data**: Candles, prices, volumes, bids, asks.
* **Domain Events**: Trade executions, slippage, PnL calculations, win rates.
* **Orchestration**: Replay engines, tick loops, simulation timelines.
* **Contextual Features**: Regimes (bull, bear, sideways), asset pairings.

## The Evaluator Contract
The core contract that binds the optimizer to the real world is the `FitnessEvaluator`:
```rust
pub trait FitnessEvaluator<T> {
    type Evaluation;
    fn evaluate(&self, candidate: &T) -> Self::Evaluation;
}
```
The financial domain must implement this trait. It accepts an opaque candidate genome and returns an evaluation payload containing a primary scalar (`fitness`) alongside any domain-specific annotations required for downstream analysis.

## Deterministic Guarantees
1. **Seed Equivalence**: Supplying the same seed to `GaConfig` must result in an identically initialized population.
2. **Crossover Equivalence**: Recombination logic is purely a function of the parent genomes and the shared RNG state.
3. **Generational Replay**: An entire evolutionary run (N generations) must trace exactly the same fitness trajectories and emit exactly the same candidates when restarted with the same initial seed and a deterministic evaluator.

## Prohibited Ontology
Words and structs that are explicitly forbidden from the `chronosentiment_optimization` source code:
* `PnL`
* `Replay`
* `Candle`
* `Regime`
* `Execution`
* `SignalAlpha`
* `Trade`
* `Market`
