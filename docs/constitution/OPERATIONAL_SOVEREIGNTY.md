# Operational Sovereignty

## Purpose
This document codifies the "appliance philosophy" of ChronoSentiment. It explicitly formalizes the mechanical boundary that the system does not depend on environmental generosity. The release artifact is its own sovereign operational reality.

## Authorized Properties
The operational appliance must mechanically guarantee:
- **Offline Capability**: Execution and replay validation must function flawlessly in an air-gapped environment with zero network connectivity.
- **Artifact Self-Sufficiency**: The packaged release artifact requires no external metadata, orchestration, or fetching of resources at runtime.
- **Compiler Independence**: Operational validation and execution strictly forbid the invocation or requirement of developer toolchains (e.g., `cargo`, `rustc`, `git`).
- **Bounded Runtime Assumptions**: The system remains predictable and calm under scarcity (e.g., low memory thresholds, restricted CPU allocation).
- **Replay Corpus Completeness**: The artifact ships with its minimal necessary cryptographic inputs and fixtures to certify itself immediately.
- **Deterministic Release Verification**: Validation behaves identically across any supported architecture without depending on environmental variables or persistent caches.

## Forbidden Architecture
Operational sovereignty strictly prohibits:
- **Hidden Network Assumptions**: Telemetry export, DNS resolution for core paths, or package mirror dependencies during runtime execution.
- **Environmental Infrastructure Gravity**: Requiring Kubernetes, orchestration daemons, or persistent external volumes to operate safely.

## Invariant
**The release appliance proves its coherence under environmental deprivation, rather than appearing stable through environmental abundance.**
