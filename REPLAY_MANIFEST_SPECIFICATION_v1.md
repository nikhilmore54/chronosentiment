# Replay Manifest Specification v1

**Version:** 1.0.0
**Status:** Active
**Date:** 2026-05-25

## 1. Purpose & Scope
This specification defines the deterministic chain-of-custody layer for the ChronoSentiment Replay Observatory. As the archive has reached local morphology saturation, raw descriptor sheets are insufficient for formal auditability. 

Every executed replay must emit a strictly formatted, immutable JSON manifest. This guarantees that all observed persistence geometries (preservation, fracture, divergence) can be deterministically cryptographically verified, traced to their exact extraction boundaries, and re-executed without semantic drift.

## 2. Abstraction Freeze Guarantee
This manifest intentionally **excludes** any fields relating to:
- Predictive confidence
- Mechanistic causality
- Higher-order taxonomies or semantic clusters
- Causal interpretations (e.g., "liquidity trap", "capitulation")

It strictly records the "what, where, and how" of the replay extraction and its raw geometric outputs.

## 3. Schema Definition

A valid Replay Manifest MUST contain the following categories and fields:

### 3.1 Replay Identity
The cryptographic and nominal identity of the replay.
*   `replay_id`: (String) Canonical name (e.g., `2026_nvda_sync_oscillatory_fragmentation_5m`).
*   `chronology_hash`: (String) SHA-256 hash of the input `.jsonl` substrate file. Guarantees the raw input data hasn't mutated.

### 3.2 Replay Class
The macroscopic categorization of the event.
*   `replay_class`: (String) Bounded descriptor (e.g., `Oscillatory Fragmentation`, `Contradiction Pairing`).
*   `authority_type`: (String) Source of the data (e.g., `Composite (Yahoo Presentation)`).

### 3.3 Session Ontology
The structural boundaries of the market session.
*   `session_ontology`: (String) e.g., `Bounded (Equity)`.

### 3.4 Extraction Metadata
The exact temporal bounding of the replay window.
*   `start_ts`: (Integer) Unix epoch timestamp (milliseconds) of the first tick.
*   `end_ts`: (Integer) Unix epoch timestamp (milliseconds) of the last tick.
*   `shift_offset`: (Integer) Temporal perturbation offset in minutes (e.g., `0`, `-10`, `+20`). Essential for tracking robustness tests.

### 3.5 Instrument Metadata
The financial instruments involved.
*   `symbols`: (Array of Strings) e.g., `["NVDA"]` or `["NVDA", "AMD"]`.
*   `timeframe`: (String) The chronological density (e.g., `5m Presentation`).

### 3.6 Replay Descriptor
The descriptive tags strictly tracking phenomenological attributes.
*   `descriptors`: (Array of Strings) Local labels only (e.g., `["high participation", "violent alternation", "compressed net displacement"]`).

### 3.7 Geometry Outputs
The empirical terminal values emitted by the rust core.
*   `baseline_a`: (Float) Output of the `rolling_50` cognition on the `osc_50_1.0` topology.
*   `baseline_b`: (Float) Output of the `event_reset` cognition on the `osc_50_1.0` topology.

### 3.8 Artifact Fingerprints
Cryptographic proofs of the emitted observability traces.
*   `trace_hashes`: (Object) Key-value pairs of artifact paths to their SHA-256 hashes (e.g., `{"tier1_5m/event_reset/trace_summary.json": "abc123..."}`).

### 3.9 Determinism Metadata
Verification of the exact engine configuration used during execution.
*   `topology_hash`: (String) Hash of the active topology definition (or simply the topology name if strictly versioned, e.g., `osc_50_1.0`).
*   `cognition_hash`: (String) Hash of the active cognition logic.

### 3.10 Provenance
Environmental auditability.
*   `commit_hash`: (String) Git commit hash of the ChronoSentiment repository at execution time.
*   `manifest_version`: (String) Always `1.0.0` for this specification.

## 4. Implementation Requirements
- The manifest MUST be serialized as a pretty-printed `manifest.json` file.
- The manifest MUST be stored in the root of the replay's artifact directory: `core/artifacts/[phase]/[replay_id]/manifest.json`.
- Any modification to the substrate data or core engine requires a complete re-execution and the generation of a new manifest with updated hashes.

## 5. Next Infrastructure Phase
Following the implementation of a script to emit these manifests, the observatory will build a **Replay Equivalence Verifier** to rapidly validate all existing replays against their manifests to detect accidental drift.
