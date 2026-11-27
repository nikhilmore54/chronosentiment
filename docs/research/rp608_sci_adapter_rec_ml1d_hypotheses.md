# RP-608-SCI-ADAPTER-REC-ML1D — Falsifiable hypotheses (H0–H11)

**Date:** 2026-11-27 Asia/Calcutta  
**Status:** SELECTED for offline implementation · all H* **OPEN** until sealed holdout  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · books never pooled · no Gate 5  
**Aspiration:** ~85% ACT-subset W/L + after-friction margins  
**Store mirror:** agent-store `docs/sci-adapter-rec-ml1d-hypotheses.md`

Each row: claim · primary falsifier · experiment map · status.

---

## Core

| ID | Claim | Falsifier | Map |
| --- | --- | --- | --- |
| **H0** | E0 (always follow Coralys / no adapter filter) is not after-cost competitive vs frozen negatives; SCI-v1 S1=b does not fix expectancy | E0 or SCI-v1 sealed holdout net/W/L beats dual ML-1D under identical exits | E0 · SCI-v1 book |
| **H1** | Adapter ACT/PASS around sealed Coralys raises ACT-subset closed W/L and after-friction net vs E0 without side/size/exit authority | E1 or E3 fail to improve W/L+net vs E0 at disclosed coverage | E1 · E3 |
| **H2** | Dual (reliability+economic) beats reliability-only and economic-only on after-cost net (not WR via vanishing coverage) | E3 net ≤ max(E1,E2) net at comparable or fair coverage | E3 vs E1/E2 |

## Reliability

| ID | Claim | Falsifier | Map |
| --- | --- | --- | --- |
| **H3** | OOS-calibrated \(Q_t=P(\text{win}\mid\text{Coralys side, context})\) calibrates better than uncalibrated scores | Brier/reliability on holdout not improved vs raw | R1 |
| **H4** | Evidence-sufficient ACT subset lifts closed W/L vs E0; path to ~85% is selective coverage | No W/L lift at any predeclared evidence policy with usable coverage | E1 · H10 |
| **H5** | Reliability drifts; adaptive/weighted conformal flags deterioration earlier than static calib | Stratum coverage / detection delay no better than static | R2 (after R1) |
| **H6** | Post-ACT reliability without selection-aware inference is miscalibrated | Selection-conditional/FCR diagnostics show no bias vs naive | R3 (E3 reporting) |

## Economic

| ID | Claim | Falsifier | Map |
| --- | --- | --- | --- |
| **H7** | \(U_t\) separates high-\(Q_t\) friction-negative cases; PASS improves net vs reliability-only | E2/E3 net ≤ E1 net | Ec1 · E2 |
| **H8** | Liquidity/session features improve \(U_t\) ranking vs constant friction | Ec2 adds no holdout net lift vs Ec1 | Ec2 |
| **H9** | Dual gate beats single fused score that double-counts \(Q_t\) | Fused score ≥ dual on holdout net | Ablation |

## Aspiration / governance

| ID | Claim | Falsifier | Map |
| --- | --- | --- | --- |
| **H10** | Predeclared coverage + Wilson LB + payoff + net can jointly approach ~85% ACT W/L with good margins | WR-alone “success”, vanishing coverage, or Wilson LB far below aspiration with thin n | E3 scorecard |
| **H11** | Replacing Coralys with a 15m side generator does not beat dual ACT/PASS on after-cost BTC metrics under identical exits | SCI-v1 S1=b holdout net/W/L ≥ ML-1D E3 | SCI-v1 comparator |

## Reject list (non-hypotheses)

Unconditional always-on 85% 15m classifier · soak remine · paper 90% coverage / hist. bps as cuts · Gate 5 from docs · Coralys edits.

---

*End — H0–H11 · 2026-11-27 Asia/Calcutta.*
