# RP-608-NSE-NEXT-SESSION-001 — Watchlist Levels Enhancement (Approved Spec)

**Status:** **APPROVED FUNCTIONAL SPECIFICATION** · 2026-11-02  
**Parent:** `RP-608-NSE-NEXT-SESSION-001`  
**Presentation / observational only**  
**Live order authority:** **0.0%**

This document is the binding implementation boundary for adding indicative entry range,
target, stop-loss, risk/reward, and level status to the Next Session · Intraday Watchlist.

---

## 1. Implementation boundary

1. Add entry range, target, stop-loss and risk/reward to each eligible watchlist row.
2. Reuse `RP608_DYNAMIC_OHLCV_STRUCTURE_V1` — **no new stop/target formula**.
3. Recalculate structural levels **only on completed 15-minute bars**.
4. Update entry-range validity on **incoming price events** (push path).
5. Suppress validity when market data is **stale** or **reconciliation is pending**.
6. Preserve calculation timestamp and source provenance on every level block.
7. Keep indicative levels **separate** from actual paper-trade entries.

---

## 2. Acceptance criteria

| Criterion | Required |
| --- | --- |
| Recommendation ranking / selection | **Unchanged** |
| OF ±0.30 | **Unchanged** |
| Estimand A | **Unchanged** |
| New stop/target formula | **Forbidden** |
| Polling-based market-data *evaluation* | **Forbidden** (UI may poll for display only) |
| Automatic execution | **Forbidden** |
| Live authority | **0%** |

---

## 3. Invalidate latch (mandatory safeguard)

An **INVALIDATED** entry range must **not** silently become **VALID** merely because
price later returns inside the prior band.

| Rule | Behaviour |
| --- | --- |
| Latch set | On first transition to `INVALIDATED` for the current `structure_generation` |
| While latched | Status may be `INVALIDATED`, `WAITING`, or `STALE_SUPPRESSED` — never `VALID` |
| Latch clear | Only when a **fresh structural evaluation** completes on a **new completed 15m bar** (new `levels_source_bar_timestamp_utc_ms` / bumped `structure_generation`) |

---

## 4. Field contract (per row)

| Field | Role |
| --- | --- |
| `entry_low` / `entry_high` | Indicative acceptable entry band |
| `stop_price` / `target_price` | Structural stop / target at indicative reference |
| `risk_reward_display` | Gross reward:risk text |
| `level_status` | `VALID` \| `WAITING` \| `INVALIDATED` \| `STALE_SUPPRESSED` |
| `invalidation_latched` | Boolean latch for current structure generation |
| `structure_generation` | Monotonic id of last structural evaluation |
| `levels_calculated_at_utc` | Structural calc timestamp |
| `levels_source_bar_timestamp_utc_ms` | Completed bar that sourced structure |
| `levels_role` | `INDICATIVE_WATCHLIST_ONLY` |
| `not_executed_entry` / `not_paper_trade_entry` | Always true |

---

## 5. Evaluation vs presentation

- **Evaluation (structure + validity state machine):** push-driven daemon / event path only.
  - Structure: completed 15m bar for watchlist symbols on the actionable next-session date.
  - Validity: price/L1 events against the frozen band + latch.
- **Presentation:** dashboard / Gate5 may poll `/api/state` to *display* the latest stamped
  levels and status. Presentation must **not** recompute structure on poll.

---

## 6. Governance

- Informational watchlist guidance only.
- Does not authorize MIS / paper / live entries.
- Indicative reference close ≠ paper-trade entry price ≠ authorized fill.
- Aligns with chronosentiment-core determinism and 0% authority.

---

## 7. Implementation mapping (in-repo)

| Boundary | Location |
| --- | --- |
| Structure + latch + status | `scripts/rp608_nse_next_session_recs.py` |
| Completed-bar structure + price validity push | `scripts/run_rp608_nse_shadow_daemon.py` (`_update_next_session_watch_levels`) |
| Mid-bar L1 validity (daemon event proxy; no structure) | `scripts/run_rp608_nse_shadow_daemon.py` (`_next_session_levels_midbar_poll`) |
| Presentation overlay only (no structure on poll) | `scripts/rp608_live_dashboard.py` (`PRESENTATION_OVERLAY_ONLY`) |
| Latch / presentation regression tests | `tests/test_rp608_nse_next_session_recs.py` |

This section records where the approved boundary is enforced. It is not a claim that a remote Pi host has been redeployed.
