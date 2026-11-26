# Portfolio Replay v0.3 — Universe Robustness: v03_A_25

**Document type:** Product validation evidence  
**Experiment:** Universe robustness — frozen engine, variable universe  
**Contract:** Same engine/period/capital/C3-002/Coralys-v0/stop/allocation as v0.2.1  
**Only change:** Universe size  

## Setup

- Config label: `v03_A_25`
- Universe size: 25 instruments
- Universe: HDFCBANK.NS, RELIANCE.NS, TCS.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, KOTAKBANK.NS, AXISBANK.NS, SBIN.NS, BAJFINANCE.NS, BHARTIARTL.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, HCLTECH.NS, ULTRACEMCO.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, ONGC.NS, TMPV.NS, TATASTEEL.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000.00
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 4.63x | 10.35x |
| Lots opened | 125 | 300 |
| TARGET exits | 63 | 132 |
| STOP exits | 0 | 141 |
| HORIZON exits | 37 | 2 |
| Open at end | 25 | 19 |
| Avg hold (sessions) | 12.8 | 3.5 |

## Portfolio Performance

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Final portfolio value | Rs.5341.78 | Rs.5437.71 |
| Total return | +6.84% | +8.75% |
| Realized P&L | Rs.+420.25 | Rs.+435.06 |
| Max drawdown | 1.74% | 1.42% |

## Stop-Loss Behaviour

- Total Coralys STOP exits: 141
- Stop rate: 47.0% of lots
- GAP_THROUGH: 0 (0.0%)
- PREMATURE_STOP: 39 (27.7%)
- TEMPORARY_EXCURSION: 79 (56.0%)
- STOP_TOO_TIGHT: 2 (1.4%)
- DIRECTION_FAILURE: 18 (12.8%)
- GENUINE_ADVERSE: 3 (2.1%)
- Net stop benefit vs hold-to-horizon: Rs.-254.06

## Integrity

v0.2.1 continuous lifecycle -- multiple lots per instrument allowed, capital recycled on exit, session-by-session loop over full P.E.2 period. Canonical execution pipeline: generate_historical_replay_decision -> seal_execution_intent/seal_coralys_execution_intent -> first_exit_with_optional_stop. Execution arms frozen: P.E.2 = +5% target / no stop / 20s max; Coralys v0 = ATR/TMV target / risk_boundary stop / 20s max.
