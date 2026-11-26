# RP-608 Gate 2E — Crypto trade-pattern sealed package (2026-11-21)

**Protocol:** `RP-608-GATE2E-CRYPTO-PATTERN-ALLOW-ONLY-001`  
**Book:** `CRYPTO_PLAN` (PRIMARY) only — BASELINE twin remains ungated control

## Live ALLOW cohort (2026-11-22 ~23:00 IST)

Gate2E `allow_only` began opening NEUTRAL BACKOFF cells (mostly `EU|LONG|…|VUNK` / `US|LONG|…|VUNK`, live vol stamped separately).

| Metric | Value |
| --- | --- |
| PLAN_CREATED since gate-on | **53** (1 still open) |
| ALLOW closed | **52** · W/L **10/42** · wr **0.192** · payoff **3.51** · net **−20.3** |
| vs BTC BASELINE since-gate | W/L 41/138 · net **−134** (more trades, worse net) |
| Match mode | BACKOFF → mined NEUTRAL parent; `feature_pattern_id` has live V* |
| Verdict | Gate thinned damage vs ungated · ALLOW cohort **not** yet positive expectancy → **no ADVANCE** / no gate loosen |
| ETH PRIMARY | still **0** ALLOW (library INSUFFICIENT-only) |

Status tool: full-file event scan (tail truncate was under-counting suppresses/creates).

**Authority:** 0.0% paper  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · `rp608_gate2_amendment_v2.md` · allow-only plan (no trade > bad trade)

## Deploy disposition

| Item | Value |
| --- | --- |
| PRIMARY `RP608_CRYPTO_PATTERN_GATE` | **`allow_only`** — open only on `ALLOW` |
| BASELINE `RP608_CRYPTO_PATTERN_GATE_BASELINE` | **`0`** — stamp/control; never suppress |
| `RP608_CRYPTO_PATTERN_EXIT_GATE` | **0** |
| Matcher | Parent-cell BACKOFF (not softened-key lookup) |
| OF ±0.30 / traj15 / Rust lifecycle | **unchanged** |

### Policy (PRIMARY)

| Prediction | Action |
| --- | --- |
| `ALLOW` | Open — **only** when label is NEUTRAL **and** train `net_mean > 0` |
| `STANDASIDE_ADVERSE_PATTERN` | Suppress `SUPPRESSED_TRADE_PATTERN` |
| `ABSTAIN_INSUFFICIENT` | Suppress (fail-closed) |
| `ABSTAIN_NEGATIVE_EXPECTANCY` | Suppress (NEUTRAL/other with `net_mean ≤ 0` or missing) |

**2026-11-23 selection fix:** all five live NEUTRAL `VUNK` parents had `net_mean < 0` but were still ALLOW’d → trail-churn cohort (W/L≈0.19 net−20). Matcher now fail-closes those. OF ±0.30 / trail / stops unchanged (Gate 5).

## Stamp / evidence window

- **Obs remine:** 2026-11-21T05:32:23Z  
- **Allow-only + parent backoff live:** 2026-11-21T06:04Z approx (Pi systemd restart; journal `gate2e primary='allow_only' baseline='0'`)
- **Verify replay (post Rust-cutover closed n=10):** match_mode **BACKOFF 10/10** (was NONE); predictions still mostly ABSTAIN on thin cells → **would block under allow_only**

## Library mine snapshot (PRIMARY outcomes only)

| Field | Value |
| --- | --- |
| Schema | `crypto_plan_trade_pattern_library_v2` |
| Entry patterns | 186 (adverse **8**) |
| Adverse min n / max WR | 30 / 0.35 |

## Predeclared Gate 2E advance criteria (frozen)

Per `rp608_gate2_amendment_v2.md` §2.1 — W/L primary; disclose payoff/friction/net/coverage; coverage floor ≥ min(40% of book, or n≥80) on holdout; books never pooled; no OF ±0.30 from 2E alone.

**Note:** allow_only **will thin** coverage by design. Thinning is expected; ADVANCE still requires holdout W/L improvement on the allowed cohort vs control (BASELINE ungated), not “zero trades = success.”

## Status

**`ALLOW_ONLY_LIVE` on PRIMARY — Gate 5 paper entry filter.**  

Formal Gate 2E **ADVANCE / DO NOT ADVANCE** still requires filled holdout table below after stamp window (compare PRIMARY allowed opens vs BASELINE same-signal control).

| Cohort | n | W | L | WR | payoff | net_sum | coverage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| PRIMARY ALLOW opens | — | — | — | — | — | — | — |
| PRIMARY suppressed (counterfactual via BASELINE) | — | — | — | — | — | — | — |
| BASELINE all opens (control) | — | — | — | — | — | — | — |

**Review outcome:** _unset_ (`ADVANCE` | `DO_NOT_ADVANCE`)

## Code / ops

- Matcher: `scripts/rp608_crypto_trade_pattern_match.py` (`_parent_cell_match`, `allow_only`)
- Overlay per-book mode: `TradePlanOverlayManager(pattern_entry_gate_mode=…)`
- Daemon: `RP608_CRYPTO_PATTERN_GATE` / `_BASELINE` in `etc/systemd/system/rp608-shadow-observer.service`
- Tests: `tests/test_rp608_crypto_trade_pattern.py`

## Shadow thin-WR stamps (2026-11-22 · observational)

- **Does not** change `allow_only`. Stamp-only `SHADOW_ALLOW_IF_N10_WR_GT_035` when suppress is ABSTAIN/thin with `n≥10` and `hist_wr>0.35`.
- Env `RP608_CRYPTO_GATE2E_SHADOW_THIN_WR` default **ON**; ledger `gate2e_shadow_allows.jsonl`.
- Doc: `docs/research/rp608_gate2e_shadow_thin_wr_allow_2026_11_22.md`
