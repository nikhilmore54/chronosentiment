# Portfolio Replay v0.4 — Allocation Experiment: v04_A_25_equal

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** EqualWeight  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_A_25_equal`
- Universe size: 25 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: EqualWeight
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.39x | 12.29x |
| Lots opened | 275 | 550 |
| TARGET exits | 127 | 160 |
| STOP exits | 0 | 263 |
| HORIZON exits | 37 | 2 |
| Open at end | 111 | 119 |
| Avg hold (sessions) | 9.5 | 3.2 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +7.76% | +10.13% |
| Realized PnL | Rs.+100237.58 | Rs.+102040.32 |
| Unrealized PnL | Rs.-22654.96 | Rs.-746.11 |
| Max drawdown | Rs.21121.88 (1.92%) | Rs.27961.54 (2.48%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 263
- Premature: 3.8% (10/263)
- Temporary excursion: 32.3% (85/263)
- Stop too tight: 8.4% (22/263)
- Direction failure: 33.5% (88/263)
- Genuine adverse: 22.1% (58/263)
- Net stop benefit: Rs.-45206.30

