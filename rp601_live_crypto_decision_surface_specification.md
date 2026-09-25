# RP-601 — Live Crypto Decision Surface Specification (Corrected Prototype Baseline)

> [!IMPORTANT]
> **Governance & Canonical Derivation Invariants**
> - **Coralys Engine (`v0.1.0`)**: 100% **FROZEN** (Zero score formula changes).
> - **Position Trajectory Interpreter (`Config A`)**: 100% **FROZEN** ($K=3 \text{ bars}, G_{\text{max}}=0.70, \Delta M_{5,\text{min}}=-0.10$).
> - **Strict Evidence Derivation**: Every evidence bullet must be a deterministic projection of the actual 15m, 30m, and 1d resolution states. No static narrative fallback strings.
> - **Canonical Risk States Only**: Only canonical states (`REVERSAL DEVELOPING`, `RECOVERY DEVELOPING`, `TRAJECTORY STABLE`, `EVIDENCE CONFLICTED`) permitted. Invented UI states like `MODERATE RISK` are forbidden.
> - **Acquisition Classification**: Classified as **Live Current Snapshot Prototype** (single-pass data acquisition at $T_0 = \text{NOW}$). Continuous daemon ingestion requires a persistent scheduled loop.
> - **Domain Status Mandate**: Every output card MUST explicitly display `"domain_status": "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO"`.

---

## 1. Objective & Operational Status

Milestone **RP-601** extends the read-only Decision Surface to a 24/7 continuous market (liquid crypto assets) via a live current snapshot acquisition protocol.

Because equity markets have fixed trading hours, testing the `OBSERVE` surface live during off-hours creates latency gaps. Crypto trades continuously, providing a real-time snapshot environment to observe how the frozen decision engine processes multi-resolution streams ($15m, 30m, 1d$).

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
                  │ Current Snapshot Crypto Feed │
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
             (24/7 Snapshot) (Historical)   (Multi-Res)
```

---

## 4. Corrected Canonical Live Crypto DecisionCard Schema

```json
{
  "card_id": "card_BTC-USD_1790352867",
  "instrument": "BTC-USD",
  "timestamp": 1790352867,
  "datetime": "2026-09-25T16:14:27Z",
  "current_price": 83944.97,
  "domain_status": "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO",
  "resolutions": {
    "15m": {
      "label": "PULLBACK",
      "score": 0.0054
    },
    "30m": {
      "label": "PULLBACK",
      "score": 0.0474
    },
    "1d": {
      "label": "REVERSAL",
      "score": -0.3326
    }
  },
  "cross_resolution_relationship": "PARTIAL AGREEMENT",
  "risk_state": "EVIDENCE CONFLICTED",
  "decision_state": "WATCH",
  "evidence": [
    "• 15m trajectory is in a PULLBACK state",
    "• 30m trajectory is also in a PULLBACK state",
    "• 1d trajectory indicates REVERSAL",
    "• Intraday and daily trajectories are not aligned"
  ]
}
```

---

## 5. UI Dashboard Hand-Off Location & Launch Status

- **Web Dashboard Location**: [`services/ui`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/services/ui) (React + Vite single-page application).
- **Current App Entrypoint**: [`services/ui/src/App.tsx`](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/services/ui/src/App.tsx).
- **Target Integration**: The dashboard renders the read-only `DecisionCard` objects from `rp600_time_machine_decision_tapes.json` (REPLAY/COMPARE modes) and `rp601_live_crypto_decision_tapes.json` (24/7 OBSERVE mode).
