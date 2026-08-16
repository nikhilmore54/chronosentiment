# CS-P-006 — Coralys Policy Discovery

**Document type:** Product / research programme brief  
**Status:** Research frozen at C.3-F; C.3-G question only; P Decision Observatory authorized  
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
| A | CS-P-006-A | Machine-readable policy contract Coralys produces and ChronoSentiment consumes | Complete |
| B | CS-P-006-B | Freeze research protocol; split dates only after 7-instrument coverage PASS | **Rules frozen; dates frozen in B.1** |
| S1 | CS-P-006-S1 | Disposable 7-instrument research snapshot (not B4/B5) | **Certified PASS / READY** |
| B.1 | CS-P-006-B.1 | Freeze TRAIN / VALIDATION / TEST from S1 coverage | **PASS** |
| C | CS-P-006-C | Coralys search / evolution on TRAIN; candidate selection on VALIDATION | **Complete / immutable** — Search #1 `9a887827…971ac0`; failed generalization; not promoted |
| C.1 | CS-P-006-C.1 | Diagnose Search #1 (no second search) | **Complete** |
| C.2 | CS-P-006-C.2 | Instrumentation and information-gap review | **Complete** — Search #2 not authorized |
| C.2-O | CS-P-006-C.2-O | Search observability (no semantic change) | **Complete** — C.3 not decided |
| C.2-P | CS-P-006-C.2-P | Population ecology of Search #1 | **Complete** |
| C.2-R | CS-P-006-C.2-R | Recommendation vs realized outcome | **Complete** |
| C.2-S | CS-P-006-C.2-S | Selection path + continuous decision-value review | **Complete** |
| C.2-D | CS-P-006-C.2-D | Decision-value landscape of the 273 recommendations | **Complete** — advantage is not fitness |
| M | CS-P-006-M | Decision-value model (protocol questions) | **Milestone** — problem stated |
| M.1 | CS-P-006-M.1 | Decision-value specification (12 clauses) | **Frozen** — regret is not fitness; Search #2 not authorized |
| N | CS-P-006-N | Decision-value research harness | **Implemented** — symbol matrices required |
| C.3 | CS-P-006-C.3 | Redesigned search protocol (same TMV; M.1 V) | **Protocol authorized** — Search #2 not started |
| C.3-I | CS-P-006-C.3-I | Implementation + identity gate | **Implemented** — Search #2 not run by that document |
| C.3-R | CS-P-006-C.3-R | One authorized Search #2 run | **Frozen** — no iteration |
| C.3-C | CS-P-006-C.3-C | Sealed #1 vs #2 review | **Complete** — Search #3 not authorized |
| C.3-D | CS-P-006-C.3-D | Search #2 live-rule ecology | **Complete** — candidate, not promoted |
| C.3-E | CS-P-006-C.3-E | Search #2 discovered-rule persistence | **Complete** — no pass threshold; no Search #3 |
| C.3-F | CS-P-006-C.3-F | Certified TMV state × action landscape | **Frozen** — no product claim; no Search #3 |
| C.3-G | CS-P-006-C.3-G | Regime-persistence question | **Stated** — experiment not authorized; research loop stopped |
| P | CS-P-006-P | Decision Observatory | **Maturity countdown started** — prospective OBSERVING; no early peek; C.3-G untouched |
| D | CS-P-006-D | ChronoSentiment evaluation of a sealed candidate on TEST | Handoff written; **not interpreted**; not a promoted strategy |
| E | CS-P-003 | Forward/paper clock as last-mile confirmation of a defensible candidate | Independent; already running |

Do not invent TRAIN / VALIDATION / TEST calendar windows from five-instrument B4 years. The frozen partition is CS-P-006-B.1.

The **test set breaks the learning loop**. Once a candidate reaches TEST, the result is evidence, not training material.

---

## Action space

Coralys must be able to emit, and ChronoSentiment must be able to evaluate:

```text
LONG | SHORT | NO_TRADE
```

Selectivity is a discovery question. CS-P-006 does **not** prescribe a NO_TRADE threshold or a hand-designed conjunction.

First search uses certified Trend / Momentum / Volatility only. Broader decision-value families (risk, cost, financing, capital) are vision in `docs/CS-P-006-V_DECISION_VALUE.md`, not this experiment’s genome.

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
* No copying broker MTF schedules, VaR multipliers, or financing-cost cutoffs into a genome
* `BaselineTrendMappingPolicy` remains an explicit fixture, not replaced by this programme until a sealed candidate exists
