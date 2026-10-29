# RP-608-PVP-003: Coralys-Driven Crypto Decision Integration Contract

**Document Identifier**: `RP-608-PVP-003`  
**Protocol Series**: `RP-608 (Momentum Exhaustion & Execution Validation)`  
**Successor To**: `RP-608-PVP-002` (Frozen Reference Protocol)  
**Date**: October 2026  
**Status**: REGISTERED INTEGRATION CONTRACT & FINAL FROZEN SPECIFICATION (v1.3)  
**Target Instrument**: Binance Spot `BTC/USDT` (15-Minute Completed Candles)  
**Live Order Authority**: Strictly **0.0%** (Shadow Simulation Only)  
**Core Architecture Boundary**: `coralys-*` Crates **100% FROZEN** (Adapter-Side Integration Only)  

---

## 1. Executive Mandate & Strategic Evolution

`RP-608-PVP-002` established a rigorous, falsifiable validation protocol for momentum exhaustion on historical and prospective Indian equity data. That protocol demonstrated that:
1. Exact strategy reversal is algebraically flawed and commercially unprofitable.
2. The pre-entry exhaustion filter acts as a descriptive loss-mitigation diagnostic rather than an autonomous alpha source.
3. Evaluating a hard-coded heuristic breakout strategy with a hard-coded filter tests the *human-designed rule*, not the *autonomous decision-making capacity* of the Coralys core engine.

To evaluate whether Coralys itself produces viable economic decisions, **RP-608-PVP-003** evolves the research programme:
- **RP-608-PVP-002 remains permanently frozen** as an immutable reference benchmark.
- **RP-608-PVP-003 establishes a Coralys-driven decision experiment** operating on continuous, liquid 24/7 cryptocurrency data (Binance `BTC/USDT`, 15-minute candles).
- **The exhaustion filter is demoted from a hard-coded execution blocker to an observational feature** available to Coralys's optimization and decision pipeline.
- Coralys is tasked with determining the trading action, timing, and risk trade-offs under realistic crypto market friction.

---

## 2. Public Interface Responsibilities & Core Freezing

In strict conformance with repository governance, all integration is confined to `adapters/chronosentiment/research/` behind the `research` feature flag. The `coralys-*` crates remain 100% untouched.

```mermaid
flowchart TD
    subgraph Data Layer
        A["Binance BTC/USDT<br/>15m Completed Candles"] -->|Strict Causal Ingestion<br/>close_time <= t_decision| B["ChronoSentiment Market State Adapter"]
    end

    subgraph Feature & Telemetry Layer
        B --> C["Base Features<br/>(Trend, Return, Range, Volatility)"]
        B --> D["Exhaustion Telemetry<br/>(VWAP Stretch, Momentum Accel, Vol Burst)"]
    end

    subgraph Coralys Decision Pipeline (Frozen Crates)
        C & D --> E["Stage 1: Candidate Evaluator<br/>(coralys-decision::CandidateEvaluator)"]
        E --> F["Stage 2: Offline Multi-Objective Engine<br/>(coralys-moga / RuleListGenome)"]
        F --> G["Stage 3: Frozen Inference Policy<br/>(first_match_action / DecisionPolicy)"]
    end

    subgraph Execution & Governance Layer
        G --> H["Sealed Decision Record<br/>(coralys-decision::DecisionRecord)"]
        H --> I["Shadow Execution Simulator<br/>(Monetary Fee & Slippage Model)"]
        I --> J["Research Observatory & Immutable Ledger<br/>(Live Order Authority: 0.0%)"]
    end
```

### 2.1 Interface Verification & State Transition Rules

1. **Candidate Action Proposals ([`coralys-decision::CandidateEvaluator`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/coralys-decision/src/traits.rs#L1-L5)):**
   - Evaluates completed 15-minute bar state to propose action eligibility ($\text{Proposal} \in \{\text{EligibleLong}, \text{EligibleExit}, \text{StandAside}\}$) and initial ATR volatility bounds.
   - **Verification Invariant:** Proposes eligibility only; does not force execution or embed hard-coded trading policy.
2. **State Transition Rules for Open Positions:**
   - When **Flat**:
     - `EligibleLong` $\implies$ Enters 1 unit long at next bar open $O_{t+1}$.
     - `StandAside` / `EligibleExit` $\implies$ Remains flat ($r_{t+1} = 0.00\text{ bps}$).
   - When **Long Position Open**:
     - `EligibleLong` $\implies$ Ignored (no pyramiding, max 1 unit exposure).
     - `StandAside` $\implies$ Ignored (position continues under trailing stop and 16-bar time limit).
     - `EligibleExit` $\implies$ Forces market close at next bar open $O_{t+1}$ unless already stopped out.
3. **Offline Rule Evolution ([`coralys-moga::EvolutionEngine`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/coralys-moga/src/engine.rs#L46-L80)):**
   - Evolves [`RuleListGenome`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/decision_support/policy_genome.rs#L13-L44) strictly on the historical training set.
   - At inference time during evaluation, the selected champion genome is evaluated deterministically via [`first_match_action`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/decision_support/policy_artifact.rs) without invoking optimization loops or mutation routines.
4. **Decision Sealing & Immutability ([`coralys-decision::DecisionRecord`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/coralys-decision/src/record.rs)):**
   - Sealed via [`DecisionRecordBuilder`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/coralys-decision/src/adapter.rs#L24-L27) and committed to [`DecisionLedger`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/coralys-decision/src/ledger.rs#L1-L28).
   - Preserves microsecond decision timestamp, input features, reference price $P_{\text{ref}}$, champion genome hash, simulated fill, and itemized friction.
5. **Execution Authority Isolation:**
   - The test binary operates in memory and local storage only. Zero live execution routes, network calls to trading endpoints, or API key configurations exist in the research harness. Live authority = **0.0%**.

---

## 3. Training / Freezing / Evaluation Boundary & Search Budget Parity

### 3.1 Historical Training Window ($T_{\text{train}}$)
- **Dataset Specification:** Continuous Binance Spot `BTC/USDT` completed 15-minute bars from `2024-01-01T00:00:00Z` to `2024-12-31T23:45:00Z` inclusive ($35,136$ bars).
- **Search Budget Parity Invariant:**
  - **Identical Population Size:** $P = 50$ genomes.
  - **Identical Generations:** $G = 30$ generations.
  - **Identical Crossover & Mutation:** Crossover rate $p_c = 0.80$, Mutation rate $p_m = 0.20$.
  - **Identical Multi-Objective Optimization:** Evaluated against the identical three-objective fitness vector:
    1. Net cumulative return after monetary friction.
    2. Maximum percentage drawdown penalty.
    3. Turnover drag penalty.
  - **Deterministic Seed Policy:** Primary champion evolution uses predeclared seed `0x42`. Seeds `0x1337` and `0xDEADBEEF` are reserved exclusively for offline stability validation, never for post-hoc cherry-picking among prospective champions.
  - **Ablation Isolation:** Policy C is allocated the **exact same** search budget, generation count, and selection mechanism as Policy B. The **sole difference** is the alphabet of factor predicates: Policy C includes the three exhaustion telemetry features ($\text{dist\_vwap\_bps}$, $\text{ret\_15m\_aligned}$, $\text{vol\_burst}$), whereas Policy B excludes them.

### 3.2 Policy Freezing Epoch ($T_{\text{freeze}}$)
- The non-dominated Pareto front is computed at Generation 30.
- Exactly one champion genome is selected per policy using the predeclared tie-breaking rule:
  $$\text{Calmar Ratio} = \frac{\text{Net Return}}{\max(\text{Max Drawdown}, 0.0001)}$$
  with tie-break to lowest turnover, then lexicographical genome identity hash.
- The champion genomes, configuration files, and training dataset checksums are locked prior to forward ingestion.

### 3.3 Prospective Evaluation Window ($T_{\text{eval}}$)
- Operates strictly on forward incoming unseen candles ($T > T_{\text{freeze}}$).
- **Zero Re-Evolution:** No in-flight mutation, threshold modification, or retraining is permitted.

---

## 4. Rigorous Portfolio Accounting & Causal P&L Mechanics

To eliminate double-counting and correctly handle same-bar entry/exit, multi-bar holding, and midnight boundaries:

### 4.1 Fixed Reference Capital & Sizing Invariant
1. **Reference Capital:** Policies A, B, and C are initialized with identical fixed starting capital:
   $$K_0 = 100,000.00 \text{ USDT}$$
2. **Fixed Unit Exposure:** Active positions are sized to exactly $1.0 \times K_0$ notional at entry:
   $$Q_t = \frac{K_0}{P_{\text{entry}}}$$
   Zero leverage, zero pyramiding. Maximum 1 open position at any time.

### 4.2 Bar-by-Bar Mark-to-Market Accounting Equations

Let $t$ index 15-minute intervals ($t = 1, \dots, 96$ per UTC calendar day).

#### Case 1: Same-Bar Entry and Exit ($t_{\text{entry}} = t_{\text{exit}} = t$)
When an entry fill occurs at bar open $O_t$ and the stop is touched within the same bar at price $P_{\text{exit}}$:
$$\text{Gross P\&L}_t = Q \cdot (P_{\text{exit}} - P_{\text{entry}})$$
$$\text{Net P\&L}_t = \text{Gross P\&L}_t - \text{Friction}_{\text{entry}} - \text{Friction}_{\text{exit}}$$

#### Case 2: Multi-Bar Position Holding ($t_{\text{entry}} < t \le t_{\text{exit}}$)
1. **Entry Bar ($t = t_{\text{entry}}$):** Fills at $P_{\text{entry}} = O_t$, position held through close $C_t$:
   $$\text{Gross P\&L}_t = Q \cdot (C_t - P_{\text{entry}})$$
   $$\text{Net P\&L}_t = \text{Gross P\&L}_t - \text{Friction}_{\text{entry}}$$
2. **Intermediate Holding Bars ($t_{\text{entry}} < t < t_{\text{exit}}$):** Position remains open; marked close-to-close:
   $$\Delta \text{Unrealized}_t = Q \cdot (C_t - C_{t-1})$$
   $$\text{Net P\&L}_t = \Delta \text{Unrealized}_t \quad (\text{zero intermediate friction charged})$$
3. **Exit Bar ($t = t_{\text{exit}}$):** Position closes at price $P_{\text{exit}}$:
   $$\text{Gross P\&L}_t = Q \cdot (P_{\text{exit}} - C_{t-1})$$
   $$\text{Net P\&L}_t = \text{Gross P\&L}_t - \text{Friction}_{\text{exit}}$$

#### Telescoping Sum Integrity Proof
For any trade spanning bars $t_0$ through $t_m$, summing $\text{Gross P\&L}_t$ yields:
$$Q(C_{t_0} - P_{\text{entry}}) + \sum_{j=1}^{m-1} Q(C_{t_j} - C_{t_{j-1}}) + Q(P_{\text{exit}} - C_{t_{m-1}}) \equiv Q(P_{\text{exit}} - P_{\text{entry}})$$
This guarantees zero double-counting, exact conservation of value across calendar day and midnight UTC boundaries, and exact trade-level parity.

#### Case 3: Flat / Stand-Aside Interval
When no position is active during bar $t$:
$$\text{Net P\&L}_t = 0.00 \text{ USDT}, \quad \text{Friction}_t = 0.00 \text{ USDT}$$

### 4.3 Normalized Opportunity Return ($\text{bps}_t$)
Every 15-minute interval $t$ produces a scale-invariant return normalized by starting capital $K_0$:
$$r_{t} = \frac{\text{Net P\&L}_t}{K_0} \times 10,000 \quad (\text{bps})$$
All flat intervals remain strictly in the denominator with $r_t = 0.00\text{ bps}$.

---

## 5. Causal Trailing Stop Sequence & Reference Baseline (Policy A)

### 5.1 Strict Causal Trailing Stop Sequence
To prevent lookahead bias, trailing stops must strictly decouple evaluation from post-bar updates:

1. **Step 1 (Prior Knowledge):** At the beginning of bar $t$, the active stop is $S_{t-1}$, determined strictly by completed bar information prior to $t$.
2. **Step 2 (Intrabar Evaluation):** The open, high, and low of bar $t$ are evaluated against $S_{t-1}$:
   - **Gap-Through-Stop:** If $O_t < S_{t-1}$, stop execution occurs at $O_t$ (adverse gap open fill), **never** at $S_{t-1}$.
   - **Intrabar Stop Touch:** If $O_t \ge S_{t-1}$ and $L_t \le S_{t-1}$, stop execution occurs at $S_{t-1}$.
   - **Dual-Touch Ambiguity:** If in the same bar $L_t \le S_{t-1}$ and $H_t$ touches profit exit or max duration, adverse priority forces execution at the stop price.
3. **Step 3 (Post-Bar Causal Update):** If and only if the position survives through bar close $C_t$, the new stop is updated for bar $t+1$:
   $$S_t = \max(S_{t-1}, C_t - 2.0 \times \text{ATR}_{14}(C_t))$$
   $S_t$ becomes active starting at the open of bar $t+1$, never retroactively inside bar $t$.

### 5.2 Deterministic Reference Specification (Policy A)
1. **Breakout Lookback:** Long trigger when completed close $C_{-1} > \max(H_{-2}, \dots, H_{-21})$ (20-bar Donchian).
2. **Entry Timing:** Executes at open of subsequent bar $O_{\text{entry}} = O_0$.
3. **Initial Stop:** $S_0 = O_{\text{entry}} - 2.0 \times \text{ATR}_{14}(C_{-1})$, active for bar $t=0$.
4. **Max Holding Horizon:** Exactly 16 bars (4 hours). If not stopped out, exit executes at $C_{15}$ close.
5. **Rolling VWAP:** 96 completed 15-minute bars (24-hour rolling window):
   $$\text{VWAP}_t = \frac{\sum_{i=0}^{95} \left(\frac{H_{t-i} + L_{t-i} + C_{t-i}}{3}\right) \cdot V_{t-i}}{\sum_{i=0}^{95} V_{t-i}}$$

---

## 6. Monetary Cost Accounting & Exact Friction Schedules

To eliminate discrepancies between entry notional, exit notional, and headline bps shorthand, all costs are calculated in exact monetary currency:

### 6.1 Monetary Friction Schedule (Binance Spot VIP0)

| Cost Component | Pricing Formula | Currency Basis |
| :--- | :--- | :--- |
| **Entry Taker Fee** | $\text{Fee}_{\text{entry}} = P_{\text{entry}} \cdot Q \cdot 0.0005$ | 5.00 bps of actual entry notional |
| **Entry Slippage** | $\text{Slip}_{\text{entry}} = P_{\text{entry}} \cdot Q \cdot 0.0001$ | 1.00 bp of actual entry notional |
| **Exit Taker Fee** | $\text{Fee}_{\text{exit}} = P_{\text{exit}} \cdot Q \cdot 0.0005$ | 5.00 bps of actual exit notional |
| **Exit Slippage** | $\text{Slip}_{\text{exit}} = P_{\text{exit}} \cdot Q \cdot 0.0001$ | 1.00 bp of actual exit notional |
| **Base Friction Total** | $\text{Friction}_{\text{base}} = \text{Fee}_{\text{entry}} + \text{Slip}_{\text{entry}} + \text{Fee}_{\text{exit}} + \text{Slip}_{\text{exit}}$ | Exact monetary currency |
| **Adverse Stress Increment** | $\text{Stress}_{\text{adverse}} = P_{\text{entry}} \cdot Q \cdot 0.0002$ | 2.00 bps of actual entry notional |
| **Stressed Friction Total** | $\text{Friction}_{\text{stressed}} = \text{Friction}_{\text{base}} + \text{Stress}_{\text{adverse}}$ | Exact monetary currency |

- **Headline "12 bps" / "14 bps" Shorthand:** Reflects the nominal round-trip drag when $P_{\text{exit}} \approx P_{\text{entry}}$. Actual simulation strictly applies the exact monetary equations above trade-by-trade.

---

## 7. Dependence-Aware Statistical Inference & Independent Gate Reporting

### 7.1 Daily Aggregation & HAC Inference
1. **Daily Aggregated Portfolio Return:**
   For each 24-hour UTC calendar day $k = 1, \dots, D$ (where each day contains 96 completed 15-minute intervals):
   $$R_{\pi, k} = \sum_{t=1}^{96} r_{\pi, k, t} \quad (\text{bps of } K_0) \quad \text{for } \pi \in \{A, B, C\}$$
2. **Paired Daily Increments:**
   $$d_{C-A, k} = R_{C, k} - R_{A, k}, \quad d_{C-B, k} = R_{C, k} - R_{B, k}$$
3. **Newey-West HAC Test Statistic:**
   Mean daily increment: $\bar{d} = \frac{1}{D} \sum_{k=1}^D d_k$.  
   The variance is estimated using the Newey-West estimator with lag truncation $L = 3$ days:
   $$\hat{V}_{\text{HAC}} = \hat{\gamma}_0 + 2 \sum_{j=1}^{L} \left(1 - \frac{j}{L + 1}\right) \hat{\gamma}_j, \quad \text{where } \hat{\gamma}_j = \frac{1}{D} \sum_{k=j+1}^D (d_k - \bar{d})(d_{k-j} - \bar{d})$$
   $$t_{\text{HAC}} = \frac{\bar{d}}{\sqrt{\hat{V}_{\text{HAC}} / D}}, \quad df = D - 1$$

### 7.2 Pre-Registered Falsification Gates & Independent Reporting

All four gates will be computed and reported independently in the final experimental evaluation report, preventing negative or inconclusive evidence from being suppressed.

| Gate | Focus | Evaluation Metric | Pre-Registered Hurdle Requirement |
| :--- | :--- | :--- | :--- |
| **Gate 1** | **Relative Superiority** | Daily Mean Delta vs Baseline ($\bar{d}_{C-A}$) | $\bar{d}_{C-A} \ge +3.00\text{ bps}$ ($p_{\text{HAC}} < 0.01$, one-sided, $df = D-1$) |
| **Gate 2** | **Ablation Validity** | Incremental Value of Exhaustion ($\bar{d}_{C-B}$) | $\bar{d}_{C-B} \ge +1.50\text{ bps}$ ($p_{\text{HAC}} < 0.05$, conditional on Gate 1) |
| **Gate 3** | **Absolute Economic Viability** | Stressed Daily Net Return ($R_{C, k}$) | Cumulative $\sum R_{C, k} > 0.00\text{ bps}$ under stressed monetary friction |
| **Gate 4** | **Regime Consistency** | Daily Win Rate Consistency | Positive Net Days $\ge \lceil 0.70 \times D \rceil$ ($21/30$ days for $D = 30$) |

- **Sample Horizon Mandate:** Minimum **$D \ge 30$ UTC calendar days** and **$N \ge 250$ completed round-trip trades**.
- **Stopping Floor Discipline:** 30 days is a minimum stopping floor, not a guarantee of power. If statistical gates fail to reject the null, the formal result is **"Inconclusive / Null Not Rejected"**, not an excuse for post-hoc threshold adjustment.

---

## 8. Precise Falsification Dispositions & Scientific Scoping

To avoid overgeneralizing experimental findings beyond what the evidence supports:

1. **If Policy C Fails Gate 3:**
   - **Defensible Conclusion:** The *registered Coralys-driven BTC/USDT 15-minute configuration* failed its absolute economic viability hurdle under the specified stressed cost model.
   - **Prohibited Claim:** Do not claim that "all Coralys-driven cryptocurrency trading is commercially unviable."
2. **If Policy C Passes Gate 3 but Fails Gate 2:**
   - **Defensible Conclusion:** Incremental value from the pre-entry momentum exhaustion features was *not demonstrated* under this registered configuration.
   - **Prohibited Claim:** Do not claim that "exhaustion features are redundant across all trading architectures," as non-inferiority requires a dedicated equivalence test.
3. **If Policy C Fails Gate 1:**
   - **Defensible Conclusion:** Coralys autonomous optimization did not demonstrate statistically significant superiority over a simple deterministic breakout heuristic under identical market conditions.

---

## 9. Governance Checklist for Implementation Readiness

- [x] `coralys-*` crates strictly untouched (0 files modified).
- [x] Live order authority locked at 0.0%.
- [x] RP-608-PVP-002 permanently frozen as historical benchmark.
- [x] Identical starting capital ($K_0 = 100\text{k}$ USDT) and mark-to-market accounting.
- [x] Same-bar entry/exit and multi-bar telescoping accounting equations verified.
- [x] Strict causal trailing stop sequence ($S_{t-1}$ evaluated during bar $t$; $S_t$ active at $t+1$).
- [x] Exact monetary friction formulas codified for base and stressed regimes.
- [x] Search budget parity ($P=50, G=30$, identical seeds) and Calmar tie-breaking defined.
- [x] Completed round-trip trades ($N \ge 250$) and 30 UTC calendar days mandated.
- [x] Daily HAC dependence-aware inference and independent gate reporting codified.
