# RP-608-SHADOW-001B: Read-Only Prospective Observation Protocol
## 30-Day Forward Microstructure & Shadow Execution Telemetry Specification

**Protocol ID:** `RP-608-SHADOW-001B`  
**Series:** Prospective Predictive Validation (Read-Only Shadow Telemetry)  
**Parent Milestone:** `RP-608-TEL-001A` (Cost Reconciliation & Provenance Audit)  
**Target Instrument:** Binance Spot BTC/USDT (`btcusdt`)  
**Live Order Authority:** Strictly **0.0%** (Read-Only Shadow Daemon; Zero API Key Trading Permissions)  
**Core Crate Modification:** Strictly **0.0%** (Zero mutations to `coralys-moga`, `coralys-decision`, `coralys-ecology`)  
**Prospective Validation Clock:** **TRANSITION_PENDING** (Technical Pilot ACTIVE; Formal 30-Day Window Activates `2026-10-26T00:00:00Z`)  
**Status:** **READINESS-AUDITED & FROZEN** (Pre-activation audit passed; zero live order authority)  

---

## 1. Executive Summary & Governance Objectives

Following the formal closure and sign-off of milestone `RP-608-TEL-001A`, this protocol transitions the project from historical holdout reconstruction to **prospective observational measurement**. 

The fundamental purpose of `RP-608-SHADOW-001B` is to collect genuinely forward-looking, unmanipulated market evidence to determine:
1. Whether the frozen strategy correctly predicts trade outcomes on unseen forward live-market data (predictive truth).
2. How that predictive accuracy behaves across predeclared historical price-volume compartments.
3. Whether that predictive edge survives realistic top-of-book microstructure and execution friction (economic viability).

### Core Governance Principles
1. **Separation of Simulation from Observation:** Historical replay outputs ($+\$2.17\text{M}$ and $+\$1.09\text{M}$) are recognized as *Controlled Microstructure Stress Proxy Scenario results*, not empirical proof of profitability. Genuine empirical evidence requires prospective observations from real exchange feeds.
2. **Permanent Closure of Historical Optimization:** MOGA genetic search, genome rule mutations, and parameter retuning are permanently closed. The 2024 dataset holdout will not be reopened or revisited.
3. **No Adaptive Steering:** The prospective observation period must not be used to select a champion, tweak indicator thresholds, alter stop distances, or revise the protocol after observing initial returns.
4. **Predeclared Gating & Missing-Data Rules:** Data-quality gates and handling rules for missing or stale quotes are locked prior to the first observation. If quality gates fail, the failure is reported as a governance breach—the window is not silently extended or selectively trimmed.

---

## 2. Fixed Observation Window Specification

To ensure zero retroactive backdating following runtime audit sign-off, the window is re-anchored as `RP-608-SHADOW-001B` with a two-phase schedule:

| Window Phase | Timestamp (UTC) | Purpose & Constraints |
| :--- | :--- | :--- |
| **Phase 1: Technical Pilot** | `2026-10-25T03:00:00.000Z` to `2026-10-26T00:00:00.000Z` (21 hours) | Feed connectivity verification, clock-drift telemetry stabilization, zero order-authority audit. Not counted in 30-day statistical performance. |
| **Phase 2: Registered 30-Day Window** | `2026-10-26T00:00:00.000Z` (`1792972800000` ms) to `2026-11-25T00:00:00.000Z` (`1795564800000` ms) | Immutable 30-day forward prospective evaluation window. Exactly **2,880 completed 15-minute bars**. |

> [!IMPORTANT]
> **Statistical Inference Limitation:** A fixed 30-day window does not guarantee statistical power. If market volatility remains low or signals cluster, the effective degrees of freedom ($N_{\text{trades}}$ and daily return independence) will be constrained. Statistical significance must be evaluated based on the realized trade count and autocorrelation-adjusted standard errors, not assumed from the 30-day duration alone. If $N_{\text{trades}} < 30$ or $N_{\text{eff}} < 10$, the outcome will be formally classified as **inconclusive due to insufficient statistical power**.

---

## 3. Market-Data Stream Architecture & Timestamp Alignment

The telemetry collector ingests two concurrent, asynchronous WebSocket streams directly from the Binance Spot Public API:

```
Binance Public WebSocket Feed
   │
   ├── wss://stream.binance.com:9443/ws/btcusdt@kline_15m
   │      └── Completed Candle Event (k.x == true) -> Decision Event T_close
   │
   └── wss://stream.binance.com:9443/ws/btcusdt@bookTicker
          └── Real-Time Top-of-Book Quote Stream -> Execution Quote T_open
```

### 3.1 Field-by-Field Capture Schema

For every candidate execution event, the telemetry engine captures:

```rust
pub struct ProspectiveL1Record {
    pub exchange_event_time_ms: i64,      // Field 'E' from exchange payload (UTC)
    pub local_wallclock_utc_ms: i64,       // Local system UTC wall-clock time
    pub local_receipt_mono_ns: u64,        // Monotonic CLOCK_MONOTONIC receipt timestamp
    pub sequence_update_id: u64,           // Field 'u' from bookTicker
    pub best_bid_price: f64,               // Field 'b'
    pub best_bid_qty: f64,                 // Field 'B'
    pub best_ask_price: f64,               // Field 'a'
    pub best_ask_qty: f64,                 // Field 'A'
    pub spread_bps: f64,                   // ((ask - bid) / mid) * 10,000
    pub clock_drift_ms: i64,               // local_wallclock_utc_ms - exchange_event_time_ms
}
```

### 3.2 Clock Mapping & Reconciliation
A monotonic timestamp cannot be directly compared with an exchange UTC timestamp without a defined mapping. The telemetry daemon enforces:
1. **Initial Clock Anchor:** At startup ($T_0$), the daemon records the anchor triplet:
   $$\text{Anchor} = (T_{\text{exchange}, 0}, T_{\text{wallclock}, 0}, T_{\text{monotonic}, 0})$$
2. **Drift Measurement:** Clock drift is measured directly between local wall-clock and exchange UTC:
   $$\Delta_{\text{clock}} = T_{\text{wallclock}} - T_{\text{exchange}}$$
   Periodic NTP synchronization must maintain $|\Delta_{\text{clock}}| \le 250\text{ ms}$.
3. **Monotonic Latency Measurement:** The interval between quote arrival and hypothetical policy fill is computed strictly using the monotonic clock ($\Delta T_{\text{mono}} = T_{\text{mono, exec}} - T_{\text{mono, recv}}$), preventing leap-second or NTP adjustments from corrupting internal latency accounting.

### 3.3 Microstructure Limitations Disclosed
1. **Top-of-Book Limitation:** Binance Spot `@bookTicker` provides the best bid and best ask ($L_1$), but does not provide market depth below top-of-book ($L_2$/$L_3$) or order-book queue priority.
2. **Quote Staleness as Quality Gate vs Execution Guarantee:** A quote freshness threshold ($\le 500\text{ ms}$) is a **data-quality criterion**, not a guarantee that an aggressive market order would have executed at that price. Fast cancellations and priority queue depletion may occur within microseconds of receipt.
3. **Displayed Depth as Liquidity Estimate:** Top-of-book quantity is treated strictly as an estimate of available immediate liquidity, not a guaranteed fill allotment.

---

## 4. Preregistered Data-Quality Gates & Strict Missing-Data Protocols

To guarantee data integrity, the prospective run is evaluated against five preregistered quality gates:

| Quality Gate | Threshold | Description & Interaction Rules |
| :--- | :--- | :--- |
| **Gate 1: Quote Coverage** | $\ge \mathbf{99.50\%}$ of bars | At least 2,866 of 2,880 completed bars must have a valid L1 quote within $500\text{ ms}$. |
| **Gate 2: Quote Staleness** | $\le \mathbf{0.50\%}$ of bars | No more than 14 bars across the 30-day window may exhibit a quote age $> 500\text{ ms}$. |
| **Gate 3: Clock Synchronization** | $|\Delta_{\text{clock}}| \le \mathbf{250\text{ ms}}$ | Local system time vs Binance exchange NTP time drift must remain under 250 ms. |
| **Gate 4: Feed Disconnections** | $\le \mathbf{3\text{ per 24 hours}}$ | Maximum 3 reconnection events in any rolling 24-hour window; max downtime $< 60\text{ s}$. |
| **Gate 5: Missing Bar Intervals** | $\mathbf{0}$ consecutive $\ge 2$ bars | Any consecutive outage lasting $\ge 2$ bars ($30\text{ min}$) triggers immediate FAIL. |

### 4.1 Precision on Isolated vs Consecutive Missing Bars
To resolve the interaction between Gate 1 ($\ge 99.5\%$ coverage) and Gate 5 (consecutive gaps):
- **Isolated Missing Bars:** If a single 15-minute bar is missing due to a transient network drop (e.g., 1 bar dropped, up to a maximum of 14 isolated occurrences across the 30 days):
  - Gate 5 **PASSES** (since no consecutive outage $\ge 2$ bars occurred).
  - Gate 1 **PASSES** (coverage remains $\ge 99.50\%$).
  - The bar is logged as `MissingBarInterval`, no trade can execute on that interval, and total evaluated bars are reported as $2,880 - N_{\text{missing}}$.
- **Consecutive Outage Breach:** If $\ge 2$ consecutive bars ($30\text{ minutes}$) are missing:
  - Gate 5 **FAILS** immediately (`FAIL: Consecutive Outage Breach`).
  - Total window status is marked as tainted by outage.
- **Coverage Breach:** If total cumulative missing bars exceed 14 (even if all are isolated):
  - Gate 1 **FAILS** immediately (`FAIL: Coverage Breach (< 99.50%)`).

### 4.2 Missing-Data Accounting Safeguard (Opportunity Registry)
A dropped signal is **NOT an executed trade with zero profit**. Treating unexecutable signals as $0.00$ trades dilutes average trade expectancy, artificially deflates loss severity, and distorts the win-rate denominator.

Therefore, the telemetry engine enforces an explicit **Opportunity Accounting Registry**:
1. **Preserved Opportunity:** The hypothetical signal is assigned an immutable `OpportunityID` and logged in `OpportunityAccountingRegistry`.
2. **Explicit Exclusion from Trade Denominator:** Dropped opportunities are marked `UnexecutableDrop` and **strictly excluded from the executed trade population** ($N_{\text{executed}}$).
3. **Execution Drop Rate:** Reported separately as an execution degradation metric:
   $$\text{Execution Drop Rate} = \frac{N_{\text{unexecutable}}}{N_{\text{total\_opportunities}}} \times 100\%$$
4. **No Backfilling or Window Shifting:** Missing intervals are **never retrospectively backfilled** from historical REST APIs, and the 30-day window is **never extended** to compensate for downtime.

---

## 5. Frozen Policy Verification & Tamper-Proof Startup Check

The shadow engine must strictly enforce read-only execution and verify champion integrity before ingesting the first quote.

### 5.1 Authoritative Locked Cryptographic Hashes

| Policy | Container File Path | Container File SHA-256 | Internal Genome Rule Hash |
| :--- | :--- | :--- | :--- |
| **Policy B (Base)** | `frozen_policy_b_champion.json` | `f7be46ea66ff6a248ad0477dfe5b082ce26fe01f9dded928b164bdaa349c6aa3` | `6a0e39600389ed489b95153498eaf244dbc75864b9259ee70d6e07211eb30ddf` |
| **Policy C (Exhaustion)** | `frozen_policy_c_champion.json` | `cd615f37b812a34fe302a0b05bc1546a0243f3d71d8947092d28d2847159553b` | `80c689bae15e5fe9ec3c525cf0dc37bdaad17d4b896faab91beacd0b8fd6de05` |

### 5.2 Mandatory Startup Assertions
At daemon initialization:
1. Re-read files from disk and calculate SHA-256 digests. If mismatch $\implies$ abort startup immediately.
2. Re-serialize internal `RuleListGenome` and verify genome hashes. If mismatch $\implies$ abort startup immediately.
3. Assert absence of live trading API keys (`BINANCE_API_SECRET` must be unset or empty).
4. Assert zero outbound order submission endpoints in codebase.
5. Record verification status in startup audit manifest.

---

## 6. Execution-Price Semantics & Friction Accounting Physics

### 6.1 Execution-Price Semantics, Clock Mapping & Staleness Filtering
1. **Decision Boundary:** At completed candle event $T_{\text{close}}$ (e.g. second `:59.999` of minute 14, 29, 44, or 59), the policy evaluates signals.
2. **First Quote Search:** The engine searches for the first quote where $T_{\text{quote\_event}} \ge T_{\text{close}}$ and local receipt $T_{\text{quote\_recv}} \ge T_{\text{decision\_eval}}$.
3. **Decoupled Clock Mapping & Sign Convention:**
   The exchange clock offset is strictly defined as:
   $$\Delta_{\text{clock}} \equiv T_{\text{local\_wall}} - T_{\text{exchange}}$$
   (positive when local clock leads exchange time), estimated via periodic NTP round-trip preflight:
   $$\Delta_{\text{clock}} = \left(T_{\text{send}} + \frac{\text{RTT}}{2}\right) - T_{\text{server}}, \quad U = \frac{\text{RTT}}{2}$$
   where $|\Delta_{\text{clock}}| \le 250\text{ ms}$ and $U \le 25\text{ ms}$.
   Exchange-aligned local receipt time is $T_{\text{recv\_aligned}} = T_{\text{quote\_recv}} - \Delta_{\text{clock}}$.
   The physical elapsed transit latency (quote age) at execution is:
   $$\Delta t = T_{\text{quote\_recv}} - T_{\text{quote\_event}} - \Delta_{\text{clock}}$$
   If nominal quote age $\Delta t > 500\text{ ms}$, the stale quote is discarded and the engine waits for the next quote.
4. **Execution Timeout (5,000 ms):** If no valid, non-stale quote arrives within $5{,}000\text{ ms}$ of $T_{\text{close}}$ $\implies$ the signal is declared `UnexecutableQuoteTimeout` and recorded in the drop registry.
5. **Distinct Timestamps Recorded:**
   - $T_{\text{decision}}$: When candle closed and rule evaluation ran.
   - $T_{\text{quote\_event}}$: When exchange emitted the fill quote ($E$).
   - $T_{\text{quote\_recv}}$: When local WebSocket received the quote.
   - $T_{\text{execution\_hypothetical}}$: When the hypothetical fill was registered in the shadow ledger.

### 6.2 Fill Physics & Itemized Cost Breakdown
- **Long Entry Fill Price:** $P_{\text{fill}} = P_{\text{ask}}$ (crosses top-of-book half-spread).
- **Long Exit Fill Price:** $P_{\text{fill}} = P_{\text{bid}}$ (crosses top-of-book half-spread).

#### Itemized Friction Ledger (Per Leg):
1. **Exchange Taker Fee:** $\text{Fee}_{\text{usdt}} = \text{Notional} \times \frac{5.00}{10,000}$ ($5.0\text{ bps}$).
2. **Exogenous Latency Slippage:** $\text{Slippage}_{\text{usdt}} = \text{Notional} \times \frac{1.00}{10,000}$ ($1.0\text{ bp}$).
3. **Market Impact (Depth Exhaustion):**  
   If $\text{Order Qty} > 0.5 \times \text{Displayed Top-of-Book Depth}$:
   $$\text{Impact}_{\text{bps}} = \left(\sqrt{\frac{\text{Order Qty}}{\text{Depth Qty}} - 0.5} \times 0.5\right) \times \text{Half-Spread}_{\text{bps}}$$
   Otherwise: $\text{Impact}_{\text{bps}} = 0.00\text{ bps}$.

#### Anti-Double-Counting Invariant:
$$\text{Fill Price} = P_{\text{ask}} \implies \text{Spread Crossing is Already Embedded in Cash Flow}$$
$$\text{Total Explicit Cash-Flow Drag} = \text{Taker Fee} + \text{Slippage} + \text{Market Impact}$$
Half-spread crossing is logged for decomposition, but **never added to cash-flow drag** when fill prices are executed at Bid/Ask.

### 6.3 Terminal Closeout Rule at Window Boundary
Any position open at the final bar boundary (`bar 2,879` close at `2026-11-25T00:00:00Z`) is **forcibly closed out using the latest observed bid price $P_{\text{bid}}$ with timestamp $T_{\text{quote}} \le 2026-11-25\text{T}00:00:00\text{Z}$**. The implementation strictly forbids ingesting any quote with timestamp $> 2026-11-25\text{T}00:00:00\text{Z}$ to compute the final exit. Standard exit taker fee and slippage are applied.

---

## 7. Dual Estimands & Tripartite Prospective Acceptance Architecture

To separate predictive signal truth from execution friction, prospective evaluation is structured across two distinct estimands and three acceptance layers:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      TRIPARTITE PROSPECTIVE ACCEPTANCE LAYERS                   │
├──────────────────────────┬──────────────────────────────────────┬───────────────┤
│ Layer                    │ Evaluated Metrics                    │ Purpose       │
├──────────────────────────┼──────────────────────────────────────┼───────────────┤
│ Layer 1: Predictive      │ Precision, Balanced Accuracy, Recall,│ Primary Gate: │
│ Accuracy                 │ Specificity, Brier Score, Wilson CI  │ Signal Truth  │
├──────────────────────────┼──────────────────────────────────────┼───────────────┤
│ Layer 2: Trade Quality & │ Gross Expectancy, Profit Factor,     │ Mandatory     │
│ Excursion Dynamics       │ Mean MFE, Mean MAE, Edge Ratio       │ Quality Gate  │
├──────────────────────────┼──────────────────────────────────────┼───────────────┤
│ Layer 3: Execution       │ Net Expectancy, Net Profit Factor,   │ Economic      │
│ Realism & Survivability  │ Net Realized P&L, Friction Drag %    │ Viability Gate│
└──────────────────────────┴──────────────────────────────────────┴───────────────┘
```

### 7.1 Formal Separation of Estimands

#### Estimand A: Trade-Outcome Prediction (Executed Trade Lifecycle)
- **Target:** For entered trades $i \in \{1, \dots, N\}$, whether the trade realizes positive gross return under the frozen exit rules:
  $$Y_{\text{trade}, i} = \mathbb{I}(R_{i, \text{gross}} > 0.00)$$
- **Reported Metrics:** Precision (Win Rate), Gross Expectancy, Gross Profit Factor, Mean MFE, Mean MAE, Edge Ratio.
- **Statistical Invariant:** Balanced accuracy is **strictly marked unavailable (N/A)** for Estimand A, as non-entry decisions are excluded from the executed trade ledger ($TN = 0, FN = 0$).

#### Estimand B: Directional Opportunity Prediction (Fixed-Horizon Task)
- **Target:** Evaluated across all 15-minute decision timestamps $t$:
  - Prediction: $\hat{D}_t \in \{+1\text{ (Long Trigger)}, 0\text{ (Flat/No Entry)}\}$.
  - Preregistered Outcome Horizon: $H = 4\text{ bars}$ (60 minutes).
  - Ground-Truth Label: $D_t = \mathbb{I}(P_{t+H} > P_t)$.
- **Reported Metrics:** Confusion Matrix ($TP, FP, TN, FN$), Sensitivity, Specificity, Balanced Accuracy $= (\text{Sensitivity} + \text{Specificity})/2$, Precision, Calibration.

### 7.2 Training Prevalence Brier Reference & Skill Score
To evaluate genuine calibration rather than arbitrary score hurdles:
- **Ex-Ante Forecasts:** Probabilities are recorded at $T_{\text{decision}}$: Policy B = 0.62, Policy C = 0.65, Flat = 0.35.
- **Brier Score:** $\text{BS} = \frac{1}{N} \sum_{i=1}^N (p_i - Y_i)^2$.
- **Direct Sample-Evaluated Reference Benchmark ($\text{BS}_{\text{ref}}$):** Evaluated directly on the identical prospective sample:
  $$\text{BS}_{\text{ref}} = \frac{1}{N} \sum_{i=1}^N (y_i - \bar{y}_{\text{train}})^2$$
  - Policy B Entered Trades (Estimand A): $\bar{y}_{\text{train}} = 0.62$.
  - Policy C Entered Trades (Estimand A): $\bar{y}_{\text{train}} = 0.65$.
  - Estimand B (Directional Market Bars): $\bar{y}_{\text{train}} = 0.508$.
- **Brier Skill Score ($\text{BSS}$):**
  $$\text{BSS} = 1 - \frac{\text{BS}}{\text{BS}_{\text{ref}}}$$
  *(The uninformative coin-toss score $\text{BS}_{\text{uninf}} = 0.25$ is reported separately as a descriptive reference).*
- **Gate Requirement:** $\text{BS} \le \text{BS}_{\text{ref}}$ AND $\text{BSS} > 0.00$ (verifying skill over unconditional training prevalence on the identical sample).

### 7.3 Locked Decision Gates
1. **Layer 1 Gate (Primary Acceptance Objective - Signal Truth):**
   - **Precision (Win Rate):** $\ge 60.0\%$.
   - **Wilson 95% Confidence Interval:** Lower bound $> 50.0\%$ (falsifying random coin-flip null at $p < 0.05$).
   - **Brier Calibration:** $\text{BS} \le \text{BS}_{\text{ref}}$ and $\text{BSS} > 0.00$.
   - **Balanced Accuracy (Estimand B):** $\ge 55.0\%$.
   - **Sample Size Floor:** $N \ge 30$, $N_{\text{eff}} \ge 10$.
2. **Layer 2 Gate (Mandatory Trade Quality & Excursion Gate):**
   - **Gross Expectancy:** $\ge +15.0\text{ bps}$.
   - **Gross Profit Factor:** $\ge 1.30$.
   - **Edge Ratio ($\text{MFE} / \text{MAE}$):** $\ge 1.25$.
3. **Layer 3 Gate (Economic Viability Gate):**
   - **Net Expectancy:** $> 0.0\text{ bps}$ after realistic exchange fees, spread crossing, slippage, and impact.
   - **Net Profit Factor:** $> 1.0$.
   - **Net Realized P&L:** $> \$0.00$.
   - **Friction Drag Ratio:** $\le 80.0\%$ of gross profit.

### 7.4 Dependence-Aware Uncertainty Analysis & $N_{\text{eff}}$ Safeguards
Standard Wilson intervals provide descriptive independent-Bernoulli bounds. For formal acceptance:
- **Block Bootstrap by 24-Hour Session:** Evaluated by resampling 24-hour daily blocks (with replacement), preserving multi-bar intraday holding dependency.
- **Effective Sample Size Estimator with Bandwidth Selection:**
  $$N_{\text{eff}} = \frac{N}{\max\left(1.0, \; 1 + 2 \sum_{k=1}^{K^*} \hat{\rho}_k\right)}$$
  where:
  - Bandwidth $K = \min\left(\lfloor 4(N/100)^{2/9} \rfloor, \; K_{\max}\right)$.
  - Truncation rule $K^*$: sum truncates at the first lag $k$ where $\hat{\rho}_k \le 0$ (Geyer initial positive sequence rule).
  - **Safeguard Invariant:** Denominator is floored at $1.0$ (VIF $\ge 1.0$). Positive autocorrelation strictly reduces $N_{\text{eff}} \le N$; negative autocorrelation or estimation noise is forbidden from artificially inflating $N_{\text{eff}} > N$.
  - **Cluster Floor:** Cell must include trades from at least 10 independent daily clusters ($S \ge 10$); otherwise, cell is classified `InconclusiveInsufficientSample`.

### 7.5 Mutually Exclusive Hierarchical Outcome Decision Tree
Every cell is classified via an ordered, non-overlapping decision tree:
1. **Sample Floor Check:** If $N < 30$ or $N_{\text{eff}} < 10 \implies$ `InconclusiveInsufficientSample`.
2. **Statistical Falsification of Null:** If Wilson 95% CI upper bound $\le 50.0\% \implies$ `FalsifiedZeroEdge` (statistically demonstrated absence of positive edge).
3. **Layer 1 Pass Evaluation:** If Layer 1 passes (Precision $\ge 60.0\%$, Wilson CI lower bound $> 50.0\%$, $\text{BSS} > 0.0$):
   - If Layer 2 fails $\implies$ `PredictiveEdgeSubstandardQuality`.
   - If Layer 2 passes and Layer 3 passes $\implies$ `ConfirmedPredictiveEdgeAndViable`.
   - If Layer 2 passes and Layer 3 fails $\implies$ `PredictiveEdgeOnlyFrictionBound`.
4. **Inconclusive Evidence:** If adequately sampled ($N \ge 30, N_{\text{eff}} \ge 10$), but meets neither Layer 1 pass criteria nor statistical falsification (e.g. precision $> 50\%$ but CI spans $50\%$, or precision between $50\%$ and $60\%$) $\implies$ `InconclusiveEvidence`.

---

## 8. Price-Volume Compartmentalization Framework

Evaluating predictive accuracy purely in aggregate risks masking regime-dependent breakdowns (e.g., strong performance during high-range trends concealing disastrous losses during choppy low-volume ranges). 

Therefore, every prospective opportunity is compartmentalized along **independent price and volume tracks** using strictly pre-decision historical data:

### 8.1 Pre-Decision Invariant & Market Tracks
All metrics are computed strictly using completed historical bars preceding the decision timestamp ($T < T_{\text{decision}}$):

1. **Price Track Dimensions:**
   - **Range Magnitude:** $\text{NATR}_{20} = (\text{ATR}_{20} / P_t) \times 10,000\text{ bps}$.
   - **Directional Efficiency:** $\text{Eff}_{20} = |P_t - P_{t-20}| / \sum_{\tau=1}^{20} \text{TR}_\tau \in [0, 1]$.
   - **Autoregressive Reversion:** $\text{AR}_1 = \text{Corr}(r_t, r_{t-1})$ over 60 bars.
   - **Range Stability:** $\text{CV}_{\text{range}} = \sigma(\text{TR}_{20}) / \mu(\text{TR}_{20})$.
   - **Opportunity Ratio:** $\text{Opp}_{\text{ratio}} = \text{NATR}_{20} / C_{\text{crypto}}$, where $C_{\text{crypto}} = 14.1\text{ bps}$.
2. **Volume Track Dimensions (Strictly Causal):**
   - **Relative Volume ($\text{RVOL}_{20}$):** Ratio of completed candle $t$ volume to trailing completed baseline ($V_t / \text{SMA}_{20}(V_{t-20..t-1})$). Calculated strictly on closed bars; zero look-ahead.
   - **Volume Burst Index ($\text{VBI}$):** Standardized volume shock ($(V_t - \mu_V) / \sigma_V$).
   - **Volume Trend Alignment ($\text{VTA}$):** Correlation between volume expansion and price directional displacement.
   - **Liquidity Depth Ratio ($\text{LDR}$):** Top-of-book displayed depth relative to order size.

### 8.2 Predeclared Compartment Taxonomy & Priority Hierarchy
To eliminate data snooping, compartment classification follows a frozen decision tree:

1. **$\mathcal{C}_4$: Low-Range / Friction-Bound / Volume-Drained:**
   - Criteria: $\text{NATR}_{20} < 160\text{ bps}$ OR $\text{Opp}_{\text{ratio}} < 18\times$ OR $\text{RVOL}_{20} < 0.60$.
   - **Friction-Boundary Derivation:** For crypto where $C_{\text{crypto}} \approx 14.1\text{ bps}$, $\text{Opp}_{\text{ratio}} < 18\times$ requires $\text{NATR}_{20} < 253.8\text{ bps}$. For NSE where $C_{\text{NSE}} \approx 9.0\text{ bps}$, $\text{Opp}_{\text{ratio}} < 18\times$ implies $\text{NATR}_{20} < 162.0\text{ bps}$. Thus $\text{Opp}_{\text{ratio}}$ acts as the market-cost-scaled coverage multiple, while $160\text{ bps}$ serves as the nominal floor.
   - Precedence: Checked first. If price or volume is drained, classified as $\mathcal{C}_4$ regardless of trend.
2. **$\mathcal{C}_1$: High-Range Directional Movers with Active Volume:**
   - Criteria: $\text{NATR}_{20} \ge 220\text{ bps}$ AND $\text{Eff}_{20} \ge 0.14$ AND $\text{AR}_1 \ge -0.10$ AND $\text{RVOL}_{20} \ge 1.00$.
3. **$\mathcal{C}_2$: High-Range Volatile Oscillators with Volume Expansion:**
   - Criteria: $\text{NATR}_{20} \ge 220\text{ bps}$ AND $\text{Eff}_{20} < 0.12$ AND $\text{AR}_1 \le -0.12$ AND $\text{RVOL}_{20} \ge 1.00$.
4. **$\mathcal{C}_3$: Stable Moderate Movers with Normal Participation:**
   - Criteria: $160\text{ bps} \le \text{NATR}_{20} < 220\text{ bps}$ AND $\text{CV}_{\text{range}} \le 0.45$.
5. **$\mathcal{C}_{\text{Unclassified}}$:**
   - Any opportunity with fewer than 20 historical bars or lying in transition boundaries.

### 8.3 The Five Compartment Governance Rules
1. **Pre-Decision Stamp:** Compartment assignment strictly uses pre-decision data ($T < T_{\text{decision}}$); never the eventual trade outcome.
2. **Tri-Level Reporting:** Report Layer 1, Layer 2, and Layer 3 metrics **within each compartment** as well as in aggregate across the entire universe.
3. **Sample Size & CI Transparency:** Each compartment must disclose sample size $N$ and 95% Wilson confidence intervals.
4. **No Selective Presentation:** Strong compartments cannot be cherry-picked to conceal weak compartments.
5. **Zero Retuning Invariant:** Strategy rules and compartment thresholds remain 100% frozen during the prospective window.

---

## 9. The Four Predefined Post-Window Deliverables

At the conclusion of the 30-day window (`2026-11-25T00:00:00Z`), the telemetry engine must emit the following four reports:

1. **Deliverable 1: Market-Data Quality & Feed Integrity Report:**  
   Quote coverage %, staleness percentiles (median, 95th, 99th, max), WebSocket reconnect log, clock drift distribution, and PASS/FAIL status on all five quality gates.
2. **Deliverable 2: Execution Microstructure & Friction Decomposition Report:**  
   Observed spread distribution at decision vs execution timestamps, displayed top-of-book liquidity distribution, and itemized friction totals (fees, spread crossing, slippage, and impact).
3. **Deliverable 3: Prospective Predictive Accuracy & Compartmentalized Performance Report:**  
   Full comparative performance of Policy B and Policy C on the identical prospective stream with zero parameter changes, including Layer 1 predictive accuracy (precision, balanced accuracy, Brier score, Wilson CI), Layer 2 trade quality (expectancy, profit factor, MFE/MAE), and Layer 3 execution realism across all five predeclared compartments and in aggregate.
4. **Deliverable 4: Governance Conformance & Anomaly Audit Manifest:**  
   Startup/shutdown cryptographic hash verification, exhaustive log of any feed drops, protocol deviations, or failed gates. **This report is published regardless of whether the strategy returns are positive, negative, or flat.**

---

## 10. Recommended Operational Sequence for Implementation Readiness

The operational deployment must strictly follow this six-step sequence:

```
[Step 1] Preflight Check (Verify Policy B & C container & genome hashes)
   │
[Step 2] Feed Connectivity Check (Confirm Binance @kline_15m & @bookTicker streams parse cleanly)
   │
[Step 3] Telemetry Invariant Verification (Clock mapping, stale quote drops, zero order authority)
   │
[Step 4] Technical Pilot (2026-10-25T03:00:00Z to 2026-10-26T00:00:00Z: live feed stabilization)
   │
[Step 5] Observation Window Activation (Start registered 30-day observation strictly at 2026-10-26T00:00:00Z)
   │
[Step 6] Window Closure & Quad-Report Publication (Emit all 4 deliverables on 2026-11-25T00:00:00Z)
```

---

## 11. Current Governance State & Locks

| Governance Invariant | Lock Status | Verification Check |
| :--- | :--- | :--- |
| **Historical Optimization** | **CLOSED** | Zero MOGA evolution; zero changes to genomes |
| **Policy Champions** | **FROZEN** | Checksums locked in `crypto_telemetry.rs` |
| **Live Order Authority** | **0.0%** | Zero trading capability; read-only shadow only |
| **Operational State Machine** | **ACTIVE_PILOT** | Technical Pilot active; formal window activation at `2026-10-26T00:00:00Z` |
| **Core Crates (`coralys-*`)** | **UNTOUCHED** | Zero diffs against clean upstream |
| **Automated Test Suite** | **23/23 PASS** | Verified across all 3 research test suites |

### 11.1 Operational State Machine & Transition Rules
The daemon execution state transitions strictly according to the following deterministic rules:
1. `PREFLIGHT_VERIFY`: Verify binary, container, dataset, and champion RuleListGenome SHA-256 hashes.
   - If any checksum fails $\implies$ `ABORTED_PREFLIGHT_FAIL` (zero feed ingestion; zero trade logging).
2. `ACTIVE_PILOT` (`2026-10-25T03:00:00Z` to `2026-10-26T00:00:00Z`): Feed connectivity validation, WebSocket reconnection resilience, and clock offset calibration ($\Delta_{\text{clock}}$). All metrics recorded in pilot buffer; zero formal acceptance trades accrued.
3. `ACTIVE_OBSERVATION` (`2026-10-26T00:00:00Z` to `2026-11-25T00:00:00Z`): Registered 30-day prospective evaluation. Zero retuning; zero policy mutations.
4. `OBSERVATION_TERMINAL_CLOSE` (`2026-11-25T00:00:00Z`): Immediate square-off of open positions, publication of the 4 predefined milestone deliverables, and permanent read-only lock.

