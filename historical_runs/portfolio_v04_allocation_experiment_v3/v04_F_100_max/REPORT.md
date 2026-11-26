# Portfolio Replay v0.4 — Allocation Experiment: v04_F_100_max

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** MaxPerLot Rs.20000  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_F_100_max`
- Universe size: 103 instruments
- Universe: HDFCBANK.NS, RELIANCE.NS, TCS.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, KOTAKBANK.NS, AXISBANK.NS, SBIN.NS, BAJFINANCE.NS, BHARTIARTL.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, HCLTECH.NS, ULTRACEMCO.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, ONGC.NS, TMPV.NS, TATASTEEL.NS, ADANIENT.NS, ADANIPORTS.NS, BAJAJFINSV.NS, BPCL.NS, BRITANNIA.NS, CIPLA.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, JSWSTEEL.NS, LT.NS, M&M.NS, PIDILITIND.NS, SBILIFE.NS, SHREECEM.NS, SIEMENS.NS, TECHM.NS, TRENT.NS, UPL.NS, VEDL.NS, ABCAPITAL.NS, ABFRL.NS, ACC.NS, AMBUJACEM.NS, APOLLOHOSP.NS, APOLLOTYRE.NS, AUROPHARMA.NS, BALKRISIND.NS, BANDHANBNK.NS, BANKBARODA.NS, BERGEPAINT.NS, BIOCON.NS, BOSCHLTD.NS, CANBK.NS, CHOLAFIN.NS, COLPAL.NS, CONCOR.NS, CUMMINSIND.NS, DABUR.NS, DLF.NS, ESCORTS.NS, EXIDEIND.NS, FEDERALBNK.NS, GAIL.NS, GODREJCP.NS, GODREJPROP.NS, HAVELLS.NS, HDFCAMC.NS, HDFCLIFE.NS, ICICIPRULI.NS, IDFCFIRSTB.NS, IGL.NS, INDUSTOWER.NS, IRCTC.NS, JUBLFOOD.NS, LICHSGFIN.NS, LUPIN.NS, MARICO.NS, UNITDSPR.NS, MFSL.NS, MPHASIS.NS, MRF.NS, MUTHOOTFIN.NS, NAUKRI.NS, NMDC.NS, PAGEIND.NS, PIIND.NS, PERSISTENT.NS, PFC.NS, PNB.NS, TATACONSUM.NS, MAHABANK.NS, IDEA.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: MaxPerLot Rs.20000
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 8.24x | 24.72x |
| Lots opened | 412 | 1236 |
| TARGET exits | 197 | 452 |
| STOP exits | 0 | 702 |
| HORIZON exits | 158 | 7 |
| Open at end | 57 | 65 |
| Avg hold (sessions) | 13.7 | 4.0 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +3.41% | +10.66% |
| Realized PnL | Rs.+75260.02 | Rs.+92567.88 |
| Unrealized PnL | Rs.-41208.06 | Rs.+14024.46 |
| Max drawdown | Rs.42930.53 (3.99%) | Rs.19047.18 (1.70%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 702
- Premature: 6.7% (47/702)
- Temporary excursion: 44.4% (312/702)
- Stop too tight: 4.0% (28/702)
- Direction failure: 39.3% (276/702)
- Genuine adverse: 5.6% (39/702)
- Net stop benefit: Rs.-139040.52

