# RP-608 Formal Activation Audit Manifest & Decision Record

**Document ID:** `RP-608-ACT-001`  
**Milestone:** RP-608 Final Prospective Activation Readiness Audit  
**Parent Protocols:** `RP-608-SHADOW-001B` & `RP-608-NSE-SHADOW-001`  
**Date / Timestamp (UTC):** `2026-10-25T06:35:00Z` (`2026-10-25T12:05:00+05:30`)  
**Audit Status:** **6 of 6 Checklist Items Verified Complete (READINESS AUDITED & LOCKED)**  
**Live Order Execution Authority:** Strictly **0.0%** (Read-Only Shadow Daemons; Zero Live Order Permissions)  
**Core Crate Modification:** Strictly **0.0%** (`coralys-moga`, `coralys-decision`, `coralys-ecology` diff = 0)  

---

## 1. Executive Summary & Activation Decision

This document records the formal activation audit for the prospective shadow observation programs `RP-608-SHADOW-001B` (Binance BTC/USDT) and `RP-608-NSE-SHADOW-001` (119 Liquid NSE Equity Universe). 

Following the successful execution of the 23-test research validation suite, verification of causal forecast-outcome alignment, inspection of feed quote-age semantics, cryptographic verification of champions and protocols, and enforcement of host-level process and ledger isolation:

> **Formal Activation Verdict:**  
> The prospective shadow system is hereby certified **READY FOR REGISTERED OBSERVATION WINDOW ACTIVATION**.  
> The system is currently executing in **`ACTIVE_PILOT`** buffer status.  
> Formal registered prospective observation clocks will activate strictly at:
> - **Binance BTC/USDT:** `2026-10-26T00:00:00.000Z` (Duration: 30 calendar days through `2026-11-25T00:00:00.000Z`).
> - **NSE Equity Universe (119 symbols):** `2026-10-26T03:45:00.000Z` / 09:15:00 IST (Duration: Exactly 20 qualifying trading sessions, governed by conditional close).

---

## 2. RP-608 Final Activation Checklist (6 of 6 Complete)

| Checklist Item | Scope & Target | Verification Method | Status |
| :--- | :--- | :--- | :--- |
| **Item 1** | Confirm all 23 reported tests pass in one reproducible run and capture complete logs | `cargo test --features research` (12 readiness + 5 tel001a + 6 invariants) | **COMPLETE (23/23 PASS)** |
| **Item 2** | Verify Brier forecasts, target labels, training baselines, and outcome timestamps are causally aligned | Causal inspection of Estimand A vs Estimand B definitions and training baselines | **COMPLETE (VERIFIED)** |
| **Item 3** | Verify live quote adapter against registered clock and execution-deadline rules | Inspection of quote transit formula ($\Delta t$), monotonic mapping, and 5000ms deadline | **COMPLETE (VERIFIED)** |
| **Item 4** | Verify champion file hashes and internal genome hashes at startup and shutdown | Verification of Policy B, Policy C, and dataset SHA-256 digests against locked constants | **COMPLETE (MATCHED)** |
| **Item 5** | Verify read-only credentials, zero order-submission capability, and separate ledgers for both daemons | Audit of public feed sockets, null trade API keys, and independent logging paths | **COMPLETE (ISOLATED)** |
| **Item 6** | Record the formal activation decision and preserve preregistered protocol hashes | SHA-256 digest preservation and state machine transition rules recorded | **COMPLETE (RECORDED)** |

---

## 3. Resolution of the Four Pre-Activation Methodological Issues

### 3.1 Issue 1: Brier Score Ex-Ante Forecast Verification (Estimand A vs Estimand B)
The audit verifies that Brier scores strictly evaluate forward-looking forecasts ($p_i$) issued at decision time $T_{\text{decision}}$, prior to outcome resolution ($T < T_{\text{outcome}}$), and maintains clean segregation between the two estimands:

1. **Estimand A: Trade-Outcome Prediction (Conditional on Entry):**
   - **Target:** $Y_{\text{trade}, i} = \mathbb{I}(R_{i, \text{gross}} > 0.00)$ for entered trades $i \in \{1, \dots, N\}$.
   - **Ex-Ante Probability:** $p_i = \mathbb{P}(\text{Win} \mid \text{Entry})$ assigned by the frozen rule surface at $T_{\text{decision}}$.
   - **Training Reference Prevalence Baseline:**
     - Policy B Champion: $\bar{y}_{\text{train}} = 0.6200 \implies \text{BS}_{\text{ref}} = 0.62 \times 0.38 = 0.2356$.
     - Policy C Champion: $\bar{y}_{\text{train}} = 0.6500 \implies \text{BS}_{\text{ref}} = 0.65 \times 0.35 = 0.2275$.
     - NSE Intraday Long: $\bar{y}_{\text{train}} = 0.5800 \implies \text{BS}_{\text{ref}} = 0.58 \times 0.42 = 0.2436$.
   - **Brier Skill Score:** $\text{BSS} = 1 - (\text{BS} / \text{BS}_{\text{ref}})$. Evaluates skill over historical strategy base rate.
   - **Balanced Accuracy Invariant:** Explicitly marked **`None` (N/A)** for Estimand A. Non-entries never enter the trade ledger; computing pseudo-balanced accuracy on trade wins/losses alone is methodologically invalid.

2. **Estimand B: Directional Opportunity Prediction (Fixed 4-Bar Horizon):**
   - **Target:** $D_{s, t} = \mathbb{I}(P_{s, t+4} > P_{s, t})$ across all 15-minute decision intervals for all universe assets.
   - **Forecast Classes & Normalization:**
     - Directional Long ($\hat{D} = 1$): $p(\text{Up}) = 0.62$ (Crypto) / $0.58$ (NSE); $p(\text{Not Up}) = 0.38$ / $0.42$.
     - Flat / No Entry ($\hat{D} = 0$): $p(\text{Up}) = 0.35$ (Crypto) / $0.38$ (NSE); $p(\text{Not Up}) = 0.65$ / $0.62$.
     - **Probabilistic Sum Constraint:** $\sum_{c \in \{\text{Up}, \text{Not Up}\}} p(c) = 1.00$ holds strictly across all decision states.
   - **Training Reference Prevalence Baseline:**
     - Unconditional training-period prevalence of positive 4-bar returns: $\bar{y}_{\text{train}} \approx 0.508$ (BTC) / $0.512$ (NSE) $\implies \text{BS}_{\text{ref}} \approx 0.2500$.
   - **Reported Confusion Metrics:** Full confusion matrix ($TP, FP, TN, FN$), Sensitivity, Specificity, and Balanced Accuracy $= (\text{Sensitivity} + \text{Specificity}) / 2$.

---

### 3.2 Issue 2: Effective Sample Size ($N_{\text{eff}}$) Alignment with Inferential Units
To prevent pooled cross-sectional trades from masquerading as hundreds of independent observations:

1. **Auto-Bandwidth & Truncation Safeguards:**
   - Bandwidth: $K = \min(\lfloor 4(N/100)^{2/9} \rfloor, K_{\max})$.
   - Geyer Initial Positive Sequence Truncation: Terminate autocovariance summation when $\hat{\Gamma}_m = \hat{\gamma}_{2m} + \hat{\gamma}_{2m+1} < 0$.
   - Variance Inflation Factor Floor: $\text{VIF} = \max(1.0, 1 + 2 \sum \hat{\rho}_k)$, guaranteeing $N_{\text{eff}} \le N$.

2. **NSE Session-Clustering Degrees of Freedom:**
   - Degrees of freedom for acceptance inference are strictly bounded by the number of independent qualifying sessions ($S \le 20$).
   - **Cluster Sufficiency Floor:** Any symbol or compartment cell with session cluster count $S < 10$ is assigned `InconclusiveInsufficientSample`, regardless of trade volume.
   - **Formal Acceptance Uncertainty:** Evaluated via the registered **session-block bootstrap** (resampling entire daily sessions with replacement), preserving intraday cross-symbol contemporaneous correlation.

3. **Crypto Time-Series Inferential Unit:**
   - For trade-level win rate, $N_{\text{eff}}$ is evaluated on the trade return sequence.
   - For prospective Sharpe and daily P&L acceptance, $N_{\text{eff}}$ is evaluated on the 30 daily return observations ($N = 30$).

---

### 3.3 Issue 3: Live Quote-Age Semantics & Execution Deadline Rules
The runtime audit verifies feed timestamp mechanics against physical exchange properties:

1. **Epoch & Unit Compatibility:**
   - Exchange timestamps ($T_{\text{event}}$) and local wall-clock timestamps ($T_{\text{local\_wall}}$) both use millisecond Unix epoch (`i64` UTC milliseconds since 1970-01-01).

2. **Clock Offset Sign Convention & Transit Decoupling:**
   $$\Delta_{\text{clock}} \equiv T_{\text{local\_wall}} - T_{\text{exchange}}$$
   $$\Delta t = T_{\text{local\_recv}} - T_{\text{quote\_event}} - \Delta_{\text{clock}}$$
   - Measurement uncertainty: $U = \text{RTT} / 2$.
   - **Freshness Boundary Guard:** Quotes approaching the 500 ms limit are tested conservatively: if $\Delta t + U > 500\text{ ms}$, the quote is discarded as stale.

3. **Monotonic Local Elapsed Timing:**
   - Decision timeout enforcement uses local monotonic hardware clock (`startup_local_monotonic_ns`), preventing NTP adjustments from causing false expirations or invalid timeouts.

4. **Execution Deadline & Retroactive Selection Prohibition:**
   - Ingestion Deadline: Candidate quote must arrive within $5{,}000\text{ ms}$ of $T_{\text{decision}}$. If no valid quote arrives within $5{,}000\text{ ms}$, the signal is logged in `OpportunityAccountingRegistry` as `ExecutionQuoteTimeout`.
   - **Causal Selection Invariant:** A quote arriving after $T_{\text{decision}} + 5{,}000\text{ ms}$ can never be retroactively selected for an earlier trade leg.

---

### 3.4 Issue 4: Protocol State Transition Machine & Conditional Close Primacy

1. **State Machine Definitions:**
   ```
   [PREFLIGHT_VERIFY]
          │
          ├── (Checksum or drift > 250ms fails) ──> [ABORTED_PREFLIGHT_FAIL] (Exit)
          ▼
   [ACTIVE_PILOT] (2026-10-25T03:00:00Z to 2026-10-26T00:00:00Z)
          │  - Socket stabilization & RTT calibration
          │  - Pilot log buffer (0 acceptance trades accrued)
          ▼
   [ACTIVE_OBSERVATION] (2026-10-26T00:00:00Z to 2026-11-25T00:00:00Z)
          │  - Formal 30-day / 20-session prospective evaluation
          │  - Zero parameter changes; zero code mutations
          ▼
   [OBSERVATION_TERMINAL_CLOSE]
             - Mandatory quad-report publication regardless of return sign
   ```

2. **NSE Conditional Close Primacy:**
   - The NSE observation duration is governed strictly by the **completion of the 20th qualifying trading session ($S = 20$)**.
   - The nominal calendar end of 20 November 2026 serves only as an unadjusted planning estimate. Any exchange holiday or truncated session automatically extends the observation window session-by-session until exactly 20 qualifying sessions have closed.

### 3.5 Final Pre-Activation Closure (The Three Narrow Safeguard Checks)

In final closure of the prospective audit prior to window launch, three narrow validity checks have been explicitly resolved and codified:

1. **Check 1: NSE Completed Bar Indexing Convention:**
   - The NSE equity trading session $[09:15:00, 15:30:00\text{ IST}]$ is partitioned into $K = 25$ non-overlapping 15-minute intervals, indexed 1-based by $k \in \{1, 2, \dots, 25\}$.
   - Every bar $k$ is indexed and emitted at its **closing timestamp** $T_{\text{close}}(k) = 09:15:00 + k\times 15\text{m}$.
   - Decision timestamp is $T_{\text{decision}}(k) \equiv T_{\text{close}}(k)$ with reference price $P_{\text{ref}}(k) \equiv P_{\text{close}}(k)$.
   - Fixed 4-bar forward outcome timestamp is $T_{\text{outcome}}(k) \equiv T_{\text{close}}(k+4) = T_{\text{close}}(k) + 60\text{ minutes}$ with outcome price $P_{\text{outcome}}(k) \equiv P_{\text{close}}(k+4)$.
   - **Same-Session Boundary Rule:** Estimand B intervals are eligible if and only if $T_{\text{outcome}}(k) \le 15:30:00\text{ IST}$ ($k+4 \le 25 \iff k \le 21$).
   - Bars 1 through 21 are eligible (21 bars/session $\times$ 5 sessions $\times$ 119 symbols = 12,495 eligible intervals in the pilot). Bars 22–25 are tagged `IntradayBoundaryExcluded`.
   - Distinct from Estimand A execution lifecycle where entries cease at Bar 20 open (14:00 IST) and open trades are squared off at Bar 24 close (15:15 IST).

2. **Check 2: Technical Quality Gates Interpretation (TG-5 and TG-6):**
   - **TG-5 (Sequence & Timestamp Monotonicity Gate):** Enforces strictly **zero sequence ID inversions (0.00% tolerated)** and **zero exchange event timestamp inversions (0.00% tolerated)**. The previously ambiguous 0.10% tolerance is eliminated; multi-socket network arrival jitter is absorbed by deterministic reordering buffers prior to quote matching.
   - **TG-6 (Comprehensive Cost Ledger & Incremental Friction Gate):** Total realized cost accounts for all four distinct components:
     $$\text{Friction}_{\text{total}} = \text{Fees}_{\text{taker}} + \text{Half-Spread Crossing} + \text{Slippage}_{\text{exogenous}} + \text{Market Impact}$$
     in quote currency (USDT / INR) and turnover basis points. Incremental friction drag $\Delta_{\text{incremental}} \equiv \text{Friction}_{\text{total}} - (\text{Fees} + \text{Slippage}) = \text{Half-Spread} + \text{Impact}$ satisfies $| \Delta - (\text{Spread} + \text{Impact}) | < 10^{-4}$. Fees and exogenous slippage remain independently audited and reconciled on every fill ledger record.

3. **Check 3: Implementation & Invariant Verification:**
   - **Identical Observation Sets:** Both reference and policy Brier scores evaluate the exact same $N$ prospective observations ($\text{BS} = \frac{1}{N}\sum (p_i - y_i)^2$ vs $\text{BS}_{\text{ref}} = \frac{1}{N}\sum (y_i - p_{\text{train}})^2$).
   - **Causal Logging Guarantee:** Forecast probabilities $p_i$ are recorded strictly at $T_{\text{decision}} < T_{\text{outcome}}$ before trade fills or forward returns resolve.
   - **Protocol Parity:** The production evaluation code enforces identical targets ($D_{s, k} = \mathbb{I}(P_{k+4} > P_k)$ for Estimand B; $Y_i = \mathbb{I}(R_{i, \text{gross}} > 0.0)$ for Estimand A), training prevalence ($0.508$ / $0.512$ for Estimand B; $0.62$ / $0.65$ / $0.58$ for Estimand A), and boundary exclusion rules.

---

## 4. Cryptographic Hash Locks & Provenance Manifest

### 4.1 Policy Champion & Dataset Hashes
```
Frozen Champion Policy B Artifact (JSON):
SHA-256: f7be46ea66ff6a248ad0477dfe5b082ce26fe01f9dded928b164bdaa349c6aa3
Internal RuleListGenome Hash:
SHA-256: 6a0e39600389ed489b95153498eaf244dbc75864b9259ee70d6e07211eb30ddf

Frozen Champion Policy C Artifact (JSON):
SHA-256: cd615f37b812a34fe302a0b05bc1546a0243f3d71d8947092d28d2847159553b
Internal RuleListGenome Hash:
SHA-256: 80c689bae15e5fe9ec3c525cf0dc37bdaad17d4b896faab91beacd0b8fd6de05

BTC/USDT 2024 Reference Dataset (JSON):
SHA-256: 0fcc08c45f5871c77b6ee83227823224670f7f945e650fe2175cd3c8fb86be7d
```

### 4.2 Preserved Preregistered Protocol & Amendment Hashes
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

Authoritative Protocol Amendment:
docs/research/rp608_dfp_001_amendment_01.md
SHA-256: c9467df6f9fa1b715c50b576f1af4e79716c86d7c8765db75f5fbe552a9f5202
```

---

## 5. Test Suite Verification Log Summary

- **Total Research Test Targets:** 3 test files, 23 tests total.
- **Passing Tests:** **23 passed, 0 failed, 0 ignored**.
- **Execution Log Path:** `rp608_test_execution_manifest.log`.
- **Breakdown:**
  1. `rp608_shadow_readiness_tests.rs`: 12 passed
     - `test_clock_synchronization_and_monotonic_mapping`
     - `test_exchange_clock_offset_and_transit_latency_decoupling`
     - `test_clock_offset_sign_convention_and_drift_scenarios`
     - `test_prospective_acceptance_criteria_defaults`
     - `test_independent_data_quality_denominators`
     - `test_quality_gate_consecutive_vs_isolated_missing_bars`
     - `test_market_compartment_classification_causal_precedence`
     - `test_tripartite_prospective_evaluation_layers`
     - `test_unexecutable_opportunity_accounting_safeguard`
     - `test_effective_sample_size_safeguards_and_brier_training_baseline`
     - `test_terminal_closeout_at_window_boundary`
     - `test_compartmentalized_prospective_evaluation_and_ci`
  2. `rp608_tel001a_reconciliation_tests.rs`: 5 passed
     - `test_round_trip_vs_one_way_rate_bridge`
     - `test_dual_hash_identity_proof`
     - `test_data_provenance_proxy_classification`
     - `test_fill_by_fill_zero_tolerance_reconciliation`
     - `test_impact_denominator_isolation`
  3. `rp608_telemetry_invariants_tests.rs`: 6 passed
     - `test_displayed_depth_as_estimate_not_guarantee`
     - `test_spread_slippage_no_double_counting`
     - `test_timestamp_integrity_and_stale_rejection`
     - `test_quote_to_execution_alignment`
     - `test_data_quality_gates_precheck`
     - `test_read_only_isolation_and_checksum_verification`

---

## 6. Host & Process Isolation Boundaries

1. **Zero Order Execution Capability:**
   - Both daemons connect strictly to public L1 market feeds.
   - Zero private API keys (`BINANCE_API_SECRET`, brokerage trade credentials) are present in the environment or host configuration.
   - Telemetry engines possess zero order submission or order cancellation endpoints.

2. **Decoupled Ledgers on VM Host:**
   - Binance Crypto Shadow Ledger: `/var/log/chronosentiment/crypto/` (`live_capture/shadow_validation/crypto/`)
   - NSE Equity Shadow Ledger: `/var/log/chronosentiment/nse/` (`live_capture/shadow_validation/nse/`)
   - Independent daemon process IDs, distinct output streams, and decoupled post-window report generators.

---

## 7. Sign-Off & Activation Certification

With all 6 items of the RP-608 Final Activation Checklist verified, all 4 methodological issues resolved, core crates frozen at zero diff, and protocol hashes locked, the prospective observation environment is formally certified:

$$\text{Final Audit Verdict: APPROVED FOR PROSPECTIVE ACTIVATION}$$

*Signed and sealed into the repository record.*  
*Activation Clock: ACTIVE_PILOT $\longrightarrow$ ACTIVE_OBSERVATION at 2026-10-26T00:00:00Z.*
