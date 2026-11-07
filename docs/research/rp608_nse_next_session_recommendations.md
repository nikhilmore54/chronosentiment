# RP-608-NSE-NEXT-SESSION-001 — Post-Close Next-Session Watchlist

**Authority:** 0.0%  
**Trigger:** Daemon `POST_CLOSE` (after 15:30 IST calendar close), once per session date  
**Artifact:** `live_capture/shadow_validation/nse/rp608_nse_next_session_recommendations.json`

## Purpose

After the continuous session (and 15:00–15:15 close window) ends, emit ranked **next-session intraday watches** from today’s completed 15m fetch history — so the Command Center can show tomorrow’s watchlist overnight without inventing overnight/CAS trades.

## Universe

- **Census:** 119 (RP-608-NSE-SHADOW-001 frozen list)
- **Watch ranking universe:** **102** RP-607 classified (C1–C4)
- **Excluded:** 17 `C_Quarantined (Insufficient Hist)` — never appear on the next-session board

## Ranking

For each **classified** symbol with ≥5 completed bars today:

- `traj15_eod` — same tanh family as live intraday entry (last ≤5 closes)
- `session_return_bps` — first→last session close
- `score = traj15 + 0.25 * (session_return_bps / 10000)`
- Keep if `traj15 ≥ 0.05` **or** session return ≥ +25 bps
- Top 20 by score (ties: symbol)

### Direction bias (structural)

The emitter is **long-only by construction** (`side = "LONG"`; positive traj / session-return filters).  
Absence of SHORT names on the board is therefore a **candidate-generation property**, not evidence that short-side setups were absent that day. Cross-session warmup research must label this bias until an observational SHORT watch track is separately authorized.

## Order-flow mark (confirm only)

Optional badge — does **not** change rank or authority:

- Source: last continuous-session L1 TOB qty from `data_quality.bid_qty` / `ask_qty` (latest evaluations), else daemon in-memory last eval
- `imbalance = (bid_qty − ask_qty) / (bid_qty + ask_qty)`
- Threshold: `0.30` (same family as crypto tick `MicroConfirm`)
- LONG watch marked `ORDER_FLOW_SUPPORTED` iff `imbalance ≥ +0.30` (and microprice ≥ mid when bid/ask prices are present)
- Otherwise **unmarked** (no negative badge)
- Fields: `order_flow_supported`, `order_flow_label`, `order_flow_imbalance`, `order_flow_source`; payload `n_order_flow_supported`

## Indicative structure levels (informational)

Each watch row may carry **calculated** entry-range / stop / target / R:R from
`RP608_DYNAMIC_OHLCV_STRUCTURE_V1` (same policy family as Estimand A entry-time bounds).

| Field | Meaning |
| --- | --- |
| `entry_low` / `entry_high` | Indicative acceptable entry band around last completed close |
| `stop_price` / `target_price` | Structural stop / target at the indicative reference |
| `risk_reward_display` | Gross reward:risk from reference → target/stop |
| `level_status` | `VALID` / `WAITING` / `INVALIDATED` / `STALE_SUPPRESSED` |

**Governance distinction:** these levels are **watchlist guidance only**.

- `levels_role = INDICATIVE_WATCHLIST_ONLY`
- `not_executed_entry = true` · `not_paper_trade_entry = true`
- Reference close ≠ authorized fill ≠ paper-trade entry price
- Existing recommendation / OF ±0.30 / Estimand A logic **unchanged**
- Live authority remains **0%**; no automatic orders

**Live behaviour (dashboard overlay):**

- Price events update `level_status` against the stamped entry band
- On the actionable next-session date after bar ≥5, completed 15m bars may refresh structure
- Stale L1 suppresses live validity (`STALE_SUPPRESSED`)
- Every level block carries `levels_calculated_at_utc` + provenance

## Governance

- Status: `WATCH_NEXT_SESSION` — **not** executable overnight or in CAS
- Becomes eligible for live MIS shadow only on `next_session_date` after bar ≥5, subject to live traj15 / L1 / entry cutoff (bar ≤19) and 15:00 square-off
- Deterministic: same bar history + same L1 → same list / marks / structure (given fixed `now_utc`)
- Aligns with chronosentiment-core: confirmatory feature only; multi-scenario evidence still required before any authority lift

## Ops

- Auto-emitted by `run_rp608_nse_shadow_daemon` on POST_CLOSE
- Manual: `python3 scripts/run_rp608_nse_shadow_daemon.py --emit-next-session-recs`
