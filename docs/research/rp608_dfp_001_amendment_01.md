# RP-608-DFP-001-AMD-001: Formal Protocol Amendment 01
## Reconciled Interim Checkpoint Mechanics, Quality Gate Harmonization, and Decision Semantics

**Amendment ID:** `RP-608-DFP-001-AMD-001`  
**Parent Protocol:** `RP-608-DFP-001` (7-Day Directional Feasibility Pilot Protocol)  
**Parent Programs:** `RP-608-SHADOW-001B` (Binance BTC/USDT) & `RP-608-NSE-SHADOW-001` (NSE 119 Universe)  
**Date / Timestamp (UTC):** Sunday, 25 October 2026, 07:30:00Z (`2026-10-25T13:00:00+05:30`)  
**Status:** **AUTHORITATIVE PROTOCOL AMENDMENT & VERSIONED RECORD**  
**Governance Invariant:** Frozen champions unmodified; core crates diff = 0; live order authority strictly 0.0%.  

---

## 1. Purpose & Scope of Amendment

This amendment formally reconciles four methodological and operational boundaries identified prior to the `2026-10-26T00:00:00Z` activation of the prospective observation programs:

1. **Window-State Reconciliation:** Explicitly positions the 7-day pilot as a **preregistered interim feasibility checkpoint within the already-active observation window**, rather than an independent precursor that attempts to activate an already-running window.
2. **Quality Gate Harmonization (TG-1 through TG-6):** Aligns the technical failure criteria strictly with the parent registered protocols across six uniquely identified quality gates (`TG-1` through `TG-6`).
3. **Directional Estimand Primacy & Sample-Evaluated Brier Reference:** Designates the fixed-horizon directional prediction (**Estimand B** across all eligible decision bars) as the primary indicator of directional value, and explicitly specifies that the reference Brier score must be calculated directly on the prospective sample outcomes:
   $$\text{BS}_{\text{ref}} = \frac{1}{N} \sum_{i=1}^N (y_i - p_{\text{train}})^2$$
4. **Separation of Win Rate from Expectancy:** Clarifies that a trade win rate below 50% does not imply absence of edge; positive edge is evaluated on Gross Expectancy ($\mathbb{E}[R_{\text{gross}}] > 0.0$) and Profit Factor ($\text{PF}_{\text{gross}} > 1.0$), with Wilson intervals reserved strictly for binomial win proportions.
5. **Exact NSE Denominator & Timestamp Schedule:** Defines Bar 21 decision at 14:30 IST close with 4-bar forward outcome at 15:30 IST session close, strictly distinct from the 15:15 IST execution square-off.
6. **Deterministic Decision Precedence (STOP $\to$ GO $\to$ HOLD):** Precludes overlap between GO and HOLD, defines GO strictly as continuation authorization, and treats the 5-session interim bootstrap as a preliminary feasibility diagnostic (reserving early STOP for technical disqualification or complete gross expectancy extinction).

The original protocol documents (`RP-608-SHADOW-001B` and `RP-608-NSE-SHADOW-001`) remain preserved as immutable historical records. This amendment supersedes conflicting text in `RP-608-DFP-001`.

---

## 2. Reconciled Amendments to RP-608-DFP-001

### 2.1 Amendment 1: Relationship to the Registered Observation Windows
- **State Transition Reality:** Both the registered 30-day Binance observation and the registered 20-session NSE observation formally commence on **Monday, 26 October 2026** (`00:00:00Z` for Binance, `03:45:00Z` / 09:15 IST for NSE).
- **Interim Checkpoint Status:** The 7-day feasibility pilot represents **Cohort 1 (Days 1–7)** within the uninterrupted 30-day/20-session prospective stream.
- **Reporting Cohort Segregation:**
  - *Feasibility Pilot Cohort (Days 1–7):* Oct 26 to Nov 01, 2026 (672 intervals for BTC/USDT; 5 qualifying NSE sessions = 14,875 total asset-bars, 12,495 eligible directional intervals). Evaluated at the Day 8 interim checkpoint (02 Nov 2026).
  - *Full Registered Prospective Cohort (Days 1–30 / Sessions 1–20):* Oct 26 to Nov 25, 2026 for Binance Spot; Oct 26 until the close of the 20th qualifying session for NSE.
- **Continuation Mechanics:** A `GO` verdict at the Day 8 review authorizes the uninterrupted continuation of the ongoing registered observation. A `HOLD` verdict schedules an amended Day 14 interim continuation review within the active window without resetting clocks or cherry-picking samples.

---

### 2.2 Amendment 2: Technical Quality Gates TG-1 through TG-6
To prevent passing on looser criteria that violate parent protocol requirements, technical feasibility requires 100% PASS across all six uniquely identified gates:

- **`TG-1` (Quote Coverage Gate):** Matched quotes $\ge 99.50\%$ of evaluated bars (missing feed intervals $< 0.50\%$).
- **`TG-2` (Quote Freshness Gate):** Stale quotes ($> 500\text{ ms}$) $\le 0.50\%$ of matched quotes.
- **`TG-3` (Clock Synchronization Gate):** Measured drift $|\Delta_{\text{clock}}| \le 250\text{ ms}$ against exchange UTC anchor.
- **`TG-4` (Feed Continuity Gate):** Maximum consecutive missing bars $< 2$ (Crypto) / $< 1$ (NSE).
- **`TG-5` (Sequence & Timestamp Monotonicity Gate):**
  - **Zero Sequence ID Inversions:** Exactly **zero (0.00%) sequence ID inversions** tolerated ($\Delta \text{Seq} \le 0 \implies \text{STOP}$).
  - **Zero Exchange Event Timestamp Inversions:** Exactly **zero (0.00%) out-of-order quote timestamps** from the exchange matching engine tolerated.
  - **Decoupling from Ingestion Jitter:** The previously stated 0.10% allowance is eliminated from ordering rules. Multi-socket network receipt jitter must be absorbed by deterministic reordering buffers before matching; no out-of-order quotes ever enter the shadow telemetry engine.
- **`TG-6` (Comprehensive Cost Ledger & Incremental Friction Gate):**
  - **Total Realized Cost Ledger:** Every executed trade leg accounts for all four distinct friction components in quote currency (USDT / INR):
    $$\text{Friction}_{\text{total}} = \text{Fees}_{\text{taker}} + \text{Half-Spread Crossing} + \text{Slippage}_{\text{exogenous}} + \text{Market Impact}$$
  - **Incremental Friction Identity:** Incremental friction drag accounts strictly for the execution wedge beyond baseline:
    $$\Delta_{\text{incremental}} \equiv \text{Friction}_{\text{total}} - \text{Friction}_{\text{baseline}} = \text{Half-Spread Crossing} + \text{Market Impact}$$
    where $\text{Friction}_{\text{baseline}} = \text{Fees}_{\text{taker}} + \text{Slippage}_{\text{exogenous}}$.
  - **Exact Arithmetic Reconciliation:** The residual accounting error must satisfy $| \Delta_{\text{incremental}} - (\text{Spread} + \text{Impact}) | < 10^{-4}$ in currency units across all fills. Exchange fees and exogenous slippage are independently reconciled and logged on every `FillReconciliationRecord`.

---

### 2.3 Amendment 3: Directional Estimand Primacy & Sample-Evaluated Brier Reference
1. **Primacy of Estimand B (Fixed-Horizon Direction):**
   - **Estimand B** (fixed 4-bar forward return direction across all eligible 15-minute decision intervals: $D_{s, k} = \mathbb{I}(P_{\text{close}}(k+4) > P_{\text{close}}(k))$) is the **primary directional estimand**.
   - Trade Win Rate (**Estimand A**) and Net P&L are designated as **supporting execution measures**, preventing conflation of trade frequency with directional predictive value.

2. **Sample-Evaluated Brier Reference Score:**
   - Rather than assuming expected baseline $p_{\text{train}}(1 - p_{\text{train}})$, the reference Brier score must be calculated directly on the prospective sample:
     $$\text{BS}_{\text{ref}} = \frac{1}{N} \sum_{i=1}^N (y_i - p_{\text{train}})^2$$
   - The Brier Skill Score is calculated as $\text{BSS} = 1 - (\text{BS} / \text{BS}_{\text{ref}})$, evaluating both terms on the **identical eligible observations**.
   - Frozen Training Prevalence Baselines ($p_{\text{train}}$):
     - Estimand B: $\bar{y}_{\text{train}} = 0.508$ (Crypto), $0.512$ (NSE).
     - Estimand A: Policy B = 0.62, Policy C = 0.65, NSE = 0.58.

3. **Separation of Win Rate from Positive Expectancy:**
   - A trade win rate below 50% does **not** imply negative edge, as average gains may exceed average losses.
   - Positive edge is evaluated strictly via Gross Expectancy ($\mathbb{E}[R_{\text{gross}}] > 0.0$) and Gross Profit Factor ($\text{PF}_{\text{gross}} > 1.0$).
   - Wilson score intervals are reserved strictly for the binomial win proportion ($p_{\text{win}}$) and must **not** be used for Balanced Accuracy or positive expectancy falsification.

4. **Preliminary Nature of 5-Session Bootstrap:**
   - A 5-session bootstrap (and 7 daily blocks for Crypto) has limited information on between-session variability.
   - Interim bootstrap results are preliminary feasibility diagnostics. Early `STOP` is never triggered solely by an interim confidence interval crossing 50%; formal statistical null falsification is reserved for the full completed 20-session / 30-day window.

---

### 2.4 Amendment 4: NSE Completed Bar Indexing Convention & Intraday Schedule
1. **Session Schedule & Bar Partitioning:** 09:15:00 to 15:30:00 IST (375 trading minutes = exactly 25 completed 15-minute bars per session).
2. **1-Based Completed Bar Indexing Convention:**
   - The 25 bars are indexed $k \in \{1, 2, \dots, 25\}$.
   - Interval $k$: $[T_{\text{open}}(k), T_{\text{close}}(k)) = [09:15:00 + (k-1)\times 15\text{m}, \; 09:15:00 + k\times 15\text{m}]$.
   - Every bar $k$ is causally indexed and emitted at its **closing timestamp** $T_{\text{close}}(k) = 09:15:00 + k\times 15\text{m}$.
   - Decision timestamp $T_{\text{decision}}(k) \equiv T_{\text{close}}(k)$ with reference price $P_{\text{ref}}(k) \equiv P_{\text{close}}(k)$.
   - Outcome timestamp for the 4-bar forward horizon is $T_{\text{outcome}}(k) \equiv T_{\text{close}}(k+4) = T_{\text{close}}(k) + 60\text{ minutes}$, with outcome price $P_{\text{outcome}}(k) \equiv P_{\text{close}}(k+4)$.
3. **Quote Coverage Denominator:** All 25 session bars $\times$ 5 sessions $\times$ 119 symbols = **14,875 asset-bars**. Monitored across all 14,875 bars for `TG-1` and `TG-2`.
4. **Intraday Boundary Invariant for Estimand B:** Forward 4-bar labels **must NOT cross an overnight session boundary** ($T_{\text{outcome}}(k) \le 15:30:00\text{ IST}$):
   $$k + 4 \le 25 \iff k \le 21$$
5. **Exact Bar Schedule for Estimand B:**
   - **Bars 1–21 (Eligible):** Bar 1 ($[09:15, 09:30)$, close $09:30$, outcome at Bar 5 close $10:30$) through Bar 21 ($[14:15, 14:30)$, close $14:30$, outcome at Bar 25 close $15:30:00$ session close). Exactly **21 eligible decision bars per session**.
   - **Bars 22–25 (Boundary-Excluded):** Bars 22 (14:45 close), 23 (15:00 close), 24 (15:15 close), and 25 (15:30 close) have $k+4 > 25$ crossing into subsequent calendar sessions and are recorded as `IntradayBoundaryExcluded`.
   - **Estimand B Denominator:** 5 sessions $\times$ 21 bars $\times$ 119 symbols = **exactly 12,495 eligible decision intervals**.
6. **Estimand A (Trade Lifecycle) Timestamps:**
   - Entry Window: Bar 1 (09:15 open) up to Bar 20 (14:00 open). Last permitted entry decision: 14:00 IST.
   - Mandatory Terminal Square-Off: Bar 24 (15:15 IST) causally squares off all open positions at prevailing bid/ask prior to 15:30 close. *(Strictly distinct from Estimand B's 15:30 session close outcome).*
   - Bar 25 (15:15–15:30 IST): Monitoring only (0 entries, 0 positions held).

---

### 2.5 Amendment 5: Deterministic Decision Precedence (STOP $\to$ GO $\to$ HOLD)

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

#### Detailed Precedence Rules

1. **Step 1 (Evaluated First) — `STOP` (Conclude & Archive Prospective Run Early):**
   - *Technical Disqualification:* Quote coverage $< 99.50\%$ (feed drops $> 0.50\%$), persistent clock desync $> 250\text{ ms}$, or unrecoverable out-of-order execution drops.
   - *Gross Expectancy Extinction:* Gross Expectancy $\le 0.0$ across all price-volume compartments ($\mathcal{C}_1$ through $\mathcal{C}_4$).
   - *Invariant:* Brief sequence of negative net P&L with positive gross expectancy does **not** trigger STOP. Zero MOGA retuning or parameter optimization is permitted on pilot data.

2. **Step 2 (Evaluated Second) — `GO` (Continue Registered Observation to Window Completion):**
   - *Prerequisites (All Must Be Satisfied):*
     - Technical Gates: **100% PASS** on all 6 gates (`TG-1` through `TG-6`).
     - Estimand B Directional Signal: Point-estimate Balanced Accuracy $\ge 51.0\%$ and sample-evaluated empirical $\text{BSS} > 0.0$ against reference baseline. *(Authorizes continuation; does not declare strategy confirmed or profitable).*
     - Estimand A Trade Quality: Gross Expectancy $> 0.0$ and Gross $\text{PF} \ge 1.05$.
     - **Numerical Economic Runway Test:**
       - In top compartments ($\mathcal{C}_1, \mathcal{C}_2$): $\text{Expectancy}_{\text{gross}} / \text{Friction}_{\text{round\_trip}} \ge 1.0$ (gross edge covers $\ge 100\%$ of realized friction).
       - Aggregate friction drag runaway: $\text{Friction}_{\text{total}} / \text{Turnover}_{\text{notional}} \le 18.0\text{ bps}$ (Crypto) / $\le 12.0\text{ bps}$ (NSE).
   - *Action:* Authorize uninterrupted continuation of the ongoing registered 30-day (Crypto) and 20-session (NSE) observation window.

3. **Step 3 (Default Fallback) — `HOLD` (Interim Continuation Review at Day 14):**
   - *Trigger:* Technical gates pass, STOP is not triggered, but one or more continuation criteria are not fully satisfied (e.g. sample size $N < 30$, trade count too sparse, or realized spreads were transiently elevated, causing the top-compartment gross/friction ratio to fall below 1.0x).
   - *Action:* Continue observation under the registered window, establishing an amended interim review on Day 14 (08 Nov 2026 / 10 NSE sessions). **Does not reset clocks or create a fresh sample.**

---

## 3. Preserved Cryptographic Digests

```
Parent Protocols (Preserved Baseline with Clarified Brier Ref & Indexing):
docs/research/rp608_shadow_001_prospective_observation_protocol.md
SHA-256: f1b07c49ccc8f4d64bb0df64c4a6654a10503e74fcc980c51cb3f740c68fd018

docs/research/rp608_nse_shadow_001_prospective_protocol.md
SHA-256: 8f85be7aa3d5731087ecdd5673ec823ccb8e1daf9e061c01fb81e525cf3647d4

Feasibility Pilot Specification:
docs/research/rp608_dfp_001_directional_feasibility_pilot_protocol.md
Pre-Amendment Historical Digest: c1dda4b5d3ee0f4a0ddc731885adb9ad9c09de2d27ba9210d00a94577ec71fad
Authoritative Current Digest:   e8bca8ad9194ff058cc4eb4315f20c290d6465fce28fcfe0b675b6b133d494c3
```

---

## 4. Operational Sign-Off & Checkpoint Schedule

This amendment is approved and incorporated into the RP-608 research framework.  
Prospective observation activates on **Monday, 26 October 2026, 00:00:00Z** (Binance) and **03:45:00Z** (NSE), with the first preregistered interim review taking place at the **Day 8 checkpoint on Monday, 02 November 2026, 09:00:00Z**.
