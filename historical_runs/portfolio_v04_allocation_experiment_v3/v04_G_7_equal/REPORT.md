# Portfolio Replay v0.4 — Allocation Experiment: v04_G_7_equal

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** EqualWeight  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_G_7_equal`
- Universe size: 7 instruments
- Universe: HDFCBANK.NS, ICICIBANK.NS, INFY.NS, RELIANCE.NS, TCS.NS, IDEA.NS, MAHABANK.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: EqualWeight
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.41x | 11.17x |
| Lots opened | 84 | 161 |
| TARGET exits | 35 | 40 |
| STOP exits | 0 | 87 |
| HORIZON exits | 12 | 2 |
| Open at end | 37 | 32 |
| Avg hold (sessions) | 10.6 | 3.1 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +8.61% | +1.26% |
| Realized PnL | Rs.+87166.65 | Rs.+12819.85 |
| Unrealized PnL | Rs.-1017.87 | Rs.-170.27 |
| Max drawdown | Rs.18462.89 (1.68%) | Rs.27352.80 (2.63%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 87
- Premature: 8.0% (7/87)
- Temporary excursion: 32.2% (28/87)
- Stop too tight: 6.9% (6/87)
- Direction failure: 31.0% (27/87)
- Genuine adverse: 21.8% (19/87)
- Net stop benefit: Rs.-194353.78

