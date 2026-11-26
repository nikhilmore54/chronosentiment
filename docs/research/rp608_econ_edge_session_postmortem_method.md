# RP-608 Same-Day Economic Postmortem Method

**Parent:** `RP-608-ECON-EDGE-DIAG-001`  
**Status:** APPROVED METHOD · 2026-11-03  
**Authority:** 0.0% · production rules frozen during analysis  

## Purpose

Turn session losses into **evidence-backed failure attribution**, not speculative multi-cause storytelling.

## Four questions (answer separately)

1. **Signal quality** — Were entries directionally wrong, or did initially valid signals fail to follow through?
2. **Exit quality** — How much PnL was lost through reverse-OF churn, stops, and time-based exits?
3. **Friction** — What remains after brokerage, statutory costs, spread, and execution assumptions (no double-count)?
4. **Concentration** — Did few symbols, entry windows, or repeated LONG exposures dominate the loss?

## Non-attribution rule

Do **not** assign the same loss to multiple causes by default.

Example: a reverse-OF exit loss is **not** automatically an OF failure. Establish, in order:

1. Did the entry have measurable edge (path after entry)?
2. Did OF add incremental information vs unmatched recommendation?
3. Did the exit capture or destroy residual edge?

## Input required

Authoritative session export covering four separate books:

- NSE Estimand A
- NSE OF paper
- Crypto recommendation / trade-plan overlay
- Crypto OF paper

Each closed row: entry/exit events, fills/prices, costs, MAE/MFE when present.  
Open rows: marks distinct from realized.

### Parallel research line (not a fifth book)

`RP-608-OFX` may appear in the same **session report document** as an independent section (event metrics / Stage status). OFX results are **never** pooled into the four-book baseline totals and do not alter primary accounting or mechanism attribution. See `docs/research/rp608_ofx_performance_tracking_boundary.md`.

## Session report sections (baseline vs OFX)

1. Baseline performance — four books, separately  
2. Open-book status — unrealized only  
3. Baseline mechanism observations — descriptive until evidence supports attribution  
4. OFX research — independent (`OFX_NOT_RUN` if Stage 0 incomplete)  
5. Comparison — only when cohorts/accounting are comparable; else `NOT_COMPARABLE`  
6. Governance — rules frozen · Step 3+ held · Gate R unreached · authority 0%

## Output (ordered layers)

0. **W/L first (required lead diagnostic, every book separately)** — start every session with closed-trade wins / losses / breakeven and win rate per book (never pooled); then loss rate / max consecutive losses; W/L by exit type, symbol, and entry hour when available. Payoff, gross, friction, net, and path/MFE evidence **explain** the W/L result — they do not override or reorder ahead of it. Actionable-MFE grades are supporting attribution only. Raw favorable excursion is not an exit explanation unless it clears recorded friction and the declared meaningful-movement floor (hypothesis only).
1. **Primary accounting** — mutually exclusive per closed row from gross/net identity only:
   - `NET_POSITIVE` — net > 0
   - `NET_FLAT` — net == 0
   - `FRICTION_DOMINATED` — net < 0 and gross ≥ 0
   - `GROSS_ADVERSE` — net < 0 and gross < 0
   - `UNRESOLVED_PNL_FIELDS` — gross or net missing (retain explicitly)
2. **Path / exit descriptors (observational)** — MFE-based path / actionable-MFE grades when available; observed `exit_reason` as metadata only. These are **not** causal labels.
3. **Concentration** — distributional share metrics only; never the primary accounting label of a trade's loss dollars.
4. **Causal / economic mechanism labels** — only when evidence supports a mutually exclusive primary mechanism. Until then the row keeps `mechanism_status=UNRESOLVED` (or `NOT_APPLICABLE` for net-positive/flat).

### Mechanisms assignable from realized rows alone

| Label | Evidence gate | Typical confidence |
| --- | --- | --- |
| `FRICTION_DOMINANCE` | `FRICTION_DOMINATED` accounting (gross≥0, net<0) | HIGH |
| `ENTRY_PATH_NEVER_FAVORABLE` | `GROSS_ADVERSE` + never-favorable path (MFE bps or exec-price) | MEDIUM |

Everything else on losing rows stays `UNRESOLVED` until session-matched exit counterfactuals / matched-cohort tests exist. Observed `exit_reason` (including reverse-OF) is metadata, not a cause. Concentration stays distributional.

### Counterfactuals

Counterfactual exits must be stored separately from realized outcomes and must not be transferred across sessions. Phase-2 A–E PATH_ONLY for 2026-11-02 does **not** label 2026-11-03 rows.

Accounting partitions and mechanism labels must remain distinct. Do not propose rule changes from aggregates or accounting buckets alone.

### Step 2 artifact contract

Evidence table fields (per closed row): `trade_id`, source entry/exit events, observed facts (gross/friction/net, path, exit_reason), `primary_mechanism` only when supported, `confidence` / `evidence_basis`, `mechanism_status`, `unresolved_questions`, and a separate `counterfactual_exit` block.
