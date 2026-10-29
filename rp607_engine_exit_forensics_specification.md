# RP-607 — Engine Exit Forensics Specification

**Status**: SPECIFIED / GATED (POST-RP-605 & RP-606)  
**Governance Standard**: RP-600 Pre-Check Verified | Orthogonal Trajectory-Economic Classification  
**Target Mechanism**: `ENGINE_EXIT` Signal Resolution, State Transitions & Post-Exit Counterfactual Lifecycle  
**Strict Invariant**: Zero `ENGINE_EXIT` Mutation or Threshold Hunting During Active RP-605 Prospective Validation  

---

## 1. Executive Summary & Forensic Motivation

Observations from canonical replay datasets and prospective shadow runs highlight a dual-behavioral characteristic in the `ENGINE_EXIT` mechanism:

- **Winning Engine Exits**: E.g., Muthoot (+₹10.50), Dabur (+₹25.99), ITC (+₹56.40), Persistent (+₹54.00), Coforge (+₹89.60), Sun Pharma (+₹70.20), ATGL (+₹182.61), HUL (+₹197.60), ICICI Bank (+₹53.20).
- **Losing Engine Exits**: E.g., Naukri (-₹197.50), Godrej Prop (-₹182.90), HCL Tech (-₹120.00), Oberoi Realty (-₹124.20), Maruti (-₹96.00), Siemens (-₹80.60).
- **Temporal Clustering**: Exits cluster tightly between **13m and 18m** holding duration.

> **Key Mechanical Insight**: `ENGINE_EXIT` is not a simple "losing mechanism", nor is it a simple "premature exit". The engine exits based on **state transitions** (e.g. pullbacks), but the economically relevant information is the **trajectory of that transition** (distinguishing temporary retracement from genuine structural deterioration).

> **Governance Invariant**: Observed winning/losing examples and 13–18m temporal clustering remain strictly illustrative motivations, NOT evidence for premature parameter shifts. The core analytical target of RP-607 is: *"Given the state that caused `ENGINE_EXIT`, what is the conditional distribution of subsequent economic and trajectory outcomes?"*

Prior findings in **RP-603** proved that blindly accelerating or delaying exit timing fails to improve net P&L. Therefore, RP-607 investigates **exit quality conditional on trajectory**, rather than exit timing alone.

---

## 1.1 Interim Failure Signal: Realized-P&L Conversion Failure & MFE Giveback

Live prospective validation snapshots reveal a systemic divergence between open-position mark-to-market (MTM) and closed-trade realized P&L:

- **NSE Ledger A**: Realized **-₹8,834**, while Unrealized MTM is **+₹4,687** (Total MTM -₹4,147).
- **NSE Ledger B**: Realized **-₹7,070**, while Unrealized MTM is **+₹4,800** (Total MTM -₹2,270).

> **RESEARCH STATUS DECLARATION**:  
> `ENGINE_EXIT` has an active, empirically observed failure mode requiring forensic resolution. Favorable open-position unrealized MTM is failing to convert into positive realized P&L upon exit. Active positions regularly achieve positive Maximum Favorable Excursion ($\text{MFE}$), but `ENGINE_EXIT` fires after the favorable excursion has evaporated into a realized loss.

### Objective Interim Taxonomy
To prevent evaluative bias before full multi-horizon reconstruction, RP-607 classifies all `ENGINE_EXIT` events into three mechanically explicit diagnostic buckets:

```text
                                  ENGINE_EXIT
                                       │
     ┌─────────────────────────────────┼─────────────────────────────────┐
     ▼                                 ▼                                 ▼
PROFIT_CONVERSION_FAILURE       PROTECTED_BY_EXIT                 OTHER_UNRESOLVED
 MFE_holding > 0                MFE_holding <= 0                  All remaining exits
 AND PnL_realized < 0           AND CF(+60m) < PnL_realized       (Profitable & equilibrium)
```

1. **`PROFIT_CONVERSION_FAILURE`**:
   $$\text{MFE}_{\text{holding}} > 0 \quad \text{AND} \quad \text{PnL}_{\text{realized}} < 0$$
   *Position achieved positive unrealized profit during holding, but closed at a realized loss. Indicates failure to convert favorable open-position economics into realized returns.*

2. **`PROTECTED_BY_EXIT`**:
   $$\text{MFE}_{\text{holding}} \le 0 \quad \text{AND} \quad \text{PnL}_{\text{counterfactual}, +60m} < \text{PnL}_{\text{realized}}$$
   *Position experienced adverse movement from entry, and `ENGINE_EXIT` protected capital relative to the +60m counterfactual.*

3. **`OTHER_UNRESOLVED`**:
   *All remaining exits (including profitable completed exits and near-equilibrium exits).*

### Giveback Diagnostic Inquiry (₹161.96 Average Giveback)
The parallel exit diagnostic revealed an average **₹161.96 MFE giveback** across 113 NSE profit-conversion failures ($31.7\%$ of completed exits). 

Rather than jumping to conclusions about delaying exits, RP-607 frames the central diagnostic question:

> **"Was the giveback associated with a recognizable trajectory deterioration before `ENGINE_EXIT`, or did the engine continue to hold a deteriorating position until an already-earned favorable excursion had largely disappeared?"**

### Core Investigative Target
> **"At what point between peak MFE and `ENGINE_EXIT` did the position transition from favorable open economics toward realized loss, and what Coralys trajectory state was observable at that point?"**

RP-607 will establish whether the surrendered opportunity was systematically recoverable and whether the pre-exit Coralys state contained information that could have distinguished those cases, while **live RP-605 prospective validation engines remain 100% untouched**.

---

## 2. Governance Status & Gated Lineage

RP-607 is formally ratified as **SPECIFIED / METHODOLOGICALLY LOCKED**.

### Parallel Observation Architecture (Zero Feedback)

To validate forensic pipeline machinery and monitor accumulating trade lifecycles without contaminating the live experiment, RP-607 operates as an **accumulating read-only observer**:

```text
                    RP-605
              FROZEN PROSPECTIVE
                     │
          ┌──────────┴──────────┐
          │                     │
          ▼                     ▼
   RP-606 READ-ONLY       RP-607 READ-ONLY
   ACCUMULATING            ACCUMULATING
   OBSERVER                OBSERVER
          │                     │
          └──────────┬──────────┘
                     │
              STRICT NO-FEEDBACK
             DATA CUTOFF SNAPSHOTS
                     │
                     ▼
              RP-605 COMPLETION
                     │
                     ▼
          FINAL FROZEN RECONSTRUCTION
```

### Governance Control Matrix

| Gate | Status | Governance Constraint |
| :--- | :---: | :--- |
| **RP-605 Prospective Validation** | 🟢 **ACTIVE / FROZEN** | 100% frozen execution across 102 NSE, 15 Crypto, 25 US tickers. |
| **RP-606 Entry Forensics** | 🔍 **READ-ONLY ACCUMULATING** | Parallel read-only snapshot analysis; zero live feedback. |
| **RP-607 Exit Forensics** | 🔍 **READ-ONLY ACCUMULATING** | Parallel read-only snapshot analysis from $T_{\text{entry}}$; zero live feedback. |
| **`ENGINE_EXIT` Mutation** | 🚫 **FORBIDDEN** | Zero modification of exit rules or thresholds during live validation. |
| **Threshold Hunting** | 🚫 **FORBIDDEN** | No speculative parameter tuning without mechanism identification. |
| **Forensic Reconstruction** | ⏳ **CONTINUOUS / FINAL POST-RP-605** | Reconstructed strictly from event journal snapshots; final run post-RP-605. |
| **Candidate Exit Mechanism** | ⏳ **NOT YET DEFINED** | Defined strictly downstream of empirical joint distribution evidence. |

### Locked Research Constrained Question
> **"Which observable pre-exit trajectory configurations distinguish economically protective exits from exits that terminate positions while meaningful opportunity remains in the observed market path?"**

### Strict Pre-Reconstruction Prohibition Matrix
Until the forensic population has been fully reconstructed from $T_{\text{entry}}$ post-RP-605, the following actions are strictly **FORBIDDEN**:

1. 🚫 **No changing `ENGINE_EXIT`** logic or triggers in live codebase.
2. 🚫 **No changing thresholds** or parameter tuning.
3. 🚫 **No introducing holding-period rules** or arbitrary duration gates.
4. 🚫 **No selecting a subset of exits** because they look problematic (full population analysis required).
5. 🚫 **No using Ledger A/B differences** as opportunity-cost measurement.
6. 🚫 **No cherry-picking LO horizons** ($LO_{15}, LO_{30}, LO_{60}$) to favor specific outcomes.
7. 🚫 **No converting the 13–18m temporal observation** into a timing hypothesis.
8. 🚫 **No turning the ₹161.96 MFE giveback** into an exit rule.

**First Legitimate Intervention Point**: Candidate mechanism design is strictly prohibited until RP-607 establishes a reproducible, empirical distributional relationship between:
$$\text{Pre-Exit Trajectory State} \longrightarrow \text{ENGINE\_EXIT} \longrightarrow \text{Post-Exit Economic Outcome}$$

---

## 3.0 Common Temporal Anchor & Two-Clock Protocol

```text
COMMON TEMPORAL ANCHOR
        │
        ▼
T_entry = POSITION OPEN
        │
        ├── Holding-period forensic clock
        │     ├── trajectory evolution
        │     ├── MFE
        │     ├── MAE
        │     ├── state transitions
        │     └── pre-exit deterioration
        │
        ▼
T_exit = ENGINE_EXIT
        │
        ├── Opportunity-cost clock
        │     ├── +15m
        │     ├── +30m
        │     └── +60m
        │
        ▼
Observed post-exit opportunity
```

### Methodological Rule
> Every `ENGINE_EXIT` forensic record MUST use the position entry timestamp (`T_entry`) as the common starting point for reconstruction of the complete trade lifecycle.

All holding-period measurements — including trajectory states, score evolution, MFE, MAE, signal episode age, and state transitions — are indexed relative to `T_entry`.

The `ENGINE_EXIT` timestamp (`T_exit`) establishes a **second temporal anchor** for opportunity-cost analysis. All post-exit counterfactual measurements are indexed relative to `T_exit` at the predefined +15m, +30m, and +60m horizons.

### Two-Clock Analytical Architecture

| Clock | Anchor | Purpose |
| :--- | :--- | :--- |
| **Position/Lifecycle Clock** | `T_entry` | Explain how the position evolved toward `ENGINE_EXIT` |
| **Opportunity Clock** | `T_exit` | Measure economically observable opportunity remaining after `ENGINE_EXIT` |

### Methodological Chronology Flow
```text
ALL ENGINE_EXITS
      ↓
Common T_entry
      ↓
Reconstruct complete lifecycle
      ↓
Reach T_exit
      ↓
Measure fixed post-exit horizons
      ↓
Apply economic / trajectory / context classifications
      ↓
Compare distributions
```

**Population Invariant:** No trade may enter the RP-607 analysis solely because it was identified as a `PROFIT_CONVERSION_FAILURE`. The complete `ENGINE_EXIT` population is reconstructed from `T_entry`; the diagnostic taxonomy is applied only after reconstruction.

---

## 3.1 Four-Timestamp Lifecycle Alignment & Trigger Latency Protocol

To distinguish **profit retreat** during holding from **exit-trigger delay**, every `ENGINE_EXIT` forensic record MUST align four explicit timestamps:

```text
T_entry (Position Open)
   │
   ├── 1. T_peak       : Timestamp when MFE was first achieved during holding
   │
   ├── 2. T_retreat    : Timestamp when unrealized PnL first retreated below 50% of peak MFE
   │
   ├── 3. T_condition  : Timestamp when engine exit condition first evaluated to TRUE
   │
   └── 4. T_exit       : Execution timestamp (actual trade close)
```

### The Four Timestamps

| Timestamp | Symbol | Definition & Measurement Rule |
| :--- | :---: | :--- |
| **Peak Timestamp** | $T_{\text{peak}}$ | Timestamp of maximum favorable excursion price during $[T_{\text{entry}}, T_{\text{exit}}]$. |
| **Retreat Timestamp** | $T_{\text{retreat}}$ | First timestamp after $T_{\text{peak}}$ where unrealized PnL dropped below 50% of peak MFE (or crossed $\le 0$). |
| **Condition Timestamp** | $T_{\text{condition}}$ | First timestamp where decision state evaluated to `EXIT` (or score crossed exit threshold). |
| **Execution Timestamp** | $T_{\text{exit}}$ | Exact trade close timestamp recorded in event journal. |

### Diagnostic Metrics

1. **Trigger Latency**:
   $$\Delta T_{\text{trigger}} = T_{\text{exit}} - T_{\text{condition}}$$
   *Measures execution/processing delay after the engine's exit condition evaluated to TRUE ($0.00\text{s}$ empirically verified).*

2. **Post-Retreat Delay**:
   $$\Delta T_{\text{retreat}} = T_{\text{exit}} - T_{\text{retreat}}$$
   *Measures the holding duration between initial profit retreat and actual exit trigger ($5.33\text{m}$ average empirical delay).*

3. **Peak Giveback vs Retreat Giveback**:
   $$\text{MFE Giveback} = \text{PnL}_{\text{peak}} - \text{PnL}_{\text{exit}}$$

---

## 4. Four-Layer Diagnostic Architecture

RP-607 evaluates every `ENGINE_EXIT` across **four complementary analytical layers**, forming a clean diagnostic chain:

```text
ENGINE_EXIT
    │
    ├── 1. What happened economically? (Economic Outcome Layer)
    │
    ├── 2. What happened after exit? (Post-Exit Trajectory Layer)
    │
    ├── 3. What trajectory context existed at exit? (Exit Context Layer)
    │
    └── 4. What opportunity remained in the observed market path? (Opportunity-Cost Layer)
```

### Layer 1: Economic Outcome
- **`PROTECTED_LOSS`**: Exit successfully limited loss ($\text{PnL}_{+60m} < \text{PnL}_{\text{exit}}$).
- **`SACRIFICED_GAIN`**: Exit terminated a trade that subsequently reached net profit ($\text{PnL}_{+60m} > 0 > \text{PnL}_{\text{exit}}$).
- **`APPROXIMATELY_NEUTRAL`**: Exit occurred near equilibrium ($|\text{PnL}_{+60m} - \text{PnL}_{\text{exit}}| < \epsilon$).
- **`EXTREME_LOSS_AVOIDED`**: Exit prevented catastrophic tail drawdown.

### Layer 2: Post-Exit Trajectory (Multi-Horizon: +15m / +30m / +60m)
- **`CONTINUED_ADVERSE`**: Asset price continued deteriorating post-exit.
- **`RECOVERED_TO_PROFIT`**: Price retraced and crossed profit threshold.
- **`CONTINUED_FAVOURABLE`**: Price continued advancing in original trade direction.
- **`STRUCTURAL_REVERSAL`**: Exit cleanly preceded macro trend inversion.

### Layer 3: Exit Context & Trigger Trajectory
- **`INTRADAY_DETERIORATION`**: Single-resolution score deterioration.
- **`MULTI_RESOLUTION_DETERIORATION`**: Concordant deterioration across 15m, 30m, and 1d scores.
- **`TRANSIENT_DIVERGENCE`**: Short-term 15m noise against stable 30m/1d trend.
- **`PULLBACK_RETRACEMENT`**: Normal counter-trend pull-back within valid setup.

### Layer 4: Opportunity-Cost & MFE Giveback Quantitative Metrics Layer

> **Critical Methodological Invariant**:  
> **Do NOT define "Lost Opportunity" as $\text{Ledger A PnL} - \text{Ledger B PnL}$.**  
> Ledger A vs Ledger B can tell us whether the qualification gate changes outcomes, but it cannot tell us whether an individual `ENGINE_EXIT` terminated a recoverable position. Opportunity loss MUST be computed directly from the **actual observed market path** following an exit.

#### Metric Hierarchy Protocol

```text
PRIMARY (Horizon-Specific Checkpoints)
  ├── LO_15 = PnL_{t+15m} - PnL_exit
  ├── LO_30 = PnL_{t+30m} - PnL_exit
  └── LO_60 = PnL_{t+60m} - PnL_exit

SECONDARY / HINDSIGHT DIAGNOSTIC
  └── LO_max = max(PnL_{t+15m}, PnL_{t+30m}, PnL_{t+60m}) - PnL_exit

SUPPORTING DIAGNOSTIC
  └── MFE Giveback = MFE_holding - PnL_exit
```

1. **Primary Opportunity Metrics ($LO_{15}, LO_{30}, LO_{60}$)**:
   $$LO_{h} = \text{PnL}_{t+h} - \text{PnL}_{\text{exit}} \quad \text{for } h \in \{15m, 30m, 60m\}$$
   *Clean, horizon-specific economic checkpoint measurements with a predefined time horizon and precise interpretation.*

2. **Secondary / Hindsight Diagnostic Metric ($LO_{\max}$)**:
   $$LO_{\max} = \max(PnL_{t+15m}, PnL_{t+30m}, PnL_{t+60m}) - PnL_{\text{exit}}$$
   *Diagnostic hindsight ceiling for potential recovery. It is a useful diagnostic tool, but must NOT be interpreted as money that could necessarily have been captured in real execution due to latency, slippage, and execution friction.*

3. **Supporting Diagnostic Metric ($\text{MFE Giveback}$)**:
   $$\text{MFE Giveback} = \text{MFE}_{\text{holding}} - \text{PnL}_{\text{exit}}$$
   *Quantifies open profit surrendered during holding prior to `ENGINE_EXIT` (e.g. ₹161.96 average giveback across 113 NSE profit-conversion failures).*

---

## 4. Forensic Reconstruction Schema

For every `ENGINE_EXIT` generated across the canonical replay and RP-605 prospective validation journals, RP-607 will generate the multi-horizon diagnostic vector:

```json
{
  "trade_id": "STRING",
  "symbol": "STRING",
  "side": "LONG | SHORT",
  "entry_timestamp_utc": "ISO-8601",
  "exit_timestamp_utc": "ISO-8601",
  "holding_duration_mins": "INT",
  "f15_at_entry": "FLOAT",
  "entry_state": {
    "score_15m": "FLOAT",
    "score_30m": "FLOAT",
    "score_1d": "FLOAT",
    "resolution_alignment": "CONCORDANT | DISCORDANT",
    "signal_episode_age_mins": "INT"
  },
  "exit_state": {
    "score_15m": "FLOAT",
    "score_30m": "FLOAT",
    "score_1d": "FLOAT",
    "score_velocity_15m": "FLOAT",
    "cross_resolution_propagation": "FLOAT"
  },
  "excursion_metrics": {
    "mfe_during_holding": "FLOAT",
    "mae_during_holding": "FLOAT"
  },
  "realized_exit_pnl": "FLOAT",
  "multi_horizon_counterfactual_pnl": {
    "pnl_plus_15m": "FLOAT",
    "pnl_plus_30m": "FLOAT",
    "pnl_plus_60m": "FLOAT"
  },
  "opportunity_metrics": {
    "lo_15m": "FLOAT",
    "lo_30m": "FLOAT",
    "lo_60m": "FLOAT",
    "lo_max_hindsight": "FLOAT",
    "mfe_giveback": "FLOAT"
  },
  "orthogonal_descriptors": {
    "economic_outcome": "PROTECTED_LOSS | SACRIFICED_GAIN | APPROX_NEUTRAL | EXTREME_LOSS_AVOIDED",
    "post_exit_trajectory": "CONTINUED_ADVERSE | RECOVERED_TO_PROFIT | CONTINUED_FAVOURABLE | STRUCTURAL_REVERSAL",
    "exit_context": "INTRADAY_DETERIORATION | MULTI_RES_DETERIORATION | TRANSIENT_DIVERGENCE | PULLBACK_RETRACEMENT"
  }
}
```

---

## 5. Core Output & Success Criteria

The primary deliverable of **RP-607** is **not** a proposed parameter shift (e.g. "move threshold from X to Y").

### Three-Question Evaluation Hierarchy

```text
1. WHAT HAPPENED DURING THE POSITION LIFECYCLE? (Descriptive Reconstruction)
   ├── Position Clock (T_entry → T_exit)
   ├── Score evolution & cross-resolution propagation
   └── MFE & MAE accumulation
           │
           ▼
2. WHAT HAPPENED AFTER THE ENGINE EXITED? (Descriptive Reconstruction)
   ├── Opportunity Clock (T_exit → +15m / +30m / +60m)
   ├── Horizon checkpoint metrics (LO_15, LO_30, LO_60)
   └── Post-exit price trajectory
           │
           ▼
3. WHAT OBSERVABLE STATE, IF ANY, DISTINGUISHES THOSE TWO POPULATIONS? (Mechanism-Identification Gate)
   ├── Pre-exit Coralys trajectory state vs post-exit economic outcome
   └── Empirical joint distribution evidence
```

> **Governance Principle**: Questions 1 and 2 produce **descriptive observations** (e.g. holding duration clustering, MFE giveback magnitude, post-exit recovery rate). Descriptive observations are **FORBIDDEN** from directly becoming execution rules. Only empirical evidence answering **Question 3** can justify candidate exit mechanism design.

Once empirical discrimination under Question 3 is established post-RP-605, candidate exit qualification rules can be designed and tested under counterfactual simulation before any live implementation.


