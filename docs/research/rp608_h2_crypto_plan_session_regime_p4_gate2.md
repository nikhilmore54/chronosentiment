# RP-608 P4 — H2 Crypto Plan Session Regime — Gate 2 / Results Memo

**Date:** 2026-11-06 (Pi run) · docs refreshed 2026-11-07  
**Protocol:** `RP-608-H2-CRYPTO-PLAN-SESSION-REGIME-P4`  
**Book:** Crypto recommendation plan only (not pooled)  
**Report:** `artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.md`  
**Script:** `scripts/research/rp608_h2_crypto_plan_session_regime_p4.py`  
**Freeze:** `docs/research/rp608_hypothesis_p3_freeze.md`  
**Authority:** 0.0% · baseline frozen · OF ±0.30 frozen  

## Formal disposition

**H2 assessment:** `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive clarity)  
**Gate 2:** **DO NOT ADVANCE**  
**Gate 3:** unauthorized  
**Confirmation:** `NEED_MORE_SESSIONS` (n=42 on 2026-11-06 only)

“Supported” here means the **session-regime account is documented and chronologically consistent in structure** (high WR dispersion). It is **not** an entry filter, not a tradable rule, and not Gate 2 advancement.

## Result card (locked windows)

| Window | n | WR% | W/L | Payoff | Net USDT | WR range (sessions n≥5) |
| --- | ---: | ---: | --- | ---: | ---: | ---: |
| Discovery ≤2026-10-31 | 554 | 41.7 | 231/323 | 1.85 | +315.12 | **25.55 pp** (22.73–48.28) |
| Validation 2026-11-01…05 | 441 | 33.11 | 146/295 | 0.53 | −747.14 | **39.24 pp** (15.60–54.84) |
| Confirmation ≥2026-11-06 | 42 | 59.52 | 25/17 | 1.06 | +32.57 | insufficient sessions |

### Concentration (discovery)

- Top session by n/wins/net: **2026-10-29** — 68.41% of trades, 79.22% of wins, net +591.24 USDT  

### Integrity

- Eligible 1037/1037 · parse failures 0  
- Confirmation **not** merged into validation  
- H1 / H3 / H4 not opened · no threshold scan · no velocity inference  

## Interpretation (programme rules)

| Finding | Meaning |
| --- | --- |
| High WR dispersion in discovery **and** validation | Session-level conditions dominate WR narrative |
| Validation aggregate WR↓ and payoff&lt;1 / net&lt;0 | Economics do not support a general improvement claim |
| Oct 29 concentration in discovery | Strong-session risk; must not be used as transferable policy |
| Confirmation thin | Do not claim confirmation; acquire more sessions |

## Non-claims

- Does not authorize entry filters, session bans, or thr scans  
- Does not reopen H4 / adaptive execution velocity  
- Does not change OF ±0.30, stops, targets, horizon, or live authority  
- Does not pool crypto OF or NSE books  

## Next (observational) — locked 2026-11-07

**Milestone:** `docs/research/rp608_h2_prospective_confirmation_accumulation.md`  

1. Accumulate confirmation sessions ≥2026-11-06; do **not** merge into validation; do not declare success from 2026-11-06 alone.  
2. Leave H1 / H3 / H4 exactly where they are (not runnable / unopened / parked).  
3. No discovery rerun, thr scan, or velocity inference from H2.  
