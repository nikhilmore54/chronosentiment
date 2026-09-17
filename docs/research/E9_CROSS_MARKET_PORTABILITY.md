# RESEARCH EXPERIMENT: E9 Cross-Market Portability

## 1. Primary Research Question
> **Is the early-favorable-excursion phenomenon specific to the current NSE signal/data regime, or does the E4 mechanism appear in other liquid markets?**

## 2. Experimental Boundaries
This study tests the **E4 mechanism itself**, completely isolated from the ChronoSentiment (LIVE-005) signal generator. 
- The study will use a generic, market-agnostic E4 diagnostic harness.
- Prices from other markets **must not** be fed into the existing LIVE-005 signal to validate ChronoSentiment.
- The `E4-0.50` threshold remains strictly frozen at `0.50%`. There is no separate optimization or threshold search per market. The question is explicitly: *Does the already-frozen E4-0.50 mechanism transfer to another market?*

## 3. Data Sourcing and Regimes

### Layer A: Crypto Substrate
Crypto provides 24/7 observations with naturally continuous 1-minute data and no synthetic session boundaries (complying strictly with `CRYPTO_SUBSTRATE_CONTRACT_v1`).
- **Source:** Binance Spot (API or historical data service). One canonical venue.
- **Universe:** BTCUSDT, ETHUSDT, SOLUSDT
- **Resolution:** 1-minute OHLCV (Open, High, Low, Close, Volume)
- **Chronology:** Continuous UTC chronology.
- **Legacy Restriction:** Do NOT resurrect the old BTC/ETH/SOL streamer. It was a legacy live launcher and is incompatible with this isolated diagnostic harness. No futures, funding, order-book, trades, or cross-venue data.

### Layer B: Liquid US Equities
Following Crypto, testing proceeds to highly liquid US Equities/ETFs to observe behavior within traditional market hour boundaries but across a different liquidity structure.
- **Source:** Yahoo Finance
- **Universe:** SPY, QQQ, NVDA, AAPL, MSFT
- **Resolution:** 1-minute OHLCV (regular market sessions only; no pre/post-market unless explicitly frozen).
- *Note:* If Yahoo cannot provide sufficiently deep/complete 1-minute history for the chosen windows, the experiment will record `DATA_UNAVAILABLE` rather than silently switching sources.

### Regime Selection (Objective Procedure)
We will not manually pick “famous” crash or breakout dates. The selection must be objective to prevent hindsight bias.
1. Define a broad candidate historical period.
2. Calculate rolling 24h realized volatility from 1m closes.
3. Identify non-overlapping 72-hour windows.
4. Select three windows:
   - High-volatility
   - Low-volatility/Range-bound
   - Reversal (High-volatility recovery)
5. Freeze the selected windows before running E4.

## 4. Harness Configuration

### Entry Grid (T0 Generation)
To test the geometry without sneaking a predictive signal into the experiment, the entry population is generated deterministically:
- **Primary Grid:** T0 every 60 minutes.
- **Sensitivity Check Grid:** T0 every 15 minutes (to analyze overlap).
- **Direction:** Both `LONG` and `SHORT` paths are evaluated at every T0.

### Rule Execution (Immutable)
- **H300 Baseline:** 300 subsequent completed 1-minute observations, strictly governed by market-specific availability/session semantics.
- **E4 Exit:** 0.50%, first completed 1-minute close, no high/low trigger. No optimization.

## 5. Metrics & Telemetry
For every evaluated market and regime, the following outputs are generated:

| Metric | Rationale |
| :--- | :--- |
| **Trigger rate** | How often 0.50% is reached |
| **Trigger timing** | How quickly the threshold is crossed |
| **Unconditional Δ** | E4 − H300 (over the entire population) |
| **Conditional Δ** | `E4_return − H300_return | E4 threshold reached` |
| **Post-trigger giveback** | Economic rationale (do triggered trades surrender remaining gains?) |
| **MFE** | Available upside within the horizon |
| **MAE** | Adverse excursion within the horizon |
| **Regime dependence** | Robustness across structural breaks |

## 6. E9 Final Interpretation & Conclusion

The cross-market evidence extracted from the diagnostic harness yields a coherent pattern:
* **Crypto:** the selected low-volatility windows produced positive paired Δ, while the selected high-volatility windows produced negative paired Δ.
* **US equities:** SPY, QQQ, AAPL and MSFT produced positive paired Δ in the tested recent window; NVDA produced negative paired Δ.
* **Both substrates:** the direction of the effect changes with the character of the price path.
* **No LIVE-005 signal entered the experiment.**
* **The 0.50% threshold was not optimized separately for each market.**
* **H300 and E4 are paired on the same T0 observations.**

> **E4 does not appear to have a market-specific identity. It appears to interact with the shape of the underlying price path.**

### Statistical Limitation
This experiment establishes the *mechanics* of the overlay across isolated examples, not a statistical universal proof. The current samples remain relatively narrow: three 72-hour crypto windows and five recent equity sessions. Therefore, the appropriate conclusion is that **the observed cross-market behavior is consistent with the proposed regime-dependent geometry of the E4 overlay.** 

It does **not** justify modifying production parameters, adopting a regime-adaptive threshold, or installing a momentum filter. E4-0.50 remains entirely frozen for prospective NSE validation.
