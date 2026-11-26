# RP-615-EE-005: Pre-Registration Protocol for C5 Maker Selection & Retail Fee Viability

**Document Identifier**: `RP-615-EE-005`  
**Author**: Antigravity AI Pair Programmer & Research Team  
**Date**: 2026-10-22  
**Status**: PRE-REGISTERED EXPERIMENTAL PROTOCOL  
**Governance Authority**: Strictly **0% Live Order Authority** (Observational Shadow Telemetry Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants)

---

## 1. Problem Formulation & Net-Edge Hurdle Mathematics

The empirical evaluation of Milestone 3 prospective telemetry (`RP-615-EE-004-SCORECARD-V2-RECONCILED`) across 21 prospective queue clearance orders established the following baseline parameters:

- **Unfiltered Gross Expectancy ($E_{\text{gross}}$)**: $-0.83\text{ bps}$ [95% CI: $-5.37, +3.30$] (measured from 243 unconstrained shadow closes).
- **Empirical 60s Maker Adverse Selection ($\delta_{\text{adv}}$)**: $4.24\text{ bps}$ (mean price drift moving against resting limit orders post-clearance).
- **Retail BNB-Discount Maker Fee ($\text{fee}$)**: $7.5\text{ bps}$ (Binance Spot VIP0 with 25% BNB fee deduction).

### Net Hurdle on Unfiltered Flow
Applying the stated cost equation:
$$E_{\text{net}} = E_{\text{gross}} - \text{fee} - \delta_{\text{adv}} = -0.83 - 7.50 - 4.24 = \mathbf{-12.57\text{ bps}} \quad [95\%\text{ CI: } -17.11, -8.44]$$

**Conclusion**: Resting maker orders on unfiltered flow suffer guaranteed negative economic drift. Passive execution alone cannot overcome retail fees and adverse selection without high-conviction alpha filtering.

---

## 2. Explicit Mathematical Definition of Candidate Arm S5-B

To eliminate ambiguity between signal scores, expected returns, and realized returns:

### 1. The C5 Hurdle ($\hat{y}_{\text{C5}}$)
- **Definition**: The out-of-sample calibrated model prediction of **expected 1-hour gross trade return** in basis points:
  $$\hat{y}_{\text{C5}} \equiv E\left[\frac{P_{t+3600} - P_t}{P_t} \times 10,000 \;\middle|\; \mathbf{x}_t\right] \ge \mathbf{+15.0\text{ bps}}$$
- **Operational Meaning**: The trade is admitted if and only if the model predicts an unconditional gross edge of at least $+15.0\text{ bps}$.

### 2. The Joint Volume Expansion Filter
- **Volume Acceleration**: $\text{VolAccel} = \frac{\text{Vol}_{5m} - \text{Vol}_{15m}}{\text{Vol}_{15m}} \ge 0.0$
- **Relative Volume**: $\text{RVOL} = \frac{\text{Vol}_{15m}}{\text{SMA}(\text{Vol}_{15m}, 20)} \ge 0.80$

### 3. Theoretical Net Hurdle Equation
Under the C5 hurdle and empirical maker friction, the pre-registered expected net return is:
$$\hat{E}_{\text{net}} = \hat{y}_{\text{C5}} - \text{fee}_{\text{retail}} - \delta_{\text{adv}} \ge +15.00 - 7.50 - 4.24 = \mathbf{+3.26\text{ bps}}$$

---

## 3. Pre-Registered Comparative Test (Gate 1 Activation)

Upon accumulating **$N = 30$ prospective queue clearance fills** across primary pairs (BTCUSDT: 15, ETHUSDT: 15), the evaluator (`scripts/evaluate_empirical_adverse_selection.py`) shall automatically execute the following paired hypothesis test:

### Cohort Definitions
- **Cohort 1 (Control — Unfiltered Maker Flow)**: All passive orders simulated at touch that cleared the queue threshold ($V_{\text{contra}} \ge V_{\text{threshold}}$).
- **Cohort 2 (Treatment — Candidate Arm S5-B)**: Passive orders satisfying $\hat{y}_{\text{C5}} \ge 15.0\text{ bps}$ AND Joint Volume Expansion.

### Test Metrics & Statistical Uncertainty
1. **Net Expectancy**: Point estimate and 95% Bootstrap CI ($B = 10,000$ resamples) under Retail BNB Discount (7.5 bps) and Standard (10.0 bps).
2. **Profit Factor**: Realized gross wins divided by gross losses.
3. **Adverse Selection Drift**: Multi-horizon mid-price movement (10s, 30s, 60s, 300s) post-clearance.
4. **Hypothesis Decision Rule**:
   - **PASS Gate 5**: Cohort 2 lower bound of 95% CI is strictly positive ($\text{CI}_{95, \text{low}} > 0.0\text{ bps}$) under Retail BNB Discount.
   - **FAIL Gate 5**: $\text{CI}_{95, \text{low}} \le 0.0\text{ bps}$.

---

## 4. Governance & Telemetry Invariants

1. **Telemetry State Discipline**:
   - `State 1: SignalQualified`
   - `State 2: PassiveOrderSimulated`
   - `State 3: QueueClearanceEstimated`
   - `State 4: ProspectiveFillEstimated`
   - `State 5: ExchangeFillConfirmed` (**STRICTLY ZERO — 0% Live Order Authority**)
   Prospective queue clearance fills must never be reported as realized exchange executions.
2. **Adapter Preservation**:
   - `adapters/chronosentiment` remains 100% frozen (13/13 discipline invariants). Exactly zero adapter files modified.
3. **Reproducibility**:
   - Telemetry logs, order IDs, timestamps, limit prices, and Binance kline traces are committed to version control in `live_capture/shadow_validation/`.
