# CS-P-006-P — Decision Observatory

**Document type:** Product protocol  
**Status:** Historical 91/91 lifecycle PASS; P.E.2.H PASS; P.E.2 live AWAITING_NEXT_SESSION; CS-P-007 specified not run; P.E.3 waits; C.3-G remains a question  
**Date:** 2026-08-15  
**Parent:** CS-P-006, CS-P-006-A, CS-P-006-C.3-F, CS-P-003  
**Does not:** run Search #3, start a C.3-G experiment, run CS-P-007, add indicators, implement a regime detector, retune Search #2, promote a strategy, freeze Decision Engine v1.0, authorize real capital, invent confidence percentages, cherry-pick winning outcomes, sell Coralys/MOGA to the customer  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision; no invented methodology.

---

## What this freeze is

The CS-P-006 research loop **stops at C.3-F**. C.3-G remains an unanswered question. This document opens the **product layer** around what ChronoSentiment already does.

```text
C.3-F frozen diagnostic
C.3-G question only
        │
        ▼
CS-P-006-P Decision Observatory     ← this document
        │
        ▼
P.1 policy registry                 (started)
        │
        ▼
P.3–P.6 sealed-then-measured slice  (started)
        │
        ▼
P.7 four-screen Observatory         (started)
        │
        ▼
Prospective C3-002 paper clock      (started — OBSERVING, no outcomes at T)
        │
        ▼
Maturity countdown                  (started — no early peek)
        │
        ▼
        Historical Observatory Replay       (achieved — P.H; replay integrity PASS)
        │
        ▼
Decision Evidence Engine            (achieved — P.H.1 / P.H.2; 20 market sessions)
        │
        ▼
Decision Evidence Dashboard         (started — P.H.3)
        │
        ▼
Targeted Decision Execution         (started — P.E; target sealed at T)
        │
        ✕  automatic price attach not started
        ✕  P.8–P.10 not started
        ✕  no Search #3
        ✕  no C.3-G experiment
        ✕  not CS-P-003 validation
        ✕  universe not expanded
```

The product question is no longer “can we make Search #2 better?”

> **Can a user derive value from seeing, understanding, recording, and evaluating these decisions?**

---

## Proposition (customer language)

Not: “Our AI predicts stocks.”

> **ChronoSentiment identifies decision opportunities from the information available at the time, records the decision, and measures the subsequent outcome without hindsight contamination.**

Coralys is the technology underneath. The customer sees ChronoSentiment.

> **ChronoSentiment is a temporal decision-evidence system that preserves what was knowable, what was decided, what execution intent was sealed, and what subsequently happened.**

That identity is the temporal boundary — not “AI trading,” and not sentiment analysis alone.

> **ChronoSentiment preserves the temporal integrity of a decision: Coralys interprets the certified state, the decision and execution intent are sealed at T, and the Observatory records the evidence that emerges afterward.**

A strategy claim can emerge only from accumulated prospective evidence, not from marketing.

### Three questions (do not collapse)

| # | Question | Where it lives | Status |
|---|---|---|---|
| 1 | Does Coralys understand the state? | C.3 / certified TMV at T | Research under C.3-F (frozen) |
| 2 | Does ChronoSentiment make a reproducible decision? | C3-002: State(T) + Policy → Direction | Integrity demonstrated |
| 3 | Can Coralys determine *how* that decision should be executed? | P.E.3: state + direction + Coralys artifact → Execution Intent | **Not performed** |

The number in (3) must be produced **before** the future path is known. That experiment has not been run. P.E.3.A freezes the artifact contract first. No target algorithm in this freeze.

```text
                    CORALYS
                       │
                       ▼
              Certified State at T
                       │
                       ▼
                 C3-002
              Decision sealed
                       │
             ┌─────────┴─────────┐
             │                   │
             ▼                   ▼
        Direction          Execution Intent
             │                   │
             │              P.E.2: +5% control
             │              P.E.3: Coralys (not started)
             │                   │
             └─────────┬─────────┘
                       ▼
                  FUTURE TIME
                       │
                       ▼
               Observatory
                       │
                       ▼
             Evidence / Outcome
```

### Ontology

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

The Coralys target is an execution hypothesis generated from the state. It is **not** evidence.

### Temporal boundaries

| Layer | Owner | Records |
|---|---|---|
| Intelligence | Coralys | What can be inferred from the certified state. C3-002 is direction only (`5a43b9df…`). |
| Decision | ChronoSentiment | Direction, certified state, policy artifact, timestamp — sealed at T. |
| Execution Intent | Sealed at T | Target, maximum hold, trigger semantics, contract/artifact — not part of C3-002. Not evidence. |
| Observation | Observatory | What happened afterward. Append-only. |
| Evidence | Measured record | TARGET, HORIZON, trigger audit, realized V. |

POLICY = immutable. DECISION = immutable. EXECUTION INTENT = immutable. OBSERVATION = append-only. EVIDENCE = the measured record after observation.

OPEN / OBSERVING = outcome not yet observed. OUTCOME DUE = window closed, not yet appended. COMPLETED / OBSERVED = outcome known.

**No early peek. No retrospective edits.** Decisions are sealed at decision time. Outcomes become visible only when their observation window closes.

### Maturity path

```text
OBSERVING
   ↓
maturity countdown
   ↓
OUTCOME DUE
   ↓
append observation
   ↓
decision value
   ↓
historical record
```

Do **not** look at 20D prices before the window closes. Do **not** retune C3-002 if all seven win or all seven lose. Do **not** treat the first tick as a statistical conclusion. Do **not** add symbols. IDEA and MAHABANK stay; they already exposed heterogeneous behaviour.

First prospective due date: **3 Sep 2026 03:45 UTC** (14 Aug 2026 + 20D).

---

## What is frozen underneath (not reopened)

| Known from C.3-F | Still unknown (C.3-G) |
|---|---|
| TMV is not informationally empty | Whether persistence/failure can be predicted from TMV at T |
| Search #2 discovered a different decision surface | Whether more certified information is required |
| Bullish ∧ Negative → SHORT is the strongest observed persistence | Whether the Bearish ∧ Negative reversal is regime or sampling |
| Bearish ∧ Negative → LONG was rational on development/selection and reversed on holdout | Whether a persistence detector would generalize |
| Historical value does not imply stationary future value | Whether a detector would improve decision value enough to enter Coralys |
| Continuous V is preferable to an accuracy band at this stage | — |

Observing a reversal does **not** require a regime detector. C.3-G must not become Search #3 in disguise.

---

## Frozen artifacts

| Artifact | Product label | Role |
|---|---|---|
| `9a887827…` | Search #1 control | Research only |
| `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121` | **ChronoSentiment Research Policy — Candidate C3-002** | Paper decisions only |
| C.3-F | Frozen diagnostic | Research / methodology surface |
| C.3-G | Research question | Quarantined |

Not Strategy v2. Not a proven AI strategy. Not Decision Engine v1.0. Not production capital.

```text
Frozen Search #2
       │
       ▼
Candidate C3-002
       │
       ▼
Prospective paper decisions
       │
       ▼
Outcome ledger
       │
       ▼
Product evidence
```

---

## Product loop

```text
         CERTIFIED STATE AT T
                 │
                 ▼
        ChronoSentiment decide
                 │
        LONG / SHORT / NO_TRADE
                 │
                 ▼
        Immutable decision record
                 │
            wait / observe
                 │
                 ▼
        Outcome measurement
                 │
                 ▼
        Decision value vs alternatives
```

Display **all** certified decisions, not only winners. No “outcomes that you can hope for.” No invented probability.

---

## Four product screens (P.7)

The primary object is the **Decision**, not the stock, indicator, strategy, or report.

| Screen | Customer question | Contents |
|---|---|---|
| Observatory | What has ChronoSentiment decided? | Counts, action mix, observed / observing, instrument heterogeneity |
| Decision Feed | What is the chronological stream? | Every certified paper decision, winners and losers |
| Decision Detail | Why this decision, and what happened? | ID, state, action, policy, horizon, outcome, decision value, audit |
| Policy / Provenance | What artifact produced these? | C3-002, `5a43b9df…`, paper-only, immutable |

This is an **evidence dashboard**, not a performance dashboard. Do not add a P&L scoreboard while the 14 August cohort is still OBSERVING. Mean / median / total V are not homepage metrics.

| Product surface | Status |
|---|---|
| Live / open decision feed | Started |
| Decision detail | Started |
| Observation countdown + close time | Started |
| Completed decision history | Started — historical 91, winners and losers |
| Decision-value analytics | Not started — after observations mature |
| Audit trail | Started |
| Policy registry | Started — C3-002 only |
| Prospective portfolio view | Not started — paper-only when opened |

The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T. No C.3-G. No Search #3. No universe expansion.

```text
14-Aug cohort
Decision only
7 OBSERVING
No execution intent
```

```text
Next eligible cohort
Decision + Execution Intent
P.E.2 control
```

A separate Historical Observatory Replay (CS-P-006-P.H) runs the **same** decide path against a historical clock. It does not rewrite the live seven. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation.

Customer-facing lifecycle:

```text
SEALED → OBSERVING → OUTCOME DUE → OBSERVED
```

Internal fields stay on the record, not in the headline:

```text
sealed_status = OPEN
observation_status = COMPLETED
```

Do not show `sealed status OPEN · derived COMPLETED` to a customer.

IDEA and MAHABANK remain visible. Heterogeneous instruments are part of the product: ChronoSentiment does not assume one name behaves like another.

Most customers never see C.3-F. Policy / Provenance is the customer-safe provenance surface. Research methodology stays underneath.

Decision cards show alternatives (LONG / SHORT / NO_TRADE) and the sealed artifact. Outcome cards show V(selected) and V(alternatives), and state that the decision was sealed before the outcome was known.

---

## Sequence

| Step | ID | Purpose | This document |
|---|---|---|---|
| P.1 | Policy registry | Bind Candidate C3-002 to `5a43b9df…`; paper-only | **Started** |
| P.2 | Certified state engine | Existing TMV strings consumed as certified state | **Consumed** — no new encoding |
| P.3 | Decision generator | Timestamped paper decide from C3-002 | **Started** — one slice with P.4–P.6 |
| P.4 | Immutable decision ledger | Record cannot change after seal | **Started** |
| P.5 | Observation | Append-only outcome; CS-P-003 remains independent | **Started** — historical 91/91; prospective not yet observed |
| P.6 | Outcome calculator | M.1 V on the observation layer | **Started** — historical only |
| P.7 | Observatory product screens | Decision object; four screens; OBSERVED / OBSERVING | **Started** |
| P.5b | Prospective C3-002 | Same pipeline on latest session ≤ now; no outcome at T | **Started** — paper OBSERVING |
| P.5c | Maturity countdown | OBSERVING → OUTCOME DUE; refuse early peek | **Started** — price attach not started |
| P.5d / P.H | Historical Observatory Replay | Same engine on a closed historical clock; no lookahead | **Achieved** — replay integrity PASS; not a statistical strategy backtest |
| P.H.1 | Decision Evidence Engine | Replay v0 20 calendar days archived; integrity ≠ strategy validation | **Achieved** |
| P.H.2 | 20 market-session horizon | Replay v1; v0 not reinterpreted; C3-002 unchanged | **Achieved** |
| P.H.3 | Decision Evidence Dashboard | Per-decision + cohort evidence; no homepage performance aggregates | **Started** |
| P.E | Targeted Decision Execution | C3-002 direction only; Execution Contract v0 owns target_pct; not P.7 | **Started** |
| P.E.1 | Execution Evidence Surface | Decision / Execution / Evidence layers; TARGET and HORIZON both shown | **Frozen** |
| P.E.2 | Live Execution Observation | Prospective lifecycle with fixed Execution Contract v0; not a 5% quality test | **Closed spec / AWAITING_NEXT_SESSION** |
| P.E.2.H | Historical P.E.2 lifecycle validation | Time-machine of the frozen P.E.2 contract; live cohort untouched | **PASS** |
| P.E.3 | Coralys Target Discovery | Execution parameters from state at T; P.E.2 is the control | **Specified — not started** |
| P.E.3.A | Coralys Target Artifact | Contract for a frozen generator/output; no algorithm | **Specified — contract only; waits for CS-P-007** |
| CS-P-007 | Statistical Strategy Validation | Frozen C3-002 + Execution Contract v0 on an untouched confirmatory sample | **Specified — not run** |
| P.8 | Basket generator | Same-T paper basket; no rebalancing | Not started |
| P.9 | Decision Brief | Daily record, not a prediction tip | Not started |
| P.10 | Shareable decision page | Public proof of sealed-then-measured | Not started |

P.8–P.10 wait. **CS-P-007 is the next research priority** (specified, not run): whether frozen C3-002 plus the frozen +5% / 20-session contract has statistically meaningful decision value on an untouched sample. P.E.3 waits for that confirmatory gate. The Observatory homepage remains an evidence surface, not a performance dashboard. P.E.1 is frozen. P.E.2 specification is closed; live demonstration remains `AWAITING_NEXT_SESSION`. **P.E.2.H historical lifecycle validation — PASS.** P.E.3.A is the artifact contract only. Decision and execution intent are separate seals at T. The Coralys target is not evidence. Baskets do not rebalance. Briefs do not cherry-pick. Real capital is last.

---

## First acceptance test

> Can ChronoSentiment generate a timestamped paper decision from the frozen policy, preserve the exact state and policy artifact used, and later attach the realized outcome without changing the original decision?

If yes, the core product exists. P.1 registers the policy. P.3–P.6 implement this as one path:

```text
C3-002 → Decision → Seal → Observe → Outcome → Measure
```

The sealed decision object is immutable and does **not** contain `future_return`, `outcome`, `regret`, `evaluation_score`, or `confidence`. Observation is append-once. `sealed_status` stays `OPEN`. `observation_status = COMPLETED` lives on the observation. Customer status is OBSERVING / OBSERVED.

Sidecar: `product_validation/CS-P-006/observatory/` — historical ledger. `observatory/prospective/` — live-clock paper seals. Not a profitability claim. Not CS-P-003 validation.

### Historical path — what the HTML established

**PASS — 91 sealed / 91 observed.** Every evaluation-slice decision travelled through certified state → policy → decision → seal → wait → outcome → V(action).

The sealed record contains instrument, timestamp, action, certified TMV, policy ID, artifact hash, horizon, engine version, and `sealed_status`. It does **not** contain future return, outcome, regret, evaluation score, or confidence derived from the future.

SHORT is scored as V(action), not the stock return: HDFCBANK SHORT +0.62% raw → −0.62% decision value; −7.36% raw → +7.36% decision value. Winners and losers are both retained (HDFCBANK −16.49%, IDEA −30.82%, INFY +15.58%). Action mix on this slice was 75 LONG / 16 SHORT.

Mean decision value was **−0.296%**. That is not a profitability claim. The acceptance criterion was decision integrity, not whether C3-002 makes money.

> ChronoSentiment can create an auditable, immutable paper decision from a certified state, seal it before the future is known, and later attach and evaluate the realized outcome without rewriting the original decision — 91 times on the historical evaluation slice.

The HTML does **not** establish that C3-002 is profitable, has predictive alpha, works prospectively, beats a benchmark, that 20D is optimal, that real capital is authorized, that 91 rows are statistically sufficient, or that CS-P-003 has been passed.

### Prospective C3-002 — the next evidence gate

```text
CURRENT MARKET DATA
        ↓
CERTIFIED TMV STATE (bars ≤ T)
        ↓
C3-002
        ↓
PAPER DECISION
        ↓
SEAL
        ↓
OBSERVING
```

The historical experiment answered: can we preserve decision integrity? **Yes.**

The prospective experiment asks: what happens when the system makes decisions on the live clock and nobody knows the outcome?

Same frozen artifact. Same seven-name universe, including IDEA and MAHABANK. No Search #3. No C.3-G. No retune. Outcomes stay off the decision object until a later 20D attachment.

First prospective tick (latest session ≤ 2026-08-15): **14 Aug 2026 03:45 UTC**, 7 sealed, 0 observed, 5 LONG / 2 SHORT. HDFCBANK LONG, ICICIBANK SHORT, INFY LONG, RELIANCE SHORT, TCS LONG, IDEA LONG, MAHABANK LONG. Status OBSERVING. Not a result.

---

## Product metrics (secondary to the ledger)

Headline is **decision value**, not accuracy. No fake confidence.

When aggregates appear they are counts and continuous V: decisions recorded, completed, mean / median V, action mix. Unique-best and regret stay on Surface 3.

---

## What stays frozen

* C.3-G experiment
* Search #3
* CS-P-007 run (specified only)
* P.E.3 target algorithm / ATR→% / target-range search
* Additional indicators
* Regime detector
* Volatility-encoding research
* Policy retune / further MOGA
* Real capital
* Decision Engine v1.0

Engine version remains **`unfrozen-dev`**. CS-P-003 remains the confirmation clock. CS-P-006-A remains the consumption contract.
