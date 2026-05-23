# Claim 004: Ontology Mismatch Confounder

**Status:** Confirmed (Negative Evidence First-Class Artifact)
**Date:** 2026-05-23
**Claim Type:** Falsification / Confounder Isolation

## Core Assertion
Apparent massive divergence between Execution Chronologies and Composite Chronologies is heavily confounded by Quote Instrument Ontology. When quote ontologies are mismatched (e.g., measuring Binance `BTCUSDT` against Yahoo `BTC-USD`), the divergence observed is primarily a measure of *liquidity density* rather than *authority reconstruction error*.

## Evidence Base
1. **Mismatched Replay (`2026_recent_crossfeed_1h`):**
   - **Binance `BTCUSDT` Tick (T0 Persistence):** 43,056
   - **Yahoo `BTC-USD` 1m (T1 Persistence):** 41
   - **Initial Interpretation:** The authority heavily smoothed the physical continuity, causing a 1000x gap in representation.

2. **Ontology-Aligned Replay (`2026_recent_crossfeed_1h_ontology`):**
   - **Binance `BTCUSD` Tick (T0 Persistence):** 34
   - **Yahoo `BTC-USD` 1m (T1 Persistence):** 41
   - **Corrected Interpretation:** When aligning the quote instrument to `USD`, the execution feed's physical continuity dropped by three orders of magnitude. The Yahoo composite feed accurately reflected this sparsity. The gap collapsed.

## Scientific Conclusion
Composite chronologies (like Yahoo Finance) are not inherently "hallucinating" or overly smoothing data; they are accurately preserving the *Sparse Fiat Continuity* of the underlying fiat-quote market. The massive density observed in `BTCUSDT` is a property of *Hypersaturated Continuity* unique to the stablecoin-quote ecology, not a universal baseline for execution truth.

## Required Methodology Going Forward
All future experimental replays must explicitly classify their intent according to `Rule 7: Replay Classification`:
- **Chronology Degradation Replays** must strictly maintain the same quote ontology.
- **Authority Divergence Replays** must align quote ontologies (e.g., `USD` to `USD`).
Failure to control for ontology mismatch will result in corrupted survivability heuristics.
