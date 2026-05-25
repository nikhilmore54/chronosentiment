# Operational Model

This document clarifies the semantic boundaries of ChronoSentiment Core.

## The Observer, Not The Actor

ChronoSentiment Core is an infrastructure instrument built for **reproducible execution divergence isolation**.

### What It Is:
- A deterministic replay engine.
- A certification layer for trace artifacts.
- A mechanical tool for locating exactly where two executions diverged.

### What It Is Not:
- A trading engine.
- An AI runtime or market cognition platform.
- A generalized observability platform.
- A stream-processing framework.

## The Architecture of Trust

Live systems produce messy, out-of-order, chaotic chronology. 
ChronoSentiment does not try to participate in that chaos.

Instead, the operational model dictates:
1. Live systems capture their session streams.
2. The streams are exported as bounded JSONL substrates.
3. ChronoSentiment Core replays the substrate in a pristine laboratory environment.
4. ChronoSentiment passively certifies that the execution matches the expected trace footprint.

This dependency asymmetry (Live Systems depend on Core's certification; Core depends on nothing) guarantees that ChronoSentiment remains a pure, auditable, and uncorrupted source of truth.
