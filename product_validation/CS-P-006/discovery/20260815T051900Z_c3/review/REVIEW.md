# CS-P-006-C.3-C — Search #1 vs Search #2

Sealed-artifact review only. Neither policy was modified. Search #3 is not authorized.

- Search #1: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`
- Search #2: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- rows: 273

## Pairwise on the same 273 rows

| Slice | n | Search #2 better | Search #1 better | Tie | Mean ΔV |
|---|---:|---:|---:|---:|---:|
| all | 273 | 83 | 69 | 121 | 0.002998 |
| evaluation | 91 | 30 | 28 | 33 | 0.002861 |

## Evaluation value by symbol

| Symbol | Search #1 V | Search #2 V | ΔV |
|---|---:|---:|---:|
| HDFCBANK.NS | 1.0282% | 0.5846% | -0.4436% |
| ICICIBANK.NS | 0.5801% | 1.0004% | +0.4203% |
| INFY.NS | 1.0375% | 1.7588% | +0.7213% |
| RELIANCE.NS | -1.1914% | -0.9855% | +0.2059% |
| TCS.NS | 1.1222% | 1.5587% | +0.4365% |
| IDEA.NS | -4.3404% | -4.9173% | -0.5769% |
| MAHABANK.NS | -2.3140% | -1.0749% | +1.2392% |

## NO_TRADE conversion

Search #1 stood aside on 152 rows. Search #2 converted 109 to LONG and 43 to SHORT; 0 remained NO_TRADE.
On those rows, mean Search #2 V = 0.005385; Search #2 unique-best 83 / 152; Search #2 better than Search #1 83 / 152.
Trend Neutral appears on 0 of 273 rows.

## Search #2 first-match rules

| # | Action | Concepts | Fired | Reachable | Contradictory | Shadowed |
|---|---|---|---:|---|---|---|
| 0 | Long | Trend | 121 | true | false | false |
| 1 | Long | Momentum+Trend+Volatility | 109 | true | false | false |
| 2 | NoTrade | Trend | 0 | false | false | true |
| 3 | Short | Trend | 43 | true | false | false |
| 4 | NoTrade | Momentum+Volatility+Volatility | 0 | false | true | false |
| 5 | Long | Trend+Volatility | 0 | false | false | true |
| 6 | Long | Volatility | 0 | false | false | true |

Unmatched action: Long. Unmatched fires: 0. Search #3 is not authorized. Unique-best is diagnostic, not fitness.
