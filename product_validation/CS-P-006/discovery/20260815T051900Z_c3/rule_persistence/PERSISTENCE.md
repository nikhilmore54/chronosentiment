# CS-P-006-C.3-E — Search #2 discovered-rule persistence

Sealed Search #2 only. Candidate research artifact. Not promoted. No pass/fail threshold. Search #3 is not authorized.

- artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- promotion_status: `candidate_research_artifact`
- pass_threshold_introduced: false
- rows: 273

## Sample size and contribution

| Live state | Action | n | Eval n | Mean V | Eval V | Value share | Eval share |
|---|---|---:|---:|---:|---:|---:|---:|
| Bearish → LONG | Long | 121 | 33 | 0.6361% | -1.6064% | 48.5% | 196.5% |
| Bullish ∧ Positive Momentum → LONG | Long | 109 | 42 | 0.5578% | 0.1905% | 38.3% | -29.7% |
| Bullish ∧ Negative Momentum → SHORT | Short | 43 | 16 | 0.4894% | 1.1272% | 13.3% | -66.9% |

Calendar windows 2021–22 / 2022–23 / 2023–24 overlap on the shared year. They are persistence views, not a second partition.

## Bearish → LONG

| Window | n | Mean V | Median V | +/− | vs NO_TRADE |
|---|---:|---:|---:|---:|---:|
| 2021–22 | 53 | 1.0470% | 1.5946% | 30/23 | 1.0470% |
| 2022–23 | 79 | 1.0748% | 0.9575% | 45/34 | 1.0748% |
| 2023–24 | 68 | 0.3158% | 0.9574% | 39/29 | 0.3158% |
| development | 49 | 1.4039% | 1.7850% | 29/20 | 1.4039% |
| selection | 39 | 1.5688% | 1.1744% | 23/16 | 1.5688% |
| evaluation | 33 | -1.6064% | 0.7150% | 17/16 | -1.6064% |

Fired states:

| Trend | Momentum | Volatility | n | Mean V | Eval n | Eval V |
|---|---|---|---:|---:|---:|---:|
| Bearish | Negative | present | 84 | 0.0485% | 24 | -2.6419% |
| Bearish | Positive | present | 37 | 1.9699% | 9 | 1.1548% |

Action advantage (all / evaluation): recommended 0.6361% / -1.6064%; LONG 0.6361% / -1.6064%; SHORT -0.6361% / 1.6064%.
Losses cluster: n_loss=52 loss_sum=-285.5692% largest_instrument=IDEA.NS largest_year=2024.

## Bullish ∧ Positive Momentum → LONG

| Window | n | Mean V | Median V | +/− | vs NO_TRADE |
|---|---:|---:|---:|---:|---:|
| 2021–22 | 38 | -0.5834% | 0.7746% | 21/17 | -0.5834% |
| 2022–23 | 66 | 1.3535% | 1.7275% | 40/26 | 1.3535% |
| 2023–24 | 71 | 1.1686% | 0.2382% | 36/35 | 1.1686% |
| development | 30 | -0.5565% | 1.5275% | 17/13 | -0.5565% |
| selection | 37 | 1.8783% | 1.4253% | 21/16 | 1.8783% |
| evaluation | 42 | 0.1905% | -0.3630% | 19/23 | 0.1905% |

Fired states:

| Trend | Momentum | Volatility | n | Mean V | Eval n | Eval V |
|---|---|---|---:|---:|---:|---:|
| Bullish | Positive | present | 109 | 0.5578% | 42 | 0.1905% |

Action advantage (all / evaluation): recommended 0.5578% / 0.1905%; LONG 0.5578% / 0.1905%; SHORT -0.5578% / -0.1905%.
Losses cluster: n_loss=52 loss_sum=-268.7681% largest_instrument=MAHABANK.NS largest_year=2022.

## Bullish ∧ Negative Momentum → SHORT

| Window | n | Mean V | Median V | +/− | vs NO_TRADE |
|---|---:|---:|---:|---:|---:|
| 2021–22 | 14 | -0.4572% | 0.6792% | 8/6 | -0.4572% |
| 2022–23 | 23 | 0.4147% | 1.1364% | 13/10 | 0.4147% |
| 2023–24 | 29 | 0.9464% | 1.2183% | 18/11 | 0.9464% |
| development | 12 | -0.3124% | 0.6792% | 7/5 | -0.3124% |
| selection | 15 | 0.4506% | 1.2597% | 8/7 | 0.4506% |
| evaluation | 16 | 1.1272% | 0.7601% | 11/5 | 1.1272% |

Fired states:

| Trend | Momentum | Volatility | n | Mean V | Eval n | Eval V |
|---|---|---|---:|---:|---:|---:|
| Bullish | Negative | present | 43 | 0.4894% | 16 | 1.1272% |

Action advantage (all / evaluation): recommended 0.4894% / 1.1272%; LONG -0.4894% / -1.1272%; SHORT 0.4894% / 1.1272%.
Losses cluster: n_loss=17 loss_sum=-76.9002% largest_instrument=MAHABANK.NS largest_year=2023.

Unique-best shares are diagnostics over this sample, not confidence. No threshold decides whether a rule persists. Search #3 is not authorized.
