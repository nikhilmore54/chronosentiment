# RP-608 NSE Coralys Input Gate V6 — trend × time-of-action × pre-open jitter

**Date:** 2026-11-20  
**Book:** NSE Estimand A only  
**Authority:** 0% · adapter gate · Gate 5 baseline frozen (`traj15≥0.05`, stops/targets, OF ±0.30)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc`

## Why

V5 stopped missing-preopen fail-open and same-symbol churn. Remaining hole: **when preopen exists**, OPEN/early prints can still be auction **jitter** (gap/imbalance noise) or **trend fight** (long into gap-down). Historical operator note: constructive direction often appears in **09:20–09:45**, not in the open jitter band.

## Time-of-action (IST)

| Phase | Clock | Policy |
|-------|-------|--------|
| `OPEN` | ≤09:15 | No turbulence/jitter; gap≥+20 aligned; traj≥0.15; prior watch |
| `EARLY_JITTER` | 09:15–09:20 | Settle hold — opposed gap blocked; need traj≥0.18 + align + prior |
| `EARLY` | 09:20–09:45 | Constructive — traj≥0.12; opposed gap needs traj≥0.18; turbulent needs ≥0.18 |
| `MID` / `LATE` | as V5 | Continuous floors (LATE ≥0.15) |

## Pre-open jitter (LONG)

- **Full turbulence** in OPEN / EARLY_JITTER → **hard standaside** (no traj escape)
- **Medium jitter:** gap-down ≤−35 bps; gap-up with ask-heavy; gap/imbalance sign fight  
  → standaside in OPEN / EARLY_JITTER  
- Clean constructive gap-up + supportive bid is **not** jitter

## Retained from V5

- Preopen `MISSING` → always stand aside  
- Daemon: max 1 entry/symbol/day; block after `STOP_LOSS`  
