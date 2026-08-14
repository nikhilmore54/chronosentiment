# Regime / Context Analysis

**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

Labels are read from the assessment at T. They are not a second Decision Engine.

| Dimension | Value | LONG | SHORT | NO_TRADE |
|---|---|---:|---:|---:|
| confidence_status | Unavailable | 110 | 85 | 0 |
| mapping_rule | Trend.Bullish→LONG; Trend.Bearish→SHORT; Trend.other→NO_TRADE; Trend.absent→NO_TRADE | 110 | 85 | 0 |
| momentum | unlabeled | 110 | 85 | 0 |
| momentum_strength | unlabeled | 110 | 85 | 0 |
| trend | Bearish | 0 | 85 | 0 |
| trend | Bullish | 110 | 0 | 0 |
| trend_strength | Strong | 110 | 85 | 0 |
| volatility | unlabeled | 110 | 85 | 0 |

Volatility labels present on 0 of 195 decisions. Absence is not imputed.
