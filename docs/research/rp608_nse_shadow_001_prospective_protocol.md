# RP-608-NSE-SHADOW-001: Prospective Equity Observation Protocol
## 119 Liquid NSE Symbol Universe Shadow Execution & Statutory Friction Verification

**Protocol ID:** `RP-608-NSE-SHADOW-001`  
**Series:** Prospective Predictive Validation (Indian Equity Shadow Telemetry)  
**Parent Framework:** RP-605 / RP-607 / RP-608-PVP-002  
**Target Instrument:** 119 Liquid NSE Equity Universe (Nifty 100 + 19 Liquid Active Trackers)  
**Live Order Authority:** Strictly **0.0%** (Read-Only Shadow Daemon; Zero Order Submission Capability)  
**Core Crate Modification:** Strictly **0.0%** (Zero mutations to `coralys-moga`, `coralys-decision`, `coralys-ecology`)  
**Prospective Validation Clock:** **TRANSITION_PENDING** (Technical Pilot ACTIVE; Formal 20-Session Window Activates `2026-10-26T03:45:00Z` / 09:15 IST)  
**Status:** **READINESS-AUDITED & FROZEN** (Pre-activation audit passed; zero live order authority)  

---

## 1. Executive Summary & Research Rationale

While `RP-608-SHADOW-001` evaluates crypto microstructure (Binance BTC/USDT) 24/7, `RP-608-NSE-SHADOW-001` evaluates the parallel empirical question in the Indian equity market:

> **Core Research Inquiry:**  
> Does the frozen NSE decision strategy demonstrate positive net expectancy after accounting for realistic Indian market friction (statutory taxes, exchange turnover charges, brokerage, bid/ask spread crossing, and execution slippage) across the predeclared 119 NSE symbol universe?

The historical NSE holdout (Sep 28 – Oct 01, 2026) showed $+₹9,994$ gross P&L but $-₹1.71\text{ lakh}$ net P&L after friction. To distinguish between a structurally unviable strategy and a friction-sensitive execution profile, prospective forward-looking observations must be captured under authentic live session conditions without reopening the historical holdout.

---

## 2. Predeclared & Reconciled 119 NSE Symbol Universe

The prospective observation is strictly restricted to the following census of **119 liquid NSE equity symbols** (reconciled 1-to-1 against the historical data cache without omissions, substitutions, or post-hoc pruning):

```
ADANIENT.NS, ADANIGREEN.NS, ADANIPORTS.NS, AMBUJACEM.NS, APOLLOHOSP.NS, ASIANPAINT.NS,
ATGL.NS, AUROPHARMA.NS, AXISBANK.NS, BAJAJ-AUTO.NS, BAJAJFINSV.NS, BAJFINANCE.NS,
BALKRISIND.NS, BANDHANBNK.NS, BANKBARODA.NS, BERGEPAINT.NS, BHARATFORG.NS, BHARTIARTL.NS,
BIOCON.NS, BOSCHLTD.NS, BPCL.NS, BRITANNIA.NS, CANBK.NS, CHOLAFIN.NS,
CIPLA.NS, COALINDIA.NS, COFORGE.NS, COLPAL.NS, CONCOR.NS, CUMMINSIND.NS,
DABUR.NS, DIVISLAB.NS, DLF.NS, DRREDDY.NS, EICHERMOT.NS, ESCORTS.NS,
FEDERALBNK.NS, GAIL.NS, GLENMARK.NS, GODREJCP.NS, GODREJPROP.NS, GRASIM.NS,
HAL.NS, HAVELLS.NS, HCLTECH.NS, HDFCBANK.NS, HDFCLIFE.NS, HEROMOTOCO.NS,
HINDALCO.NS, HINDUNILVR.NS, ICICIBANK.NS, ICICIGI.NS, ICICIPRULI.NS, IDEA.NS,
IDFCFIRSTB.NS, IGL.NS, INDHOTEL.NS, INDUSINDBK.NS, INDUSTOWER.NS, INFY.NS,
IOC.NS, IPCALAB.NS, IRCTC.NS, ITC.NS, JINDALSTEL.NS, JSWSTEEL.NS,
JUBLFOOD.NS, KOTAKBANK.NS, LICI.NS, LT.NS, LUPIN.NS, M&M.NS,
MAHABANK.NS, MARICO.NS, MARUTI.NS, MPHASIS.NS, MRF.NS, MUTHOOTFIN.NS,
NATIONALUM.NS, NAUKRI.NS, NESTLEIND.NS, NMDC.NS, NTPC.NS, OBEROIRLTY.NS,
OFSS.NS, ONGC.NS, PAGEIND.NS, PERSISTENT.NS, PETRONET.NS, PFC.NS,
PIDILITIND.NS, PNB.NS, POLYCAB.NS, POWERGRID.NS, RECLTD.NS, RELIANCE.NS,
SAIL.NS, SBICARD.NS, SBILIFE.NS, SBIN.NS, SHREECEM.NS, SIEMENS.NS,
SRF.NS, SUNPHARMA.NS, TATACONSUM.NS, TATASTEEL.NS, TCS.NS, TECHM.NS,
TITAN.NS, TMCV.NS, TORNTPHARM.NS, TRENT.NS, TVSMOTOR.NS, ULTRACEMCO.NS,
UNITDSPR.NS, UPL.NS, VBL.NS, VEDL.NS, WIPRO.NS
```

*(Governance Note: Earlier draft text referenced '102 symbols' due to typographical transposition of the ~120 target universe. This protocol formally freezes the census at exactly the 119 listed scrips. Zero dynamic substitutions or universe modifications are permitted during observation).*

---

## 3. Session Schedule & Fixed Calendar Boundaries

- **Observation Window:** Exactly 20 qualifying standard NSE trading sessions.
- **Conditional Close Primacy:** The observation window is strictly governed by the completion of the 20th qualifying trading session ($S = 20$). The nominal calendar end (20 November 2026) serves only as an unadjusted planning baseline. In the event of exchange holidays, unscheduled closures, or truncated sessions, the window extends session-by-session until exactly 20 qualifying sessions have been observed.
- **Fixed Calendar Period:**
  - **Start Boundary:** Monday, 26 October 2026, 09:15:00 IST (`2026-10-26T03:45:00.000Z`).
  - **Nominal End Boundary:** Friday, 20 November 2026, 15:30:00 IST (`2026-11-20T10:00:00.000Z`) [Subject to conditional 20th qualifying session completion].
- **Session Timings:** 09:15 to 15:30 IST (03:45 to 10:00 UTC), Monday through Friday.
- **Pre-Open Exclusion:** 09:00 to 09:15 IST excluded from signal execution (discovery phase only).
- **Intraday Terminal Closeout:** All positions open at 15:15 IST are causally squared off at the prevailing market bid/ask prior to the 15:30 IST market close. Zero overnight gap risk.
- **Exchange Holiday Extension:** Any declared NSE market holiday automatically extends the observation window day-for-day until exactly 20 qualifying sessions are completed. The actual terminal timestamp is determined strictly by the close of the 20th qualifying session.
- **Exceptional Session Exclusions:**
  - **Diwali Muhurat Trading Session:** The 1-hour special evening Muhurat session is excluded from standard intraday evaluation due to non-standard structural liquidity.
  - **Disrupted / Truncated Sessions:** Any session truncated to less than 4 trading hours (exchange outage, regulatory halts) is excluded from the 20-session tally and recorded in the manifest.


---

## 4. Indian Statutory & Transaction Friction Physics

Every simulated equity trade leg incurs realistic statutory and broker charges modeled with exact Indian regulatory schedules. The $9.0\text{ bps}$ opportunity-ratio denominator is an ex-ante screening reference; actual trade accounting computes full multi-component fees:

| Fee Component | Statutory / Market Schedule | Applied Leg |
| :--- | :--- | :--- |
| **Brokerage** | $\min(₹20, \; \text{Turnover} \times 0.03\%)$ | Both Buy & Sell legs |
| **STT (Securities Transaction Tax)** | **0.025%** on turnover | Sell leg only (Equity Intraday) |
| **Exchange Transaction Charges** | **0.00297%** on turnover | Both Buy & Sell legs |
| **GST** | **18.0%** on (Brokerage + Exchange Charges + SEBI Charges) | Both Buy & Sell legs |
| **SEBI Turnover Charges** | **₹10 per Crore** (0.0001% on turnover) | Both Buy & Sell legs |
| **Stamp Duty** | **0.003%** on turnover | Buy leg only |
| **Spread Crossing** | Half-spread based on observed L1 top-of-book quote | Both Buy & Sell legs |
| **Slippage** | **2.0 bps** exogenous adverse execution latency | Both Buy & Sell legs |

---

## 5. Dual Estimands & Tripartite Acceptance Architecture

To prevent mixing distinct prediction tasks, the protocol formally separates two estimands:

### 5.1 Estimand A: Trade-Outcome Prediction (Executed Trade Lifecycle)
- **Target:** For entered trades $i \in \{1, \dots, N\}$, whether the trade realizes positive gross return under the frozen exit rules:
  $$Y_{\text{trade}, i} = \mathbb{I}(R_{i, \text{gross}} > 0.00)$$
- **Reported Metrics:** Precision (Win Rate), Gross Expectancy, Gross Profit Factor, Mean MFE, Mean MAE, Edge Ratio.
- **Statistical Invariant:** Balanced accuracy is **strictly marked unavailable (N/A)** for Estimand A, because non-entry decisions do not enter the trade ledger ($TN = 0, FN = 0$).

### 5.2 Estimand B: Directional Opportunity Prediction (Fixed-Horizon Task)
- **Target:** Evaluated across all eligible 15-minute completed bar decision timestamps $k$ for all 119 symbols.
  - **1-Based Completed Bar Indexing Convention:**
    - The NSE session $[09:15:00, 15:30:00\text{ IST}]$ is partitioned into $K = 25$ fifteen-minute intervals: $\text{Interval}(k) = [09:15 + (k-1)\times 15\text{m}, \; 09:15 + k\times 15\text{m}]$, indexed $k \in \{1, \dots, 25\}$.
    - Every bar $k$ is indexed and emitted at its **closing timestamp** $T_{\text{close}}(k) = 09:15:00 + k\times 15\text{m}$.
    - Decision timestamp: $T_{\text{decision}}(k) \equiv T_{\text{close}}(k)$, with reference price $P_{\text{ref}}(k) \equiv P_{\text{close}}(k)$.
    - Outcome timestamp: $T_{\text{outcome}}(k) \equiv T_{\text{close}}(k+4) = T_{\text{close}}(k) + 60\text{ minutes}$, with outcome price $P_{\text{outcome}}(k) \equiv P_{\text{close}}(k+4)$.
  - **Same-Session Boundary Rule:** To prevent overnight holding risk, an interval is eligible if and only if $T_{\text{outcome}}(k) \le 15:30:00\text{ IST}$ ($k+4 \le 25 \iff k \le 21$).
  - **Eligible Intervals:** Bars $k \in \{1, \dots, 21\}$ (21 bars/session $\times$ 20 sessions $\times$ 119 symbols = **49,980 eligible intervals**). Bars 22–25 are tagged `IntradayBoundaryExcluded`.
  - Prediction: $\hat{D}_{s, k} \in \{+1\text{ (Long Entry)}, 0\text{ (Flat/No Entry)}\}$.
  - Ground-Truth Label: $D_{s, k} = \mathbb{I}(P_{\text{close}}(k+4) > P_{\text{close}}(k))$.
- **Reported Metrics:** Complete Confusion Matrix ($TP, FP, TN, FN$), Sensitivity (Recall), Specificity, Balanced Accuracy $= (\text{Sensitivity} + \text{Specificity})/2$, Precision, and Calibration.

### 5.3 Training Prevalence Brier Reference & Skill Score
To evaluate genuine calibration rather than arbitrary score hurdles:
- **Ex-Ante Forecasts:** Probabilities are recorded at $T_{\text{decision}}$: Directional Long = 0.58, Flat = 0.38.
- **Brier Score:** $\text{BS} = \frac{1}{N} \sum_{i=1}^N (p_i - Y_i)^2$.
- **Direct Sample-Evaluated Reference Benchmark ($\text{BS}_{\text{ref}}$):** Evaluated directly on the identical prospective sample:
  $$\text{BS}_{\text{ref}} = \frac{1}{N} \sum_{i=1}^N (y_i - \bar{y}_{\text{train}})^2$$
  - Directional Long Trades (Estimand A): $\bar{y}_{\text{train}} = 0.58$.
  - Directional Session Bars (Estimand B): $\bar{y}_{\text{train}} = 0.512$.
- **Brier Skill Score ($\text{BSS}$):**
  $$\text{BSS} = 1 - \frac{\text{BS}}{\text{BS}_{\text{ref}}}$$
  *(The uninformative coin-toss score $\text{BS}_{\text{uninf}} = 0.25$ is reported separately as a descriptive reference).*
- **Gate Requirement:** $\text{BS} \le \text{BS}_{\text{ref}}$ AND $\text{BSS} > 0.00$ (verifying skill over unconditional training prevalence on the identical sample).

### 5.4 Pre-Registered Acceptance Decision Gates
1. **Layer 1 Gate (Primary Predictive Acceptance Gate):**
   - **Precision (Win Rate):** $\ge 56.0\%$ across the 119-symbol universe.
   - **Wilson 95% Confidence Interval:** Lower bound $> 50.0\%$ (rejecting the random coin-flip null).
   - **Brier Calibration:** $\text{BS} \le \text{BS}_{\text{ref}}$ and $\text{BSS} > 0.00$.
   - **Balanced Accuracy (Estimand B):** $\ge 55.0\%$.
   - **Sample Size Floor:** $N \ge 30$, $N_{\text{eff}} \ge 10$.
2. **Layer 2 Gate (Mandatory Trade Quality & Excursion Gate):**
   - **Gross Expectancy:** $\ge +12.0\text{ bps}$.
   - **Gross Profit Factor:** $\ge 1.30$.
   - **Edge Ratio ($\text{MFE} / \text{MAE}$):** $\ge 1.25$.
3. **Layer 3 Gate (Economic Viability Gate):**
   - **Net Expectancy:** $> 0.0\text{ bps}$ after all statutory friction and execution slippage.
   - **Net Profit Factor:** $> 1.0$.
   - **Net Realized P&L:** $> ₹0.00$.
   - **Friction Drag Ratio:** $\le 80.0\%$ of gross profit.

### 5.5 Dependence-Aware Uncertainty Analysis & $N_{\text{eff}}$ Safeguards
Standard Wilson intervals provide descriptive independent-Bernoulli bounds. For formal acceptance:
- **Block Bootstrap by Trading Session:** Confidence intervals are evaluated by resampling entire 1-day trading sessions as blocks (with replacement), preserving cross-sectional correlation across the 119 symbols and intraday temporal autocorrelation.
- **Effective Sample Size Estimator with Bandwidth Selection:**
  $$N_{\text{eff}} = \frac{N}{\max\left(1.0, \; 1 + 2 \sum_{k=1}^{K^*} \hat{\rho}_k\right)}$$
  where:
  - Bandwidth $K = \min\left(\lfloor 4(N/100)^{2/9} \rfloor, \; K_{\max}\right)$.
  - Truncation rule $K^*$: sum truncates at the first lag $k$ where $\hat{\rho}_k \le 0$ (Geyer initial positive sequence rule).
  - **Safeguard Invariant:** Denominator is floored at $1.0$ (VIF $\ge 1.0$). Positive autocorrelation strictly reduces $N_{\text{eff}} \le N$; negative autocorrelation or noise is forbidden from inflating $N_{\text{eff}} > N$.
- **Cluster Degrees of Freedom Boundary:** Across 20 trading sessions, cluster-level degrees of freedom cannot exceed 20 ($S \le 20$). For symbol-level cells, a minimum of 10 independent qualifying sessions ($S_s \ge 10$) is mandatory; if a symbol trades in fewer than 10 sessions, its cluster count is flagged and classified as `InconclusiveInsufficientSample`.

### 5.6 Mutually Exclusive Hierarchical Outcome Decision Tree
Every cell is classified via an ordered, non-overlapping decision tree:
1. **Sample Floor Check:** If $N < 30$, $N_{\text{eff}} < 10$, or session cluster count $S < 10 \implies$ `InconclusiveInsufficientSample`.
2. **Statistical Falsification of Null:** If Wilson 95% CI upper bound $\le 50.0\% \implies$ `FalsifiedZeroEdge` (statistically demonstrated absence of positive edge).
3. **Layer 1 Pass Evaluation:** If Layer 1 passes (Precision $\ge 56.0\%$, Wilson CI lower bound $> 50.0\%$, $\text{BSS} > 0.0$):
   - If Layer 2 fails $\implies$ `PredictiveEdgeSubstandardQuality`.
   - If Layer 2 passes and Layer 3 passes $\implies$ `ConfirmedPredictiveEdgeAndViable`.
   - If Layer 2 passes and Layer 3 fails $\implies$ `PredictiveEdgeOnlyFrictionBound`.
4. **Inconclusive Evidence:** If adequately sampled ($N \ge 30, N_{\text{eff}} \ge 10, S \ge 10$), but meets neither Layer 1 pass criteria nor statistical falsification (e.g. precision $> 50\%$ but CI spans $50\%$, or precision between $50\%$ and $56\%$) $\implies$ `InconclusiveEvidence`.

---

## 6. Price-Volume Compartmentalization Framework (119 Symbols)

### 6.1 Strictly Causal Pre-Decision Invariant & Market Tracks
All metrics are computed strictly using historical daily bars completed prior to each session ($T < 09:15\text{ IST}$):
1. **Price Track Dimensions:**
   - **Range Magnitude:** $\text{NATR}_{20} = (\text{ATR}_{20} / P_t) \times 10,000\text{ bps}$.
   - **Directional Efficiency:** $\text{Eff}_{20} = |P_t - P_{t-20}| / \sum_{\tau=1}^{20} \text{TR}_\tau \in [0, 1]$.
   - **Autoregressive Reversion:** $\text{AR}_1 = \text{Corr}(r_t, r_{t-1})$ over 60 daily sessions.
   - **Range Stability:** $\text{CV}_{\text{range}} = \sigma(\text{TR}_{20}) / \mu(\text{TR}_{20})$.
   - **Opportunity Ratio:** $\text{Opp}_{\text{ratio}} = \text{NATR}_{20} / 9.0\text{ bps}$ (reference friction multiple).
2. **Volume Track Dimensions (Strictly Causal Look-Ahead Protection):**
   - **Relative Volume ($\text{RVOL}_{20}$):** Ratio of completed session $d-1$ volume to trailing 20-day daily SMA ($V_{d-1} / \overline{V}_{20}$). Never accesses session $d$'s total volume.
   - **Volume Burst Index ($\text{VBI}$):** Standardized volume shock ($(V_{d-1} - \mu_V) / \sigma_V$).
   - **Intraday Volume Pace ($\text{IVP}_t$):** Cumulative volume through candle $t-1$ relative to historical mean cumulative volume at time $t-1$.
   - **Liquidity Depth Ratio ($\text{LDR}$):** Pre-decision top-of-book depth relative to hypothetical order size.

### 6.2 Friction-Boundary Redundancy Resolution
In the $\mathcal{C}_4$ compartment definition:
$$\text{NATR}_{20} < 160\text{ bps} \quad \text{OR} \quad \text{Opp}_{\text{ratio}} < 18\times$$
Since $\text{Opp}_{\text{ratio}} = \text{NATR}_{20} / 9.0\text{ bps}$, $\text{Opp}_{\text{ratio}} < 18$ implies $\text{NATR}_{20} < 162\text{ bps}$. While $\text{Opp}_{\text{ratio}} < 18$ mathematically subsumes $\text{NATR}_{20} < 160\text{ bps}$, both conditions remain frozen to enforce the dimensional rule that daily movement must provide at least $18\times$ one-way friction coverage.

### 6.3 Predeclared Compartment Taxonomy & Hierarchy
1. **$\mathcal{C}_4$: Low-Movement / Friction-Bound / Volume-Drained:**
   - Criteria: $\text{NATR}_{20} < 160\text{ bps}$ OR $\text{Opp}_{\text{ratio}} < 18\times$ OR $\text{RVOL}_{20} < 0.60$.
2. **$\mathcal{C}_1$: High-Range Directional Movers with Active Volume:**
   - Criteria: $\text{NATR}_{20} \ge 220\text{ bps}$ AND $\text{Eff}_{20} \ge 0.14$ AND $\text{AR}_1 \ge -0.10$ AND $\text{RVOL}_{20} \ge 1.00$.
3. **$\mathcal{C}_2$: High-Range Volatile Oscillators with Volume Expansion:**
   - Criteria: $\text{NATR}_{20} \ge 220\text{ bps}$ AND $\text{Eff}_{20} < 0.12$ AND $\text{AR}_1 \le -0.12$ AND $\text{RVOL}_{20} \ge 1.00$.
4. **$\mathcal{C}_3$: Stable Moderate Movers with Normal Participation:**
   - Criteria: $160\text{ bps} \le \text{NATR}_{20} < 220\text{ bps}$ AND $\text{CV}_{\text{range}} \le 0.45$.
5. **$\mathcal{C}_{\text{Unclassified}}$:**
   - Incomplete historical records (< 20 daily bars) or boundary transition points.

### 6.4 Reporting Invariants
1. **Strictly Pre-Decision Assignment:** Compartment assignment strictly uses $T < T_{\text{decision}}$ data.
2. **Tri-Level Reporting:** Aggregate, per-symbol (all 119 scrips), and per-compartment.
3. **Sample Size & CI Disclosure:** Disclose $N, N_{\text{eff}}$, Wilson CI, and bootstrap CI for every cell.
4. **Zero Retuning Rule:** Zero threshold tuning or universe modifications during the 20-session window.

---

## 7. Architectural Isolation on Shared Host

The Binance and NSE shadow daemons run on the same VM host under strict process isolation:
1. **Zero Credential Overlap:** Read-only feeds; zero order placement API keys.
2. **Independent Ledgers:** Separate directories, databases, and log streams.
3. **Decoupled Reporting:** Both publish independent 4-part deliverables at window completion.

---

## 8. Predefined Deliverables & Decision Gates

At the completion of the 20-session observation window, the NSE daemon must publish:
1. **NSE Data Integrity Report:** Feed uptime, missing ticks, tick latency, and quote freshness across all 119 symbols.
2. **NSE Microstructure & Friction Report:** Realized statutory friction, exchange fees, STT paid, brokerage, and effective spread across all 119 symbols.
3. **NSE Prospective Predictive Accuracy & Compartmentalized Performance Report:** Estimand A (Trade-Outcome) and Estimand B (Directional Opportunity), Wilson CIs, block bootstrap CIs, and 6-tier outcome breakdown.
4. **NSE Governance Conformance Manifest:** Checksum verification of strategy engine, log of all drops, session qualification audit, and conclusive verdict on predictive edge. **Published regardless of outcome.**
