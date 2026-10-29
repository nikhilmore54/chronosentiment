# RP-607-RANGE-001: Historical Range-Based Stock Compartmentalisation Specification

**Document Identifier**: `RP-607-RANGE-001`  
**Protocol Series**: `RP-607 (Asset Morphology & Range Compartmentalisation)`  
**Date**: 2026-10-22  
**Status**: REGISTERED SPECIFICATION & RESEARCH CONTRACT (PHASE 1)  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Research Motivation & Core Distinction

`RP-607-RANGE-001` establishes a new research line distinct from the rejected `RP-605-BEH-001` gating hypothesis:

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    THE FUNDAMENTAL DISTINCTION                                       |
+──────────────────────────────────────────────────+───────────────────────────────────────────────────+
| RP-605-BEH-001: Situational Trade Gating         | RP-607-RANGE-001: Structural Asset Characterisation   |
+──────────────────────────────────────────────────+───────────────────────────────────────────────────+
| • Asked: "Is this stock's immediate short-term   | • Asks: "What kind of price movement does this    |
|   momentum a favourable entry condition?"        |   stock historically exhibit over rolling horizons|
| • Evaluated 15m/30m intraday bars.               |   and where is it currently trading relative to   |
| • Suffered momentum climax traps (buying tops).  |   its own historical range envelope?"             |
| • Falsified as a single-trade filter.            | • Evaluates structural asset morphology.          |
+──────────────────────────────────────────────────+───────────────────────────────────────────────────+
```

### Confirmation of Option C: The Decoupled Dual-Layer Architecture:
Following stakeholder alignment, this specification implements **Option C**:
1. **Layer 1 (Macro Stock Profile)**: Classifies the asset's characteristic historical movement DNA (Range magnitude, efficiency, reversion, stability, and friction burden) strictly from historical bars preceding the session.
2. **Layer 2 (Situational Range Position)**: Classifies the asset's point-in-time location relative to its own recent multi-week and intraday high-low envelope.

This decouples the structural nature of the asset (*what kind of mover it tends to be*) from its tactical state (*where it is currently positioned*), preventing the conflation of volatility with edge.

---

## 2. Layer 1: The Five Structural Movement Dimensions

Computed on rolling 20-day and 60-day historical bars strictly prior to the current trading date:

| Dimension | Quantitative Metric | Economic Meaning | Formula |
| :--- | :--- | :--- | :--- |
| **1. Range Magnitude** | Normalized ATR ($\text{NATR}_{20}$) | How much the stock typically moves relative to its price. | $\text{NATR}_{20} = \frac{\text{ATR}_{20}}{P_t} \times 10,000\text{ bps}$ |
| **2. Directional Efficiency** | 20-Day Displacement Ratio | Path efficiency: trend persistence vs. choppy oscillation. | $\text{Eff}_{20} = \frac{\|P_t - P_{t-20}\|}{\sum_{\tau=1}^{20} \text{TR}_\tau} \in [0, 1]$ |
| **3. Reversion Tendency** | Return Autocorrelation ($\text{AR}_1$) | Whether daily price shocks tend to persist or bounce. | $\text{AR}_1 = \text{Corr}(r_t, r_{t-1})$ over 60 days |
| **4. Range Stability** | Range Coefficient of Variation | Whether the asset's volatility profile is stationary. | $\text{CV}_{\text{range}} = \frac{\sigma(\text{TR}_{20})}{\mu(\text{TR}_{20})}$ |
| **5. Cost-Adjusted Opportunity** | Friction Coverage Ratio | Multiple of statutory costs ($\bar{C} = 9\text{ bps}$) covered by range. | $\text{Opp}_{\text{ratio}} = \frac{\text{NATR}_{20}}{9.00\text{ bps}}$ |

---

## 3. Pre-Declared Compartments & Positional Zones

To prevent retrospective data snooping, compartment definitions are locked prior to evaluating trading outcomes:

### 3.1 Layer 1: Structural Stock Compartments
1. **Compartment I: High-Range Directional Movers ($\mathcal{C}_{\text{dir}}$)**:
   - Criteria: $\text{NATR}_{20} \ge 220\text{ bps}$, $\text{Eff}_{20} \ge 0.15$, $\text{AR}_1 \ge -0.10$.
   - Characteristics: Wide price swings that exhibit directional trending follow-through.
2. **Compartment II: High-Range Volatile Oscillators ($\mathcal{C}_{\text{osc}}$)**:
   - Criteria: $\text{NATR}_{20} \ge 220\text{ bps}$, $\text{Eff}_{20} < 0.12$, $\text{AR}_1 \le -0.15$.
   - Characteristics: Large daily swings that frequently reverse; strong mean-reversion tendencies.
3. **Compartment III: Stable Moderate Movers ($\mathcal{C}_{\text{stab}}$)**:
   - Criteria: $160\text{ bps} \le \text{NATR}_{20} < 220\text{ bps}$, $\text{CV}_{\text{range}} \le 0.45$.
   - Characteristics: Dependable, predictable range expansion without erratic volatility spikes.
4. **Compartment IV: Low-Movement / Friction-Bound Scrips ($\mathcal{C}_{\text{fric}}$)**:
   - Criteria: $\text{NATR}_{20} < 160\text{ bps}$ or $\text{Opp}_{\text{ratio}} < 18\times$.
   - Characteristics: Narrow price envelope where statutory friction ($9\text{ bps}$) consumes $> 6\%$ of typical daily movement.

### 3.2 Layer 2: Situational Range Position Zones
Normalized location within trailing 20-day high-low envelope: $\text{Pos}_{20\text{d}} = \frac{P_t - L_{20\text{d}}}{H_{20\text{d}} - L_{20\text{d}}} \in [0, 1]$
- **Zone $\mathcal{Z}_{\text{upper}}$ (Upper Decile / Resistance)**: $\text{Pos}_{20\text{d}} \ge 0.85$.
- **Zone $\mathcal{Z}_{\text{mid}}$ (Channel Interior / Equilibrium)**: $0.35 \le \text{Pos}_{20\text{d}} \le 0.65$.
- **Zone $\mathcal{Z}_{\text{lower}}$ (Lower Decile / Support)**: $\text{Pos}_{20\text{d}} \le 0.15$.

---

## 4. Phase 1 Execution Protocol (Descriptive Attribution Only)

Phase 1 is strictly restricted to **classification, stability analysis, and descriptive performance mapping**. It is **not** an authorization to tune trading rules or optimize exits.

### Step 1: Historical Profile Reconstruction
- Construct rolling daily profiles for all 99 symbols using historical bars strictly prior to each trading session.
- Verify zero forward leakage across all five dimensions.

### Step 2: Compartment Stability & Dwell Time Measurement
- Measure the transition matrix $\mathbf{T}_{ij} = \mathbb{P}(\mathcal{C}_j(t) \mid \mathcal{C}_i(t-1))$.
- Measure average compartment dwell time (how many weeks a symbol stays in a single compartment before migrating).
- Identify which symbols are structurally stationary vs. dynamically fluid.

### Step 3: Descriptive Performance Mapping (Compartment × Zone Matrix)
- Map the frozen baseline strategy's completed trades ($N = 2,029$) into the $4 \times 3$ matrix ($\mathcal{C}_i \times \mathcal{Z}_j$).
- Evaluate gross return, net expectancy, session-clustered variance, and trade coverage across each cell.

### Step 4: Pre-Declared Falsification Gate
- If no compartment/zone cell produces a gross return statistically distinguishable from zero and sufficient to cover statutory friction ($\ge 14.0\text{ bps}$ after clustering), the hypothesis will be **rejected without trading rule tuning**.
- If a specific cell shows robust gross expectancy, the hypothesis will be **frozen and prospectively validated on genuinely fresh sessions under RP-606-FRESH-001**.

---

## 5. Governance Constraints

1. **Adapter Immutability**: `adapters/chronosentiment` remains 100% frozen (13/13 discipline invariants green).
2. **Authority Ceiling**: Strictly **0.0% Live Order Authority** (Observational research only).
3. **Closed Historical Holdout**: The Sep 28 – Oct 01 dataset is restricted to descriptive attribution; no optimization or parameter fitting against it.
