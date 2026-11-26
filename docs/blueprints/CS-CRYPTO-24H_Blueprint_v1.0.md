# CS-CRYPTO-24H Blueprint v1.0

## 1. Core Philosophy
The Crypto 24H product track is a fundamentally separate entity from the ChronoSentiment NSE (`LIVE-005`) engine. It is designed to navigate continuous 24/7 market topologies without artificial session boundaries, overnight resets, or clock-driven limitations. 

The architecture achieves adaptivity through continuous state estimation rather than parameter optimization, ensuring a deterministic and mechanically robust response to shifting market regimes (volatility, momentum, and mean-reversion).

## 2. Immutable Substrate Rules
As strictly governed by `CRYPTO_SUBSTRATE_CONTRACT_v1`:
- **Canonical Venue:** Binance Spot 1m OHLCV. No cross-venue aggregation.
- **Topology:** Continuous UTC chronology. Zero synthetic sessions.
- **Microstructure Ban:** No L2/L3 order book data, no funding rates, no latency arbitrage.

## 3. Core Component Pipeline

### 3.1 Observation Store (`observation_store.py`)
Ingests and manages a rolling window of continuous 1-minute OHLCV observations. Serves as the strictly immutable data ground-truth for all subsequent layers.

### 3.2 Rolling State Engine (`rolling_state_engine.py`)
Evaluates the continuous observation sequence *synchronously*. For every incoming minute $t$, it deterministically updates the continuous state vector (volatility, trend strength, excursion behaviour, etc).

### 3.3 Native Crypto IC (`crypto_ic.py`)
Because the `LIVE-005` signal formulation was built for bounded sessions, Crypto 24H uses a natively designed Information Coefficient layer. This is an explicit research/validation layer isolated from the execution mechanics.

### 3.4 Admission Gate (`admission_gate.py`)
Evaluates the output of the IC layer against stable operating thresholds. It is solely responsible for determining if the current market state warrants generating an event-driven `T0` (issuing `ACT LONG`, `ACT SHORT`, or `WAIT`).

### 3.5 Decision Brief (`decision_brief.py`)
An immutable synthesis of the generated `T0`, the evaluated direction, and the synchronous state vector at the exact moment of generation.

### 3.6 Lifecycle Manager (`lifecycle_manager.py`)
Handles the execution mechanics of an admitted decision. In V1, this consists strictly of the proven `E4-0.50` close-only protective overlay combined with a deterministic, continuous tracking horizon. Adaptive horizons are explicitly excluded in V1 to prevent curve fitting.

### 3.7 Deterministic Replay (`replay.py`)
Enforces the mandatory execution equivalence contract: processing the same historical 1-minute stream must yield byte-for-byte identical state vectors, decisions, and lifecycles as live execution.

## 4. Verification Mandates
Before any aspect of this engine can generate live recommendations or shadow execution signals, it must mechanically pass:
1. **No Future Leakage:** Mathematical guarantees that state at time $t$ has zero dependencies on $t+1$.
2. **Replay Determinism:** Perfect deterministic stability between run instances.
3. **Out-of-Sample Prospective Validation:** Live, unseen data validation of the Native Crypto IC.
