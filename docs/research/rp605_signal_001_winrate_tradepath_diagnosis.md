# RP-605-SIGNAL-001: Win-Rate & Trade-Path Diagnostic Report

**Document Identifier**: `RP-605-SIGNAL-001`  
**Protocol Series**: `RP-605 (NSE Prospective Freshness Validation)`  
**Date**: 2026-10-22  
**Status**: EMPIRICAL DIAGNOSTIC REPORT & TARGET CONTRACT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Executive Mandate & Target Definition

Following formal stakeholder consensus, the operational target for the RP-605 research program is explicitly defined:

```
+------------------------------------------------------------------------------------------------------+
|                                   RP-605 TARGET CONTRACT                                             |
+--------------------------+-----------------------------------+---------------------------------------+
| Target Metric            | ~75% Net Profitable Trades        | After all statutory taxes, brokerage, |
|                          |                                   | and estimated execution slippage      |
+--------------------------+-----------------------------------+---------------------------------------+
| Target Scope             | Entire Portfolio                  | Universal portfolio-level standard,   |
|                          | with Regime-Level Reporting       | broken down by behavioral compartment |
+--------------------------+-----------------------------------+---------------------------------------+
```

This report analyzes **2,029 completed historical trades** across four replication sessions (September 28 – October 01, 2026) to diagnose why the baseline achieves only a **33.0% Gross Win Rate** and a **20.4% Net Win Rate**, identifying the exact mathematical divergence between entry failure and exit failure.

---

## 2. Payout Geometry & Required Break-Even Win Rate

Evaluating the 2,029 historical trades reveals a favorable underlying payoff asymmetry:

$$\bar{W} = +₹263.52 \quad (+26.64\text{ bps}) \quad [\text{Average Gross Win}]$$
$$\bar{L} = -₹122.56 \quad (-12.35\text{ bps}) \quad [\text{Average Gross Loss}]$$
$$\bar{C} = ₹89.22 \quad (9.00\text{ bps}) \quad [\text{Average Statutory Friction}]$$

### Payoff Ratio
$$\text{Payoff Ratio} = \frac{\bar{W}}{\bar{L}} = \frac{263.52}{122.56} = \mathbf{2.15\times}$$

### Break-Even Win Rate Formula
$$\mathbb{E}[P_{\text{net}}] = p \bar{W} - (1-p) \bar{L} - \bar{C} = 0 \implies p_{\text{BE}} = \frac{\bar{L} + \bar{C}}{\bar{W} + \bar{L}}$$

$$p_{\text{BE}} = \frac{122.56 + 89.22}{263.52 + 122.56} = \frac{211.78}{386.08} = \mathbf{54.85\%}$$

### Significance of the 75% Target:
Because the strategy possesses a $2.15:1$ payoff asymmetry, achieving a **75% Net Win Rate** does not merely achieve break-even; it produces explosive positive net expectancy:

$$\mathbb{E}[P_{\text{net}}]_{75\%} = 0.75(263.52) - 0.25(122.56) - 89.22 = 197.64 - 30.64 - 89.22 = \mathbf{+₹77.78 \text{ per trade}} \quad (+7.8\text{ bps net})$$

Across a standard 2,000-trade cycle, this transforms a **−₹171,035 net loss** into a **+₹155,560 net profit**.

---

## 3. Trade-Path Excursion Diagnosis: Entry Failure vs. Exit Failure

To determine whether trades fail due to poor entry signals or poor exit timing, we analyze the Maximum Favorable Excursion (MFE) and Maximum Adverse Excursion (MAE) across all **1,616 net losing trades**:

```
Total Net Losing Trades: 1,616 (79.6% of all completed flow)
  ├── 1. Immediate Adverse Selection (Zero MFE):           487 trades (30.1% of losers)
  └── 2. Faded Profitable Excursions (Positive MFE):     1,129 trades (69.9% of losers)
        ├── Reached Net Profitability (MFE ≥ Friction):    480 trades (29.7% of losers)
        └── Large Profit Givebacks (MFE ≥ ₹200 / +20 bps): 201 trades (12.4% of losers)
```

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    FAILURE MODE DECOMPOSITION                                        |
+──────────────────────────────────────────────────+───────────────────────────────────────────────────+
| 1. Exit Management Failure (69.9% of Losers)     | 2. Entry Quality Failure (30.1% of Losers)        |
+──────────────────────────────────────────────────+───────────────────────────────────────────────────+
| • 1,129 losing trades moved into profit first.   | • 487 trades had MFE ≤ 0 (moved immediately       |
| • 480 trades reached MFE ≥ ₹89.22 (net green)    |   against entry position).                        |
|   before reversing into full losses.             | • Suffered immediate adverse microstructure drift.|
| • 257 trades won gross but lost net due to fees. | • True entry signal invalidity.                   |
| ──► Remedy: Dynamic Trailing Lock & ATR Stop     | ──► Remedy: Dynamic Behavioral Compartment Gate   |
+──────────────────────────────────────────────────+───────────────────────────────────────────────────+
```

### Key Diagnostic Discovery (MFE Opportunity vs. Executable Reality):
While 480 losing trades reached $\text{MFE} \ge \text{friction}$ at some point in their life cycle, **MFE is an upper bound, not an executable backtest**. Reaching an excursion does not guarantee that an executable policy can capture it after giveback, intrabar volatility, and statutory costs.

Under executable counterfactual replay (Gate 1 & Gate 2 of `RP-605-EXIT-001`), an executable trailing lock (activating at $+15\text{ bps}$, locking $50\%$ of peak excursion) converts **240 trades** into net winners, raising the empirical net win rate from **20.4% to 32.2%** (+11.8 percentage points), rather than the theoretical ceiling of 44.0%.

---

## 4. Behavioural Archetype Dispersion: Why Flat Rules Fail

Analyzing symbol-level performance reveals radical dispersion across behavioural archetypes:

### Top 5 Performing Symbols (Point-in-Time Directional Movers)
| Symbol | Completed Trades | Gross Win Rate | Net Win Rate | Total Net P&L |
| :--- | :---: | :---: | :---: | :---: |
| `AMBUJACEM.NS` | 20 | **45.0%** | **45.0%** | **+₹1,501.19** |
| `ADANIENT.NS` | 22 | **50.0%** | **40.9%** | **+₹12.38** |
| `HEROMOTOCO.NS`| 26 | **50.0%** | **34.6%** | **+₹479.65** |
| `BOSCHLTD.NS` | 22 | **40.9%** | **27.3%** | **+₹2,399.14** |
| `COLPAL.NS` | 20 | **35.0%** | **30.0%** | **−₹767.72** |

### Bottom 5 Performing Symbols (Friction-Sensitive / Mean-Reverting Chop)
| Symbol | Completed Trades | Gross Win Rate | Net Win Rate | Total Net P&L |
| :--- | :---: | :---: | :---: | :---: |
| `IPCALAB.NS` | 22 | 22.7% | **4.5%** | **−₹3,188.28** |
| `ASIANPAINT.NS`| 29 | 20.7% | **6.9%** | **−₹4,344.20** |
| `DIVISLAB.NS` | 26 | 30.8% | **7.7%** | **−₹2,923.43** |
| `ESCORTS.NS` | 24 | 37.5% | **8.3%** | **−₹2,437.87** |
| `NTPC.NS` | 23 | 8.7% | **8.7%** | **−₹4,519.61** |

### Root Cause of the C5 Hurdle Breakdown:
When evaluating trades by predicted return quartile (`macro_r60_bps`), the **Strongest Positive Quartile** delivered the **worst net win rate (15.6%)** and the largest net loss (**−₹61,663.51**). 
- *Causal Diagnosis*: Highly positive macro prediction scores occurred when a stock had already experienced an aggressive momentum thrust, causing entry at the upper bound of the intraday range immediately prior to sharp mean-reversion.

---

## 5. Failure Mode Taxonomy: Inversion vs. Filtering vs. Exit Repair

Losing trades must not be conflated into a single bucket or blindly inverted. When an entry loses money, three mutually exclusive mechanisms exist:

| Failure Mechanism | Manifestation in Excursion Data | Strategic Remedy | Empirical Status |
| :--- | :--- | :--- | :--- |
| **1. Bad Timing** | Trade moved into substantial profit ($\text{MFE} > 0$) before reversing. Direction was initially correct, but entry was early or exit was late. | **Exit Management Repair** (`RP-605-EXIT-001`): Dynamic trailing profit lock. | **Empirically Proven**: Converts 240 trades into net wins (+11.8% win rate boost). |
| **2. False Signal** | Market regime or compartment made original entry unreliable (e.g. friction-sensitive chop). | **Entry Filtering** (`RP-605-BEH-001`): Avoid unsupportive conditions. | **Under Investigation**: Ablation shows gross improvements; requires conditioning on soft belief states. |
| **3. Contrarian Signal** | Pre-entry conditions reliably predict a move in the *opposite* direction. | **Conditional Inversion**: Test taking the opposite position under verified conditions. | **Hypothesis Only**: Linear correlation between pre-entry features and Zero MFE is negligible ($r \approx 0.03$). Inversion requires non-linear out-of-sample predictability. |

---

## 6. Friction Sensitivity & Break-Even Reconciliation

Under the empirical baseline:
- $\bar{W} = +₹263.52$ (+26.64 bps gross win)
- $\bar{L} = -₹122.56$ (-12.35 bps gross loss)
- $\bar{C} = ₹89.22$ (9.00 bps statutory friction)
- Baseline Break-Even Win Rate: $p_{\text{BE}} = \frac{122.56 + 89.22}{263.52 + 122.56} = \mathbf{54.85\%}$

If friction is reduced by 4.0 bps (from 9.0 bps down to 5.0 bps, reducing average charges by ₹40 to ₹49.22):
$$p_{\text{BE}} = \frac{122.56 + 49.22}{263.52 + 122.56} = \frac{171.78}{386.08} = \mathbf{44.49\% \approx 44.5\%}$$

*(Note: Prior draft projections citing 42.0% were arithmetically inconsistent with the reported win/loss magnitudes. The true break-even rate is 44.5%).*

---

## 7. Decoupled Research Program Governance

The ~75% Net Win Rate target remains an explicit overarching program objective, but **must never be treated as an assumed output at each stage**. Progress is evaluated strictly via sequential, controlled counterfactual experiments:

1. **`RP-605-EXIT-001`**: Controlled Exit Policy Replay (Isolate exit management on frozen identical entries).
2. **`RP-605-BEH-001`**: Dynamic Behavioural Perception Engine (Soft belief states and compartment evolution).
3. **`RP-605-INV-001`**: Conditional Direction Inversion Study (Evaluate whether predictable contrarian signals exist).
4. **`RP-605-MAKER-001`**: Passive Execution & Friction Reduction (Realistic queue dynamics and fill modeling).
