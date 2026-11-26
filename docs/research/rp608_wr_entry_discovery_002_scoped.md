# RP-608-WR-ENTRY-DISCOVERY-002 — Selection Frontier and Stability Audit

**Protocol:** `RP-608-WR-ENTRY-DISCOVERY-002`  
**Parent:** `RP-608-WR-ENTRY-DISCOVERY-001` / WR Improvement v2  
**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · Gate 2 criteria unchanged · Gate 3 unauthorized  
**Book:** NSE Estimand A only  
**Host:** Pi

## Why this iteration

001 produced a best powered holdout selection (`P_hat≥0.5`) at **70% WR** with only **8.6% coverage**. That is not an 85% strategy and is too selective.  
**Do not expand feature mining yet.** Audit ranking quality, coverage frontier, and stability of the existing score.

## Objectives

1. Exact candidate feature + score definition (from 001)  
2. Discovery/holdout boundary audit (was threshold pre-specified?)  
3. WR–coverage curve across score thresholds  
4. W/L counts at each threshold  
5. Session-level WR and coverage  
6. Calibration assessment  
7. Rejected-trade outcome analysis  
8. Selection concentration and redundancy  
9. Explicit 85% target status  
10. Existing Gate 2 criteria applied unchanged  

## Score definition (locked from 001)

- Features: `trend`, `momentum`, `price_location` (and empirical tables also include `side`, `hour_ist` for scoring mean)  
- Fit: on **discovery** only (2026-11-03…05), empirical \(P(\mathrm{win}\mid\mathrm{level})\) per feature level (min n=5)  
- Score: unweighted mean of available level probabilities for a trade  
- Selection: `score >= threshold`  
- Labels: existing net W/L (unchanged)  

## Holdout contamination note

001 evaluated multiple thresholds on the same holdout and then reported the best. That holdout is **no longer untouched validation** for threshold choice.  
002 reports the full frontier honestly and flags that **genuinely independent confirmation requires later untouched sessions**.

## Forbidden

- Retuning thresholds until 85% appears on the same holdout  
- Hour/symbol bans from holdout  
- Stop/target/OF changes  
- Book pooling · Gate 2 advance on coverage-thin slices  

## Artifacts

- `scripts/research/rp608_wr_entry_discovery_002.py`  
- `artifacts/rp608_econ_edge_diag/rp608_wr_entry_discovery_002.{md,json}`  
- Gate 2 memo: `docs/research/rp608_wr_entry_discovery_002_gate2.md`
