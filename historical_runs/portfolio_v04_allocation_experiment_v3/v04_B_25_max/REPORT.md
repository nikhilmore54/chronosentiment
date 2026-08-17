# Portfolio Replay v0.4 — Allocation Experiment: v04_B_25_max

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** MaxPerLot Rs.20000  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_B_25_max`
- Universe size: 27 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS, MAHABANK.NS, IDEA.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: MaxPerLot Rs.20000
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 4.32x | 9.72x |
| Lots opened | 216 | 486 |
| TARGET exits | 113 | 165 |
| STOP exits | 0 | 257 |
| HORIZON exits | 40 | 3 |
| Open at end | 63 | 55 |
| Avg hold (sessions) | 10.5 | 3.4 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +7.16% | +3.51% |
| Realized PnL | Rs.+93532.90 | Rs.+36516.60 |
| Unrealized PnL | Rs.-21949.26 | Rs.-1421.21 |
| Max drawdown | Rs.19202.37 (1.76%) | Rs.34273.32 (3.21%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 257
- Premature: 5.1% (13/257)
- Temporary excursion: 35.8% (92/257)
- Stop too tight: 6.6% (17/257)
- Direction failure: 40.1% (103/257)
- Genuine adverse: 12.5% (32/257)
- Net stop benefit: Rs.-47632.25

