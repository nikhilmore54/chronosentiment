# Outcome Stratification

**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

Trading means use LONG+SHORT attached lake returns only. Missing SHORT stays missing.

| Dimension | Value | n | 60D n obs | 60D mean |
|---|---|---:|---:|---:|
| action | LONG | 110 | 110 | -0.009887 |
| action | SHORT | 85 | 0 | n/a |
| year | 2021 | 15 | 8 | -0.021608 |
| year | 2022 | 60 | 30 | -0.017052 |
| year | 2023 | 60 | 33 | 0.013573 |
| year | 2024 | 60 | 39 | -0.021822 |
| instrument | HDFCBANK.NS | 39 | 21 | -0.010012 |
| instrument | ICICIBANK.NS | 39 | 26 | -0.003443 |
| instrument | INFY.NS | 39 | 22 | -0.024279 |
| instrument | RELIANCE.NS | 39 | 20 | -0.005169 |
| instrument | TCS.NS | 39 | 21 | -0.007157 |
| trend | Bearish | 85 | 0 | n/a |
| trend | Bullish | 110 | 110 | -0.009887 |
| momentum | unlabeled | 195 | 110 | -0.009887 |
| action+trend | LONG+Bullish | 110 | 110 | -0.009887 |
| action+trend | SHORT+Bearish | 85 | 0 | n/a |
