# B4 Historical Product Validation

**Engine version: `unfrozen-dev`.** Decision Engine v1.0 is **not frozen**. This is a product-validation baseline, not a production trading strategy, not a strategy score, and not G-GATE v1.1.

Parent: CS-P-002. Does not reopen EV-GOV-003.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no randomness in strategy logic; this run does not change decision rules.

## Identity

| Field | Value |
|---|---|
| Kind | `product_validation` |
| Decision engine version | `unfrozen-dev` |
| Decision Engine v1.0 frozen | `no` |
| B4 dump SHA-256 | `f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6` |
| Disposable database | `chrono_replay_b4_historical` |
| Git HEAD | `668350e47d002476626cd6f945da85da26d0bd80` |
| Working tree dirty | `yes` |
| Ledger identity hash | `091f241e6557124ea18b25c893d2e6661a0ea16ea890ae38e086eea37f31e5af` |
| Outcome identity hash | `b3b3156cf953071f26b14e945096970280f1d477eb2c352eedb6e77570ed86cd` |
| Performance report hash | `158416895a7f884a40d157d7eedfa2993a63311f1d4b10251aa602fc5c741cbe` |
| Lineage SHA-256 | `62f4015828fbe8466e36c5d63acb1fdd04019c7260e76233ecf488a601a39181` |
| Performance schema | `csp002.performance.0` |

## Decision behaviour

| Field | Value |
|---|---|
| Historical decisions generated | `195` |
| LONG | `110` |
| SHORT | `85` |
| NO_TRADE | `0` |
| First as-of | `2021-10-31T15:30:00+00:00` |
| Last as-of | `2024-12-31T15:30:00+00:00` |
| Span (calendar days) | `1157` |
| Decisions per calendar day | `0.168539` |

`NO_TRADE` is not treated as a zero-return trade. Trading tables below use LONG and SHORT only. Opportunity tables use NO_TRADE only.

Attached `outcome_return` is the B4 lake path as stored. `cumulative_return` is the sum of per-decision simple returns in ledger order (overlapping horizons are not a portfolio).

## Trading outcomes (LONG + SHORT)

| Horizon | n obs | n missing | mean | median | win | loss | zero | win rate | cumulative sum |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 5D | 110 | 85 | -0.002461 | -0.002757 | 51 | 58 | 1 | 0.4636 | -0.270655 |
| 10D | 110 | 85 | -0.001763 | 0.000420 | 55 | 54 | 1 | 0.5000 | -0.193900 |
| 20D | 110 | 85 | -0.005219 | -0.012859 | 44 | 65 | 1 | 0.4000 | -0.574055 |
| 60D | 110 | 85 | -0.009887 | -0.022931 | 41 | 68 | 1 | 0.3727 | -1.087593 |

## Risk (trading path)

| Horizon | max drawdown | volatility | downside vol | worst |
|---|---:|---:|---:|---:|
| 5D | 0.343125 | 0.019961 | 0.015671 | -0.062178 |
| 10D | 0.343600 | 0.027432 | 0.020380 | -0.062178 |
| 20D | 0.630550 | 0.042048 | 0.031074 | -0.133231 |
| 60D | 1.201002 | 0.058162 | 0.043826 | -0.133231 |

## LONG only

| Horizon | n obs | n missing | mean | median | win | loss | cumulative sum |
|---|---:|---:|---:|---:|---:|---:|---:|
| 5D | 110 | 0 | -0.002461 | -0.002757 | 51 | 58 | -0.270655 |
| 10D | 110 | 0 | -0.001763 | 0.000420 | 55 | 54 | -0.193900 |
| 20D | 110 | 0 | -0.005219 | -0.012859 | 44 | 65 | -0.574055 |
| 60D | 110 | 0 | -0.009887 | -0.022931 | 41 | 68 | -1.087593 |

## SHORT only

| Horizon | n obs | n missing | mean | median | win | loss | cumulative sum |
|---|---:|---:|---:|---:|---:|---:|---:|
| 5D | 0 | 85 | n/a | n/a | 0 | 0 | n/a |
| 10D | 0 | 85 | n/a | n/a | 0 | 0 | n/a |
| 20D | 0 | 85 | n/a | n/a | 0 | 0 | n/a |
| 60D | 0 | 85 | n/a | n/a | 0 | 0 | n/a |

## Opportunity cost (NO_TRADE)

What the attached path did after standing aside. Not trading P&L.

| Horizon | n obs | n missing | mean | median | win | loss | zero | cumulative sum | worst |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 5D | 0 | 0 | n/a | n/a | 0 | 0 | 0 | n/a | n/a |
| 10D | 0 | 0 | n/a | n/a | 0 | 0 | 0 | n/a | n/a |
| 20D | 0 | 0 | n/a | n/a | 0 | 0 | 0 | n/a | n/a |
| 60D | 0 | 0 | n/a | n/a | 0 | 0 | 0 | n/a | n/a |

## Coverage

| Horizon | trading observed | trading missing | opportunity observed | opportunity missing |
|---|---:|---:|---:|---:|
| 5D | 110 | 85 | 0 | 0 |
| 10D | 110 | 85 | 0 | 0 |
| 20D | 110 | 85 | 0 | 0 |
| 60D | 110 | 85 | 0 | 0 |

## Lineage

Complete decision → outcome lineage is in `lineage.json` (195 rows). Each row maps ledger `decision_id` / `as_of` / action to lake outcome and decision IDs per 5/10/20/60D horizon. Lineage SHA-256: `62f4015828fbe8466e36c5d63acb1fdd04019c7260e76233ecf488a601a39181`.

## What this is not

- Not G-GATE v1.1, not DETECTED/INCONCLUSIVE under that protocol.
- Not a freeze of Decision Engine v1.0.
- Not a ranking of horizons and not an optimizer output.
- Not a recommendation to trade.
