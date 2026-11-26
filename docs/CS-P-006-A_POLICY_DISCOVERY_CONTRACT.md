    # CS-P-006-A — Policy Discovery Contract

**Document type:** Frozen consumption contract  
**Status:** Active — contract only; no optimizer  
**Date:** 2026-08-14  
**Parent:** CS-P-006  
**Schema:** `csp006a.policy_artifact.1`  
**Does not:** invent a trading rule, run Coralys search, freeze split dates, regenerate B3/B4, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: same artifact + same information at T → same `TradingDecision`; outcomes never construct the decision.

---

## Success criterion

> ChronoSentiment can consume a sealed `PolicyArtifact` and emit a `TradingDecision` from historical state at T, and it cannot use future outcomes while doing so.

```text
PolicyArtifact + historical state at T  →  TradingDecision
PolicyArtifact + future outcome         →  forbidden
```

This PR defines the **representation and evaluator**. It does not discover a policy.

---

## Artifact

Coralys (later) produces; ChronoSentiment consumes:

```text
PolicyArtifact
├── schema_version
├── policy_id
├── policy_version
├── discovery_engine
├── discovery_run_id
├── input_schema
├── factor_definitions
├── action_space          LONG | SHORT | NO_TRADE  (all required)
├── rules / genome        ordered conjunctions → action
├── unmatched_action      required; Coralys chooses; evaluator does not invent it
├── training_provenance   TRAIN / VALIDATION / TEST roles
├── allowed_information_timestamp
├── artifact_hash
└── methodology_hash
```

`artifact_hash` is SHA-256 of the canonical identity payload **excluding** the hash field itself. `TradingDecision.policy_name` is `{policy_id}@{policy_version}`. The hash is copied into `mapping_rule` so decision identity traces the exact artifact.

---

## Genome (CS-P-006-A)

An ordered list of rules. Each rule is an AND of factor predicates over the certified input schema. First match wins. If none match, `unmatched_action` is used.

Certified input schema for this schema version (from Assessment Enrichment / CS-P-005, at T):

| Concept | Representable states |
|---------|----------------------|
| Trend | Bullish, Bearish, Neutral, absent |
| Momentum | Positive, Negative, Neutral, absent |
| Volatility | present / absent only (magnitude; no High/Low tertile) |

Instrument and timestamp are part of the replay information set. They are **not** genome predicates in `csp006a.policy_artifact.1`. Richer predicates are a later schema version, not invented here.

This representation can express NO_TRADE on any conjunction. It does **not** assert that any particular conjunction *should* be NO_TRADE.

---

## Evaluation semantics

ChronoSentiment wraps a sealed artifact as `ArtifactDecisionPolicy` (`DecisionPolicy`).

* Inputs: `AssessmentProfile` at T (factors ≤ T already enforced by replay)
* Outputs: `PolicyDecision` → product `TradingDecision`
* Confidence remains `UNAVAILABLE`
* Fabricated assessment scores are not copied onto evidence
* `as_of` is not a trading feature in this schema; replay already drops inputs after T
* `allowed_information_timestamp` is **discovery provenance** (latest T Coralys may have trained on). Evaluation on later TEST / forward timestamps is allowed and required. The evaluator does not reject `as_of` after that cutoff.
* Training windows, if present, must be complete and strictly ordered TRAIN then VALIDATION then TEST. CS-P-006-A does **not** fill those dates.

Determinism: same sealed artifact + same profile at T → same action and same decision identity.

---

## Provenance roles

```text
TRAIN        Coralys may learn (outcomes allowed here only)
VALIDATION   candidate selection
TEST         untouched evaluation — never participates in evolution
```

Exact calendar windows are frozen in **CS-P-006-B.1** from the certified seven-instrument snapshot. This contract does not invent those dates.

Until a Coralys-discovered artifact is sealed:

* Contract fixtures use `discovery_engine = contract.fixture` and empty windows
* A Coralys-discovered candidate (`discovery_engine` prefix `coralys.`) **must** carry complete windows from CS-P-006-B.1

Forbidden `discovery_engine` values: `chronosentiment.handwritten`, `threshold.grid`.

---

## What this is not

* Not `BaselineTrendMappingPolicy` replacement on CS-P-002/003/004 product binaries
* Not a hand-designed confluence candidate
* Not an optimizer, fitness function, or grid search
* Not CS-P-006-B (protocol) or CS-P-006-C (search)

---

## Code

| Piece | Location |
|-------|----------|
| Types + evaluator | `adapters/chronosentiment/src/decision_support/policy_artifact.rs` |
| Tests | `adapters/chronosentiment/tests/policy_artifact_contract_tests.rs` |
| Fixture | `fixtures/contracts/csp006a.policy_artifact.empty_rules.json` |

Engine version remains **`unfrozen-dev`**. No real capital.
