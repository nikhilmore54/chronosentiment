# Portfolio Replay v0.3 — Universe Robustness: v03_B_50

**Document type:** Product validation evidence  
**Experiment:** Universe robustness — frozen engine, variable universe  
**Contract:** Same engine/period/capital/C3-002/Coralys-v0/stop/allocation as v0.2.1  
**Only change:** Universe size  

## Setup

- Config label: `v03_B_50`
- Universe size: 50 instruments
- Universe: HDFCBANK.NS, RELIANCE.NS, TCS.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, KOTAKBANK.NS, AXISBANK.NS, SBIN.NS, BAJFINANCE.NS, BHARTIARTL.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, HCLTECH.NS, ULTRACEMCO.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, ONGC.NS, TMPV.NS, TATASTEEL.NS, ADANIENT.NS, ADANIPORTS.NS, BAJAJFINSV.NS, BPCL.NS, BRITANNIA.NS, CIPLA.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, JSWSTEEL.NS, LT.NS, M&M.NS, PIDILITIND.NS, SBILIFE.NS, SHREECEM.NS, SIEMENS.NS, TECHM.NS, TRENT.NS, UPL.NS, VEDL.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000.00
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 4.05x | 4.07x |
| Lots opened | 200 | 200 |
| TARGET exits | 99 | 89 |
| STOP exits | 0 | 104 |
| HORIZON exits | 74 | 2 |
| Open at end | 27 | 3 |
| Avg hold (sessions) | 13.4 | 4.7 |

## Portfolio Performance

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Final portfolio value | Rs.5155.77 | Rs.5179.54 |
| Total return | +3.12% | +3.59% |
| Realized P&L | Rs.+244.77 | Rs.+173.37 |
| Max drawdown | 1.77% | 0.06% |

## Stop-Loss Behaviour

- Total Coralys STOP exits: 104
- Stop rate: 52.0% of lots
- GAP_THROUGH: 0 (0.0%)
- PREMATURE_STOP: 36 (34.6%)
- TEMPORARY_EXCURSION: 49 (47.1%)
- STOP_TOO_TIGHT: 1 (1.0%)
- DIRECTION_FAILURE: 18 (17.3%)
- GENUINE_ADVERSE: 0 (0.0%)
- Net stop benefit vs hold-to-horizon: Rs.+28.17

## Integrity

v0.2.1 continuous lifecycle -- multiple lots per instrument allowed, capital recycled on exit, session-by-session loop over full P.E.2 period. Canonical execution pipeline: generate_historical_replay_decision -> seal_execution_intent/seal_coralys_execution_intent -> first_exit_with_optional_stop. Execution arms frozen: P.E.2 = +5% target / no stop / 20s max; Coralys v0 = ATR/TMV target / risk_boundary stop / 20s max.
