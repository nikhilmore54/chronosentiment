# RP-608-WR-ENTRY-DISCOVERY-001 — Scoped

**Protocol:** `RP-608-WR-ENTRY-DISCOVERY-001`  
**Program:** RP-608 Win Rate Improvement v2  
**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · Gate 2 **criteria unchanged** · Gate 3 unauthorized  
**Book:** NSE Estimand A only (never pooled)  
**Host:** Pi ([rp608_pi_research_execution.md](rp608_pi_research_execution.md))

## Objective

Improve \(P(\mathrm{Win}\mid X_{\mathrm{entry}})\) via decision-time entry selection.  
**Aspirational WR 85%** is a research objective — **not** a Gate 2 acceptance criterion.

Primary metric: win rate with absolute W/L counts.  
Required supporting: payoff · net · trade count / coverage · rejected-trade W/L · calibration.

## Algorithm (v2)

1. Preserve book + frozen baseline  
2. Decision-time features only (no post-entry MFE/MAE/exit path as predictors)  
3. Preserve existing net W/L labels  
4. Baseline WR + coverage with counts  
5. Discover entry-quality candidates (session-aware)  
6. Stability / redundancy / selection-bias audits  
7. Falsifiable hypotheses  
8. Existing Gate 2 criteria unchanged  

## Windows (predeclared)

| Window | Days (IST) |
| --- | --- |
| Discovery | 2026-11-03, 2026-11-04, 2026-11-05 |
| Holdout pre | 2026-10-27, 2026-10-28, 2026-10-30, 2026-11-02 |
| Holdout forward | 2026-11-06 |

## Stages in this deliverable

| Stage | Work |
| --- | --- |
| 1 | Feature completeness / missingness / leakage audit |
| 2 | Entry-time W/L associations + \(P(\mathrm{Win}\mid X)\) scores + calibration |
| 3 | Session / symbol stability; reject unstable |
| 4 | Hypothesis specification (including failed) |
| 5 | Gate 2 assessment (unchanged criteria) |

Stages 6–7 (offline experiment / further unseen validation) require Gate 2 advance — **not** in this run.

## Forbidden

- Changing OF ±0.30, stops, targets, Estimand A entry thresholds  
- Manufacturing WR by unreported coverage collapse  
- Redefining W/L labels; pooling books; symbol bans from single-day loss lists  
- Using exit reasons as entry predictors  

## Artifacts

- Script: `scripts/research/rp608_wr_entry_discovery_001.py`  
- Report: `artifacts/rp608_econ_edge_diag/rp608_wr_entry_discovery_001.{md,json}`  
- Join input: `artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl`
