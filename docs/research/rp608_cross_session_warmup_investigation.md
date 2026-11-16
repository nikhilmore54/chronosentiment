# RP-608 Cross-Session Warmup Investigation

**Protocol:** `RP-608-CROSS-SESSION-WARMUP-DIAG-001`  
**Disposition:** INVESTIGATION ONLY · **0.0% order authority**  
**Live rules:** unchanged (G1 1–19, G4 `traj15 ≥ +0.05`, reverse-flow / OF thresholds frozen)

## Problem

Same-session G3 (`history ≥ 5` today only) is internally consistent but systematically discards early-session bars (09:30–10:30). Indicator warmup and session eligibility are distinct requirements and should not be conflated.

## Policies under comparison

| Policy | History provenance | Overnight gap |
| --- | --- | --- |
| `same_session` | Today’s completed 15m bars only | N/A (current live) |
| `cross_naive` | Prior session + today | Included in traj15 (artefact probe) |
| `cross_gap_neutral` | Prior session + today | Overnight return zeroed; open→close of first today bar retained |

RP-607 profiles remain descriptive and are **not** used as candle substitutes.

## Diagnostic runner

```bash
python3 scripts/research/rp608_cross_session_warmup_diagnostic.py \
  --today YYYY-MM-DD \
  --lookback-days 5 \
  --out artifacts/rp608_cross_session_warmup/diagnostic_report.json
```

Emits per-symbol first G3-ready bar, first G4 signal bar, and early-bar disagreements between naive vs gap-neutral.

## Acceptance before any live G3 change

1. Gap-neutral recovers G3 readiness before bar 5 without raising early false signals vs same-session after bar 5.
2. Naive-vs-gap-neutral disagreements on bars 1–4 are explained by measured overnight gaps.
3. Thresholds and historical strategy ledgers remain untouched during comparison.
4. Separately versioned experimental entry policy only after this evidence review.

## First diagnostic run · 2026-11-02 (bars 1–3 available)

Artifact: `artifacts/rp608_cross_session_warmup/diagnostic_report_2026-11-02.json`  
Universe: 20 watchlist symbols · prior session used: **2026-10-30** (24 bars) · today: 3 completed bars.

| Policy | G3 ready by bar ≤4 | Signals bars 1–4 | Median first G3 |
| --- | ---: | ---: | ---: |
| `same_session` (live) | 0 / 20 | 0 | n/a (cold) |
| `cross_naive` | 20 / 20 | 17 | 1 |
| `cross_gap_neutral` | 20 / 20 | 16 | 1 |

**Gap artefact evidence:** 8 / 20 symbols disagree naive vs gap-neutral on bars 1–4. Example RELIANCE bar 1: overnight gap **+596 bps** → naive `traj15=+0.36` (signal) vs gap-neutral `+0.04` (no signal).

## Full-session bar-5+ replay · 2026-10-30

Artifact: `artifacts/rp608_cross_session_warmup/diagnostic_report_2026-10-30_bar5plus.json`  
Prior: **2026-10-29** · entry-window bars through 19.

| Policy | Signals 1–4 | Incremental early-only | Timing advance | Agree vs A after bar 5 | Still-signal @+4 | Sign flips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A same_session | 0 | — | — | — | — | — |
| B cross_naive | 13 | 2 | 11 | 71.7% (215/300) | 6/13 | 4 |
| C cross_gap_neutral | 11 | **1** | **10** | **79.0%** (237/300) | **8/11** | **1** |

Most early C/B hits are **timing advances** of later same-session signals, not net-new opportunities. True incremental early-only under C is rare on this day (1/20).

## Today bar-5+ snapshot · 2026-11-02 (bars through 6)

Artifact: `artifacts/rp608_cross_session_warmup/diagnostic_report_2026-11-02_bar5plus.json`

| Policy | Signals 1–4 | Incremental\* | Timing advance | Agree vs A (bars 5–6 only) |
| --- | ---: | ---: | ---: | ---: |
| B cross_naive | 17 | 7 | 10 | 70.0% (28/40) |
| C cross_gap_neutral | 16 | 6 | 10 | 65.0% (26/40) |

\*Partial-day incremental counts are upward-biased: same-session has only just unlocked; later baseline matches may still appear. Prefer the Oct-30 full-window rates for agreement/persistence.

Gap sensitivity today: **9/20** naive≠gap-neutral on bars 1–4 (RELIANCE gap +596 bps still decisive).

**Live status (updated 2026-11-03):** Experimental live G3 policy **`cross_gap_neutral_v1`** enabled via `RP608_G3_WARMUP_POLICY` (default). Same-session cold-until-10:30 remains available by setting `RP608_G3_WARMUP_POLICY=same_session`. OF ±0.30, G1 window, and Estimand A exit formulas unchanged. Authority 0%.

Implementation: `scripts/rp608_nse_g3_warmup.py` · daemon `evaluate_symbol_bar` stamps `g3_warmup_policy` / `g3_history_length`.
