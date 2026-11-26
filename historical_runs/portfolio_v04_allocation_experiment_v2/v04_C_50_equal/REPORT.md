# Portfolio Replay v0.4 — Allocation Experiment: v04_C_50_equal

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** EqualWeight  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_C_50_equal`
- Universe size: 52 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS, BAJAJFINSV.NS, JSWSTEEL.NS, TMPV.NS, ADANIENT.NS, ADANIPORTS.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, M&M.NS, SBILIFE.NS, TATACONSUM.NS, TATASTEEL.NS, UPL.NS, VEDL.NS, BPCL.NS, CIPLA.NS, HDFCLIFE.NS, PIDILITIND.NS, SHREECEM.NS, UNITDSPR.NS, MAHABANK.NS, IDEA.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: EqualWeight
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.12x | 12.93x |
| Lots opened | 572 | 1144 |
| TARGET exits | 270 | 329 |
| STOP exits | 0 | 574 |
| HORIZON exits | 80 | 3 |
| Open at end | 222 | 230 |
| Avg hold (sessions) | 9.9 | 3.2 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +4.39% | +8.09% |
| Realized PnL | Rs.+74411.09 | Rs.+77333.01 |
| Unrealized PnL | Rs.-30505.64 | Rs.+3548.23 |
| Max drawdown | Rs.25587.58 (2.39%) | Rs.20284.03 (1.84%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 574
- Premature: 4.2% (24/574)
- Temporary excursion: 32.4% (186/574)
- Stop too tight: 5.6% (32/574)
- Direction failure: 37.1% (213/574)
- Genuine adverse: 20.7% (119/574)
- Net stop benefit: Rs.-19609.00

