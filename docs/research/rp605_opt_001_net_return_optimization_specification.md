# RP-605-OPT-001: Net Return Optimization Specification (Revised)

**Document Identifier**: `RP-605-OPT-001-REV2`  
**Protocol Series**: `RP-605 (NSE Prospective Freshness Validation)`  
**Date**: 2026-10-22  
**Status**: REGISTERED SPECIFICATION & EXPERIMENTAL CONTRACT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Mode Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants)

---

## 1. Executive Context & Baseline Reality

The Tick #54 forensic audit on October 22, 2026 established that while the RP-605 freshness gate ($F_{15} \ge -0.0100$) delivers a significant gross relative advantage (+₹5,877.27 gross, saving ₹8,155.81 in transaction friction), **both ledgers remain deeply negative net after modeled statutory and broker friction**:
- **Baseline Ledger A**: Gross −₹5,818.10 | Modeled Charges ₹26,201.42 | **Net after Modeled Charges: −₹32,019.52**
- **Freshness Ledger B**: Gross +₹59.17 | Modeled Charges ₹18,045.61 | **Net after Modeled Charges: −₹17,986.44**
*(Note: Flat ₹70/trade sensitivity scenario produced −₹28,078.10 and −₹15,270.83; all metrics exclude unmodeled spread crossing and slippage).*

Under standard retail cash-equity execution (~₹82.40/trade all-in statutory charges), Freshness B's gross expectancy of +₹0.27/trade is inadequate to overcome execution friction. The objective is to increase expected net returns without indiscriminately starving trade flow or reducing statistical validity.

---

## 2. Optimization Triad Formulation

This document officially locks the optimization target and experimental constraints for RP-605:

```
+------------------------------------------------------------------------------------------------------+
|                                   RP-605 OPTIMIZATION TRIAD                                          |
+--------------------------+-----------------------------------+---------------------------------------+
| Dimension                | Selected Setting                  | Operational Definition                |
+--------------------------+-----------------------------------+---------------------------------------+
| 1. Primary Objective     | Maximize Risk-Adjusted Returns    | Downside-Penalized Portfolio Sortino  |
|                          | while Controlling Drawdown        | Ratio net of all statutory friction   |
+--------------------------+-----------------------------------+---------------------------------------+
| 2. Risk Constraint       | Balanced                          | Strict Intraday Max Drawdown Limit    |
|                          |                                   | (MDD ≤ 1.5% of nominal capital base)  |
+--------------------------+-----------------------------------+---------------------------------------+
| 3. Trade Frequency       | High                              | Statistical power / coverage floor    |
|                          |                                   | (N ≥ 150 trades; never a quota)       |
+--------------------------+-----------------------------------+---------------------------------------+
```

---

## 3. Mathematical Objective Function & Constraints

### 3.1 Primary Return Series: Mark-to-Market Portfolio Increments
To eliminate ambiguities arising from overlapping trade durations, unequal capital allocation, and variable leverage, the primary return series is evaluated on periodic mark-to-market portfolio equity increments:

$$r_t = \frac{W_t - W_{t-1}}{W_0} \quad \text{for } t \in \{1, \dots, T\}$$

Where:
- $W_0 \equiv ₹10,000,000\text{ INR}$ (Fixed nominal initial capital base).
- $W_t$ is total mark-to-market portfolio equity at minute $t$ (cash + cumulative realized P&L + open position mark-to-market minus accrued statutory charges).
- $T = 375$ intraday 1-minute ticks per session.

### 3.2 Canonical Sortino Ratio Formulation
Downside semi-deviation is calculated strictly against a minimum acceptable return threshold of zero ($\tau \equiv 0.0\text{ bps}$):

$$\sigma_{\text{downside}}(\mathbf{\theta}) = \sqrt{\frac{1}{T} \sum_{t=1}^T \min(0, r_t)^2}$$

$$\mathcal{S}_{\text{portfolio}}(\mathbf{\theta}) = \frac{\frac{1}{T} \sum_{t=1}^T r_t}{\sigma_{\text{downside}}(\mathbf{\theta})}$$

*Canonical Invariant*: There is no dual definition; the downside threshold is fixed at zero ($\tau = 0$), penalizing strictly negative mark-to-market intervals.

### 3.3 Explicit Constraints

1. **Intraday Drawdown Ceiling (Balanced Risk)**:
   $$\text{MDD}(\mathbf{\theta}) = \max_{t \in [0, T]} \left( \frac{\max_{s \le t} W_s - W_t}{W_0} \right) \le 1.50\% \quad (₹150,000\text{ INR ceiling})$$
   Mark-to-market includes all open position valuations at each minute bar.

2. **Frequency & Scrip Coverage Constraint (Not a Quota)**:
   $$N_{\text{completed}}(\mathbf{\theta}) \ge 150 \quad \text{trades per full 375-tick session}$$
   $$\text{Scrip Breadth}(\mathbf{\theta}) \ge 40\% \quad \text{of monitored liquid universe (≥ 48 / 119 scrips)}$$
   *Governance Rule*: This is a **breadth and statistical power constraint, never a quota**. The strategy is strictly prohibited from taking marginal or negative-expectancy trades to satisfy this condition. If qualified signals are absent, the strategy remains in cash. Any policy generating positive net P&L but failing $N \ge 150$ is classified as "insufficient empirical sample" rather than promoted.

3. **Core Invariant**:
   $$\Delta(\text{adapters/chronosentiment}) \equiv \mathbf{0}$$
   The core adapter remains 100% frozen. All candidate logic operates in the external decision evaluation harness.

---

## 4. Disambiguation of the C5 Hurdle: Predicted vs Realized

To ensure rigorous causal attribution, three distinct quantities are formally separated:

1. **Predicted C5 Expected Return ($\hat{y}_{\text{C5}}$)**:
   $$\hat{y}_{\text{C5}} \equiv \mathbb{E}\left[\frac{P_{t+60\text{m}} - P_t}{P_t} \times 10,000 \;\middle|\; \mathbf{x}_t\right] \ge \mathbf{+15.0\text{ bps}}$$
   What the C5 model estimates at decision time $t$ using strictly prior information ($\mathbf{x}_t$).

2. **Realized Gross Return ($R_{\text{gross}}$)**:
   $$R_{\text{gross}} = \frac{P_{\text{exit}} - P_{\text{entry}}}{P_{\text{entry}}} \cdot \text{dir}$$
   The actual price change achieved by the trade before any transaction costs. Targeting $+45\text{ bps}$ gross edge is a testable hypothesis, not an assumed property of the $+15\text{ bps}$ hurdle.

3. **Realized Net Return ($R_{\text{net}}$)**:
   $$R_{\text{net}} = R_{\text{gross}} - \frac{\text{Statutory Friction} + \text{Slippage}}{\text{Notional}}$$
   The realized return remaining after all modeled fees (~8.24 bps) and execution slippage.

---

## 5. Controlled Experimental Sequence (Step-by-Step Ablation)

Rather than running a monolithic multi-parameter search, four orthogonal experiments must be conducted sequentially:

```
[Step 1: Reproduce Baseline]
       │
       ▼
[Step 2: Entry-Policy Ablation]
       ├── Arm 1: Control (Baseline Flow)
       ├── Arm 2: Freshness Gate Only (F_15 >= -0.01)
       ├── Arm 3: C5 Threshold Alone (y_hat >= 15 bps)
       ├── Arm 4: Volume Expansion Alone (Vol >= 1.5x SMA20)
       └── Arm 5: Joint C5 + Volume Expansion
       │
       ▼
[Step 3: Warm-Up Policy Ablation]
       ├── Arm C-1: 3-Tick Trajectory Delay
       └── Arm C-2: 15-Minute Lookback Window
       │
       ▼
[Step 4: Exit & Realism Ablation]
       ├── Volatility-Scaled Stop Loss (1.5x ATR)
       ├── Trailing Profit Lock
       └── Limit-Order Queue Priority Simulation
       │
       ▼
[Step 5: Multi-Regime Holdout Evaluation]
```

---

## 6. Statistical Validation Protocol & Acceptance Gates

To guard against data snooping across overlapping sessions:

1. **Session-Block Bootstrap**:
   - Resampling must occur at the **session block level** (preserving full intraday cross-sectional correlation and temporal structure across all 119 scrips).
   - $B = 10,000$ session-block bootstrap iterations.

2. **Untouched Prospective Holdout**:
   - Development & ablation calibration restricted to September 28, 29, 30, and October 1 replication sessions.
   - Forward holdout sessions (October 2–21) remain **strictly untouched** until the final candidate policy is frozen.

3. **Multi-Regime Prospective Evidence Floor**:
   - Prospective shadow qualification requires evaluation across at least three distinct market regimes:
     1. Trending Up session ($\Delta \text{NIFTY} \ge +0.75\%$)
     2. Trending Down session ($\Delta \text{NIFTY} \le -0.75\%$)
     3. Chop / Mean-Reverting session ($|\Delta \text{NIFTY}| < 0.30\%$)
