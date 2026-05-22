# OBSERVABILITY SEMANTICS CONTRACT v1

## 1. Core Principle
This document bounds the semantic interpretation of the ChronoSentiment observability layer. Observability exists exclusively to measure temporal infrastructure visibility. It does **not** measure or predict market sentiment, volatility, opportunity, or direction.

## 2. Allowed Metric Definitions (Mechanical)
*   **`strict_ratio`**: The percentage of the cohort that successfully crossed the barrier timestamp with zero delay.
*   **`acceptance_ratio`**: The percentage of the cohort that fell within the admissible temporal window (e.g., lag <= 120s).
*   **`lag_stddev`**: The mechanical spread of publication delays across the cohort. High variance indicates a fragmented publication wave.
*   **`regime_state`**: A purely descriptive classification of the cohort's temporal synchronization derived exclusively through boolean intersections of the ratios above.

## 3. Forbidden Interpretations (Anti-Mythology)
*   High `lag_stddev` does **not** mean the market is "stressed" or "panicking"; it means the upstream provider's publication pipeline is staggered.
*   `TRANSITIONAL_RECOVERY` does **not** indicate "bullish momentum" or "market recovery"; it solely indicates that the mechanical synchronization ratio is rising.
*   `DEGRADED_OBSERVABILITY` is **not** a trading signal to short or exit; it is an environmental constraint halting new entries due to causal blindness.

## 4. Strict Layer Boundaries
*   **Layer 1 (Chronology)**: Immutable Rust ingestion substrate. Owns deduplication, ordering, and deterministic replay. Blind to regimes.
*   **Layer 2 (Admissibility)**: Python orchestration boundary. Derives the explicit regime state from the raw ingestion telemetry and emits formal boolean constraints (`new_entries_allowed`, `exits_allowed`).
*   **Layer 3 (Alpha / Signal Generation)**: Execution logic. **Must** consume Layer 2's constraints without attempting to override or reinterpret them. Alpha may not redefine what "observable" means.

## 5. Replay Determinism Guarantee
The raw telemetry (e.g., `symbols_attempted`, `symbols_returned`) embedded in the `live_session_steps.jsonl` ledger acts as the authoritative measurement of historical visibility. Any backtest or simulation traversing a frozen timeline **must** perfectly reproduce the exact regime transitions and admissibility blockades experienced by the live engine at that barrier tick. Admissibility is a recomputable derivative of raw historical topology, not a stochastic variable.
