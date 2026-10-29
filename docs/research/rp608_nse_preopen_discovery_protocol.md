# RP-608-NSE-PREOPEN-OBS-001: Pre-Open Discovery & Microstructure Research Protocol

**Document ID:** `RP-608-NSE-PREOPEN-OBS-001`  
**Series:** Prospective Microstructure Research (Pre-Open Information Extraction)  
**Parent Framework:** `RP-608-NSE-SHADOW-001` (119 Liquid NSE Equity Universe)  
**Date / Timestamp (UTC):** `2026-10-25T21:10:00Z` (`2026-10-26T02:40:00+05:30`)  
**Status:** **READINESS-AUDITED & NON-INTERFERING OBSERVATIONAL RESEARCH TRACK**  
**Execution Authority:** Strictly **0.0%** (Read-Only Telemetry; Zero Live Order Permissions)  
**Governance Invariant:** **Strict Isolation from Frozen Continuous Trading Policies (Estimand A & B)**

---

## 1. Executive Rationale & Research Objective

While `RP-608-NSE-SHADOW-001` formally excludes the 09:00–09:15 IST pre-open session from continuous signal execution, the pre-opening auction serves as the exchange's primary price-discovery mechanism. Overnight information, international sentiment shocks, and earnings releases are resolved through a multilateral call auction without trade executions until equilibrium is established.

> **Core Research Inquiry:**  
> Do the indicative equilibrium price, opening gap magnitude, order book quantity imbalance, and indicative imbalance persistence captured during the 09:00–09:15 IST auction contain statistically significant predictive information for:
> 1. **Immediate 60-Minute Directional Outcome** ($T_{\text{close}} = 10:15\text{ IST}$)?
> 2. **First-Hour Return Magnitude & Excursion** ($R_{\text{first\_hour}}$)?
> 3. **Full-Session Directional Sign** ($T_{\text{close}} = 15:30\text{ IST}$)?

---

## 2. Structural Exchange Physics (09:00 to 09:15 IST)

The official NSE pre-open session operates in three distinct phases:

| Phase Window | Exchange Operation | Observable Market-Data Features |
| :--- | :--- | :--- |
| **09:00:00 – 09:08:00 IST**<br>(Order Collection) | Limit & market orders entered, modified, or cancelled. No matching. | Accumulating `total_buy_qty` and `total_sell_qty`. Early indicative equilibrium price evolves dynamically. |
| **09:08:00 – 09:12:00 IST**<br>(Order Matching) | Order entry halts with random closure (variable system cutoff). Matching occurs at single clearing price. | Final confirmed indicative price ($P_{\text{ind}}$), matched executable volume, and remaining unmatched order imbalance (`total_imbalance_qty`). |
| **09:12:00 – 09:15:00 IST**<br>(Buffer Period) | Exchange transition to continuous session. No new orders or matching. | Confirmed final opening price ($P_{\text{open}}$) locked for 09:15 continuous trading. |

---

## 3. Mathematical Pre-Open Feature Definitions

All features are computed strictly using data timestamped prior to continuous trading ($T < 09:15:00\text{ IST}$):

### 3.1 Indicative Opening Gap ($\text{Gap}_{\text{open}}$)
$$\text{Gap}_{\text{open}} = \frac{P_{\text{ind}}(09:12) - P_{\text{prev\_close}}}{P_{\text{prev\_close}}} \times 10,000\text{ bps}$$
- Categorized as:
  - **Large Gap-Up:** $\text{Gap}_{\text{open}} \ge +75\text{ bps}$
  - **Moderate Gap-Up:** $+20\text{ bps} \le \text{Gap}_{\text{open}} < +75\text{ bps}$
  - **Flat / Neutral:** $-20\text{ bps} < \text{Gap}_{\text{open}} < +20\text{ bps}$
  - **Moderate Gap-Down:** $-75\text{ bps} < \text{Gap}_{\text{open}} \le -20\text{ bps}$
  - **Large Gap-Down:** $\text{Gap}_{\text{open}} \le -75\text{ bps}$

### 3.2 Quantity Demand–Supply Imbalance ($\text{Imbalance}_{\text{qty}}$)
$$\text{Imbalance}_{\text{qty}} = \frac{Q_{\text{buy}} - Q_{\text{sell}}}{Q_{\text{buy}} + Q_{\text{sell}} + \epsilon} \in [-1.0, \; +1.0]$$
where $Q_{\text{buy}}$ and $Q_{\text{sell}}$ represent total cumulative order quantities across the entire pre-open book at 09:08 IST.

### 3.3 Indicative Clearing Efficiency ($\text{Clear}_{\text{ratio}}$)
$$\text{Clear}_{\text{ratio}} = 1.0 - \frac{|Q_{\text{imbalance}}|}{Q_{\text{matched}} + |Q_{\text{imbalance}}| + \epsilon}$$
Measures the proportion of order flow absorbed at the equilibrium price versus residual overhang.

### 3.4 Pre-Open Trajectory Persistence ($\Delta P_{\text{evol}}$)
$$\Delta P_{\text{evol}} = P_{\text{ind}}(09:10) - P_{\text{ind}}(09:05)$$
Distinguishes between a stable equilibrium that built persistently across the auction versus a late-stage reversal.

---

## 4. Evaluated Target Outcomes

Pre-open features are evaluated against three strictly causal prospective targets:

1. **Target 1: Continuous 60-Minute Directional Outcome (Estimand B Aligned):**
   $$Y_{60} = \mathbb{I}(P_{\text{close}}(10:15) > P_{\text{open}}(09:15))$$
2. **Target 2: First-Hour Return Magnitude:**
   $$R_{\text{first\_hour}} = \frac{P_{\text{close}}(10:15) - P_{\text{open}}(09:15)}{P_{\text{open}}(09:15)} \times 10,000\text{ bps}$$
3. **Target 3: Full-Day Session Direction:**
   $$Y_{\text{day}} = \mathbb{I}(P_{\text{close}}(15:30) > P_{\text{open}}(09:15))$$

---

## 5. Architectural & Governance Safeguards

1. **Non-Contamination Rule:** Pre-open observations are written exclusively to `preopen_discovery_ledger.jsonl`. They do not enter `rp608_nse_decision_cards.jsonl` or `rp608_nse_trade_journal.jsonl`.
2. **Zero Live Execution Authority:** 0.0% order submission capability. Strictly observational telemetry.
3. **Preservation of RP-608 Frozen Baseline:** Continuous strategy signals continue to evaluate only completed 15-minute bars starting at Bar 1 close (09:30 IST). Pre-open signals will **never** alter position entries or trade boundaries during the 20-session prospective observation.
4. **Future Operational Gate:** If pre-open signals demonstrate positive Brier skill ($\text{BSS} > 0$) and balanced accuracy $> 55\%$ over 20 qualifying sessions, any operational use requires a formal protocol amendment and governance review prior to live deployment.
