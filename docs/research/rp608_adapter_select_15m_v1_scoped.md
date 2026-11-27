# RP-608-ADAPTER-SELECT-15M-V1 — scoped freeze (slice 1)

**Protocol:** `RP-608-ADAPTER-SELECT-15M-V1`  
**Schema:** `1`  
**Date:** 2026-11-27 (Asia/Calcutta)  
**Authority:** 0.0% · observational twin only  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 · Coralys no-touch · Rust-first / Python fail-closed I/O  
**Parent charter:** `docs/research/rp608_clean_slate_15m_adapter_selection.md`

## Frozen D1–D5 (Anuj 2026-11-27)

| # | Decision | Freeze |
| --- | --- | --- |
| **D1** | Candidate source | **(a)** Score as filter on existing Coralys/Estimand candidate side |
| **D2** | Enter band | **\|S_side\| ≥ 0.35** → ENTER; else STANDASIDE |
| **D3** | First books | **BTC + ETH** observational; NSE not armed in slice 1 |
| **D4** | Gate 2E B | Leave B soak running; **exclude from this promote path** |
| **D5** | Design | Approved as coding charter |

## Predictors (decision-time only)

Completed **15m** closes (no partial/forming bar):

| Feature | Definition |
| --- | --- |
| Signed returns | \(r_k=(c_{-1}-c_{-k})/\|c_{-k}\|\) for \(k\in\{3,8,20\}\) |
| Magnitude ranks | Rank of \|r_k\| vs up to 50 prior completed lookback returns |
| Abs saturation | `sat=\|r\|/(\|r\|+0.02)` — keeps mature drift from collapsing when ranks are low |
| Soft agreement | Weighted sum of `sign(r_k)·(0.35·mag_rank + 0.65·sat)` with weights **0.45 / 0.35 / 0.20** |
| Path consistency | Fraction of L20 consecutive close steps moving with `r_20` |
| Score S | `clamp(0.85·soft + 0.15·path_signed, −1, +1)` |
| Side score | `S · (+1 LONG \| −1 SHORT)` |

**Bands:** `INSUFFICIENT` if `len(closes) < 20`; `ENTER` if `side_score ≥ 0.35`; else `STANDASIDE`. Flat/unknown candidate → STANDASIDE (`CANDIDATE_SIDE_NOT_TRADEABLE`).

## Rust boundary

| Piece | Location |
| --- | --- |
| Pure scorer | `adapters/chronosentiment/src/product/adapter_select_15m.rs` |
| CLI loop | `rp608_position_decide --adapter-select-15m-loop` |
| Python client | `scripts/rp608_adapter_select_15m.py` (fail-closed if bin missing) |
| Twin arm | `scripts/run_rp608_shadow_daemon.py` when `RP608_ADAPTER_SELECT_15M_TWIN=1` |
| Scorecard | `scripts/research/rp608_adapter_select_15m_status.py` |

## Twin book ids (locked)

| Book | Dir (under symbol output) | Book id |
| --- | --- | --- |
| BTC | `trade_plan_overlay_adapter_select_15m_v1` | `CRYPTO_PLAN_ADAPTER_SELECT_15M` |
| ETH | `trade_plan_overlay_adapter_select_15m_v1` | `CRYPTO_PLAN_ADAPTER_SELECT_15M_ETHUSDT` |
| NSE | — | **Not armed** (slice 1) |

## Env

| Var | Default | Role |
| --- | --- | --- |
| `RP608_ADAPTER_SELECT_15M_TWIN` | `0` | Arm BTC/ETH observational twin (Anuj-gated deploy) |
| `RP608_ADAPTER_SELECT_15M_BIN` / `RP608_POSITION_DECIDE_BIN` | resolve path | Rust binary with `--adapter-select-15m-loop` |

## Out of scope (slice 1)

- `coralys-*` edits · Gate 5 floor / OF ±0.30 / traj15 / stop-target retunes  
- Gate 2E B binary/config/journal changes · NSE arm · Pi restart without Anuj  
- Python reimplementation of scoring math · promote twin → PRIMARY · book pooling

## Advance rule (predeclared; Gate 2 v2)

Per book after chronological soak from twin arming: W/L improves vs BASELINE (and vs A when \(n_A\ge20\)); coverage ≥ min(40% matched BASELINE opens, or \(n\ge80\)); payoff/friction/net disclosed. Net-hostile WR-only = fail. No Gate 5 from this memo alone.
