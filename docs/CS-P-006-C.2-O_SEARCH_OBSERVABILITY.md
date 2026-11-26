# CS-P-006-C.2-O — Search observability

**Document type:** Instrumentation implementation  
**Status:** Complete — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.2  
**Does not:** run Search #2, change Search #1, change fitness/seed/universe/horizon, choose a volatility encoding, amend `csp006a.policy_artifact.1`, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: same seed + same inputs → same PolicyArtifact whether observability is on or off; evaluation outcomes never enter fitness or the archive scorer.

---

## What this freeze does

Implements the C.2 observability contract as a **read-only** generation observer.

```text
same seed + same inputs + observability OFF
             =
same seed + same inputs + observability ON
             ↓
same PolicyArtifact
```

Search #1 (`9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`) remains the immutable control. `evolve_on_development` is unchanged (observer absent). `evolve_on_development_observed` attaches the observer only.

C.2 stays frozen. Volatility representation is still not chosen. C.3 is still a later decision.

---

## What is persisted

Per generation (from already-evaluated individuals; no extra RNG):

* population size
* unique genome count
* best / median / mean / worst development fitness
* action-symbol histogram (LONG / SHORT / NO_TRADE present on the genome)
* factor-consumption histogram (Trend / Momentum / Volatility mentioned)
* serialized generation-best rules
* near-best genomes (fitness within `1e-9` of best)

After each offspring is produced (after mutation/processors, before it enters the next population):

* parent_a / parent_b / child identities

The Coralys engine did not previously expose parent pointers. This records the parent genomes the loop already held. It does not add selection pressure, extra RNG, or a new Genome type.

After selection (development and selection slices only):

* per-instrument mean signed traded return, traded count, stood-aside count

Evaluation is rejected by `per_instrument_scores` and by `score_genome`.

---

## Boundary

The observer must not:

* alter selection pressure
* consume search RNG
* change mutation/crossover ordering
* alter fitness evaluation
* change population ordering

`methodology_hash` is unchanged. Enabling observation is not a new discovery methodology.

---

## Stop

Observability contract verified by regression. **Do not run Search #2.** Whether C.3 is justified remains an open decision.

---

## Code

| Piece | Location |
|-------|----------|
| Engine hook (optional, no-op if absent) | `coralys-moga` `GenerationObserver` |
| Archive | `adapters/chronosentiment/src/decision_support/search_observability.rs` |
| Observed evolve | `evolve_on_development_observed` |
| Tests | `adapters/chronosentiment/tests/csp006c2o_observability_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital.
