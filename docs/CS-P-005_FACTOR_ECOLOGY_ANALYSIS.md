# CS-P-005 — Factor Ecology Analysis v0.1

**Document type:** Product research brief  
**Status:** Active  
**Date:** 2026-08-14  
**Parent:** CS-P-001, CS-P-004-E1-S1, CS-P-TEST-001  
**Input snapshot SHA-256:** `e7685d936bdfaf53d7055ca683a87b4ca85149dd0eb89402dfaa93facfd8616f`  
**Does not open:** B5, G-GATE, Decision Engine v1.0, candidate policy, threshold search

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology.

**Question:**

> Do Trend, Momentum and Volatility contain sufficiently distinct and stable states to justify a selective decision policy?

This report must **not** produce a trading recommendation.

---

## 1. Sequence

```text
CS-P-TEST-001 (must be green)
          ↓
Certified enrichment snapshot (not B5)
          ↓
Factor Ecology v0.1   ← this document
          ↓
STOP — candidate specification is a later, frozen document
```

---

## 2. Outputs

1. Factor distributions (`roc_20`, `atr_14`) from bars with `effective_from ≤ T`
2. Factor availability confirmation
3. Cross-factor state matrix
4. Temporal coverage (year)
5. Instrument coverage
6. Current `TrendMappingPolicy` action frequencies (descriptive)
7. Subsequent lake outcomes attached **as measurement only**
8. Design constraints for a later candidate — not the candidate itself

---

## 3. Forbidden

- Tuning X/Y because 2022/2023/2024 look better
- Using outcomes during decision generation
- Implementing `DecisionPolicyCandidate_v0.1` in this step
- G-GATE, B3/B4 mutation, v1.0 freeze, another performance backtest

Volatility remains magnitude-only. Tertiles, if shown, are descriptive of this sample and are **not** policy thresholds.

---

## 4. Implementation

| Piece | Location |
|-------|----------|
| Brief | this document |
| Analysis | `decision_support/factor_ecology.rs` |
| Binary | `csp005_factor_ecology` |
| Runner | `./run_csp005_factor_ecology.sh` |
| Artifacts | `product_validation/CS-P-005_factor_ecology_v0.1/` |

Engine version remains **`unfrozen-dev`**. No real capital.
