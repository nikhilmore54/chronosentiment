# RP-605-BEH-001: Dynamic Behavioural Perception & Compartment Evolution Specification

**Document Identifier**: `RP-605-BEH-001`  
**Protocol Series**: `RP-605 (NSE Prospective Freshness Validation)`  
**Date**: 2026-10-22  
**Status**: REGISTERED ARCHITECTURAL SPECIFICATION & RESEARCH CONTRACT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Foundational Paradigm Shift: Perception Over Permanent Taxonomy

Traditional quantitative modeling frequently attempts to assign static categorical labels to equities (e.g., "high-beta momentum stock" or "defensive mean-reverter"). In real financial markets—particularly liquid Indian cash equities—this assumption fails:
1. **Regime Non-Stationarity**: A stock that exhibits persistent intraday momentum during earnings or macro expansion may revert violently during index consolidation.
2. **Multi-Horizon Duality**: A single symbol simultaneously exhibits short-horizon mean-reversion (at 1-minute to 5-minute microstructure scale) and multi-hour directional persistence (at 60-minute macro trend scale).
3. **Continuous Belief State**: A symbol is not a member of a mutually exclusive bin. It exhibits a vector of continuous behavioural tendencies with varying empirical confidence.

**Core Axiom of RP-605-BEH-001**:
> **A symbol does not have an immutable identity. Compartments are dynamic, evolving representations of current observed behaviour. Compartments can split, merge, emerge, or become dormant as market dynamics shift.**

---

## 2. Three-Layer Epistemic Decoupling

To prevent the subtle risk of self-confirming feedback loops (defining a compartment by the trades that succeeded, and then using those trades to validate the compartment), three operational layers are strictly separated:

```
+─────────────────────────────────────────────────────────────────────────────+
| LAYER 1: BEHAVIOURAL PERCEPTION ENGINE (Observable Dynamics)                |
| - Consumes rolling point-in-time price, volume, quote, and spread flow.     |
| - Computes multi-horizon behavioural fingerprint vector b_i(t).            |
| - Infers dynamic compartment membership and novelty without P&L knowledge.  |
+─────────────────────────────────────────────────────────────────────────────+
                                       │
                                       ▼
+─────────────────────────────────────────────────────────────────────────────+
| LAYER 2: POLICY EVALUATION & PREDICTION (Hypothesis Formulation)            |
| - Maps perceived behavioural state to candidate policy arms (A, B, C-1, C5).|
| - Generates expected net outcome hypotheses conditioned on the state.       |
+─────────────────────────────────────────────────────────────────────────────+
                                       │
                                       ▼
+─────────────────────────────────────────────────────────────────────────────+
| LAYER 3: INDEPENDENT PROSPECTIVE VALIDATION (Empirical Accounting)          |
| - Evaluates policy execution strictly on subsequent, unseen observations.    |
| - Charges exact statutory friction (~₹82.40/trade) and execution slippage.  |
| - Updates perception confidence; NEVER alters policy rules in-flight.       |
+─────────────────────────────────────────────────────────────────────────────+
```

---

## 3. The Multi-Horizon Behavioural Fingerprint Vector

At any observation timestamp $t$, each symbol $i$ is represented by a standardized continuous vector:

$$\mathbf{b}_i(t) = \left[ \rho_{\text{dir}, 5m}, \rho_{\text{dir}, 15m}, \text{MR}_{15m}, \sigma_{\text{instability}}, \text{Range}_{\text{ratio}}, \text{Vol}_{\text{elasticity}}, \text{Friction}_{\text{drag}}, \text{Tail}_{\text{kurtosis}} \right]$$

### Detailed Dimensions:
1. **Directional Persistence ($\rho_{\text{dir}}$)**:
   $$\rho_{\text{dir}} = \frac{|P_t - P_{t-k}|}{\sum_{j=1}^k |P_{t-j+1} - P_{t-j}|} \in [0, 1]$$
   Evaluated at $k=5$ bars (5-min) and $k=15$ bars (15-min). Measures path efficiency versus chop.
2. **Mean-Reversion Tendency ($\text{MR}_{15m}$)**:
   Autocorrelation of 1-minute returns $\text{Corr}(r_t, r_{t-1})$ over rolling 15 minutes. Negative values signify active bounce/reversal.
3. **Volatility Instability ($\sigma_{\text{instability}}$)**:
   Ratio of short-term realized range to rolling baseline range $\text{ATR}_{5}(t) / \text{ATR}_{60}(t)$. Measures expansion shocks.
4. **Range Structure ($\text{Range}_{\text{ratio}}$)**:
   Position of current price within session high-low channel $(P_t - L_{\text{session}}) / (H_{\text{session}} - L_{\text{session}})$.
5. **Volume Elasticity ($\text{Vol}_{\text{elasticity}}$)**:
   Relative volume multiplier $V_t / \text{SMA}_{20}(V)$ paired with price displacement per unit volume.
6. **Friction Drag Sensitivity ($\text{Friction}_{\text{drag}}$)**:
   $$\text{Friction Drag} = \frac{\text{Statutory Round-Trip Charge (₹82.40)}}{\mathbb{E}[\text{Intraday MFE (₹)}]}$$
   If expected maximum favorable excursion is only ₹120, friction consumes 68.7% of gross edge (classified as friction-sensitive).
7. **Tail Jump Frequency ($\text{Tail}_{\text{kurtosis}}$)**:
   Frequency of $> 3\sigma$ 1-minute price jumps over trailing 30 days.

---

## 4. Dynamic Compartment Evolution Mechanics

### 4.1 Continuous Belief State (Soft Membership)
Instead of hard categorical assignments, symbol $i$ maintains belief weights across active compartments $\mathcal{C}_k$:

$$w_{ik}(t) = \frac{\exp(-\gamma \|\mathbf{b}_i(t) - \mathbf{\mu}_k(t)\|^2)}{\sum_m \exp(-\gamma \|\mathbf{b}_i(t) - \mathbf{\mu}_m(t)\|^2)}$$

Where $\mathbf{\mu}_k(t)$ is the dynamic centroid of compartment $k$, and $\gamma$ is the bandwidth scaling parameter.

### 4.2 Compartment Lifecycle Operations
1. **Splitting**: When intra-cluster variance exceeds threshold $\theta_{\text{split}}$, a compartment bifurcates into distinct sub-states (e.g. "Low-Beta Reverter" vs "High-Beta Impulse Reverter").
2. **Merging**: When the Kullback-Leibler divergence between two centroids drops below $\theta_{\text{merge}}$, the compartments collapse into a unified state.
3. **Novelty Detection (Provisional Emergence)**:
   If a symbol exhibits $\min_k \|\mathbf{b}_i(t) - \mathbf{\mu}_k(t)\| > \theta_{\text{novel}}$, it is **not forced into a misfit category**. It enters a **Provisional Unclassified Cell**, accumulating evidence before a new compartment is legitimized.
4. **Dormancy**: If zero symbols occupy a compartment for $\ge 5$ consecutive sessions, it becomes dormant.

---

## 5. Policy Evaluation Matrix: Compartment × Regime Cell

Decision rules are tested within specific behavioural cells rather than across the aggregate market:

| Dynamic Compartment | Current Micro-Regime | Operational Hypothesis to Test |
| :--- | :--- | :--- |
| **Persistent Directional** | Confirmed Trend ($\Delta \text{NIFTY} > 0.5\%$) | Does freshness filtering ($F_{15} \ge -0.01$) preserve true breakout continuation while avoiding late entries? |
| **Persistent Directional** | Volatility Shock ($\text{ATR}_5 / \text{ATR}_{60} > 2.0$) | Does the gate prevent buying the high tick of exhaustion spikes? |
| **Mean-Reverting** | Strong Impulse ($|r_{5m}| > 1.5\%$) | Are momentum breakouts disproportionately reversed? Does fading entry outperform? |
| **Mean-Reverting** | Range-Bound ($|\Delta P| \le 0.2\%$) | Does the strategy over-trade and bleed statutory friction on noise? |
| **Volatility-Expansion**| Opening Breakout (Ticks #1–15) | Does Candidate C-1 (3-tick delay) or C-2 (15-min lookback) prevent opening gap-and-fade traps? |
| **Friction-Sensitive** | Low Expected Excursion | Are gross returns structurally insufficient to overcome ₹82.40 statutory charges? |

### Quantitative Reporting Standard per Cell:
Every cell with $N \ge 30$ observations must report:
- Realized Net Expectancy ($E_{\text{net}}$ in bps and ₹)
- Win Rate (%) and Payout Ratio (Gross Win / Gross Loss)
- Mark-to-Market Portfolio Sortino Ratio ($\tau = 0$)
- Maximum Intraday Drawdown (% of nominal capital base $W_0$)
- Flow Retention (%) and Average Holding Period (minutes)

---

## 6. Implementation & Governance Invariants

1. **Point-in-Time Only**: No symbol fingerprint may incorporate future bar information or whole-day statistics.
2. **Deterministic Provenance**: Every fingerprint calculation must log its exact input window and git commit hash.
3. **Adapter Isolation**: `adapters/chronosentiment` remains **100% frozen** (13/13 discipline invariants green). Exactly 0 adapter files modified.
4. **Authority Guard**: Production live order authority remains strictly **0.0%**.
