# Decision Behaviour Report

**Engine version: `unfrozen-dev`.** Not G-GATE. Not a v1.0 freeze. Not a strategy score. Not parameter tuning.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

| Field | Value |
|---|---|
| Decisions | `195` |
| LONG | `110` |
| SHORT | `85` |
| NO_TRADE | `0` |

## By instrument

| Instrument | LONG | SHORT | NO_TRADE |
|---|---:|---:|---:|
| HDFCBANK.NS | 21 | 18 | 0 |
| ICICIBANK.NS | 26 | 13 | 0 |
| INFY.NS | 22 | 17 | 0 |
| RELIANCE.NS | 20 | 19 | 0 |
| TCS.NS | 21 | 18 | 0 |

## By year

| Year | LONG | SHORT | NO_TRADE |
|---|---:|---:|---:|
| 2021 | 8 | 7 | 0 |
| 2022 | 30 | 30 | 0 |
| 2023 | 33 | 27 | 0 |
| 2024 | 39 | 21 | 0 |

## Confidence

| Confidence | n |
|---|---:|
| 0.8200 | 195 |

## Transitions (same instrument, consecutive as-of)

| From → To | n |
|---|---:|
| LONG → SHORT | 38 |
| SHORT → LONG | 38 |
| LONG → LONG | 68 |
| SHORT → SHORT | 46 |
| involving NO_TRADE | 0 |

## Streak lengths (consecutive same action, per instrument)

| Length | n streaks |
|---|---:|
| 1 | 28 |
| 2 | 24 |
| 3 | 11 |
| 4 | 10 |
| 5 | 4 |
| 6 | 3 |
| 8 | 1 |
