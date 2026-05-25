# Known Limitations

ChronoSentiment Core is explicitly designed for deterministic divergence isolation. To preserve these guarantees, we enforce strict architectural boundaries.

## 1. No Live Execution
**ChronoSentiment is not a live trading engine.** 
It does not connect to live broker feeds, manage websockets, or handle network packet loss. It strictly ingests pre-captured, bounded chronological event streams (substrates). 
*Why?* Live execution destroys reproducibility. ChronoSentiment is the observer, not the actor.

## 2. No Autonomous Action
**ChronoSentiment does not make decisions.**
It does not contain adaptive logic, ML models, or heuristic fallback mechanisms. It simply replays state transitions identically and verifies them against canonical hashes.
*Why?* Intelligence obscures operational mechanical failure.

## 3. Strict Deterministic Assumptions
**The host must support deterministic arithmetic.**
While the core relies heavily on integers to isolate drift, the host environment must still adhere to standard architectural norms. If you modify the rust kernel to introduce randomness, the certification manifest will immediately fail.

## 4. Bounded Retention
Artifacts (`trace_v1.json`, `metadata.json`) can become extremely large during soak testing. The engine explicitly truncates raw JSON traces at 500,000 events to prevent disk exhaustion, while preserving the full-length cryptographic metadata hash. 
*Why?* Operational survivability always supersedes infinite logging.

## 5. No Distributed Orchestration
**ChronoSentiment runs on a single host.**
It is not designed for Kubernetes deployment, distributed cluster scheduling, or multi-tenant cloud architectures.
*Why?* Horizontal platform scaling is orthogonal to divergence isolation, and introduces unnecessary operational complexity.
