This is a **critical foundational artifact**—your entire system (ESE, strategy logic, analytics, UI explainability) depends on a **precise, shared vocabulary**.

Below is a **production-grade Market Microstructure Dictionary (India-first, simulation-aligned)** designed for:

* engineers
* quants
* product
* UI/UX explainability

---

# 📘 ChronoSentiment — Market Microstructure Dictionary (v1.0)

---

# 1. 🧭 Purpose

This dictionary defines **canonical terms, behaviors, and system mappings** used across:

* Simulation Kernel
* Execution Simulation Engine (ESE)
* Strategy Engine
* Analytics
* UI

---

## Core Principle

> Every term MUST have:

* a **precise definition**
* a **mathematical or behavioral interpretation**
* a **mapping to system implementation**

---

# 2. 📊 Order Book Concepts

---

## 2.1 Order Book

**Definition**
A structured list of all active buy and sell orders for a given instrument.

**Structure**

```text
Bids (Buy Orders) | Asks (Sell Orders)
```

**System Mapping**

* Stored in ESE as:

  * price levels
  * FIFO queues

---

## 2.2 Bid

**Definition**
Highest price a buyer is willing to pay.

**Formula**

```text
best_bid = max(bid_prices)
```

**System Mapping**

* Top of bid book
* Used in strategy pricing (`best_bid`)

---

## 2.3 Ask (Offer)

**Definition**
Lowest price a seller is willing to accept.

```text
best_ask = min(ask_prices)
```

---

## 2.4 Spread

**Definition**
Difference between best ask and best bid.

```text
spread = best_ask - best_bid
```

**Importance**

* Liquidity indicator
* Slippage baseline

---

## 2.5 Mid Price

```text
mid_price = (best_bid + best_ask) / 2
```

---

## 2.6 Depth

**Definition**
Total quantity available at each price level.

**Types**

* L1: best bid/ask
* L2: aggregated levels
* L3: full order queue

---

# 3. 📦 Order Types

---

## 3.1 Market Order

**Definition**
Order to execute immediately at best available price.

**System Behavior**

* Consumes liquidity
* May cause slippage

---

## 3.2 Limit Order

**Definition**
Order to buy/sell at a specified price or better.

**System Behavior**

* Adds liquidity (if resting)
* Subject to queue position

---

## 3.3 IOC (Immediate-Or-Cancel)

**Definition**
Execute immediately; cancel remainder.

---

## 3.4 Day Order

**Definition**
Valid until market close.

---

# 4. ⚙️ Matching & Execution Concepts

---

## 4.1 Matching Engine

**Definition**
System that matches buy and sell orders.

**Rule**

```text
FIFO (price-time priority)
```

---

## 4.2 Price-Time Priority (FIFO)

**Definition**
Orders are matched based on:

1. Best price
2. Earliest timestamp

---

## 4.3 Fill

**Definition**
Execution of part or full order.

---

## 4.4 Partial Fill

**Definition**
Only part of the order is executed.

---

## 4.5 Trade

**Definition**
A completed match between buyer and seller.

---

## 4.6 VWAP (Volume Weighted Average Price)

```text
VWAP = Σ(price × qty) / Σ(qty)
```

---

# 5. 🧮 Liquidity & Impact

---

## 5.1 Liquidity

**Definition**
Ease of executing trades without moving price.

---

## 5.2 Liquidity Provider (Maker)

**Definition**
Places resting orders.

---

## 5.3 Liquidity Taker

**Definition**
Consumes existing orders.

---

## 5.4 Market Impact (Simulated)

```text
impact ∝ order_size / available_liquidity
```

---

# 6. ⏱️ Time & Latency

---

## 6.1 Latency

**Definition**
Delay between decision and execution.

---

## 6.2 Types

| Type     | Description      |
| -------- | ---------------- |
| Network  | Chennai → Mumbai |
| Exchange | processing delay |
| Internal | system overhead  |

---

## 6.3 Slippage

**Definition**
Difference between expected and actual execution price.

```text
slippage = execution_price - expected_price
```

---

# 7. 📉 Price Behavior

---

## 7.1 Tick Size

**Definition**
Minimum price increment.

```text
price % tick_size == 0
```

---

## 7.2 Volatility

**Definition**
Rate of price change.

---

## 7.3 Last Traded Price (LTP)

**Definition**
Price of most recent trade.

---

# 8. 🧾 Regulatory (India-Specific)

---

## 8.1 Market Price Protection (MPP)

**Definition**
Rejects orders too far from reference price.

```text
|order_price - reference_price| ≤ threshold
```

---

## 8.2 Circuit Limits

**Definition**
Upper and lower bounds for price movement.

---

## 8.3 Auction Phases

| Phase      | Description      |
| ---------- | ---------------- |
| Pre-open   | order collection |
| Continuous | normal trading   |
| Closing    | final auction    |

---

# 9. 📊 Strategy & Execution Metrics

---

## 9.1 PnL (Profit & Loss)

```text
PnL = realized + unrealized
```

---

## 9.2 Sharpe Ratio

```text
Sharpe = (returns - risk_free_rate) / std_dev
```

---

## 9.3 Drawdown

**Definition**
Peak-to-trough decline.

---

## 9.4 Win Rate

```text
wins / total_trades
```

---

# 10. 🧠 Queue & Execution Dynamics

---

## 10.1 Queue Position

**Definition**
Position of order within price level.

---

## 10.2 Queue Depletion

Occurs when:

* trades execute ahead
* orders cancel ahead

---

## 10.3 Fill Probability (Simulated)

```text
P(fill) ∝ liquidity_ahead / total_flow
```

---

# 11. 🔄 Simulation-Specific Terms

---

## 11.1 Virtual Clock

Simulation time independent of real time.

---

## 11.2 Determinism

```text
same input → same output
```

---

## 11.3 Replay

Re-running events to reproduce results.

---

## 11.4 Event

Atomic state change unit.

---

# 12. 🔍 Explainability Terms (UI)

---

## 12.1 Decision Trace

Breakdown of:

* signals
* conditions
* actions

---

## 12.2 Execution Trace

Breakdown of:

* queue position
* latency
* fill path

---

## 12.3 Slippage Breakdown

Components:

* spread
* impact
* latency

---

# 13. ⚠️ Edge Concepts

---

## 13.1 Thin Market

Low liquidity → high slippage

---

## 13.2 Order Sweep

Large order consuming multiple price levels

---

## 13.3 Ghost Liquidity

Displayed but not actually executable (simulated via factor)

---

# 14. 🔗 System Mapping Summary

---

| Concept    | System Component |
| ---------- | ---------------- |
| Order Book | ESE              |
| Matching   | ESE              |
| Latency    | Kernel           |
| Strategy   | Strategy Engine  |
| Replay     | Kernel           |
| Metrics    | Analytics        |

---

## 15. Strategy Evolution vs Strategy Execution

A strategy evolves across simulations, but it executes within a single one.

This distinction is subtle, but essential.

During execution, a strategy expresses intent under a fixed understanding of the market. That intent is then exposed to time, competition, and constraint.

If the strategy were allowed to adapt during this process, the system would no longer be observing how intent behaves under reality. It would instead be observing a feedback loop between intent and outcome.

ChronoSentiment prevents this.

It ensures that:

- execution remains a closed, causal process
- evolution occurs only between simulations

This allows the system to answer two different questions clearly:

- What happened in this simulation?
- How should the strategy change for the next one?

By separating these questions, the system preserves both clarity of behavior and validity of learning.

---

# 🔚 FINAL SUMMARY

This dictionary defines:

> A **shared language between markets and system design**
> ensuring consistency across:

* simulation
* execution
* analytics
* UI
