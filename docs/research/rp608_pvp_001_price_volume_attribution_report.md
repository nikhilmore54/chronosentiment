# RP-608-PVP-001: Price–Volume Pattern Attribution Report
**Phase 1: Pre-Entry Observational Forensics & Outcome Discrimination**

**Document Identifier**: `RP-608-PVP-001-REPORT`  
**Protocol Series**: `RP-608 (Execution Reversals & Directional Attribution)`  
**Date**: 2026-10-22  
**Status**: COMPLETED OBSERVATIONAL FORENSICS REPORT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)  
**Sample Population**: 2,029 historical trades across 99 NSE symbols (Sep 28 – Oct 01, 2026)

---

## 1. Executive Summary & Core Research Findings

In accordance with stakeholder guidance, `RP-608-PVP-001` evaluated the preceding price structure, volume behaviour, and market context strictly prior to entry across:
1. **Top 10% Largest Winners** ($N=202$, Mean Gross: **+64.84 bps**)
2. **Top 10% Largest Losers** ($N=202$, Mean Gross: **−34.81 bps**)
3. **Matched Comparison Trades** ($N=202$, Mean Gross: **−2.50 bps**, matched on symbol, session, and time-of-day)
4. **All 2,029 historical trades** stratified into deciles ($D_1$ to $D_{10}$).

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    THREE-WAY HYPOTHESIS DISPOSITION                                  |
+──────────────────────────────────────────────────────────────+───────────────────────────────────────+
| Hypothesized Case                                            | Empirical Finding & Status            |
+──────────────────────────────────────────────────────────────+───────────────────────────────────────+
| Case 1: Losing trades share a pre-entry pattern that         | REJECTED. Inverting overextended      |
| predicts subsequent adverse returns (Actionable signal).     | trades yields -4.30 bps net after fees|
|                                                              |                                       |
| Case 2: Losing trades share a pattern, but it does not       | CONFIRMED DESCRIPTIVELY.              |
| predict future returns (Descriptive explanation only).       | Identifies momentum exhaustion drag.  |
|                                                              |                                       |
| Case 3: The inverse improves gross returns but loses         | CONFIRMED ECONOMICALLY.               |
| after costs (No demonstrated economic edge).                 | Gross +4.70 bps fails 9.00 bps hurdle.|
+──────────────────────────────────────────────────────────────+───────────────────────────────────────+
```

### Key Analytical Takeaways:
1. **The Anatomy of a Losing Trade (Momentum Climax / Exhaustion)**:
   - The Top 10% largest losing trades entered after an aggressive **15-minute momentum run (+35.3 bps in the trade direction)**, compared to **+21.9 bps** for big winners and **+19.3 bps** for matched trades ($d = 0.47, p = 1.9 \times 10^{-5}$).
   - Losers entered when price was stretched **+57.4 bps away from session VWAP**, compared to **+35.5 bps** for winners ($d = 0.42, p = 7.0 \times 10^{-6}$).
   - Losers entered during an abnormal **5-minute volume surge (1.56× 30m median volume)** vs **1.12×** for winners ($d = 0.34, p = 0.0003$).
   - **Diagnosis**: The baseline entry signal systematically triggers at the climax of an overextended micro-trend, buying directly into exhaustion wicks immediately preceding mean-reversion.
2. **Is this an Actionable Inverse Signal?**:
   - Isolating the 242 trades (11.9% of sample) meeting this severe overextension profile (`ret_15m_aligned` $\ge 30\text{ bps}$ AND `dist_vwap_bps` $\ge 45\text{ bps}$):
     - In the baseline, this cohort lost **−₹11,405.25 gross** (**−4.70 bps**) and **−₹32,959.93 net** (**−13.70 bps**).
     - Taking the opposite position (shorting the overextension) generates **+₹11,405.25 gross** (**+4.70 bps gross**, 66.1% gross win rate).
     - However, statutory friction is **9.00 bps** (₹21,554.68).
     - The resulting inverse net performance is **−₹10,149.43** (**−4.30 bps net expectancy**, 43.4% net win rate).
   - **Economic Verdict**: The pattern provides a valuable **descriptive post-mortem** explaining why the baseline fails, but its gross reversal magnitude (+4.70 bps) is **less than half the statutory friction hurdle (9.00 bps)**.

---

## 2. Statistical Discrimination Matrix: Winners vs Losers vs Matched

Features were extracted strictly from completed 1-minute bars preceding each trade's entry timestamp:

### Table 1: Pre-Entry Feature Distributions & Effect Sizes
| Feature Category | Pre-Entry Metric | Mean Losers ($N=202$) | Mean Winners ($N=202$) | Mean Matched ($N=202$) | Cohen's $d$ ($L$ vs $W$) | Mann-Whitney $p$ ($L$ vs $W$) | KS $p$ ($L$ vs Matched) | Correlation with Gross PnL | Statistically Discriminative? |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Price Structure** | `ret_5m_aligned` (bps) | +8.90 | +5.41 | +3.35 | +0.24 | $p = 0.040$ | $p = 2.0 \times 10^{-4}$ | $r = +0.004$ ($p = 0.85$) | Marginal |
| **Price Structure** | `ret_15m_aligned` (bps)| **+35.32** | **+21.93** | **+19.31** | **+0.47** | **$p = 1.9 \times 10^{-5}$** | **$p = 1.7 \times 10^{-7}$** | **$r = -0.069$ ($p = 0.002$)** | **YES ($p < 0.001$)** |
| **Price Structure** | `ret_30m_aligned` (bps)| +40.12 | +32.37 | +30.66 | +0.23 | $p = 0.064$ | $p = 7.0 \times 10^{-4}$ | $r = -0.026$ ($p = 0.24$) | Marginal |
| **Price Structure** | `consec_run` (candles) | 1.06 | 0.96 | 0.80 | +0.08 | $p = 0.846$ | $p = 0.473$ | $r = +0.021$ ($p = 0.34$) | NO |
| **Price Structure** | `body_to_range` ratio | 0.53 | 0.55 | 0.48 | −0.06 | $p = 0.483$ | $p = 0.334$ | $r = +0.036$ ($p = 0.11$) | NO |
| **Price Structure** | `adverse_wick_ratio` | 0.24 | 0.22 | 0.27 | +0.06 | $p = 0.525$ | $p = 0.635$ | $r = -0.037$ ($p = 0.10$) | NO |
| **Price Structure** | `dist_vwap_bps` | **+57.42** | **+35.46** | **+40.84** | **+0.42** | **$p = 7.0 \times 10^{-6}$** | **$p = 1.5 \times 10^{-3}$** | **$r = -0.070$ ($p = 0.002$)** | **YES ($p < 0.001$)** |
| | | | | | | | | | |
| **Volume Behaviour**| `rel_vol_5m` (vs 30m) | **1.56×** | **1.12×** | **1.15×** | **+0.34** | **$p = 3.2 \times 10^{-4}$** | **$p = 6.5 \times 10^{-3}$** | **$r = -0.045$ ($p = 0.042$)** | **YES ($p < 0.01$)** |
| **Volume Behaviour**| `vol_spike_last_1m` | 2.30× | 1.73× | 2.18× | +0.19 | $p = 0.084$ | $p = 0.031$ | $r = -0.025$ ($p = 0.26$) | NO |
| **Volume Behaviour**| `vol_divergence` flag | 36.1% | 33.2% | 26.2% | +0.06 | $p = 0.531$ | $p = 0.276$ | $r = -0.005$ ($p = 0.81$) | NO |
| **Volume Behaviour**| `vol_exhaustion` flag | 4.0% | 3.5% | 4.0% | +0.03 | $p = 0.794$ | $p = 1.000$ | $r = +0.000$ ($p = 0.99$) | NO |
| | | | | | | | | | |
| **Market Context** | `volatility_30m_bps` | 8.44 | 7.17 | 7.62 | +0.28 | $p = 0.019$ | $p = 0.041$ | $r = -0.024$ ($p = 0.29$) | Marginal |
| **Market Context** | `dist_from_open_bps` | 77.01 | 40.96 | 49.67 | +0.25 | $p = 0.031$ | $p = 0.023$ | $r = -0.051$ ($p = 0.021$) | Marginal |
| **Market Context** | `pos_in_day_range` | 0.43 | 0.35 | 0.43 | +0.23 | $p = 0.130$ | $p = 0.400$ | $r = -0.066$ ($p = 0.003$) | NO |

---

## 3. Univariate Predictive Power (AUC-ROC for Separating Big Losers)

Using pre-entry features to classify whether an entry will belong to the Top 10% Losers versus Top 10% Winners:

```text
Feature                      AUC-ROC   Predictive Strength
──────────────────────────────────────────────────────────
dist_vwap_bps                0.629     Moderate Separation
ret_15m_aligned              0.623     Moderate Separation
rel_vol_5m                   0.604     Moderate Separation
volatility_30m_bps           0.568     Weak Separation
dist_from_open_bps           0.562     Weak Separation
ret_5m_aligned               0.559     Weak Separation
ret_30m_aligned              0.553     Weak Separation
vol_spike_last_1m            0.550     Weak Signal
pos_in_day_range             0.544     Weak Signal
dist_30m_extremum            0.526     Near Random Drift
body_to_range                0.520     Near Random Drift
adverse_wick_ratio           0.518     Near Random Drift
vol_divergence               0.515     Near Random Drift
consec_run                   0.505     Near Random Drift
vol_exhaustion               0.502     Pure Noise
```

**Interpretation**:
- Only three features achieve $\text{AUC} > 0.60$: `dist_vwap_bps` (0.629), `ret_15m_aligned` (0.623), and `rel_vol_5m` (0.604).
- While statistically distinguishable from random noise ($p < 0.001$), an $\text{AUC}$ of $\approx 0.62$ demonstrates that these patterns are **descriptive symptoms of overextension**, not an executable edge that can overcome 9.00 bps friction.

---

## 4. Decile Stratification Across All 2,029 Trades

### Table 2: Pre-Entry Metrics by Outcome Decile
Dividing the entire 2,029-trade ledger into 10 outcome deciles ($D_1$ = worst losses, $D_{10}$ = best gains):

| Decile | Trades | Mean Gross Return | Pre-Entry 15m Momentum | 5m Relative Volume | Adverse Wick Ratio | Distance from 30m Extreme | Position in Day Range |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **D1 (Worst Losers)** | 203 | **−34.75 bps** | **+35.62 bps** | **1.56×** | 0.24 | 1.54 bps | 0.43 |
| **D2** | 203 | −17.96 bps | +19.71 bps | 1.24× | 0.25 | 4.19 bps | 0.45 |
| **D3** | 203 | −12.20 bps | +13.18 bps | 1.17× | 0.28 | 2.95 bps | 0.41 |
| **D4** | 203 | −8.38 bps | +17.27 bps | 1.18× | 0.27 | 3.05 bps | 0.43 |
| **D5** | 203 | −5.58 bps | +15.57 bps | 0.95× | 0.25 | 4.78 bps | 0.44 |
| **D6** | 202 | −3.05 bps | +17.44 bps | 1.24× | 0.26 | 3.04 bps | 0.35 |
| **D7** | 203 | −0.60 bps | +14.99 bps | 1.08× | 0.27 | 3.50 bps | 0.44 |
| **D8** | 203 | +4.70 bps | +19.56 bps | 1.12× | 0.25 | 4.63 bps | 0.40 |
| **D9** | 203 | +18.36 bps | +20.40 bps | 1.22× | 0.23 | 1.06 bps | 0.39 |
| **D10 (Best Winners)** | 203 | **+64.67 bps** | **+21.88 bps** | **1.12×** | 0.22 | 0.63 bps | 0.35 |

---

## 5. Synthesis: Why the Strategy Loses and Why Inversion Fails

This empirical investigation provides answers to the user's framing:

1. **Why the Baseline Loses**:
   - The strategy suffers from **momentum exhaustion**: entries in $D_1$ occur when 15-minute momentum (+35.6 bps) and volume surges (1.56×) have peaked.
   - The trade is triggered at the apex of a short-term wick, forcing an immediate adverse excursion that trips stops or closes at a loss.
2. **Why Inversion Does Not Create a Viable Strategy**:
   - Inverting the trade direction flips $D_1$ from a −34.8 bps loss into a +34.8 bps gain.
   - However, inverting the entire strategy flips the massive winners ($D_{10}$, +64.7 bps) into catastrophic −64.7 bps losses.
   - Even if restricted to the overextended cohort ($N=242$), the resulting gross gain is **+4.70 bps**, which is insufficient to overcome the **9.00 bps statutory fee hurdle**, leaving net expectancy negative at **−4.30 bps**.
3. **Core Conclusion**:
   - The failure of ChronoSentiment is **structural to the signal-generation mechanism**: it does not capture persistent trend momentum. Instead, it enters late into noisy intraday runs that lack follow-through.
   - Attempting to trade the inverse direction is defeated by the same friction physics that destroy the baseline.

---

## 6. Governance Closure

1. **`RP-608-PVP-001` Formally Concluded**: Pre-entry price-volume patterns confirm momentum exhaustion as the mechanism of catastrophic losses, but falsify trade inversion as a viable edge after costs.
2. **Strategy Code Frozen**: No inverse trading logic or post-hoc entry filters will be incorporated into `adapters/chronosentiment`.
3. **Operational State**: `adapters/chronosentiment` remains **100% frozen**; live order authority remains strictly **0.0%**.
