# RP-606-ECO-001: Gross Edge, Friction Attribution & Economic Viability Contract

**Document Identifier**: `RP-606-ECO-001`  
**Protocol Series**: `RP-606 (Economic Edge & Execution Attribution)`  
**Date**: 2026-10-22  
**Status**: REGISTERED CONTRACT & COMPLETED PHASE 1 DIAGNOSTIC  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Executive Mandate & Strategic Purpose

Following the closure and archival of `RP-605-EXIT-001` and `RP-605-BEH-001`, the research program pivots from win-rate targeting to **fundamental economic viability**.

The central finding of the RP-605 provenance audit established that:
- **Pooled Gross P&L**: $+₹9,993.52$ ($+0.52\text{ bps}$ of notional turnover).
- **Trading Friction**: $-₹181,029.04$ ($9.00\text{ bps}$ of notional turnover).
- **Net P&L**: $-₹171,035.52$ (Net Expectancy: $-8.50\text{ bps}$).

The fundamental question of RP-606 is not *"how to improve the win rate,"* but:
> **"Does the underlying directional strategy generate a statistically significant, repeatable gross advantage per unit of traded notional sufficient to survive realistic execution costs?"**

---

## 2. The Five Core Economic Inquiries & Empirical Results

Evaluated across all **2,029 completed trades** across 99 liquid NSE equities (September 28 – October 01, 2026):

### Inquiry 1: Is the baseline's gross performance statistically distinguishable from zero?
- **Finding**: **NO. Gross performance is statistically indistinguishable from zero.**
- **Trade-Level Statistics ($N = 2,029$)**:
  - Mean Gross Return: **+0.5222 bps** ($+₹4.93$ per trade on $₹100\text{k}$ notional).
  - Standard Error ($SE$): **0.6106 bps**.
  - $95\%$ Confidence Interval: **$[-0.6746\text{ bps}, +1.7190\text{ bps}]$**.
  - $t$-statistic: **$t = 0.8552$** ($p$-value $= 0.3926$).
- **Session-Clustered Statistics ($df = 3$)**:
  - Session Returns: `[-2.92 bps, +0.36 bps, +0.35 bps, +4.23 bps]`
  - Clustered $t$-statistic: **$t = 0.3471$** ($p$-value $= 0.7514$).
- **Diagnostic Implication**: The positive gross return of $+0.52\text{ bps}$ cannot be distinguished from pure random drift. Optimizing exits or adding entry filters around an unproven gross edge represents curve-fitting noise.

---

### Inquiry 2: Which costs account for the gap between gross and net performance?
- **Finding**: **Statutory taxes and broker fees account for 92.4% of total trading friction.**
- **Cost Decomposition on a Standard ₹100,000 Notional Intraday Trade (₹200,000 Turnover)**:
  | Cost Component | Legal / Regulatory Rate | Cost (₹) | Drag (bps) | Share of Friction (%) |
  | :--- | :--- | :---: | :---: | :---: |
  | **Brokerage** | ₹20 per executed order | ₹40.00 | 4.00 bps | 44.8% |
  | **STT (Securities Transaction Tax)** | 0.025% on sell turnover | ₹25.00 | 2.50 bps | 28.0% |
  | **GST** | 18% on (Brokerage + Exchange + SEBI) | ₹8.31 | 0.83 bps | 9.3% |
  | **NSE Exchange Charges** | 0.00297% on both buy & sell | ₹5.94 | 0.59 bps | 6.7% |
  | **Stamp Duty** | 0.003% on buy turnover | ₹3.00 | 0.30 bps | 3.4% |
  | **SEBI Turnover Charges** | ₹10 per crore (0.0001%) | ₹0.20 | 0.02 bps | 0.2% |
  | **Statutory & Brokerage Subtotal** | *Non-negotiable cash fees* | **₹82.45** | **8.25 bps** | **92.4%** |
  | **Execution Slippage / Spread Drift**| ~0.75 bps round-trip | ₹6.77 | 0.75 bps | 7.6% |
  | **Total Realized Friction** | *Authoritative Ledger Mean* | **₹89.22** | **9.00 bps** | **100.0%** |

Brokerage (₹40) and STT (₹25) alone create a 6.50 bps non-negotiable hurdle before market microstructure begins.

---

### Inquiry 3: Is the result concentrated in a small number of symbols, sessions or trades?
- **Finding**: **YES. Gross profit is heavily clustered in a single session and a minority of symbols.**
- **Session Dispersion**:
  - **Session 1 (2026-09-28)**: Gross **−₹13,370.89** (−2.92 bps) | Net −₹54,455.71 (WR: 14.1%)
  - **Session 2 (2026-09-29)**: Gross **+₹1,773.21** (+0.36 bps) | Net −₹47,077.54 (WR: 18.5%)
  - **Session 3 (2026-09-30)**: Gross **+₹1,715.60** (+0.35 bps) | Net −₹47,076.40 (WR: 21.8%)
  - **Session 4 (2026-10-01)**: Gross **+₹19,875.61** (+4.23 bps) | Net −₹22,425.87 (WR: 26.9%)
  - *Observation*: Without Session 4, the prior three sessions produced a cumulative **Gross Loss of −₹9,882.08**.
- **Symbol Concentration**:
  - Only **44 symbols (44.4%)** achieved positive gross P&L.
  - **55 symbols (55.6%)** were gross negative before costs.
  - Top 3 symbols (`BOSCHLTD.NS` +₹4.2k, `AMBUJACEM.NS` +₹3.3k, `JINDALSTEL.NS` +₹3.2k) generated $+₹10,719.43$, which exceeds the entire strategy's pooled gross P&L (+₹9,993.52).

---

### Inquiry 4: What gross edge is required to overcome costs with an economic safety margin?
- **Finding**: **A viable strategy requires a 14.00 to 18.00 bps gross edge (27x to 35x current levels).**
- **Economic Hurdle Calculation**:
  - Average Realized Friction ($\bar{C}$): **9.00 bps** ($₹89.22$ per trade).
  - Break-Even Requirement: Gross Expectancy $\ge \mathbf{9.00\text{ bps}}$.
  - Minimum Commercial Viability (+5.00 bps net edge): Gross Expectancy $\ge \mathbf{14.00\text{ bps}}$ ($₹140.00$ per trade).
  - Robust Institutional Viability (+9.00 bps net edge, Profit Factor $\ge 1.6$): Gross Expectancy $\ge \mathbf{18.00\text{ bps}}$ ($₹180.00$ per trade).
- **The Economic Gap**:
  $$\text{Gap} = \frac{\text{Required Gross Edge (14.00 bps)}}{\text{Current Gross Edge (0.52 bps)}} = \mathbf{26.8\times}$$

---

### Inquiry 5: Decision Gate Verdict: Continue, Redesign, or Terminate?
- **Predeclared Phase 1 Decision Gate**:
  - *Condition A*: If gross edge cannot be statistically distinguished from zero ($p > 0.05$), halt entry/exit policy tuning immediately.
  - *Condition B*: If gross edge is $< \bar{C}$ ($< 9.00\text{ bps}$), the strategy cannot be deployed in cash equities under standard retail fee schedules.
- **Formal Decision**:
  - **HALT all exit, trailing lock, and entry-filter optimization on the current baseline signal.**
  - The small gross profit (+0.52 bps) is statistical noise ($p = 0.39$). Optimizing around noise produces curve-fitting.
  - Proceed to **Phase 2 (`RP-606-FRESH-001`)**: Run an untouched, prospective shadow validation on genuinely fresh sessions to observe whether the signal has any forward gross persistence, while exploring whether the underlying thesis requires larger holding horizons or institutional fee tiering.

---

## 3. Four-Phase Program Architecture

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                  RP-606 FOUR-PHASE ROADMAP                                           |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
| PHASE 1: RP-606-ECO-001 (Economic Attribution) [COMPLETE]                                           |
| • Falsified standalone gross edge (0.52 bps vs 9.00 bps friction; p = 0.39).                         |
| • Identified 27x economic gap between current gross and viable edge.                                 |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
                                      │
                                      ▼
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
| PHASE 2: RP-606-FRESH-001 (Prospective Shadow Validation) [NEXT MILESTONE]                           |
| • Run untouched baseline in shadow mode on fresh sessions (Oct 2026 onward).                         |
| • Observe gross price drift without tuning or parameter changes.                                     |
| • Record executable bid/ask spreads, order timing, and fill physics.                                 |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
                                      │
                                      ▼
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
| PHASE 3: Signal Genesis Redesign (Subject to Phase 2 Gross Edge Evidence)                            |
| • Re-evaluate signal horizon: Can multi-hour / swing horizons generate 20–30 bps gross drift?       |
| • Evaluate institutional fee schedules (reducing brokerage from 4.0 bps to 0.5 bps).                |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
                                      │
                                      ▼
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
| PHASE 4: Independent Economic Readiness Review                                                       |
| • Formal review before any production consideration. Adapter remains frozen.                         |
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
```
