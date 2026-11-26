# RP-608 Crypto — Order-Flow Recommendation (exit-signal driven)

**Authority:** 0.0%  
**Shared mark:** `scripts/rp608_order_flow_mark.py`  
**Lifecycle owner:** `TradePlanOverlayManager` (`active_order_flow_recommendation`)

## Construct (LOCKED 2026-11-07)

Despite the historical “order flow” name, this baseline mark is **L1 resting TOB quantity imbalance** (displayed bid/ask size at the touch), **not** executed aggressor volume \(\Delta Q\).

- `OF_SUP` / reverse-OF = relative **resting liquidity** support, not buying/selling aggression.  
- Do not read as absorption or institutional flow.  
- Aggression + response research → **RP-608-OFX** only (`ACCEPT_CONSTRUCT_DISTINCTION`; audit §1b).  
- Formula / ±0.30 / reverse-OF rules remain **frozen** — terminology clarification only.

## Entry

When Policy B emits `LONG` / `SHORT` and L1 confirms:

- `imbalance = (bid_qty − ask_qty) / (bid_qty + ask_qty)`
- LONG: `imbalance ≥ +0.30` (+ micro ≥ mid when prices present)
- SHORT: `imbalance ≤ −0.30` (+ micro ≤ mid when prices present)

Stamp on decision card + activate `ORDER_FLOW_SUPPORTED` recommendation.

## Exit (not 60m)

OF stays **active until `ORDER_FLOW_EXIT`**: opposite-side order-flow confirm fires.

- Active LONG exits only when SHORT OF support appears
- Active SHORT exits only when LONG OF support appears
- Neutral book does **not** exit
- Trade-plan TP / SL / 60m horizon does **not** clear the OF recommendation

Plan lifecycle remains independent (still 60m max hold for simulated overlay).

## UI

Command Center shows green **ORDER FLOW** while `order_flow_recommendation.active` is true; copy states exit is reverse flow, not the 60m clock.

## Paper trade observability

See [`rp608_of_paper_trade_observability.md`](rp608_of_paper_trade_observability.md) (`RP-608-OF-PAPER-TRADE-001`).  
Each OF latch is a **PAPER** trade (0% authority): open MTM + closed ledger with dual exit timestamps; independent of plan TP/SL/60m.
