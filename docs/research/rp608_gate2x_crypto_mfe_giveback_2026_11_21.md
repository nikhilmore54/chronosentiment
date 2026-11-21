# RP-608 Gate 2X — Crypto MFE give-back protect (2026-11-21)

**Protocol:** `RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001`  
**Authority:** 0.0%  
**Books:** `CRYPTO_PLAN` / `CRYPTO_PLAN_BASELINE_PARALLEL` (never pooled)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · Gate 2E study `rp608_gate2e_crypto_abstain_entry_study_2026_11_20.md`

## Decision authority (2026-11-21 — adapter + Coralys)

Gate 2X **Hold | Protect | Exit** is decided in the ChronoSentiment adapter:

1. Feature snapshot built by shadow (peak MFE, tp_frac, vol surge, dwell clock)
2. `gate2x_decide` → `evaluate_coralys_direction_aligned_state` → frozen `RecommendationEngineV1`
3. Crypto REC001h: `datasets/recommendation/historical_crypto/BTCUSDT.jsonl` (never pooled with NSE)
4. In-process manage: `Gate2xPositionManager` · CLI `rp608_position_decide --manage-loop` (preferred) / one-shot fallback · Python: `scripts/rp608_gate2x_adapter_client.py` (**executor only**, fail-closed Hold if no Rust bin) · goldens: `product::gate2x_decide` + `product::gate2x_position_manage` · see `docs/research/rp608_position_decide_cli.md`
5. If binary missing: **Hold only** (`GATE2X_NO_RUST_BIN`) — no Python Protect invent; deploy aarch64 binary
6. `coralys-*` crates frozen (PVP-003). Full Rust daemon port remains incremental; Gate 2X authority already Rust-side.

## Question

Session W/L remains weak after Gate 2E. Which **win patterns** can be promoted, and where did **losses miss an exit**?

## Session facts (CRYPTO_PLAN since twin restart `2026-11-20T12:59:49Z`)

| Metric | Value |
| --- | --- |
| Closed (pre–v1 tighten) | 49 → then live giveback on open |
| W/L | **16/33** then **16/34** after first Gate 2X scratch |
| Payoff asym | ≈ 1.15 |

### Wins — promote vs do-not-promote

| Cohort | n | Action |
| --- | --- | --- |
| Instant TP &lt;30s cascade | 6 | **Do not promote as entry** (path-through; gaps 1–13s). **Not suppressed** — killing cascade would delete ~+62 net this session |
| Quality hold ≥2m → TP | 6 | **Promote path** — progress toward TP is the signal |
| Quality hold ≥2m → horizon | 4 | Small win / scratch — keep |
| Pattern / HIGH / ABSTAIN | — | **Reject** as W/L separator (ABSTAIN 15/31) |

### Losses — missed exits (hypothesis-only)

Peak MFE must clear **costs + 5 bps floor** (§3b) before counting support.

| Bucket | n | Domain |
| --- | --- | --- |
| Fast SL, MFE not clearing | 12 | Gate 2E churn (pre-deploy) |
| Fast SL with wick MFE | 12 | Weak exit claim |
| **Actionable ≥2m, MFE clears** | **7→8** | Gate 2X |
| └ horizon give-back | 3 | |
| └ MFE→SL (incl. tp_frac 0.97) | 4 | |
| └ live `GATE2X_MFE_GIVEBACK_PROTECT` | 1 | PLAN-252313 net −1.21 (first fire) |

**Counterfactual (actionable only → exit at entry-scratch ≈ −1.2):** session net −97 → ≈ **−60** (W/L unchanged; large loss tails cut). Full MFE-clear rewrite (incl. short holds) overstates benefit and is **not** the live rule.

## Gate 2X rule (v1.1 — quality path)

In `evaluate_market_tick`:

1. Track peak MFE (USDT) and peak **tp_frac**
2. **Arm** only when:
   - peak MFE &gt; max(round-trip friction, 5 bps×notional), **and**
   - peak tp_frac ≥ **0.40** (quality progress toward TP)
3. **Exit** only after hold ≥ **120s**, when current MFE ≤ **cost-clear floor** (armed threshold)  
   → `GATE2X_MFE_GIVEBACK_PROTECT` / `CLOSED_GIVEBACK` / detail `MFE_GIVEBACK_TO_COST_CLEAR`
4. Config: `gate2x_protect_floor=COST_CLEAR` (alt: `ENTRY` for cur_mfe≤0)

**Intent:** promote the quality-hold-toward-TP path; cut give-backs that had the trade then reversed into SL/horizon. Avoid arming on tiny MFE wicks that quality winners also print before the real move.

## v1.2 — volume-confirmed reversal (2026-11-21)

Fake/small reversals must not cut quality paths:

1. Soft-breach when cur MFE ≤ protect floor  
2. If candle `exit_vol_surge` ∈ **HIGH** → exit now (`…_VOL_HIGH`)  
3. If **LOW/MED/UNK** → start dwell; keep holding while watching  
4. If still through floor after dwell → exit  
5. If price recovers above floor → clear soft-breach (no exit)

Candle OHLCV burst buckets (not OFX aggression). Same twin baseline books.

## v1.3 — ratchet + tiered/deep dwell (2026-11-21)

1. **Ratchet floor:** `max(cost_clear, 0.30 × peak_mfe)` — protect a slice of printed MFE, not only scratch  
2. **Vol dwell tiers:** HIGH → now; **MED → 40s**; **LOW/UNK → 90s**  
3. **Deep giveback:** if `(peak−cur)/peak ≥ 0.50`, multiply dwell by **0.40** (min 5s) — grinding reversal ≠ fake wick  

Config keys: `gate2x_ratchet_peak_frac`, `gate2x_reversal_med_vol_dwell_ms`, `gate2x_deep_giveback_*`.

## v1.4 — Adaptive TP runner (2026-11-21)

Paired with Gate 5 packet `RP-608-GATE5-CRYPTO-ADAPTIVE-TP-001`:

1. First TP is still **1.5×ATR** at open. On TP touch with `tp_level < max_extensions`: **extend** TP by **+0.5×ATR**, **lock SL** to prior TP — **do not** hard-close.
2. Gate 2X **Hold | Protect | Exit** decides whether to keep running toward the new TP or take profit (giveback protect at mark).
3. Snapshot fields: `tp_level`, `adaptive_armed` (for decide/Coralys; fail-closed Hold if Rust bin missing).
4. Hard cap: after max extensions, next TP touch → `PROFIT_TARGET_HIT`.

**Intent:** capture more profit on quality paths that already cleared 1.5×ATR, while Gate 2X still cuts give-backs. W/L-first per book; payoff + net required.

Config (overlay): `adaptive_tp_enabled`, `adaptive_tp_extend_atr_mult`, `adaptive_tp_max_extensions`.

## Gate 2E (unchanged role)

Fast SL re-entry cooldown + path-invalid remain the entry anti-churn layer. Post–Gate 2E deploy, session bleed was give-back — hence Gate 2X.

## Not changed

OF ±0.30 · **initial** ATR stop/target multipliers at open (1.0× / 1.5×) · 60m horizon · pattern-gate default off · no hour/symbol bans · books never pooled  

**Changed under Gate 5 Adaptive TP:** post-touch TP may extend; SL may lock to prior TP.

## Live validation

| Plan | Exit | Net | Note |
| --- | --- | --- | --- |
| PLAN-252313 | `GATE2X_MFE_GIVEBACK_PROTECT` | −1.21 | Armed peak MFE ≈ 3.48; gave back → scratch vs open-ended risk |

## NSE port

Same v1.1 rule on Estimand A LONG: `docs/research/rp608_gate2x_nse_mfe_giveback_2026_11_21.md` · `scripts/run_rp608_nse_shadow_daemon.py`.
