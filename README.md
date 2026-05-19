# ChronoSentiment — Propagation Ecology Observatory

> **ChronoSentiment is a deterministic execution-aware propagation observatory designed to study how market asymmetry emerges, persists, collapses, and decays across scale, regime, and asset ecology.**

Instead of projecting a false narrative of global predictive "alpha," ChronoSentiment treats markets as dynamic interaction systems. The platform provides a rigorous scientific framework to falsify propagation models and map the conditional boundaries of market behavior.

---

## 🌍 The Core Philosophy: Conditional Market Physics

ChronoSentiment has moved beyond the search for static, globally invariant trading laws. Our out-of-sample (OOS) validation and cross-asset experiments have revealed a fundamental truth: **market behavior is conditionally activated rather than universally uniform.**

We categorize market behavior into three distinct observational layers:

1. **Ecology Laws:** Market dynamics are determined by the underlying participant structure.
2. **Scale Laws (Granularity):** Microstructure patterns are scale-sensitive and compress as observations aggregate.
3. **Regime Laws:** Volatility states conditionally alter how asymmetry decays over time.

---

## 🧬 Two Discovered Ecology Classes

Through cross-asset replay validation (Crypto, Equities, and Commodities), we have mathematically isolated two distinct classes of propagation physics:

```
                  ┌─────────────────────────────────────────┐
                  │      COMPARATIVE PROPAGATION PHYSICS     │
                  └────────────────────┬────────────────────┘
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            ▼                                                     ▼
┌───────────────────────┐                             ┌───────────────────────┐
│ LIQUIDITY-FLOW ECOLOGY│                             │  EVENT-DRIVEN ECOLOGY │
├───────────────────────┤                             ├───────────────────────┤
│ Assets: Crypto, Equity│                             │ Assets: Commodities   │
│                       │                             │                       │
│ 💧 Pre-Bias: Toxic    │                             │ ⚡ Pre-Bias: Beneficial│
│ 💧 Age Decay: Real    │                             │ ⚡ Age Decay: Inverted│
│ 💧 Smoothness: 1m Only│                             │ ⚡ Smoothness: None   │
├───────────────────────┤                             ├───────────────────────┤
│ Asymmetry is consumed │                             │ Asymmetry is amplified│
│ by participation flow.│                             │ by persistent macro   │
│ Consensus exhausts.   │                             │ narratives (OPEC/etc).│
└───────────────────────┘                             └───────────────────────┘
```

### 1. Liquidity-Flow Ecology (Crypto & Equities)
*   **Mechanic:** Asymmetry is consumed by participation. Once a directional bias is established, the executable edge rapidly exhausts.
*   **Pre-Bias Toxicity (✅ Active):** Low-bias entries outperform high-bias entries.
*   **Elasticity Age Decay (✅ Active):** Fresh propagation (≤10 bars) dramatically outperforms stale entries (>15 bars).
*   **Smoothness Trap (✅ Resolution-Conditional):** Monotonic efficiency ordering exists *only* at 1m granularity.

### 2. Event-Driven Ecology (Commodities)
*   **Mechanic:** Asymmetry is amplified by narrative persistence. Established direction signals the entry of real-world macro constraints (supply shocks, geopolitics, OPEC decisions) rather than flow exhaustion.
*   **Laws Inverted (⊘ Inverted):** High prior bias *amplifies* subsequent propagation; stale, persistent trends perform *better* than short-lived breakouts.

---

## 🔬 The 6-Tab Observatory Architecture

The system's findings are explored via a high-performance **observability dashboard** that visualizes the translation of execution intent into realized outcomes:

| Tab | Diagnostic Domain | Key Scientific Metric |
| :--- | :--- | :--- |
| **Execution Ecology** | Realized outcomes & friction | Slippage capture, Elastic Recovery Ratio (ERR) |
| **Smoothness Trap** | Granularity & local topology | Local Excursion Efficiency ordering |
| **Edge Genesis** | Asymmetry birth mechanics | Compressive pre-range vs. directional release |
| **Toxicity Atlas** | Temporal decay profiles | Age-based decay curves & structural mortality |
| **Trade Replay** | Step-by-step causal auditor | Microsecond order queue traversal & late-bias |
| **Comparative Ecology** | Cross-asset & scale physics | Activation domains, 1m vs 5m resolution collapse |

---

## ⚙️ Architectural Disentanglement

ChronoSentiment separates the physical forces of execution through a frozen back-end engine:

*   **The Execution Simulation Engine (ESE):** Implements exact integer-scaled tick-by-tick order reconstruction (`PRICE_SCALE = 10000`). Decouples timing latency (`latency_ticks`) and slippage from raw signal strength.
*   **The Fertility Governor:** Dynamically constraints elastic re-expansion limits (`[0.85, 1.15]`) to prevent overfitting.
*   **Logistic Freshness Decay:** Automatically suppresses decaying directional signals through a strict logistic decay curve.

---

## 📊 Summary of Out-of-Sample (OOS) Experiments

*Frozen Architecture, 30-Day Windows*

| Condition | Class | Resolution | Pre-Bias Toxicity | Elasticity Age Decay | Smoothness Trap |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **A: Crypto Training** | Liquidity-Flow | 1m | **✅ Active** | **✅ Active** | **✅ Active** |
| **B: Crypto Same-Regime** | Liquidity-Flow | 5m | **⊘ Inverted** | — | **⊘ Inverted** |
| **C: Crypto OOS** | Liquidity-Flow | 5m | **✅ Active** | **✅ Active** | **⊘ Inverted** |
| **D: Equities** | Liquidity-Flow | 5m | **✅ Active** | **✅ Active** | **⊘ Inverted** |
| **E: Commodities** | Event-Driven | 5m | **⊘ Inverted** | **⊘ Inverted** | **⊘ Inverted** |

---

## 🚀 Scientific Roadmap

Our current development cycle targets **Track B: Edge Genesis Science**:

1.  **Latent-State Classification:** Replacing static filters with real-time volatility clustering and entropy classification.
2.  **State-Dependent Gating:** Automatically switching between *Liquidity-Flow* mode (mean-reverting, bias-avoiding) and *Event-Driven* mode (momentum-seeking) based on the asset's active ecology state.
3.  **Consensus Porosity:** Exploring pre-topology compression phases to predict early-stage asymmetry birth.

---

## 📂 Quick Start for Systematic Researchers

Run offline replays to export state telemetry directly to the Observatory:

```bash
# 1. Execute highly-rigorous out-of-sample replay
python3 scripts/replay_from_file.py --file archive/xasset_equities_30d_5m.jsonl --gen equities

# 2. Export deterministic telemetry to UI JSON DTO
python3 scripts/export_observatory_data.py archive/replay_xasset_equities.log observatory/xasset_equities.json

# 3. Serve the Observatory
cd observatory
python3 -m http.server 8888
```
