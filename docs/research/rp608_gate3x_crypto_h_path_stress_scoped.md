# RP-608 — Gate 3X Crypto H* Path-Stress (next)

**Protocol:** `RP-608-GATE3X-CRYPTO-H-PATH-001`  
**Status:** REGISTERED · not started execution  
**Date:** 2026-11-15  
**Depends on:** Gate 5 G deployed  

## Why

H60/H120 journal scratch@entry was ADVANCE_CANDIDATE but **optimistic**. Path stress required before any Gate 5 for min-hold.

## Planned checks (offline)

1. For historical `STOP_LOSS_HIT` with hold &lt; 60s/120s: reconstruct whether price revisited entry ± friction before SL (needs tick/L1 or aggTrade marks).  
2. CF fills: scratch only if mid crosses entry after SL signal within window; else keep SL.  
3. Holdout Δnet + session robustness under conservative fill.  

## Non-goals

Deploy H* with G; change ATR stops; OF ±0.30.
