# RP-605-BEH-001: Pre-Entry Behavioural Discrimination Empirical Report

**Document Identifier**: `RP-605-BEH-001-EVAL`  
**Protocol Series**: `RP-605 (NSE Prospective Freshness Validation)`  
**Date**: 2026-10-22  
**Status**: EMPIRICAL EVALUATION & STOPPING RULE AUDIT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Executive Summary: Rigorous Falsification Under Pre-Entry Information Integrity

In accordance with the stakeholder design principle and governance protocol, **RP-605-BEH-001** was executed as a controlled test of **predictive decision-time information**. Realized outcomes (MFE, MAE, realized losses) were strictly excluded from model inputs. All features were extracted point-in-time from 1-minute OHLCV candles strictly prior to `entry_timestamp` ($t \le t_{\text{entry}} - 1\text{s}$) across all **2,029 historical trades** (September 28 – October 01, 2026) across 99 NSE scrips.

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    CORE EMPIRICAL FINDINGS                                           |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
| 1. Experiment A: Frozen Baseline     | Out-of-sample Net Expectancy: −6.87 bps (−₹68.01/trade).       |
|                                     | Net Win Rate: 24.2%. Net Realized P&L: −₹69,502.27.            |
+─────────────────────────────────────+────────────────────────────────────────────────────────────────+
| 2. Experiment B: Behavioural Gating  | Filtering low-persistence chop and high friction drag excluded |
|                                     | 74.2% of opportunities (758 trades). Retained net expectancy   |
|                                     | improved marginally to −5.87 bps, but remained negative.       |
|                                     | Cost of exclusion: 176 winning trades were discarded           |
|                                     | (₹50,567.51 lost profit). Retained Net P&L: −₹15,364.16.       |
+─────────────────────────────────────+────────────────────────────────────────────────────────────────+
| 3. Experiment C: Soft Belief State   | Continuous directional belief score exhibited near-zero        |
|    Non-Monotonicity                 | correlation with out-of-sample Net P&L (r = +0.0015).          |
|                                     | • Highest belief quintile (Q5) suffered momentum exhaustion:   |
|                                     |   Net Expectancy collapsed to −10.31 bps (Net WR: 22.0%).      |
|                                     | • Zero quintiles achieved positive net expectancy after costs. |
+─────────────────────────────────────+────────────────────────────────────────────────────────────────+
| 4. Stopping Rule Invocation         | Under the locked protocol, pre-entry behavioural gating fails  |
|                                     | to generate positive net expectancy out-of-sample. The stopping|
|                                     | rule is enforced: no feature tuning or live authority granted. |
+─────────────────────────────────────+────────────────────────────────────────────────────────────────+
```

---

## 2. Experimental Protocol & Three-Way Comparison

To evaluate whether behavioural representations provide genuine forward discrimination, three controlled arms were evaluated across In-Sample (Sep 28–29, $N=1,007$) and Out-of-Sample (Sep 30–Oct 01, $N=1,022$):

### Table 1: Comparative Performance Matrix
| Experiment / Cohort | Trades ($N$) | Coverage (%) | Net Win Rate (%) | Gross P&L (₹) | Statutory Friction (₹) | Net Realized P&L (₹) | Net Exp (bps) | Max Drawdown (₹) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **A. In-Sample Baseline** | 1,007 | 100.0% | 16.5% | −₹11,597.69 | ₹89,935.56 | −₹101,533.25 | −10.16 bps | −₹112,129.52 |
| **A. Out-of-Sample Baseline** | 1,022 | 100.0% | 24.2% | +₹21,591.20 | ₹91,093.47 | −₹69,502.27 | −6.87 bps | −₹78,025.94 |
| **A. Pooled Baseline (Full)** | 2,029 | 100.0% | 20.4% | +₹9,993.52 | ₹181,029.04 | −₹171,035.52 | −8.50 bps | −₹179,575.75 |
| | | | | | | | | |
| **B. OOS Retained (Gated)** | 264 | 25.8% | 26.9% | +₹8,170.16 | ₹23,534.32 | **−₹15,364.16** | **−5.87 bps** | −₹22,502.24 |
| **B. OOS Discarded (Excluded)** | 758 | 74.2% | 23.2% | +₹13,421.04 | ₹67,559.15 | −₹54,138.11 | −7.21 bps | −₹58,089.17 |
| | | | | | | | | |
| **C. OOS Q4 (Moderate Trend)**| 204 | 20.0% | **30.9%** | +₹15,517.23 | ₹18,193.54 | **−₹2,676.32** | **−1.32 bps** | −₹7,412.18 |
| **C. OOS Q5 (Extreme Trend)** | 205 | 20.1% | 22.0% | −₹2,650.65 | ₹18,276.09 | −₹20,926.74 | −10.31 bps | −₹21,842.15 |

---

## 3. Analysis of Experiment B: The Cost of Exclusion

Filtering out friction-sensitive chop ($\text{dir\_pers} < 0.20$, $\text{f\_drag} > 1.10$, $\text{trend\_align} < 0$) removed 74.2% of the trade opportunities. While this avoided substantial losses, it came at severe economic cost:

1. **Discarded Opportunities**:
   - Out of 758 excluded trades, **176 were net winning trades** that would have generated **+₹50,567.51 in net profits**.
   - While 582 losing trades were avoided (sparing −₹104,705.62 in net losses), the retained portfolio was starved of trade volume ($N = 264$ across 2 full sessions).
2. **Net Expectancy Failure**:
   - The retained portfolio improved net expectancy by only **+1.00 bps** (from $-6.87\text{ bps}$ to $-5.87\text{ bps}$).
   - Total Net P&L remained negative ($-₹15,364.16$). Gating reduces total rupee losses solely by reducing turnover, **not by discovering positive expectancy**.

---

## 4. Analysis of Experiment C: Soft Belief Quintile Calibration

Evaluating continuous directional belief $w_{\text{directional}} \in [0, 1]$ across Out-of-Sample quintiles reveals a severe non-monotonic breakdown:

### Table 2: Out-of-Sample Belief Quintiles ($N = 1,022$)
| Quintile | Mean Belief ($w$) | Trades | Gross P&L (₹) | Statutory Charges (₹) | Net P&L (₹) | Net Win Rate (%) | Net Expectancy (bps) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Q1 (Lowest Belief / Chop)** | 0.118 | 205 | −₹424.85 | ₹18,299.56 | −₹18,724.41 | 19.5% | −9.21 bps |
| **Q2 (Weak Directional)** | 0.296 | 204 | +₹5,550.96 | ₹18,130.82 | −₹12,579.86 | 26.0% | −6.24 bps |
| **Q3 (Median)** | 0.510 | 204 | +₹3,598.52 | ₹18,193.46 | −₹14,594.94 | 22.5% | −7.22 bps |
| **Q4 (Strong Directional)** | 0.738 | 204 | +₹15,517.23 | ₹18,193.54 | −₹2,676.32 | **30.9%** | **−1.32 bps** |
| **Q5 (Extreme Climax)** | 0.945 | 205 | −₹2,650.65 | ₹18,276.09 | −₹20,926.74 | 22.0% | −10.31 bps |

### The Momentum Exhaustion Trap (Q5 Breakdown):
- In Q4, directional consistency produces strong gross profits (+₹15,517), coming close to covering statutory friction (−₹2,676 net, $-1.32\text{ bps}$).
- However, as belief reaches its maximum in Q5 ($w > 0.90$), performance collapses: Gross P&L turns negative (−₹2,650) and Net Expectancy drops to **−10.31 bps**.
- **Root Cause**: Stocks exhibiting near-perfect 15m/30m directional persistence are in the final climax of intraday momentum thrusts. Buying at the peak of persistence enters right before aggressive intraday mean-reversion.
- **Statistical Correlation**:
  $$\text{Corr}(w_{\text{directional}}, \text{Net P&L}) = \mathbf{+0.0015}$$
  $$\text{Corr}(w_{\text{directional}}, \text{Net Win Rate}) = \mathbf{+0.0312}$$
  The pre-entry behavioural metric has **effectively zero linear correlation** with forward net profitability.

---

## 5. Formal Application of the Stopping Rule

The success criteria required:
> *"Demonstrate that a frozen, pre-entry behavioural representation consistently separates economically favourable from unfavourable trading conditions on unseen data, after transaction costs, without relying on retrospective labels."*

### Stopping Rule Assessment:
1. **Positive Net Expectancy on Unseen Data**: **FAILED** (0 out of 5 quintiles achieved positive net expectancy; best cohort was $-1.32\text{ bps}$).
2. **Improvement Over Baseline**: **MARGINAL / INSUFFICIENT** (Retained expectancy improved by 1.0 bps, while starving coverage by 74.2%).
3. **Monotonic Discrimination**: **FAILED** (Q5 collapses due to climax exhaustion; overall $r = +0.0015$).

### Strategic Conclusion:
In accordance with governance rules, **the research program will not repeatedly cycle through alternative ad-hoc feature definitions to force a positive result**. 
- The current pre-entry feature set (intraday price persistence, return autocorrelation, volatility ratios) does **not** contain sufficient predictive information to overcome the 9.0 bps statutory cost hurdle in NSE cash equities.
- **Governance Mandate**:
  - Do NOT modify entry rules or exits in `adapters/chronosentiment`.
  - Maintain **0.0% Live Order Authority**.
  - Formally record the negative result as an authoritative research finding.
