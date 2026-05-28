# ChronoSentiment

**Deterministic market replay and evaluation infrastructure for certifiable financial simulation.**

[![Constitutional Architecture Gates](https://github.com/nikhil/ChronoSentiment_MEGA_FINAL/actions/workflows/constitution.yml/badge.svg)](https://github.com/nikhil/ChronoSentiment_MEGA_FINAL/actions/workflows/constitution.yml)

## Overview

Most financial simulation systems suffer from *silent semantic drift*: identical strategies produce disparate outcomes under structurally inconsistent replay conditions. ChronoSentiment is engineered to solve this hidden infrastructure problem. 

ChronoSentiment is a highly constrained, deterministic market replay and evaluation infrastructure designed to enforce scientific reproducibility and structural integrity in algorithmic discovery. By establishing strict constitutional isolation between mechanical search and market causality, ChronoSentiment guarantees that optimization remains mathematically pure while evaluation remains completely deterministic.

## Constitutional Architecture

![Architecture Snapshot](file:///Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/architecture_snapshot_1779950072920.png)

ChronoSentiment enforces a rigid 4-layer acyclic topology, ensuring that financial domain vocabulary never leaks into mathematical search algorithms, and simulation mechanics never leak into strategy evaluation:

1. **`infrastructure/optimization`**: A mathematically pure evolutionary search engine (GA). It maintains absolute seed determinism and operates entirely blind to financial semantics (no knowledge of assets, regimes, or PnL).
2. **`financial/core`**: The chronological substrate. Enforces market causality, strict monotonic timestamps, deterministic latency execution bounds, and perfectly reproducible event trace hashing.
3. **`financial/strategies`**: The semantic bridge. Translates pure candidate bytes into domain logic, manages strategy behavior, and guarantees lossless semantic projections.
4. **`infrastructure/observatory/api`**: External observability, tracking, and serialization.

*Dependency Law: The pure optimization layer must never import domain logic. Violation of this dependency direction triggers an immediate CI failure.*

## Adversarial Replay Certification

We don't just test the "happy path." ChronoSentiment relies on an **Adversarial Fixture Philosophy** for verification. 

Using raw, captured market slices and malformed synthetic streams (timestamp collisions, missing ticks, reversed chunks, negative latency), ChronoSentiment forces the simulation through a rigid chronology normalizer. If equivalent inputs fail to converge to an **identical canonical replay hash**, the system blocks the build.

**Semantic Drift Authorization:** Any change that modifies a canonical replay hash, classification boundary, or sequence trace is considered a constitutional violation unless explicitly documented, authorized, and recertified.

## Core Capabilities

- **Lossless State Snapshots**: Simulate serialize/deserialize roundtripping of active execution bounds to enable distributed, pause-and-resume cluster architectures.
- **Chunking Invariance**: Replay stability is guaranteed whether data is streamed individually, batched, or completely loaded in-memory.
- **Certifiable Semantic Equivalence**: Robust testing guarantees that the classification, edge scoring, and regime assignments for a strategy do not drift across execution boundaries.

## Quick Start (Development)

Run the fast constitutional gates to verify vocabulary isolation and basic topology constraints:
```bash
./scripts/ci_fast.sh
```

Run the complete replay certification loop (including single-threaded release mode to verify hash stability):
```bash
./scripts/ci_full.sh
```

## Documentation

- [Constitutional Architecture & Drift Rules](docs/constitution/architecture.md)
- [Optimization Capability Details](docs/capabilities/optimization.md)
- [Runtime Replay Capability Details](docs/capabilities/runtime_replay.md)
- [Strategy Evaluation Capability Details](docs/capabilities/strategy_evaluation.md)
