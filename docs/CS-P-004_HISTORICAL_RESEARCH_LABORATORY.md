# CS-P-004 — Historical Research & Robustness Laboratory v0.1

**Document type:** Product architecture brief  
**Status:** Active — primary historical discovery programme  
**Date:** 2026-08-14  
**Parent:** CS-P-001, CS-P-002, CS-P-002-R1  
**Does not supersede:** EV-GOV-003, G-Extension Methodology v1.1, B3, B4, CS-P-002-R1, CS-P-003  
**Does not open:** G-GATE v1.2, B5, v1.1 rerun, Decision Engine v1.0 freeze, parameter search

This is **not** a predictive-value methodology and must not be used to reopen G-GATE v1.1.

**Objective:**

> Understand when, where, and why ChronoSentiment’s reconstructed `unfrozen-dev` decisions work or fail on the certified B4 historical record — without changing the Decision Engine, without treating the 60-day forward clock as the research laboratory, and without manufacturing a candidate policy by tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no randomness in strategy logic; evaluate across scenarios; no invented methodology.

---

## 1. Sequence (why this exists)

CS-P-002-R1 is a **baseline**, not a research programme. One aggregate (e.g. 60D mean ≈ −0.0099 on the 110 LONG lake rows) does not explain where that number comes from. Waiting two months of CS-P-003 to discover basic behaviour would waste the 2021–2024 record we already possess.

```text
                    HISTORICAL RESEARCH (CS-P-004)
                           │
                           ▼
                  B4 certified historical data
                           │
                           ▼
                  Decision reconstruction   (Replay Adapter; unchanged)
                           │
                           ▼
                  Outcome reconstruction   (Outcome Engine v0.1; unchanged)
                           │
                           ▼
                  Research Laboratory
                    regime / action / robustness /
                    sensitivity / walk-forward / stability
                           │
                           ▼
                  Candidate policy (human-authored, later)
                           │
                           ▼
                  PAPER / FORWARD (CS-P-003) — final confirmation
                           │
                           ▼
                     REAL CAPITAL (not now)
```

CS-P-003’s operational clock **may keep running** so genuine future observations accumulate. It is **not** the discovery mechanism. Do not interpret Observation #1, or any immature forward sample, as the research result. Do not treat CS-P-002-R1 as the forward test.

---

## 2. What the laboratory consumes

Reuse the product pipeline. Do **not** change the `decide_at(T)` temporal contract (inputs ≤ T) and do **not** retune the Trend action map from historical returns.

```text
B4 dump (read-only restore)
   → Replay Adapter (information-fidelity v0.1; same Trend→action map)
   → DecisionLedger
   → Outcome Engine v0.1 → OutcomeReport
   → Performance Engine v0.1 (on slices)
   → Laboratory (this programme)
```

Context labels (trend / momentum / volatility **if present on the assessment at T**) are read from the same `knowledge_assessments` already consumed by replay. They are **explanatory strata**, not new decision inputs and not a second engine.

Certified B4 dump SHA-256 remains `f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6`. Never write `chrono_b3_test` / `chrono_b4_test`.

The existing G-GATE `adapters/chronosentiment/src/research/` module (Uuid::new_v4 / wall-clock runs) is **not** this laboratory.

---

## 3. Walk-forward (no training in v0.1)

`unfrozen-dev` has no fitted parameters on the product path. Walk-forward is therefore **temporal slicing of already-reconstructed `decide_at(T)` records**, not a train/fit loop:

```text
Train window: describe behaviour (counts, coverage, action mix)
Test window:  measure the same frozen policy (Performance Engine v0.1)
Constraint:   every test as_of is strictly after train_end
```

Default folds (UTC calendar years on the B4 span 2021-10-31 → 2024-12-31):

| Fold | Train (as_of <) | Test |
|------|-----------------|------|
| 2022 | 2022-01-01 | 2022 calendar year |
| 2023 | 2023-01-01 | 2023 calendar year |
| 2024 | 2024-01-01 | 2024 calendar year |

The test period remains genuinely unseen relative to the train description. Do **not** search thresholds on train and apply them on test. That would be a new engine version, which this phase forbids.

---

## 4. Deliverables

| # | Report | Question |
|---|---------|----------|
| 1 | Decision Behaviour | What did the engine actually decide? LONG/SHORT/NO_TRADE, instrument, time, confidence, transitions, streaks |
| 2 | Regime / Context | How do those decisions sit on Trend / Momentum / Volatility labels present at T? |
| 3 | Outcome Stratification | Where do attached returns come from (action × regime × year × instrument × horizon)? |
| 4 | Walk-Forward | Does the same policy’s measured behaviour persist on later unseen years? |
| 5 | Robustness | Does the sign/coverage hold across periods, instruments, horizons — or disappear outside a slice? |
| 6 | Decision-vs-Baseline | When it says LONG/SHORT, is that better than standing aside? When it says NO_TRADE, was standing aside useful? |
| 7 | Historical Research Summary | What is understood; what is not; what coverage is missing; **no** auto-selected candidate policy |

Artifact directory (CS-P-004-R1, pre-enhancement): `product_validation/CS-P-004_unfrozen_dev/`  
Adapter Enhancement v0.1 re-run: `product_validation/CS-P-004_adapter_v0.1/` (does not overwrite R1).

---

## 5. Coverage honesty (do not “repair” B4)

CS-P-002-R1: 195 decisions; 110 LONG with lake outcomes; **85 SHORT with `available: false`**; **0 NO_TRADE**.

- SHORT is **unevaluated**, not zero.
- NO_TRADE cannot be judged until it occurs.
- Do not invent lake rows. Do not retune Trend mapping to manufacture NO_TRADE.
- Volatility labels are emitted only if the assessment at T actually contains `Concept::Volatility`. Absence is reported, not imputed from prices in v0.1.

Forward/Paper (CS-P-003) remains the path that measures LONG and SHORT from raw prices after T. That does not replace this laboratory.

---

## 6. Forbidden

- Changing LONG/SHORT/NO_TRADE rules, Trend thresholds, or horizons
- Optimizer / “best horizon” / magic parameters
- Feeding laboratory output back into `TradingDecision`
- Reopening G-GATE, mutating B3/B4, freezing Decision Engine v1.0
- Treating CS-P-003 as the place we discover basic problems
- Manufacturing forward observations from historical replay

A later **candidate policy** is a documented successor engine version, authored only after these reports are satisfactory — not a silent rerun of `unfrozen-dev`.

---

## 7. Implementation

| Piece | Location |
|-------|----------|
| Brief | this document |
| Laboratory | `adapters/chronosentiment/src/decision_support/laboratory.rs` |
| Context labels (read-only) | `decision_support/lab_context.rs` |
| Binary | `csp004_historical_lab` |
| Runner | `./run_csp004_historical_lab.sh` |
| Proof | `adapters/chronosentiment/tests/historical_laboratory_tests.rs` |

Official proof is unit tests (walk-forward exclusion, determinism, NO_TRADE not zero, SHORT missing stays missing, temporal `decide_at(T)`). The B4 restore runner writes the seven reports. Adapter Enhancement v0.1 is representation-only (see §8).

Engine version remains **`unfrozen-dev`**. No real capital.

---

## 8. Adapter Enhancement v0.1 (one bounded change)

Not a new strategy. Not parameter tuning. Not G-GATE. Not v1.0.

The CS-P-004-R1 lab showed the first adapter was a thin Trend translator: confidence 0.82 everywhere, NO_TRADE = 0, Momentum/Volatility dropped at the boundary.

Authorized change — **information fidelity only**:

1. Decision confidence is `UNAVAILABLE` until a confidence model exists. Do not copy assessment `0.82`.
2. Preserve Trend / Momentum / Volatility factors present **or absent** on the assessment at T.
3. Keep the Trend→LONG/SHORT/NO_TRADE map **unchanged** and write it on `evidence.mapping_rule`.
4. `evidence.diagnostics` states why the action was emitted.
5. Identity remains deterministic (`csp004.decision.0`). Same inputs → same `decision_id`.
6. `decide_at(T)` still uses only inputs ≤ T.
7. B4 is not mutated. No return-based threshold search.
8. Re-run the laboratory **once** into `product_validation/CS-P-004_adapter_v0.1/`. Do not overwrite CS-P-002-R1 or CS-P-004-R1.

Producer: `csp004.adapter.v0.1`. This is still **Decision Adapter v0.x**, not Trading Strategy v1.0.

SHORT lake coverage is a separate measurement problem (observation-path from prices, not B4 mutation) and is not this enhancement.

---

## 9. Assessment Enrichment v0.1

Not B5. Not a candidate policy. Not G-GATE. Not a performance experiment.

The adapter faithfully showed that B4 assessments do not contain independent Momentum/Volatility. This change is **upstream of the adapter**:

- `assess_at(T)` records Trend, Momentum, and Volatility as `AVAILABLE` or `UNAVAILABLE` from metrics that exist at T (`ma_20`/`ma_50`, `roc_20`, `atr_14`).
- Missing factors are **UNAVAILABLE**, not filled.
- Volatility ATR is magnitude-only: AVAILABLE when `atr_14` exists; no invented High/Low threshold.
- `evaluation_timestamp = T`; `created_at` remains persist wall-clock.
- `TrendMappingPolicy` is the default `DecisionPolicy` and keeps the current Trend→action map.
- Factor-availability diagnostics: `decision_support/factor_availability.rs`.

### 9.1 Information-fidelity snapshot (authorized)

This snapshot answers whether ChronoSentiment now has a sufficiently rich, temporally valid information state — **not** whether a strategy made money.

```text
CODE CHANGE
    ↓
generate NEW historical snapshot
    ↓
temporal / lineage certification
    ↓
factor-availability report
    ↓
STOP
```

- Runner: `./run_assessment_enrichment_snapshot.sh`
- Database: disposable `chrono_enrichment_v01` (never `chrono_b3_test` / `chrono_b4_test`)
- Artifacts: `product_validation/assessment_enrichment_v0.1/`
- Do **not** designate this snapshot B5. That is decided from evidence after review.
- Do **not** implement `FutureCandidatePolicy`.
- Do **not** run CS-P-004 performance reports, G-GATE, or freeze Decision Engine v1.0.
- Next engineering gate: `docs/CS-P-TEST-001_DECISION_INTELLIGENCE_VERIFICATION_MATRIX.md`.
- Next research step on this dump: `docs/CS-P-005_FACTOR_ECOLOGY_ANALYSIS.md` (ecology, not a candidate).

B3 and B4 dumps remain immutable. Engine version remains **`unfrozen-dev`**. No real capital.


