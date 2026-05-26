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

4. **The Observatory UI (React)**
   - Located in `my-chrono-sentiment-ui/`.
   - A schema-bound causal replay instrument — not an analytics dashboard.
   - Projects backend-certified causal propagation topology, divergence state, and execution traces into navigable visual surfaces.
   - **Core invariant:** The frontend projects relationships — it does not invent relationships.
   - All rendered data derives from backend-certified sources via canonical schemas:
     - `schemas/canonical/replay_response.schema.json` — replay blocks, certification state, causal chain
     - `schemas/canonical/observatory_state.schema.json` — system phase, kernel metrics, snapshot sequence
   - Observability surfaces: replay certification badge, causal ancestry panel, forward propagation panel, transition-aware causal arrows, divergence accumulation summary, replay-position-aware event counts.
   - `useSystemStatus()` hook: tries `GET /observatory` (canonical `ObservatoryState`) then `GET /health`; null defaults on failure — no fabricated operational state.
   - See `docs/ui/uiux.md` for full synchronization status and component documentation.

## Dependency Asymmetry
The Replay Engine does not know about external live streams, databases, or orchestrators. It only understands bounded file descriptors. This asymmetry is intentional and permanently frozen.

The Observatory UI does not know about Rust internals, JSONL substrates, or certification algorithms. It only understands API responses conforming to canonical schemas. This asymmetry is intentional — the UI is a projection surface, not a computation layer.
