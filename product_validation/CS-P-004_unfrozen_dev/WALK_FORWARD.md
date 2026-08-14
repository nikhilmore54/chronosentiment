# Walk-Forward Historical Analysis

**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

No fitting. Train describes the same `unfrozen-dev` policy. Test measures it on a later unseen year.

| Fold | Train n | Train LONG/SHORT/NO_TRADE | Test n | Test 60D n obs | Test 60D mean |
|---|---:|---|---:|---:|---:|
| test_2022 | 15 | 8/7/0 | 60 | 30 | -0.017052 |
| test_2023 | 75 | 38/37/0 | 60 | 33 | 0.013573 |
| test_2024 | 135 | 71/64/0 | 60 | 39 | -0.021822 |

Test `as_of` is always `>= test_start` and `< test_end`, and `test_start == train_end`.
