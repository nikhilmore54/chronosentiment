# CS-P-006-C.2-P — Search #1 population ecology

**Document type:** Bounded analysis of an immutable search  
**Status:** Complete — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.2-O  
**Does not:** run Search #2, change Search #1, change fitness/seed/universe/horizon, choose a volatility encoding, amend `csp006a.policy_artifact.1`, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: diagnose the sealed experiment; same seed + same inputs remain the Search #1 control; do not invent methodology after seeing the holdout.

---

## What this freeze is

Analysis of the Search #1 evolutionary ecology, materialized by an **identity-gated C.2-O replay**.

```text
same seed + same inputs + observability ON
        ↓
PolicyArtifact 9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0
        ↓
SearchArchive
        ↓
population ecology report
```

This is **not** Search #2. The binary refuses to write if the replayed artifact is not Search #1.

C.2 stays frozen. Volatility encoding is still not chosen. C.3 remains a later decision.

Sidecar (does not replace Search #1 files):

`product_validation/CS-P-006/discovery/20260814T195327Z/ecology/`

---

## Pre-declared classification

A factor or action family is:

| Occupancy | Rule |
|-----------|------|
| ABSENT | 0 population-slots |
| TRACE | present, but fewer than 8 slots or fewer than 3 generations |
| RECURRENT | at least 8 slots and at least 3 generations |

Diversity band from mean unique-genome count: LOW `< 4`, MEDIUM `4–8`, HIGH `≥ 8`.

Overall verdict:

```text
SEARCH-SPACE EXPLORED
  iff Momentum RECURRENT ∧ SHORT RECURRENT ∧ diversity HIGH

SEARCH-SPACE UNDER-EXPLORED
  iff Momentum is ABSENT or TRACE
  ∧ SHORT is ABSENT or TRACE

otherwise INDETERMINATE
```

The Momentum question (C.2): unused by the winner means **A** if Momentum was RECURRENT, **B** if ABSENT or TRACE.

These gates were declared before the replay numbers were read.

---

## Verdict

```text
SEARCH-SPACE EXPLORED
```

Momentum question:

```text
A — Coralys explored Momentum-rich candidates and the winner still did not use Momentum.
```

Not B. The factory-accessible families were in the living population. The simple sealed rule is not evidence that Coralys never looked.

**C.3 / Search #2 is not authorized by this document.**

---

## Answers

Population size is 32 for all 12 generations (384 evaluated slots). Replay sealed `9a887827…71ac0`.

### How many unique genomes existed per generation?

| Generation | Unique | Best | Median | Mean | Worst |
|------------|--------|------|--------|------|-------|
| 0 | 32 | 0.016325 | 0.001884 | 0.001754 | −0.017399 |
| 1 | 30 | 0.016325 | 0.008982 | 0.007548 | −0.006137 |
| 2 | 30 | 0.017399 | 0.013120 | 0.010913 | −0.009396 |
| 3 | 28 | 0.017399 | 0.013340 | 0.011551 | −0.006928 |
| 4 | 32 | 0.017399 | 0.013340 | 0.011659 | 0.000000 |
| 5 | 32 | 0.017399 | 0.013340 | 0.011634 | 0.000000 |
| 6 | 32 | 0.017399 | 0.013340 | 0.010493 | 0.000000 |
| 7 | 32 | 0.017399 | 0.013340 | 0.010993 | −0.006137 |
| 8 | 32 | 0.017399 | 0.016325 | 0.010621 | −0.017399 |
| 9 | 29 | 0.017399 | 0.016325 | 0.012640 | −0.017399 |
| 10 | 31 | 0.017399 | 0.016325 | 0.012572 | −0.006137 |
| 11 | 29 | 0.017399 | 0.016325 | 0.012246 | −0.013120 |

Mean unique count **30.75** (min 28, max 32). Diversity band **HIGH**.

### How quickly did diversity change?

It did not collapse. Unique count never fell below 28. C.1’s “95% the same rule by generation 3” is **rejected** for this archive.

What did tighten is the *fitness* distribution: median rose from 0.0019 to the selected winner’s development fitness (0.016325) by generation 8, while worst remained able to go negative. The population stayed syntactically diverse and became fitter, not identical.

### What proportion used Trend, Momentum, and Volatility?

| Family | Slots | Share of 384 | Generations | Occupancy |
|--------|------:|-------------:|------------:|-----------|
| Trend | 358 | 93.2% | 12 | RECURRENT |
| Momentum | 327 | 85.2% | 12 | RECURRENT |
| Volatility | 334 | 87.0% | 12 | RECURRENT |

“Used” means a genome mentioned the concept in at least one predicate. That includes dead or contradictory conjunctions (the development-best contains unsatisfiable Volatility present ∧ absent clauses). Occupancy is not proof of a coherent factor thesis. It is proof the family was not sealed out of the search.

### How often did LONG / SHORT / NO_TRADE appear?

| Action | Slots | Share | Occupancy |
|--------|------:|------:|-----------|
| LONG | 350 | 91.1% | RECURRENT |
| SHORT | 243 | 63.3% | RECURRENT |
| NO_TRADE | 362 | 94.3% | RECURRENT |

SHORT declined from 26/32 in generation 0 to 15–17 later, but it never left the population. The sealed winner emits no SHORT; that is selection among elites, not absence of SHORT in the ecology.

### Were there near-best candidates materially different from the winner?

Yes.

* 43 unique near-best identities (fitness within `1e-9` of that generation’s best).
* 42 of those differ from the selected winner in factor or action profile.
* All 42 mention Momentum. 40 mention Volatility. 22 also emit SHORT.

The development-best identity `9eb80355…` (fitness 0.017399) uses Trend, Momentum, and Volatility and can emit LONG, SHORT, and NO_TRADE. The selected winner `d8363a93…` (fitness 0.016325) is Trend-Bearish → LONG only.

Search #1’s selection step still only compared **two** unique generation-best genomes. The ecology now shows a crowded development plateau that the original elite archive did not persist.

### Did the population explore Momentum-containing policies?

**Yes.** 327/384 slots, every generation. That is A, not B.

### Did it explore SHORT policies?

**Yes.** 243/384 slots, every generation.

### Was the winner genuinely dominant, or one of many nearly equivalent candidates?

Neither “the only genome” nor “just one of a pile that selection never distinguished.”

* On development, the winner was **not** the best after generation 1. A higher-fitness genome appeared at generation 2 and held the elite slot.
* Many other genomes later tied that higher development fitness (near-best count grew from 1 to 8–10).
* Selection, using only the two persisted generation-best identities, preferred the simpler generation-0 rule because it scored higher on the selection slice.

The winner is a **selection-preferred simple rule**, not a population monopoly.

### How did candidate performance vary by instrument?

Selected genome only; development and selection slices; evaluation not scored.

| Instrument | Dev mean | Dev traded | Sel mean | Sel traded |
|------------|----------|------------|----------|------------|
| HDFCBANK.NS | +0.001401 | 7 | +0.019643 | 7 |
| ICICIBANK.NS | +0.024771 | 5 | +0.010712 | 7 |
| INFY.NS | +0.018261 | 5 | +0.007676 | 7 |
| RELIANCE.NS | +0.009300 | 8 | +0.004786 | 5 |
| TCS.NS | −0.001157 | 10 | +0.032206 | 3 |
| IDEA.NS | +0.022897 | 9 | −0.016543 | 6 |
| MAHABANK.NS | +0.038802 | 5 | +0.081086 | 4 |

This confirms C.1: selection is sensitive to MAHABANK on four bars. That is visibility, not a ticker drop.

---

## What this does and does not mean

**Does mean:** under the frozen Search #1 configuration, Coralys sampled Trend, Momentum, Volatility, LONG, SHORT, and NO_TRADE throughout the run. The unused-by-winner status of Momentum is not an exploration failure.

**Does not mean:** every possible rule list was tried; Momentum predicates were coherent; Volatility presence carried information; TMV is sufficient; or another search with the same representation is justified.

Volatility remains **present, not designed**. Recurrent Volatility *mentions* in genomes do not make presence-only Volatility discriminative on S1 (still 91/91 present). That is still C.2’s information question, not an encoding choice.

Failed generalization of `9a887827…71ac0` stands. Evaluation was not scored here and was not fed back.

---

## What C.2-P does not authorize

* C.3 / Search #2
* Changing Search #1 fitness, seed, horizon, or universe
* Hand-written Bearish → SHORT
* Dropping IDEA or MAHABANK
* A new PolicyArtifact schema
* A volatility encoding
* Feeding evaluation back to Coralys

Whether C.3 is scientifically justified is still a later decision. This freeze only removes “maybe they never looked at Momentum” as a reason to rerun the same search.

---

## Code

| Piece | Location |
|-------|----------|
| Analysis | `adapters/chronosentiment/src/decision_support/population_ecology.rs` |
| Binary | `src/bin/csp006_population_ecology.rs` |
| Runner | `run_csp006_population_ecology.sh` |
| Sidecar | `product_validation/CS-P-006/discovery/20260814T195327Z/ecology/` |
| Tests | `adapters/chronosentiment/tests/csp006c2p_population_ecology_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
