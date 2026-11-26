# Portfolio Replay v0.4 — Allocation Experiment: v04_D_50_max

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** MaxPerSymbol Rs.20000  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_D_50_max`
- Universe size: 50 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS, BAJAJFINSV.NS, JSWSTEEL.NS, TMPV.NS, ADANIENT.NS, ADANIPORTS.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, M&M.NS, SBILIFE.NS, TATACONSUM.NS, TATASTEEL.NS, UPL.NS, VEDL.NS, BPCL.NS, CIPLA.NS, HDFCLIFE.NS, PIDILITIND.NS, SHREECEM.NS, UNITDSPR.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: MaxPerSymbol Rs.20000
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.88x | 13.72x |
| Lots opened | 294 | 686 |
| TARGET exits | 149 | 281 |
| STOP exits | 0 | 339 |
| HORIZON exits | 74 | 2 |
| Open at end | 71 | 57 |
| Avg hold (sessions) | 12.0 | 3.6 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +5.70% | +10.38% |
| Realized PnL | Rs.+99618.48 | Rs.+99957.31 |
| Unrealized PnL | Rs.-42637.66 | Rs.+3879.21 |
| Max drawdown | Rs.34463.13 (3.16%) | Rs.23079.39 (2.05%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 339
- Premature: 5.0% (17/339)
- Temporary excursion: 37.2% (126/339)
- Stop too tight: 5.3% (18/339)
- Direction failure: 47.2% (160/339)
- Genuine adverse: 5.3% (18/339)
- Net stop benefit: Rs.+1666.22

