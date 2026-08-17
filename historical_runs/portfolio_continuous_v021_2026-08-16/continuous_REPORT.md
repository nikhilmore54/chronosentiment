# Portfolio Replay v0.2.1 — Continuous Lifecycle

**Document type:** Product validation evidence  
**Experiment:** Continuous portfolio lifecycle with position upgrades  
**Design:** Session-by-session loop, capital recycling, multiple lots per instrument  
**Does not:** modify C3-002, modify coralys-exec-v0, touch any prior archive  

## Setup

- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000.00
- Universe: HDFCBANK.NS, ICICIBANK.NS, INFY.NS, RELIANCE.NS, TCS.NS, IDEA.NS, MAHABANK.NS
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity Comparison

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.58x | 10.79x |
| Lots opened | 49 | 119 |
| TARGET exits | 24 | 34 |
| STOP exits | 0 | 70 |
| HORIZON exits | 12 | 2 |
| Open at end | 13 | 13 |
| Avg hold (sessions) | 11.8 | 3.3 |

## Portfolio Performance

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Final portfolio value | Rs.6172.78 | Rs.5074.59 |
| Total return | +23.46% | +1.49% |
| Realized P&L | Rs.+1195.55 | Rs.+73.86 |
| Unrealized P&L | Rs.-22.77 | Rs.+0.73 |
| Max drawdown | 1.62% (Rs.101.50) | 2.42% (Rs.125.36) |

## Integrity

v0.2.1 continuous lifecycle -- multiple lots per instrument allowed, capital recycled on exit, session-by-session loop over full P.E.2 period. Canonical execution pipeline: generate_historical_replay_decision -> seal_execution_intent/seal_coralys_execution_intent -> first_exit_with_optional_stop. Execution arms frozen: P.E.2 = +5% target / no stop / 20s max; Coralys v0 = ATR/TMV target / risk_boundary stop / 20s max.
