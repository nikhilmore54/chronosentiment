# Portfolio Replay v0.4 — Allocation Experiment: v04_A_25_equal

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** EqualWeight  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_A_25_equal`
- Universe size: 27 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS, MAHABANK.NS, IDEA.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: EqualWeight
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.39x | 12.58x |
| Lots opened | 297 | 594 |
| TARGET exits | 138 | 171 |
| STOP exits | 0 | 295 |
| HORIZON exits | 40 | 3 |
| Open at end | 119 | 119 |
| Avg hold (sessions) | 9.8 | 3.2 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +8.10% | +8.67% |
| Realized PnL | Rs.+101498.14 | Rs.+87563.40 |
| Unrealized PnL | Rs.-20528.44 | Rs.-874.59 |
| Max drawdown | Rs.18803.34 (1.71%) | Rs.28147.03 (2.52%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 295
- Premature: 4.7% (14/295)
- Temporary excursion: 32.5% (96/295)
- Stop too tight: 7.5% (22/295)
- Direction failure: 34.9% (103/295)
- Genuine adverse: 20.3% (60/295)
- Net stop benefit: Rs.-74943.78

