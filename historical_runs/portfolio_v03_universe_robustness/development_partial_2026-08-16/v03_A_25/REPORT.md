# Portfolio Replay v0.3 — Universe Robustness: v03_A_25

**Document type:** Product validation evidence  
**Experiment:** Universe robustness — frozen engine, variable universe  
**Contract:** Same engine/period/capital/C3-002/Coralys-v0/stop/allocation as v0.2.1  
**Only change:** Universe size  

## Setup

- Config label: `v03_A_25`
- Universe size: 5 instruments
- Universe: HDFCBANK.NS, RELIANCE.NS, TCS.NS, INFY.NS, ICICIBANK.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000.00
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.35x | 9.44x |
| Lots opened | 40 | 75 |
| TARGET exits | 19 | 28 |
| STOP exits | 0 | 36 |
| HORIZON exits | 9 | 1 |
| Open at end | 12 | 10 |
| Avg hold (sessions) | 10.8 | 3.1 |

## Portfolio Performance

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Final portfolio value | Rs.5356.45 | Rs.5243.63 |
| Total return | +7.13% | +4.87% |
| Realized P&L | Rs.+381.96 | Rs.+244.03 |
| Max drawdown | 1.68% | 2.69% |

## Stop-Loss Behaviour

- Total Coralys STOP exits: 36
- Stop rate: 48.0% of lots
- GAP_THROUGH: 0 (0.0%)
- PREMATURE_STOP: 8 (22.2%)
- TEMPORARY_EXCURSION: 21 (58.3%)
- STOP_TOO_TIGHT: 2 (5.6%)
- DIRECTION_FAILURE: 4 (11.1%)
- GENUINE_ADVERSE: 1 (2.8%)
- Net stop benefit vs hold-to-horizon: Rs.-576.03

## Integrity

v0.2.1 continuous lifecycle -- multiple lots per instrument allowed, capital recycled on exit, session-by-session loop over full P.E.2 period. Canonical execution pipeline: generate_historical_replay_decision -> seal_execution_intent/seal_coralys_execution_intent -> first_exit_with_optional_stop. Execution arms frozen: P.E.2 = +5% target / no stop / 20s max; Coralys v0 = ATR/TMV target / risk_boundary stop / 20s max.
