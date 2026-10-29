# RP-608 Trade-Plan Overlay v1 — Shadow Specification

**Protocol ID:** `RP-608-TRADE-PLAN-OVERLAY-001`  
**Target Instrument:** `BTC/USDT` Binance Spot  
**Status:** DRAFT PRE-REGISTRATION / SHADOW ONLY  
**Order Authority:** Strictly 0.0% (Zero Execution Authority)  
**Signal Source:** Policy B Only (Canonical Policy)  
**Comparator:** Policy C (Preserved as observational baseline)  

---

## 1. Purpose & Governing Principles

The **Trade-Plan Overlay v1** converts the existing Policy B directional recommendation (`LONG` / `SHORT`) into an explicit, deterministic, time-bounded simulated trade plan containing:
1. **Eligible Entry Price:** Bid-side for `SHORT`, Ask-side for `LONG` (accounting for bid/ask spread).
2. **ATR-Based Risk Bounds:** Fixed $1.0 \times \text{ATR}(14)$ stop-loss and $1.5 \times \text{ATR}(14)$ profit target.
3. **Maximum Holding Horizon:** Exactly 60 minutes from simulated entry timestamp.
4. **Net Reward/Risk Expectancy:** Calculated post-friction including 12.0 bps round-trip fees/slippage and half-spread crossing.
5. **Auditable Lifecycle Records:** Append-only ledger in an isolated directory.

### Invariant Separation from RP-608
- The underlying RP-608 prospective observer (`RP-608-SHADOW-001B`) remains 100% frozen, read-only, and untouched.
- Decision cards in `live_capture/shadow_validation/crypto/rp608_decision_cards.jsonl` are never rewritten or mutated.
- The overlay writes strictly to `live_capture/shadow_validation/crypto/trade_plan_overlay_v1/`.

---

## 2. Fixed Initial Research Parameters

| Parameter | Value | Rule & Governance Justification |
| :--- | :--- | :--- |
| **Stop-Loss Multiplier ($k_s$)** | **`1.0`** | $SL = E \pm 1.0 \times \text{ATR}_{14}$ (captured at signal time, frozen) |
| **Profit-Target Multiplier ($k_t$)** | **`1.5`** | $TP = E \mp 1.5 \times \text{ATR}_{14}$ (captured at signal time, frozen) |
| **Maximum Horizon ($H$)** | **`60 min`** | Exactly 3,600 seconds from accepted simulated entry timestamp |
| **Simulated Notional ($N$)** | **`1,000 USDT`** | Fixed research size for consistent friction and depth accounting |
| **Price Tick Increment** | **`0.01 USDT`** | All levels rounded to valid Binance BTC/USDT price increments |
| **Concurrency Rule** | **`1 Plan Max`** | At most 1 active plan at a time. No pyramiding or auto-reversals. |

---

## 3. Mathematical Formulation

Let:
- $E$ = Eligible entry price from top-of-book ($P_{\text{ask}}$ for LONG, $P_{\text{bid}}$ for SHORT).
- $A$ = Frozen $\text{ATR}(14)$ from originating decision card.
- $k_s = 1.0$, $k_t = 1.5$.

### Price Levels
$$\text{For LONG:}\quad SL = \text{round}(E - k_s A, 0.01),\quad TP = \text{round}(E + k_t A, 0.01)$$
$$\text{For SHORT:}\quad SL = \text{round}(E + k_s A, 0.01),\quad TP = \text{round}(E - k_t A, 0.01)$$

### Transaction Cost Model (Registered RP-608 Friction)
- **Taker Fee:** $5.0\text{ bps}$ per side ($0.05\%$).
- **Exogenous Slippage:** $1.0\text{ bp}$ adverse per side ($0.01\%$).
- **Spread Crossing:** Incorporated inherently by buying at $P_{\text{ask}}$ and selling at $P_{\text{bid}}$.
- **Total Fixed Drag:** $F_{\text{drag}} = 2 \times (5.0 + 1.0) = 12.0\text{ bps}$ ($0.12\%$).

### Expected Reward/Risk Metrics
$$\text{Gross Reward} = k_t A \cdot Q,\quad \text{Gross Risk} = k_s A \cdot Q$$
$$\text{Gross R:R} = \frac{1.5}{1.0} = 1.50 : 1$$
$$\text{Net Target Profit} = \text{Gross Reward} - (\text{Entry Friction} + \text{Exit Friction})$$
$$\text{Net Stop Loss} = \text{Gross Risk} + (\text{Entry Friction} + \text{Exit Friction})$$
$$\text{Net R:R} = \frac{\text{Net Target Profit}}{\text{Net Stop Loss}}$$

---

## 4. Lifecycle State Machine & Edge Cases

```
[Policy B Signal] ───► [Quality & Freshness Gate] ───► [PLAN_CREATED]
                              │ (Fail)
                              ▼
                         [REJECTED]
                              │
                              ▼
                         [PLAN_OPEN] (Holding ≤ 60m)
                         /    |    \
                        /     |     \
       (Hit TP)        / (Hit SL)    \ (60m elapsed)
                      ▼       ▼       ▼
                [CLOSED_TP] [CLOSED_SL] [TIME_EXIT]
```

### Critical Edge Cases
1. **Ambiguous Intra-Observation Crossings:** If a price gap or interval crosses both $SL$ and $TP$, the event is flagged `AMBIGUOUS_STOP_TARGET` and resolved conservatively to `CLOSED_SL` in the primary evaluation.
2. **Quote Staleness:** If quote age $> 500\text{ ms}$ at trigger time, mark `EXIT_PENDING_DATA` without fabricating prices.
3. **Signal Overlap:** If a new Policy B signal arrives while a plan is open, log `SUPPRESSED_OPEN_PLAN` without extending the 60m horizon or modifying levels.
4. **Zero Order Authority:** No orders, keys, or endpoints exist. All exits are simulated from top-of-book execution quotes.

---

## 5. Ledger Schema & Storage

Isolated directory:
`live_capture/shadow_validation/crypto/trade_plan_overlay_v1/`
- `overlay_config.json`: Frozen hash and configuration parameters.
- `trade_plan_events.jsonl`: Append-only event log (creation, entries, triggers, exits).
- `trade_plan_outcomes.jsonl`: Terminal settlement records with realized net P&L and holding duration.
