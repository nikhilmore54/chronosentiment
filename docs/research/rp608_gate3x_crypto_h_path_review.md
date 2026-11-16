# RP-608-GATE3X-CRYPTO-H-PATH-001 — Review Memo

**Date:** 2026-11-16  
**Authority:** 0% · **no live overlay edit in this memo**  
**Artifacts:** `artifacts/rp608_gate3x_crypto_h_path/`  
**Script:** `scripts/research/rp608_gate3x_crypto_h_path_stress.py`  
**Scope:** `docs/research/rp608_gate3x_crypto_h_path_stress_scoped.md`  
**Depends on:** Gate 5 Crypto G deployed · Gate 2X-001 H* ADVANCE_CANDIDATE  

## Disposition

**`PASS_GATE3X_DRAFT_GATE5_H_PACKET_BOTH`**

Both **H60_PATH** and **H120_PATH** clear the predeclared promote rule on chronological holdout.

| Check | H60_PATH | H120_PATH |
| --- | --- | --- |
| Holdout Δnet vs A | **+255.36 USDT** | **+381.17 USDT** |
| Paired sessions Δ≥0 | **9/9 (1.0)** | **9/9 (1.0)** |
| Path scratches (holdout) | 66 | 95 |
| Promote fails | none | none |

## Method (predeclared)

- Entries frozen from `trade_plan_outcomes.jsonl` (sha in results JSON).  
- **OPT:** early `STOP_LOSS_HIT` → scratch@entry − 12 bps RT friction (optimistic Gate2X bound).  
- **PATH:** early SL → scratch **only if** Binance **1s** mid leaves entry±6 bps adverse, then **revisits** the band inside `[entry, entry+T]` (T=60/120s); else keep production SL net.  
- Chronological 60/40 day split (same family as Gate3X-G).  
- Books not pooled; OF ±0.30 / ATR / authority untouched.

## Holdout card

| Policy | net USDT | Δ vs A | WR% | payoff |
| --- | ---: | ---: | ---: | ---: |
| A production | −928.63 | — | 32.55 | 1.00 |
| H60_OPT | — | +858.37 | — | — |
| H120_OPT | — | +880.52 | — | — |
| **H60_PATH** | **−673.27** | **+255.36** | 32.55 | **1.17** |
| **H120_PATH** | **−547.46** | **+381.17** | 32.55 | **1.27** |

WR unchanged under PATH: scratch nets are −friction (still booked as losses). Payoff and net improve via smaller loss magnitude — acceptable under Gate 2X (net primary; W/L disclosed).

### Path taxonomy (holdout early SL &lt;60s)

| Reason | n |
| --- | ---: |
| NO_ADVERSE | 132 |
| REVISIT (scratch) | 66 |
| ADVERSE_ONLY (keep SL) | 46 |

PATH is materially stricter than OPT (holdout scratches 66 vs 244 early60).

## Interpretation

1. Optimistic H* still upper-bounds the lift; PATH recovers a **robust fraction** with 1s revisit evidence.  
2. **H120_PATH** dominates **H60_PATH** on holdout Δnet without losing session robustness.  
3. Prefer Gate 5 draft for **H120_PATH** as primary (larger net, same paired share); H60_PATH as nested/fallback.  
4. Production meaning (draft only): when ATR stop would fire with hold &lt; T, **do not** take full ATR SL if path has revisited entry±friction — exit scratch@entry (or equivalent min-hold scratch). Exact live predicate must match PATH, not OPT.

## Authorized next

- Draft Gate 5 packet: `RP-608-GATE5-CRYPTO-H-001` (H120_PATH primary; H60 optional nested).  
- **Do not** edit `rp608_trade_plan_overlay.py` until Gate 5 acceptance.  

## Non-claims / unauthorized

- Live daemon deploy from this memo alone  
- OF ±0.30 / ATR multiplier / horizon / authority changes  
- Treating OPT scratch as fill-realistic  
- NSE books / OFX pooling  
