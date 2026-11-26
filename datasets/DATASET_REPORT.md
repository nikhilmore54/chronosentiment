# Coralys Stop Research Dataset v0.1 — Dataset Report

**Built:** 2026-08-17 04:24 UTC  
**Total decisions:** 1144  
**Schema version:** 0.1  

---

## Purpose

Pure evidence/data construction layer. One row per Coralys decision.
No stop parameters are calculated or optimised in this script.
Stop-policy discovery is a separate experiment after dataset validation.

---

## Config Roles

| Config | Role | Stop Diagnostics |
|--------|------|-----------------|
| A — ₹5K EqualWeight | Capital-gating / realization baseline | ❌ None (0 realizations) |
| B — ₹1M EqualWeight | Primary stop regime | ✅ Yes |
| C — ₹1M MaxPerLot ₹20K | Primary stop regime | ✅ Yes |

---

## Decision Universe

- Total certified decisions: **1144**
- A realized: **0** (capital-gating baseline)
- B realized: **1144**
- C realized: **728**

---

## Outcome States — Config B

| Outcome | Count | % |
|---------|-------|---|
| not_realized | 230 | 20.1% |
| realized_horizon | 3 | 0.3% |
| realized_stopped | 574 | 50.2% |
| realized_target | 329 | 28.8% |
| realized_unknown:AMBIGUOUS | 8 | 0.7% |

## Outcome States — Config C

| Outcome | Count | % |
|---------|-------|---|
| not_realized | 473 | 41.3% |
| realized_horizon | 3 | 0.3% |
| realized_stopped | 372 | 32.5% |
| realized_target | 288 | 25.2% |
| realized_unknown:AMBIGUOUS | 8 | 0.7% |

---

## Stop Taxonomy — Config B

| Category | Count |
|----------|-------|
| DirectionFailure | 213 |
| TemporaryExcursion | 186 |
| GenuineAdverse | 119 |
| StopTooTight | 32 |
| PrematureStop | 24 |
| **Total stopped** | **574** |

## Stop Taxonomy — Config C

| Category | Count |
|----------|-------|
| DirectionFailure | 176 |
| TemporaryExcursion | 141 |
| StopTooTight | 20 |
| GenuineAdverse | 18 |
| PrematureStop | 17 |
| **Total stopped** | **372** |

---

## Coralys Decision State Distribution

*(Sourced from B sealed_decision; identical for C — same Coralys artifact)*

### Trend

| Value | Count |
|-------|-------|
| Bearish | 494 |
| Bullish | 650 |

### Momentum

| Value | Count |
|-------|-------|
| Negative | 466 |
| Positive | 678 |

### Volatility

| Value | Count |
|-------|-------|
| present | 1144 |

---

## Scaffolded Fields (null in v0.1)

These fields require bar cache / ATR computation and are reserved for future enrichment:

| Field | Description |
|-------|-------------|
| `atr_at_decision` | ATR(14) at decision_time |
| `stop_distance_atr_ratio` | Declared stop distance / ATR |
| `mae_atr_ratio_B` / `_C` | MAE / ATR for each config |
| `mfe_atr_ratio_B` / `_C` | MFE / ATR for each config |
| `cf_stop_0atr_outcome_B/C` | Counterfactual: no stop |
| `cf_stop_025atr_outcome_B/C` | Counterfactual: 0.25 ATR stop |
| `cf_stop_05atr_outcome_B/C` | Counterfactual: 0.5 ATR stop |
| `cf_stop_075atr_outcome_B/C` | Counterfactual: 0.75 ATR stop |
| `market_regime` | Market-level regime label |
| `instrument_regime` | Instrument-level regime label |

---

## Data Quality

- B trades with MAE data: **914**
- C trades with MAE data: **671**
- B stop diagnostics attached: **574**
- C stop diagnostics attached: **372**
- Coralys state populated: **1144**

---

## Next Steps

1. **Validate** this dataset against known counts (574 B stops, 1144 B realizations, 728 C realizations)
2. **Enrich** with ATR at decision time (requires bar cache)
3. **Compute** ATR-normalised MAE/MFE ratios
4. **Simulate** counterfactual stop policies (0, 0.25, 0.5, 0.75 ATR)
5. **Add** regime labels (market, instrument)
6. **Define** objective function before any stop parameter optimisation

*Stop-policy discovery is a separate experiment after dataset validation.*
