# RP-610 Crypto Multi-Horizon Observation Protocol

**Protocol Reference:** `RP-610-CRYPTO-MULTI-HORIZON-001`  
**Companion (frozen, do not alter):** `RP-608` BTCUSDT 15m / Trade-Plan Overlay (≤60m)  
**Execution Authority:** Strictly Read-Only Observation (**Live Order Authority: 0.0%**)  
**Market:** Binance BTCUSDT (spot-sim paper book)  
**Date:** 2026-10-29  

Aligned with `.cursor/rules/chronosentiment-core.mdc` and the RP-609 NSE multi-horizon posture: observe first; no live orders; do not mutate the frozen intraday book.

---

## 1. Objective

Extend crypto shadow observation beyond the existing **≤60-minute** trade-plan overlay into retail-relevant longer holds:

| Horizon ID | Intent | Hold clock | Default review |
|---|---|---|---|
| `H0_INTRADAY_60M` | Existing RP-608 overlay (link only) | ≤ 60 minutes | Owned by RP-608 |
| `H1_MULTI_DAY` | Multi-day swing | **2–14 calendar days** | On exit / max days |
| `H2_MULTI_WEEK` | Multi-week position | **14–56 calendar days** (2–8 weeks) | Weekly mark + max days |
| `H3_LONG_TERM` | Long-term hold | **≥ 90 calendar days** | Quarterly thesis review |

RP-608 continues full-session / full-day intraday observation. RP-610 does **not** replace or filter H0.

---

## 2. Constraints (crypto retail-sim)

| Field | v1 rule |
|---|---|
| `product_type` | `SPOT_SIM` |
| `friction_model_id` | `CRYPTO_SPOT_RT_BPS_V1` (default **12.0 bps** round-trip) |
| `notional_usdt` | Fixed paper notional per open (default 1000) |
| `live_order_authority` | Always `0.0` |
| Overnight / weekend | Allowed (crypto continuous); gap risk recorded via marks |
| Funding / borrow | Out of scope in v1 (spot-sim only; no perpetual funding) |

---

## 3. Artifacts

Root: `artifacts/rp610_crypto_multi_horizon/journals/`

- `RP610_horizon_registry.json`
- `RP610_promotion_gates.json` (all CLOSED)
- `RP610_candidates.jsonl`
- `RP610_day_trades.jsonl` / `RP610_day_book.json` (H1)
- `RP610_week_trades.jsonl` / `RP610_week_book.json` (H2)
- `RP610_long_term_holds.jsonl` / `RP610_long_term_book.json` (H3)
- `RP610_daily_observation_log.jsonl`

---

## 4. Lifecycle (deterministic)

### H1 / H2
`CANDIDATE` → `OPEN` → daily `MARK` → optional `AMEND` → terminal `TARGET` | `INVALIDATION` | `MAX_HOLD_DAYS`

### H3
`THESIS_OPEN` → `HOLDING` → daily mark → `REVIEW_DUE` → `HOLD_CONTINUE` | `THESIS_INVALIDATED`

### Amendments (new information)
Journals are **append-only**. Never rewrite `ADMIT` / entry economics.

| Change | Event | Frozen | Mutable |
|---|---|---|---|
| Target / stop / thesis / max hold | `AMEND` (same `candidate_id`) | `entry_price`, `entry_ts`, `side`, `symbol`, `quantity` | `target_pct`, `stop_pct`, `hold_days_*`, `max_exit_ts`, `thesis` |
| Horizon change (e.g. H1→H2) | `SUPERSEDE` parent + `ADMIT` child | same entry fill on child via `parent_candidate_id` | `horizon_id` + horizon defaults |

Marks use last available BTCUSDT reference from RP-608 crypto decision cards / bar feed when present; otherwise explicit `--price`.

---

## 5. Operator entrypoints

```bash
python3 scripts/rp610_crypto_observe.py bootstrap
python3 scripts/rp610_crypto_observe.py day-open --side LONG --thesis multi_day_observe --price 68000
python3 scripts/rp610_crypto_observe.py week-open --side LONG --thesis multi_week_observe --price 68000
python3 scripts/rp610_crypto_observe.py long-open --side LONG --thesis long_term_observe --version v1 --price 68000
python3 scripts/rp610_crypto_observe.py eod --price 69000
python3 scripts/rp610_crypto_observe.py amend --candidate-id <id> --reason new_info --target-pct 0.12
python3 scripts/rp610_crypto_observe.py amend --candidate-id <id> --reason extend_swing --new-horizon H2_MULTI_WEEK --mark-price 69000
python3 scripts/rp610_crypto_observe.py report
```

Unit tests: `python3 -m unittest scripts.tests.test_rp610_crypto_multi_horizon`

---

## 6. Promotion

All gates `CLOSED` until multi-week observation evidence exists. No auto-promotion from H0 60m expectancy.
