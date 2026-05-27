# Observatory Authority

## Purpose
This document formalizes the mechanical boundary of the Observatory layer within ChronoSentiment. Its primary function is to guarantee that **observability never becomes operational authority**.

Observability systems inherently pressure architectures toward semantic interpretation, decision-making, and adaptive governance. The Observatory must remain strictly evidentiary.

## Allowed Authority
The Observatory is explicitly permitted to hold authority over:
- **Replay Attestation**: Cryptographically or deterministically verifying that an execution trace matches an artifact.
- **Evidentiary Reconstruction**: Rebuilding states from the causal history for inspection.
- **Deterministic Verification**: Proving that the platform behaves correctly against fixed fixtures.
- **Causal Visualization**: Displaying the chronology of events without altering them.
- **Bounded Diagnostics**: Reporting mechanical failures (e.g., OOM, networking bounds) without intervening.

## Forbidden Authority
The Observatory is constitutionally forbidden from holding authority over:
- **Adaptive Remediation**: Automatically fixing, adjusting, or compensating for runtime failures.
- **Semantic Optimization**: Altering the execution pathway to improve business outcomes or latency.
- **Replay Mutation**: Changing the state, ordering, or outcome of a replay execution.
- **Automated Governance**: Dictating routing rules, tranche escalations, or architectural state changes based on telemetry.
- **Causal Reinterpretation**: Explaining the "meaning" or "intent" of an event sequence.
- **Execution Influence**: Injecting metadata or side-effects that modify the Core's behavior.
- **Ontology Correction**: Automatically expanding or resolving semantic types based on observed data.

## Invariant
**The Observatory certifies replay equivalence; it does not interpret it.**
