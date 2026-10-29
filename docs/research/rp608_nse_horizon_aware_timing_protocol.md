# RP-608 — Horizon-Aware Entry & Session Timing Protocol (RP-608-TIMING-EXP-001)

**Document ID:** `DOC-RP608-TIMING-001`  
**Protocol ID:** `RP-608-TIMING-EXP-001`  
**Target:** Horizon-Aware Temporal Eligibility, Pre-Closeout Excursions, and Entry-Window Performance Decomposition  
**Applicability:** NSE 119-Universe Prospective Counterfactual Replay  
**Governance Invariant:** Strictly Read-Only (0.0% Order Authority) | Zero Mutation of Active Prospective Observer Baseline  

---

## 1. Executive Summary & Problem Formulation

The canonical RP-608 NSE shadow observer enforces an operational entry restriction:
$$\text{If } \text{bar\_idx} > 20 \implies \text{Reject entry with } \texttt{INTRADAY\_BOUNDARY\_EXCLUDED}$$

While this rule ensures that entered trades have at least 75 minutes of holding time prior to the 15:15 IST mandatory square-off, **14:00 IST is an ex-ante strategy hypothesis rather than an empirical optimum**. NSE equity trading continues with institutional liquidity until 15:30 IST.

This protocol establishes the formal research framework to test **Horizon-Aware Entry Eligibility**:
Replacing a fixed, blanket clock cutoff with a dynamic eligibility test based on the trade's intended holding horizon $H$:
$$T_{\text{available}} = 15:15:00\text{ IST} - T_{\text{decision}}$$
$$\text{Eligible}(H) \iff H \le T_{\text{available}}$$

---

## 2. Completed Bar Indexing & Precise Non-Overlapping Boundaries

To eliminate ambiguity between bar opening and closing timestamps, this protocol establishes the exact 1-based completed bar indexing convention for the 25 fifteen-minute intervals of the standard NSE session $[09:15, 15:30\text{ IST}]$:

| Bar Index ($k$) | Interval Span (IST) | Completed Close Timestamp ($T_{\text{close}}$) | Time Remaining to 15:15 ($T_{\text{available}}$) | $H=60\text{m}$ Eligible | $H=30\text{m}$ Eligible | $H=15\text{m}$ Eligible | $H=7\text{m}$ Eligible |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **1** | 09:15:00 – 09:30:00 | 09:30:00.000 | 345 min | Yes | Yes | Yes | Yes |
| **2** | 09:30:00 – 09:45:00 | 09:45:00.000 | 330 min | Yes | Yes | Yes | Yes |
| ... | ... | ... | ... | Yes | Yes | Yes | Yes |
| **18** | 13:30:00 – 13:45:00 | 13:45:00.000 | 90 min | Yes | Yes | Yes | Yes |
| **19** | 13:45:00 – 14:00:00 | 14:00:00.000 | 75 min | **Yes (Last 60m)** | Yes | Yes | Yes |
| **20** | 14:00:00 – 14:15:00 | 14:15:00.000 | 60 min | **Boundary (60m)**| Yes | Yes | Yes |
| **21** | 14:15:00 – 14:30:00 | 14:30:00.000 | 45 min | Excluded | **Yes (Last 45m)** | Yes | Yes |
| **22** | 14:30:00 – 14:45:00 | 14:45:00.000 | 30 min | Excluded | **Yes (Last 30m)** | Yes | Yes |
| **23** | 14:45:00 – 15:00:00 | 15:00:00.000 | 15 min | Excluded | Excluded | **Yes (Last 15m)** | Yes |
| **24** | 15:00:00 – 15:15:00 | 15:15:00.000 | 0 min | Excluded | Excluded | Excluded | **Square-Off** |
| **25** | 15:15:00 – 15:30:00 | 15:30:00.000 | -15 min | Excluded | Excluded | Excluded | **Session End** |

### Execution Rules & Non-Overlapping Invariants
1. **Decision Timestamp ($T_{\text{decision}}$):** Defined strictly at the **closing timestamp** of the completed 15-minute bar: $T_{\text{decision}} \equiv T_{\text{close}}(k) \in \{09:30, 09:45, \dots, 15:00\text{ IST}\}$.
2. **Signal Cadence & Seven-Minute Horizon Constraint:**
   * Under the canonical completed 15-minute bar observer, signal evaluation occurs exclusively at completed bar boundaries.
   * Therefore, $H = 7\text{m}$ operates as a **maximum holding duration** for trades entered at completed bar closes (e.g., entered at 15:00:00 IST and held until 15:07:00 IST).
   * The theoretical mathematical limit of 15:08:00 IST applies only if an asynchronous event-driven or 1-minute streaming feed is deployed. Under the frozen 15-minute bar observer, the **latest observable decision timestamp** for any trade is Bar 23 close at **15:00:00 IST** ($T_{\text{available}} = 15\text{m}$).
3. **Pricing Information Set:** Signal calculations ($\text{traj}_{15}$, volume ratio) utilize strictly information up to $T_{\text{close}}(k)$.
4. **Execution Pricing:** Fill price is modeled at the prevailing top-of-book executable quote immediately succeeding $T_{\text{close}}(k)$ (Ask for Long entry, Bid for Long exit). Zero look-ahead.

---

## 3. Six Session Partitions (Windows) for Empirical Comparison

To answer whether the strategy's statistical edge varies by time of day, all prospective evaluations will be partitioned into six mutually exclusive, non-overlapping windows:

```
09:15                11:00                13:00         14:00         14:30         15:00    15:15   15:30
  │─── Window 1 ───────│─── Window 2 ───────│── Window 3 ─│── Window 4 ─│── Window 5 ─│─ W6 ─│ Square│ Close
  Bars 1 to 7          Bars 8 to 15         Bars 16 to 19 Bars 20 to 21 Bars 22 to 23  Bar 24 Off
```

1. **Window 1: Opening Discovery & Volatility Surge (09:15 – 11:00 IST / Bars 1–7)**  
   *Research Focus:* Does high morning turnover generate genuine momentum or excessive false breakouts?
2. **Window 2: Mid-Session Structural Continuation (11:00 – 13:00 IST / Bars 8–15)**  
   *Research Focus:* Does the momentum edge persist when turnover declines and spreads compress?
3. **Window 3: European Open Alignment (13:00 – 14:00 IST / Bars 16–19)**  
   *Research Focus:* Does institutional macro flow re-ignite directional momentum before the current cutoff?
4. **Window 4: Pre-Closeout Extended Window (14:00 – 14:30 IST / Bars 20–21)**  
   *Research Focus:* Are 45–75 minutes of holding time sufficient for $+1.5\%$ targets to be realized?
5. **Window 5: Late-Day Acceleration & Short Covering (14:30 – 15:00 IST / Bars 22–23)**  
   *Research Focus:* Can shortened horizons ($H \in \{7\text{m}, 15\text{m}, 30\text{m}\}$) overcome fixed turnover friction?
6. **Window 6: Terminal Intraday Window (15:00 – 15:15 IST / Bar 24)**  
   *Research Focus:* Does any defensible statistical edge exist when maximum duration is capped at $< 15$ minutes?

---

## 4. Dual Estimand Separation & Right-Censoring Rules

To prevent confounding trade profitability with directional predictive validity, the two estimands are evaluated independently:

### 4.1 Estimand A: Simulated Trade Lifecycle
- **Entry Rules:** $\text{traj}_{15} \ge +0.0500$, history $\ge 5$ bars, $H \le T_{\text{available}}$.
- **Terminal Exit State Taxonomy:**
  Every trade is resolved into exactly one of five explicit, non-overlapping terminal states:
  1. `TARGET_HIT`: Executable bid $\ge P_{\text{entry}} \times 1.015$ ($+1.5\%$ gross) prior to stop or timeout.
  2. `STOP_LOSS_HIT`: Executable bid $\le P_{\text{entry}} \times 0.990$ ($-1.0\%$ gross) prior to target or timeout.
  3. `MAX_HOLDING_EXPIRED`: Elapsed duration reaches $H$ minutes while neither target nor stop was breached; closed at prevailing market bid.
  4. `MANDATORY_INTRADAY_SQUARE_OFF`: Session reaches 15:15:00 IST prior to $T_{\text{entry}} + H$; forcibly closed at market bid.
  5. `DATA_QUALITY_UNRESOLVED`: Insufficient subsequent quote depth to establish valid exit pricing.
- **Conservative Ambiguity Resolution:**
  If intraday price range indicates that *both* target and stop levels were crossed within the same evaluation interval, the outcome is **conservatively resolved as `STOP_LOSS_HIT`** (adverse execution assumption to eliminate optimistic fill bias).
- **Cost Model:** Full Indian statutory schedule (Brokerage, STT, Exchange charges, GST, SEBI fee, Stamp duty, Spread crossing, and 2.0 bps slippage).

### 4.2 Estimand B: Fixed-Horizon Directional Prediction
- **Definition:** Predicts signed price change over horizon $H$:
  $$Y_H = \mathbb{I}\left(P_{t + H} > P_t\right)$$
- **Right-Censoring Invariant:** If $T_{\text{decision}} + H > 15:30:00\text{ IST}$ (market close), future within-session prices are structurally unavailable. Such intervals are formally tagged:
  $$\texttt{status} = \texttt{RIGHT\_CENSORED\_INCOMPLETE\_HORIZON}$$
  They are **strictly excluded from the balanced accuracy denominator** rather than recorded as negative predictions ($TN$ or $FN$), preventing artificial penalty on late-session directional models.
- **Non-Censoring Independence:** A trade that enters and resolves under Estimand A (e.g. hitting target at +18m or squared off at 15:15) remains **100% valid and included in Estimand A**, even if its corresponding Estimand B 60m horizon extends beyond 15:30 IST and is right-censored. The two estimands are completely decoupled.

### 4.3 Data-Granularity Matching & Target/Stop Observation Rules
1. **Timestamped L1 Bid/Ask Quotes (Primary Replay Source):**
   * Path traversal evaluates quotes in chronological order ($t > T_{\text{decision}}$).
   * First valid crossing governs: If bid $\ge P_{\text{target}}$ first $\implies \texttt{TARGET\_HIT}$. If bid $\le P_{\text{stop}}$ first $\implies \texttt{STOP\_LOSS\_HIT}$.
2. **OHLC Candle Aggregation (Fallback When High-Frequency Quotes Absent):**
   * If both $+1.5\%$ target and $-1.0\%$ stop levels fall within $[L, H]$ of the candle and internal sequence is unknown, resolve **conservatively as $\texttt{STOP\_LOSS\_HIT}$** (adverse execution priority).
   * If quote data are missing, crossed, or stale (> 15m), do **not** infer favorable exits from candle high/low; resolve as $\texttt{DATA\_QUALITY\_UNRESOLVED}$ or exit at candle close bid.
3. **Simultaneous Timeout & Threshold Crossing Resolution:**
   * If an executable quote at the exact expiry instant $t = T_{\text{entry}} + H$ triggers both a boundary crossing and horizon expiry:
     * If bid $\ge P_{\text{target}} \implies \texttt{TARGET\_HIT}$ at target price.
     * If bid $\le P_{\text{stop}} \implies \texttt{STOP\_LOSS\_HIT}$ at stop price.
     * If neither boundary is crossed $\implies \texttt{MAX\_HOLDING\_EXPIRED}$ at executable bid.

### 4.4 Replay Row Schema Invariant
Every row in the replay output ledger must record the full audit trail:
- `bar_index`: 1..25
- `candle_interval_ist`: e.g. `[13:45, 14:00 IST)`
- `decision_timestamp_ist`: ISO-8601 string, e.g. `2026-10-26T14:00:00.000+05:30`
- `decision_timestamp_utc_ms`: epoch milliseconds
- `entry_eligible`: boolean
- `entry_exclusion_reason`: `NONE_ELIGIBLE`, `COLD_START_INSUFFICIENT_HISTORY`, `SUB_THRESHOLD_MOMENTUM`, `INSUFFICIENT_RUNWAY_TO_SQUARE_OFF`, `EXISTING_POSITION_ACTIVE`, `DATA_QUALITY_FAILURE`
- `estimand_a`: `{status, entry_price, exit_price, exit_timestamp_iso, gross_return_bps, statutory_friction_bps, net_return_bps, mfe_bps, mae_bps, holding_seconds}`
- `estimand_b`: `{status, horizon_seconds, forward_return_bps, directional_label, directional_hit}`

---

## 5. Statistical Rigor, Excursions, and Zero-MAE Regularization

### 5.1 Maximum Adverse & Favorable Excursions
For each evaluated signal over its holding path:
$$\text{MFE}_{\text{bps}} = \max_{t \in [T_{\text{entry}}, T_{\text{exit}}]} \left( \frac{P_{\text{bid}}(t) - P_{\text{entry}}}{P_{\text{entry}}} \times 10{,}000 \right)$$
$$\text{MAE}_{\text{bps}} = \max_{t \in [T_{\text{entry}}, T_{\text{exit}}]} \left( \frac{P_{\text{entry}} - P_{\text{bid}}(t)}{P_{\text{entry}}} \times 10{,}000 \right)$$

### 5.2 Zero-MAE Regularization for Edge Ratio
When price never moves adversely below the entry fill, $\text{MAE} = 0.00\text{ bps}$, causing the unregularized ratio $\text{MFE} / \text{MAE}$ to diverge to $+\infty$.  
To maintain numerical stability across window aggregates:
$$\text{Edge Ratio}_{\text{window}} = \frac{\overline{\text{MFE}}}{\max\left(\overline{\text{MAE}}, \; 0.50\text{ bps}\right)}$$
**Primary Metric Invariant:** The $0.50\text{ bps}$ floor is strictly a numerical reporting convention to prevent division by zero; it is **not** evidence of strategy edge. The raw distributions of $\overline{\text{MFE}}$ and $\overline{\text{MAE}}$ remain the primary empirical evidence. Individual trades with $\text{MAE} = 0.00\text{ bps}$ are tagged $\texttt{PERFECT\_FAVORABLE\_PATH}$, with raw excursion values explicitly recorded.

### 5.3 Uncertainty & Confidence Bounds
- **Win Rate:** Wilson 95% binomial score confidence intervals:
  $$w \pm \frac{z^2}{2N} \pm z \sqrt{\frac{w(1-w)}{N} + \frac{z^2}{4N^2}} \Big/ \left(1 + \frac{z^2}{N}\right)$$
- **Net Expectancy:** Bootstrapped 95% confidence intervals (1,000 resamples).
- **Sample Floor:** Minimum $N \ge 30$ evaluations per window required before comparative claims are evaluated.

### 5.4 Pre-Registered Analysis Parameters & Multiple Comparisons
- **Frozen Parameter Vector:**
  * Evaluated Horizons: $H \in \{7\text{m}, 15\text{m}, 30\text{m}, 60\text{m}\}$
  * Target Boundary: $+1.50\%$ ($+150.0\text{ bps}$)
  * Stop Loss Boundary: $-1.00\%$ ($-100.0\text{ bps}$)
  * Intraday Square-Off: Exactly 15:15:00 IST ($T_{\text{squareoff}}$)
  * Notional Per Trade: ₹100,000 fixed
- **Multiple Comparisons Architecture:**
  * **Baseline Comparator:** Windows 1–3 ($k \le 19$, 09:15–14:00 IST) at $H=60\text{m}$.
  * **Hypothesis Family ($M = 23$ tests):** Each alternative cell $(w, H) \neq (\text{Windows 1–3}, 60\text{m})$ is tested for superior net expectancy against the registered baseline:
    $$H_0^{(w, H)}: \mathbb{E}[\text{Net Return}_{(w, H)}] \le \mathbb{E}[\text{Net Return}_{\text{baseline}}]$$
  * **Primary Inferential Test:** Exact paired session-level **sign-flip permutation test** on the $N = 20$ paired differences $D_s = \bar{R}_s^{(w, H)} - \bar{R}_s^{(\text{baseline})}$.
    - *Theoretical Condition:* Validity relies on exchangeability under sign changes under the null ($D_s \stackrel{d}{=} -D_s$), assuming conditional symmetry of paired differences under $H_0$.
    - *Exact Enumeration:* Evaluates all $2^{20} = 1,048,576$ sign configurations.
  * **Secondary Diagnostic Tests:** Paired Wilcoxon signed-rank test (applying Pratt's adjustment for zero-difference sessions) and paired Welch's t-test reported alongside for sensitivity.
  * **Zero-Trade Sessions:** If neither candidate nor baseline emits a trade on session $s$, $D_s = 0.0\text{ bps}$ (capital preserved; permutes neutrally).
  * **Effect Sizes & Reproducible Bootstrap:**
    - Reports sample mean paired difference $\bar{D}$, median paired difference, and standard error.
    - 95% percentile bootstrap confidence intervals generated using **1,000 resamples with pre-registered random seed `20261026`** for deterministic reproducibility.
  * **Multiple Testing Family Invariant:** The full family of $M = 23$ non-baseline cells is preserved without post-hoc cherry-picking or exclusion of zero-trade partitions.
  * **Null Standard:** Non-significance is strictly interpreted as **"insufficient evidence of superiority"**, never equivalence.

---

## 6. Pre-Replay Data Acceptance Checklist & Implementation Roadmap

Before any multi-session counterfactual replay results are authorized for strategy conclusions, the observation ledger must satisfy all seven real-data acceptance gates on an active trading session:

| Gate | Check Name | Verification Requirement | Status |
| :---: | :--- | :--- | :---: |
| **G1** | **History Restoration** | Persisted bar history (`rp608_nse_bar_history.json`) loads correctly across process restarts without data loss. | Verified (Test run) |
| **G2** | **Causal Bar Progression** | Each completed 15m candle advances history buffer by exactly one bar; repeated queries never inflate history. | Verified (Test run) |
| **G3** | **Eligible-Window Evaluation** | Decisions evaluated within $1 \le \text{bar\_idx} \le 20$ ($T_{\text{close}} \le 14:15\text{ IST}$) with $\ge 5$ bars history. | **Pending Live Session** |
| **G4** | **Quote Integrity** | Entry/exit quotes are timestamped, executable, non-crossed, and within registered age limit ($< 60\text{s}$). | Verified (Kite L1) |
| **G5** | **Forensic Reconciliation** | At least one genuine qualifying breakout ($\text{traj}_{15} \ge +0.0500$) traced end-to-end through entry, exit, and costs. | **Pending Live Session** |
| **G6** | **Estimand Separation** | Estimand A lifecycle trade resolves cleanly and independently of whether Estimand B is right-censored. | Verified (Unit tests) |
| **G7** | **Reproducibility** | Immutable input session ledger produces bitwise-identical SHA-256 JSON replay outputs on identical rules. | Verified (Unit tests) |

### 6.1 Live-Session Evidence Invariants (Gates G3 & G5)
1. **Honest Abstention:** If zero breakout signals satisfy $\text{traj}_{15} \ge +0.0500$ during the session, this must be recorded as genuine market abstention. Thresholds must NEVER be relaxed, nor windows extended, to force an artificial signal.
2. **Unresolved Lifecycles:** If a qualifying signal enters but does not hit target or stop within the session, it must remain explicitly tagged `UNRESOLVED` (or closed strictly under the 15:15 mandatory square-off rule). It must NEVER be artificially resolved as a completed target or stop.

```text
                  CURRENT LIVE HOST (Frozen)
         ┌───────────────────────────────────────────┐
         │  run_rp608_nse_shadow_daemon.py           │
         │  Entry Rule: bar_idx <= 20 (14:15 IST)    │
         │  Order Authority: 0.0%                    │
         │  Status: IMMUTABLE BASELINE               │
         └─────────────────────┬─────────────────────┘
                               │
                               ▼ Next Live Trading Session (09:15–15:30 IST)
         ┌───────────────────────────────────────────┐
         │  Live Data-Integrity & Causal Audit Drill │
         │  Verify: G1 to G7 Acceptance Checklist    │
         │  Trace: First Real Qualifying Breakout    │
         └─────────────────────┬─────────────────────┘
                               │
                               ▼ Prospective 20-Session Accumulation
         ┌───────────────────────────────────────────┐
         │  RP-608-TIMING-EXP-001 Replay Harness     │
         │  Evaluates: Windows 1 to 6                │
         │  Horizons: H in {7m, 15m, 30m, 60m}       │
         │  Test: Paired Permutation Test (M=23)     │
         └───────────────────────────────────────────┘
```

1. **Frozen Live Daemon:** The production shadow daemon on `LCUtest` remains strictly frozen with 0.0% order authority.
2. **Next Milestone:** Conduct the real-session data-integrity and forensic causal audit during the next active trading window. No multi-session performance claims will be evaluated until all 7 gates are satisfied.
