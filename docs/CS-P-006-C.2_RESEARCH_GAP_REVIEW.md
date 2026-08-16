# CS-P-006-C.2 — Research instrumentation and information-gap review

**Document type:** Research-gap decision  
**Status:** Complete — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C, CS-P-006-C.1  
**Does not:** run Search #2, retune Search #1, choose a volatility encoding, add MTF/VaR/financing, amend `csp006a.policy_artifact.1`, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: keep the decide path deterministic; do not invent methodology; do not repair a holdout number.

---

## Preserved control

```text
CS-P-006-C
    Search #1
       ↓
PolicyArtifact 9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0
       ↓
C.1 diagnosis
       ↓
FAILED GENERALIZATION
       ↓
NO RETUNING
```

This is a Coralys research result, not a failed project. ChronoSentiment does not secretly fix Coralys after seeing evaluation.

---

## Conclusion (this freeze)

> Coralys can discover non-obvious policies reproducibly, but the first discovered policy did not generalize. We do not yet know whether the problem is the information space, the policy representation, search instrumentation, or the absence of a stable TMV relationship.

C.2 decides **what is justified to investigate next**. It does not pick a winner among those causes, and it does not authorize another search.

| Finding | Meaning | C.2 decision |
|---------|---------|--------------|
| Reproducible artifact | Discovery machinery works | Keep Search #1 as control |
| Bearish → LONG emerged | We are not imposing the strategy | Do not invert the rule |
| Development + selection positive | Historical structure existed | Not a reason to promote |
| Evaluation reversed | Structure was not stable | Failed generalization stands |
| Momentum could discriminate, unused by winner | Available information may have been unexplored | Undecidable without a population archive |
| Volatility present on every row | Presence-only Volatility cannot split this snapshot | Information-model gap identified; **encoding not chosen** |
| Only 2 genomes archived | Search observability is inadequate | **Justified engineering deficiency** |
| Generation-best stagnated early | May be under-powered; collapse unproven | Do not claim premature convergence |
| MAHABANK dominated selection | Cross-instrument robustness needs visibility | Observability requirement, not a ticker drop |

---

## Answers to the six C.2 questions

### 1. What did Coralys actually explore?

**Unknown beyond the elite archive.** Search #1 recorded two unique generation-best genome identities and the sealed winner’s rules. The development-best genome’s rules were not serialized. The rest of each generation of 32 is gone.

We know the *factory can* emit Trend, Momentum, Volatility presence, conjunctions, LONG, SHORT, and NO_TRADE. We do not know which of those appeared in the population.

### 2. How much policy diversity existed?

**Undecidable.** Median, worst, and unique-genome counts per generation were not persisted. Average fitness rose toward ~0.012 while best sat at 0.0174 after generation 2. That is compatible with either a mixed population or a tight cluster under the best. C.1 already forbade claiming 95% collapse.

### 3. Which factors were consumed by candidate policies?

**Winner only:** Trend (Bearish). Momentum and Volatility were not consumed by the sealed artifact. Population-level factor consumption is not in the archive. Unused by the winner ≠ unused by the search.

### 4. Was the search sufficiently observable to diagnose convergence?

**No.** Convergence, premature collapse, and “near-best alternatives” cannot be diagnosed from generation-best hashes. This is the clearest **engineering** deficiency. It is justified to specify an observability contract before any future search. Specifying that contract is not Search #2 and does not change Search #1.

### 5. Does the current volatility representation contain discriminative information?

**No, not as certified for this genome.** On the CS-P-006-S1 slices used by Search #1, Volatility is present on 91/91 observations in development, selection, and evaluation (273/273 states). In `csp006a.policy_artifact.1`, Volatility may be present or absent only.

```text
Volatility present = true   → matches every certified row
Volatility present = false  → matches none
```

`atr_14 AVAILABLE` therefore carries essentially no state information for discovery. That is different from “ATR should have a hand-chosen threshold.” CS-P-005 already forbade a global ATR cutoff because ATR is in price units.

### 6. Is there a justified reason to change the certified information representation?

**Identified, not chosen.** There is a justified *question*:

> Does volatility need a scale-normalized or otherwise explicitly certified state before it can participate in discovery?

There is **not** yet a justified *answer*. A future representation would require its own fidelity argument, a new `schema_version`, and a frozen protocol. C.2 does not invent High/Low tertiles, z-scores, or instrument-relative ranks.

CS-P-006-V’s broader families (risk, cost, capital, financing) remain vision. They are not smuggled in here because evaluation was −0.000229.

---

## Two deficiencies, ranked

### A. Search instrumentation (justified to specify now)

A future observable search — if ever authorized — must persist at least:

| Record | Purpose |
|--------|---------|
| Per generation: unique genome count | Diversity |
| Per generation: best / median / worst / mean fitness | Trajectory, not just the elite |
| Population action-symbol histogram | How often LONG / SHORT / NO_TRADE appear in genomes |
| Population factor-consumption histogram | How often Trend / Momentum / Volatility are used |
| Serialized rules of every unique generation-best | Not hashes only |
| Near-best genomes (identity + rules + fitness) | “Were alternatives almost as good?” |
| Selected candidate’s per-instrument development and selection scores | Cross-name visibility (C.1 computed this after the fact for the winner) |

This contract lives in `search_observability.rs`. Implementing a collector that mutates Coralys MOGA or re-runs Search #1 is **not** this document. Search #1 evidence **fails** this contract; that failure is the finding.

### B. Volatility representation (justified as a question, not a design)

```text
Current certified concept:
  Volatility = present | absent
  On S1: present everywhere

Potential future certified state:
  Volatility = some reproducible, scale-aware state
```

Do not choose the state set here. Do not put a numeric ATR cutoff in a genome. Do not treat CS-P-005 quantiles as labels.

---

## What C.2 does not authorize

* C.3 / Search #2
* Changing Search #1 fitness, seed, horizon, or universe
* Hand-written Bearish → SHORT
* Dropping IDEA or MAHABANK after holdout
* A new PolicyArtifact schema
* MTF / VaR / leverage / financing variables
* Feeding evaluation back to Coralys

The next authorized step, if any, is to **implement the observability contract** (still without a new search), or to open a separate information-certification programme for volatility. Either one is a later freeze. Neither is a rescue of the holdout number.

---

## Code

| Piece | Location |
|-------|----------|
| Observability contract | `adapters/chronosentiment/src/decision_support/search_observability.rs` |
| Tests | `adapters/chronosentiment/tests/csp006c2_gap_review_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
