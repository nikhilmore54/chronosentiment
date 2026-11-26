# Portfolio Replay v0.4 — Allocation Experiment: v04_H_7_max

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** MaxPerLot Rs.20000  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_H_7_max`
- Universe size: 7 instruments
- Universe: HDFCBANK.NS, ICICIBANK.NS, INFY.NS, RELIANCE.NS, TCS.NS, IDEA.NS, MAHABANK.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: MaxPerLot Rs.20000
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 2.10x | 3.22x |
| Lots opened | 105 | 161 |
| TARGET exits | 40 | 40 |
| STOP exits | 0 | 87 |
| HORIZON exits | 12 | 2 |
| Open at end | 53 | 32 |
| Avg hold (sessions) | 10.2 | 3.1 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +3.58% | -0.52% |
| Realized PnL | Rs.+34637.94 | Rs.-4901.10 |
| Unrealized PnL | Rs.+1201.61 | Rs.-292.86 |
| Max drawdown | Rs.8062.99 (0.77%) | Rs.12136.03 (1.21%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 87
- Premature: 8.0% (7/87)
- Temporary excursion: 32.2% (28/87)
- Stop too tight: 6.9% (6/87)
- Direction failure: 31.0% (27/87)
- Genuine adverse: 21.8% (19/87)
- Net stop benefit: Rs.-46919.69

