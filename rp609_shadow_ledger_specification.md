# RP-609-003 — Shadow Ledger Specification & Replay Harness Contract

**Status**: SPECIFIED / GATED  
**Governance Standard**: RP-600 Pre-Check Verified | Deterministic Simulation Contract  
**Deliverable ID**: `RP-609-003`  
**Control Baseline**: RP-605 Control Baseline — Frozen Ledger A ($N=982$ Unique Composite Identities)  
**Execution Authority**: None (Shadow-Only Replay Harness)  

---

## 1. Executive Summary & Specification Scope

Deliverable **RP-609-003** establishes the exact deterministic replay harness specification, mathematical unit conversion formulas, same-tick event priority order, gap-through execution rules, friction modeling, partial-realization lifecycle accounting, and $+60\text{m}$ opportunity-cost tracking for testing Candidate Mechanisms A, B, C, and D against the frozen RP-605 control baseline.

---

## 2. Pre-Registered Frozen Policy Parameters

To prevent post-hoc threshold hunting or selective parameter tuning, candidate parameters are explicitly frozen prior to replay execution:

| Candidate Mechanism | Frozen Policy Identifier | Activation Condition | Policy Parameter Schema | Update Cadence |
| :--- | :--- | :--- | :--- | :--- |
| **Candidate A — Peak-Profit Trailing (PFT)** | `PFT-v1.0-FROZEN` | $\text{MFE}_t \ge 1.0 R_0$ | $D_{\text{trail}} = 0.50 R_0$ | `EACH_TICK` |
| **Candidate B — Stepwise Profit Locking (SPL)** | `SPL-v1.0-FROZEN` | $\text{MFE}_t \ge A_i$ | $\mathcal{M} = \{(1.0R_0, 0.25R_0), (2.0R_0, 1.00R_0), (3.0R_0, 2.00R_0)\}$ | `EACH_TICK` |
| **Candidate C — Volatility-Adjusted Trailing (VAT)** | `VAT-v1.0-FROZEN` | $\text{MFE}_t \ge 1.0 R_0$ | $W = 14 \text{ bars (1m)}, k_{\text{ATR}} = 2.0, D \in [0.25R_0, 3.00R_0]$ | `BAR_CLOSE_1M` |
| **Candidate D — Partial Profit Realization (PPR)** | `PPR-v1.0-FROZEN` | $\text{MFE}_t \ge 1.5 R_0$ | $\alpha = 0.50 \text{ (50\% scale-out)}, \text{Residual SL} = P_{\text{entry}}$ | `EACH_TICK` |

---

## 3. Mathematical Unit & Price Conversions

For a trade lifecycle $i$ with entry price $P_{\text{entry}}$, initial stop loss $SL_0$, position size $Q_0$, and direction $\mathcal{S} \in \{\text{LONG}, \text{SHORT}\}$:

1. **Initial Risk Unit ($R_0$)**:
   $$R_0 = |P_{\text{entry}} - SL_0|$$

2. **Risk Multiple to Stop Price Conversion ($SL(R)$)**:
   - **LONG Position**: $SL_{\text{LONG}}(R) = P_{\text{entry}} + R \cdot R_0$
   - **SHORT Position**: $SL_{\text{SHORT}}(R) = P_{\text{entry}} - R \cdot R_0$

3. **Running Unrealized PnL at Timestamp $t$ ($\text{PnL}_{\text{unrealized}}(t)$)**:
   - **LONG Position**: $\text{PnL}_{\text{LONG}}(t) = Q_t \cdot (P(t) - P_{\text{entry}})$
   - **SHORT Position**: $\text{PnL}_{\text{SHORT}}(t) = Q_t \cdot (P_{\text{entry}} - P(t))$

4. **Running Maximum Favorable Excursion ($\text{MFE}_t$)**:
   $$\text{MFE}_t = \max_{s \le t} \text{PnL}_{\text{unrealized}}(s)$$

---

## 4. Deterministic Same-Tick Event Priority Order

When multiple market events (stop loss breach, target fill, baseline engine exit, or candidate boundary update) occur within the same price tick or bar interval, the replay engine MUST resolve event precedence in the following deterministic order:

```text
                       SIMULATED TICK / BAR DATA
                                   │
                                   ▼
                    [1] Gap-Through Hard SL Check
                 (Did Open/Tick cross active SL?)
                                   │
                     ┌─────────────┴─────────────┐
                     ▼ YES                       ▼ NO
               Execute SL Fill            [2] Target Fill Check
              (Terminal Exit)            (Did Tick cross TP?)
                                                 │
                                   ┌─────────────┴─────────────┐
                                   ▼ YES                       ▼ NO
                             Execute TP Fill             [3] Baseline Engine Exit
                            (Terminal Exit)             (Did t reach t_exit?)
                                                               │
                                                 ┌─────────────┴─────────────┐
                                                 ▼ YES                       ▼ NO
                                           Execute Engine Exit         [4] Policy Boundary Update
                                            (Terminal Exit)            (Evaluate PFT/SPL/VAT/PPR)
```

---

## 5. Market Execution, Gap-Through & Friction Modeling

1. **Gap-Through Fill Price Rule**:
   - If market open or tick price $P_{\text{tick}}$ gaps through active stop loss $SL_t$:
     - **LONG Position**: $P_{\text{fill}} = \min(SL_t, P_{\text{tick}})$
     - **SHORT Position**: $P_{\text{fill}} = \max(SL_t, P_{\text{tick}})$
   - Stop loss orders MUST NOT be filled at $SL_t$ when price gaps past the stop.

2. **Transaction Costs & Slippage Friction**:
   - **Exchange & Brokerage Fee**: $f_{\text{bps}} = 5.0 \text{ bps}$ ($0.05\%$) of gross fill value per trade leg.
   - **Fixed Slippage Penalty**: $\delta_{\text{slip}} = 0.02\%$ of price per fill.
   - Total Cost per Fill: $\text{Cost} = \text{Fill Value} \cdot (f_{\text{bps}} + \delta_{\text{slip}})$.

---

## 6. Consolidated Lifecycle Accounting for Partial Realization (PPR)

For Candidate D (Partial Profit Realization):
1. **Partial Fill Execution**: At $t_{\text{partial}}$ when $\text{MFE}_t \ge M_{\text{partial}}$, execute partial exit for quantity $Q_{\text{partial}} = \alpha \cdot Q_0$.
2. **Realized Cash Flow**: Record partial cash flow:
   $$\text{Cash}_{\text{partial}} = Q_{\text{partial}} \cdot (P_{\text{fill, partial}} - P_{\text{entry}}) - \text{Costs}_{\text{partial}} \quad (\text{LONG})$$
3. **Residual Position Management**: Remaining quantity $Q_{\text{residual}} = (1 - \alpha) \cdot Q_0$ remains open with stop loss advanced to breakeven ($SL_{\text{residual}} = P_{\text{entry}}$) until final exit at $t_{\text{final}}$.
4. **Total Consolidated PnL**:
   $$\text{PnL}_{\text{consolidated}} = \text{Cash}_{\text{partial}} + \text{Cash}_{\text{residual}}$$

---

## 7. Standardized $+60\text{m}$ Opportunity Cost Diagnostic

For any candidate trade lifecycle $i$ that exits at $T_{\text{candidate\_exit}} < T_{\text{control\_exit}}$:
1. Reconstruct market price $P(+60\text{m})$ at timestamp $T_{\text{candidate\_exit}} + 60\text{ minutes}$.
2. Calculate counterfactual PnL at $+60\text{m}$:
   $$\text{PnL}_{\text{counterfactual}, +60\text{m}} = Q_0 \cdot \left(P(+60\text{m}) - P_{\text{entry}}\right) \quad (\text{LONG})$$
3. Calculate Continuation Opportunity Cost:
   $$\text{OpportunityCost}_{60\text{m}} = \max\left(0, \, \text{PnL}_{\text{counterfactual}, +60\text{m}} - \text{PnL}_{\text{candidate\_realized}}\right)$$

---

## 8. Immutable Shadow Ledger Event Schema

Every shadow replay event MUST emit a structured JSON record with the following mandatory schema:

```json
{
  "run_id": "RUN-RP609-REPLAY-20260928-163000",
  "candidate_id": "PFT-v1.0-FROZEN",
  "policy_version": "1.0.0",
  "matched_composite_id": ["ATGL.NS", "LONG", "2026-09-28T04:50:50Z", "RP605-NSE-0004"],
  "event_timestamp_utc": "2026-09-28T04:55:00Z",
  "event_type": "BOUNDARY_UPDATED",
  "market_price": 850.50,
  "active_sl_before": 840.00,
  "active_sl_after": 845.25,
  "open_quantity_before": 100.0,
  "open_quantity_after": 100.0,
  "running_mfe_pnl": 1050.00,
  "running_mae_pnl": -200.00,
  "reason_code": "PEAK_PROFIT_TRAILING_ADVANCE",
  "fill_price": null,
  "transaction_costs": 0.0,
  "snapshot_sha256": "0190c56603ce59b9db2f695b536a751ac70371e010621c04633e2366982bb0b5"
}
```

---

## 9. Controlled Research Deliverables Checklist

- [x] **RP-609-001 — Baseline Profit Giveback Audit**: Reconciled population distribution ($N=982$ unique trades), composite key matching, MFE giveback, and PCR reconciliation.
- [x] **RP-609-002 — Candidate Policy Registry**: Frozen executable policy contracts for Candidates A, B, C, and D.
- [x] **RP-609-003 — Shadow Ledger Specification**: Event schema, tick accounting model, and matched identity harness.
- [ ] **RP-609-004 — Historical Discovery Report**: Independent replay results for candidates A–D on discovery cohort.
- [ ] **RP-609-005 — Temporal Validation Report**: Confirmatory validation with bootstrap uncertainty on validation cohort.
- [ ] **RP-609-006 — Prospective Shadow Validation Report**: Bounded prospective paired replication.
- [ ] **RP-609-007 — Promotion Review Dossier**: Independent operational readiness review.
