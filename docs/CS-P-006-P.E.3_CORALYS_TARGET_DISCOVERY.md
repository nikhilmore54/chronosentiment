# CS-P-006-P.E.3 — Coralys Target Discovery

**Document type:** Product / research protocol  
**Status:** Specified — not started; waits for CS-P-007 confirmatory validation; P.E.2 / P.E.2.H remain the +5% control  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P.E, CS-P-006-P.E.2, CS-P-006-P.E.3.A  
**Predecessor note:** CS-P-006-P.E.B sketched this experiment early; this document is the canonical id  
**Does not:** run in this freeze, skip CS-P-007, replace P.E.2’s fixed 5%, rewrite P.E.1 or P.E.2 sidecars, mutate the 14 August seals, retune C3-002, run Search #3, start C.3-G, path-optimize a target from future OHLC, invent a handwritten ATR→% mapping and call it Coralys  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same C3-002 + same Coralys artifact → same direction **and** same execution intent. Future prices never choose the target.

---

## Hypothesis

> Given only the certified state available at T, can Coralys determine an appropriate target magnitude that is reproducible, temporally valid, and potentially more informative than a fixed target?

P.E.2 is the **control** (fixed `target_pct = 5.0%` attached at decision time). P.E.2.H executed that control on a historical clock (**PASS**). Live P.E.2 remains `AWAITING_NEXT_SESSION`. P.E.3 is the **treatment** (Coralys-derived execution parameters from state at T). Do not collapse them.

P.E.2 is not a test of whether 5% is a good target. P.E.3 is the first experiment that is allowed to ask about target quality — and only after the artifact contract is frozen and a generator artifact exists with `effective_timestamp` preceding every evaluated T.

We are not asking: can we find a target that worked?

> Can Coralys determine an execution target using only the information available at T, and does that information subsequently demonstrate value beyond the fixed-target control?

---

## Anti-leakage contract

```text
TARGET(T) = f(
    certified_state(T),
    Coralys_artifact,
    C3-002,
    permitted historical information ≤ T
)
```

```text
TARGET(T) ≠ f(future_price_path)
TARGET(T) ≠ f(realized_return)
TARGET(T) ≠ f(outcome)
```

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

> **P.E.3 is not authorized to learn the target from historical realized outcomes.**
>
> A Coralys target artifact must be frozen independently of the evaluation paths against which it is subsequently tested. Any target-generation methodology, parameters, model weights, or policy rules used to generate the target must have an identifiable artifact/version and effective timestamp preceding the evaluated decision.

```text
CORALYS_TARGET_SEARCH_AUTHORIZED = false
TARGET_LOOKAHEAD_AUTHORIZED = false
TARGET_PATH_OPTIMIZATION_AUTHORIZED = false
TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED = false
ASYMMETRIC_TARGET_AUTHORIZED = false
HORIZON_SEARCH_AUTHORIZED = false
CORALYS_TARGET_ARTIFACT_PRESENT = false
```

Search is not opened here. This document specifies the experiment; it does not run it.

---

## Two seals at T (do not merge)

The target must not appear to be part of C3-002.

```text
Decision
    ├── direction
    ├── certified state
    ├── policy artifact
    └── timestamp

Execution Intent
    ├── target
    ├── maximum hold
    ├── trigger semantics
    └── execution-contract artifact
```

Both are frozen at T. They answer different questions.

```text
Certified state at T
        │
        ▼
Frozen Coralys target artifact
        │
        ▼
target = X%
        │
        ▼
Execution Intent sealed
        │
        ▼
Future path
```

Not:

```text
historical path
     ↓
discover best target
     ↓
call it Coralys
```

That distinction is the difference between a legitimate research experiment and hindsight optimization. P.E.3 must remain exactly this constrained. Do not invent a target model in this freeze.

---

## What stays frozen until this experiment is authorized

* CS-P-007 confirmatory run (specified, not this experiment)
* P.E.1 historical +5% replay
* P.E.2 prospective fixed-contract lifecycle
* 14 August direction-only cohort
* C3-002 / Search #2
* Search #3 / C.3-G
* Adaptive / per-name targets
* Homepage comparison of P.E.3 vs P.E.2

---

## When it may start

**Not yet.** CS-P-007 must first establish whether frozen C3-002 plus the fixed +5% control has information on an untouched confirmatory sample. Building a Coralys target generator before that baseline would make the treatment uninterpretable.

Do not invent ATR→%. Do not search a target range. Do not optimize against historical realized V.

Live P.E.2 remains `AWAITING_NEXT_SESSION`. P.E.2.H is the historical +5% lifecycle control (**PASS**), not a statistical strategy backtest. This freeze does not start P.E.3 search.
