# ChronoSentiment

**Reproducible execution divergence isolation.**

## What is ChronoSentiment?
ChronoSentiment is a deterministic infrastructure system. It provides a tightly constrained execution engine designed to guarantee byte-for-byte replay equivalence and strictly isolate operational divergence when that equivalence breaks.

## What problem does it solve?
In complex execution pipelines, small numerical coercions, ordering bugs, or cross-language serialization drift can silently poison execution state. Traditional debugging struggles to locate these non-crashing drift errors. ChronoSentiment solves this by answering one specific engineering question:

**Where did deterministic state equality first stop holding?**

It achieves this through:
- Strictly ordered, integer-scaled state ingestion.
- Deterministic trace artifacts (`trace_v1.json`).
- Passive manifest attestation.
- Mechanical isolation of the exact event index where equivalence fails.

## Evaluation Documentation Suite
The operational behavior and boundaries of the system are heavily documented. Please review these before operating the system.

- [Getting Started](GETTING_STARTED.md) - 10-minute installation and onboarding guide.
- [Known Limitations](KNOWN_LIMITATIONS.md) - Explicit boundaries and failure semantics.
- [Operational Model](OPERATIONAL_MODEL.md) - The Observer vs. The Actor.
- [Workflow Guide](WORKFLOW_GUIDE.md) - Replay → Certify → Inspect.
- [Divergence Walkthrough](DIVERGENCE_WALKTHROUGH.md) - Guided deterministic failure isolation.
- [Architecture](ARCHITECTURE.md) - Bounded component design.
- [Governance](GOVERNANCE.md) - Scope enforcement policy.
- [FAQ](FAQ.md) - Frequently asked questions.

## Installation
Requires Python 3.10+ and Rust (cargo).

```bash
# Build the deterministic core engine (release mode)
./chrono install
```

## Quickstart
Run the pre-configured replay demo to verify the engine operates deterministically on your host architecture.

```bash
./chrono demo
```

This will:
1. Replay a canonical substrate.
2. Emit an architectural artifact footprint.
3. Certify that the newly generated footprint matches the expected canonical determinism bounds exactly.

## Certification Workflow
To replay a known substrate and passively certify its execution footprint:

```bash
./chrono replay osc_50_1.0 rolling_50 --substrate-file core/chronology/historical/.../file.jsonl
./chrono certify artifacts/<substrate_namespace>/osc_50_1.0/rolling_50
```

*Note: Certification asserts that equivalence holds, or safely aborts if it does not.*

## Divergence Isolation Workflow
When equivalence fails (e.g. `ATTESTATION_FAILED`), you must locate the exact mechanical index of the drift.

```bash
./chrono inspect <baseline_artifact_dir> <suspect_artifact_dir>
```

This performs a strict chronological trace traversal and isolates the precise tick and geometric field where divergence first emerged.

To see this interactive workflow (including controlled injection of a subtle float-coercion defect):

```bash
./chrono demo-divergence
```

## Artifact Lifecycle
ChronoSentiment outputs mechanical traces and certification manifests to the `artifacts/` directory.

To keep the operational environment bounded and sanitary, explicitly remove all generated replay footprints with:

```bash
./chrono clean
```

*(Note: `clean` will never modify or target the canonical event substrates located in `core/chronology/historical`.)*

## CLI Commands
Run `./chrono help` to see the full operational interface:

```text
./chrono install
    Build the deterministic engine (trace_replay) in release mode.

./chrono demo
    Run deterministic replay certification walkthrough.

./chrono demo-divergence
    Inject controlled divergence and isolate first mismatch.

./chrono survive <tier>
    Execute long-duration deterministic survivability test.
    Tiers: 1 (100k), 2 (1M), 3 (5M), 4 (10M)

./chrono replay <topology> <cognition> --substrate-file <file>
    Run the deterministic trace replay engine manually.

./chrono certify <artifact_dir>
    Certify a generated artifact directory manifest.

./chrono inspect <baseline> <suspect>
    Strict mechanical trace divergence isolation.

./chrono clean
    Mechanically remove generated artifacts and soak substrates.
```

## Operational Notes
- **Substrate**: The immutable chronological event log.
- **Replay Artifact**: The deterministic execution footprint resulting from ingestion (`trace_v1.json`, `metadata.json`).
- **Certification**: The passive structural verification of an artifact against a canonical signature.
- **Divergence**: The explicit breaking of replay equivalence.
