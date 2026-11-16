# RP-608 — Gate 3E NSE A `BAR_ge_12` Confound Audit

**Protocol:** `RP-608-GATE3E-NSE-BAR-GE12-001`  
**Parent:** Gate 2E ADVANCE_CANDIDATE (`BAR_ge_12`) · Gate 2 v2  
**Date:** 2026-11-15  
**Authority:** 0% · **no Estimand A entry threshold / bar cutoff change** until Gate 5  

## Hypothesis under audit

Preferred cohort: `entry_bar_idx >= 12` has higher holdout WR and positive net vs complement.  
**Risk:** late-bar filter is a **session-boundary confound** (less path left, different stop/square-off mix), not transferable entry quality.

## Required falsification checks

1. Within-session bar curve (WR/net by entry_bar).  
2. Exit-reason mix early vs late.  
3. Holdout-only recompute (Gate 2 split).  
4. Same-day same-symbol pairs when available.  
5. Distance-to-square-off (23 − entry_bar).  

## Labels

| Label | Meaning |
| --- | --- |
| `CONFOUND_DOMINANT` | Timing/exit-mix explains the edge → do not promote |
| `EDGE_SURVIVES_AUDIT` | Edge persists after strata → Gate 5 packet eligible to draft |
| `INCONCLUSIVE` | Underpowered strata |

## Explicit non-goals

Hour bans; symbol bans; live `traj15` change; pooling books.
