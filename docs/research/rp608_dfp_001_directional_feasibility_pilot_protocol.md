# RP-608-DFP-001: 7-Day Directional Feasibility Pilot Protocol
## Dual-Collector Prospective Microstructure & Predictive Signal Verification

**Protocol ID:** `RP-608-DFP-001`  
**Series:** Prospective Predictive Validation (Directional Feasibility Stage)  
**Parent Framework:** `RP-608-SHADOW-001B` (Binance) & `RP-608-NSE-SHADOW-001` (NSE 119)  
**Activation Date (UTC):** Monday, 26 October 2026, 00:00:00Z  
**Checkpoint Date (UTC):** Sunday, 01 November 2026, 23:59:59Z (7 Calendar Days / 5 NSE Sessions)  
**Decision Gate (UTC):** Monday, 02 November 2026, 09:00:00Z (Stage 4 Go / Hold / Stop)  
**Target Hardware:** Raspberry Pi 5 (8GB RAM, Dedicated ARM64 Host)  
**Live Order Authority:** Strictly **0.0%** (Read-Only Shadow Feeds; Zero Order Submission API Keys)  
**Core Crate Modification:** Strictly **0.0%** (`coralys-moga`, `coralys-decision`, `coralys-ecology` diff = 0)  
**Status:** **ACTIVE PILOT SPECIFICATION & FROZEN GATES**  

---

## 1. Executive Summary & Strategic Rationale

Committing blindly to a 30-calendar-day (or 20-session) prospective window without intermediate empirical checkpoints introduces substantial operational risk. Rather than waiting a month to discover basic feed flaws or structural friction limits, `RP-608-DFP-001` (as amended by `RP-608-DFP-001-AMD-001`) establishes a **preregistered 7-day interim feasibility checkpoint within the active registered observation window**.

Both the registered 30-day Binance observation and the registered 20-session NSE observation formally commence on **Monday, 26 October 2026**. The feasibility pilot represents **Cohort 1 (Days 1–7)** within the ongoing prospective stream. The explicit objective of this interim checkpoint is **directional verification**, not premature statistical claims of profitability. It evaluates whether the system is technically sound, whether frozen signals contain genuine directional information on live feeds, and whether observed execution friction leaves a viable economic runway.

```
       RP-608 PREREGISTERED INTERIM CHECKPOINT TIMELINE
───────────────────────────────────────────────────────────────────────────
 [Day 0: 25 Oct]   Stage 1: Deployment, Hash Locks & Hardware Audits
 [Days 1-3: 26-28] Stage 2: Technical & Operational Feasibility (Gates 1-5)
 [Days 4-7: 29-01] Stage 3: Preliminary Directional Evidence (Layers 1-3)
 [Day 8: 02 Nov]   Stage 4: Formal Go / Hold / Stop Interim Decision Review
───────────────────────────────────────────────────────────────────────────
```

---

## 2. The Five Priority Inquiries & Evidence Standards

| Inquiry Domain | Primary Research Question | Registered Evidence Standard | Threshold Gate |
| :--- | :--- | :--- | :--- |
| **1. Operational Integrity** | Is the application capturing and accounting correctly? | Continuous quote coverage $\ge 99.5\%$, stale quotes $\le 0.5\%$, $|\Delta_{\text{clock}}| \le 250\text{ ms}$, consecutive missing bars $<2$ (Crypto) / $<1$ (NSE), zero out-of-order executions, fill-by-fill arithmetic reconciled ($<10^{-4}$). | **Gate 1–5 PASS** *(Mandatory Technical Prerequisite)* |
| **2. Directional Information** | Do frozen signals predict forward price direction? | **Estimand B** (fixed 4-bar horizon across all decision bars): confusion matrix ($TP, FP, TN, FN$), Balanced Accuracy $> 50.0\%$, and empirical Brier Skill Score against training baselines ($\text{BSS} > 0.0$ reported with Wilson CIs and $N_{\text{eff}}$). | **Directional Information Plausible** *(Primary Research Focus)* |
| **3. Triggered Trade Quality** | Do entered trades exhibit positive excursion mechanics? | **Estimand A** (entered trade lifecycle): Gross expectancy $> 0.0$, gross profit factor ($\text{PF}_{\text{gross}} > 1.0$), and Edge Ratio ($\text{MFE} / \text{MAE} > 1.0$). *(Supporting execution measure)* | **Gross Expectancy $> 0.0$** *(Quality Gate)* |
| **4. Friction Survivability** | Does the observed gross edge survive real friction? | Itemized realized fees, bid/ask spread crossing, slippage, and statutory charges (STT, GST, SEBI). Net expectancy vs friction drag in top compartments ($\mathcal{C}_1, \mathcal{C}_2$). | **Plausible Economic Runway** *(Economic Gate)* |
| **5. Statistical Credibility** | Is the sample credible enough to justify full 30 days? | Autocorrelation-adjusted $N_{\text{eff}}$, session-cluster degrees of freedom ($S = 5$ qualifying NSE sessions), Wilson 95% CIs, and single-trade sensitivity. | **No Statistical Falsification** *(Wilson Upper $> 50\%$)* |


---

## 3. Four-Stage Pilot Execution Architecture

### 3.1 Stage 1: Deployment & Integrity (Day 0 — Sunday, 25 Oct 2026)
- **Host Architecture:** Deployed on dedicated Raspberry Pi 5 (8GB RAM, quad-core Cortex-A76 @ 2.4GHz, NVMe SSD storage).
- **Process Isolation:** Two independent daemon systemd units:
  - `rp608-crypto-shadow.service`: Binance Spot BTC/USDT read-only stream.
  - `rp608-nse-shadow.service`: NSE 119-symbol liquid equity universe stream.
- **Ledger Path Isolation:**
  - Binance: `/var/log/chronosentiment/crypto/`
  - NSE: `/var/log/chronosentiment/nse/`
- **Zero-Authority Attestation:** `BINANCE_API_SECRET` and broker trading keys explicitly absent; order-submission endpoints compiled out.
- **Preflight Checksum Gate:** Startup verification of Policy B, Policy C, 2024 dataset, and protocol SHA-256 hashes.

### 3.2 Stage 2: Technical & Operational Feasibility (Days 1–3 — 26 to 28 Oct 2026)
- **Clock Drift & Latency Monitoring:**
  $$\Delta_{\text{clock}} \equiv T_{\text{local\_wall}} - T_{\text{exchange}} \quad (\text{Target: } |\Delta_{\text{clock}}| \le 250\text{ ms})$$
  $$\Delta t = T_{\text{local\_recv}} - T_{\text{quote\_event}} - \Delta_{\text{clock}} \quad (\text{Target: } \Delta t + U \le 500\text{ ms})$$
- **Feed Coverage Invariant:** Quote coverage $\ge 99.5\%$; stale quotes ($>500\text{ ms}$) $\le 0.5\%$.
- **Causal Execution Accounting:** Signals emitted at $T_{\text{decision}} = \text{Open}(t)$ matched to contemporaneous L1 quotes within 5,000 ms; quotes arriving after $T_{\text{decision}} + 5{,}000\text{ ms}$ registered as `ExecutionQuoteTimeout` drops (never retroactively matched).
- **Daemon Resilience:** Auto-reconnect on WebSocket disconnects with zero state corruption.

### 3.3 Stage 3: Preliminary Directional Evidence (Days 4–7 — 29 Oct to 01 Nov 2026)
- **Continuous 24/7 Crypto Accumulation:** 7 days $\times$ 96 bars/day = 672 completed 15-minute intervals for BTC/USDT.
- **5 Qualifying NSE Trading Sessions & Denominator Physics:**
  - Session Timings: 09:15 to 15:30 IST (375 trading minutes = exactly 25 completed 15-minute bars per session).
  - Total Session Bars: 5 sessions $\times$ 25 bars $\times$ 119 symbols = **14,875 asset-bars**. This serves strictly as the denominator for **quote coverage and data feed integrity**.
  - **Completed Bar Indexing Convention & Mathematical Rigor:**
    - The NSE equity trading session $[09:15:00, 15:30:00\text{ IST}]$ is partitioned into $K = 25$ non-overlapping 15-minute intervals, indexed 1-based by $k \in \{1, 2, \dots, 25\}$.
    - **Interval Boundaries:** Each Bar $k$ covers the interval:
      $$\text{Interval}(k) = [T_{\text{open}}(k), T_{\text{close}}(k)) = [09:15:00 + (k-1)\times 15\text{m}, \; 09:15:00 + k\times 15\text{m}]$$
    - **Emitted at Bar Close:** Every bar $k$ is causally indexed and emitted upon completion at its **closing timestamp** $T_{\text{close}}(k) = 09:15:00 + k \times 15\text{m}$.
    - **Decision Timestamp:** $T_{\text{decision}}(k) \equiv T_{\text{close}}(k)$, with reference price $P_{\text{ref}}(k) \equiv P_{\text{close}}(k)$.
    - **Fixed 4-Bar Forward Horizon:** The prospective evaluation horizon is defined by the close of the 4th subsequent bar:
      $$T_{\text{outcome}}(k) \equiv T_{\text{close}}(k+4) = T_{\text{close}}(k) + 60\text{ minutes}$$
      with outcome price $P_{\text{outcome}}(k) \equiv P_{\text{close}}(k+4)$.
    - **Same-Session Horizon Invariant (Estimand B):** An interval $k$ is eligible for Estimand B if and only if its 4-bar forward endpoint does **NOT** cross an overnight session boundary ($T_{\text{outcome}}(k) \le 15:30:00\text{ IST}$), because overnight holding incurs structural gap risk not modeled in the intraday strategy:
      $$k + 4 \le 25 \iff k \le 21$$
    - **Session Bar Eligibility Breakdown:**
      - **Bars 1–21 (Eligible):** Bar 1 ($[09:15, 09:30)$, $T_{\text{close}} = 09:30$, outcome at Bar 5 close $10:30$) through Bar 21 ($[14:15, 14:30)$, $T_{\text{close}} = 14:30$, outcome at Bar 25 close $15:30:00$ official market close). Exactly **21 eligible decision bars per session**.
      - **Bars 22–25 (Boundary-Excluded):** Bars 22 ($14:45$ close), 23 ($15:00$ close), 24 ($15:15$ close), and 25 ($15:30$ close) have forward endpoints crossing into subsequent calendar dates ($k+4 \in \{26, 27, 28, 29\} > 25$) and are recorded as `IntradayBoundaryExcluded` for Estimand B.
  - **Total Eligible Directional Intervals:** 5 sessions $\times$ 21 bars $\times$ 119 symbols = **exactly 12,495 eligible decision intervals for Estimand B**.
  - **Eligible Timestamps for Estimand A (Trade Execution Lifecycle):**
    - Entry Decision Window: Bar 1 (09:15 open) up to Bar 20 (14:00 open). Last permitted new entry decision: 14:00 IST.
    - Mandatory Square-Off: Bar 24 (15:15 IST) causally closes all remaining open positions at prevailing market bid/ask prior to 15:30 IST close. *(This execution square-off at 15:15 is strictly distinct from Estimand B's 15:30 session close outcome).*
    - Post-Square-Off Interval: Bar 25 (15:15–15:30 IST) is strictly discovery / monitoring only (0 new entries, 0 positions held).

- **Strict Segregation of Dual Estimands & Scoring Rules:**
  1. **Estimand B: Directional Opportunity Prediction (Primary Research Inquiry):**
     - Target: $D_{s, k} = \mathbb{I}(P_{\text{close}}(k+4) > P_{\text{close}}(k))$ across all 12,495 eligible intervals for NSE and 668 eligible intervals for Crypto.
     - Ex-Ante Probability Forecasts (recorded at $T_{\text{decision}} < T_{\text{outcome}}$, summing to 1.00):
       - Policy B Champion (Crypto): EnterLong $\implies p(\text{Up}) = 0.62, p(\text{Not Up}) = 0.38$; StandAside/Hold $\implies p(\text{Up}) = 0.35, p(\text{Not Up}) = 0.65$.
       - Policy C Champion (Crypto): EnterLong $\implies p(\text{Up}) = 0.65, p(\text{Not Up}) = 0.35$; StandAside/Hold $\implies p(\text{Up}) = 0.35, p(\text{Not Up}) = 0.65$.
       - NSE Strategy: EnterLong $\implies p(\text{Up}) = 0.58, p(\text{Not Up}) = 0.42$; StandAside/Hold $\implies p(\text{Up}) = 0.38, p(\text{Not Up}) = 0.62$.
     - **Direct Sample-Evaluated Reference Benchmark:**
       $$\text{BS}_{\text{ref}} = \frac{1}{N} \sum_{i=1}^N (y_i - p_{\text{train}})^2$$
       where $y_i \in \{0, 1\}$ are the realized prospective outcomes on the identical eligible sample, and $p_{\text{train}}$ is the constant frozen training prevalence ($0.508$ for BTC, $0.512$ for NSE).
     - **Brier Skill Score:** $\text{BSS} = 1 - (\text{BS} / \text{BS}_{\text{ref}})$. Evaluated on the identical observations for both terms.
     - **Preliminary Nature of 5-Session Bootstrap:** A 5-session bootstrap (and 7 daily blocks for Crypto) has limited degrees of freedom to capture between-session variance. Interim bootstrap bounds are preliminary diagnostics; formal null falsification is reserved for the completed 20-session / 30-day registered sample. Wilson intervals are **strictly excluded** from Balanced Accuracy.
  2. **Estimand A: Trade-Outcome Prediction (Supporting Execution Measure):**
     - Target: $Y_{\text{trade}, i} = \mathbb{I}(R_{i, \text{gross}} > 0.00)$ for entered trades only ($i \in \{1, \dots, N_{\text{entered}}\}$).
     - Direct Sample-Evaluated Reference: $\text{BS}_{\text{ref}} = \frac{1}{N} \sum_{i=1}^N (y_i - p_{\text{train}})^2$, with $p_{\text{train}} = 0.62$ (Policy B), $0.65$ (Policy C), $0.58$ (NSE).
     - **Separation of Win Rate from Expectancy:** A trade win rate below 50% does **not** imply negative edge, as average gains may exceed average losses. Positive edge is evaluated strictly via Gross Expectancy ($\mathbb{E}[R_{\text{gross}}] > 0.0$) and Gross Profit Factor ($\text{PF}_{\text{gross}} > 1.0$), with return-distribution bootstrap uncertainty. Wilson intervals are used solely to describe uncertainty around the binomial win proportion ($p_{\text{win}}$).

### 3.4 Stage 4: Formal Go / Hold / Stop Decision Review (Day 8 — Monday, 02 Nov 2026)
On Day 8 (09:00 UTC / 14:30 IST), before the start of the second observation week, the team reviews the 7-day pilot scorecard against the predefined decision matrix.

---

## 4. Predefined Stage 4 Decision Matrix (Go / Hold / Stop)

### 4.1 The Six Technical Quality Gates (TG-1 through TG-6)
Technical feasibility requires 100% PASS across all six uniquely identified gates:
- **`TG-1` (Quote Coverage Gate):** Matched quotes $\ge 99.50\%$ of evaluated bars (missing feed intervals $< 0.50\%$).
- **`TG-2` (Quote Freshness Gate):** Stale quotes ($> 500\text{ ms}$) $\le 0.50\%$ of matched quotes.
- **`TG-3` (Clock Synchronization Gate):** Measured drift $|\Delta_{\text{clock}}| \le 250\text{ ms}$ against exchange UTC anchor.
- **`TG-4` (Feed Continuity Gate):** Maximum consecutive missing bars $< 2$ (Crypto) / $< 1$ (NSE).
- **`TG-5` (Sequence & Timestamp Monotonicity Gate):**
  - **Zero Sequence ID Inversions:** Exactly **zero sequence ID inversions (0.00% tolerated)** ($\Delta \text{Seq} \le 0 \implies \text{STOP}$).
  - **Zero Exchange Event Timestamp Inversions:** Exactly **zero (0.00%) out-of-order quote timestamps** from the exchange matching engine tolerated.
  - **Decoupling from Ingestion Jitter:** The previously stated 0.10% allowance is eliminated from ordering rules. Multi-socket network receipt jitter must be absorbed by deterministic reordering buffers before matching; no out-of-order quotes ever enter the shadow telemetry engine.
- **`TG-6` (Comprehensive Cost Ledger & Incremental Friction Gate):**
  - **Total Realized Cost Ledger:** Every executed trade leg accounts for all four distinct friction components in quote currency (USDT / INR):
    $$\text{Friction}_{\text{total}} = \text{Fees}_{\text{taker}} + \text{Half-Spread Crossing} + \text{Slippage}_{\text{exogenous}} + \text{Market Impact}$$
  - **Incremental Friction Identity:** Incremental friction drag accounts strictly for the execution wedge beyond baseline:
    $$\Delta_{\text{incremental}} \equiv \text{Friction}_{\text{total}} - \text{Friction}_{\text{baseline}} = \text{Half-Spread Crossing} + \text{Market Impact}$$
    where $\text{Friction}_{\text{baseline}} = \text{Fees}_{\text{taker}} + \text{Slippage}_{\text{exogenous}}$.
  - **Exact Arithmetic Reconciliation:** The residual accounting error must satisfy $| \Delta_{\text{incremental}} - (\text{Spread} + \text{Impact}) | < 10^{-4}$ in currency units across all fills. Exchange fees and exogenous slippage are independently reconciled and logged on every `FillReconciliationRecord`.

### 4.2 Deterministic Decision Precedence Rules
To eliminate ambiguity, decision rules follow a strict **deterministic precedence order** (as amended by `RP-608-DFP-001-AMD-001`):

```
                        DETERMINISTIC DECISION PRECEDENCE
                                        │
           ┌────────────────────────────┴────────────────────────────┐
           ▼                                                         ▼
[Rule 1] Did Technical Disqualification                     Are All 6 Technical Gates
         or Extinction Trigger STOP?                        (TG-1 to TG-6) 100% Passed?
         (Quote Coverage < 99.5%,                                    │
          Clock Desync > 250ms, or                                   ▼
          Gross Expectancy <= 0.0 across all C1-C4)         [Rule 2] Are ALL Continuation
                         │                                           Criteria Satisfied?
                         ▼                                           - Balanced Accuracy >= 51.0%
                      [STOP]                                         - Sample BSS > 0.0
         (Halt Prospective Stream;                                   - Gross Expectancy > 0.0
          Archive Forensic Closure)                                  - Gross PF >= 1.05
                                                                     - Top Compartment Ratio >= 1.0x
                                                                     - Net Drag <= 18bps (Crypto) / 12bps (NSE)
                                                                             │
                                                            ┌────────────────┴────────────────┐
                                                            ▼                                 ▼
                                                          [GO]                              [HOLD]
                                                (Authorize Continuation           (Interim Continuation Review
                                                 of Registered Window              Maintained; Schedule Day 14
                                                 to Completion)                    Check without resetting clocks)
```

### Criteria Table & Deterministic Precedence Rules

| Precedence Step | Verdict | Technical Standard | Predictive Standard (Estimand B) | Economic Standard (Estimand A & Friction) | Operational Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Step 1 (Evaluated First)** | **`STOP` (Conclude & Archive Prospective Run Early)** | **FAIL** on any Technical Gate `TG-1` through `TG-6` (feed drops $> 0.5\%$, quote coverage $< 99.5\%$, persistent clock desync $> 250\text{ ms}$, or unrecoverable out-of-order execution drops). | Gross Expectancy $\le 0.0$ across all price-volume compartments ($\mathcal{C}_1$ through $\mathcal{C}_4$). *(Note: Brief negative net P&L alone does NOT trigger STOP; 5-session bootstrap crossing 50% does NOT trigger STOP early).* | Catastrophic failure across all execution tracks. | Halt prospective collection early. Publish forensic closure report. **Strictly zero MOGA retuning or parameter optimization on pilot data.** |
| **Step 2 (Evaluated Second)** | **`GO` (Continue Registered Observation to Window Completion)** | **100% PASS** on all 6 Technical Quality Gates (`TG-1` through `TG-6`). | **`PASS`:** Estimand B point-estimate Balanced Accuracy $\ge 51.0\%$ and sample-evaluated empirical $\text{BSS} > 0.0$ against reference baseline. *(Authorizes continuation; does not declare strategy confirmed or profitable).* | **Numerical Economic Runway Test:**<br>1. In top compartments ($\mathcal{C}_1, \mathcal{C}_2$): $\text{Expectancy}_{\text{gross}} / \text{Friction}_{\text{round\_trip}} \ge 1.0$ (gross edge covers $\ge 100\%$ of realized friction).<br>2. Gross Expectancy $> 0.0$ and Gross $\text{PF} \ge 1.05$.<br>3. Total friction drag $\le 18.0\text{ bps}$ (Crypto) / $\le 12.0\text{ bps}$ (NSE). | Authorize uninterrupted continuation of the ongoing registered 30-day (Crypto) and 20-session (NSE) observation window. |
| **Step 3 (Default Fallback)** | **`HOLD` (Interim Continuation Review at Day 14)** | **PASS** on all 6 Technical Quality Gates. | **`INCONCLUSIVE`:** Point estimate shows positive skill ($\text{BSS} > 0.0$), but trade sample size $N < 30$, or qualifying sessions $S < 5$. | Gross Expectancy positive, but realized spreads were transiently elevated, causing top-compartment gross/friction ratio to fall below 1.0x. | Continue observation under the registered window, establishing an amended interim review on Day 14 (08 Nov 2026 / 10 NSE sessions). **Does not reset clocks or alter the sample.** |




---

## 5. Raspberry Pi 5 Host Resource Budget & Sizing

The Pi 5 host resources are partitioned to guarantee zero contention between the two collectors:

| System Metric | Binance Crypto Shadow Daemon | NSE 119-Symbol Shadow Daemon | Host Operating System & Headroom | Total Allocation |
| :--- | :--- | :--- | :--- | :--- |
| **CPU Core Affinity** | Cores 0 & 1 (Dedicated) | Cores 2 & 3 (Dedicated) | All cores (dynamic priority) | 4 Cores @ 2.4 GHz |
| **RAM Utilization** | ~140 MB (Working set + ring buffer) | ~320 MB (119 symbol state machines) | ~800 MB (OS, caching, systemd) | ~1.26 GB / 8.00 GB (<16%) |
| **Storage IOPS** | Append-only JSONL; ~5 MB/day | Append-only JSONL; ~25 MB/day | NVMe SSD (PCIe Gen 2/3) | < 30 MB/day |
| **Network Bandwidth** | 1 L1 top-of-book stream (~15 KB/s) | 119 ticker stream (~65 KB/s) | Fiber connection (~80 KB/s total) | Negligible (<1 Mbps) |
| **Failure Recovery** | `Restart=always`, `RestartSec=5s` | `Restart=always`, `RestartSec=5s` | Watchdog auto-alerting | Zero state loss |

---

## 6. Preregistered Pilot Governance Invariants

1. **Strictly Zero Retuning:** If the 7-day pilot shows flat or negative returns, it is strictly forbidden to tweak MOGA parameters, adjust stop-losses, prune the 119-symbol universe, or modify compartment thresholds.
2. **Causal Transparency:** All decisions, probability forecasts, and L1 quotes must be logged with millisecond timestamps and monotonically increasing sequence IDs before hypothetical outcomes are resolved.
3. **Dual-Ledger Isolation:** Crypto and NSE data pipelines share zero variables, memory pools, or file paths.
4. **Permanent Audit Trail:** The 7-day pilot log files are preserved with cryptographic SHA-256 hashes and appended to the repository archive.
