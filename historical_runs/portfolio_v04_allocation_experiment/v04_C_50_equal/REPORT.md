# Portfolio Replay v0.4 — Allocation Experiment: v04_C_50_equal

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** EqualWeight  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_C_50_equal`
- Universe size: 50 instruments
- Universe: RELIANCE.NS, TCS.NS, HDFCBANK.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, SBIN.NS, BHARTIARTL.NS, KOTAKBANK.NS, LT.NS, AXISBANK.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, ULTRACEMCO.NS, BAJFINANCE.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, TECHM.NS, HCLTECH.NS, ONGC.NS, BAJAJFINSV.NS, JSWSTEEL.NS, TMPV.NS, ADANIENT.NS, ADANIPORTS.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, M&M.NS, SBILIFE.NS, TATACONSUM.NS, TATASTEEL.NS, UPL.NS, VEDL.NS, BPCL.NS, CIPLA.NS, HDFCLIFE.NS, PIDILITIND.NS, SHREECEM.NS, UNITDSPR.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: EqualWeight
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.15x | 12.73x |
| Lots opened | 539 | 1078 |
| TARGET exits | 259 | 313 |
| STOP exits | 0 | 527 |
| HORIZON exits | 74 | 2 |
| Open at end | 206 | 229 |
| Avg hold (sessions) | 9.7 | 3.2 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +4.36% | +9.11% |
| Realized PnL | Rs.+76122.97 | Rs.+87318.42 |
| Unrealized PnL | Rs.-32540.64 | Rs.+3736.10 |
| Max drawdown | Rs.26675.30 (2.49%) | Rs.19980.99 (1.80%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 527
- Premature: 3.8% (20/527)
- Temporary excursion: 31.1% (164/527)
- Stop too tight: 5.7% (30/527)
- Direction failure: 37.4% (197/527)
- Genuine adverse: 22.0% (116/527)
- Net stop benefit: Rs.-2231.16

