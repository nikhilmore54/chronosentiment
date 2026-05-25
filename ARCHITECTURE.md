# Bounded Architecture

ChronoSentiment Core is designed to be horizontally constrained and mechanically explicit. 

## Component Layers

1. **The Replay Engine (Rust)**
   - Located in `core/`.
   - Strictly single-threaded, deterministic execution kernel.
   - Responsible for ingesting JSONL substrates and emitting byte-for-byte reproducible trace artifacts.
   - Built for bounded survivability (O(1) memory for streaming ingestion).

2. **The Verification Layer (Python)**
   - Located in `scripts/verify_manifest_v1.py` and `scripts/inspect_divergence.py`.
   - Implements passive structural attestation.
   - Operates entirely outside the engine boundary.

3. **The Substrate (JSONL)**
   - The immutable, chronological event log format.
   - Represents the canonical input state.

## Dependency Asymmetry
The Replay Engine does not know about external live streams, databases, or orchestrators. It only understands bounded file descriptors. This asymmetry is intentional and permanently frozen.
