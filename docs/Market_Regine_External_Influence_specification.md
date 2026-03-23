# 🧠 ChronoSentiment — Market Regime & External Influence Specification (v1.0)

---

# 🎯 1. Purpose

This document defines how **external signals (e.g., news, sentiment)** influence **market behavior** within ChronoSentiment, while preserving:

* Determinism
* Event purity
* Causal traceability
* Replay fidelity

---

## ❗ Core Principle

```text
External signals MUST NOT directly affect strategies.

External signals MUST ONLY affect market behavior.
```

---

# 🧱 2. Architectural Placement

---

## Existing Flow (Current System)

```text
Strategy → OrderIntent → Kernel → ESE → Execution → Analytics → GA
```

---

## Extended Flow (With External Influence)

```text
External Signal (News)
        ↓
Market Regime Transformation Layer
        ↓
Market Events (OrderFlow / Trades / Cancels)
        ↓
Kernel → ESE → Execution → Outcome → GA
```

---

## ✅ Key Constraint

```text
GA NEVER consumes external signals directly.
```

---

# 🧠 3. External Signal Model

---

## 3.1 Event Definition

```rust
struct NewsEvent {
    timestamp: u64,
    sentiment_score: i64,     // Range: [-100, +100]
    impact_strength: u64,     // Scale: 1–10
    duration: u64,            // microseconds
}
```

---

## 3.2 Interpretation Rules

| Sentiment   | Meaning         |
| ----------- | --------------- |
| +80 to +100 | Strong positive |
| +20 to +80  | Mild positive   |
| -20 to +20  | Neutral         |
| -80 to -20  | Mild negative   |
| -100 to -80 | Strong negative |

---

# 🧱 4. Market Regime Model

---

## 4.1 Regime Types

```text
1. Calm Market
2. Trending Market (Bullish / Bearish)
3. High Volatility
4. News Shock
```

---

## 4.2 Regime State

```rust
struct MarketRegime {
    volatility_level: u64,
    order_flow_bias: i64,
    cancellation_rate: u64,
    trade_intensity: u64,
}
```

---

---

# ⚙️ 5. Transformation Layer (CRITICAL)

---

## 5.1 Function Definition

```rust
fn apply_external_influence(
    regime: &mut MarketRegime,
    news: &NewsEvent
)
```

---

## 5.2 Transformation Rules

---

### Positive News

```text
↑ buy-side market orders
↑ aggressive liquidity consumption
↓ queue waiting time
↑ trade intensity
```

---

### Negative News

```text
↑ sell-side pressure
↑ cancellations
↑ volatility
↓ queue stability
```

---

### High Impact News

```text
large spike in:
- volatility
- order flow imbalance
- cancellation bursts
```

---

---

## 5.3 Determinism Requirement

```text
Transformation MUST be deterministic.

Given:
- same NewsEvent
- same initial regime

Output MUST be identical.
```

---

# 📊 6. Market Event Generation

---

## 6.1 Mapping Regime → Events

```text
MarketRegime → generates:

- MarketTrade
- MarketOrderAdded
- MarketOrderCancelled
```

---

## 6.2 Example

---

### Calm Market

```text
low trade frequency
stable queue
low cancellations
```

---

### News Shock

```text
high trade bursts
frequent cancellations
rapid queue depletion
```

---

---

# 🧪 7. Dataset Integration

---

## 7.1 Dataset Structure

```json
[
  { "type": "news", "timestamp": 5, "sentiment": 80, "impact": 8 },
  { "type": "market", "timestamp": 6, "event": "trade", "qty": 300 }
]
```

---

## 7.2 Generation Rule

```text
NewsEvent → modifies MarketRegime → produces MarketEvents
```

---

---

# 🧠 8. GA Interaction Model

---

## ❗ Critical Constraint

```text
GA MUST NOT consume:
- sentiment_score
- news directly
```

---

## GA Observes ONLY:

```text
- realized_pnl
- drawdown
- trade_count
- fill efficiency
```

---

## Result

```text
GA learns:
- robustness to volatility
- behavior under regime shifts
```

---

---

# 🧪 9. Testing Requirements

---

## Test 1 — Determinism

```text
Same NewsEvent → identical MarketEvents
```

---

## Test 2 — Regime Consistency

```text
High sentiment → increased trade intensity
```

---

## Test 3 — Replay Fidelity

```text
News + Market → identical replay outcome
```

---

## Test 4 — No Direct Leakage

```text
GA MUST NOT access sentiment data
```

---

---

# 🚫 10. Prohibitions

---

DO NOT:

* feed sentiment directly into strategy
* store derived state in events
* introduce randomness outside deterministic RNG
* modify ESE logic
* break causal chain

---

---

# 🧠 11. System Law Alignment

---

This layer preserves:

---

## Law 1 — Causality

```text
News → Market → Execution → Outcome
```

---

## Law 2 — Event Purity

```text
NewsEvent is a fact
MarketEvents are facts
State is derived
```

---

## Law 3 — Determinism

```text
Same inputs → identical outputs
```

---

---

# 🎯 12. Expected Outcome

After implementation:

---

## Without External Influence

```text
GA optimizes for static conditions
```

---

## With External Influence

```text
GA evolves strategies that:
- adapt to volatility
- handle liquidity shocks
- avoid overfitting
```

---

---

# 🚀 13. Implementation Priority

---

## Phase Order

```text
1. API Layer (FIRST)
2. GA Validation (current system)
3. THEN → Implement this spec
```

---

---

# 🏁 FINAL STATEMENT

---

This specification introduces **environmental realism** into ChronoSentiment while preserving:

```text
✔ determinism
✔ causal traceability
✔ execution correctness
```

---

It transforms the system from:

```text
Execution Simulator → Market Behavior Simulator
```

---