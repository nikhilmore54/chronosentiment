# Portfolio Replay v0.4 — Allocation Experiment: v04_D_50_max

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** MaxPerLot Rs.20000  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_D_50_max`
- Universe size: 52 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS, BAJAJFINSV.NS, JSWSTEEL.NS, TMPV.NS, ADANIENT.NS, ADANIPORTS.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, M&M.NS, SBILIFE.NS, TATACONSUM.NS, TATASTEEL.NS, UPL.NS, VEDL.NS, BPCL.NS, CIPLA.NS, HDFCLIFE.NS, PIDILITIND.NS, SHREECEM.NS, UNITDSPR.NS, MAHABANK.NS, IDEA.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: MaxPerLot Rs.20000
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 6.24x | 14.56x |
| Lots opened | 312 | 728 |
| TARGET exits | 155 | 288 |
| STOP exits | 0 | 372 |
| HORIZON exits | 80 | 3 |
| Open at end | 77 | 57 |
| Avg hold (sessions) | 12.1 | 3.6 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +6.15% | +9.79% |
| Realized PnL | Rs.+104012.74 | Rs.+94017.70 |
| Unrealized PnL | Rs.-42481.76 | Rs.+3879.21 |
| Max drawdown | Rs.34572.12 (3.15%) | Rs.23777.37 (2.12%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 372
- Premature: 4.6% (17/372)
- Temporary excursion: 37.9% (141/372)
- Stop too tight: 5.4% (20/372)
- Direction failure: 47.3% (176/372)
- Genuine adverse: 4.8% (18/372)
- Net stop benefit: Rs.-20138.66

