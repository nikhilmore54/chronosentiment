# Archive Ingestion Justification: 2026 RELIANCE Open Auction Transition (Phase 2E-B)

**Universe Target:** `2026_reliance_open_auction_1m`
**Date:** 2026-05-23

## 1. What pressure class?
`auction-driven session` / `synthetic continuity`

## 2. What recurrence axis?
Ecology Transfer. Testing whether synthetic liquidity aggregation behaves consistently across entirely different geographic market structures (NSE vs NASDAQ).

## 3. What existing assumption does it pressure?
Pressures the assumption that auction mechanics have universal topological effects. Tests whether the NSE pre-open auction logic (09:00 - 09:15 IST) induces the exact same geometric behavior in `event_reset` as the US opening cross.

## 4. What makes it phenomenologically distinct?
Provides geographic negative evidence or cross-validation for the synthetic continuity behavior observed in the US market, ensuring we don't accidentally overfit to the NASDAQ matching engine's unique structural artifacts.
