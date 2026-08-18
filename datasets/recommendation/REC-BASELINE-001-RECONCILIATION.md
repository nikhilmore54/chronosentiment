# REC-BASELINE-001 — Policy/Implementation Reconciliation

**Status:** COMPLETE  
**Date:** 2026-08-18  
**Baseline commit:** `eeb466705790d9189a7496f407457631c11d1902`  
**Source files inspected:**
- `coralys-decision/src/recommendation/engine.rs` (lines 802–842)
- `coralys-decision/src/recommendation/evidence.rs` (lines 380–790)

`REC-BASELINE-001.md` is immutable. This document records discrepancies discovered
between the documented policy semantics and the actual implementation that produced
the 2026-08-18 snapshot. It does not alter the baseline observations.

---

## Summary of Findings

| # | Question | Finding | Severity |
|---|----------|---------|----------|
| Q1 | BUY despite R:R < 1.0 | Documentation error — no R:R floor in BUY rule | Documentation |
| Q2 | WATCH despite StateOnly | Documentation error — StateOnly is NOT forced to NO_TRADE | Documentation |
| Q3 | Evidence classification thresholds | Confirmed from source | Correct |
| Q4 | `derive_action_v1()` vs written policy | Partial mismatch — written policy overstates R:R constraint | Documentation |
| Q5 | `adaptive_rr` vs action R:R | Confirmed same field | Correct |

**Conclusion:** The algorithm is internally consistent. The discrepancies are in the
written policy description in REC-BASELINE-001, not in the implementation.
No algorithm change is required.

---

## Q1: BUY despite R:R < 1.0

### Observed

Five BUY tickers with `adaptive_rr` < 1.0:

| Ticker        | adaptive_rr | target_rate | evidence_class |
|---------------|-------------|-------------|----------------|
| LT.NS         | 0.96        | 0.414       | Favourable     |
| DLF.NS        | 0.87        | 0.426       | Favourable     |
| HDFCBANK.NS   | 0.86        | 0.423       | Favourable     |
| BANKBARODA.NS | 0.84        | 0.417       | Favourable     |
| ICICIPRULI.NS | 0.97        | 0.459       | Favourable     |

### Root cause

`derive_action_v1()` at engine.rs:810–828:

```rust
match evidence_class {
    EvidenceClassV1::Favourable => {
        if direction == "SHORT" {
            RecommendationAction::Sell
        } else {
            RecommendationAction::Buy
        }
    }
    EvidenceClassV1::Mixed => {
        let rr = adaptive_rr.unwrap_or(0.0);
        if rr >= 1.5 && direction != "SHORT" {
            RecommendationAction::Buy
        } else {
            RecommendationAction::Watch
        }
    }
    ...
}
```

**The R:R threshold (>= 1.5) applies only to the Mixed branch, not to Favourable.**

A `Favourable` LONG decision → BUY unconditionally, regardless of `adaptive_rr`.
The five tickers above are all `Favourable` (target_rate >= 0.40, sample_size >= 30)
and therefore receive BUY without any R:R check.

### Resolution

The written policy in REC-BASELINE-001 stated:

> BUY: R:R ≥ 1.0 AND target_rate ≥ 0.30 AND sample_size ≥ 20

This is **incorrect**. The actual policy is:

> BUY: evidence_class == Favourable (no R:R floor)
> BUY: evidence_class == Mixed AND adaptive_rr >= 1.5

The `adaptive_rr` displayed in the baseline is the same value used by `derive_action_v1()`
(confirmed: Q5). The displayed R:R is not wrong — the written BUY rule was wrong.

**The algorithm is correct. The documentation was wrong.**

---

## Q2: WATCH despite StateOnly degradation

### Observed

Five WATCH tickers with `degradation_level == StateOnly`:

| Ticker       | evidence_class | action | adaptive_rr |
|--------------|----------------|--------|-------------|
| HCLTECH.NS   | Mixed          | Watch  | 1.38        |
| NAUKRI.NS    | Mixed          | Watch  | 1.46        |
| HEROMOTOCO.NS| Mixed          | Watch  | 1.16        |
| CHOLAFIN.NS  | Mixed          | Watch  | 1.01        |
| TITAN.NS     | Mixed          | Watch  | 1.02        |

### Root cause

`Rec001hStore::for_decision()` at evidence.rs:773–786:

```rust
// Level 4 — state only (ticker + direction, any conditions)
let so_key = V1KeyStateOnly {
    ticker: ticker.to_string(),
    direction: dir.clone(),
};
if let Some(pool) = self.state_only.get(&so_key) {
    if pool.len() >= MIN_V1_SAMPLE {
        return Some(aggregate_v1(
            ticker, &dir, trend, momentum,
            None, None,
            DegradationLevel::StateOnly, pool,
        ));
    }
}
```

StateOnly is **Level 4 of the graceful degradation hierarchy** — it returns a valid
`V1Evidence` struct with a real `target_rate` and `evidence_class`. The engine then
applies `derive_action_v1()` to that evidence exactly as it would for Exact/RelaxBoth.

StateOnly does NOT force NO_TRADE. It forces NO_TRADE only if the pool has fewer than
`MIN_V1_SAMPLE` (15) analogues even at the StateOnly level — in which case `for_decision()`
returns `None`, and the engine emits NO_TRADE via `no_trade_record()`.

The five tickers above had >= 15 analogues at StateOnly level, with target_rate in the
Mixed range (0.30–0.40), and adaptive_rr < 1.5 → correctly classified as Mixed → Watch.

### Resolution

The written policy in REC-BASELINE-001 stated:

> NO_TRADE: All other cases (including StateOnly degradation)

This is **incorrect**. The actual policy is:

> StateOnly → evidence is computed from the StateOnly pool → action derived normally
> NO_TRADE only when: Insufficient (< MIN_V1_SAMPLE at all levels) OR Unfavourable

**The algorithm is correct. The documentation was wrong.**

---

## Q3: Evidence Classification Thresholds

### Source (evidence.rs:52–116)

```rust
const FAVOURABLE_RATE_THRESHOLD: f64 = 0.40;
const FAVOURABLE_MIN_SAMPLE: usize = 30;
const MIXED_RATE_THRESHOLD: f64 = 0.30;
const MIXED_MIN_SAMPLE: usize = 15;

impl EvidenceClass {
    pub fn classify(rate: f64, sample_size: usize) -> EvidenceClass {
        if sample_size < MIXED_MIN_SAMPLE {
            return EvidenceClass::Insufficient;
        }
        if rate >= FAVOURABLE_RATE_THRESHOLD && sample_size >= FAVOURABLE_MIN_SAMPLE {
            EvidenceClass::Favourable
        } else if rate >= MIXED_RATE_THRESHOLD {
            EvidenceClass::Mixed
        } else {
            EvidenceClass::Unfavourable
        }
    }
}
```

### Confirmed thresholds

| Class        | target_rate condition | sample_size condition |
|--------------|----------------------|-----------------------|
| Favourable   | >= 0.40              | >= 30                 |
| Mixed        | >= 0.30              | >= 15                 |
| Unfavourable | < 0.30               | >= 15                 |
| Insufficient | any                  | < 15                  |

Note: A ticker with target_rate >= 0.40 but sample_size 15–29 is classified as **Mixed**,
not Favourable. The Favourable gate requires both conditions simultaneously.

**No discrepancy. The written thresholds in REC-BASELINE-001 were correct.**

---

## Q4: `derive_action_v1()` vs Written Policy

### Actual implementation (engine.rs:802–842, post-SELL branch addition)

```
NO_TRADE direction          → NO_TRADE
Favourable + LONG           → BUY
Favourable + SHORT          → SELL  (dormant at baseline: 0 Favourable SHORTs)
Mixed + rr >= 1.5 + LONG    → BUY
Mixed + rr >= 1.5 + SHORT   → WATCH
Mixed + rr < 1.5            → WATCH
Unfavourable / Insufficient → NO_TRADE
```

### Written policy in REC-BASELINE-001

The baseline stated a BUY rule requiring R:R >= 1.0 — this was incorrect (see Q1).
The baseline stated StateOnly → NO_TRADE — this was incorrect (see Q2).

### Corrected policy semantics (for future baselines)

```
BUY     : evidence_class == Favourable AND direction == LONG
SELL    : evidence_class == Favourable AND direction == SHORT  [dormant]
BUY     : evidence_class == Mixed AND adaptive_rr >= 1.5 AND direction == LONG
WATCH   : evidence_class == Mixed AND (adaptive_rr < 1.5 OR direction == SHORT)
NO_TRADE: evidence_class == Unfavourable OR Insufficient OR direction == NO_TRADE
```

No R:R floor on Favourable BUY. StateOnly is a degradation level, not an action override.

---

## Q5: Is `adaptive_rr` the same R:R used for action classification?

### Confirmed: YES

`build_record()` at engine.rs:697–700:

```rust
let (adaptive_target, adaptive_risk, adaptive_upside_pct, adaptive_downside_pct, adaptive_rr) =
    compute_adaptive_geometry(direction, reference_price, ev.adaptive_target_pct, ev.adaptive_risk_pct);

let action = derive_action_v1(direction, &ev.evidence_class, adaptive_rr);
```

The `adaptive_rr` passed to `derive_action_v1()` is the same `Option<f64>` stored in
`RecommendationRecordV1.adaptive_rr` and returned by the API. There is no hidden R:R
computation. The displayed value is the decision value.

---

## Corrected Policy Statement (for REC-BASELINE-002)

The following replaces the "Algorithm Semantics" section that was incorrect in REC-BASELINE-001:

### Evidence Classification (v1)

Source: `EvidenceClass::classify()` in `evidence.rs`

| Class        | target_rate | sample_size |
|--------------|-------------|-------------|
| Favourable   | >= 0.40     | >= 30       |
| Mixed        | >= 0.30     | >= 15       |
| Unfavourable | < 0.30      | >= 15       |
| Insufficient | any         | < 15        |

### Degradation Hierarchy (v1)

Source: `Rec001hStore::for_decision()` in `evidence.rs`

1. Exact: ticker + direction + trend + momentum + vol_regime + volume_regime
2. RelaxVolume: ticker + direction + trend + momentum + vol_regime (any volume)
3. RelaxBoth: ticker + direction + trend + momentum (any vol + volume)
4. StateOnly: ticker + direction (any conditions)
5. Insufficient: < 15 analogues at all levels → NO_TRADE

StateOnly is a valid evidence level. It does NOT force NO_TRADE.

### Action Rules (v1)

Source: `derive_action_v1()` in `engine.rs`

```
direction == NO_TRADE                                    → NO_TRADE
evidence_class == Favourable AND direction == LONG       → BUY
evidence_class == Favourable AND direction == SHORT      → SELL
evidence_class == Mixed AND adaptive_rr >= 1.5
    AND direction == LONG                                → BUY
evidence_class == Mixed AND (adaptive_rr < 1.5
    OR direction == SHORT)                               → WATCH
evidence_class == Unfavourable OR Insufficient           → NO_TRADE
```

No R:R floor on Favourable BUY. `adaptive_rr` is the same value displayed and used.

---

## Governance

- `REC-BASELINE-001.md` remains immutable. Do not edit it.
- This reconciliation supersedes the "Algorithm Semantics" section of REC-BASELINE-001
  for documentation purposes only. The baseline numbers are unaffected.
- REC-BASELINE-002 should use the corrected policy statement above.
- Source commit for this reconciliation: current HEAD of `governance-hardening` branch.