Excellent—this is where your **product becomes programmable** and where your **GA engine derives power**.

Below is a **production-grade Strategy Definition Specification** designed to be:

* deterministic
* extensible
* GA-compatible
* UI + API compatible
* execution-aware (not just signal-based)

---

# 🧠 ChronoSentiment — Strategy Definition Specification (v1.0)

---

# 1. 🎯 Purpose

This specification defines how trading strategies are:

* **structured**
* **validated**
* **executed**
* **evolved (GA)**

A strategy in ChronoSentiment is:

> A **deterministic event-driven decision system** that consumes market + sentiment data and produces **orders with execution constraints**

---

# 2. 🧱 Strategy Architecture

---

## 2.1 Core Components

```text
Strategy
 ├── Metadata
 ├── Inputs
 ├── Signals
 ├── Conditions
 ├── Actions
 ├── Risk Rules
 ├── Execution Constraints
 ├── Parameters (GA-tunable)
```

---

## 2.2 Design Principles

| Principle     | Requirement                |
| ------------- | -------------------------- |
| Deterministic | Same input → same output   |
| Declarative   | JSON-defined               |
| Modular       | Signals/Rules reusable     |
| Observable    | Every decision explainable |
| GA-Compatible | Parameters mutable         |

---

# 3. 📦 Strategy Schema (Canonical JSON)

---

```json
{
  "strategy_id": "uuid",
  "name": "mean_reversion_v1",
  "version": "1.0",
  "description": "RSI-based mean reversion",

  "metadata": {
    "author": "user_id",
    "created_at": 1710000000,
    "asset_class": "EQUITY",
    "market": "NSE"
  },

  "inputs": {
    "symbols": ["RELIANCE", "INFY"],
    "timeframe": "1m",
    "data_sources": ["price", "volume", "sentiment"]
  },

  "parameters": {
    "rsi_period": 14,
    "rsi_buy_threshold": 30,
    "rsi_sell_threshold": 70,
    "position_size": 100
  },

  "signals": [
    {
      "id": "rsi_signal",
      "type": "RSI",
      "params": {
        "period": "{{rsi_period}}"
      }
    }
  ],

  "conditions": [
    {
      "id": "buy_condition",
      "expression": "rsi_signal < {{rsi_buy_threshold}}"
    },
    {
      "id": "sell_condition",
      "expression": "rsi_signal > {{rsi_sell_threshold}}"
    }
  ],

  "actions": [
    {
      "id": "buy_action",
      "when": "buy_condition",
      "type": "PLACE_ORDER",
      "side": "BUY",
      "order_type": "LIMIT",
      "price": "best_bid",
      "quantity": "{{position_size}}"
    },
    {
      "id": "sell_action",
      "when": "sell_condition",
      "type": "PLACE_ORDER",
      "side": "SELL",
      "order_type": "LIMIT",
      "price": "best_ask",
      "quantity": "{{position_size}}"
    }
  ],

  "risk": {
    "max_position": 500,
    "max_loss_per_day": 10000,
    "stop_loss": 2.0,
    "take_profit": 3.5
  },

  "execution": {
    "time_in_force": "DAY",
    "slippage_tolerance_bps": 20,
    "latency_profile": "standard"
  }
}
```

---

# 4. 🧩 Schema Breakdown (Detailed)

---

## 4.1 Metadata

| Field       | Description       |
| ----------- | ----------------- |
| strategy_id | unique identifier |
| version     | version control   |
| market      | NSE/BSE           |

---

## 4.2 Inputs

Defines data dependencies:

```json
{
  "symbols": ["NIFTY"],
  "timeframe": "1m"
}
```

---

## 4.3 Parameters (GA-Compatible)

---

### Key Rule:

> ALL tunable variables MUST be defined here

---

### Example

```json
{
  "rsi_period": 14,
  "threshold": 30
}
```

---

## 4.4 Signals

---

### Signal Types (MVP)

| Type      | Description     |
| --------- | --------------- |
| RSI       | momentum        |
| SMA       | trend           |
| VWAP      | volume-weighted |
| SENTIMENT | NLP score       |

---

### Signal Schema

```json
{
  "id": "signal_id",
  "type": "RSI",
  "params": {}
}
```

---

## 4.5 Conditions

---

### Expression Language

* Boolean expressions
* Supports:

  * signals
  * parameters
  * constants

---

### Example

```text
rsi_signal < 30 AND sentiment_score > 0.5
```

---

## 4.6 Actions

---

### Types

| Type         | Description  |
| ------------ | ------------ |
| PLACE_ORDER  | submit order |
| CANCEL_ORDER | cancel       |
| MODIFY_ORDER | modify       |

---

### Action Schema

```json
{
  "type": "PLACE_ORDER",
  "side": "BUY",
  "price": "best_bid"
}
```

---

## 4.7 Risk Module

---

### Enforced by system (not strategy)

| Rule           | Description |
| -------------- | ----------- |
| max_position   | cap         |
| stop_loss      | exit        |
| drawdown limit | kill switch |

---

## 4.8 Execution Constraints

---

Defines behavior at ESE level:

```json
{
  "order_type": "LIMIT",
  "latency_profile": "low_latency",
  "slippage_tolerance_bps": 10
}
```

---

# 5. ⚙️ Execution Semantics

---

## 5.1 Event Loop Integration

```text
Tick → Signals → Conditions → Actions → Orders → ESE
```

---

## 5.2 Evaluation Order

```text
1. Update signals
2. Evaluate conditions
3. Trigger actions
4. Apply risk rules
5. Submit orders
```

---

## 5.3 Determinism Requirements

* No randomness inside strategy
* All parameters fixed per run
* Event-driven only

---

# 6. 🧬 GA Compatibility

---

## 6.1 Parameter Mutation

```json
{
  "rsi_period": [10, 20],
  "threshold": [25, 35]
}
```

---

## 6.2 Fitness Inputs

* PnL
* Sharpe
* Drawdown

---

## 6.3 Constraints

* Parameter bounds MUST be defined
* Invalid configs MUST be rejected

---

# 7. 🔍 Validation Rules

---

## 7.1 Schema Validation

* Required fields present
* Types correct

---

## 7.2 Logical Validation

* No circular dependencies
* Conditions reference valid signals

---

## 7.3 Execution Validation

* Order size valid
* Price valid (tick size)

---

# 8. 🧪 Explainability Model

---

Every action MUST produce:

```json
{
  "decision_trace": {
    "signals": {...},
    "conditions": {...},
    "reason": "rsi < 30"
  }
}
```

---

# 9. 🔄 Versioning

---

## 9.1 Strategy Version

* Increment on change
* Immutable past versions

---

## 9.2 Compatibility

* Backward compatible parsing required

---

# 10. 📊 Strategy Lifecycle

---

```text
Create → Validate → Simulate → Analyze → Evolve → Replay
```

---

# 11. ⚠️ Edge Cases

---

## 11.1 Conflicting Actions

* Multiple buy/sell signals → resolve via priority

---

## 11.2 Overtrading

* Throttle via risk rules

---

## 11.3 No Liquidity

* Orders remain pending

---

# 12. 🔌 Interfaces

---

## 12.1 Input to Strategy Engine

```json
{
  "event": "MARKET_TICK",
  "data": {...}
}
```

---

## 12.2 Output

```json
{
  "action": "PLACE_ORDER",
  "payload": {...}
}
```

---

# 13. 🔒 Constraints

---

* Must remain deterministic
* Must be JSON serializable
* Must be replayable

---

# 14. 🚀 Extensibility

---

Future support:

* Python-based strategies
* ML models
* Multi-asset portfolios

---

# 🔚 FINAL SUMMARY

The strategy system is:

> A **deterministic, declarative, event-driven rule engine**
> that transforms **market + sentiment inputs into execution-aware orders**

