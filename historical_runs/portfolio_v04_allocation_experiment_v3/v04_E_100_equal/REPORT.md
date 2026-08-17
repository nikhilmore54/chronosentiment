# Portfolio Replay v0.4 — Allocation Experiment: v04_E_100_equal

**Document type:** Product validation evidence  
**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  
**Allocation:** EqualWeight  
**Initial capital:** Rs.1000000  

## Setup

- Config label: `v04_E_100_equal`
- Universe size: 103 instruments
- Universe: HDFCBANK.NS, RELIANCE.NS, TCS.NS, INFY.NS, ICICIBANK.NS, HINDUNILVR.NS, ITC.NS, KOTAKBANK.NS, AXISBANK.NS, SBIN.NS, BAJFINANCE.NS, BHARTIARTL.NS, ASIANPAINT.NS, MARUTI.NS, TITAN.NS, SUNPHARMA.NS, WIPRO.NS, HCLTECH.NS, ULTRACEMCO.NS, NESTLEIND.NS, POWERGRID.NS, NTPC.NS, ONGC.NS, TMPV.NS, TATASTEEL.NS, ADANIENT.NS, ADANIPORTS.NS, BAJAJFINSV.NS, BPCL.NS, BRITANNIA.NS, CIPLA.NS, COALINDIA.NS, DIVISLAB.NS, DRREDDY.NS, EICHERMOT.NS, GRASIM.NS, HEROMOTOCO.NS, HINDALCO.NS, INDUSINDBK.NS, JSWSTEEL.NS, LT.NS, M&M.NS, PIDILITIND.NS, SBILIFE.NS, SHREECEM.NS, SIEMENS.NS, TECHM.NS, TRENT.NS, UPL.NS, VEDL.NS, ABCAPITAL.NS, ABFRL.NS, ACC.NS, AMBUJACEM.NS, APOLLOHOSP.NS, APOLLOTYRE.NS, AUROPHARMA.NS, BALKRISIND.NS, BANDHANBNK.NS, BANKBARODA.NS, BERGEPAINT.NS, BIOCON.NS, BOSCHLTD.NS, CANBK.NS, CHOLAFIN.NS, COLPAL.NS, CONCOR.NS, CUMMINSIND.NS, DABUR.NS, DLF.NS, ESCORTS.NS, EXIDEIND.NS, FEDERALBNK.NS, GAIL.NS, GODREJCP.NS, GODREJPROP.NS, HAVELLS.NS, HDFCAMC.NS, HDFCLIFE.NS, ICICIPRULI.NS, IDFCFIRSTB.NS, IGL.NS, INDUSTOWER.NS, IRCTC.NS, JUBLFOOD.NS, LICHSGFIN.NS, LUPIN.NS, MARICO.NS, UNITDSPR.NS, MFSL.NS, MPHASIS.NS, MRF.NS, MUTHOOTFIN.NS, NAUKRI.NS, NMDC.NS, PAGEIND.NS, PIIND.NS, PERSISTENT.NS, PFC.NS, PNB.NS, TATACONSUM.NS, MAHABANK.NS, IDEA.NS
- Certified T: 2026-07-15T03:45:00+00:00
- Sessions simulated: 23
- Initial capital: Rs.5000
- Allocation model: EqualWeight
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## Capital Velocity

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Capital velocity | 5.03x | 12.77x |
| Lots opened | 1030 | 2266 |
| TARGET exits | 506 | 617 |
| STOP exits | 0 | 1180 |
| HORIZON exits | 158 | 7 |
| Open at end | 366 | 449 |
| Avg hold (sessions) | 10.3 | 3.4 |

## Returns

| Metric | P.E.2 | Coralys v0 |
|--------|-------|------------|
| Total return | +2.92% | +4.06% |
| Realized PnL | Rs.+63547.75 | Rs.+32075.05 |
| Unrealized PnL | Rs.-34385.87 | Rs.+8475.17 |
| Max drawdown | Rs.26155.01 (2.48%) | Rs.12165.93 (1.16%) |

## Stop-Loss Analysis (Coralys arm)

- Total stops: 1180
- Premature: 4.7% (56/1180)
- Temporary excursion: 37.5% (443/1180)
- Stop too tight: 3.6% (42/1180)
- Direction failure: 33.8% (399/1180)
- Genuine adverse: 20.3% (240/1180)
- Net stop benefit: Rs.-57558.06

