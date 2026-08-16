# P.E.3 — Coralys Execution Model Specification

**Status:** FROZEN — pending artifact hash computation (CORALYS_TARGET_ARTIFACT_PRESENT = false until hash is set)  
**Parent:** CS-P-006-P.E.3  
**Depends on:** C3-002 (frozen), P.E.2 control (frozen)  
**Does not:** retune C3-002, start C.3-G, run Search #3, path-optimize targets, use future information

---

## 1. Purpose

P.E.3 asks a single, bounded question:

> **Given exactly the same certified state and C3-002 direction, can a Coralys-derived execution model generate deterministic target and risk parameters that produce better subsequent execution outcomes than the fixed +5% control?**

P.E.3 does **not** ask whether Coralys can improve direction. Direction remains C3-002.

P.E.3 v0 does **not** use a Coralys search or optimization to discover parameters. It applies a deterministic execution heuristic with frozen design parameters. Whether those parameters outperform the +5% control is the question CS-P-007 will answer.

---

## 2. Experiment structure

| | Control (P.E.2) | Treatment (P.E.3) |
|---|---|---|
| Direction | C3-002 | C3-002 |
| Target | +5.0% fixed | Coralys-derived |
| Risk boundary | Not authorized | Coralys-derived |
| Maximum hold | 20 market sessions | 20 market sessions |
| Direction information | bars ≤ T | bars ≤ T |
| Execution information | fixed contract | bars ≤ T + entry price at E |
| Future path | Same | Same |

The comparison is valid because both arms use the same direction policy and the same direction information boundary (bars ≤ T). The execution information boundary differs: P.E.2 uses a fixed contract; P.E.3 uses bars ≤ T plus the actual entry price at E.

---

## 3. Two information boundaries

P.E.3 introduces two distinct information boundaries:

### Decision boundary — T

```
T = last certified session close
```

The direction is sealed at T. Only information from bars ≤ T may influence the direction.

```
Direction = f(state ≤ T)
```

### Execution boundary — E

```
E = next eligible session open
```

The execution intent is sealed at E. It may use:

```
Execution Intent = f(state ≤ T, direction, entry_open(E))
```

Absolutely forbidden after E:

```
Execution Intent ≠ f(intraday prices after E)
Execution Intent ≠ f(E close)
Execution Intent ≠ f(any subsequent OHLC)
```

### Two distinct seals

```
DIRECTION SEAL (at T)
    direction = LONG / SHORT
    policy = C3-002
    state_hash = ...
    decision_information_cutoff = T

EXECUTION SEAL (at E)
    entry_price = actual fill at E open
    target_price = Coralys(entry_price, atr_14_at_T, tmv_at_T)
    risk_boundary = Coralys(entry_price, atr_14_at_T, tmv_at_T)
    entry_source = NEXT_SESSION_OPEN
    execution_information_cutoff = E
```

---

## 4. Authorized inputs

The Coralys execution model may use **only** the following inputs:

```
From bars ≤ T:
    certified_tmv_labels        — Trend / Momentum / Volatility
    state_hash                  — deterministic state identity
    atr_14                      — ATR(14), price units, magnitude only
    frozen_coralys_target_artifact_id

    NOTE: roc_20 is NOT used in v0. It is reserved for coralys-exec-v1.
    Do not add it to v0 without a new artifact hash.

From entry session open (E):
    entry_price                 — certified execution price at E open
    entry_source                — "NEXT_SESSION_OPEN"
    execution_price_source      — "SESSION_OPEN" (v0 historical replay)

From direction seal (at T):
    c3_002_direction            — LONG / SHORT
```

---

## 4b. Entry price abstraction

`entry_price` is the **certified execution price** for the entry session.

For v0 historical replay:
```
certified execution price = session open
entry_source = NEXT_SESSION_OPEN
```

For future live trading:
```
certified execution price = actual broker fill
entry_source = BROKER_FILL
```

The Coralys model itself does not change between these two cases. The abstraction is in the entry source, not the model.

---

## 5. Forbidden inputs

```
bars_after_T
realized_future_return
realized_V
target_hit
path_optimized_hit_rate
per_name_hindsight_target
coralys_evolved_after_T
new_indicator_families
intraday_price_after_open
E_close
any_bar_after_E_open
```

---

## 6. Coralys Execution Model v0 — ATR-anchored, TMV-scaled

### 6.1 Model identity

```
model_id:      coralys-exec-v0
methodology:   atr_anchored_tmv_scaled
version:       0.1.0
```

### 6.2 Multiplier provenance

The multipliers below are **frozen design parameters for v0**. They are not the output of a Coralys search or optimization. They represent a reasonable ATR-based execution heuristic.

Whether they outperform the +5% control is the question CS-P-007 will answer.

The frozen artifact must state:

> "The multipliers in coralys-exec-v0 are frozen design parameters. They were not discovered by a Coralys search. They represent a deterministic ATR/TMV execution heuristic. Whether they outperform the +5% control is the question CS-P-007 will answer."

### 6.3 Target derivation

```
base_target_pct = atr_14 / entry_price

tmv_multiplier (frozen design parameter):
    Bullish / Positive  → 2.0
    Bullish / Negative  → 1.5
    Bearish / Positive  → 1.5
    Bearish / Negative  → 1.0

target_pct = base_target_pct × tmv_multiplier

Clamp: min 0.02 (2%), max 0.15 (15%)
```

For SHORT decisions, the target is applied symmetrically downward.

### 6.4 Risk boundary derivation

```
base_risk_pct = atr_14 / entry_price

risk_multiplier (frozen design parameter):
    Bullish / Positive  → 1.0
    Bullish / Negative  → 0.75
    Bearish / Positive  → 0.75
    Bearish / Negative  → 0.5

risk_pct = base_risk_pct × risk_multiplier

Clamp: min 0.01 (1%), max 0.08 (8%)

LONG:  risk_boundary = entry_price × (1 - risk_pct)
SHORT: risk_boundary = entry_price × (1 + risk_pct)
```

### 6.5 ATR unavailable or zero

**Do not fall back to +5%.** That would blend the treatment with the P.E.2 control.

```
ATR unavailable or zero
        ↓
CoralysExecutionResult::Invalid {
    reason: "atr_14 unavailable or zero — cannot derive execution intent",
    instrument: ...,
    decision_time: ...,
}
```

The position is recorded as `NO_CORALYS_EXECUTION` and excluded from the P.E.3 treatment sample.

### 6.6 LONG/SHORT symmetry invariant

For identical state and entry price:

```
|LONG target_price - entry_price|  = |SHORT target_price - entry_price|
|LONG risk_boundary - entry_price| = |SHORT risk_boundary - entry_price|

Example (entry = 1000, target_pct = 6%, risk_pct = 3%):

LONG:
    target_price    = 1060
    risk_boundary   = 970

SHORT:
    target_price    = 940
    risk_boundary   = 1030
```

### 6.7 Determinism guarantee

```
same entry_price
+ same atr_14 (from bars ≤ T)
+ same tmv_state
+ same frozen artifact
= same target_pct
= same risk_pct
= same target_price
= same risk_boundary
```

---

## 7. Output schema

```rust
CoralysExecutionIntent {
    instrument: String,
    decision_time: String,                  // T — when C3-002 decided
    decision_information_cutoff: String,    // T — bars ≤ T used for state/ATR
    entry_time: String,                     // E — entry session open timestamp
    execution_information_cutoff: String,   // E — entry open is the last allowed input
    entry_source: String,                   // "NEXT_SESSION_OPEN"
    direction: String,                      // LONG / SHORT
    entry_price: f64,                       // actual fill price at E open
    target_pct: f64,                        // Coralys-derived
    target_price: f64,                      // entry_price × (1 ± target_pct)
    target_basis: String,                   // "atr_anchored_tmv_scaled (frozen v0 parameters)"
    risk_pct: f64,                          // Coralys-derived
    risk_boundary: f64,                     // entry_price × (1 ∓ risk_pct)
    risk_basis: String,                     // "atr_anchored_tmv_scaled (frozen v0 parameters)"
    atr_14_at_t: f64,                       // ATR(14) from bars ≤ T
    tmv_state: String,                      // "Bullish / Positive" etc.
    state_hash: String,                     // certified state hash
    maximum_hold_sessions: u32,             // 20
    coralys_model_id: String,               // "coralys-exec-v0"
    coralys_model_version: String,          // "0.1.0"
    coralys_artifact_hash: String,          // SHA256 of frozen model spec
    intent_hash: String,                    // SHA256 of sealed intent
    sealed_at_entry: bool,                  // true — sealed at E, not at T
    direction_sealed_at_t: bool,            // true — direction was sealed at T by C3-002
}
```

---

## 8. Integrity tests required before freeze

- [ ] **Determinism test** — same inputs → same outputs across 100 runs
- [ ] **No-lookahead test** — strip future bars, verify ATR unchanged, verify output unchanged
- [ ] **Poison test** — inject future bar, verify `bars_at_or_before` filters it, verify output unchanged
- [ ] **LONG/SHORT symmetry test** — verify `|target_price - entry_price|` is equal for LONG and SHORT
- [ ] **Clamp test** — verify min/max bounds are enforced
- [ ] **ATR zero test** — verify `CoralysExecutionResult::Invalid` is returned, not a fallback
- [ ] **ATR unavailable test** — verify `CoralysExecutionResult::Invalid` is returned
- [ ] **Pathological volatility test** — very high ATR → clamped to 15%
- [ ] **Gap scenario test** — entry_price >> previous close → ATR still from bars ≤ T
- [ ] **Multiplier provenance test** — artifact hash matches frozen spec
- [ ] **entry_source field test** — verify `entry_source = "NEXT_SESSION_OPEN"`
- [ ] **execution_information_cutoff test** — verify cutoff = entry session open timestamp

---

## 9. What P.E.3 does NOT prove

- That Coralys-derived targets are better than +5% (that is CS-P-007)
- That ATR-anchored targets are optimal
- That the TMV multipliers are correct or learned
- That the risk boundary prevents losses
- That the model generalises beyond the research universe
- That Coralys "discovered" the multipliers (they are frozen design parameters)

P.E.3-A proves only that the model is **deterministic, no-lookahead, and reproducible**.

P.E.3-B (historical treatment) provides the evidence for CS-P-007 comparison.

---

## 10. Frozen research boundaries

```
C3-002 direction policy
Search #2 artifact
Search #3 (not authorized)
C.3-G (not started)
14-August cohort (decision-only, untouched)
P.E.1 (frozen)
P.E.2 control (frozen)
Universe (unchanged)
```

---

## 11. Version roadmap

| Version | What it is |
|---|---|
| **P.E.2** | Fixed execution control (+5%, no risk boundary) |
| **P.E.3 / coralys-exec-v0** | Coralys applies frozen ATR/TMV execution heuristic |
| **Future coralys-exec-v1** | Coralys discovers execution parameters via search |
| **Future coralys-exec-v2** | Adaptive execution profile (per-session reassessment) |

Each version requires its own artifact hash, its own integrity tests, and its own P.E. experiment before it can be used in a prospective cohort.

---

## 12. Revised experiment plan

```
P.E.3-A — Implement + integrity tests
        ↓
FREEZE coralys-exec-v0 artifact
        ↓
P.E.3-B — Full historical characterization
        (every eligible timestamp; do not modify v0 based on results)
        ↓
P.E.3-C — Matched P.E.2 vs P.E.3 replay
        (same T → same C3-002 direction → +5% vs Coralys v0 → same future)
        ↓
CS-P-007 — Statistical evaluation
        ↓
Holdout / prospective confirmation
```

### P.E.3-B eligibility criteria

A historical timestamp is eligible for P.E.3-B if:

- C3-002 can be generated from bars ≤ T
- Sufficient T-history exists for ATR(14) (≥ 14 bars)
- The next eligible session exists
- 20 subsequent market sessions exist
- Required OHLC data exists for the holding period

### P.E.3-B characterization metrics

Run across all eligible timestamps. Do not modify v0 based on these results.

- Target distribution (by TMV state, by instrument)
- Risk distribution
- Target/risk ratio
- TARGET frequency
- RISK frequency
- HORIZON frequency
- AMBIGUOUS frequency (same-bar TARGET/RISK collision)
- Holding duration distribution
- Gap-through frequency
- Invalid execution intents (ATR=0)
- Behaviour by TMV state
- Behaviour by instrument
- Behaviour by volatility regime

### P.E.3-B selection-bias guard

**P.E.3-C MUST use the predeclared P.E.3-B eligible population.**

No timestamp, instrument, TMV state, volatility regime, target range, risk range, or outcome observed during P.E.3-B may be used to include or exclude observations from P.E.3-C.

P.E.3-B is descriptive only. It does not authorize any subset selection for P.E.3-C.

### P.E.3-C comparison baselines

Compare at minimum:

1. P.E.2 fixed +5% target (no risk boundary)
2. P.E.3 Coralys target + Coralys risk boundary
3. No-target / 20-session horizon baseline
4. Possibly fixed symmetric risk baseline (if protocol permits)

### Same-bar TARGET/RISK ambiguity rule

P.E.3 introduces a risk boundary. When both TARGET and RISK are crossed within the same OHLC bar, intraday ordering is unavailable from daily data.

```
If both TARGET and RISK boundaries are crossed within the same OHLC bar
and intraday ordering is unavailable:

    exit_reason = AMBIGUOUS

Do NOT assume TARGET-first or RISK-first.
```

AMBIGUOUS observations:
- Remain in the evidence ledger (append-only)
- Are reported separately in P.E.3-B characterization
- Are excluded from the primary execution comparison in P.E.3-C
- Are not used in CS-P-007 primary analysis unless explicitly authorized

This prevents hidden look-ahead from bar-order assumptions. For eventual live trading, intraday/broker execution data can eliminate much of this ambiguity — but that requires a different execution-data contract, not retroactive changes to the historical experiment.

### Temporal train/test boundary

If sufficient historical data exists, establish a development/evaluation split:

```
DEVELOPMENT PERIOD          EVALUATION PERIOD
────────────────────────┬──────────────────────
                        │
  use to verify:        │  NEVER touch until
  - ATR implementation  │  artifact is frozen
  - TMV mapping         │
  - edge cases          │  P.E.3-B/C run here
  - no-lookahead        │
  - reproducibility     │
                        │
                   freeze point
```

### Critical constraint

**Do not modify v0 multipliers based on historical results.**

Once the artifact is hashed, the parameters are frozen. The historical paths tell us whether the model was useful — they do not feed back into the model.

## 13. Execution Feedback Ledger — learning infrastructure

The `execution_feedback` module provides the learning-ready evidence stream that will feed `coralys-exec-v1` and beyond. It does **not** modify `coralys-exec-v0`.

### Architecture

```
P.E.3-v0 execution
        ↓
Outcome observed
        ↓
ExecutionFeedbackRecord sealed
        ↓
feedback_available_at validated > exit_time
        ↓
Feedback Ledger (append-only)
        ↓
training_set_at(T_train) — DateTime<Utc> comparison
        ↓
Coralys Learner (future)
        ↓
coralys-exec-v1
```

### Critical temporal invariant

`feedback_available_at` is **validated** to be strictly after `exit_time` at seal time. The ledger enforces this — it cannot be bypassed by the caller.

For any future Coralys model trained at T_train:
```
training_set = { feedback | feedback_available_at ≤ T_train }
```

Comparison uses `DateTime<Utc>`, not lexicographic string comparison.

### What the feedback record contains

```
decision_id, instrument, decision_time
c3_002_direction (not bare "direction" — Coralys does not learn direction)
state_hash, tmv_state
entry_time, entry_price, entry_source
coralys_model_id, coralys_model_version, coralys_artifact_hash
atr_14_at_t, target_pct, target_price, risk_pct, risk_boundary
execution_features (complete feature snapshot)
execution_feature_hash (SHA256 of feature snapshot)
exit_time, exit_price, exit_reason, holding_sessions
realized_return, target_reached, risk_reached, horizon_reached, ambiguous
eligible_for_primary_comparison
learning_scope = ExecutionOnly
feedback_available_at (validated > exit_time)
record_hash (SHA256, append-only integrity)
```

### Exit reason taxonomy (P.E.3)

```
Target              — target price reached intraday
Risk                — risk boundary reached intraday
Horizon             — max hold elapsed
Ambiguous           — both target and risk in same OHLC bar (excluded from primary)
TargetGapThrough    — gap open beyond TARGET boundary
RiskGapThrough      — gap open beyond RISK boundary
SessionClose        — forced by market close
```

`TargetGapThrough` and `RiskGapThrough` are distinct — a learner must know which boundary was crossed.

### Learning scope

`learning_scope = ExecutionOnly` — Coralys learns target/risk configuration given the C3-002 direction. It does **not** learn whether LONG or SHORT was correct. That boundary preserves the architectural separation between decision intelligence (C3-002) and execution intelligence (Coralys).

### Feature snapshot

`execution_features` captures the complete information state that produced the intent:
```
tmv_state, atr_14, atr_14_normalized, target_pct, risk_pct, direction, entry_price
```

`execution_feature_hash` is the SHA256 of this snapshot. This allows a future learner to verify it is training on the correct information state.

### AMBIGUOUS handling in feedback

AMBIGUOUS records are retained in the feedback ledger but:
- `eligible_for_primary_comparison = false`
- Excluded from P.E.3-C primary analysis
- May be used by a future Coralys learner if explicitly authorized

### Version lineage

```
coralys-exec-v0   — frozen heuristic (this spec)
coralys-exec-v1   — evidence-informed (trained on feedback ledger)
coralys-exec-v2   — adaptive execution profile (future)
```

Each version has its own artifact hash, training cutoff, and P.E. experiment.

## 14. Implementation status

| Component | Status |
|---|---|
| `coralys_execution_model.rs` | ✅ Implemented, 13/13 tests pass |
| `execution_feedback.rs` | ✅ Implemented, tests pass |
| `mod.rs` registration | ✅ Done |
| `coralys_artifact_hash` | ⏳ Pending — compute from frozen spec sections 1–11 |
| `CORALYS_TARGET_ARTIFACT_PRESENT` | ⏳ Pending — set after hash is computed |
| P.E.3-B full historical characterization | ⏳ Pending |
| P.E.3-C matched control/treatment replay | ⏳ Pending |
| CS-P-007 statistical evaluation | 🔴 Deferred — do not start until P.E.3-C complete |

## 15. Next steps to complete freeze

1. ✅ Spec finalized and frozen (sections 1–11)
2. ✅ `CoralysExecutionModelV0` implemented in Rust
3. ✅ `ExecutionFeedbackRecord` implemented in Rust
4. ✅ All integrity tests pass (13/13)
5. ⏳ Compute `coralys_artifact_hash` from frozen spec sections 1–11
6. ⏳ Set `CORALYS_TARGET_ARTIFACT_PRESENT = true` in `decision_intent.rs`
7. ⏳ Run P.E.3-B full historical characterization
8. ⏳ Run P.E.3-C matched control/treatment replay
9. 🔴 Do not compare P.E.2 vs P.E.3 until CS-P-007 protocol is run
10. 🔴 Do not modify v0 multipliers based on historical results