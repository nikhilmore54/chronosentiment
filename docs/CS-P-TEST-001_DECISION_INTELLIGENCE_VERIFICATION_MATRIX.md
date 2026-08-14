# CS-P-TEST-001 — Decision Intelligence Verification Matrix v0.1

**Document type:** Product verification specification  
**Status:** Active  
**Date:** 2026-08-14  
**Parent:** CS-P-001, CS-P-002, CS-P-004-E1  
**Does not supersede:** EV-GOV-003, B3, B4, CS-P-002-R1, CS-P-003, CS-P-004  
**Does not open:** G-GATE v1.2, B5, Decision Engine v1.0 freeze, candidate policy, parameter search

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

**Objective:** prove the implementation satisfies the ChronoSentiment **product vision**, not merely that the current code is internally consistent.

```text
                    ChronoSentiment Vision
                             │
          ┌──────────────────┼──────────────────┐
          ▼                  ▼                  ▼
   Temporal Integrity   Decision Integrity   Information Integrity
          │                  │                  │
          ▼                  ▼                  ▼
     No lookahead       No hidden inputs     No invented factors
                             │
                    Reproducible Decision
                             │
                     Honest Measurement
```

One-way layer flow (a lower layer cannot write into an upper layer):

```text
Assessment → TradingDecision → Ledger → Outcome → Performance
```

---

## 1. When this runs

```text
Assessment Enrichment v0.1
          ↓
CS-P-TEST-001  ← this document
          ↓
ALL TESTS PASS
          ↓
(no new snapshot in this step)
          ↓
CS-P-005 Factor Ecology (existing certified dump)
```

Do **not** generate another historical snapshot until this matrix is green. Do not tune Trend. Do not implement `FutureCandidatePolicy`. Do not freeze v1.0.

Proof: `adapters/chronosentiment/tests/csp_test_001_verification_matrix.rs`

---

## 2. Matrix

| ID | Requirement | Layer | Adversarial input | Expected invariant | Status |
|----|-------------|-------|-------------------|--------------------|--------|
| TEMP-001 | No future observations | Assessment / Adapter | future bar | decision bit-identical | PASS |
| TEMP-002 | No future-derived feature | Assessment | future bar in metric path | factor_status + decision unchanged | PASS |
| TEMP-003 | No future assessment | Adapter | future Bearish assessment | action unchanged | PASS |
| TEMP-004 | No future lake decision | Adapter | future lake decision id | not consumed | PASS |
| TEMP-005 | No future outcome | Decision | +50% outcome row | decision identity unchanged | PASS |
| FACT-001 | Momentum availability | Assessment | missing `roc_20` | UNAVAILABLE, not zero | PASS |
| FACT-002 | Volatility availability | Assessment | missing `atr_14` | UNAVAILABLE, not zero | PASS |
| FACT-003 | Zero is a value | Assessment | `roc_20 = 0.0` | AVAILABLE, not UNAVAILABLE | PASS |
| FACT-004 | No fabricated volatility direction | Assessment | `atr_14` present | no High/Low DomainAssessment | PASS |
| FACT-005 | Trend never silently substituted | Assessment | missing MAs | UNAVAILABLE, no invented Bullish | PASS |
| DEC-001 | Deterministic identity | Decision | repeated run | identical id/hash/lineage/action | PASS |
| DEC-002 | Outcome independence | Decision | outcome in environment | identity unchanged | PASS |
| DEC-003 | NO_TRADE validity | Policy | Neutral / absent Trend | NO_TRADE | PASS |
| DEC-004 | Momentum does not alter Trend map | Policy | Momentum present/absent | same Trend action | PASS |
| DEC-005 | Volatility unused by current policy | Policy | change `atr_14` | action + decision_id unchanged | PASS |
| DEC-006 | Adapter SQL cannot select outcomes | Adapter | source inspection | no `FROM/JOIN knowledge_outcomes` | PASS |
| LIN-001 | Complete lineage | Contract | missing parent | fail | PASS |
| LIN-002 | Explicit as-of | Decision | wall clock vs T | `as_of` is T | PASS |
| LED-001 | Append-only ledger | Ledger | later tick | prior row unchanged | PASS |
| OUT-001 | Independent outcome | Outcome | mutate copy of decision | ledger/decision unchanged | PASS |
| PERF-001 | No feedback | Performance | measure performance | ledger/decision unchanged | PASS |
| ADV-001 | Persistence-time attack | Identity | change `created_at` | identity unchanged | PASS |
| ADV-002 | Instrument contamination | Adapter | extra observation | action + decision_id unchanged | PASS |
| ADV-003 | Ordering attack | Adapter | shuffled assessments | identical decision | PASS |
| ADV-004 | Duplicate-input attack | Metrics / Adapter | duplicate observation | identical decision / metrics | PASS |
| ADV-005 | T / instrument / engine / Trend change identity | Identity | one-at-a-time | identity changes | PASS |

---

## 3. Identity contract (unfrozen-dev)

`decision_id` / `content_hash` hash **policy consumption**, not the diagnostic blob.

Included: engine version, instrument, as-of T, action, confidence, mapping rule, **consumed** factors (Trend only under `TrendMappingPolicy`), assessment_id, rationale (`action_reason`).

Excluded: unused factor values (Momentum/Volatility under current policy), `knowledge_outcomes`, wall-clock `created_at` / `recorded_at`, `input_set_hash` (audit only), full assessment `to_hash()`.

`lineage.input_set_hash` remains the audit hash of artifacts available at T.

---

## 4. Forbidden

- Using these tests to justify threshold search
- Reopening G-GATE
- Mutating B3/B4
- Treating CS-P-003 as the verification laboratory
