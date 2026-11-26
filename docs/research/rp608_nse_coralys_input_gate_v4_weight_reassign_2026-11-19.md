# RP-608 NSE Coralys Input Gate — V4 weight reassignment (2026-11-19)

**Book:** NSE Estimand A only (never pooled)  
**Authority:** 0% · adapter gate only · Estimand A `traj15≥0.05` / OF ±0.30 / stops-targets **unchanged** (Gate 5 not opened)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` · W/L-first · payoff+net disclosed

## Effect of V3 weights (POST 16–19 Nov, closed A)

| Cohort | n | W/L | WR | net (INR) |
|--------|---|-----|----|-----------|
| All POST | 2003 | 272/1731 | 13.6% | −1,035,493 |
| `PASS_PREOPEN_MISSING_FAIL_OPEN` | 1329 | ~72/1257 | **5.4%** | **−873,324** |
| `PASS_CONTINUOUS` (preopen OK) | 535 | — | **27.9%** | −118,031 |
| Preopen OK trades | 544 | — | 28.1% | −119,670 |
| Preopen MISSING trades | 1335 | — | 5.5% | −875,832 |

Uniform traj-floor raises (0.08→0.30) barely moved WR (~13%→10%) — losers were often **high traj** under fail-open.  
Counterfactual **no fail-open**: keep 34%, WR **29.7%**, net −162k.

## Reassigned weights (V4)

| Lever | V3 | V4 |
|-------|----|----|
| Preopen MISSING | fail-open (`PASS_PREOPEN_MISSING_FAIL_OPEN`) | stand aside unless `prior_support` ∧ `traj≥0.20` |
| `CONTINUOUS_TRAJ_FLOOR` | 0.08 | **0.10** |
| `CONTINUOUS_TRAJ_FLOOR_NONPOSITIVE_GAP` | 0.10 | **0.12** |
| `CONTINUOUS_TRAJ_FLOOR_OPPOSED_GAP` | 0.12 | **0.15** |
| `LATE_TRAJ_FLOOR` | (same as continuous) | **0.12** |
| `OPEN_TRAJ_FLOOR` | none | **0.12** |
| Next-session `SESSION_RETURN_WEIGHT` | 0.25 | **0.40** |

## Explicit non-goals

- No Gate 5 change to frozen Estimand A entry threshold / stops / OF ±0.30  
- No OF / OFX authority  
- No book pooling  
