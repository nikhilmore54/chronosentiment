# Gate 2E — 15m OHLCV multi-lookback trend agreement (scoped)

**Protocol:** `RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001`  
**Date:** 2026-11-26  
**Authority:** 0.0% · observational twin only  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 (`rp608_gate2_amendment_v2.md`) · NSE push/Rust production mandate  
**Coralys:** no-touch (`coralys-*` frozen). **Logic owned in Rust** (`adapters/chronosentiment` → `ohlcv_trend_agreement` + `rp608_position_decide --trend-agree-loop`). Python is I/O / journal fan-out only.

## Intent

15m-first redesign of **entry selection** (lever B). Do not further stack live PRIMARY Gate 2E knobs (lever A) as the first move. Run B as parallel twins; compare W/L-first vs A and ungated BASELINE.

## Predictors (decision-time only)

Completed **15m** closes available at decision time (no partial/forming bar):

| Lookback | Definition |
| --- | --- |
| L3 | `sign(close[-1] - close[-3])` |
| L8 | `sign(close[-1] - close[-8])` |
| L20 | `sign(close[-1] - close[-20])` |

Flat band: `|ret| < 1e-12` → `FLAT` (no vote).

**Agreement (predeclared):** among lookbacks with enough history, need **≥2 votes equal to candidate side** and **0 opposing votes**. Otherwise `STANDASIDE`.

**Selector:** Coralys/Policy B or Estimand A candidate side must **agree** → B twin may open; else stamp standaside. Frozen geometry (traj15 threshold, OF ±0.30, stops/targets) unchanged.

## Rust boundary

| Piece | Location |
| --- | --- |
| Pure agreement | `adapters/chronosentiment/src/product/ohlcv_trend_agreement.rs` |
| CLI loop | `rp608_position_decide --trend-agree-loop` |
| Python client | `scripts/rp608_gate2e_15m_trend_agree.py` (fail-closed if bin missing) |

## Books (never pooled)

| Book | Role |
| --- | --- |
| CRYPTO PRIMARY / ETH PRIMARY | A — live Gate 2E stack (unchanged) |
| CRYPTO/ETH BASELINE | Ungated control |
| CRYPTO/ETH `…_trend_agree_parallel_v1` | B twin |
| NSE Estimand A PRIMARY | A |
| NSE Estimand A BASELINE | Ungated control |
| NSE `estimand_a_trend_agree_parallel_v1` | B twin |

## Windows / coverage

- Chronological soak from twin arming; no remine of consumed 2026-11-03…05 windows.
- Coverage floor (Gate 2 v2): preferred cohort ≥ **min(40% of matched BASELINE opens, or absolute n ≥ 80)** on holdout unless tighter floor justified in review.
- Primary metric: **W/L** per book; payoff / friction / net required disclosure.

## Advance rule (predeclared)

B → `ADVANCE_CANDIDATE` on a book only if holdout W/L improves vs BASELINE **and** (vs A when A has closed n≥20, else vs BASELINE with coverage floor met). Payoff/net disclosed. No Gate 5 / no PRIMARY cutover from this memo alone.

## Env

| Var | Default | Role |
| --- | --- | --- |
| `RP608_GATE2E_TREND_AGREE_TWIN` | `1` | Arm crypto B twin |
| `RP608_NSE_GATE2E_TREND_AGREE_TWIN` | `1` | Arm NSE B twin |
| `RP608_TREND_AGREE_BIN` / `RP608_POSITION_DECIDE_BIN` | resolve path | Rust binary with `--trend-agree-loop` |

## Out of scope

- `coralys-*` edits · multi-TF candle sizes · traj15 / OF ±0.30 / stop-target changes · promoting B to PRIMARY · book pooling · OFX authority · Python reimplementation of agreement math
