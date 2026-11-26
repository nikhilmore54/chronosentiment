# CS-P-006-C.3-I — Controlled implementation and identity-gated verification

**Document type:** Implementation and identity gate  
**Status:** Implementation PASS — Search #2 not run  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3  
**Does not:** run Search #2, retune Search #1, change the TMV information set, add indicators, drop IDEA or MAHABANK, invent ATR/Momentum cutoffs, use `unique_best` or `−regret` as fitness, feed evaluation to Coralys, overwrite Search #1 evidence, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy; Coralys discovers mappings; the same input must produce the same output; holdout does not re-enter the same search.

---

## What this freeze is

C.3 authorized a redesigned search protocol. This document implements that protocol and **verifies identity** against Search #1. It does **not** authorize a run.

```text
C.3 protocol FROZEN
        │
        ▼
C.3-I implementation          ← this document
        │
        ├── M.1 ProtocolValue
        ├── full living-population selection pool
        ├── C.2-O observer
        ├── N symbol matrices (required after a future seal)
        └── identity / lineage guards
        │
        ▼
Implementation verification
        │
        ├── same TMV snapshot
        ├── same 7 instruments
        ├── same 20D horizon
        ├── seed = 42
        ├── same MOGA parameters
        ├── evaluation inaccessible
        └── Search #1 remains byte-for-byte / evidence immutable
        │
        ▼
C.3-I IMPLEMENTATION PASS
        │
        ✕  not authorized
   AUTHORIZE RUN → Search #2
```

Search #1 remains `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`.

Sidecar: `product_validation/CS-P-006/discovery/20260814T195327Z/c3i/`

---

## Hard gate

`SEARCH_TWO_RUN_AUTHORIZED = false`.

`evolve_on_development_value` returns an error. C.3-I PASS is an implementation result. It is not a run authorization.

The Search #1 path (`evolve_on_development`, `select_on_selection`, `DevelopmentFitness`, `methodology_hash`) is unchanged.

---

## M.1 fitness (new path only)

For every `(instrument, T)`:

```text
LONG      →  R
SHORT     → -R
NO_TRADE  →  0
```

Then:

```text
instrument_value = mean(V for that instrument)
protocol_value   = mean(instrument_value across 7 instruments)
```

NO_TRADE enters the instrument mean as 0. An empty instrument is a protocol error, not a silent 0.

These quantities must not reach the fitness function:

```text
regret                 ❌
unique_best            ❌
accuracy               ❌
advantage_vs_no_trade  ❌
evaluation             ❌
```

They belong to the measurement layer (N). After a future seal, Table A and Table B remain mandatory.

Fitness must preserve magnitude. `+0.10%`, `+0.50%`, and `+5.00%` remain distinct values. They are not collapsed to GOOD / GOOD / GOOD. The same holds for losses.

---

## Selection pool

```text
population slot
      ↓
genome identity
      ↓
unique observed genome
      ↓
selection candidate
```

Not:

```text
every mutation ever produced
      ↓
selection pool
```

The pool is unique genome identities observed in **living-population slots**. Offspring edges that never entered a living slot are excluded. Deduplicate by identity. Score the pool on the selection slice with M.1 protocol V (`select_on_selection_value`).

---

## Observer and lineage

The C.2-O observer now records `living_slots` (serde default empty so C.2-P `ecology/archive.json` still deserializes). The observer must not consume search RNG. Seed 42 + observer ON = OFF = Search #1 artifact hash.

Identity lineage that must hold before any later run:

| Guard | Value |
|-------|--------|
| Snapshot | `c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6` |
| Universe | 7 certified names |
| Horizon | 20 calendar days |
| Seed / pop / gens / elite / mutation / crossover / tournament / max rules | 42 / 32 / 12 / 4 / 0.25 / 0.8 / 3 / 16 |
| Search #1 methodology hash | `6e92ef3e097d52f923b6028258f6442bcb5de6163c45a94628dead9aa954e3a5` |
| Search #1 `selected_policy.json` SHA-256 | `a973446fb2a62c046a3837898603d71830f6b4daaedf6ce0f7803d5364858c2f` |
| Evaluation | inaccessible to search-admissible scoring |

---

## NO_TRADE learnability

The representation can emit LONG, SHORT, and NO_TRADE from state at T. There is no coded threshold that converts a small positive `V` into NO_TRADE. A later search may learn to stand aside where both directional actions have poor expected value. That is a discovered mapping, not a handwritten boundary.

---

## Naming

Reusable types use `development_value`, `selection_value`, `evaluation_value`, `candidate_population`, and `decision_value`. Protocol TRAIN / VAL / TEST and C.3 remain provenance / workflow labels. Do not introduce `train_fitness`, `validation_candidates`, `test_score`, or `phase_c3_population`.

---

## What a later Search #2 must be judged by

Not “accuracy improved.”

Primary comparison:

```text
development value → selection value → evaluation value
```

Regret, unique-best, symbol-level matrices, action distribution, and population ecology are the diagnostic picture.

---

## Authorization

| Item | Status |
|------|--------|
| C.3 protocol | Authorized (frozen) |
| C.3-I implementation | **PASS** (`…/discovery/20260814T195327Z/c3i/`) |
| Search #2 evolution | **Not run by this document** — C.3-R is the run authorization |
| Promotion / CS-P-003 interpretation | Unchanged; CS-P-003 stays last |

Engine version remains **`unfrozen-dev`**. No real capital.
