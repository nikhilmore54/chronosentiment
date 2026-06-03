# Coralys Platform & ChronoSentiment Adapter

## Repository Philosophy

This repository is the home of **Coralys**, a domain-agnostic AI simulation and optimization platform, and its first domain-specific adapter, **ChronoSentiment**.

The repository prioritizes:
- deterministic replay equivalence
- constitutional authority isolation
- execution‑order certification
- semantic stability across migrations
- replay‑certified topology evolution
- governance‑first infrastructure discipline

All architectural evolution proceeds through:
1. certification
2. replay validation
3. topology freeze
4. governance review
5. incremental extraction

**Deterministic simulation, ecological memory, and generic mathematical optimization infrastructure.**

[![Constitutional Architecture Gates](https://github.com/nikhil/ChronoSentiment_MEGA_FINAL/actions/workflows/constitution.yml/badge.svg)](https://github.com/nikhil/ChronoSentiment_MEGA_FINAL/actions/workflows/constitution.yml)

## Overview

Most AI simulation systems suffer from *silent semantic drift*: identical strategies produce disparate outcomes under structurally inconsistent replay conditions. 

**Coralys** is engineered to solve this hidden infrastructure problem. It provides a generic, pure platform comprising Ecological Physics, Optimization Mechanics, and Simulation Contracts.

**ChronoSentiment** is a highly constrained, deterministic market replay and evaluation adapter designed to enforce scientific reproducibility and structural integrity in financial algorithmic discovery. It is built entirely on top of Coralys.

## Constitutional Architecture

The architecture enforces strict separation between generic platform capabilities and domain-specific adapters:

```text
Coralys Platform
├── Ecology Physics (Topology, Memory, Deformation)
├── Optimization Mechanics (Population Management, Evolution Loop)
├── Simulation Contracts
├── Decision Contracts
└── Recommendation Contracts

Adapters
├── ChronoSentiment (Finance Adapter)
└── UltraCrew (Upcoming Workforce Adapter)
```

1. **`coralys-moga` / `coralys-ecology`**: The pure domain-agnostic platform crates. They maintain absolute seed determinism and operate entirely blind to financial semantics (no knowledge of assets, regimes, or PnL).
2. **`adapters/chronosentiment`**: The semantic bridge. Translates pure candidate bytes into financial domain logic, manages strategy behavior, and guarantees lossless semantic projections.

*Dependency Law: Coralys platform crates must NEVER import domain logic from ChronoSentiment or any adapter. Violation of this dependency direction triggers an immediate CI failure.*

## Adversarial Replay Certification

We don't just test the "happy path." Coralys and ChronoSentiment rely on an **Adversarial Fixture Philosophy** for verification. 

Using raw, captured market slices and malformed synthetic streams, the system forces the simulation through a rigid chronology normalizer. If equivalent inputs fail to converge to an **identical canonical replay hash**, the system blocks the build.

**Semantic Drift Authorization:** Any change that modifies a canonical replay hash, classification boundary, or sequence trace is considered a constitutional violation unless explicitly documented, authorized, and recertified.

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
