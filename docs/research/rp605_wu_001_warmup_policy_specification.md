# RP-605-WU-001: Counterfactual Warm-Up Policy Specification

**Document Identifier**: `RP-605-WU-001`  
**Author**: Antigravity AI Pair Programmer & Research Team  
**Date**: 2026-10-22  
**Status**: PRE-REGISTERED SPECIFICATION (Shadow Counterfactual Protocol)  
**Governance Authority**: Strictly **0% Live Order Authority**  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants)

---

## 1. Executive Summary & Problem Formulation

In the empirical audit of the October 22, 2026 live NSE cash market session (`Tick #54` checkpoint), an analytical trade-level reconciliation demonstrated two critical findings:
1. **Decay Blocking Works**: The frozen freshness rule ($F_{15} \ge -0.0100$) successfully blocked **103 decaying candidate entries** ($F_{15} < -0.0100$), which under Baseline A incurred **₹-5,417.37 in losses** (win rate only 20.4%). Blocking these trades accounted for the entire net performance advantage of Ledger B.
2. **Cold-Start Weakness**: In the early morning session (Ticks #1 and #2 at 09:15–09:16 AM IST), $N=1$ observations forced $\text{peak}_{15} \equiv a_{15, t_0} \implies F_{15} = 0.0000$. Because $0.0000 \ge -0.0100$ trivially satisfies the gate, **218 out of 221 completed trades (98.6%)** entered under cold-start conditions with zero prior trajectory evidence, achieving a win rate of only **27.1% (59W / 159L)** and Realized P&L of **₹-568.17**.

This specification formalizes **Ledger C (Warmup-Qualified)** as a separate, pre-registered counterfactual evaluation policy in shadow mode to determine whether enforcing a minimum trajectory history before admitting opening trades improves session expectancy.

---

## 2. Three-Ledger Semantic Architecture

The shadow validation runner shall evaluate three parallel ledgers on the exact same underlying opportunity stream:

| Ledger | Governing Policy | Semantics & Entry Invariant |
| :--- | :--- | :--- |
| **Ledger A** | Baseline Unconstrained | Admitted if signal fires, regardless of lookback history length or $F_{15}$ score. Serves as control. |
| **Ledger B** | Frozen Freshness Gate | Admitted if $F_{15} \ge -0.0100$. Fallback score $F_{15} = 0.0000$ passes trivially when $N < 2$. |
| **Ledger C** | Proposed Warm-Up Policy | Admitted if and only if **both** Warm-Up Criteria ($N \ge N_{\text{min}}$) **AND** Freshness Gate ($F_{15} \ge -0.0100$) are satisfied. Fallback $F_{15} = 0.0$ is strictly disallowed. |

---

## 3. Candidate Warm-Up Policy Definitions

To avoid conflating distinct timing concepts, two separate candidate warm-up rules are formalized:

### Candidate Policy C-1: $N_{\text{warmup}} \ge 3\text{ Ticks}$
- **Definition**: Requires at least 3 distinct 1-minute price observations ($t_0, t_1, t_2$) before admitting a position.
- **Earliest Admissible Tick**: Tick #3 (09:17 AM IST).
- **Trajectory Metric**: $F_{15}$ computed over $N \in [3, 14]$ bars.
- **Operational Hypothesis**: Catches initial opening gap decay within 2 minutes of cash open, preventing immediate entry on gap-and-fade traps.

### Candidate Policy C-2: Full 15-Minute History Window ($N_{\text{warmup}} \ge 15\text{ Ticks}$)
- **Definition**: Requires a complete 15-minute lookback window before any trade admission.
- **Earliest Admissible Tick**: Tick #15 (09:30 AM IST).
- **Trajectory Metric**: $F_{15}$ computed strictly over a full 15-bar trajectory window ($N \ge 15$).
- **Operational Hypothesis**: Completely eliminates all partial-history approximations; trades are admitted only after the initial opening price discovery regime settles.

---

## 4. Counterfactual Recording & Attribution Rules

1. **Exact Opportunity Set Matching**:
   Every prospective candidate trade emitted by the decision loop is logged with:
   - `symbol`
   - `entry_timestamp`
   - `entry_price`
   - `quantity`
   - `history_length_n`
   - `computed_f15`
   - `admitted_in_ledger_a`: `bool`
   - `admitted_in_ledger_b`: `bool`
   - `admitted_in_ledger_c1`: `bool`
   - `admitted_in_ledger_c2`: `bool`
2. **Zero Hypothetical Loss Assignment**:
   A rejected candidate under Ledger B or Ledger C shall **never** be assigned an arbitrary loss. Instead:
   - The rejected position's counterfactual exit is tracked using the exact same exit logic (60m target exit, 180m max exit, or signal flip) against actual exchange tick data.
   - Its realized counterfactual P&L is logged in `rp605_nse_rejected_opportunities.jsonl`.
3. **P&L Reconciliation Invariant**:
   For any evaluation period:
   $$\text{PnL}_A = \text{PnL}_{\text{Matched}(A, C)} + \text{PnL}_{\text{BlockedByC}}$$
   The causal advantage of Ledger C over Ledger A must equal the negative sum of counterfactual P&L on the blocked trades.

---

## 5. Decision Gates & Pre-Registered Statistical Protocol

To ensure robust generalisation and prevent spurious claims from correlated intraday observations:

### 1. Pre-Registered Statistical Methodology
- **Primary Outcome**: Matched Net Session P&L Delta ($\Delta = \text{PnL}_C - \text{PnL}_A$) and Mean Trade Expectancy.
- **Resampling Unit**: **Session-Block / Symbol Cluster Bootstrap** ($B = 10,000$ resamples). Individual trades within the same session/symbol are clustered to preserve the empirical dependence structure.
- **Multiple Policy Correction**: Bonferroni-Holm adjustment across Candidate C-1 ($N_{\text{warmup}} \ge 3$) and Candidate C-2 ($N_{\text{warmup}} \ge 15$) to control Family-Wise Error Rate at $\alpha = 0.05$.
- **Stopping Rule**: Fixed sample horizon of at least 5 consecutive cash market sessions and $N \ge 100$ completed trajectory-qualified exits. Early stopping on transient gains is strictly prohibited.

### 2. Decision Gates

| Gate | Requirement | Metric Threshold | Statistical Protocol |
| :--- | :--- | :--- | :--- |
| **Gate WU-1: Sample Floor** | Evidence floor across sessions | $N_{\text{warm}} \ge 100$ completed exits across $\ge 5$ distinct sessions | Minimum sample requirement before hypothesis testing |
| **Gate WU-2: Cohort Alpha** | Expectancy of Warmed-Up vs. Cold-Start | $E[\text{PnL}_{\text{Warm}}] > E[\text{PnL}_{\text{Cold}}]$ | Cluster bootstrap 95% CI lower bound $> 0$ |
| **Gate WU-3: Net Session Advantage** | Ledger C net session P&L vs Ledger A & B | Ledger C improves net P&L after accounting for all excluded winning trades | Session-block bootstrap test ($p < 0.025$ adjusted) |
| **Gate WU-4: Zero Adapter Drift** | Repository Governance | `adapters/chronosentiment` remains 100% frozen | 13/13 discipline tests passing |

---

## 6. Implementation Guardrails

- The warm-up evaluation runner shall be implemented in an external shadow script (`scripts/run_rp605_nse_warmup_counterfactual.py`) or as an extension to `run_rp605_nse_prospective_freshness_validation.py`.
- **Zero changes to `adapters/chronosentiment` are permitted.**
