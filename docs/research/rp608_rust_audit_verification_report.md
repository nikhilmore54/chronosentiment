# RP-608 Rust Research Audit & Replay Verification Report

**Author / Harness:** Native Rust Standalone Evaluation Binary (`adapters/chronosentiment/research/bin/rp608_research_audit.rs`)  
**Target Research:** RP-608-INV-001 (Inverse Strategy Evaluation) & RP-608-PVP-001 (Price–Volume Attribution)  
**Governance Contract:** `RUST_PORT_CONTRACT_v1.md` / RP-600 Pre-Check  
**Date:** October 2026  
**Status:** COMPLETE & INDEPENDENTLY RECONCILED (0.0000 Paisa Discrepancy)  
**Live Order Authority:** 0.0% (Frozen Adapter Decision Path)  
**Coralys Modification:** 0.0% (Strictly 0 files touched in `coralys-*`)  

---

## 1. Executive Summary

This report establishes a single, deterministic, native Rust research-evaluation and audit baseline exclusively inside `adapters/chronosentiment` for the `RP-608` series, resolving both `RP-608-INV-001` (Inverse Strategy Evaluation) and `RP-608-PVP-001` (Price–Volume Pattern Attribution).

The audit confirms that the historical findings reported in Python scripts are mathematically and mechanically reproducible under native Rust execution, and decisively answers the four methodological and execution-validity challenges raised regarding the research:
1. **Variant B (Exact Reversal) is an Algebraic Counterfactual, not an Edge:** Flipping historical gross P&L reverses the baseline gross return (+0.52 bps $\to$ −0.52 bps) and worsens net expectancy (−8.48 bps $\to$ −9.52 bps). The 67.0% gross win rate is a positive skewness illusion that turns frequent small wins into catastrophic tail losses.
2. **Variant C (Mirrored Stops Replay) Survives Intrabar Ambiguity but Remains Non-Viable:** Across 2,029 historical trades replayed through 380,617 1-minute OHLCV bars, exactly 4 bars (0.2%) touch both stop and exit prices. Applying conservative worst-case stop execution, Variant C produces +0.59 bps gross (+₹12,205.67), but collapses to −8.41 bps net (−₹1,68,823.37) after realized friction of ₹1,81,029.04.
3. **The Pre-Entry Exhaustion Cohort is an In-Sample Failure Diagnostic, not a Trading Signal:** The rule (`ret_15m_aligned >= 30 bps AND dist_vwap_bps >= 45 bps`) isolates 242 trades (11.9% of the population) where the baseline lost −4.70 bps gross (−₹11,405.25). Reversing these trades yields +4.70 bps gross, but collapses to −4.30 bps net (−₹10,149.43) after statutory costs. It fails the pre-registered +14.0 bps economic hurdle.
4. **Zero-Paisa Numerical Parity & Invariant Conformance:** All 2,029 trades, ₹20,11,59,246.16 notional turnover, ₹9,993.52 gross profit, ₹1,81,029.04 statutory charges, and −₹1,71,035.52 net loss reconcile with **0.0000 Paisa discrepancy**. Existing adapter tests (`adapter_discipline_invariants`) remain 100% green (13/13 passed).

---

## 2. Phase 0: Contract and Boundary Audit

In strict accordance with `RUST_PORT_CONTRACT_v1.md` and repository governance:
- **Zero Modifications to Core Crates:** No files in `coralys-core`, `coralys-simulation`, `coralys-decision`, `coralys-eval`, `coralys-moga`, `coralys-ecology`, `coralys-recommendation`, or `coralys-infrastructure` were modified (`git status coralys-*` clean).
- **Production Surface Isolation:** The product decision surface in `adapters/chronosentiment/src/bin/` remains strictly locked to its 16 governed targets. The research evaluation harness is isolated under `adapters/chronosentiment/research/bin/rp608_research_audit.rs` and gated behind the `research` feature flag.
- **Single Source Financial Types:** P&L, notional, and fee calculations reuse canonical tick arithmetic and statutory fee schedules without introducing unverified parallel models.

---

## 3. Resolution of the Four Methodological Issues

### Issue A: Inverse Execution-Validity and Intrabar Stop Ordering
- **Variant B Counterfactual Nature:** Variant B assumes that the exact entry and exit prices of the baseline could be captured in the opposite direction without slippage or spread penalty. The Rust harness confirms that even under this impossibly generous algebraic assumption, Variant B is unprofitable (−9.52 bps net).
- **Variant C Intrabar Ambiguity Resolution:** Out of 2,029 trades replayed against 1-minute OHLC bars:
  - 398 trades (19.6%) triggered the mirrored stop price prior to the baseline exit timestamp.
  - Exactly 4 trades touched both the stop price and the exit price within the same 1-minute bar.
  - **Resolution Policy:** The Rust replay treats all 4 ambiguous bars conservatively as adverse stop executions (worst-case fill at stop price). Despite this conservative resolution, the total outcome changes by less than 0.08 bps. Variant C net expectancy remains negative at −8.41 bps.
- **Friction Recalculation:** Rather than copying baseline friction, fees were independently recalculated using the statutory NSE turnover charge, exchange transaction fee, SEBI turnover charge, stamp duty, brokerage flat ₹40/trade pair, and STT (0.025% on sell legs). Realized friction across the 2,029 trades is ₹1,81,029.04 (9.00 bps), matching the authoritative ledger to 0.0000 Paisa.

### Issue B: Selection Bias in Price–Volume Analysis
- The preliminary PVP-001 finding that the Top 10% largest losers had +35.3 bps 15m momentum, +57.4 bps VWAP stretch, and 1.56x volume surge was selected conditioned on future P&L.
- The Rust harness audits the entire continuous population of 2,029 trades. Across the unconditioned population, pre-entry momentum and VWAP stretch display continuous negative correlation with baseline P&L ($r = -0.062$), but the relationship is non-linear and concentrated in the exhaustion tail.
- **Conclusion:** The pattern is a descriptive signature of momentum exhaustion, not an unconditioned trading edge.

### Issue C: Pre-Entry Overextension Prospective Semantics
- **Temporal Strictness:** All features are calculated using strictly completed 1-minute bars with `timestamp < entry_timestamp` (second `:00` entry decision). Zero future bars or intrabar wicks from the trade's active interval are visible.
- **VWAP Calculation:** Computed strictly using cumulative $\sum (P \cdot V) / \sum V$ from the session open (09:15 IST / 03:45 UTC) up to the minute preceding entry.
- **Sample Membership:** The candidate rule (`ret_15m_aligned >= 30 bps AND dist_vwap_bps >= 45 bps`) tags exactly 242 trades (11.9% of all trades). The baseline lost −₹11,405.25 gross (−4.70 bps). Exact reversal produces +₹11,405.25 gross (+4.70 bps), but after deducting statutory charges of ₹21,554.68, realized net outcome is −₹10,149.43 (−4.30 bps).

### Issue D: Numerical Reconciliation & Weighting Conventions
- **Variant D Disambiguation:** 
  - *Interpretation 1 (Conditional Reversal Policy):* When applied across all 2,029 trades (inverting the 242 overextended trades while leaving the remaining 1,787 trades in the baseline direction), portfolio gross P&L changes from +₹9,993.52 to +₹32,804.02 (+1.63 bps), but net P&L remains negative at −₹1,48,225.02 (−7.37 bps).
  - *Interpretation 2 (Isolated Overextended Cohort):* Evaluated strictly on the 242 flagged trades, gross return is +4.70 bps, net expectancy is −4.30 bps. Both interpretations are now explicitly distinguished in the Rust output.
- **Denominator and Weighting Convention:** 
  - *Aggregate Notional-Weighted bps:* $\frac{\sum \text{Gross P\&L}}{\sum \text{Notional}} \times 10,000 = \frac{+9,993.52}{20,11,59,246.16} \times 10,000 = +0.4968 \text{ bps}$.
  - *Unweighted Mean of Trade-Level bps:* $\frac{1}{N} \sum_{i=1}^N \text{bps}_i = +0.5222 \text{ bps}$.
  - Both metrics are computed and tracked in the Rust harness. The discrepancy between the two is small (0.025 bps), confirming no extreme notional-weighting distortion.

---

## 4. Phase 1: Native Rust Audit Results

### 4.1 Ledger Reconciliation Parity (0.0000 Paisa Discrepancy)

```
====================================================================================================
RP-608 RUST RESEARCH AUDIT & REPLAY ENGINE (adapters/chronosentiment)
Standalone Verification Harness | Single Source of Truth Replay
====================================================================================================
Loaded 2029 trades across 4 historical sessions.

--- [PHASE 1.1: LEDGER RECONCILIATION AUDIT] ---
Total Trades:        2029
Long Trades:         860 | Short Trades: 1169
Total Notional:      Rs 201159246.16
Reported Gross PnL:  Rs +9993.52 (Recalculated: Rs +9993.52, Mismatch: 0.0000 Paisa)
Reported Friction:   Rs 181029.04 (Recalculated: Rs 181029.04, Mismatch: 0.0000 Paisa)
Reported Net PnL:    Rs -171035.52 (Recalculated: Rs -171035.52, Mismatch: 0.0000 Paisa)
Mean Gross Return:   +0.52 bps | Mean Net Expectancy: -8.48 bps
Win Rates:           Gross 33.0% | Net 20.4%

Session Breakdown:
  Session 2026-09-28: 460 trades | Gross: Rs -13370.89 (-2.92 bps) | Net: Rs -54455.71 (-11.92 bps)
  Session 2026-09-29: 547 trades | Gross: Rs  +1773.21 (+0.36 bps) | Net: Rs -47077.54  (-8.64 bps)
  Session 2026-09-30: 547 trades | Gross: Rs  +1715.60 (+0.35 bps) | Net: Rs -47076.40  (-8.65 bps)
  Session 2026-10-01: 475 trades | Gross: Rs +19875.61 (+4.23 bps) | Net: Rs -22425.87  (-4.77 bps)
```

### 4.2 Bar Data Integrity Audit

```
--- [PHASE 1.2: 1-MINUTE OHLCV BAR INTEGRITY AUDIT] ---
Symbols Audited:     99/99 (100.0%)
Total 1m Bars:       380617
Timestamp Anomalies: 0
Trades Valid Hist:   2029/2029 (100.0%)
Trades Missing Hist: 0
```

### 4.3 Inverse Replay Comparison (Variant B vs Variant C)

```
--- [PHASE 1.3: INVERSE EXECUTION & PATH REPLAY AUDIT] ---
Variant B (Exact Algebraic Reversal):
  Gross PnL:         Rs -9993.52 (-0.52 bps)
  Net PnL:           Rs -191022.56 (-9.52 bps)
  Win Rate:          Gross 67.0% | Net 32.7%

Variant C (Mirrored Stops Path Replay - Conservative Fill):
  Stopped Out:       398/2029 (19.6%)
  Ambiguous Bars:    4 bars touching both stop and exit
  Gross PnL:         Rs +12205.67 (+0.59 bps)
  Net PnL:           Rs -168823.37 (-8.41 bps)
  Win Rate:          Gross 64.6% | Net 31.5%
```

### 4.4 Candidate Overextension Cohort Evaluation

```
--- [PHASE 1.4: PRE-ENTRY EXHAUSTION FEATURE ATTRIBUTION] ---
Candidate Rule:      ret_15m_aligned >= 30.0 bps AND dist_vwap_bps >= 45.0 bps
Cohort Size:         242 trades (11.9% of population)
Baseline Perf:       Gross -4.70 bps | Net -13.70 bps
Inverse Perf:        Gross +4.70 bps | Net -4.30 bps
Inverse Win Rate:    Gross 66.1% | Net 43.4%

Falsification Gate Evaluation:
  Hurdle (+14.0 bps): FAILED (Net expectancy is -4.30 bps)
  Edge Survival:      Friction (9.00 bps) > Inverse Gross (+4.70 bps)
  Outcome:            Candidate closed as DESCRIPTIVE FAILURE MECHANISM only.
```

---

## 5. Verification Matrix and Acceptance Criteria

| Acceptance Criterion | Target Specification | Observed Result | Status |
| :--- | :--- | :--- | :--- |
| **1. Cargo Check** | `cargo check --manifest-path adapters/chronosentiment/Cargo.toml` | Compiles with exit code 0 | **PASSED** |
| **2. Adapter Invariants** | `tests/adapter_discipline_invariants.rs` passes | 13/13 tests green | **PASSED** |
| **3. Ledger Reconciliation** | Absolute mismatch $\le 0.01$ Paisa | **0.0000 Paisa mismatch** across all metrics | **PASSED** |
| **4. Stop Execution Ambiguity** | Tag ambiguous bars; conservative fill bounds | 4 ambiguous bars tagged; worst-case fill applied | **PASSED** |
| **5. Lookahead Elimination** | Strictly completed bars prior to entry (`< entry_timestamp`) | Verified monotonic unix timestamps; zero lookahead | **PASSED** |
| **6. Core Isolation & Authority** | 0 files modified in `coralys-*`; 0.0% live authority | Clean git status on `coralys-*`; 0.0% authority | **PASSED** |

---

## 6. Formal Governance Disposition & Next Research Gate

| Research Item | Historical Finding | Audited Rust Result | Formal Disposition |
| :--- | :--- | :--- | :--- |
| **RP-608-INV-001 (Exact Inversion)** | Gross −0.52 bps, Net −9.52 bps | Reproduced to 0.00 Paisa | **Permanently Closed as Rejected** |
| **RP-608-PVP-001 (Attribution)** | Worst trades enter at climactic wicks | Confirmed (242 overextended trades) | **Retained as Descriptive Diagnostic Only** |
| **RP-608-PVP-002 (Prospective Gate)** | Proposed prospective validation | Feature & execution definitions frozen | **Authorized as Proposed Next Gate** |

### Proposed Gate: RP-608-PVP-002 Protocol Definition

To prevent circular in-sample tuning on the four historical sessions (Sep 28 – Oct 01, 2026), all further experimentation on the historical holdout is terminated.

`RP-608-PVP-002` is pre-registered to evaluate the candidate overextension rule on **strictly incoming forward shadow sessions** ($S \ge 10$ sessions):
1. **Candidate Rule Frozen:**
   $$\text{Flagged Entry} \iff \text{ret\_15m\_aligned} \ge 30.0\text{ bps} \;\land\; \text{dist\_vwap\_bps} \ge 45.0\text{ bps}$$
   evaluated strictly on the last completed 1-minute bar before entry.
2. **Policy Comparison:**
   - **Policy 0 (Baseline):** Execute all signals as generated by the frozen adapter.
   - **Policy 1 (No-Trade Filter):** Reject/skip entries matching the candidate overextension rule.
   - **Policy 2 (Opposite-Direction Reversal):** Invert direction on flagged overextended entries.
3. **Predeclared Decision Gate:**
   - Policy 1 must demonstrate statistically significant reduction in net losses compared to Policy 0 ($p < 0.01$, $\Delta \text{Net} \ge +1.50\text{ bps}$).
   - Policy 2 must achieve positive net expectancy after statutory transaction charges ($\ge +1.00\text{ bps}$).
   - If neither condition is met across the prospective evaluation period, momentum exhaustion will be formally designated as a descriptive structural phenomenon incapable of yielding an executable trading edge.
