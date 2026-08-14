# CS-P-006 — Coralys Policy Discovery

**Document type:** Product / research programme brief  
**Status:** Active — CS-P-006-A in progress; optimizer not started  
**Date:** 2026-08-14  
**Parent:** CS-P-001, CS-P-CLEAN-002, CS-P-004, CS-P-005  
**Does not supersede:** EV-GOV-003, G-Extension Methodology v1.1, B3, B4, CS-P-002-R1, CS-P-003  
**Does not open:** G-GATE v1.2, B5, v1.1 rerun, Decision Engine v1.0 freeze, real capital  

This is **not** a G-GATE protocol and must not be used to reopen v1.1.

`.cursor/rules/chronosentiment-core.mdc`: deterministic evaluation; no invented methodology; no randomness on the ChronoSentiment decide path.

---

## Question

> Can Coralys discover a decision policy from the historical information available at T, and can ChronoSentiment independently evaluate that policy without allowing future outcomes to influence the decision?

Hand-writing a confluence rule (for example Bullish + Positive → LONG) is **not** an answer to this question.

---

## Boundary

```text
                 B4 / certified historical data
                              │
                              ▼
                    ┌──────────────────┐
                    │      Coralys     │
                    │ Policy Discovery │
                    │ (TRAIN / VAL)    │
                    └────────┬─────────┘
                             │
                       Policy Artifact
                             │
                             ▼
                  ┌─────────────────────┐
                  │  ChronoSentiment    │
                  │  Replay / Ledger /  │
                  │  Outcome / Perf.    │
                  └──────────┬──────────┘
                             │
                     Independent evidence
                     (TEST, then CS-P-003)
```

**Coralys discovers. ChronoSentiment evaluates.**

ChronoSentiment remains the domain decision/evaluation system. It does not search, evolve, or retune a policy. Coralys may use subsequent outcomes **during TRAIN only**. The ChronoSentiment `decide_at(T)` path never receives those outcomes.

---

## Sequence (strict)

| Step | ID | Purpose | This PR |
|------|----|---------|---------|
| A | CS-P-006-A | Machine-readable policy contract Coralys produces and ChronoSentiment consumes | **Yes** |
| B | CS-P-006-B | Freeze research protocol (splits, fitness, stop rules) from certified B4 coverage | No |
| C | CS-P-006-C | Coralys search / evolution on TRAIN; candidate selection on VALIDATION | No |
| D | CS-P-006-D | ChronoSentiment evaluation of a sealed candidate on TEST | No |
| E | CS-P-003 | Forward/paper clock as last-mile confirmation of a defensible candidate | Independent; already running |

Do not begin C until B is frozen. Do not invent TRAIN / VALIDATION / TEST calendar windows in CS-P-006-A code.

The **test set breaks the learning loop**. Once a candidate reaches TEST, the result is evidence, not training material.

---

## Action space

Coralys must be able to emit, and ChronoSentiment must be able to evaluate:

```text
LONG | SHORT | NO_TRADE
```

Selectivity is a discovery question. CS-P-006 does **not** prescribe a NO_TRADE threshold or a hand-designed conjunction.

---

## Stop conditions

* No B4 mutation, no B5
* No modification of v1.1; no G-GATE rerun
* No real trading
* No forward-data feedback into discovery
* No threshold grid search disguised as learning
* No hand-designed candidate promoted as the ChronoSentiment strategy
* No tuning against the final test set
* No feeding Performance results back into ChronoSentiment’s decide path
* CS-P-003 continues independently and is not the discovery laboratory
* `BaselineTrendMappingPolicy` remains an explicit fixture, not replaced by this programme until a sealed candidate exists
