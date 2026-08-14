# Robustness Report

**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

Sign of trading mean by slice. This is not an optimizer and not G-GATE inference.

| Dimension | Value | Horizon | n obs | n missing | mean | sign |
|---|---|---:|---:|---:|---:|---:|
| year | 2021 | 5D | 8 | 7 | -0.008137 | − |
| year | 2021 | 10D | 8 | 7 | 0.003401 | + |
| year | 2021 | 20D | 8 | 7 | -0.011874 | − |
| year | 2021 | 60D | 8 | 7 | -0.021608 | − |
| year | 2022 | 5D | 30 | 30 | -0.004258 | − |
| year | 2022 | 10D | 30 | 30 | -0.005813 | − |
| year | 2022 | 20D | 30 | 30 | -0.011380 | − |
| year | 2022 | 60D | 30 | 30 | -0.017052 | − |
| year | 2023 | 5D | 33 | 27 | 0.001755 | + |
| year | 2023 | 10D | 33 | 27 | 0.002747 | + |
| year | 2023 | 20D | 33 | 27 | 0.009434 | + |
| year | 2023 | 60D | 33 | 27 | 0.013573 | + |
| year | 2024 | 5D | 39 | 21 | -0.003481 | − |
| year | 2024 | 10D | 39 | 21 | -0.003522 | − |
| year | 2024 | 20D | 39 | 21 | -0.011513 | − |
| year | 2024 | 60D | 39 | 21 | -0.021822 | − |
| instrument | HDFCBANK.NS | 5D | 21 | 18 | -0.003408 | − |
| instrument | HDFCBANK.NS | 10D | 21 | 18 | -0.008824 | − |
| instrument | HDFCBANK.NS | 20D | 21 | 18 | -0.007418 | − |
| instrument | HDFCBANK.NS | 60D | 21 | 18 | -0.010012 | − |
| instrument | ICICIBANK.NS | 5D | 26 | 13 | -0.001725 | − |
| instrument | ICICIBANK.NS | 10D | 26 | 13 | -0.001351 | − |
| instrument | ICICIBANK.NS | 20D | 26 | 13 | -0.006257 | − |
| instrument | ICICIBANK.NS | 60D | 26 | 13 | -0.003443 | − |
| instrument | INFY.NS | 5D | 22 | 17 | -0.001180 | − |
| instrument | INFY.NS | 10D | 22 | 17 | -0.001195 | − |
| instrument | INFY.NS | 20D | 22 | 17 | -0.006029 | − |
| instrument | INFY.NS | 60D | 22 | 17 | -0.024279 | − |
| instrument | RELIANCE.NS | 5D | 20 | 19 | -0.006439 | − |
| instrument | RELIANCE.NS | 10D | 20 | 19 | -0.001195 | − |
| instrument | RELIANCE.NS | 20D | 20 | 19 | -0.007790 | − |
| instrument | RELIANCE.NS | 60D | 20 | 19 | -0.005169 | − |
| instrument | TCS.NS | 5D | 21 | 18 | 0.000023 | + |
| instrument | TCS.NS | 10D | 21 | 18 | 0.003653 | + |
| instrument | TCS.NS | 20D | 21 | 18 | 0.001565 | + |
| instrument | TCS.NS | 60D | 21 | 18 | -0.007157 | − |
| trend | Bearish | 5D | 0 | 85 | n/a | n/a |
| trend | Bearish | 10D | 0 | 85 | n/a | n/a |
| trend | Bearish | 20D | 0 | 85 | n/a | n/a |
| trend | Bearish | 60D | 0 | 85 | n/a | n/a |
| trend | Bullish | 5D | 110 | 0 | -0.002461 | − |
| trend | Bullish | 10D | 110 | 0 | -0.001763 | − |
| trend | Bullish | 20D | 110 | 0 | -0.005219 | − |
| trend | Bullish | 60D | 110 | 0 | -0.009887 | − |
| momentum | unlabeled | 5D | 110 | 85 | -0.002461 | − |
| momentum | unlabeled | 10D | 110 | 85 | -0.001763 | − |
| momentum | unlabeled | 20D | 110 | 85 | -0.005219 | − |
| momentum | unlabeled | 60D | 110 | 85 | -0.009887 | − |
