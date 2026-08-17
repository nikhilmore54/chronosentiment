# Portfolio Replay v0.2 — Horizon Matrix

Execution contracts: FROZEN throughout.
Purpose: characterize Coralys v0 position behaviour across evaluation horizons.
Do not optimize based on these results.

## Return Matrix

| Horizon | P.E.2 Return | Coralys Return | P.E.2 TARGET | Coralys TARGET | Coralys STOP | Coralys HORIZON | P.E.2 MaxDD | Coralys MaxDD | P.E.2 AvgHold | Coralys AvgHold |
|---------|-------------|----------------|-------------|----------------|-------------|-----------------|------------|--------------|--------------|----------------|
| 5s | 1.18% | 0.89% | 3 | 4 | 2 | 1 | 1.51% | 0.61% | 15.0s | 4.7s |
| 10s | 1.18% | 0.89% | 3 | 4 | 2 | 1 | 1.51% | 0.61% | 15.0s | 4.7s |
| 15s | 1.18% | 0.89% | 3 | 4 | 2 | 1 | 1.51% | 0.61% | 15.0s | 4.7s |
| 20s | 1.18% | 0.89% | 3 | 4 | 2 | 1 | 1.51% | 0.61% | 15.0s | 4.7s |

## Notes

- All results are exploratory (n=7 instruments, single historical window).
- Execution horizon (20 sessions max hold) is unchanged across all rows.
- Evaluation horizon controls how many sessions are observed for exit scanning.
- HORIZON count = positions that reached the evaluation horizon without TARGET or STOP.
- Coralys stop is enforced in all rows (stop_authorized=true).
