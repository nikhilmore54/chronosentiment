# RP-608-OF-PAPER-TRADE-001 — OF Paper Trade Observability

**Protocol:** `RP-608-OF-PAPER-TRADE-001` (crypto)  
**NSE sibling:** [`rp608_nse_of_paper_trade.md`](rp608_nse_of_paper_trade.md) (`RP-608-NSE-OF-PAPER-TRADE-001`)  
**Execution mode:** `PAPER` only  
**Live order authority:** **0.0%**  
**Intent:** Accounting / observability over the existing Order-Flow latch lifecycle. Not a new strategy.

## Isolation

- Does **not** change OF entry (Policy B/C + same-direction L1 ±0.30).
- Does **not** change OF exit (opposite-side OF confirm only; neutral does not close).
- Trade-plan TP / SL / 60m expiry do **not** close OF paper trades.
- Independent of RP-611 Volume Absorption (no shared candidates / observers / journals).

## Lifecycle

1. Opening OF confirmation → unique `of_trade_id` + paper open fill + open journal.
2. While open → mark-to-market (executable side) with explicit cost breakdown.
3. Opposite OF confirmation → `signal_exit_*` then synthetic close → `paper_close_*` (+ lag).
4. Restart → reconstruct single open trade from `of_paper_trades_open.json` / overlay state; never duplicate id.
5. **Ledger wins:** if `of_trade_id` is already in `of_trade_outcomes.jsonl`, it must not appear as OPEN
   (`reconcile_of_paper_already_closed`). Prevents dual-writer ghosts where chips show n=1 closed while
   OPEN still mirrors the same trade.


### Anti-churn (PAPER)

L1 imbalance can cross ±0.30 within seconds. Without guards that produces fee-dominated
round-trips that look like “selling without holding.”

Frozen defaults in overlay `CONFIG` (PAPER anti-churn / open quality — Stage-1 ledger):

- `of_paper_min_hold_ms` = 300_000 — reverse OF cannot close before **5m** of paper hold
- `of_paper_opposite_persist_ms` = 30_000 — opposite OF must persist continuously for **30s**
- `of_paper_entry_persist_ms` = 30_000 — same-side OF must persist continuously for **30s** before paper open (WS continuation via `continue_of_paper_entry_persist`)
- `of_paper_one_per_15m_bar` = true — at most one paper open per 15m bar
- `of_paper_reject_thin_opposite` = true — PAPER open suppressed on hollow opposite (OF ±0.30 mark unchanged)
- `of_paper_min_entry_signal_qty` = 0.25 — PAPER open needs minimum same-side L1 qty
- `of_paper_min_same_side_notional_usdt` = 5000 — PAPER open needs minimum same-side notional
- `of_paper_min_combined_touch_notional_usdt` = 0 — **not raised** (2026-11-21 QUALITY vs WEAK diag: depth floors did not separate; entry persist is the quality lever)
- `RP608_OF_PAPER_QUALITY_GATE` = `1` suppress open / `0` stamp-only (default **1** on Pi crypto OF)

Open/suppress stamp `of_entry_quality` (`RP-608-OF-PAPER-ENTRY-QUALITY-001`): `{persist_ok, depth_ok, reason}` — paper booking only; L1 ±0.30 classify unchanged.

Close of a LONG is a **paper sell-to-flat**, not a naked short. UI must not imply
inventory that was never opened.

Ghost reconcile must **hydrate** `last_closed` from `of_trade_outcomes.jsonl` (never leave
`RECONCILED_ALREADY_CLOSED` stubs without exit imb/candles). Decision-card OF ENTRY/EXIT
badges are **event-precise** (decision_event_idx or ±5s mark_time) — never whole-bar paint.

### Paper size (entry OF candle signal → exit fill must match)

OF paper qty is **not** a fixed $1000 notional. At open on the OF entry candle:

- `entry_signal_qty` = entry-side L1 depth (`best_bid_qty` for LONG, `best_ask_qty` for SHORT).
- Paper qty = participation **10%** of that signal, hard-clamped to **[5%, 15%]**, then notional
  clamped to **[25, 500] USDT** (missing signal → **100 USDT** fallback).
- Stamped on `entry_candle.signal_qty` / `entry_candle.paper_quantity`.

### Depth-quality telemetry (RP-608-OF-DEPTH-QUALITY-001)

**Disposition: observe only — does not change frozen OF ±0.30 entry or reverse-flow exit.**

Journaled on every paper OF open and reverse-flow exit (`depth_quality` / `depth_quality_entry` /
`depth_quality_exit`):

| Field | Role |
| --- | --- |
| `l1_imbalance` | Relative bid/ask dominance |
| `combined_touch_qty` / `combined_touch_notional_usdt` | Absolute displayed liquidity (**primary**) |
| `same_side_touch_*` | Liquidity supporting the directional signal |
| `opposing_side_touch_*` | Whether extreme imb is an empty opposite |
| `thin_opposite_diagnostic` | Flag only (opp ≪ same-side); **not a gate** |
| `signal_persistence_hold_seconds` / `opposite_persist_ms_at_exit` | Persistence at exit |

Next steps (not implemented): retrospective depth buckets → outcomes; then a **separately
versioned** experimental min-depth entry policy if evidence supports it.

At reverse-OF exit:

- `exit_signal_qty` = opposite-side L1 on the exit candle (LONG→askQty, SHORT→bidQty) — **recorded only**.
- `exit_quantity` / `simulated_quantity` **must equal** `entry_paper_quantity` from the entry candle
  (`qty_matched_entry_to_exit=true`). Never resize the close from exit-book depth.

CONFIG keys: `of_paper_participation_pct`, `_min`, `_max`, `of_paper_min_notional_usdt`,
`of_paper_max_notional_usdt`, `of_paper_fallback_notional_usdt`.

## Cost model (v1, deterministic)

- **Spread:** half-spread × 2 legs on notional (entry + exit bid/ask crossing).
- **Fees:** 6 bps per leg on entry notional and exit notional (`taker_fee_bps` + allocation from existing 12 bps RT convention).
- **Slippage:** modeled field always present; v1 `slippage_model: none` → `slippage_usdt = 0.0`.
- **Friction:** `spread_cost_usdt + fees_usdt + slippage_usdt`.

## Journals

Under trade-plan overlay dir (`…/trade_plan_overlay_v1/`):

- `of_paper_trades_open.json` — at most one open paper trade (restart source of truth).
- `of_paper_trade_opens.jsonl` — append-only open events.
- `of_trade_outcomes.jsonl` — closed outcomes (schema extended; older rows remain readable).

## Surfaces

Pi `/api/state`: `of_shadow_trade`, `of_trade_pnl` (both `execution_mode: PAPER`, authority `0.0`).  
Gate5 Operator maps the same fields (presentation only).
