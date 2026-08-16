# CS-P-006-P.E.3.A — Coralys Target Artifact Contract

**Document type:** Product / research contract  
**Status:** Specified — contract only; no generator; no first target  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P.E.3  
**Does not:** invent a target model, map ATR→%, search a target range, optimize against historical paths, replace P.E.2’s +5% control, start Search #3, start C.3-G, mutate 14 August, rewrite P.E.1 / P.E.2 / P.E.2.H  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same C3-002 + same frozen Coralys target artifact → same Execution Intent. Future OHLC never chooses the target.

---

## What this freeze is

P.E.3 is the later treatment experiment. It does **not** run in this freeze. It waits for **CS-P-007** confirmatory validation of C3-002 + fixed +5%. This document only freezes the artifact contract.

It freezes the **artifact contract** that any later Coralys target generator must satisfy before a single target may be sealed.

```text
Certified state at T
        │
        ▼
Coralys target generator     ← not built in this freeze
        │
        ▼
Target Artifact              ← contract specified here
        │
        ▼
Execution Intent
        │
        ▼
seal
        │
        ▼
future
```

Research before tuning. Sealed information before outcome. Evidence before claims.

---

## Ontology (do not collapse)

The Coralys target is **not evidence**. It is an execution hypothesis generated from the state.

```text
INTELLIGENCE
    ↓
DECISION
    ↓
EXECUTION INTENT
    ↓
OBSERVATION
    ↓
EVIDENCE
```

| Object | Question |
|---|---|
| Decision | What direction? |
| Execution Intent | How should that direction be acted upon? |
| Observation | What happened afterward? |
| Evidence | The resulting measured record |

P.E.2.H is the **control**: C3-002 + fixed +5% + 20 market sessions. P.E.3 is the **treatment**: C3-002 + Coralys-derived target + the same temporal rules. Same direction engine. Same Observatory. Different execution-intent generator.

---

## Two artifacts (do not merge)

Like C3-002, there is a frozen **generator** and a per-T **output**. The output is not the generator. The generator is not the Observatory record.

```text
CoralysTargetGeneratorArtifact
    ├── artifact_id
    ├── content_hash
    ├── generator_id
    ├── generator_version
    ├── methodology_hash
    ├── parameter/weight identity
    └── effective_timestamp   (must precede every evaluated T)

CoralysTargetOutput at T
    ├── target_pct
    ├── max_holding_sessions
    ├── trigger semantics (inherited from Execution Contract unless later authorized)
    ├── generator artifact hash
    └── sealed_at_t = true
```

The output becomes Execution Intent. It does not become Evidence.

---

## Anti-overfitting rule (central)

> **P.E.3 is not authorized to learn the target from historical realized outcomes.**
>
> A Coralys target artifact must be frozen independently of the evaluation paths against which it is subsequently tested. Any target-generation methodology, parameters, model weights, or policy rules used to generate the target must have an identifiable artifact/version and effective timestamp preceding the evaluated decision.

```text
TARGET_T =
    f(
        certified_state_T,
        C3-002,
        Coralys_artifact
    )

TARGET_T ≠
    f(
        future_OHLC,
        future_return,
        realized_V,
        target_hit
    )
```

```text
CORALYS_TARGET_SEARCH_AUTHORIZED = false
TARGET_LOOKAHEAD_AUTHORIZED = false
TARGET_PATH_OPTIMIZATION_AUTHORIZED = false
TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED = false
ASYMMETRIC_TARGET_AUTHORIZED = false
HORIZON_SEARCH_AUTHORIZED = false
CORALYS_TARGET_ARTIFACT_PRESENT = false
```

We are not asking: can we find a target that worked?

We are asking:

> Can Coralys determine an execution target using only the information available at T, and does that information subsequently demonstrate value beyond the fixed-target control?

---

## Required fields (generator artifact)

When an artifact is later admitted, it must carry:

| Field | Rule |
|---|---|
| `artifact_id` | Stable product id |
| `content_hash` | SHA-256 of the identity payload |
| `generator_id` / `generator_version` | Named, versioned |
| `methodology_hash` | Frozen description of how T-state maps to a target |
| `effective_timestamp` | Strictly before every T against which it is evaluated |
| `input_schema` | Subset of authorized T-state fields only |
| `output_schema` | `target_pct` and, if later authorized, hold/triggers |
| `evaluation_paths_excluded` | Must not include the paths later used as P.E.3 evidence |

Identity-gate: ChronoSentiment refuses to seal a Coralys Execution Intent unless `CORALYS_TARGET_ARTIFACT_PRESENT` is true **and** `effective_timestamp < T`.

---

## What is not in this freeze

* ATR→% mapping
* Target-range search
* Per-name adaptive 5%
* Asymmetric long/short targets
* Horizon search
* Homepage comparison of P.E.3 vs P.E.2
* Calling a handwritten rule “Coralys”

P.E.2.H remains the executed +5% control. Live P.E.2 remains `AWAITING_NEXT_SESSION`. 14 August remains decision-only. Search #3 and C.3-G stay closed.
