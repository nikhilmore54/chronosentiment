# Coralys Position Trajectory Resolution Validation Report

> [!IMPORTANT]
> **Governance & Operational Invariants**
> - **Coralys Engine (`v0.1.0`)**: 100% **FROZEN** (Zero score formula changes).
> - **Candidate Model**: Position Trajectory Interpreter **Config A ($K=3 \text{ bars}, G_{\text{max}}=0.70, \Delta M_{5,\text{min}}=-0.10$)** evaluated strictly as-is across independent resolution tracks.
> - **Resolution Independence**: 15m, 30m, and 1d datasets are evaluated independently on their native Yahoo Finance observation streams. $K=3$ bars means 3 observations of the respective resolution.

---

## 1. Consolidated Cross-Resolution Comparison Table

| Validation Metric | Track 15m | Track 30m | Track 1d |
|---|---:|---:|---:|
| **Available Date Range** | 2026-07-06 to 2026-09-25 | 2026-07-06 to 2026-09-25 | 1996-01-01 to 2026-09-25 |
| **Historical Period Span** | ~3 Months | ~3 Months | ~30 Years |
| **Number of Instruments** | 102 | 102 | 102 |
| **Total Observation Bars** | 157,852 | 78,234 | 587,735 |
| **Total Position Lifecycles** | 6,018 | 6,018 | 39,029 |
| **Coralys Baseline Entries** | 6,018 | 6,018 | 39,029 |
| **`CONTINUATION -> PULLBACK` (Pullback Events)** | **53** | **14** | **319** |
| **`PULLBACK -> CONTINUATION` (Recovery)** | **32** | **8** | **38** |
| **`PULLBACK -> REVERSAL` (Termination)** | **21** | **6** | **281** |
| **`CONTINUATION -> REVERSAL` (Direct Exit)** | 5,984 | 5,647 | 38,740 |
| **Pullback Recovery Share %** | **60.38%** | **57.14%** | **11.91%** |
| **Pullback Reversal Share %** | **39.62%** | **42.86%** | **88.09%** |
| **Event Sample Size Class** | *Exploratory (N=53)* | *Highly Sparse (N=14)* | *Materially Strong (N=319)* |
| **Mean Pullback Duration** | **1.32 15m-bars** | **1.43 30m-bars** | **2.89 1d-bars** |
| **Median Pullback Duration** | **1 15m-bar** | **1.0 30m-bar** | **3 1d-bars** |
| **Coralys Baseline Mean Return %** | +0.0280% | +0.0053% | +0.3093% |
| **Interpreter Mean Return %** | +0.0164% | +0.0075% | +0.5715% |
| **Mean Paired Net Improvement ($\Delta$)** | **-1.16 bps** | **+0.22 bps** | **+26.22 bps** |
| **Median Paired Net Improvement ($\Delta$)** | **0.00 bps** | **0.00 bps** | **0.00 bps** |
| **Worst Session Paired $\Delta$** | 2026-07-23 (-19.86 bps) | 2026-07-08 (-21.56 bps) | 2003-10-20 (-4,051.24 bps) |
| **Best Session Paired $\Delta$** | 2026-07-07 (+13.36 bps) | 2026-08-31 (+21.39 bps) | 2004-03-26 (+6,552.90 bps) |
| **Worst Single Position** | DABUR.NS (-898.51 bps) | CIPLA.NS (-308.93 bps) | CIPLA.NS (-9,332.55 bps) |
| **Cache Integrity Audit** | 100% Verified | 100% Verified | 100% Verified |

---

## 2. Core Methodological Synthesis & Structural Invariance

> **The validation demonstrates that the same qualitative position-trajectory lifecycle — CONTINUATION $\to$ PULLBACK, followed by either RECOVERY or REVERSAL — is observable across 15m, 30m, and daily observation streams. However, the transition frequencies and economic effect are strongly resolution-dependent.**

### Pullback Outcome Regime Divergence

```text
15m Track (N=53)  :  Recovery 60.38%   |   Reversal 39.62%   |  Mean Paired Δ:  -1.16 bps
30m Track (N=14)  :  Recovery 57.14%   |   Reversal 42.86%   |  Mean Paired Δ:  +0.22 bps
1d  Track (N=319) :  Recovery 11.91%   |   Reversal 88.09%   |  Mean Paired Δ: +26.22 bps
```

1. **Intraday vs. Daily Regime Contrast**:
   - On intraday observations (15m & 30m), recovery dominates observed pullbacks (~57%–60%).
   - On daily observations (1d), reversal dominates pullbacks (~88%).
   - This contrast confirms that while the qualitative lifecycle topology ($T_t$) exists at every resolution, temporal scale fundamentally alters the conditional transition dynamics of that topology.

2. **Sample Size Disparity & Statistical Weight**:
   - The 15m and 30m tracks contain sparse pullback event counts ($N=53$ and $N=14$ respectively).
   - The 1d track provides materially stronger statistical evidence with $N=319$ observed pullback events over 30 years.

3. **Temporal Coverage Confounding Analysis**:
   - The datasets differ in both temporal resolution (15m/30m vs 1d) AND historical period span (~3 months vs ~30 years).
   - Consequently, current evidence supports **qualitative cross-resolution structural persistence**, but does not establish resolution-independent predictive performance or parameter invariance.

---

## 3. Formal Agent Hand-off Synthesis

> **Coralys v0.1.0 remains frozen. Config A was evaluated without modification across independent 15m, 30m, and 1d Yahoo Finance streams covering 102 instruments. The same qualitative trajectory lifecycle — CONTINUATION $\to$ PULLBACK $\to$ RECOVERY or REVERSAL — was observed at all three resolutions. However, the conditional transition structure is strongly resolution-dependent: recovery dominates observed intraday pullbacks (~57%–60%), whereas reversal dominates daily pullbacks (~88%). The daily track provides substantially stronger statistical evidence because it contains 319 pullback events versus only 53 and 14 on 15m and 30m respectively. Config A's economic effect is also resolution-dependent: -1.16 bps mean paired delta on 15m, +0.22 bps on 30m, and +26.22 bps on daily data. Therefore the current evidence supports qualitative cross-resolution structural persistence, but does not establish resolution-independent predictive performance or parameter invariance. Coralys remains frozen; the next research question is whether the observed resolution dependence is a genuine temporal-scale effect or a consequence of differing sample periods and event frequencies.**

---

## 4. RP-600 — Time Machine Decision Surface & View Model Refinement

With the resolution validation complete, RP-600 completes the read-only decision surface layer above the frozen canonical substrate (`TIME-004`/`TIME-005`/`TIME-006`).

### Explicit Canonical Field Derivation Invariant
> **Every DecisionCard field must either be copied directly from the canonical DecisionState/evidence or be a deterministic presentation projection whose derivation is explicitly specified. No field may introduce new semantic inference.**

```text
Canonical DecisionState
        │
        ├── direction
        ├── trajectory (resolution-qualified)
        ├── resolutions (15m, 30m, 1d)
        ├── cross-resolution relationship
        ├── risk state
        ├── decision state
        └── evidence
                │
                ▼
        Deterministic View Model (DecisionCard)
                │
                ├── display labels
                ├── formatting
                ├── grouping
                └── timeline positioning
                │
                ▼
             UI (OBSERVE | REPLAY | COMPARE)
```

### Canonical DecisionCard Schema
```json
{
  "card_id": "card_RELIANCE.NS_1782186300",
  "instrument": "RELIANCE.NS",
  "timestamp": 1782186300,
  "datetime": "1782186300",
  "current_price": 1324.60,
  "resolutions": {
    "15m": {
      "label": "REVERSAL",
      "score": -0.6360
    },
    "30m": {
      "label": "INSUFFICIENT",
      "score": 0.5000
    },
    "1d": {
      "label": "REVERSAL",
      "score": -0.4561
    }
  },
  "cross_resolution_relationship": "PARTIAL AGREEMENT",
  "risk_state": "REVERSAL DEVELOPING",
  "decision_state": "EXIT",
  "evidence": [
    "• 15m trajectory crossed reversal boundary",
    "• 30m intermediate horizon observation insufficient",
    "• 1d macro horizon indicates structural reversal risk",
    "• 15m and 1d reversal confirmation, with 30m still insufficient"
  ]
}
```

### Product Surface Architecture & Hand-off Directive
```text
                  ┌─────────────────────┐
                  │ Historical / Live   │
                  │ Market Observations │
                  └──────────┬──────────┘
                             │
                             ▼
                  ┌─────────────────────┐
                  │ Existing Time       │
                  │ Machine Substrate   │
                  │ TIME-004/005/006    │
                  └──────────┬──────────┘
                             │
                             ▼
                    T₀ FIREWALL
                             │
                             ▼
                  ┌─────────────────────┐
                  │ Coralys v0.1.0      │
                  │ FROZEN              │
                  └──────────┬──────────┘
                             │
                             ▼
                  ┌─────────────────────┐
                  │ Config A            │
                  │ FROZEN              │
                  └──────────┬──────────┘
                             │
                             ▼
                  ┌─────────────────────┐
                  │ Canonical           │
                  │ DecisionState       │
                  └──────────┬──────────┘
                             │
                             ▼
                  ┌─────────────────────┐
                  │ DecisionTape        │
                  │ Immutable sequence  │
                  └──────────┬──────────┘
                             │
                             ▼
                  ┌─────────────────────┐
                  │ DecisionCard        │
                  │ Read-only View Model│
                  └──────────┬──────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
           OBSERVE         REPLAY         COMPARE
```

**Directive for UI Agent**:
> **Implement the OBSERVE / REPLAY / COMPARE UI against the canonical DecisionTape/View Model. Do not modify `time_machine/`, Coralys v0.1.0, Config A, scoring, thresholds, or decision semantics. Any missing information required by the UI must be traced back to the canonical substrate rather than reconstructed in the frontend.**
