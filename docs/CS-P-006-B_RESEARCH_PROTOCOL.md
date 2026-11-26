# CS-P-006-B — Coralys Discovery Research Protocol

**Document type:** Frozen research protocol  
**Status:** Protocol rules frozen; CS-P-006-S1 certified; **CS-P-006-B.1 chronological partition frozen**  
**Date:** 2026-08-14  
**Parent:** CS-P-006, CS-P-006-A, CS-P-005  
**Does not:** invent split dates, mutate B3/B4, open B5, reopen G-GATE, freeze Decision Engine v1.0, replace the CS-P-003 clock, encode financing or margin-facility strategy rules  

`.cursor/rules/chronosentiment-core.mdc`: no invented methodology; evaluate across instruments and regimes; ChronoSentiment decide path stays deterministic.

---

## Authorization gate

```text
Intended research universe (7)
        │
        ▼
CS-P-006-S1 disposable snapshot (not B4, not B5)  ✓ PASS / READY
        │
        ▼
CS-P-006-B.1  chronological partition             ✓ PASS
        │
        ▼
CS-P-006-C    Coralys search                      complete (first run sealed)
```

This document freezes **rules**. Calendar membership of each timestamp is frozen in **CS-P-006-B.1**. CS-P-004 year folds are **not** the CS-P-006-B split.

Manifest: `docs/CS-P-006-B.1_CHRONOLOGICAL_PARTITION.md` and `product_validation/CS-P-006/partition/`.

---

## 1. Intended research universe

| Script | Role |
|--------|------|
| HDFCBANK.NS | Large private bank |
| ICICIBANK.NS | Large private bank, different profile |
| INFY.NS | IT / services |
| RELIANCE.NS | Diversified large-cap |
| TCS.NS | IT / services peer |
| IDEA.NS | Telecom / different risk profile |
| MAHABANK.NS | PSU bank / different banking profile |

Seven instruments is the starting discovery universe. Do not expand N to dilute the first experiment. Regime diversity matters more than adding correlated large-caps.

CS-P-003’s live ticker list is a **confirmation clock**, not this universe. Do not silently treat `DEFAULT_TICKERS` as the certified discovery dataset.

---

## 2. Coverage audit (current certified snapshots)

| Snapshot | Instruments | Status vs 7-name universe |
|----------|-------------|---------------------------|
| B4 dump `f74e576e…f8cd6` | HDFCBANK, ICICIBANK, INFY, RELIANCE, TCS | **5 / 7** |
| CS-P-005 ecology (195 rows) | same five | **5 / 7** |
| CS-P-003 `DEFAULT_TICKERS` | five + IDEA.NS | forward clock; **not** a historical discovery dump |
| CS-P-006-S1 `20260814T183851Z_7instrument` identity `c21ec256…28c6` | all seven | **7 / 7**, TMV 273/273, certified PASS |

**B4 / CS-P-005 audit result: INSUFFICIENT.** Those dumps still lack IDEA.NS and MAHABANK.NS. They must not stand in for the research universe.

**CS-P-006-S1 audit result: PASS / READY.** All seven names are present on the month-end 15:30 UTC grid 2021-10 through 2024-12, with Trend / Momentum / Volatility available at every `(instrument, T)`.

Do **not** substitute the five-instrument B4/CS-P-005 record for the seven-instrument universe. Do **not** freeze split dates on 2021–2024 five-name coverage, and do **not** copy G-GATE 55/27/28.

CS-P-006-B.1 has frozen the partition from the certified S1 39-point seven-name coverage (`docs/CS-P-006-B.1_CHRONOLOGICAL_PARTITION.md`).

---

## 3. What information is available at T

Certified on the **five-instrument** CS-P-005 snapshot (descriptive of that snapshot, not a 7-name claim):

| Field | CS-P-005 evidence | Discovery use |
|-------|-------------------|---------------|
| Trend | 195/195 available | state at T |
| Momentum | 195/195 available | state at T |
| Volatility | 195/195 available (ATR magnitude; no High/Low) | presence at T only; **no global ATR threshold** |
| Trend × Momentum | 4 observed states | search space, not a hand-written map |
| Momentum disagrees with Trend | 60/195 = 30.8% | reason selectivity might exist; **not** a NO_TRADE rule |
| Baseline actions | 110 LONG / 85 SHORT / 0 NO_TRADE | fixture behaviour, not the objective |
| Instrument id | present on assessments at T | conditioning **permitted** once schema + 7-name snapshot allow it |
| 60D lake SHORT | 85/195 unavailable | **must not** be the discovery objective |

As-of timestamp is the temporal key. It is **not** a trading feature and **must not** appear as a year/regime genome predicate (that would encode “it worked in 2023”).

---

## 4. What Coralys is allowed to discover

Permitted (after coverage PASS and CS-P-006-B.1 date freeze):

* `PolicyArtifact` structure already defined in CS-P-006-A
* Mapping from certified factor states to `{LONG, SHORT, NO_TRADE}`
* Whether NO_TRADE is useful (including on Trend/Momentum disagreement) — Coralys chooses; we do not
* Interactions among Trend, Momentum, and Volatility **presence**
* Optional instrument conditioning (ticker equality), only after the artifact schema can express it and the 7-name snapshot exists

Forbidden as “discovery”:

* Hand-written confluence candidate (`DecisionPolicyCandidate_v0.1`)
* Threshold grid on ATR / ROC
* Calendar-year or “this year was good” predicates
* Promoting `BaselineTrendMappingPolicy` as the learned policy

`csp006a.policy_artifact.1` currently matches factor predicates only. Instrument-conditioned genomes require a documented schema amendment **before** 006-C if that permission is used. That amendment is not 006-C.

---

## 5. Objective (defined before search; not operational until dates freeze)

Coralys may use subsequent outcomes **only inside TRAIN** (and, for selection, VALIDATION). ChronoSentiment `decide_at(T)` never receives them.

| Rule | Requirement |
|------|-------------|
| Measurable path | Subsequent **observation-path** returns (raw prices after T), LONG and SHORT with opposite sign |
| NO_TRADE | Standing aside; **not** a zero-return trade |
| Incomplete lake SHORT | B4/CS-P-005 60D lake `available=110 / unavailable=85` **cannot** be the search objective |
| TRAIN | Evolution / fitness only |
| VALIDATION | Candidate **selection** only; no further evolution |
| TEST | Untouched. Result is evidence. No feedback into Coralys |
| Aggregation | Fitness aggregated across the seven intended instruments and across TRAIN regimes — not a single-name champion |

Exact numeric fitness formula is implemented in **CS-P-006-C** from these rules. This document forbids using TEST / evaluation or lake-SHORT-as-zero to define that formula.

---

## 6. TRAIN / VALIDATION / TEST (roles frozen; dates frozen in CS-P-006-B.1)

```text
TRAIN
  Coralys searches
        │
        ▼
Candidate population
        │
        ▼
VALIDATION
  selection only → ONE sealed candidate
        │
        ▼
TEST
  ChronoSentiment evaluates
  NO Coralys feedback
        │
        ▼
Evidence → only then consider CS-P-003
```

Ordering required: TRAIN exclusive_end ≤ VALIDATION inclusive_start, VALIDATION exclusive_end ≤ TEST inclusive_start. TEST / evaluation must be later history the policy has never been selected on.

**Dates: frozen in CS-P-006-B.1** from the 39 common timestamps (equal contiguous thirds). Not G-GATE 55/27/28. Not CS-P-004 year folds.

---

## 7. What constitutes a candidate

Exactly one sealed `PolicyArtifact` such that:

* `discovery_engine` starts with `coralys.`
* `action_space` is exactly LONG, SHORT, NO_TRADE
* `unmatched_action` is explicit
* training windows are the CS-P-006-B.1 frozen triple
* selected on VALIDATION only
* ChronoSentiment can `decide_from_inputs` without outcomes

Until a sealed Coralys artifact exists, **no candidate may be declared**. CS-P-006-C has now sealed one artifact (`9a887827…971ac0`). It is a discovery result, not a promoted ChronoSentiment strategy.

---

## 8. What constitutes a failed candidate

A sealed artifact still **fails** (must not proceed to TEST as the promoted candidate) if any of:

* TEST or forward observations entered evolution or selection
* Genome encodes calendar year / as_of as a trading feature
* Uses a global ATR cutoff
* Cannot represent NO_TRADE
* Works on only one instrument of the intended seven (overfit to a name)
* Was chosen by inspecting TEST performance
* Hash / methodology does not match the frozen protocol

---

## 9. Complexity / overfitting constraint

* First-match rule list; no unbounded numeric grids
* No year predicates
* No global ATR threshold (CS-P-005: ATR is instrument-scale dependent)
* Maximum **16** rules in the first discovery run (enough for the four Trend×Momentum states and limited ticker exceptions; not a per-bar lookup table)
* Instrument rules, if used, are ticker equality, not year×ticker cells

---

## 10. Information forbidden during discovery

* Outcomes after the TRAIN (evolution) or VALIDATION (selection) boundary
* Any TEST outcome
* CS-P-003 forward observations
* Performance summaries from TEST
* Information with effective time **after T** when constructing the state at T
* B4 mutation, B5, v1.1 edits, G-GATE rerun
* Real capital / brokerage

---

## Stop

| Item | Status |
|------|--------|
| Protocol rules (this document) | **Frozen** |
| B4 / CS-P-005 vs 7-name universe | **INSUFFICIENT** (still 5/7; not the discovery dump) |
| CS-P-006-S1 7-instrument snapshot | **PASS / READY** |
| Split dates | **Frozen** — CS-P-006-B.1 PASS |
| CS-P-006-C Coralys search | **Complete** — first sealed artifact; not promoted; evaluation not fed back |
| CS-P-003 | Continues independently |

Engine version remains **`unfrozen-dev`**. No real capital.
