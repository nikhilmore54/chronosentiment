# RP-608-NSE-OF-PAPER-TRADE-001 — NSE OF Paper Ledger

**Protocol:** `RP-608-NSE-OF-PAPER-TRADE-001`  
**Execution mode:** `PAPER` only  
**Live order authority:** **0.0%**  
**Sibling:** crypto [`rp608_of_paper_trade_observability.md`](rp608_of_paper_trade_observability.md) (`RP-608-OF-PAPER-TRADE-001`)

## Intent

Activate an Order-Flow **paper accounting ledger** for NSE equities — same confirmatory OF mark (±0.30 L1 TOB), independent of Estimand A / Arm A TP·SL·60m shadow trades.

## Isolation

- Does **not** change OF inspection stamps on evals / next-session watchlist.
- Does **not** change Arm A entry/exit or `rp608_nse_trade_journal.jsonl`.
- Does **not** raise live order authority above 0.0%.
- Depth-quality fields remain telemetry (`RP-608-OF-DEPTH-QUALITY-001`).
- **OF-REC-PERF cohort:** this ledger is a **distinct** performance cohort from live OF inspection, EOD watchlist OF, and Estimand A — see [`rp608_of_recommendation_performance_queued.md`](rp608_of_recommendation_performance_queued.md).
- **Exit integrity:** close only on reverse-OF (after anti-churn) or forced square-off — **never** because a later eval becomes `NO_TRADE`.
- **Intrabar absorption** (`RP-608-NSE-OF-INTRABAR-ABSORPTION-001`) and **order-book intelligence** (`RP-608-NSE-OF-BOOK-INTELLIGENCE-001`) are **separate observational streams** — see [`rp608_nse_of_intrabar_absorption_scoped.md`](rp608_nse_of_intrabar_absorption_scoped.md) and [`rp608_nse_of_book_intelligence_scoped.md`](rp608_nse_of_book_intelligence_scoped.md). Neither may open, close, or suppress this paper ledger.

## Lifecycle

1. **Open** when `decision=RECOMMEND_LONG` **and** L1 OF supports LONG (`|imb|≥0.30`, micro agree).
2. **Hold** with mid-bar L1 polls (~15s daemon loop) while awaiting next 15m close.
3. **Exit** on reverse OF (SHORT support) after **5m** min hold + **30s** opposite persist.
4. **Forced flat** at mandatory square-off bar / POST_CLOSE / session rollover.
5. Restart reconstructs opens from `of_paper_trades_open.json`; ledger outcomes win over ghosts.

## Journals

Under `live_capture/shadow_validation/nse/of_paper_ledger_v1/`:

| File | Role |
| --- | --- |
| `of_paper_trades_open.json` | Multi-symbol open map (≤1 per symbol) |
| `of_paper_trade_opens.jsonl` | Append-only opens |
| `of_trade_outcomes.jsonl` | Closed PAPER outcomes (INR P&amp;L + Indian friction) |
| `of_paper_state.json` | Summary / last suppressed / last closed |

## Surfaces

- Pi `/api/state` → `nse_operations.of_paper_ledger`
- NSE Operations card: open count, cumulative net ₹, protocol id

## Config defaults (PAPER)

Mirror crypto anti-churn; NSE share/INR clamps:

- `of_paper_min_hold_ms=300000`, `of_paper_opposite_persist_ms=30000`
- `of_paper_one_per_15m_bar=true`
- `of_paper_reject_thin_opposite=true`
- `of_paper_min_entry_signal_qty=10` (shares)
- `of_paper_min_same_side_notional_inr=5000`
- Participation 10% of bid qty, notional clamp ₹[2500, 50000]

## Disposition — feature freeze · evidence collection

**OF feature development is frozen.** Next work is operational validation on live cash-session activity, then (only if checks pass) the queued OF-REC-PERF report.

| # | Validation | Pass criterion |
| --- | --- | --- |
| 1 | Lifecycle | Qualifying OF entry → exactly one paper trade |
| 2 | Accounting | Qty / fill / fees / spread / P&L reconcile |
| 3 | Exit | Reverse OF only after 5m + 30s; square-off closes |
| 4 | State integrity | `NO_TRADE` flip does **not** close the paper trade |
| 5 | Restart recovery | Opens survive restart without dup/loss |

See [`rp608_of_recommendation_performance_queued.md`](rp608_of_recommendation_performance_queued.md) for cohort separation and report gate.

### Live-ledger observation rules (2026-11-02)

Early closed/open rows (e.g. n_closed=1) are **operational telemetry only** — far too small for performance conclusions. No rule change is justified by a first-session snapshot.

Preserve these states distinctly:

| State | Meaning |
| --- | --- |
| reverse-OF **armed** | Opposite flow seen; persist clock running — trade still **open** |
| reverse-OF **confirmed** | Persist + min-hold satisfied — close may execute |
| exit **executed & journaled** | Row in `of_trade_outcomes.jsonl` |
| realized **net after friction** | Only journaled closes count as outcomes |

Also:

- Quantity / thin-opposite **suppression** = paper eligibility deny; **OF ±0.30 mark unchanged** (not an OF signal failure).
- This ledger stays **entirely separate** from Estimand A and from [`rp608_nse_exit_path_falsification_scoped.md`](rp608_nse_exit_path_falsification_scoped.md) — OF paper outcomes must **not** enter the Arm A exit-path report.
