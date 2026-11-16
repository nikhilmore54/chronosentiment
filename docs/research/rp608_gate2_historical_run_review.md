# RP-608 Gate 2 v2 — Historical Run Review

**Date:** 2026-11-15  
**Runner:** `scripts/research/rp608_gate2_historical_run.py`  
**Results:** `artifacts/rp608_gate2_historical/gate2_historical_run.md`  
**Authority:** 0% · Gate 5 not authorized  

## Summary

Gate 2 was executed on frozen historical journals under amendment v2 (**2E** + **2X**), books separate.

| Track | Book | Result |
| --- | --- | --- |
| 2E | Crypto plan | **NO_ADVANCE** on side/OF/ATR; hour filter WL-only weak |
| 2E | NSE A | **ADVANCE_CANDIDATE** — `BAR_ge_12` (holdout WR gap +3.7pp, payoff 1.34, net +₹3.9k) |
| 2X | Crypto plan | **ADVANCE_CANDIDATE** — G / H60 / H120 |
| 2X | NSE A | **DO_NOT_ADVANCE** — friction-horizon scratch |

## Authorized next (offline only)

1. Crypto **Gate 3X** design for **G** (then path-stress H*).  
2. NSE **Gate 3E** confound audit for `BAR_ge_12` (not daemon edit).  
3. Restore NSE bar history → 2X path-continue package.  

## Explicitly not authorized

Live stop/target/horizon/`traj15`/OF ±0.30 changes.
