# RP-601 — Live Crypto Decision Surface Specification

> [!IMPORTANT]
> **Governance & Operational Invariants**
> - **Coralys Engine (`v0.1.0`)**: 100% **FROZEN** (Zero score formula changes).
> - **Position Trajectory Interpreter (`Config A`)**: 100% **FROZEN** ($K=3 \text{ bars}, G_{\text{max}}=0.70, \Delta M_{5,\text{min}}=-0.10$).
> - **Domain Status Mandate**: Every output card MUST explicitly display `"domain_status": "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO"`.
> - **Live Data Firewall**: Information cutoff $T_0 = \text{NOW}$.
> - **Zero Model Tuning**: The experiment evaluates the frozen ChronoSentiment pipeline on continuous live crypto feeds. Zero crypto-specific parameter tuning or model variants.

---

## 1. Objective & Domain Rationale

Milestone **RP-601** extends the RP-600 Decision Surface to a 24/7 continuous trading market (liquid crypto assets). 

Because equity markets have fixed trading hours, testing the `OBSERVE` surface live during off-hours creates latency gaps. Crypto trades continuously, providing an ideal real-time environment to observe how the frozen decision engine processes multi-resolution streams ($15m, 30m, 1d$) in real time.

---

## 2. Liquid Crypto Universe (Liquid 8)

| Instrument Ticker | Asset Name | Observation Streams | Trading Horizon |
|---|---|---|---|
| **`BTC-USD`** | Bitcoin / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`ETH-USD`** | Ethereum / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`BNB-USD`** | BNB / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`XRP-USD`** | XRP / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`SOL-USD`** | Solana / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`ADA-USD`** | Cardano / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`LINK-USD`** | Chainlink / USD | 15m, 30m, 1d | Continuous 24/7 |
| **`LTC-USD`** | Litecoin / USD | 15m, 30m, 1d | Continuous 24/7 |

---

## 3. Data Pipeline & System Architecture

```text
                  ┌──────────────────────────────┐
                  │ Real-Time Yahoo Crypto Feed  │
                  │ (BTC, ETH, SOL, BNB, etc.)   │
                  └──────────────┬───────────────┘
                                 │
                                 ▼
                     T₀ = NOW FIREWALL
                                 │
                                 ▼
                      ┌──────────────────────┐
                      │ Coralys v0.1.0       │
                      │ FROZEN               │
                      └──────────┬───────────┘
                                 │
                                 ▼
                      ┌──────────────────────┐
                      │ Config A             │
                      │ FROZEN               │
                      └──────────┬───────────┘
                                 │
                                 ▼
                      ┌──────────────────────┐
                      │ Canonical            │
                      │ DecisionState        │
                      └──────────┬───────────┘
                                 │
                                 ▼
                      ┌──────────────────────┐
                      │ Live Crypto          │
                      │ DecisionTape         │
                      └──────────┬───────────┘
                                 │
                                 ▼
                      ┌──────────────────────┐
                      │ DecisionCard         │
                      │ Read-Only View Model │
                      └──────────┬───────────┘
                                 │
                  ┌──────────────┼──────────────┐
                  ▼              ▼              ▼
               OBSERVE         REPLAY         COMPARE
             (24/7 Live)     (Historical)   (Multi-Res)
```

---

## 4. Canonical Live Crypto DecisionCard Schema

```json
{
  "card_id": "card_BTC-USD_1790352644",
  "instrument": "BTC-USD",
  "timestamp": 1790352644,
  "datetime": "2026-09-25T16:10:44Z",
  "current_price": 84025.95,
  "domain_status": "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO",
  "resolutions": {
    "15m": {
      "label": "PULLBACK",
      "score": 0.0199
    },
    "30m": {
      "label": "PULLBACK",
      "score": 0.0619
    },
    "1d": {
      "label": "REVERSAL",
      "score": -0.3199
    }
  },
  "cross_resolution_relationship": "PARTIAL AGREEMENT",
  "risk_state": "MODERATE RISK",
  "decision_state": "WATCH",
  "evidence": [
    "• 15m trajectory in pullback window",
    "• 30m intermediate horizon observed PULLBACK",
    "• 1d macro horizon indicates structural reversal risk",
    "• 15m and 1d reversal confirmation, with 30m still insufficient"
  ]
}
```

---

## 5. Formal Hand-off Directive

> **Goal**: Validate that the frozen decision engine produces coherent multidimensional recommendations in real time on a continuously traded market.
>
> **Domain Status**: The live crypto surface is a prospective live-domain experiment. Recommendations must NOT be interpreted as validated trading signals for crypto until formal cross-domain backtests are executed.
>
> **Contract Guarantee**: The UI consumes `rp601_live_crypto_decision_tapes.json` using the exact same `DecisionCard` View Model contract established in RP-600.
