# RP-605-BEH-001: Provenance, Timestamp Semantics & Integrity Audit

**Document Identifier**: `RP-605-BEH-001-AUDIT`  
**Protocol Series**: `RP-605 (NSE Prospective Freshness Validation)`  
**Date**: 2026-10-22  
**Status**: COMPLETED FORENSIC PROVENANCE & INTEGRITY AUDIT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants verified green)

---

## 1. Executive Summary: Verification & Archival Recommendation

In compliance with governance directives, an exhaustive forensic audit of the experimental provenance, timestamp semantics, preprocessing isolation, and arithmetic integrity of **RP-605-BEH-001** was conducted across all **2,029 historical trades** (September 28 – October 01, 2026).

### Audit Verdict:
1. **Methodological Validity Confirmed**: The negative result is **robust and untainted by lookahead leakage**. When all preprocessing parameters are fitted strictly on the In-Sample development set (Sep 28–29) and evaluated on untouched Out-of-Sample data (Sep 30–Oct 01), the pre-entry behavioural representation still fails to generate positive net expectancy ($r = +0.0072$; zero positive quintiles).
2. **Provenance Verified**: Candle boundaries strictly preceded entry; zero trades hit the missing-history fallback; metric denominators and accounting ledgers reconcile to ₹0.0000.
3. **Formal Archival Disposition**: `RP-605-BEH-001` is formally archived as a **valid, trustworthy negative result** for the tested feature set and hypotheses. The stopping rule is upheld. Governance remains 100% frozen.

---

## 2. Comprehensive Itemized Audit Findings

### Item 1: Candle Timestamp Semantics & Point-in-Time Guarantee
- **Concern**: If `candle_timestamp <= entry_timestamp - 1s` is used, mid-candle entries could absorb future data if timestamps represent candle start times.
- **Forensic Verification**:
  - In `trades_df`, **100.0% of trades (2,029 / 2,029)** have entry timestamps ending on `:00` seconds (`entry_dt.second == 0`).
  - In Yahoo 1-minute candle caches, timestamps represent the start of the 1-minute interval (e.g. `04:09:00` covers $[04:09:00, 04:10:00)$).
  - For a trade entering at `04:10:00`, `entry_t - 1s = 04:09:59`. The latest candle selected by `cdf.loc[:'04:09:59']` is `04:09:00`.
  - The `04:09:00` candle concluded at `04:10:00` (the exact instant of entry). The subsequent candle `04:10:00` was strictly excluded.
  - **Conclusion**: Zero future price or volume data leaked into feature computation for these on-the-minute entries.
  - **Architectural Requirement**: For future asynchronous intraday execution, the engine must formally enforce `candle_end_time <= entry_timestamp` rather than subtracting a 1-second delta.

### Item 2: Full-Sample Standardisation vs. Strict Preprocessing Isolation
- **Concern**: The initial exploratory run calculated $z$-score means and standard deviations across all 2,029 trades before splitting into In-Sample and Out-of-Sample, leaking the holdout feature distribution.
- **Forensic Verification**:
  - The scalar parameters $(\mu, \sigma)$ were re-fit strictly on In-Sample (Sep 28–29, $N=1,007$):
    - $\mu_{\text{dir}} = 0.2172, \sigma_{\text{dir}} = 0.2081$
    - $\mu_{\text{fdrag}} = 1.2117, \sigma_{\text{fdrag}} = 0.4760$
    - $\mu_{\text{align}} = 0.002634, \sigma_{\text{align}} = 0.003310$
  - These frozen parameters were applied forward to Out-of-Sample (Sep 30–Oct 01, $N=1,022$).
- **Audited Out-of-Sample Quintile Performance (Strict Isolation)**:
  | Quintile | Trades ($N$) | Mean Belief | Gross P&L (₹) | Statutory Friction (₹) | Net P&L (₹) | Net Win Rate (%) | Net Expectancy (bps) |
  | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
  | **Q1 (Chop)** | 205 | 0.134 | +₹235.66 | ₹18,302.91 | −₹18,067.25 | 19.5% | −8.88 bps |
  | **Q2 (Weak)** | 204 | 0.345 | +₹5,158.22 | ₹18,127.99 | −₹12,969.78 | 26.5% | −6.44 bps |
  | **Q3 (Median)** | 204 | 0.586 | +₹3,894.28 | ₹18,194.00 | −₹14,299.71 | 22.1% | −7.07 bps |
  | **Q4 (Strong)** | 204 | 0.806 | +₹14,455.80 | ₹18,196.08 | −₹3,740.29 | **30.9%** | **−1.85 bps** |
  | **Q5 (Extreme)**| 205 | 0.966 | −₹2,152.76 | ₹18,272.49 | −₹20,425.24 | 22.0% | **−10.06 bps** |
- **Conclusion**: Strict preprocessing isolation yields identical scientific conclusions. Correlation between isolated belief and OOS Net P&L is $r = +0.0072$. Zero quintiles achieve positive net expectancy.

### Item 3: Missing-History Fallback Audit
- **Concern**: Trades with $< 30$ prior bars received fallback values that could artificially pass gates.
- **Forensic Verification**:
  - Exactly **0 of 2,029 trades (0.00%)** had fewer than 30 prior bars.
  - Every traded symbol had continuous 1-minute bar history extending back to 2026-09-17 (over 10 trading sessions prior to Sep 28). The fallback code was never executed.

### Item 4: Scientific Interpretation of Q5 (Distinguishing Hypothesis from Finding)
- **Clarification**: Q5 is an empirical composite of directional persistence, low friction drag, and trend alignment.
- **Audit Nuance**: The empirical finding is that **Q5 delivered gross losses (−₹2,152.76) and poor net expectancy (−10.06 bps)**. The attribution of this failure to "momentum exhaustion" is an **unproven observational hypothesis**, not a proven causal mechanism. It is recorded strictly as a hypothesis for future research.

### Item 5: Chronological Ordering & Maximum Drawdown Definition
- **Verification**: Maximum drawdown was evaluated under both execution sequence conventions:
  1. **Realized Cash Drawdown (Ordered by `exit_timestamp`)**:
     - Pooled Full Baseline ($N=2,029$): **−₹179,731.68**
     - Out-of-Sample Baseline ($N=1,022$): **−₹78,181.87**
  2. **Vintage Sequence Drawdown (Ordered by `entry_timestamp`)**:
     - Pooled Full Baseline ($N=2,029$): **−₹175,389.14**
     - Out-of-Sample Baseline ($N=1,022$): **−₹73,576.73**
- Both metrics reflect deep, sustained capital erosion in the baseline.

### Item 6: Metric Definitions & Arithmetic Reconciliation
- **Expectancy in Basis Points**:
  $$\text{Net Exp (bps)} = \frac{\sum \text{Net P\&L}_i}{\sum (\text{Entry Price}_i \times \text{Quantity}_i)} \times 10,000$$
- **Accounting Reconciliation ($N=2,029$)**:
  - Total Gross P&L: **+₹9,993.52**
  - Total Statutory Charges: **₹181,029.04** (exact Zerodha equity intraday rates)
  - Total Net P&L: $+9,993.52 - 181,029.04 = \mathbf{-₹171,035.52}$
  - Total Notional Turnover: **₹201,223,546.73**
  - Realized Net Expectancy: **−8.500 bps** (−₹84.30 / trade)
  - Average Friction: **₹89.2208 / trade** (8.996 bps)
  - Balance Discrepancy: Exactly **₹0.000000**.

### Item 7: Feature Indexing Rigor (Interval Alignment)
- **Detail**: The original persistence snippet compared `closes[-1] - closes[-15]` (14 intervals) against `np.sum(np.abs(np.diff(closes[-16:])))` (15 intervals).
- **Rigor Check**: When numerator is matched to 15 intervals (`closes[-1] - closes[-16]`), Out-of-Sample correlation is $r = +0.0069$, and quintile distributions remain virtually identical. The 1-interval offset had zero material impact on the result.

---

## 3. Epistemic Scope & Scientific Boundaries

To preserve scientific rigor in the permanent research archive, the scope of these findings is explicitly bounded:

1. **Specific Setup Falsified, Not Universal Absence of Signal**:
   - The finding is that the **specified pre-entry persistence/drag/alignment score failed to achieve positive net expectancy on this holdout**.
   - Near-zero linear correlation ($r \approx 0.007$) does not formally prove the absence of every complex non-linear relationship, nor does it establish that all pre-entry market microstructure information is universally useless.
2. **Holdout Session Clustering Limitation**:
   - The holdout comprises **two trading sessions (Sep 30 & Oct 01)**. While it contains 1,022 trades across 99 symbols, these trades are **not 1,022 independent experiments**. Many share simultaneous market-wide index regimes, sector co-movements, and liquidity shocks.
   - The evidence is persuasive against promoting or deploying this specific configuration, but does not claim to observe all possible macroeconomic or volatility regimes.
3. **Prohibition of Retrospective Tweaking**:
   - In accordance with the stopping rule, repeatedly searching for alternative feature permutations against this identical holdout until it appears profitable is strictly prohibited. Any future research must originate from a distinct theoretical hypothesis and evaluate on genuinely unseen future data.

---

## 4. Formal Research Archival & Programme State

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    FORMAL RESEARCH STATUS RECORD                                     |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| Workstream                         | Formal Decision & Archival Status                               |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| RP-605-EXIT-001 (Trailing Exits)   | ARCHIVED — Tested trailing-exit hypothesis failed to establish  |
|                                    | an executable edge in forward-time path-dependent replay.       |
|                                    | Baseline adapter exits preserved.                               |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| RP-605-BEH-001 (Pre-Entry Gating)  | ARCHIVED — Tested pre-entry gate and continuous belief score     |
|                                    | failed to achieve positive net expectancy out-of-sample.        |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| RP-605-INV-001 (Trade Inversion)   | DEFERRED — No demonstrated predictive pre-entry basis for       |
|                                    | taking opposite positions. Realized-outcome inversion prohibited|
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| ChronoSentiment Production Adapter | 100% FROZEN (13/13 discipline invariants verified green).       |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| Live-Order Authority               | STRICTLY 0.0% (Observational and shadow simulation only).       |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
| Holdout Feature Retuning           | STRICTLY PROHIBITED under the locked stopping rule.             |
+────────────────────────────────────+─────────────────────────────────────────────────────────────────+
```
