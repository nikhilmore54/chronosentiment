# CS-P-006-C.1 — Search #1 post-search diagnosis

**Document type:** Bounded diagnosis of an immutable search  
**Status:** Complete — Search #1 not overwritten; Search #2 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C  
**Does not:** re-run Coralys, retune the genome, promote the artifact, add MTF/VaR/financing, open a new schema, freeze Decision Engine v1.0, interpret CS-P-003  

`.cursor/rules/chronosentiment-core.mdc`: diagnose the sealed experiment; do not invent a corrective mapping; evaluate across instruments and regimes.

---

## Milestone (immutable)

> **CS-P-006-C Search #1 — Reproducible discovery, failed generalization.**

Identity: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`  
Bundle: `product_validation/CS-P-006/discovery/20260814T195327Z/`  
Diagnosis sidecar: `…/diagnosis/` (does not replace Search #1 files)

The loop worked. The policy is **not** promoted.

```text
Evaluation ─────X────→ Coralys
```

---

## What this diagnosis is allowed to conclude

Search #1 showed that **this frozen search space + configuration + 20-day objective** produced a candidate that did not generalize.

It does **not** show that Trend + Momentum + Volatility cannot contain useful structure.

The eight open possibilities remain open. The tables below only rank which of them are *supported by this archive*, which are *undecidable*, and which are *not yet justified as a new information family*.

---

## 1. Search-space utilization

The factory can sample Trend, Momentum, Volatility presence, 1–3 predicate conjunctions, and all three actions. Those capabilities were **not** sealed out of the representation.

The **selected** artifact used one predicate:

```text
Trend present ∧ Bearish  →  LONG
otherwise                →  NO_TRADE
```

| Capability | Accessible to factory | Used by selected artifact |
|------------|-----------------------|---------------------------|
| Trend | yes | yes (Bearish only) |
| Momentum | yes | no |
| Volatility | yes (presence only) | no |
| Conjunctions | yes | no |
| LONG | yes | yes |
| SHORT | yes | no |
| NO_TRADE | yes | yes (unmatched) |

Certified occupancy on all three slices: Volatility is **present on 91/91 rows**. Trend is only Bullish or Bearish (no Neutral). Momentum is only Positive or Negative (no Neutral).

So Volatility-as-presence cannot split this snapshot. That is a property of the certified state, not proof the factory failed to sample it. Momentum *can* split the snapshot and was unused by the winner. Whether Momentum/SHORT/conjunctions appeared in the **population** is not in the archive (see §2).

---

## 2. Population diversity — archive gap

Search #1 persisted generation-best identity and fitness only.

| Recorded | Value |
|----------|--------|
| Generation bests | 12 |
| Unique generation-best genomes | 2 (`d8363a93…` then `9eb80355…`) |
| Candidates given to selection | 2 |
| Population median / worst / diversity | **not recorded** |
| Development-best genome rules | **not serialized** |

We **cannot** say the population was 95% the same rule by generation 3. We also cannot say it remained diverse.

What the elite archive does show: the selected genome was already the generation-0 best. A different genome took the development-best slot at generation 2 and held it. Selection then preferred the generation-0 genome. That is protocol-correct. It is also a thin sample of what the search explored.

A later instrument may persist population identity counts. That is an **archive** change, not Search #2, and not a reason to alter TMV inputs.

---

## 3. Fitness trajectory

From `search_evidence.json` (development only):

| Generation | Best | Average |
|------------|------|---------|
| 0 | 0.016325 | 0.001754 |
| 1 | 0.016325 | 0.007548 |
| 2 | 0.017399 | 0.010913 |
| 3–11 | 0.017399 | 0.0105–0.0126 |

The recorded best improved once, then sat. Average rose toward ~0.012 and never caught the best. That is consistent with finding an easy local optimum at initialization, then a slightly better development genome, then elite stasis. Without median/worst/unique-genome counts, “collapsed” vs “still mixed around 0.012” is undecidable.

---

## 4. Why Bearish → LONG was attractive on search-visible data

Do not “correct” this to Bearish → SHORT. Ask why the frozen objective rewarded it.

Pooled 20-day raw close-to-close return after T:

| Slice | n Bearish | Mean raw if Bearish | LONG payoff | SHORT payoff | Mean raw if not Bearish |
|-------|-----------|---------------------|-------------|--------------|-------------------------|
| development | 49 | +0.01404 | +0.01404 | −0.01404 | −0.00308 |
| selection | 39 | +0.01569 | +0.01569 | −0.01569 | +0.01207 |
| evaluation (holdout) | 33 | −0.01606 | −0.01606 | +0.01606 | −0.00173 |

On development, subsequent 20-day returns after Bearish were positive, and after Bullish slightly negative. Longing Bearish and standing aside otherwise is exactly the sign pattern the objective would keep. Shorting Bearish would have been the losing sign on both search-visible slices.

On selection the same Bearish-positive pattern continued, but Bullish was also positive — standing aside left subsequent return on the table. The rule still survived because Bearish longs remained positive.

On evaluation the Bearish-positive relationship **reversed**. That is the generalization failure. It is not a reason to invert the rule by hand.

---

## 5. Instrument behaviour

Equal-weight fitness (mean of seven per-instrument means) matches Search #1: development 0.016325, selection 0.019938, evaluation −0.000229.

Development (search-visible): six of seven names were positive after Bearish. TCS was slightly negative. Largest contributors: MAHABANK +0.0388 (5 bars), ICICIBANK +0.0248, IDEA +0.0229. Not a single-name rule, but not uniform.

Selection (search-visible): MAHABANK +0.0811 on **4** bars dominates the slice. IDEA is already negative (−0.0165). The selection score is not a shared seven-name relationship.

Evaluation (holdout diagnosis only): IDEA −0.0806, MAHABANK −0.0602, RELIANCE −0.0258. HDFCBANK / ICICIBANK / INFY / TCS remain positive, some on very small Bearish counts (ICICIBANK n=1). The holdout mean near zero is cancellation, not a quiet zero on every name.

The two names that most helped the search-visible score are the names that most hurt the holdout. That is regime/instrument instability, not a prompt to drop IDEA and MAHABANK after seeing evaluation.

---

## 6. Action coverage

SHORT count is **0 / 0 / 0** on all three slices. That is **by construction** of the sealed rule (the only emitted actions are LONG and NO_TRADE), not a coincidence of “SHORT never matched.”

What we can say:

* Under this objective, SHORT of the Bearish state was the losing sign on development and selection. If the search locked onto that state, SHORT of it would not survive selection of *this* mapping.
* We cannot say SHORT was never sampled in the population. The factory can emit it. The archive does not list population actions.

NO_TRADE is real standing-aside: 42 / 52 / 58 of 91 rows. Selectivity was discovered. It is not “I don’t know.”

---

## 7. Temporal stability

The three partitions are the 2021–22 / 2022–23 / 2023–24 windows.

```text
development  Bearish → subsequent 20D  > 0
selection    Bearish → subsequent 20D  > 0
evaluation   Bearish → subsequent 20D  < 0
```

The discovered relationship is period-concentrated. It is not persistent across the frozen holdout.

---

## What is and is not justified next

**Not justified by −0.000229:** another search, a handwritten Bearish→SHORT, dropping two tickers, adding MTF/VaR, or changing the 20-day horizon to chase holdout.

**Supported as diagnosis, not as Search #2:**

* Volatility presence is constant on this snapshot, so this schema’s Volatility family cannot discriminate. A richer vol family would be a **new certified concept + new schema version**, and only if we decide that deficiency matters. It is not authorized here.
* Selection fitness was sensitive to one name with n=4. The objective aggregates seven names equally; it does not cap single-name leverage inside a name.
* The elite archive is too thin to claim the representation was fully explored.
* Momentum was representable and unused by the winner; unused ≠ inaccessible.

**Still open:** weak TMV information; too-simple rule lists; 20-day horizon mismatch; small history (13 timestamps per slice); unstable cross-section; interactions the schema cannot write (including instrument predicates, which 006-A forbids).

The research-gap decision is **CS-P-006-C.2**. Search #2 remains unauthorized. Search #1 stays on disk as the control.

---

## Code

| Piece | Location |
|-------|----------|
| Diagnosis | `adapters/chronosentiment/src/decision_support/policy_search_diagnosis.rs` |
| Binary | `src/bin/csp006_search_diagnosis.rs` |
| Runner | `run_csp006_search_diagnosis.sh` |
| Sidecar | `product_validation/CS-P-006/discovery/20260814T195327Z/diagnosis/` |
| Tests | `adapters/chronosentiment/tests/policy_search_diagnosis_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
