# RP-606-FRESH-001: Prospective Baseline Validation Protocol

**Document Identifier**: `RP-606-FRESH-001`  
**Protocol Series**: `RP-606 (Economic Edge & Execution Attribution)`  
**Date**: 2026-10-22  
**Status**: REGISTERED PRE-COLLECTION PROTOCOL & LOCKED RESEARCH SPECIFICATION  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Executive Mandate & Purpose

Following the empirical completion of `RP-606-ECO-001`, which established that the historical baseline gross return ($+0.52\text{ bps}$) is statistically indistinguishable from zero ($p = 0.39$) and heavily clustered in a single session, this protocol governs **Phase 2: Prospective Baseline Validation**.

### Purpose of RP-606-FRESH-001:
To establish—under prospective, forward-time shadow conditions with point-in-time execution telemetry—whether the unchanged baseline strategy possesses a repeatable, statistically significant gross trading advantage in fresh market sessions, or whether the signal-generation hypothesis should be permanently retired.

---

## 2. Pre-Declared Controls & Locked Decisions

Before prospective shadow collection begins, the following parameters and evaluation procedures are locked:

| Control Dimension | Pre-Declared Decision & Governance Standard |
| :--- | :--- |
| **Strategy Configuration** | **Exact Frozen Baseline**: Coralys v0.1.0 + Config A ($K=3, G_{\max}=0.70$, 60m target exit, 180m max exit, engine signal reversal). Zero freshness gates, zero trailing profit locks, zero behavioural filters. |
| **Data Provenance** | **Genuinely Fresh Market Sessions**: Prospective capture of NSE live trading sessions (09:15 to 15:30 IST) recorded strictly forward in time. Historical sessions (Sep 28 – Oct 01) remain closed. |
| **Execution Telemetry** | **Order-Book Physics Captured at Decision Time**:<br>1. Signal emission timestamp ($t_{\text{signal}}$).<br>2. Prevailing best bid, best ask, and mid-quote ($P_{\text{bid}}, P_{\text{ask}}, P_{\text{mid}}$).<br>3. Quoted bid-ask spread ($\text{Spread}_{\text{bps}} = (P_{\text{ask}} - P_{\text{bid}}) / P_{\text{mid}} \times 10,000$).<br>4. Simulated fill price ($P_{\text{fill}}$): Immediate crossing of the spread plus 0.5 bps adverse microstructure drift.<br>5. Fill latency ($\Delta t_{\text{fill}} = t_{\text{fill}} - t_{\text{signal}}$). |
| **Cost Model** | **Audited Cash Intraday Schedule (Zerodha Standard)**:<br>• Flat Brokerage: ₹20 per executed leg (max ₹40 round-trip).<br>• STT: 0.025% on sell-side turnover.<br>• Exchange Turnover Fees: 0.00297% on both legs.<br>• Stamp Duty: 0.003% on buy-side turnover.<br>• SEBI Charges: ₹10 per crore (0.0001%).<br>• GST: 18% on (Brokerage + Exchange + SEBI).<br>• *Execution slippage is measured and reported separately from statutory charges.* |
| **Primary Metric** | **Gross Return & Net Expectancy**: Measured in basis points ($\text{bps}$) of notional turnover: $$\text{Exp (bps)} = \frac{\sum \text{P\&L}_i}{\sum (\text{Entry Price}_i \times \text{Quantity}_i)} \times 10,000$$ |
| **Secondary Metrics** | 1. Net and Gross Win Rates (with binomial 95% confidence intervals).<br>2. Cash Drawdown (ordered chronologically by `exit_timestamp`).<br>3. Session Dispersion & Inter-Session Covariance.<br>4. Symbol Concentration (Gini coefficient & top-3 scrip profit contribution). |
| **Statistical Analysis** | **Cluster-Robust Inference**: Standard errors and $t$-statistics must be clustered at the session level ($df = S - 1$) to account for market-wide regime dependence across concurrent trades. |
| **Governance Invariants** | Strictly **0.0% Live Order Authority**. `adapters/chronosentiment` remains 100% frozen. Interim threshold tuning or feature adjustments are **strictly prohibited**. |

---

## 3. Evidence Sufficiency & Sample Size Rationale

### The Small-Sample Problem Identified in RP-606-ECO-001:
The historical sample of 2,029 trades spanned only **4 sessions**, where a single day (October 01: $+₹19,875.61$) artificially turned a $-₹9,882.08$ three-day deficit into a nominal pooled profit. Individual trade counts do not substitute for independent market days.

### Pre-Declared Prospective Sample Requirement:
To ensure statistical power and protect against single-day regime anomalies:
1. **Minimum Session Count**: $S \ge \mathbf{10 \text{ full, independent trading sessions}}$.
2. **Minimum Trade Count**: $N \ge \mathbf{1,000 \text{ completed trade observations}}$.
3. **Degrees of Freedom**: Session-clustered hypothesis test must have $df = S - 1 \ge 9$.
4. **No Interim Stopping**: Evaluation will run to full completion of the 10 sessions before formal disposition gates are applied (preventing early-stopping bias).

---

## 4. Pre-Declared Post-Validation Decision Gates

Upon completion of the $S \ge 10$ prospective validation sessions, the research program will execute one of three mutually exclusive decision pathways:

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                  RP-606-FRESH-001 DECISION TREE                                      |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    PROSPECTIVE VALIDATION RESULT                                     |
|                                       (S >= 10, N >= 1,000)                                          |
+───────────────────────────────────+──────────────────────────────────+───────────────────────────────+
                  │                                  │                                 │
                  ▼                                  ▼                                 ▼
+───────────────────────────────────+──────────────────────────────────+───────────────────────────────+
| BRANCH 1: CONTINUE RESEARCH       | BRANCH 2: REDESIGN HYPOTHESIS    | BRANCH 3: TERMINATE SIGNAL    |
+───────────────────────────────────+──────────────────────────────────+───────────────────────────────+
| CRITERIA:                         | CRITERIA:                        | CRITERIA:                     |
| 1. Gross Return is statistically  | 1. Gross Return is positive      | 1. Gross Return is negative   |
|    positive (p < 0.05 clustered). |    (R_gross > 0 bps), but        |    (R_gross <= 0 bps), OR     |
| 2. Mean Gross Return >= +5.0 bps. | 2. R_gross < 5.0 bps (too weak   | 2. Statistically indistin-    |
| 3. Positive in >= 65% of sessions.|    to survive 9 bps friction).   |    guishable from zero        |
| 4. Top session <= 40% of profit.  | 3. Fails economic safety hurdle. |    (p >= 0.05 clustered).     |
+───────────────────────────────────+──────────────────────────────────+───────────────────────────────+
| ACTION:                           | ACTION:                          | ACTION:                       |
| Investigate execution engineering | Redesign signal genesis:         | Permanently retire the signal |
| (passive queue pegging, maker     | Evaluate multi-hour/swing signal | hypothesis. Stop exit tuning. |
| rebates, institutional fees) to   | horizons capable of capturing    | Preserve adapter and archive  |
| capture the demonstrated edge.    | 20-30 bps gross movement.        | as unsupported by evidence.   |
+───────────────────────────────────+──────────────────────────────────+───────────────────────────────+
```

---

## 5. Implementation Protocol & Execution Verification

1. **Protocol Registration**: This document is registered in `docs/research/` prior to inspecting fresh data.
2. **Shadow Telemetry Harness**: Verify that the live shadow logger records bid, ask, spread, and timestamped fill drift alongside each generated signal.
3. **Zero Contamination**: The production adapter remains frozen; no live execution authority is granted; no retrospective changes to the historical holdout.
