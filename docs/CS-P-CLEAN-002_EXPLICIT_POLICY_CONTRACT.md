# CS-P-CLEAN-002 — Explicit DecisionPolicy contract (PR-2)

**Document type:** Architecture hardening  
**Status:** Complete — CS-P-006-A is the next programme (contract only)  
**Date:** 2026-08-14  
**Parent:** CS-P-AUDIT-001, CS-P-CLEAN-001  
**Does not:** invent a trading strategy, run a backtest, regenerate B3/B4, reopen G-GATE, freeze v1.0  

`.cursor/rules/chronosentiment-core.mdc`: same inputs + same policy version → same decision; no invented methodology.

---

## Success criterion

> ChronoSentiment cannot produce a product `TradingDecision` unless an explicit policy is supplied, and the baseline policy remains reproducible only when explicitly selected.

```text
decide_at(T, instrument, engine_version, policy)
       │
       ▼
TradingDecision { policy_name, action, identity, … }
```

`BaselineTrendMappingPolicy` (`baseline.trend_mapping.v0`) is a **historical/product fixture**, not the ChronoSentiment strategy.

---

## Contract

* `decide_from_inputs(inputs, policy)` — no one-argument default
* `DecideAt::decide_at(..., policy: &dyn DecisionPolicy)`
* Replay, forward, and backtest all thread the policy
* `TradingDecision.policy_name` is required and hashed into `decision_id` / `content_hash`
* Schema `csp004.decision.1` (prior unfrozen-dev IDs are not comparable across this change; do not regenerate CS-P-002-R1)
* Fabricated assessment scores (0.82 / 0.73) are not copied onto product evidence and are stripped from identity
* Decision confidence remains `UNAVAILABLE`
* Outcomes still cannot enter `decide_at`

Identity change is expected: `policy_name` is now a first-class identity field. Existing CS-P-003 journal rows remain append-only; new ticks use the new identity. Do not treat that as a strategy result.

---

## Not done

* CS-P-006-B research protocol freeze (split dates)
* CS-P-006-C Coralys search / evolution
* Repairing `StrategyEngine` SHORT omission (B4 provenance)
* Promoting the baseline fixture
