# RP-608 · 3 Nov 2026 Full-Session Disposition

**Parent:** `RP-608-ECON-EDGE-DIAG-001`  
**Status:** SERIOUS TRADING FAILURE · RESEARCH HOLD ON RULES · AUTHORITY 0%  
**As-of:** ~18:31 IST · Pi journals  

## Verdict

Dismal full-session economic performance. Do **not** soften with architecture progress, win-rate optics, or 0% authority.

The midday narrative is **invalid** for the full session. Estimand A’s final closed book is deeply negative after exits and friction—not a small open-MTM hole around a green realized pocket.

## Closed-book snapshot (books never pooled)

| Book | Closed | Gross | Friction | Net |
| --- | ---: | ---: | ---: | ---: |
| NSE Estimand A | 189 | −₹17,498 | ₹28,139 | **−₹45,637** |
| NSE OF | 132 | −₹1,411 | ₹3,277 | **−₹4,688** |
| Crypto plan | 42 | −$45.49 | $50.41 | **−$95.90** |
| Crypto OF | 23 | +$3.89 | $13.91 | **−$10.02** |

- NSE closed net (separate books, same currency for sum only as exposure context): ≈ **−₹50,325**
- Crypto closed net (USD books only): ≈ **−$105.92**
- Currencies and strategies remain **separate**.

### Book notes

- **NSE A:** 67W / 122L · **0 open** at snapshot → −₹45,637 is **realized**.
- **NSE OF:** 25W / 107L · reverse-OF exits dominate (does **not** prove exit vs entry causation).
- **Crypto plan:** 8W / 34L · 28 stop-loss exits · gross loss **and** friction both contributed.
- **Crypto OF:** gross positive, net negative · friction exceeded gross by $10.02.

## What this changes

Broad failure across four independently maintained books. Warrants investigation of whether current signal + lifecycle have **positive net expectancy at all**.

## What must not be claimed yet

Not established from the aggregate snapshot:

- G3 Policy C caused NSE A losses.
- Reverse-OF caused NSE OF losses.
- Stops caused crypto-plan negative expectancy.
- Friction alone explains results.
- Any particular entry/exit change would have improved the day.

Those remain hypotheses for row-level reconstruction and counterfactuals.

## Operating posture

- Production rules **frozen**.
- Live authority **0%**.
- Treat as serious failure of current trading behaviour — **not** a successful research milestone.
- Next: reconcile final journal rows → mutually exclusive loss attribution (signal / exit / friction / concentration), open exposure separate, **no mechanism labels until supported**.
- Success bar: sustained, reproducible **net** expectancy after costs, with drawdown and concentration accounted for — not passing tests or a green pocket surrounded by losses.

## Step 1b status (accounting only)

Primary accounting attribution from the authoritative export is recorded at:

`artifacts/rp608_econ_edge_diag/accounting_attribution_2026-11-03.md`

- NSE A `GROSS_ADVERSE` **net −₹63,180.79** reconciles as gross **−₹51,760.28** + friction **₹11,420.51** (identity residual 0).
- Path / exit descriptors and concentration shares are attached as observational metadata.
- **Causal mechanism labels are not assigned.** Every row remains `mechanism_status=UNRESOLVED`.
