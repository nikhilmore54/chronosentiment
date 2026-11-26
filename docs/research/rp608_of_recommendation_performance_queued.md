# RP-608-OF-REC-PERF-001 — OF Recommendation Performance (Queued)

**Disposition:** QUEUED · NO RULE CHANGE  
**Specification:** Accepted (authoritative scope) · **amended 2026-11-02** for NSE OF paper cohort  
**Report execution:** Pending — **no report produced yet**  
**Date locked:** 2026-11-02  
**Authority:** 0.0% (unchanged)

**Decision standard:** Establish whether OF confirmation adds measurable **net** economic value after costs, under comparable conditions.

Until the dedicated report is produced and reviewed, OF remains **observational support** — not an established trading advantage.

## Current Order Flow architecture (locked view)

| Capability | Crypto BTC/USDT | NSE |
| --- | --- | --- |
| OF inspection | Live | Live |
| ±0.30 side-agreement | Yes | Yes |
| Paper-trade lifecycle | Active (`RP-608-OF-PAPER-TRADE-001`) | Active (`RP-608-NSE-OF-PAPER-TRADE-001`) |
| Open/closed ledger | Yes | Yes (`nse/of_paper_ledger_v1/`) |
| Reverse-OF exit | Yes | Yes |
| Anti-churn | 5m hold + 30s opposite persist | 5m hold + 30s opposite persist |
| Forced square-off | N/A (24/7 crypto) | Session-end / bar-23 flat |
| Live order authority | 0.0% | 0.0% |

Shared paper-trade concept; different market-session and execution constraints. RP-608 rules frozen; depth telemetry observational.

## Distinctions — four non-combinable cohorts

Do **not** pool these processes. They are different observation and trade-generation paths:

| # | Cohort | What it is | Outcome surface |
| --- | --- | --- | --- |
| 1 | NSE live OF inspection recommendations | Per-eval L1 OF stamp on shadow daemon bars | Join to Estimand A **only when IDs allow**; stamp ≠ trade |
| 2 | NSE EOD watchlist OF recommendations | Post-close next-session badges (`latest_evaluations_l1`) | Separate from live inspection |
| 3 | NSE OF paper trades | Independent PAPER lifecycle (`RP-608-NSE-OF-PAPER-TRADE-001`) | `of_paper_ledger_v1` opens / outcomes |
| 4 | NSE Estimand A shadow trades | Arm A TP/SL/60m journal | `rp608_nse_trade_journal.jsonl` |
| — | Crypto OF paper trades | PAPER lifecycle (`RP-608-OF-PAPER-TRADE-001`) | Crypto overlay OF ledger (stratify by market) |
| — | NSE intrabar absorption telemetry | Observational only (`RP-608-NSE-OF-INTRABAR-ABSORPTION-001`) | Scoped; **do not pool** until amended — see [`rp608_nse_of_intrabar_absorption_scoped.md`](rp608_nse_of_intrabar_absorption_scoped.md) |
| — | NSE order-book intelligence | Displayed depth pressure/replenishment (`RP-608-NSE-OF-BOOK-INTELLIGENCE-001`) | Scoped; **do not pool** — see [`rp608_nse_of_book_intelligence_scoped.md`](rp608_nse_of_book_intelligence_scoped.md) |
| — | OFX order-flow response research | Aggression + price-response mechanisms (`RP-608-OFX`) | Research-only; **never** a fifth pooled book — see [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md) |

Also preserve:

| Layer | Role |
| --- | --- |
| OF recommendation | Stamp / badge that L1 OF support was observed |
| OF paper trade | Paper lifecycle when latched (crypto or NSE protocol) |
| Economic outcome | Realized shadow/paper P&L after costs — **not** open uPNL |

Do not conflate recommendation presence with trade existence, or open mark-to-market with realized expectancy.

**Critical integrity test (NSE OF paper):** a paper trade must close only on defined **reverse-OF** or **forced square-off** — never merely because a later intraday evaluation flips to `NO_TRADE`.

## Governance lock

| Item | Status |
| --- | --- |
| Specification | **Accepted** (+ NSE OF paper cohort amendment) |
| OF entry ±0.30 | **Frozen** |
| Reverse-flow exit | **Frozen** |
| Depth telemetry | Observational |
| RP-608 | Frozen |
| Report execution | **Pending** (after sufficient closed observations + validation) |
| Live authority | **0.0%** |

No entry gate, exit rule, sizing, or authority change may be inferred from partial results.

## Objective

Compare **OF-supported** vs **non-OF** under equivalent instruments, directions, and windows — **within each cohort separately**, then optionally contrast cohorts in a meta-section (never as a single pooled sample).

## Required metrics (per cohort: OF vs non-OF where applicable)

| Metric | Required |
| --- | --- |
| Recommendation / trade count | OF vs non-OF (or paper open count for ledger cohorts) |
| Win rate | **Closed / realized only** |
| Net P&L | Gross and after costs · realized |
| Profit factor | Realized |
| Holding duration | Average and distribution |
| Exit reasons | Reverse OF, session square-off, other applicable |
| Fees and spread | Absolute and bps (USDT or INR as market dictates) |
| Expectancy | Net per unit · realized |
| Max adverse / favourable excursion | Where available |
| Open-trade uPNL | **Separate table** — never mixed into closed win rate |
| No-trade / unmatched | **Retained** · separate cohort (selection-bias control) |

## Methodology (mandatory)

1. **No-trade recommendations stay in the dataset** (selection-bias control).
2. **Never pool** live inspection, EOD watchlist, NSE OF paper, Estimand A, or crypto OF paper into one sample.
3. **Open trades are not realized outcomes.**
4. Prefer unique IDs (`of_trade_id` / `trade_id`); composite joins must validate uniqueness or remain unmatched.
5. De-duplicate multiple observations of the same OF latch.
6. Report unmatched / missing outcomes explicitly.

## Surfaces in scope (when executed)

1. Crypto OF paper ledger (`RP-608-OF-PAPER-TRADE-001`).
2. NSE live OF inspection stamps on evals → Estimand A join (when IDs allow) — **cohort 1↔4 join study**, not a merge of generators.
3. NSE EOD next-session watchlist OF badges — **cohort 2 alone**.
4. **NSE OF paper ledger** (`RP-608-NSE-OF-PAPER-TRADE-001`, `nse/of_paper_ledger_v1/`) — **cohort 3 alone** (amendment).

## Current disposition — DOCUMENTATION LOCKED (2026-11-02)

| Area | Status |
| --- | --- |
| OF architecture | **Frozen** — no further feature expansion |
| Four NSE cohorts | Separately maintained (do not conflate) |
| Crypto OF paper | Separate, stratified cohort |
| NSE OF paper ledger | Awaiting operational validation |
| Exit integrity | Reverse OF / square-off only |
| `NO_TRADE` flip | Must **not** close OF paper trades |
| Performance report | Queued |
| Live authority | **0.0%** |

Freeze OF feature development. Concentrate on operational correctness and trustworthy paper-trade evidence. That is what makes the eventual OF performance report meaningful.

## Operational validation (NSE OF paper) — before report execution

Once the NSE cash session produces activity, validate **in this order**:

| # | Check | Pass criterion |
| --- | --- | --- |
| 1 | Lifecycle | Qualifying OF entry creates **exactly one** paper trade (`of_trade_id` unique; one-per-bar) |
| 2 | Accounting | Quantity, synthetic fill, fees, spread, and realized P&L reconcile to journal rows |
| 3 | Exit | Reverse OF closes only after **5m hold + 30s persist**; square-off also closes |
| 4 | State integrity | `NO_TRADE` evaluation transitions do **not** close open OF paper trades |
| 5 | Restart recovery | Open positions survive daemon/dashboard restarts without duplication or loss |

Only after these checks pass — and sufficient closed observations exist — may the dedicated OF vs non-OF analysis proceed.

## Out of scope until report reviewed

- Further OF feature expansion (gates, sizing experiments, new exit modes).
- Promoting OF to an entry qualification gate.
- Changing ±0.30, reverse-exit, or paper anti-churn from this analysis alone.
- Raising live order authority.
- Combining the four NSE/crypto generator cohorts into one expectancy number.

## Status

| Artifact | Exists? |
| --- | --- |
| Authoritative specification (this doc) | **Yes** · disposition locked |
| Dedicated OF vs non-OF performance report | **No** (pending validation + observations) |
| Rule / authority change | **No** |
| OF feature freeze | **Yes** |
