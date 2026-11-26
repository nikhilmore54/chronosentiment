# CS-P-006-P.H.3 — Decision Evidence Dashboard

**Document type:** Product capability protocol  
**Status:** Started — Observatory is an evidence dashboard; replay integrity ≠ strategy validation  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P, CS-P-006-P.H, CS-P-006-P.H.1, CS-P-006-P.H.2  
**Does not:** start C.3-G, run Search #3, retune C3-002, mutate the 14 August seals, reinterpret Replay v0, put mean / median / total V on a homepage, authorize real capital  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.

---

## What this freeze is

P.H demonstrated the evidence lifecycle:

```text
C3-002 → certified state at T → sealed action → 20 market sessions → outcome appended
```

That is a product milestone. It is not a reason to reopen research.

Research stays at C.3-F. C.3-G remains a question. The next work is the **Decision Evidence Dashboard** around this lifecycle.

---

## Wording correction

Do not say an unqualified **“not a backtest.”**

Historical replay already takes a historical timestamp, generates a decision as-of that timestamp, and later measures subsequent market data. That *is* a backtesting mechanism.

The precise claim is:

> **Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest.**

| Claim | Status |
|---|---|
| Replay integrity | **PASS** on v1 |
| Strategy validation | **Not done** |

```text
Replay integrity  ≠  strategy validation
```

Fourteen observations are evidence of the lifecycle and of the horizon contract. They are not a homepage performance number. Do not put +1.46% — or any mean / median / total V — on the ChronoSentiment homepage.

---

## Horizon is part of the contract

The same sealed decisions produced different observed values under Replay v0 (20 calendar days) and Replay v1 (20 market sessions). That is expected.

The observation horizon is part of the decision-observation contract. ChronoSentiment must make the contract explicit rather than hide it.

v0 remains archived. It is not reinterpreted as a 20-session result.

---

## What the dashboard shows

For each decision:

| Field | Rule |
|---|---|
| Instrument | Seven-name paper universe; IDEA and MAHABANK remain |
| Decision timestamp | Certified market session T |
| Certified TMV state | Trend / Momentum / Volatility at T |
| Policy ID / artifact | C3-002 / `5a43b9df…` |
| Action | Sealed; never rewritten after the outcome |
| Horizon | 20 market sessions |
| Observation close | 20th eligible market session after T |
| Status | OBSERVING / OUTCOME DUE / COMPLETED |
| Raw outcome | Append-only; hidden until close |
| Decision value | M.1 V(selected) |
| Alternative values | LONG / SHORT / NO TRADE |
| Audit trail | Created → sealed → observed without rewrite |

At cohort level, show **evidence structure**, not a performance scoreboard:

* decisions generated
* observing
* completed
* winners / losers among completed evidence
* LONG / SHORT distribution
* certified-state distribution
* decision-value sign counts
* instrument-level evidence
* time-period evidence

Aggregate performance statistics wait until there is enough completed prospective and historical evidence. They are not started here.

---

## Why IDEA and MAHABANK stay

The same certified-state framework produces different outcomes across instruments and across time. IDEA and MAHABANK make that visible. They are not extra stocks to hide, and they are not a claim that those names “always work.”

The Observatory preserves **instrument + state + action + timestamp + outcome**. It does not collapse the ledger into a single prediction-accuracy score.

A wrong observation — for example a Bullish / Negative SHORT that later has negative decision value — stays on the record. The state/action relationship is not rewritten.

```text
What did Coralys know?
        ↓
What did C3-002 decide?
        ↓
What happened afterward?
        ↓
What was the decision value?
```

---

## Three layers

```text
                 CORALYS
                    │
                    │ policy
                    ▼
             CHRONOSENTIMENT
                    │
          certified state at T
                    │
                    ▼
             SEALED DECISION
                    │
             ┌──────┴──────┐
             │             │
          decision       future
          immutable      unknown
             │             │
             └──────┬──────┘
                    │
             20 market sessions
                    │
                    ▼
             OBSERVATION
                    │
                    ▼
            DECISION VALUE
```

---

## What stays frozen

* C.3-G
* Search #3
* C3-002 retune
* Universe expansion
* Homepage performance aggregates
* Real capital
* The 14 August prospective seven
* Replay v0 files
