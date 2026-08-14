# Decision-vs-Baseline Analysis

**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

Stand-aside baseline is return 0. `mean_vs_stand_aside` is the attached mean minus 0.

SHORT with no lake rows cannot be judged. NO_TRADE with n=0 cannot be judged.

| Action | Horizon | n decisions | n obs | n missing | mean | vs stand-aside |
|---|---:|---:|---:|---:|---:|---:|
| Long | 5D | 110 | 110 | 0 | -0.002461 | -0.002461 |
| Short | 5D | 85 | 0 | 85 | n/a | n/a |
| NoTrade | 5D | 0 | 0 | 0 | n/a | n/a |
| Long | 10D | 110 | 110 | 0 | -0.001763 | -0.001763 |
| Short | 10D | 85 | 0 | 85 | n/a | n/a |
| NoTrade | 10D | 0 | 0 | 0 | n/a | n/a |
| Long | 20D | 110 | 110 | 0 | -0.005219 | -0.005219 |
| Short | 20D | 85 | 0 | 85 | n/a | n/a |
| NoTrade | 20D | 0 | 0 | 0 | n/a | n/a |
| Long | 60D | 110 | 110 | 0 | -0.009887 | -0.009887 |
| Short | 60D | 85 | 0 | 85 | n/a | n/a |
| NoTrade | 60D | 0 | 0 | 0 | n/a | n/a |
