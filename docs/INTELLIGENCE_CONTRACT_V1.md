# Intelligence Contract v1
## ChronoSentiment Intraday Opportunity Engine

**Status:** FROZEN — do not modify without a new discovery iteration  
**Frozen:** 2026-09-12  
**Source:** Nine-iteration discovery loop (v0.1–v0.9) on 459-decision historical dataset  
**Reference dataset:** `datasets/p4_opportunity_dataset.json`  
**Reference engine:** `scripts/p4_historical_intraday_replay.py`  
**Reference state engine:** `scripts/decision_state_engine.py`

---

## 1. Purpose

This document is the authoritative specification of the product intelligence model. It defines:

- All input features and their information-availability constraints
- All classification states and their rules
- All checkpoint transitions
- All action codes and their semantics
- Historical performance reference (frozen, immutable)

Any Rust implementation, backtest, or UI must conform to this contract exactly. Deviations require a new discovery iteration and a new contract version.

---

## 2. Information Timeline

The engine operates at four checkpoints. Each checkpoint may only use information available at that point in time.

```
T0 (Decision time)
│  Available: direction, OQS, h60_classification
│  NOT available: momentum_persistence (requires full H300 path)
│
├── ENTRY checkpoint
│
T+60min (H60)
│  Available: h15_ret, h30_ret, h60_ret, mfe_h60, mae_h60
│
├── H60 checkpoint
│
T+120min (H120)
│  Available: h120_ret, mfe_h120
│
├── H120 checkpoint  ← LONG WAIT-MID reassessment occurs here
│
T+180min (H180)
│  Available: h180_ret
│
├── H180 checkpoint
│
T+300min (H300)
│  Outcome measured: h300_ret
│  momentum_persistence computed (retrospective only)
```

**Critical constraint:** `momentum_persistence` is a retrospective path variable computed over the full H300 window. It is NOT available at entry time. Entry-time classification uses OQS only. Momentum persistence is used only in historical analysis and monitoring-mode products where the path has already been observed.

---

## 3. Entry-Time Classification Rules

### 3.1 Inputs (decision-time only)

| Input | Type | Source |
|---|---|---|
| `direction` | `"LONG"` \| `"SHORT"` | Watch decision |
| `h60_classification` | `"ENTER"` \| `"WAIT"` \| `"AVOID"` | Frozen Phase 4 state machine |
| `opportunity_quality_score` (OQS) | `int` 0–100 | Composite path score |

### 3.2 Classification rules

**SHORT:**

| Condition | State |
|---|---|
| h60_class == ENTER | `ENTER` |
| h60_class == AVOID | `AVOID` |
| h60_class == WAIT AND OQS ≥ 50 | `WAIT-HIGH` |
| h60_class == WAIT AND OQS < 50 | `WAIT-LOW` |

**LONG:**

| Condition | State |
|---|---|
| h60_class == ENTER | `ENTER` |
| h60_class == AVOID | `AVOID` |
| h60_class == WAIT AND OQS ≥ 65 | `WAIT-HIGH` |
| h60_class == WAIT AND 40 ≤ OQS < 65 | `WAIT-MID` |
| h60_class == WAIT AND OQS < 40 | `WAIT-LOW` |

### 3.3 Threshold constants

```
OQS_LONG_HIGH   = 65
OQS_LONG_MID    = 40
OQS_SHORT_HIGH  = 50
FAV_ADV_THRESHOLD = 0.002   # 0.2%
MFE_FLOOR       = 0.001     # 0.1% — below this → AVOID-LATE override
```

---

## 4. H120 Reassessment Rules

**Applies to:** LONG WAIT-MID only. All other states pass through unchanged.

### 4.1 Inputs

| Input | Type | Available at |
|---|---|---|
| `h120_ret` | `float` | H120 checkpoint |
| `mfe_h120` | `float` | H120 checkpoint |

### 4.2 Reassessment logic

```
if mfe_h120 < MFE_FLOOR (0.001):
    → AVOID-LATE

elif h120_ret > FAV_ADV_THRESHOLD (0.002):
    → ENTER-LATE

elif h120_ret < -FAV_ADV_THRESHOLD (-0.002):
    → AVOID-LATE

else:
    → WAIT-LATE
```

**Note:** AVOID-LATE has N=1 in historical data. It is implemented but should not be treated as a hard avoidance rule until more evidence accumulates.

---

## 5. State Vocabulary

| State | Meaning | Product action |
|---|---|---|
| `ENTER` | Strong intraday signal, H15+H60 both FAV | Act at entry |
| `WAIT-HIGH` | WAIT with high OQS (LONG) or high momentum (SHORT) | Act at entry |
| `WAIT-MID` | WAIT with moderate OQS and positive momentum (LONG only) | Monitor → reassess at H120 |
| `WAIT-LOW` | WAIT with low OQS or low momentum | Do not act |
| `AVOID` | H15+H60 both ADV | Do not act |
| `ENTER-LATE` | WAIT-MID upgraded at H120 (FAV) | Act at H120 |
| `WAIT-LATE` | WAIT-MID at H120 (FLAT) | Continue monitoring |
| `AVOID-LATE` | WAIT-MID at H120 (ADV or MFE<0.1%) | Do not act (N=1, not yet hard rule) |
| `UNKNOWN` | Unclassifiable | Do not act |

---

## 6. Action Vocabulary

| Action | Meaning |
|---|---|
| `ACT` | Enter or maintain position |
| `MONITOR` | Observe path; do not act yet |
| `AVOID` | Do not enter; exit if already in |
| `UPGRADE` | Promote from MONITOR to ACT (H120 FAV for WAIT-MID) |
| `CONTINUE` | Maintain current monitoring posture |

---

## 7. Historical Performance Reference (Frozen)

These numbers are frozen from the v0.9 discovery loop. They are reference values for the product UI and confidence calibration. They must not be updated without a new discovery iteration.

| Direction | State | N | H300 win% | H300 PF | H300 median |
|---|---|---|---|---|---|
| SHORT | ENTER | 53 | 71.7% | 13.35x | +0.76% |
| SHORT | WAIT-HIGH | 71 | 77.5% | >99x | +0.56% |
| SHORT | WAIT-LOW | 45 | 4.4% | 0.04x | −0.30% |
| SHORT | AVOID | 46 | 26.1% | 0.23x | −0.32% |
| LONG | ENTER | 16 | 75.0% | 19.46x | +0.61% |
| LONG | WAIT-HIGH | 17 | 82.4% | 72.36x | +0.66% |
| LONG | WAIT-MID | 42 | 59.5% | 10.06x | +0.26% |
| LONG | WAIT-LOW | 57 | 28.3% | 0.67x | −0.23% |
| LONG | AVOID | 112 | 11.6% | 0.08x | −0.65% |
| LONG | ENTER-LATE | 23 | 69.6% | 11.67x | +0.46% |
| LONG | WAIT-LATE | 18 | 50.0% | 6.47x | +0.18% |
| LONG | AVOID-LATE | 1 | 0.0% | — | +0.02% |

**Note on WAIT-HIGH historical N:** The LONG WAIT-HIGH N=17 and SHORT WAIT-HIGH N=71 reflect the v0.9 momentum-based split applied retrospectively. The entry-time backtest uses OQS-based rules (v0.3), which produce LONG WAIT-HIGH N=23 and SHORT WAIT-HIGH N=81. The performance reference above is the retrospective (path-observed) quality; the backtest uses the entry-time (information-safe) classification.

---

## 8. Two Product Modes

### Mode A — Entry-time action

Applies to: ENTER, WAIT-HIGH (both directions)

The product makes a decision at entry time. No further monitoring required. The opportunity is identifiable from decision-time information alone.

### Mode B — Session monitoring

Applies to: LONG WAIT-MID

The product cannot make a reliable entry-time decision. It monitors the intraday path and reassesses at H120. The H120 state determines the final action.

```
LONG WAIT-MID
    │
    ├── H120 FAV  → ENTER-LATE  → ACT    (PF 11.67x)
    ├── H120 FLAT → WAIT-LATE   → CONTINUE (PF 6.47x, still positive)
    └── H120 ADV  → AVOID-LATE  → AVOID  (N=1, not yet hard rule)
```

---

## 9. What Is NOT in This Contract

The following were discovered in the research loop but are NOT part of the v1 production contract:

- **Momentum persistence as an entry-time rule:** mp is a retrospective variable. It cannot be used at entry time. It is a monitoring-mode feature only.
- **AVOID-LATE as a hard avoidance rule:** N=1 historically. Implement the state but do not treat it as a reliable signal.
- **SHORT WAIT-HIGH momentum split:** The v0.6 improvement (mp≥0.5 → PF >99x) is a retrospective finding. The entry-time backtest uses OQS≥50 (v0.3 rule). The momentum split is a monitoring-mode enhancement.
- **H180 confirmation layer:** Implemented in the Decision State Engine but not yet validated as a production rule. Surface as informational only.

---

## 10. Script Status Map

| Script | Status | Role |
|---|---|---|
| `scripts/p4_historical_intraday_replay.py` | 🟢 REFERENCE | Historical opportunity engine — do not modify |
| `scripts/decision_state_engine.py` | 🟢 PRODUCT LOGIC | Python reference implementation of this contract |
| `scripts/p4_loss_analysis_1m_vs_5m.py` | 🟢 REFERENCE | Failure/path analysis — do not modify |
| `datasets/p4_opportunity_dataset.json` | 🟢 GOLDEN FIXTURE | 459-decision reference dataset — do not modify |
| `scripts/p4_track_b_intraday.py` | 🟡 DISCOVERY | Generated Track B results — frozen |
| `scripts/p4_sprint1_rank.py` | 🟡 DISCOVERY | Sprint 1 ranking analysis — frozen |
| `scripts/p4_track_a_full_replay.py` | 🟡 DISCOVERY | Track A walk-forward — frozen |
| `scripts/p4_mfe_h60_distribution.py` | 🟡 DISCOVERY | MFE distribution analysis — frozen |
| `scripts/p4_recalculate_predictions.py` | 🟡 DISCOVERY | Prediction recalculation — frozen |
| All `parse_phase2e_*.py` scripts | 🟡 DISCOVERY | Phase 2e analysis — frozen |
| All `phase*.py` scripts | 🟡 DISCOVERY | Earlier phase analysis — frozen |

**Legend:**
- 🟢 REFERENCE / PRODUCT LOGIC — authoritative, do not modify
- 🟡 DISCOVERY / FROZEN — generated the current rules, do not modify, do not port
- 🔵 NEW — to be created (backtest, Rust implementation)
- 🔴 OBSOLETE — superseded, safe to archive

---

## 11. Implementation Checklist

For any new implementation (Rust, backtest, UI) to be considered conformant:

- [ ] Entry-time classification matches Python reference for all 459 records
- [ ] H120 reassessment matches Python reference for all LONG WAIT-MID records
- [ ] Action codes match Python reference at ENTRY, H60, H120, H180 checkpoints
- [ ] Historical performance reference values are displayed correctly
- [ ] AVOID-LATE is implemented but flagged as low-evidence (N=1)
- [ ] momentum_persistence is NOT used as an entry-time classifier
- [ ] Differential test passes: 0 classification mismatches vs Python reference

---

## 12. Next Steps

1. **Backtest v1** — replay v0.3 entry-time rules (information-safe) over 459 decisions; generate equity curve, attribution, and baseline comparison
2. **Rust implementation** — implement this contract in Rust; differential test against Python reference
3. **Decision Cockpit v0.4** — connect Rust engine to UI; surface ACTION/STATE/CONFIDENCE/HORIZON/WHY/RISK at each checkpoint