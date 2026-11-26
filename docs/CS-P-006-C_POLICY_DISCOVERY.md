# CS-P-006-C — Coralys TMV discovery (first search)

**Document type:** First discovery run record  
**Status:** Complete — Search #1 immutable (reproducible discovery, failed generalization)  
**Date:** 2026-08-14  
**Parent:** CS-P-006-A, CS-P-006-B, CS-P-006-B.1, CS-P-006-S1  
**Does not:** invent a handwritten confluence map, seed from CS-P-005 60D lake counts, add MTF/VaR/financing, reopen G-GATE, mutate B3/B4, open B5, freeze Decision Engine v1.0, interpret CS-P-003, retune against evaluation  

`.cursor/rules/chronosentiment-core.mdc`: same certified development state + frozen seed → same PolicyArtifact identity; ChronoSentiment decide path stays deterministic and never receives outcomes.

---

## Question this run answers

> Can Coralys search the certified Trend / Momentum / Volatility state and emit one sealed `PolicyArtifact`, such that repeating the search with the same frozen configuration reproduces the same artifact identity?

It does **not** ask whether the resulting policy is profitable, and it does **not** authorize hand-tuning if it is not.

```text
Certified 7-instrument state at T
        │
        ├── Trend
        ├── Momentum
        └── Volatility
                │
                ▼
             Coralys
                │
                ▼
       PolicyArtifact v1
                │
       ┌────────┼────────┐
       ▼        ▼        ▼
     LONG     SHORT   NO_TRADE
```

ChronoSentiment has stopped inventing the policy. Coralys learns a mapping from historical experience under the frozen protocol. ChronoSentiment evaluates the sealed artifact independently.

---

## What Coralys received

* development observations and permitted outcomes (evolution / fitness)
* selection observations and permitted outcomes (selection only)
* certified TMV state at T from CS-P-006-S1
* action space `{LONG, SHORT, NO_TRADE}`
* `PolicyArtifact` contract `csp006a.policy_artifact.1`

## What Coralys did not receive

* evaluation / TEST outcomes or performance
* forward observations
* G-GATE results or B3/B4 G-GATE conclusions
* handwritten Trend/Momentum rules
* MTF / VaR / financing variables
* thresholds chosen from historical returns
* the CS-P-005 60D lake counts (110 LONG / 85 SHORT / 85 unavailable SHORT) as seeds or labels

---

## Frozen search configuration

| Item | Value |
|------|--------|
| Engine | `coralys.moga.rulelist.v0` |
| Seed | 42 |
| Population | 32 |
| Generations | 12 |
| Elite | 4 |
| Mutation rate | 0.25 |
| Crossover rate | 0.8 |
| Tournament | 3 |
| Max rules | 16 |
| Horizon | **20 calendar days** (observation-path; not the 60D lake series) |
| Fitness | mean across 7 instruments of per-instrument mean signed traded return; `NO_TRADE` stands aside; untraded instrument contributes 0 |
| Snapshot | `c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6` |
| Partition | `4354c81ef546003b1d11ec98cba83dd5f8c56b13c8b6055b8451614abdc4cfca` |
| Methodology hash | `6e92ef3e097d52f923b6028258f6442bcb5de6163c45a94628dead9aa954e3a5` |

Genome: ordered first-match rule list over certified TMV predicates. Volatility is presence-only. No year predicates. No global ATR cutoff.

Domain names in reusable types: `development` / `selection` / `evaluation`. Protocol TRAIN / VALIDATION / TEST appear only on provenance (`TrainingProvenance` field names frozen in 006-A) and in this research record. Programme-phase identifiers stay out of reusable Coralys and ChronoSentiment types.

---

## Evidence bundle

`product_validation/CS-P-006/discovery/20260814T195327Z/`

| Output | Files | Contents |
|--------|-------|----------|
| 1. Search evidence | `SEARCH.md`, `search_evidence.json` | population metadata, generation lineage, fitness trajectory, selection record on **development** and **selection** only |
| 2. Selected PolicyArtifact | `selected_policy.json`, `SELECTED.md` | one sealed, hashed artifact |
| 3. Evaluation handoff | `EVALUATION.md`, `evaluation_handoff.json` | ChronoSentiment holdout score; **not** returned to Coralys |

`SHA256SUMS` and `PROVENANCE.md` sit beside those three outputs.

In-process repeat of the same development slice + seed produced the **same** `artifact_hash`.

---

## What the search selected

This mapping **emerged**. It was not written as a confluence candidate.

```text
Trend present ∧ Bearish  →  LONG
otherwise                →  NO_TRADE
```

* `n_rules`: 1
* `unmatched_action`: `NO_TRADE`
* artifact hash: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`
* genome identity: `d8363a93e5afe518b7a4cbb8f5c3ac59efcf396f0d318ccdae0dd683e9d730d3`

SHORT is in the action space and was available to the genome factory. The selected artifact does not emit SHORT. That is a search result, not a missing feature.

The evolution archive’s development-best genome (`9eb80355…`, fitness 0.017399) is **not** the sealed artifact. Selection used the selection slice and chose `d8363a93…` (development 0.016325; selection 0.019938). That separation is required by CS-P-006-B.

Candidate pool presented to selection: **2** unique genomes (per-generation best plus global best). Coralys MOGA does not return the final population; this first instrument records that limitation rather than expanding the engine.

---

## Independent ChronoSentiment holdout

Performed **after** the artifact was sealed. Coralys received no feedback.

| Slice | Mean signed traded return | Traded | Stood aside |
|-------|---------------------------|--------|-------------|
| development (search-visible) | 0.016325 | 49 | 42 |
| selection (search-visible) | 0.019938 | 39 | 52 |
| evaluation (handoff only) | −0.000229 | 33 | 58 |

The evaluation number is evidence that the frozen TMV search produced a policy whose holdout mean signed traded return is about zero. **Do not retune the genome. Do not add thresholds. Do not start CS-P-003 interpretation from this number.**

A poor or uninformative policy is a valid first-search outcome. It is information about the frozen research space, not a prompt to invent rules by hand.

---

## Acceptance

> Same certified seven-instrument development state + same frozen configuration/seed → same PolicyArtifact identity?

**Yes.** Hash `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`.

This is the beginning of a Coralys research instrument. It is not Decision Engine v1.0, not a promoted ChronoSentiment strategy, and not a replacement for `BaselineTrendMappingPolicy` on CS-P-002/003/004 product binaries.

---

## Immutability

Search #1 is a permanent research milestone. A later search, if ever authorized, is a **new** evidence directory. It must not overwrite this bundle.

Post-search diagnosis (no evolution): `docs/CS-P-006-C.1_SEARCH_DIAGNOSIS.md`.

## What this does not authorize

* Promoting this artifact as the ChronoSentiment strategy
* CS-P-006-D interpretation as a trading go
* Expanding the genome with MTF / VaR / leverage / financing because holdout was poor
* Search #2 before a justified information or instrumentation deficiency is frozen
* Feeding evaluation fitness back into Coralys
* B5, G-GATE v1.2, v1.1 rerun, real capital

---

## Code

| Piece | Location |
|-------|----------|
| Genome / operators | `adapters/chronosentiment/src/decision_support/policy_genome.rs` |
| Observation-path value | `observation_value.rs` |
| Evolution + selection | `policy_discovery.rs` |
| ChronoSentiment handoff | `policy_handoff.rs` |
| Binary | `src/bin/csp006_policy_discovery.rs` |
| Runner | `run_csp006_policy_discovery.sh` |
| Tests | `adapters/chronosentiment/tests/policy_discovery_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital.
