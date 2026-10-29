# RP-608-PVP-002: Prospective Predictive Validation Protocol
## Pre-Entry Exhaustion Signal & Policy Comparison Gate

**Protocol ID:** RP-608-PVP-002  
**Series:** Prospective Predictive Validation (Shadow Research Only)  
**Parent Diagnostic:** RP-608-PVP-001 (Price–Volume Attribution)  
**Governance Scope:** `adapters/chronosentiment` (Native Rust Harness)  
**Live Order Authority:** 0.0% (Frozen Adapter Production Engine)  
**Core Crate Modification:** Strictly 0.0% (Zero mutations to `coralys-*`)  
**Status:** PRE-REGISTERED & FROZEN (Awaiting Incoming Forward Sessions)  

---

## 1. Governance Context & Separation of Research Lines

Following the decisive falsification and permanent closure of **RP-608-INV-001** (Exact Direction Reversal) and the retention of **RP-608-PVP-001** as descriptive diagnostic evidence, this protocol pre-registers the rules, feature definitions, statistical inference methods, dual fee models, and multi-dimensional decision gates for prospective validation.

To prevent circular in-sample experimentation, the historical development holdout (28 September – 01 October 2026, $N=2,029$ trades across 99 symbols) is **permanently closed to further threshold tuning or post-hoc parameter search**.

All empirical tests under this protocol will be executed exclusively on **genuinely unseen prospective trading sessions** ($S \ge 15$ forward sessions, $N \ge 500$ eligible signals) captured under live shadow conditions.

### 1.1 Audit Revision Note: Reference Price Calculation Integrity (186 vs 242 Trades)
In preliminary exploratory scripts, VWAP distance was inadvertently evaluated against the eventual execution fill price ($P_{\text{fill}}$), yielding an apparent cohort of 242 flagged trades. Enforcing the strict decision-time reference price $P_{\text{ref}} = C_{-1}$ removes post-decision fill information from the feature calculation, resulting in exactly **186 flagged trades (9.2% of the population)** across the historical sample. 
This reduction is strictly documented as a **feature-definition data-integrity correction and historical audit revision**—not an optimization, post-hoc tuning step, or newly discovered trading improvement. The historical holdout sample remains permanently closed, and all pre-registered signal thresholds ($\text{ret\_15m\_aligned} \ge +30.0\text{ bps}$, $\text{dist\_vwap\_bps} \ge +45.0\text{ bps}$) remain frozen.

---

## 2. Frozen Feature Definitions & Decision-Time Price Integrity

Every feature must use only information observable at the decision timestamp (second `:00` of the candidate entry candle). No intrabar data, wick information, or subsequent execution fills from the candidate trade are visible to the feature engine.

```
       Prior Completed 1m Bars                  Entry Candle
[...][t-17][t-16] ... [t-2][t-1]  | [t_entry :00] ---------------> Exit
 <------ 15m Momentum Lookback -> | (Decision Point - Second :00)
```

### 2.1 Decision-Time Reference Price ($P_{\text{ref}}$) vs Execution Fill Price ($P_{\text{fill}}$)
To eliminate information leakage and maintain strict alignment between research and execution:
- **Decision-Time Reference Price ($P_{\text{ref}}$):** The price observable to the candidate signal engine at second `:00` of the candidate entry minute $t$. In discrete 1-minute data, this is authoritatively defined as the close of the immediately preceding completed candle:
  $$P_{\text{ref}} = C_{-1}$$
  $P_{\text{ref}} = C_{-1}$ is the **sole authoritative feature input** for all lookback returns and VWAP stretch calculations. Equality with the candidate candle open ($O_{\text{entry}} = C_{-1}$) is **not** an operational assumption of the formula, but an **empirical data-integrity check** ($|O_{\text{entry}} - C_{-1}|$). In our historical holdout audit, candle opening gap averaged $1.82\text{ bps}$ (max $29.91\text{ bps}$ across 2,029 signals due to auction cross mechanics and data vendor timestamps), confirming why feature extraction must decouple strictly from candidate bar opening data.
- **Realized Fill Price ($P_{\text{fill}}$):** The actual execution price returned by the market broker. This price is reserved strictly for trade accounting, gross P&L, transaction costs, and net P&L. It is never accessed by feature calculation logic.

### 2.2 Aligned 15-Minute Momentum (`ret_15m_aligned`)
Measures the directional run over the preceding 15 completed 1-minute bars prior to decision time:
$$\text{ret\_15m\_aligned} = \left(\frac{C_{-1} - C_{-16}}{C_{-16}}\right) \times 10,000 \times \text{dir\_mult}$$
- $C_{-1}$: Close of the immediately preceding completed 1-minute bar.
- $C_{-16}$: Close of the bar 15 minutes prior to $C_{-1}$.
- $\text{dir\_mult}$: $+1.0$ if candidate entry side is `LONG`; $-1.0$ if candidate entry side is `SHORT`.
- **Frozen Threshold:** $\text{ret\_15m\_aligned} \ge +30.0\text{ bps}$.

### 2.3 Aligned Session VWAP Distance (`dist_vwap_bps`)
Measures how far the decision-time reference price has stretched beyond the volume-weighted anchor in the direction of the trade:
$$\text{dist\_vwap\_bps} = \left(\frac{P_{\text{ref}} - \text{VWAP}}{\text{VWAP}}\right) \times 10,000 \times \text{dir\_mult}$$
where VWAP is computed from session open (09:15 IST / 03:45 UTC) up to minute $t_{-1}$:
$$\text{VWAP} = \frac{\sum_{k=\text{open}}^{-1} P_k \cdot V_k}{\sum_{k=\text{open}}^{-1} V_k}$$
- **Frozen Sign Convention:**
  - For `LONG`: $P_{\text{ref}} \ge \text{VWAP} \times 1.0045$ ($+45.0$ bps above VWAP).
  - For `SHORT`: $P_{\text{ref}} \le \text{VWAP} \times 0.9955$ ($+45.0$ bps below VWAP).
- **Frozen Threshold:** $\text{dist\_vwap\_bps} \ge +45.0\text{ bps}$.

### 2.4 Candidate Flag Definition
A candidate entry signal is flagged as **Overextended / Exhaustion-Prone** if and only if:
$$\text{Flagged}(j) \iff \text{ret\_15m\_aligned}(j) \ge 30.0\text{ bps} \;\land\; \text{dist\_vwap\_bps}(j) \ge 45.0\text{ bps}$$

---

## 3. Policy Architecture on Identical Opportunity IDs

All three policies are evaluated across the exact same sequence of eligible entry opportunities generated by the frozen adapter ($j \in \{1, \dots, N\}$). Opportunity IDs must match 1:1 across all comparisons.

| Policy | Treatment of Unflagged Entries | Treatment of Flagged (Overextended) Entries | Rationale |
| :--- | :--- | :--- | :--- |
| **Policy 0 (Baseline)** | Execute baseline direction | Execute baseline direction | Control benchmark (frozen adapter) |
| **Policy 1 (No-Trade Gate)** | Execute baseline direction | **Skip entry (0.00 Gross, 0.00 Fee, 0.00 Net)** | Test whether skipping bad trades improves net expectancy |
| **Policy 2 (Contrarian Reversal)** | Execute baseline direction | **Reverse direction (Mirrored Stop / Path Replay)** | Test whether exhaustion provides executable contrarian alpha |

### 3.1 Accounting for Skipped Trades (Policy 1)
To ensure Policy 1 is not artificially favored by simply taking fewer trades:
- Every eligible signal $j$ is accounted for in the evaluation denominator $N$.
- Skipped trades have:
  $$\text{Gross P\&L}_1(j) = 0.00, \quad \text{Friction}_1(j) = 0.00, \quad \text{Net P\&L}_1(j) = 0.00$$
- **Primary Metric:** Net Expectancy per eligible signal opportunity:
  $$\mathbb{E}[\text{Net}] = \frac{\sum_{j=1}^N \text{Net P\&L}(j)}{N}$$

---

## 4. Execution Physics & Dual Cost-Model Specification

Transaction friction is recalculated independently at the individual execution level for every simulated trade. It must never be copied from the baseline.

To avoid ambiguity, the protocol explicitly distinguishes between the **Historical Ledger-Reconciliation Model** and the **Prospective Execution-Cost Model**, and explicitly declares the denominator for every basis point figure:
- **One-Leg Entry Notional Basis:** Denominator is $P_{\text{entry}} \cdot Q$.
- **Total Turnover Basis:** Denominator is $(P_{\text{entry}} + P_{\text{exit}}) \cdot Q \approx 2 \times \text{Notional}$.
- *Rule of Conversion:* $1.00\text{ bps}$ of Total Turnover $\approx 2.00\text{ bps}$ of One-Leg Entry Notional.

### 4.1 Cost Model 1: Authoritative Ledger-Reconciliation Model (Historical Benchmark)
Used strictly to reproduce historical friction with exact accounting parity:
$$\text{Friction}_j = (P_{\text{entry}, j} \cdot Q_j + P_{\text{exit}, j} \cdot Q_j) \times 0.00045$$
- **Effective Turnover Rate:** $0.00045$ on total turnover ($4.50\text{ bps}$ of turnover).
- **Entry Notional Equivalence:** $9.00\text{ bps}$ of one-leg entry notional.
- **Reconciliation Audit:** Reconciles the historical ledger ($N=2,029$) with **0.0000 Paisa row-level error** across Gross P&L, Friction, and Net P&L (total friction ₹1,81,029.04).

### 4.2 Cost Model 2: Prospective Itemized Execution-Cost Model (Forward Shadow)
For prospective validation and sensitivity testing, costs are decomposed into explicit statutory, brokerage, and slippage schedules. 

To eliminate any ambiguity between raw percentage rates, statutory tax bases, and basis point conventions, the exact derivation is specified below:

| Component | Statutory Tax Base | Official Rate / Amount | Rupee Amount on ₹2L Entry Notional (₹4L Turnover) | Turnover Basis (bps) | Entry Notional Basis (bps) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Exchange Turnover Fee** | Total Turnover ($P_{\text{entry}}Q + P_{\text{exit}}Q$) | $0.00345\%$ | $₹4,00,000 \times 0.0000345 = ₹13.80$ | $0.345\text{ bps}$ | $0.690\text{ bps}$ |
| **SEBI Regulatory Fee** | Total Turnover ($P_{\text{entry}}Q + P_{\text{exit}}Q$) | $0.00010\%$ | $₹4,00,000 \times 0.0000010 = ₹0.40$ | $0.010\text{ bps}$ | $0.020\text{ bps}$ |
| **Stamp Duty** | **Buy Leg Only** ($P_{\text{buy}}Q$) | $0.00300\%$ | $₹2,00,000 \times 0.0000300 = ₹6.00$ | $0.150\text{ bps}$ | $0.300\text{ bps}$ |
| **Securities Transaction Tax (STT)** | **Sell Leg Only** ($P_{\text{sell}}Q$) | $0.02500\%$ | $₹2,00,000 \times 0.0002500 = ₹50.00$ | $1.250\text{ bps}$ | $2.500\text{ bps}$ |
| **GST on Exchange Turnover** | Exchange Fee Only | $18\%$ on Exchange Fee | $₹13.80 \times 0.18 = ₹2.484$ | $0.062\text{ bps}$ | $0.124\text{ bps}$ |
| **Statutory Subtotal** | — | — | **$₹13.80 + ₹0.40 + ₹6.00 + ₹50.00 + ₹2.484 = ₹72.684 \approx ₹72.68$** | **$1.817 \approx 1.82\text{ bps}$** | **$3.634 \approx 3.63\text{ bps}$** |
| **Brokerage & GST** | Per Order Pair | ₹40 + 18% GST (₹7.20) flat | **₹47.20** | $1.180\text{ bps}$ | $2.360\text{ bps}$ |
| **Execution Slippage Allowance** | Per Leg (Entry + Exit) | $2.00\text{ bps}$ entry + $2.00\text{ bps}$ exit | $₹2,00,000 \times 0.0002 \times 2 = ₹80.00$ | $2.000\text{ bps}$ | $4.000\text{ bps}$ |
| **Total Round-Trip Friction** | — | — | **$₹72.684 + ₹47.20 + ₹80.00 = ₹199.884 \approx ₹199.88$** | **$4.997 \approx 5.00\text{ bps}$** | **$9.994 \approx 10.00\text{ bps}$** |
| **Adverse Slippage Stress** | One-Leg Entry Notional | $+1.00\text{ bps}$ on $P_{\text{entry}}Q$ (once) | $₹2,00,000 \times 0.0001000 = ₹20.00$ | $+0.500\text{ bps}$ | $+1.000\text{ bps}$ |
| **Stressed Total Friction** | — | — | **$₹199.884 + ₹20.00 = ₹219.884 \approx ₹219.88$** | **$5.497 \approx 5.50\text{ bps}$** | **$10.994 \approx 11.00\text{ bps}$** |

*Why Simple Percentage Summation is Invalid:*
Summing the raw statutory percentages yields $0.00345\% + 0.00010\% + 0.00300\% + 0.02500\% + 0.000621\% = 0.032171\%$. This aggregate percentage cannot be multiplied directly against ₹2L or ₹4L because Stamp Duty applies strictly to the Buy Leg (₹2L) and STT applies strictly to the Sell Leg (₹2L), while Exchange and SEBI fees apply to Total Turnover (₹4L). When each component is evaluated on its legal tax base, the exact statutory amount is ₹72.68, which corresponds identically to:
- **$1.8171\text{ bps}$** on Total Turnover of ₹4,00,000 ($₹4,00,000 \times 0.00018171 = ₹72.684$).
- **$3.6342\text{ bps}$** on One-Leg Entry Notional of ₹2,00,000 ($₹2,00,000 \times 0.00036342 = ₹72.684$).

*Specification & Implementation Note:*
- **Actual Trade Execution Basis:** The prospective model calculates actual itemized statutory fees, flat brokerage, and slippage for each simulated execution based on actual filled quantities and prices ($P_{\text{entry}}, P_{\text{exit}}, Q$). The ₹2,00,000 notional is an empirical illustration, not a flat universal rate.
- **Unambiguous Adverse Stress Denominator:** The adverse slippage stress test adds exactly $+1.00\text{ bps}$ of one-leg entry notional ($+0.0001 \times P_{\text{entry}} \cdot Q$, or equivalently $+0.50\text{ bps}$ of total turnover), applied **strictly once per trade** to avoid double-charging.

### 4.3 Stop Execution Ambiguity
If a 1-minute OHLC bar touches both the stop price and the exit price, the simulator must apply the conservative worst-case stop fill.

---

## 5. Statistical Inference, Testing Plan & Stopping Rules

### 5.1 Primary Inferential Statistic: Scale-Invariant Normalized Session Test
Because trade opportunities cluster within sessions and correlate across symbols due to market-wide volatility, trade-level paired tests overstate sample size and precision.

To guarantee that sessions with higher trade counts do not exert disproportionate leverage, the primary statistical inference is conducted on **scale-invariant normalized session outcomes**:
1. For each session $s \in \{1, \dots, S\}$, calculate the session-level net return per eligible opportunity:
   $$\text{NetReturn}_{\text{bps}, 0}(s) = \frac{1}{N_s} \sum_{j \in s} \frac{\text{Net}_0(j)}{P_{\text{entry}, j} \cdot Q_j} \times 10,000$$
   $$\text{NetReturn}_{\text{bps}, 1}(s) = \frac{1}{N_s} \sum_{j \in s} \frac{\text{Net}_1(j)}{P_{\text{entry}, j} \cdot Q_j} \times 10,000$$
2. Compute the normalized session paired delta:
   $$\delta_{\text{bps}, s} = \text{NetReturn}_{\text{bps}, 1}(s) - \text{NetReturn}_{\text{bps}, 0}(s) \quad (\text{bps per eligible signal opportunity})$$
3. Compute the session-clustered test statistic:
   $$\bar{\Delta}_{\text{bps}} = \frac{1}{S} \sum_{s=1}^S \delta_{\text{bps}, s}, \quad s_{\text{bps}}^2 = \frac{1}{S-1} \sum_{s=1}^S (\delta_{\text{bps}, s} - \bar{\Delta}_{\text{bps}})^2, \quad \text{SE}_{\text{bps}} = \frac{s_{\text{bps}}}{\sqrt{S}}$$
   $$t_{\text{session, bps}} = \frac{\bar{\Delta}_{\text{bps}}}{\text{SE}_{\text{bps}}}, \quad \text{with } df = S - 1$$

### 5.2 Hypothesis Declaration & Tail Conventions
Because Policy 1 is strictly a risk-mitigation / loss-reduction filter (skipping flagged trades), the pre-registered scientific question is directional: does Policy 1 improve net expectancy relative to Policy 0?

- **Pre-Registered Primary Hypothesis:**
  $$H_0: \mathbb{E}[\delta_{\text{bps}}] \le 0 \quad \text{vs} \quad H_1: \mathbb{E}[\delta_{\text{bps}}] > 0$$
- **Primary Tail Probability:** Evaluated using the exact Student-$t$ regularized incomplete beta upper-tail integral:
  $$p_{\text{one-sided}} = P(T_{S-1} \ge t) = \frac{1}{2} I_{\frac{df}{df + t^2}}\left(\frac{df}{2}, \frac{1}{2}\right) \quad \text{for } t \ge 0$$
- **Two-Sided Cross-Check:** For completeness and non-directional auditing, the two-sided p-value is also reported:
  $$p_{\text{two-sided}} = 2 \times p_{\text{one-sided}} = I_{\frac{df}{df + t^2}}\left(\frac{df}{2}, \frac{1}{2}\right)$$
- **95% Confidence Interval:** Two-sided 95% confidence interval:
  $$[\bar{\Delta}_{\text{bps}} - t_{0.025, S-1} \cdot \text{SE}_{\text{bps}}, \; \bar{\Delta}_{\text{bps}} + t_{0.025, S-1} \cdot \text{SE}_{\text{bps}}]$$

### 5.3 Secondary Diagnostics & Robustness Reporting
1. **Pooled Descriptive Delta ($\Delta \mathbb{E}[\text{Net}]_{\text{pooled}}$):** The unweighted pooled net return delta across all $N$ eligible signals:
   $$\Delta \mathbb{E}[\text{Net}]_{\text{pooled}} = \frac{\sum_{j=1}^N \text{Net}_1(j) - \sum_{j=1}^N \text{Net}_0(j)}{\sum_{j=1}^N (P_{\text{entry}, j} \cdot Q_j)} \times 10,000$$
   Reported strictly as a secondary descriptive economic metric, while the primary inferential test and Gate 1 decision use the scale-invariant equal-weighted session mean $\bar{\Delta}_{\text{bps}}$.
2. **Rupee Session Delta ($\Delta_{\text{inr}, s}$):** Total rupee net difference per session $\Delta_{\text{inr}, s} = \text{Net}_1(s) - \text{Net}_0(s)$ with session-clustered $t$-test.
3. **Trade-Level Paired Student-$t$ Test ($df = N - 1$):** Paired trade-by-trade test $t_{\text{trade}} = \bar{d} / \text{SE}_d$.
4. **Pratt-Modified Wilcoxon Signed-Rank Test:** Evaluated across non-zero differences ($D_j \ne 0$) after ranking all $|D_j|$, preventing artificial inflation of significance from inactive rows.
5. **Session Reporting Matrix:** Every prospective session must report: date, eligible signals $N_s$, flagged signals $K_s$, Policy 0 Net (₹ and bps), Policy 1 Net (₹ and bps), Policy 2 Net (₹ and bps), and Delta (₹ and bps).

### 5.4 Stopping Rules & Terminal Evaluation
- **Minimum Prospective Sample:** Terminal evaluation requires at least $S \ge 15$ forward sessions and $N \ge 500$ eligible trade opportunities.
- **Pure Terminal Evaluation (No Interim Inferential Looks):** By default, the prospective test operates strictly as a **pure terminal evaluation** at the pre-registered horizon ($S \ge 15, N \ge 500$). Passive shadow collection accumulates without interim hypothesis testing, preserving the full nominal significance level ($\alpha = 0.010$) and eliminating alpha-spending ambiguity. Interim results looking favorable must never trigger premature cessation. If an exceptional interim safety halt review is mandated by governance, an O'Brien-Fleming alpha spending function applies with adjusted terminal threshold $\alpha = 0.008$.

### 5.5 Three Ingestion & Governance Controls for Forward Collection
1. **Artefact & Binary Freezing:** The protocol specification revision, Git source commit hash, compiled Rust binary hash (`rp608_research_audit`), execution configuration, fee schedules, and dataset schema are locked prior to the first prospective session. Any software or parameter change requires a new governed revision and cannot be applied retroactively.
2. **Immutable Opportunity Ledger:** For every eligible signal, the forward engine records an immutable row: Opportunity ID, decision timestamp, $P_{\text{ref}} = C_{-1}$, completed-bar feature snapshots (`ret_15m_aligned`, `dist_vwap_bps`), candidate flag status, baseline direction, and simulated trade outcomes for all three policies. Skipped Policy 1 trades must remain in the evaluation denominator $N$ with $0.00$ gross P&L, $0.00$ friction, and $0.00$ net P&L.
3. **Unambiguous Evaluation Horizon & Defect Governance:** Forward collection continues until the sample size floor ($S \ge 15, N \ge 500$) is achieved. If an implementation bug or data-feed corruption occurs during collection, the affected sessions must be isolated and formally documented, with governance determining whether the forward run must restart.

---

## 6. Pre-Declared Multi-Dimensional Acceptance Gates

### 6.0 Tripartite Statistical & Economic Decision Framework
The protocol strictly distinguishes between three hierarchical evaluation questions:
1. **Relative Improvement (Filter Viability):** Does Policy 1 perform better than Policy 0 ($\bar{\Delta}_{\text{bps}} \ge +1.50\text{ bps}$, $p < 0.01$)? This measures loss mitigation and noise reduction. *Passing this gate does NOT establish economic profitability.*
2. **Absolute Economic Viability:** Does Policy 1 achieve positive net expectancy ($\mathbb{E}[\text{Net}] > 0.00\text{ bps}$) after paying actual itemized costs and absorbing $+1.00\text{ bps}$ adverse slippage stress?
3. **Production Canary Readiness:** Are both relative improvement and absolute viability satisfied simultaneously alongside $\ge 75\%$ session consistency and formal governance authorization?

Passing Gate 3 requires more than merely achieving a positive point estimate:

| Decision Gate | Quantitative Hurdle | Consequence if Passed | Consequence if Failed |
| :--- | :--- | :--- | :--- |
| **Gate 1: Policy 1 Filter Viability** | $p_{\text{session, bps}} < 0.01$ (one-sided, $df = S-1$) AND Equal-Weighted Session Mean $\bar{\Delta}_{\text{bps}} \ge +1.50\text{ bps}$ per signal | Policy 1 validated as effective loss-reduction gate | Reject entry filter as statistical noise |
| **Gate 2: Policy 2 Contrarian Alpha** | Net Expectancy $\ge +1.00\text{ bps}$ after itemized friction | Policy 2 promoted for contrarian sleeve development | **Permanently close contrarian reversal hypothesis** |
| **Gate 3: Production Canary Readiness** | **All 4 Multi-Dimensional Criteria Met** (see §6.1) | Authorize formal Canary Phase proposal to Governance | **Retain 0.0% Live Order Authority; freeze production deployment** |

### 6.1 Multi-Dimensional Gate 3 Criteria
A positive net expectancy ($\mathbb{E}[\text{Net}] > 0.00\text{ bps}$) is a **necessary but strictly insufficient** condition to deploy capital. To pass Gate 3, Policy 1 must satisfy all four of the following:

1. **Statistical Significance:** Session-clustered paired $t$-test on normalized session returns achieves $p_{\text{session, bps}} < 0.01$ (one-sided, $df = S - 1$).
2. **Session Consistency:** At least $\lceil 0.75 \times S \rceil$ of evaluated forward sessions show positive net improvement ($\delta_{\text{bps}, s} > 0$). For the pre-registered sample of $S=15$ sessions, at least $\lceil 11.25 \rceil = 12$ sessions must be positive.
3. **Adverse Slippage Stress Test:** Re-evaluating under the itemized prospective model with $+1.00\text{ bps}$ additional execution slippage on one-leg entry notional ($+0.0001 \times P_{\text{entry}} \cdot Q$, added strictly once per trade) must preserve positive net expectancy ($\mathbb{E}[\text{Net}]_{\text{stressed}} > 0.00\text{ bps}$).
4. **Explicit Governance Sign-Off:** Live Order Authority remains locked at 0.0% until formal RP-600 architectural sign-off is granted.

---

## 7. Audit Baseline on Historical Holdout (Reference Benchmark)

For reproducible benchmarking, the native Rust binary `rp608_research_audit` evaluates this exact protocol across the four historical sessions (Sep 28 – Oct 01, 2026, $N=2,029$):

```
Policy Performance Across All 2029 Eligible Opportunity IDs:
  Policy 0 (Baseline):       Net: -Rs 1,71,035.52 | Per Signal: -Rs 84.30 | Return: -8.48 bps
  Policy 1 (No-Trade Gate):  Net: -Rs 1,44,308.32 | Per Signal: -Rs 71.12 | Return: -7.16 bps
  Policy 2 (Reversal):       Net: -Rs 1,50,747.42 | Per Signal: -Rs 74.30 | Return: -7.48 bps

Primary Statistical Inference (Scale-Invariant Normalized Session Test, df = 3):
  Sessions Evaluated:        4
  Positive Sessions:         4/4 (100.0%) [Required >= ceil(0.75*S) = 3 | Passed: true]
  Mean Session Delta:        +1.33 bps per opportunity [Gate 1 Hurdle >= +1.50 bps | Passed: false]
  Pooled Descriptive Delta:  +1.32 bps per opportunity across all 2029 signals
  SE of Session Delta:       0.33 bps
  Session t-statistic:       4.0022
  One-sided p-value:         0.0140
  Two-sided p-value:         0.0280
  95% Confidence Interval:   [+0.27 bps, +2.39 bps]

Empirical Data-Integrity Check (P_ref = C_-1 vs Candidate Candle Open O_entry):
  Audited Candle Gaps:       Mean 1.8207 bps | Max 29.9065 bps (Formula strictly uses authoritative P_ref)

Secondary Statistical Inference (Rupee Session Aggregates, df = 3):
  Mean Session Delta:        +Rs 6,681.80
  SE of Session Delta:       Rs 1,602.70
  Session t-statistic:       4.1691
  One-sided p-value:         0.0126
  Two-sided p-value:         0.0251
  95% Confidence Interval:   [+Rs 1,581.37, +Rs 11,782.23]

Secondary Statistical Inference (Trade-Level Paired Test, df = 2028):
  Delta Net PnL (Saved):     +Rs 26,727.20 (+Rs 13.17 per signal)
  Paired t-statistic:        5.3417
  One-sided p-value:         5.12e-8
  Two-sided p-value:         1.02e-7

Fee Model Reconciliation & Friction Breakdown:
  [Turnover Basis (Turnover = Entry + Exit Notional)]:
    Statutory & Exchange:  1.82 bps (0.345 bps turnover + 0.010 bps SEBI + 0.150 bps stamp + 1.250 bps STT + 0.062 bps GST)
    Brokerage & GST:       1.18 bps (Flat Rs 47.20 / Rs 4L turnover)
    Execution Slippage:    2.00 bps (1.0 bps/side on turnover)
    Total Turnover Rate:   5.00 bps (Historical Rate: 0.00045 per side)
  [Entry Notional Basis (Notional = Entry Price * Qty)]:
    Statutory & Exchange:  3.63 bps (Rs 72.68 on Rs 2L entry notional)
    Brokerage & GST:       2.36 bps (Rs 47.20 on Rs 2L entry notional)
    Execution Slippage:    4.00 bps (2.0 bps entry + 2.0 bps exit = Rs 80.00)
    Total Round-Trip:      10.00 bps (Rs 199.88 on Rs 2L entry notional)
  Total Realized Friction: Rs 181029.04 (0.0000 Paisa historical reconciliation mismatch)

Prospective Itemized Fee & Adverse Slippage Stress Evaluation (Gate 3 Criterion 3):
  Policy 1 Base Itemized Net: Rs -2,06,344.56 (-10.24 bps per opportunity)
  Policy 1 Stressed Net:      Rs -2,24,617.72 (-11.15 bps per opportunity with +1.00 bps slippage stress on entry notional)
  Gate 3 Stressed Edge Preserved: false (Requires E[Net]_stressed > 0.00 bps)

Session Breakdown:
  Session 2026-09-28: 460 signals (22 skipped) | P0: -Rs 54,455.71 (-11.92 bps) | P1: -Rs 50,381.25 (-11.03 bps) | Delta: +Rs 4,074.46 (+0.89 bps)
  Session 2026-09-29: 547 signals (36 skipped) | P0: -Rs 47,077.54 ( -8.64 bps) | P1: -Rs 38,758.63 ( -7.10 bps) | Delta: +Rs 8,318.91 (+1.53 bps)
  Session 2026-09-30: 547 signals (42 skipped) | P0: -Rs 47,076.40 ( -8.65 bps) | P1: -Rs 43,140.18 ( -7.92 bps) | Delta: +Rs 3,936.23 (+0.72 bps)
  Session 2026-10-01: 475 signals (86 skipped) | P0: -Rs 22,425.87 ( -4.77 bps) | P1: -Rs 12,028.26 ( -2.59 bps) | Delta: +Rs 10,397.61 (+2.18 bps)
```

### Critical Takeaway for Prospective Evaluation:
1. **Fails Gate 1 on the Historical Sample:** Under the scale-invariant normalized session test ($df = 3$), $p_{\text{one-sided}} = 0.0140 > 0.0100$ (and $p_{\text{two-sided}} = 0.0280 > 0.0100$) and the equal-weighted session mean delta $+1.33\text{ bps}$ fails the $\ge +1.50\text{ bps}$ hurdle. The historical holdout does **not** achieve pre-registered statistical or economic significance. This confirms why historical holdout data is insufficient to claim a validated edge, and establishes the necessity of collecting $S \ge 15$ forward sessions where $df \ge 14$.
2. **Fails Gate 3 Criterion 3 on the Historical Sample:** Under prospective itemized friction with $+1.00\text{ bps}$ adverse slippage stress on one-leg entry notional, Policy 1 stressed net expectancy is $-11.15\text{ bps} < 0.00\text{ bps}$.
3. **Policy 1 is strictly superior to Policy 2:** Skipping overextended entries mitigates losses (saving ₹26,727.20), whereas reversing them still loses −7.48 bps net after paying transaction costs.
4. **Policy 1 remains net negative (−7.16 bps):** Filtering bad trades reduces total losses, but does not convert an unprofitable baseline into a profitable strategy on historical data.
5. **Prospective Necessity:** Unseen forward sessions will determine whether Policy 1 can achieve economic profitability under novel regimes, or whether structural market edge remains unviable after transaction friction.

---

## 8. Cryptographic Baseline & Prospective Ingestion Invariants

To guarantee scientific immutability and eliminate post-hoc tuning during forward data intake, all artefacts, executables, configurations, and evaluation thresholds are cryptographically locked:

### 8.1 Frozen Artefacts & Checksums

| Artefact Description | File Path / Target | Cryptographic Identifier / SHA256 |
| :--- | :--- | :--- |
| **Git Source Commit** | Repository `HEAD` | `680e18640aba2e30d06c3301005bce9c6e32cbd7` |
| **Compiled Release Executable** | `target/release/rp608_research_audit` | `f57b94807d32852f397c3e6813eb75160fb5a87d09f4e81b00ee2602696e6af4` |
| **Compiled Debug Executable** | `target/debug/rp608_research_audit` | `365965bd194f74241af3e021e68c416bcf9e6076e4f739257c3710b62ded918c` |
| **Audit Harness Source** | `adapters/chronosentiment/research/bin/rp608_research_audit.rs` | `837b7c5ebc04c81926dfe079a298f780e7ce2d65b2e575a64031e6a659f98558` |
| **Audit Test Suite Source** | `adapters/chronosentiment/research/tests/rp608_audit_reconciliation_tests.rs` | `5927789e5f3011aa24d118031465a0e4a645ff25b65aa5afa5415c2960030f5b` |
| **Protocol Specification** | `docs/research/rp608_pvp_002_prospective_validation_protocol.md` | `d896bac0d1e91a17fabedf68e585d24e43184fccd4a7de74765c19add769d4b5` (Revision `RP-608-PVP-002-v1.0-FROZEN`) |

### 8.2 Immutable Prospective Ingestion Invariants

1. **Immutable Opportunity Ledger:**
   - Every eligible signal evaluated by the forward engine must be committed with an immutable opportunity ID, microsecond-accurate decision timestamp, authoritative reference price $P_{\text{ref}} = C_{-1}$, prior completed-bar inputs, calculated feature values, policy choices, simulated execution fills, and itemized friction.
   - Observations must be strictly append-only. No recorded observation may be retrospectively altered or deleted.
2. **Comparable Opportunity Denominator:**
   - All three policies (Policy 0, Policy 1, Policy 2) must be evaluated across the identical population of eligible opportunity IDs.
   - Signals filtered/skipped by Policy 1 remain in the denominator with $0.00\text{ bps}$ P&L and ₹0.00 friction to prevent survivorship or denominator bias.
3. **Pure Terminal Evaluation:**
   - The primary inferential statistics and decision gates will be computed exactly once, when the sample reaches both $S \ge 15$ sessions and $N \ge 500$ eligible opportunities.
   - Interim inferential looks and in-flight threshold modifications are strictly prohibited.
4. **Governance Separation:**
   - Satisfying all validation gates is a necessary condition for governance review, but does not grant automatic production authority. Live order authority remains frozen at **0.0%**.
   - No core crate (`coralys-*`) may be modified.

