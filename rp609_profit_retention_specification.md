# RP-609 — Profit Retention and Giveback Control
## Formal Research Specification · Version 1.0 · Canonical Freeze

**Status**: SPECIFIED / GATED (POST-RP-605, RP-606 & RP-607)  
**Governance Standard**: RP-600 Pre-Check Verified | Execution-Layer Policy Research  
**Target Mechanism**: Unrealized Profit Giveback Reduction, Peak-Profit Trailing Protection & Loss-Reversal Prevention  
**Primary Method**: Matched Counterfactual Shadow-Ledger Evaluation  
**Control**: RP-605 Control Baseline — Frozen Ledger A  
**Execution Authority**: None (Shadow-Only Evaluation)  

---

## 1. Executive Summary

RP-609 investigates a distinct operational failure: a trade may accumulate positive unrealized P&L but subsequently surrender some or all of that profit before its exit. In some cases, the trade ultimately closes at a loss.

The project compares four candidate mechanisms independently:
1. **Peak-profit trailing boundary (PFT)**
2. **Stepwise profit locking (SPL)**
3. **Volatility-adjusted trailing boundary (VAT)**
4. **Partial profit realization (PPR)**

The study first establishes the canonical baseline distribution of profit accumulation and giveback (Deliverable RP-609-001), then evaluates each mechanism under matched entry, market-data and cost assumptions. A combination of mechanisms may be considered only in a separately specified research phase after the individual mechanisms have been evaluated.

RP-609 is not an attempt to improve Coralys signal generation or to establish a new entry filter. It is an investigation into the management of positions after entry.

---

## 2. Research Question

### MRQ-609 (Main Research Question)
> *Can a pre-specified, independently evaluated trade-management mechanism improve realized trade outcomes by retaining accumulated unrealized profit, without imposing an unacceptable reduction in favourable continuation or increasing execution risk?*

### 2.1 Supporting Research Questions
- **RQ-609.1 — Baseline Characterization**: How frequently do trades enter positive unrealized P&L, and how much of their maximum favourable excursion is retained at exit?
- **RQ-609.2 — Mechanism Effect**: How does each candidate mechanism change realized P&L, profit capture, loss conversion and holding duration relative to the frozen baseline?
- **RQ-609.3 — Opportunity Cost**: How much additional favourable movement is missed when a candidate mechanism exits earlier than the baseline?
- **RQ-609.4 — Robustness**: Are observed effects consistent across market classes, instruments, trade directions, volatility conditions and time periods?
- **RQ-609.5 — Execution Integrity**: Are measured improvements retained after transaction costs, price gaps, slippage assumptions and partial-position accounting?
- **RQ-609.6 — Prospective Replication**: Do the results persist on untouched prospective observations under a frozen candidate specification?

---

## 3. Governance and Architectural Boundaries

### Non-Negotiable Constraints
- **RP-605 Control Baseline**: RP-605 Ledger A is the frozen control population. Its entry, exit, boundary-management and accounting rules remain unchanged throughout RP-609. All candidate mechanisms operate exclusively in independent shadow ledgers and have no authority to modify the control baseline or live execution.
- **RP-607 Q3 Status Integrity**: RP-607 remains governed by its existing Q3 discrimination specification. Its current status must be reported accurately, and RP-609 does not assume that Q3 has qualified or treat a pending status as authorization to alter the engine.
- **RP-608 Shadow Isolation**: RP-608 remains an isolated shadow policy. RP-609 may reuse compatible boundary-management infrastructure, but it must not alter RP-608's policy, results or execution authority.
- **Coralys Core Invariant**: Coralys Core v0.1.0, Config A, score formulas, thresholds and signal semantics remain 100% frozen.
- **Read-Only Execution Authority**: All RP-609 candidate decisions are read-only with respect to RP-605 and the live trading system. Zero live execution authority is granted.
- **No Automatic Promotion**: No candidate may be promoted to live execution based solely on historical results, shadow-ledger profitability or a favourable aggregate metric.

### 3.1 Separation of Responsibilities

```text
                 Frozen Coralys Core
                 Config A / Entry
                        │
                        ▼
               RP-605 Ledger A
                Frozen control
                        │
           ┌────────────┴────────────┐
           │                         │
           ▼                         ▼
      RP-606 / RP-607            RP-609
      Observation and            Candidate
      discrimination             evaluation
           │                         │
           │                 ┌───────┼────────┐
           │                 │       │        │
           │                PFT     SPL      VAT
           │                         │        │
           │                    Partial      │
           │                    realization  │
           │                         │        │
           │                 Independent shadow
           │                     ledgers
           │                         │
           └─────────────────────────▼
                           Reconciliation
                           and validation
```

---

## 4. Population Reconciliation & Reconciled Baseline Diagnostic Matrix (RP-609-001)

### 4.1 Population Identity Accounting
- **Canonical Composite Trade Key**: `(symbol, side, t_entry, trade_id)`
- **Primary Control Baseline Population (RP-605 Ledger A)**: **982 unique composite trade identities** (**870 NSE Equities**, **112 Crypto Assets**).
- **Freshness Observer Population (Ledger B)**: **569 matched sub-population** (**502 NSE Equities**, **67 Crypto Assets**). Overlaps 100% with Ledger A trade identities ($N_{\text{overlap}}=569$).
- **Unique Trade Identity Population ($N_{\text{union}}$)**: **982 unique trades**.
- **Record Evaluations Count**: **1,551 ledger evaluations** across baseline and observer streams. Combined A+B rows represent duplicated evaluation diagnostics, NOT unique-trade totals.

### 4.2 Reconciled Baseline Diagnostic Matrix

| Session / Population Unit | Currency | Total Exits ($N$) | Unique Composite IDs | Trades with MFE > 0 ($N, \% $) | Reversals to Loss ($N, \% $) | Total Peak MFE PnL | Total Realized PnL | Total MFE Giveback | Reversal Opportunity Loss |
| :--- | :---: | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary Control Baseline (NSE Ledger A)** | `₹` | `870` | `870` | `535 (61.5%)` | `264 (49.3%)` | `₹82,914.55` | `-₹17,263.27` | **`₹46,401.63`** | **`₹20,404.03`** |
| **Freshness Observer (NSE Ledger B)** | `₹` | `502` | `502 (100% Matched Overlap)` | `300 (59.8%)` | `161 (53.7%)` | `₹47,965.79` | `-₹12,942.87` | `₹27,688.13` | `₹12,946.87` |
| *Duplicated Record Evaluations (NSE)* | `₹` | `1,372` | `870 Unique IDs` | `835 (60.9%)` | `425 (50.9%)` | `₹130,880.34` | `-₹30,206.14` | `₹74,089.76` | `₹33,350.90` |
| :--- | :---: | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary Control Baseline (Crypto Ledger A)** | `$` | `112` | `112` | `80 (71.4%)` | `44 (55.0%)` | `$34.30` | `-$10.38` | **`$24.50`** | **`$10.49`** |
| **Freshness Observer (Crypto Ledger B)** | `$` | `67` | `67 (100% Matched Overlap)` | `49 (73.1%)` | `28 (57.1%)` | `$19.92` | `-$6.89` | `$13.54` | `$5.01` |
| *Duplicated Record Evaluations (Crypto)* | `$` | `179` | `112 Unique IDs` | `129 (72.1%)` | `72 (55.8%)` | `$54.22` | `-$17.27` | `$38.04` | `$15.50` |

### 4.3 Reconciled Diagnostic Metrics & Denominators
- **Primary Control NSE Ledger A Per-Trade Mean PCR**: **`-1.3384`** (evaluated across $K=535$ positive-MFE trades; median `0.0000`).
- **Primary Control NSE Ledger A Aggregate Cohort PCR**: **`-0.2082`** ($\frac{-17,263.27}{82,914.55}$).
- **Source Snapshot SHA256**: `0190c56603ce59b9db2f695b536a751ac70371e010621c04633e2366982bb0b5` (Timestamp: `2026-09-28 07:31:46 UTC`).

---

## 5. Candidate Mechanisms

### Candidate A — Peak-Profit Trailing Boundary (PFT)
- **Hypothesis**: A protective stop based on maximum favourable movement achieved after entry reduces profit giveback while preserving participation in continued favourable movement.
- **LONG Boundary**: $SL_t = \max(SL_{t-1}, P_{\text{peak},t} - D_t)$
- **SHORT Boundary**: $SL_t = \min(SL_{t-1}, P_{\text{trough},t} + D_t)$

### Candidate B — Stepwise Profit Locking (SPL)
- **Hypothesis**: Discrete profit-protection milestones reduce unnecessary boundary changes while retaining a defined portion of accumulated gains.
- **Milestone Set**: $\mathcal{M} = \{(A_1, L_1), (A_2, L_2), \dots, (A_n, L_n)\}$

### Candidate C — Volatility-Adjusted Trailing Boundary (VAT)
- **Hypothesis**: Scaling protective stop distance by volatility ($D_t = k \times \text{ATR}_t$) accommodates regime changes better than fixed-distance trailing.

### Candidate D — Partial Profit Realization (PPR)
- **Hypothesis**: Realizing a pre-defined fraction (e.g. 50%) at a specified milestone reduces reversal exposure while retaining residual trend participation.

---

## 6. Phase Gates & Qualification Decision Taxonomy

1. **Gate 1 — Runtime & Accounting Integrity**: Mandatory deterministic replay, event ordering, and isolation checks.
2. **Gate 2 — Historical Discovery**: Exploratory evaluation on discovery cohort.
3. **Gate 3 — Temporal Validation**: Confirmatory frozen evaluation on untouched validation cohort.
4. **Gate 4 — Prospective Shadow Replication**: Execution-isolated accumulation on live prospective stream.
5. **Gate 5 — Separate Promotion Review**: Independent operational review prior to any live authority.

---

## 7. Controlled Research Deliverables

- [x] **RP-609-001 — Baseline Profit Giveback Audit**: Reconciled population distribution ($N=1,551$), MFE/MAE, profit capture, and currency isolation.
- [ ] **RP-609-002 — Candidate Policy Registry**: Frozen parameter schemas for PFT, SPL, VAT, and PPR.
- [ ] **RP-609-003 — Shadow Ledger Specification**: Event schema, tick accounting model, and matched identity.
- [ ] **RP-609-004 — Historical Discovery Report**: Independent results for candidates A–D.
- [ ] **RP-609-005 — Temporal Validation Report**: Confirmatory validation with bootstrap uncertainty.
- [ ] **RP-609-006 — Prospective Shadow Validation Report**: Bounded prospective paired replication.
- [ ] **RP-609-007 — Promotion Review Dossier**: Independent operational readiness review.
