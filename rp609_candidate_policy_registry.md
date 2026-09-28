# RP-609-002 — Candidate Policy Registry Specification

**Status**: SPECIFIED / GATED  
**Governance Standard**: RP-600 Pre-Check Verified | Execution-Layer Policy Contract  
**Target Class**: Isolated Shadow Policy Evaluation (Candidates A, B, C, D)  
**Control Baseline**: RP-605 Control Baseline — Frozen Ledger A ($N=982$ Unique Trades)  
**Execution Authority**: None (Shadow-Only Evaluation)  

---

## 1. Executive Summary & Policy Scope

Deliverable **RP-609-002** establishes the formal executable policy registry for the four candidate profit-retention mechanisms defined under RP-609:

1. **Candidate A — Peak-Profit Trailing Boundary (PFT-v1.0)**
2. **Stepwise Profit Locking (SPL-v1.0)**
3. **Volatility-Adjusted Trailing Boundary (VAT-v1.0)**
4. **Partial Profit Realization (PPR-v1.0)**

Each candidate mechanism is an independent, versioned policy contract with explicit parameter schemas, activation conditions, boundary update formulas, stop monotonicity rules, gap-through fill handling, friction modeling, and premature exit opportunity-cost diagnostics.

---

## 2. Non-Negotiable Governance & Replay Constraints

1. **Baseline Isolation**: RP-605 control baseline remains 100% frozen. All candidates are evaluated in isolated shadow ledgers receiving identical entry events, position sizes, and market tick sequences.
2. **RP-607/RP-608 Governance Integrity**: RP-607 Q3 status remains **`PENDING / INCONCLUSIVE`**. RP-608 dynamic engine remains **`SHADOW-ONLY`**. Neither provides live execution authority for RP-609.
3. **No Threshold Hunting**: Candidate parameters and primary outcome metrics MUST be pre-registered before replay execution. Iterative parameter optimization on validation or prospective streams is strictly prohibited.
4. **Contemporaneous Input Availability**: At any simulated timestamp $t$, policy calculations MUST use only market and trade state data available at $t$. Zero lookahead or retrospective indicator recalculation is permitted.

---

## 3. Candidate Policy Contracts

### 3.1 Candidate A — Peak-Profit Trailing Boundary (`PFT-v1.0`)

#### A. Policy Purpose & Hypothesis
A dynamic protective stop moving in the direction of trade profitability as new peak unrealized profit ($\text{MFE}_t$) is achieved reduces profit giveback while allowing favorable trends to continue.

#### B. Executable Specification
- **Activation Condition**: Activate profit trailing when open unrealized P&L reaches or exceeds activation threshold $\text{MFE}_0$:
  $$\text{MFE}_t \ge \text{MFE}_0$$
- **Trailing Stop Calculation**:
  - **LONG Position**: $SL_t = \max\left(SL_{t-1}, \, P_{\text{peak},t} - D_{\text{trail}}\right)$
  - **SHORT Position**: $SL_t = \min\left(SL_{t-1}, \, P_{\text{trough},t} + D_{\text{trail}}\right)$
  where $P_{\text{peak},t}$ is the maximum favorable price observed since entry, and $D_{\text{trail}}$ is the frozen trailing distance.
- **Monotonicity Rule**: $SL_t$ MUST be non-decreasing for LONG positions and non-increasing for SHORT positions. $SL_t$ never retreats after advancing.

#### C. Parameter Schema (`PFT-v1.0`)
- `activation_threshold_mfe`: Initial risk multiple $R_0$ (e.g. $1.0 R$) or absolute currency offset.
- `trailing_distance`: Fixed price distance $D$ or risk multiple fraction.
- `update_cadence`: `EACH_TICK` or `BAR_CLOSE_1M`.

---

### 3.2 Candidate B — Stepwise Profit Locking (`SPL-v1.0`)

#### A. Policy Purpose & Hypothesis
Advancing protective stops across discrete profit milestones prevents unnecessary boundary churn during minor price fluctuations while securing locked profit tiers.

#### B. Executable Specification
- **Milestone Set**: Pre-registered set of activation milestones and locked profit levels:
  $$\mathcal{M} = \{(A_1, L_1), (A_2, L_2), \dots, (A_n, L_n)\}$$
  where $A_i$ is the peak profit activation milestone, and $L_i$ is the guaranteed minimum locked profit level ($L_i < A_i$).
- **Update Logic**: When $\text{MFE}_t \ge A_i$, advance protective stop to lock profit level $L_i$:
  - **LONG Position**: $SL_t = \max\left(SL_{t-1}, \, P_{\text{entry}} + \frac{L_i}{Q}\right)$
  - **SHORT Position**: $SL_t = \min\left(SL_{t-1}, \, P_{\text{entry}} - \frac{L_i}{Q}\right)$
- **Progression Invariant**: Milestones MUST trigger in sequence ($A_1 \to A_2 \to \dots \to A_n$). Locked profit level $L_k$ can never regress to $L_{k-1}$.

#### C. Parameter Schema (`SPL-v1.0`)
- `milestones`: List of $(A_i, L_i)$ pairs (e.g., $\{(1.0R, 0.25R), \, (2.0R, 1.0R), \, (3.0R, 2.0R)\}$).
- `lock_type`: `RISK_MULTIPLE` or `CURRENCY_FIXED`.

---

### 3.3 Candidate C — Volatility-Adjusted Trailing Boundary (`VAT-v1.0`)

#### A. Policy Purpose & Hypothesis
Scaling the trailing stop distance dynamically by market volatility ($\text{ATR}_t$) accommodates changing volatility regimes, reducing premature exits during high-volatility expansions.

#### B. Executable Specification
- **Volatility Calculation**: Calculate Average True Range ($\text{ATR}_W$) over lookback window $W$ using bars completed prior to update timestamp $t$.
- **Dynamic Trailing Distance**:
  $$D_t = \min\left(D_{\max}, \, \max\left(D_{\min}, \, k_{\text{ATR}} \times \text{ATR}_{W,t}\right)\right)$$
  where $k_{\text{ATR}}$ is the volatility multiplier, and $[D_{\min}, D_{\max}]$ are safety bounds.
- **Stop Update Logic**:
  - **LONG Position**: $SL_t = \max\left(SL_{t-1}, \, P_{\text{peak},t} - D_t\right)$
  - **SHORT Position**: $SL_t = \min\left(SL_{t-1}, \, P_{\text{trough},t} + D_t\right)$

#### C. Parameter Schema (`VAT-v1.0`)
- `atr_window`: Bar lookback $W$ (e.g. 14 bars).
- `atr_multiplier`: Multiplier $k_{\text{ATR}}$ (e.g. 2.0).
- `min_distance` / `max_distance`: Safety boundary constraints.

---

### 3.4 Candidate D — Partial Profit Realization (`PPR-v1.0`)

#### A. Policy Purpose & Hypothesis
Closing a pre-defined portion (e.g. 50%) of the position upon reaching a profit milestone secures immediate realized gains while leaving the residual quantity open to capture continued trends.

#### B. Executable Specification
- **Partial Execution Trigger**: When $\text{MFE}_t \ge M_{\text{partial}}$, execute a market order to close fraction $\alpha \in (0, 1)$ of the open position quantity $Q_0$:
  $$Q_{\text{realized}} = \alpha \cdot Q_0, \quad Q_{\text{residual}} = (1 - \alpha) \cdot Q_0$$
- **Residual Quantity Lifecycle**: Manage $Q_{\text{residual}}$ under its baseline SL/TP or a designated residual trailing policy until final exit $T_{\text{exit}}$.
- **Consolidated Lifecycle Accounting**: Total trade realized P&L is the consolidated sum of partial realized P&L and residual realized P&L net of transaction costs:
  $$\text{PnL}_{\text{total}} = \text{PnL}_{\text{partial}} + \text{PnL}_{\text{residual}} - \text{Costs}_{\text{partial}} - \text{Costs}_{\text{residual}}$$

#### C. Parameter Schema (`PPR-v1.0`)
- `partial_trigger_mfe`: Profit milestone $M_{\text{partial}}$ (e.g. $1.5R$).
- `realization_fraction`: Quantity fraction $\alpha$ (e.g. $0.50$).
- `residual_policy`: `BASELINE_STOP` or `BREAKEVEN_STOP`.

---

## 4. Common Execution & Friction Model

All candidate shadow ledgers MUST apply identical market execution rules:

1. **Gap-Through Fill Rule**: If price gaps past a protective stop $SL_t$ (e.g. at session open), the fill price MUST equal the first available tick price $P_{\text{open}}$, NOT the stop price.
2. **Slippage & Spread Model**: Apply realistic bid/ask spread and fixed per-share/contract slippage $\delta_{\text{slip}}$.
3. **Transaction Fee Model**: Apply exchange fees, brokerage, and statutory taxes on every partial and full fill.
4. **Premature Exit Opportunity-Cost Diagnostic**: For any trade where a candidate exits earlier than control ($T_{\text{candidate\_exit}} < T_{\text{control\_exit}}$), track subsequent post-exit price trajectory over $+60\text{m}$ horizon to quantify forgone favorable continuation.

---

## 5. Controlled Research Artifacts

- [x] **RP-609-001 — Baseline Profit Giveback Audit**: Reconciled population distribution ($N=982$ unique trades), composite key matching, MFE giveback, and PCR reconciliation.
- [x] **RP-609-002 — Candidate Policy Registry**: Frozen executable policy contracts for Candidates A, B, C, and D.
- [ ] **RP-609-003 — Shadow Ledger Specification**: Event schema, tick accounting model, and matched identity harness.
- [ ] **RP-609-004 — Historical Discovery Report**: Independent replay results for candidates A–D on discovery cohort.
- [ ] **RP-609-005 — Temporal Validation Report**: Confirmatory validation with bootstrap uncertainty on validation cohort.
- [ ] **RP-609-006 — Prospective Shadow Validation Report**: Bounded prospective paired replication.
- [ ] **RP-609-007 — Promotion Review Dossier**: Independent operational readiness review.
