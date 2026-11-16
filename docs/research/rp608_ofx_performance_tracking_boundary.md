# RP-608 Performance Tracking Boundary — Baseline vs OFX

**Parent:** `RP-608-OFX` · `RP-608-ECON-EDGE-DIAG-001`  
**Status:** LOCKED · 2026-11-04  
**Authority:** 0.0%  

## Roles

| Component | Role |
| --- | --- |
| NSE Estimand A | Existing **execution** baseline |
| NSE OF paper | Existing **order-flow** baseline |
| Crypto plan | Existing **execution** baseline |
| Crypto OF paper | Existing **order-flow** baseline |
| RP-608-OFX | Independent **research candidate** — never a fifth pooled book |

## Tracking principles

1. **Preserve the baseline** — Existing execution logic, thresholds, exits, and configuration remain unchanged.
2. **Keep books separate** — NSE ≠ crypto; execution ≠ OF paper; OFX ≠ either.
3. **Track realized performance** — Closed-trade gross, friction, net, expectancy, profit factor, W/L, exit distribution.
4. **Track open exposure separately** — Open count and MTM reported independently; never added to realized net as if closed.
5. **Track mechanism evidence separately** — OFX event classifications and experimental outcomes do **not** alter baseline attribution.
6. **Compare only on comparable cohorts** — Same instrument universe, time window, and accounting methodology; disclose coverage differences.
7. **Authority stays 0.0%** — No live-order authorization from baseline tracking or OFX research.

## Session report structure (required sections)

Every RP-608 session performance report should contain, in order:

1. **Baseline performance** — all four existing books, separately.
2. **Open-book status** — exposure and MTM, clearly marked unrealized.
3. **Baseline mechanism observations** — descriptive only until row-level evidence supports attribution (ECON-EDGE Step 2 rules).
4. **OFX research** — independent event metrics and research results (or `OFX_NOT_RUN` / Stage status).
5. **Comparison** — baseline versus OFX **only** when the experiment has comparable data and a valid evaluation design; otherwise omit or mark `NOT_COMPARABLE`.
6. **Governance** — rules frozen · Step 3+ held · Gate R unreached · authority 0%.

## What must never happen

| Anti-pattern | Why forbidden |
| --- | --- |
| Summing four books + OFX into one “session P&L” | Hides which mechanism failed |
| Replacing OF paper PnL with OFX simulation PnL in the baseline table | Destroys the reference series |
| Treating OFX labels as causes of baseline losses | Attribution without evidence |
| Changing ±0.30 / reverse-OF because OFX Stage 0–2 looks promising | Parameter swap preserves the conceptual flaw |
| Reporting open MTM as closed net | Inflates/deflates realized economics |

## Central distinction

> The baseline measures what the current system actually does. OFX investigates whether a different order-flow mechanism can do better.

No baseline rule changes, no replacement, and no pooling of research results into execution performance.

### Construct distinction (locked 2026-11-07)

Baseline NSE/crypto **OF paper** (`OF_SUP`, ±0.30, reverse-OF) measures **L1 resting bid/ask quantity imbalance** — displayed liquidity at the touch. It is **not** executed aggressor \(\Delta Q\), and must **not** be read as absorption or institutional aggression.

**RP-608-OFX** remains the only governed home for aggression + liquidity response + price-response research. Do not treat baseline OF stamps and OFX mechanism evidence as equivalent. See `docs/research/rp608_orderflow_side_quantity_semantics_audit.md` §1b (`ACCEPT_CONSTRUCT_DISTINCTION`).

### OFX volume-paper path gates (2026-11-15)

Crypto path analysis (n=170) drove **OFX paper-only** corrections: suppress MIXED entry tempo; cap clubbed opposing overshoot at +50% with 15m min-hold; early-adverse abort (−8 bps / 60s). These do **not** change baseline OF ±0.30, reverse-OF, Gate 2, or authority. See `docs/research/rp608_ofx_volume_paper_trades.md`.
