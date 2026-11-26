# Recommendation Engine Architecture

**Document ID:** ARCH-006
**Version:** 1.0
**Status:** Active — locked-in direction for RecommendationEngine v1
**Created:** 2026-08-18
**Author:** Coralys / ChronoSentiment Engineering

---

## Purpose

This document records the architectural comparison between the current RecommendationEngine v0 and the locked-in direction for RecommendationEngine v1. It captures the design decisions made on 2026-08-18 following completion of REC-001-H (121,805 historical records across 101 NSE tickers) and the evidence quality analysis.

This document is the authoritative reference for:
- What v0 does and why it is insufficient
- What v1 must do and why
- Where the implementation lives
- What is frozen and must not change
- The MVP path (not a large research programme)

---

## 1. Current Architecture — RecommendationEngine v0

### 1.1 Data flow

```
Market data (Yahoo Finance)
        │
        ▼
C3-002 policy (frozen)
        │
        ▼
DecisionRecord
  ├── direction (LONG / SHORT / NO_TRADE)
  ├── trend (Bullish / Bearish / absent)
  ├── momentum (Positive / Negative)
  ├── volatility
  ├── atr_14
  ├── reference_price
  └── geometry (target, risk, R:R)
        │
        ▼
RecommendationEngine v0
  ├── evidence_class: HDV-001 state-level (Favourable / Mixed / Unfavourable / Insufficient)
  ├── action: BUY / WATCH / NO_TRADE
  └── score: 0.5 × evidence + 0.3 × R:R + 0.2 × freshness
        │
        ▼
RecommendationRecord
        │
        ▼
Decision Server (:3001)
        │
        ▼
chrono-ui (/live)
```

### 1.2 Problems with v0

**Problem 1 — Static evidence (clustering)**

HDV-001 evidence is state-level, not ticker-specific. Every stock in the same C3-002 state (e.g. Bullish+Positive) receives the same evidence class and the same score. This produces identical scores for structurally different stocks:

```
HDFCBANK   0.6211   BUY
RELIANCE   0.6211   BUY
INFY       0.5660   BUY
TCS        0.5660   BUY
```

The differences are not evidence-driven — they are artefacts of the scoring formula.

**Problem 2 — Fixed R:R = 2.0**

C3-002 geometry always produces R:R ≈ 2.0 (1 ATR target, 0.5 ATR risk). The R:R term in the scoring formula therefore adds no differentiation. Every BUY recommendation has the same geometry regardless of the stock's historical behaviour.

**Problem 3 — No horizon**

v0 produces no estimate of how long a trade is expected to take. The user cannot distinguish a 2-session trade from a 10-session trade.

**Problem 4 — No analogue population**

v0 does not consult historical outcomes for the specific ticker in the specific state. It uses aggregate HDV-001 statistics that were computed across all tickers and all time.

**Problem 5 — Score ≠ probability**

The v0 score (0.6211) is a ranking score, not a probability of success. This is correct by design but means the output is not interpretable as a confidence level.

### 1.3 What v0 is correct about

- The geometry-absent invariant: BUY requires valid reference_price, target, risk, R:R. If geometry is absent, action = NO_TRADE.
- The evidence class hierarchy: Favourable > Mixed > Unfavourable > Insufficient.
- The action mapping: BUY requires geometry AND (Favourable OR (Mixed AND R:R ≥ 1.5)).
- The independence from HDV-002: v0 does not modify C3-002 risk boundaries.
- The independence from C3-002: v0 consumes C3-002 output; it does not modify it.

---

## 2. Locked-In Direction — RecommendationEngine v1

### 2.1 Architectural principle

**The recommendation engine is a Coralys capability, not a ChronoSentiment post-processing layer.**

```
                 CORALYS
                    │
       ┌────────────┼─────────────┐
       │            │             │
     C3-002       Evidence    Recommendation
     FROZEN       HDV-001       Engine v1
       │            │             │
       └────────────┼─────────────┘
                    │
              RecommendationRecord
                    │
                    ▼
             ChronoSentiment adapter
                    │
                    ▼
                 chrono-ui
```

ChronoSentiment is an **adapter/application of Coralys**, not the intelligence layer.

### 2.2 Three separated concerns

**Concern 1 — Market-data acquisition**

Highly concurrent. Bounded parallelism (8–16 workers). Retry with exponential backoff. Per-symbol checkpointing. Fetch metrics (requested / successful / failed / retried / elapsed / throughput).

This is a scalability problem, not a research problem. Target: 6,800 NSE/BSE instruments as a batch-data problem, not 6,800 sequential API calls.

**Concern 2 — Coralys historical reconstruction (REC-001-H)**

Deterministic and reproducible. Runs Coralys over cached market data. Produces the historical evidence store. Does not serve recommendations directly.

**Concern 3 — Coralys recommendation serving**

Precomputed historical evidence in an indexed store. Current Coralys decision evaluated against the appropriate historical analogue population. Target: <100ms response time.

### 2.3 Data flow (v1)

```
Market data cache
        │
        ▼
Coralys historical reconstruction (REC-001-H)
        │
        ▼
Historical evidence store (JSONL per ticker)
  ├── ticker
  ├── date
  ├── direction
  ├── trend / momentum / volatility
  ├── atr_14 / reference_price
  ├── volume / relative_volume_20
  ├── geometry (target, risk, R:R)
  ├── mfe_pct[10] / mae_pct[10]
  ├── outcome
  └── sessions_to_outcome
        │
        ▼ (precomputed, indexed)
Analogue population index
        │
        ▼
Current Coralys decision (C3-002 output)
        │
        ▼
Analogue filtering (same ticker → same state → similar volatility → similar volume)
        │
        ▼
Outcome distribution
  ├── target_rate (TARGET_BEFORE_RISK / n_analogues)
  ├── median_mfe5 / median_mfe10
  ├── median_mae5 / median_mae10
  └── median_sessions_to_outcome
        │
        ▼
Adaptive geometry
  ├── adaptive_target_pct (from historical MFE)
  ├── adaptive_risk_pct (from historical MAE)
  └── adaptive_rr (adaptive_target / adaptive_risk)
        │
        ▼
RecommendationEngine v1
  ├── action: BUY / WATCH / NO_TRADE
  ├── score: evidence-driven (not formula-driven)
  ├── horizon: median sessions to outcome
  ├── n_analogues: size of analogue population
  └── confidence: target_rate (not a probability, but interpretable)
        │
        ▼
RecommendationRecord v1
        │
        ▼
Decision Server (:3001)
        │
        ▼
chrono-ui (/live)
```

### 2.4 Analogue selection hierarchy (MVP)

Start simple. Do not build a sophisticated similarity model yet.

```
1. Same ticker
2. Same C3-002 state (direction + trend + momentum)
3. Similar volatility regime (Low / Normal / High — ATR-based)
4. Similar relative-volume regime (< 0.75 LOW / 0.75–1.25 NORMAL / > 1.25 HIGH)
5. Similar geometry (R:R bucket), where available
6. Require minimum analogue count (configurable, default 15)
7. If insufficient evidence → broaden population rather than inventing confidence
```

This is enough to make recommendations meaningfully different without building a similarity model.

### 2.5 Volume regime (MVP)

Volume is stored in REC-001-H. Use it as an analogue feature only. Do not build volume prediction.

```
relative_volume_20:
  < 0.75       → LOW
  0.75–1.25    → NORMAL
  > 1.25       → HIGH
```

Determine empirically whether volume regime materially differentiates outcomes before promoting it to a scoring lever.

### 2.6 Adaptive R:R (MVP)

Replace fixed R:R = 2.0 with evidence-derived geometry:

```
historical MFE (median, conservative percentile)
        ↓
adaptive target %
        ↓
current ATR
        ↓
adaptive risk %
        ↓
adaptive R:R
```

The resulting R:R is ticker/state/condition dependent, not globally fixed.

### 2.7 Horizon (MVP)

Include horizon in every recommendation:

```
Target:       +3.1%
Risk:         -1.5%
R:R:           2.07
Horizon:       5 sessions
Analogues:     187
Target rate:   34.2%
```

### 2.8 Implementation location

```
coralys-decision/
    src/
      recommendation/
        mod.rs          ← public API
        evidence.rs     ← analogue lookup + outcome distribution
        engine.rs       ← action derivation + scoring (v1)
```

This is a Coralys capability. It lives in `coralys-decision`, not in the ChronoSentiment adapter.

---

## 3. What Is Frozen and Must Not Change

| Component | Status | Constraint |
|-----------|--------|------------|
| C3-002 | **FROZEN** | v1 consumes C3-002 output; does not modify it |
| HDV-001 | **FROZEN** | State-level evidence baseline; v1 supplements but does not replace |
| REC-001 v0 policy | **FROZEN** | Prospective observation continues; v0 is not tuned |
| REC-001-H schema | **FROZEN** | Leakage boundary is fixed; no forward-looking features |
| UNIV-001 | **FROZEN for v0** | 102-stock universe; versioned stepping stone |
| HDV-002 | **Independent** | Risk-boundary research; cannot use REC-001 outcomes as feedback |

---

## 4. C3-002 State Mapping (Verified 2026-08-18)

Verified from REC-001-H JSONL data (121,805 records, 101 tickers):

| Trend | Momentum | Direction | Type |
|-------|----------|-----------|------|
| Bullish | Positive | LONG | Trend-following |
| Bullish | Negative | SHORT | Momentum divergence |
| Bearish | Positive | LONG | Counter-trend momentum bounce |
| Bearish | Negative | LONG | Counter-trend oversold bounce |
| absent | Positive | LONG | Momentum only |
| absent | Negative | SHORT | Momentum only |

**Key finding:** `Bearish+Negative` → LONG (not SHORT). The only SHORT-producing state with a directional trend signal is `Bullish+Negative`.

---

## 5. Evidence Quality Summary (2026-08-18)

From [`datasets/recommendation/historical/evidence_quality_report.csv`](../datasets/recommendation/historical/evidence_quality_report.csv):

**LONG evidence (101 tickers):**
- Min LONG bucket (smallest of Bull+Pos, Bear+Pos, Bear+Neg): median=170, min=25 (TMCV.NS), max=239
- All large-cap tickers: 130–240+ records per LONG state bucket
- LONG target rate: 29.6% mean (range 23–39%)
- Median MFE at session 5: ~2.5%; at session 10: ~3.8%
- Per-bucket target rates vary meaningfully (e.g. BAJFINANCE Bear+Pos=53.1%, LT Bear+Neg=46.8%)

**SHORT evidence (101 tickers):**
- Only one SHORT-producing state with directional signal: Bull+Neg
- Min SHORT bucket: median=187, min=17 (TMCV.NS), max=255
- All large-cap tickers: 140–255 SHORT records
- SHORT target rates: 13–41% across tickers

**Conclusion:** Evidence is sufficient for ticker-specific analogue-based recommendations for all large-cap tickers. TMCV.NS (172 total records) is the only ticker with sparse evidence; it should use a broader analogue population.

---

## 6. MVP Path

Do not build a large research programme before v1. The 121,805 records are sufficient to make the first version non-static and ticker-specific.

### 6.1 Execution order

**Step 1 — RecommendationEngine v1 (immediate)**

Build the actual Coralys recommendation path against the completed 101-ticker historical evidence. The 101-stock dataset is already sufficient to build and test the recommendation capability. Do not wait for the fetcher refactor.

```
REC-001-H COMPLETE (2026-08-18)
        ↓
Build Coralys RecommendationEngine v1
  ├── ticker/state analogue lookup
  ├── volume + volatility conditioning
  ├── adaptive target (from historical MFE)
  ├── adaptive R:R
  └── adaptive horizon
        ↓
Rank 102 stocks
        ↓
Run MVP
        ↓
Observe failures
        ↓
Improve
```

**Step 2 — MarketDataFetcher refactor (parallel/separate)**

Infrastructure scaling task. Not a prerequisite for proving RecommendationEngine v1. Acceptance criterion: fetch the 102-stock MVP universe reliably and substantially faster using bounded concurrency, retries, checkpointing and metrics. Do not scale to 6,800 yet.

**Do not do this:**
```
REC-001-E large research experiment
    ↓ feature importance
    ↓ interaction analysis
    ↓ statistical validation
    ↓ engine
```
Too slow for MVP. The right trade-off is: build a small, deliberate v1, observe its failures, then improve.

### 6.2 Graceful degradation in analogue selection

The analogue selection hierarchy must degrade gracefully rather than collapsing to NO_TRADE when the population is too small. This is particularly relevant for TMCV.NS (minimum buckets already much smaller). R:R bucket should NOT be an early hard filter if it collapses the population.

```
Exact analogue population
(same ticker + same state + volatility regime + volume regime + R:R bucket)
        │
        ├── >= 15 analogues → use it
        │
        └── < 15 analogues
                ↓
          relax R:R bucket (softest filter — drop first)
                ↓
          >= 15? → use
                │
                └── < 15
                        ↓
                  relax volume regime
                        ↓
                  >= 15? → use
                        │
                        └── < 15
                                ↓
                          relax volatility regime
                                ↓
                          >= 15? → use
                                │
                                └── < 15
                                        ↓
                                  same ticker + same state only
                                        ↓
                                  >= 15? → use
                                        │
                                        └── < 15
                                                ↓
                                          NO_TRADE (insufficient evidence)
```

Do not generate NO_TRADE simply because the analogue definition is too restrictive. Broaden the population first.

### 6.3 MVP gates (definition of done)

**G1 — Coralys ownership**
Recommendation is generated inside `coralys-decision`. Not in the ChronoSentiment adapter. Not in the UI.

**G2 — Differentiation**
Ticker recommendations are no longer identical merely because they share a C3-002 state. The following must NOT occur:
```
ADANI    0.62    5%    2.0
TCS      0.62    5%    2.0
INFY     0.62    5%    2.0
```
The following (or similar) MUST occur — different outputs because historical evidence differs:
```
TICKER      SCORE   TARGET   RISK   R:R    HORIZON
HINDALCO    0.78    4.1%     1.8%   2.28   5d
TRENT       0.74    3.7%     1.7%   2.18   6d
TITAN       0.69    3.0%     1.6%   1.88   5d
TCS         0.51    2.1%     1.7%   1.24   8d
```
(Numbers illustrative — not claims about what Coralys will produce.)

**G3 — Adaptive execution geometry**
Target, risk/R:R and horizon are derived from historical evidence rather than globally fixed constants. R:R = 2.0 everywhere is a gate failure.

**G4 — Leakage boundary**
No T+1 onward information enters the recommendation features. The leakage boundary established in REC-001-H must be preserved.

If all four gates pass, ship the MVP and observe it. Do not turn this into another long research cycle.

---

## 7. MarketDataFetcher Refactor (Parallel/Separate from v1)

Infrastructure scaling task. Refactor the market-data acquisition layer before scaling beyond 102 stocks:

```
MarketDataFetcher
  ├── fetch(symbol, period)
  └── fetch_batch(symbols, period)
        ├── bounded concurrency (8–16 workers)
        ├── rate limiting
        ├── retry with exponential backoff
        ├── per-symbol checkpointing
        └── fetch metrics:
              requested / successful / failed / retried / elapsed / throughput/sec
```

The batch implementation owns all reliability concerns. `REC-001-H` should not know whether data came from 1 worker or 16.

Acceptance criterion: fetch the 102-stock MVP universe reliably and substantially faster. Do not scale to 6,800 until v1 is shipped and observed.

---

## 8. Document History

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-18 | Initial — architectural comparison v0 vs v1; locked-in direction; C3-002 mapping verified; evidence quality summary |
| 1.1 | 2026-08-18 | Added execution order (v1 first, fetcher parallel); graceful degradation spec; four MVP gates (G1–G4); definition of done |