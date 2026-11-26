# RP-608 NSE Estimand A — Trade Pattern Prediction v1

**Protocol:** `RP-608-NSE-A-TRADE-PATTERN-V1`  
**Date:** 2026-11-20  
**Authority:** 0.0% (adapter observation)  
**Book:** NSE Estimand A only (never pooled)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc`, Gate 2 amendment v2 (`rp608_gate2_amendment_v2.md`)

## Intent

Two uses of the same past-trade memory — **not** scrip ban lists:

1. **ENTRY (Gate 2E):** match live decision-time Coralys + **pre-entry candle path** to historical cohorts → predict ALLOW / STANDASIDE / ABSTAIN / ABSTAIN_NEGATIVE_EXPECTANCY from W/L (payoff+net disclosed). NEUTRAL → ALLOW only when train **net_mean > 0** (crypto Gate2E parity, 2026-11-23).
2. **EXIT (Gate 2X observational):** match **post-entry candle path** (where trades historically reversed after green MFE) → stamp HOLD / EXIT_REVERSE / ABSTAIN. Frozen stops/targets/holds unchanged unless Gate 5; optional paper-only `RP608_NSE_PATTERN_EXIT_GATE=1`.

## Non-goals

- No symbol / sector / “BAN these 13 names” rules.
- No OF ±0.30, traj15≥0.05, stop/target/hold changes from this memo alone.
- Raw MFE is not an entry cause; post-entry path is exit diagnostic / Gate 2X only.
- Books never pooled.

## Pattern features

### Entry (decision-time only)

| Axis | Source |
| --- | --- |
| `session_phase` | Coralys ToA (OPEN…LATE) |
| `traj15_bucket` | decision traj15 |
| `preopen_status` / `gap_bucket` / jitter / turbulent / trend_aligned | Coralys / gap |
| `prior_support` | next-session watch |
| `same_session_prior_stop_in_similar_context` | ToA\|traj family stop earlier today (**not** symbol) |
| `bar_tertile` | entry bar index |
| `candle_path3` / `signal_body` / `runup_3b` / `vol_surge` | last ≤3 completed 15m candles before entry (`vol_surge` = signal-bar volume vs mean of prior 5) |

**Excluded from key:** `symbol`, tradingsymbol, sector.

### Exit (path after entry)

| Axis | Meaning |
| --- | --- |
| `exit_path3` | first ≤3 post-entry candle directions |
| `mfe_bucket` | peak favorable excursion class |
| `first_reverse` | first strong down candle after green vs entry |
| `exit_vol_surge` | volume on reverse bar (else latest path bar) vs mean of prior 5 (`LOW`/`MED`/`HIGH`/`UNK`) |

Library schema: `nse_a_trade_pattern_library_v2` (remine required after volume-on-exit). Candle OHLCV only — not OFX tick aggression (`.cursor/rules/rp608-ofx.mdc`).

## Mining

```bash
python3 scripts/rp608_nse_trade_pattern_library.py \
  --journal live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl \
  --ohlcv-dir intraday_capture/yahoo_cache_15m \
  --output live_capture/shadow_validation/nse/rp608_nse_a_trade_pattern_library.json
```

- Chronological split: train on older sessions; last 2 sessions holdout.
- Entry ADVERSE when train **n≥30** and **WR≤35%** (payoff+net disclosed).
- Exit REVERSE_STANDASIDE_HOLD when reverse-bearing cohort n≥20 and WR≤40%.
- **Candle path join:** prefers 15m OHLCV for `candle_path3` / `exit_path3`. If cache sessions do not overlap the journal (e.g. yahoo_cache ends before live journal days), exit cohorts still mine from journal `mfe_bps` + `exit_reason` (`REV_AFTER_GREEN` / `REV_NO_MFE`). Live daemon uses in-session `price_history` candles for both entry and exit stamps.
- Refresh `intraday_capture/yahoo_cache_15m` through current sessions to densify candle-path keys offline.

## Live wiring

| Env | Default / live | Effect |
| --- | --- | --- |
| `RP608_NSE_PATTERN_GATE` | **`allow_only`** (systemd 2026-11-23) | PRIMARY book Gate2E: `0`/`off` stamp only · `1`/`adverse` · **`allow_only`/`2` block unless ALLOW** |
| `RP608_NSE_PATTERN_GATE_BASELINE` | **`0`** | BASELINE twin ungated control (crypto parity) |
| `RP608_NSE_PATTERN_EXIT_GATE` | `0` | When `1`, reverse exit match may close paper with `PATTERN_REVERSE` |

**Dual books (crypto-style trade + baseline):**

| Book | Dir / journal | Pattern axes | Gate |
| --- | --- | --- | --- |
| `NSE_ESTIMAND_A` (PRIMARY) | `…/nse/rp608_nse_trade_journal.jsonl` | volume axes | `allow_only` |
| `NSE_ESTIMAND_A_BASELINE_PARALLEL` | `…/nse/estimand_a_baseline_parallel_v1/rp608_nse_trade_journal.jsonl` | pre-volume key | `0` (ungated) |

Same traj15 / Coralys / OF±0.30 / stops / TICK_PATH fills; independent active maps + journals; **never pooled**. Gate G4 quote freshness is vs **wall clock** (not bar open).

**Gate2E selection:** `NEUTRAL` → `ALLOW` only if train `net_mean > 0`; else `ABSTAIN_NEGATIVE_EXPECTANCY` (fail-closed under `allow_only`). No OF ±0.30 / traj / stop/target change (Gate 5).

Stamps: `trade_pattern_match` on evaluations / opens; `trade_pattern_exit` on open manages; nested `baseline_parallel` on PRIMARY stamps.

Coralys V6 ToA/jitter/trend + missing-candle fetch **kept**. Per-symbol session re-entry hardblocks **removed** (churn is a pattern feature).

## Gate status

- Gate 2E live: PRIMARY `allow_only` + net_mean>0 ALLOW; BASELINE ungated twin — thinning expected; ADVANCE still needs holdout W/L+payoff+net **per book**.
- Gate 2X: exit reverse stamps are observational until a sealed exit-geometry review + Gate 5.
- Coverage floor for advance: per Gate 2 amendment v2 (≥ min(40% book, n≥80) on holdout preferred cohort).

## Code

- `scripts/rp608_nse_trade_pattern_library.py`
- `scripts/rp608_nse_trade_pattern_match.py` (`entry_gate_mode_baseline`)
- Daemon: `scripts/run_rp608_nse_shadow_daemon.py` (`active_trades` + `active_trades_baseline`)
- systemd: `etc/systemd/system/rp608-nse-shadow-daemon.service`
- Tests: `tests/test_rp608_nse_trade_pattern.py`
