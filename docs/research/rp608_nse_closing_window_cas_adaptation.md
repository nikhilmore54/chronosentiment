# RP-608 NSE — Closing Window Adaptation (15:00–15:15)

**Status:** Active (shadow, 0.0% authority)  
**Parent:** `RP-608-NSE-SHADOW-001`  
**Context:** SEBI Closing Auction Session (CAS) era — F&O-eligible cash continuous trading ends **15:15 IST**; CAS ~**15:15–15:35**.

## Decision locked

Recommendations and simulated trades **adapt to the 15:00–15:15 closing window** rather than treating 15:30 as continuous liquidity.

| Rule | Value | Rationale |
|---|---|---|
| Closing window | 15:00–15:15 IST (bars 23–24) | Final continuous wind-down before halt/CAS |
| Mandatory square-off | **Bar 23 close (15:00 IST)** | Do not hold open MIS shadow through 15:00–15:15 / into CAS |
| Max new entry bar | **19** (~14:45 close) | 60m (4 bars) completes by bar 23 (15:00) |
| Estimand B eligible | Bars **1–19** | Forward T+4 endpoint ≤ 15:00 continuous reference |

## Non-goals

- Live order authority > 0%
- Modeling CAS equilibrium matching as a trade venue
- Splitting F&O vs non-F&O universes (119 book treated as F&O-liquid)

## Alignment

Deterministic SimEvent bar path; same inputs → same entry/exit decisions (`.cursor/rules/chronosentiment-core.mdc`).
