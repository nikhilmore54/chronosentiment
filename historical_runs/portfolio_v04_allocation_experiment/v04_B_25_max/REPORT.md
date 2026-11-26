# Portfolio Replay v0.4 — Allocation Experiment: v04_B_25_max

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** MaxPerSymbol Rs.20000  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_B_25_max`
- Universe size: 25 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: MaxPerSymbol Rs.20000
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 4.00x | 9.00x |
| Lots opened | 200 | 450 |
| TARGET exits | 105 | 159 |
| STOP exits | 0 | 228 |
| HORIZON exits | 37 | 2 |
| Open at end | 58 | 55 |
| Avg hold (sessions) | 10.3 | 3.3 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +6.41% | +4.03% |
| Realized PnL | Rs.+86573.39 | Rs.+41757.22 |
| Unrealized PnL | Rs.-22457.23 | Rs.-1421.21 |
| Max drawdown | Rs.20727.03 (1.91%) | Rs.33338.98 (3.11%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 228
- Premature: 4.4% (10/228)
- Temporary excursion: 35.5% (81/228)
- Stop too tight: 7.5% (17/228)
- Direction failure: 38.6% (88/228)
- Genuine adverse: 14.0% (32/228)
- Net stop benefit: Rs.-18364.95

