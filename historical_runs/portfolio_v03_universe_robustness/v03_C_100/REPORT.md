# Portfolio Replay v0.3 — Universe Robustness: v03_C_100

**Document type:** Product validation evidence  
**Experiment:** Universe robustness — frozen engine, variable universe  
**Contract:** Same engine/period/capital/C3-002/Coralys-v0/stop/allocation as v0.2.1  
**Only change:** Universe size  

## Setup

- Config label: `v03_C_100`
- Universe size: 100 instruments
- Universe: HDFCBANK.NS, RELIANCE.NS, TCS.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, KOTAKBANK.NS, AXISBANK.NS, SBIN.NS, BAJFINANCE.NS, BHARTIARTL.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, HCLTECH.NS, ULTRACEMCO.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, ONGC.NS, TMPV.NS, TATASTEEL.NS, ADANIENT.NS, ADANIPORTS.NS, BAJAJFINSV.NS, BPCL.NS, BRITANNIA.NS, CIPLA.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, JSWSTEEL.NS, LT.NS, M&M.NS, PIDILITIND.NS, SBILIFE.NS, SHREECEM.NS, SIEMENS.NS, TECHM.NS, TRENT.NS, UPL.NS, VEDL.NS, ABCAPITAL.NS, ABFRL.NS, ACC.NS, AMBUJACEM.NS, APOLLOHOSP.NS, APOLLOTYRE.NS, AUROPHARMA.NS, BALKRISIND.NS, BANDHANBNK.NS, BANKBARODA.NS, BERGEPAINT.NS, BIOCON.NS, BOSCHLTD.NS, CANBK.NS, CHOLAFIN.NS, COLPAL.NS, CONCOR.NS, CUMMINSIND.NS, DABUR.NS, DLF.NS, ESCORTS.NS, EXIDEIND.NS, FEDERALBNK.NS, GAIL.NS, GODREJCP.NS, GODREJPROP.NS, HAVELLS.NS, HDFCAMC.NS, HDFCLIFE.NS, ICICIPRULI.NS, IDFCFIRSTB.NS, IGL.NS, INDUSTOWER.NS, IRCTC.NS, JUBLFOOD.NS, LICHSGFIN.NS, LUPIN.NS, MARICO.NS, UNITDSPR.NS, MFSL.NS, MPHASIS.NS, MRF.NS, MUTHOOTFIN.NS, NAUKRI.NS, NMDC.NS, PAGEIND.NS, PIIND.NS, PERSISTENT.NS, PFC.NS, PNB.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000.00
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | -0.00x | -0.00x |
| Lots opened | 0 | 0 |
| TARGET exits | 0 | 0 |
| STOP exits | 0 | 0 |
| HORIZON exits | 0 | 0 |
| Open at end | 0 | 0 |

## Portfolio Performance

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Final portfolio value | Rs.5000.00 | Rs.5000.00 |
| Total return | +0.00% | +0.00% |
| Realized P&L | Rs.+0.00 | Rs.+0.00 |
| Max drawdown | 0.00% | 0.00% |

## Stop-Loss Behaviour

- Total Coralys STOP exits: 0

## Integrity

v0.2.1 continuous lifecycle -- multiple lots per instrument allowed, capital recycled on exit, session-by-session loop over full P.E.2 period. Canonical execution pipeline: generate_historical_replay_decision -> seal_execution_intent/seal_coralys_execution_intent -> first_exit_with_optional_stop. Execution arms frozen: P.E.2 = +5% target / no stop / 20s max; Coralys v0 = ATR/TMV target / risk_boundary stop / 20s max.
