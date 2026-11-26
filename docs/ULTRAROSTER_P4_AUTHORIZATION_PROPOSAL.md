# ULTRAROSTER P4 — Authorization Proposal
## Optimal Trade Discovery

**Date:** 2026-09-11
**Status:** AUTHORIZED — 2026-09-11
**Prerequisite:** v0.3 complete ✓, TIME-009 frozen ✓, explicit authorization received ✓

---

## Three independent tracks

| Track | Purpose | State |
|---|---|---|
| TIME-009 | Frozen scientific reference | FROZEN |
| Decision Cockpit | Interface for decisions/results | v0.3 complete |
| Optimal Trade Discovery | Find a better-performing policy from historical data | Ready to start on authorization |

The Cockpit does not need external validation before pursuing the trading objective. External validation remains appropriate for commercial product validation — it is a separate concern.

---

## Objective

> **Given only information available at the time, what trading policy would have produced the best risk-adjusted outcome?**

Use the historical data already available to discover and maximize the quality/returns of a trading policy through rapid product-development iteration.

This is not: "Prove the current algorithm works."  
This is not: "Optimize TIME-009 against its own completed observations."

---

## Phase structure

### Phase 1 — Inventory the information set

Inspect exactly what exists at the moment each historical decision was made. Classify every field:

- **Allowed** — genuinely available before the decision
- **Forbidden** — generated from future prices/outcomes
- **Derived** — calculable exclusively from information available at decision time

This establishes the legitimate search space.

### Phase 2 — Search aggressively

Try many candidate policies on the historical development set:
- Direction combinations (LONG only, SHORT only, both)
- Evidence thresholds
- Analogue quality thresholds
- Target/risk/horizon variations
- Ranking functions
- Selection percentages
- Feature combinations

Potentially thousands of candidates. Rank by composite objective, not raw return.

### Phase 3 — Find what drives returns

Ask: **What characteristics distinguish the best historical decisions from the worst?**

Let the data reveal the structure rather than imposing it.

### Phase 4 — Walk-forward evaluation

Lock the best candidate policy. Move it forward through an unseen historical period. If it survives, it is a prospective candidate. If it collapses, that is an excellent failure — learn and develop the next policy.

```
Historical development set
        │
        ├── Policy 1 ... Policy N
                 │
                 ▼
          Select finalists
                 │
                 ▼
        Locked walk-forward test
                 │
                 ▼
           Prospective test
                 │
                 ▼
        (only then: real capital)
```

---

## Optimization objective

Composite — not maximum return:

- Return
- Drawdown
- Win rate
- Downside
- Trade frequency
- Consistency

**Best usable trade policy, not highest backtest number.** Guards against pathological high-turnover/high-leverage strategies that win the backtest but are unusable as a product.

---

## Six-sprint roadmap

### Sprint 1 — Rank
Turn existing decisions into ranked opportunities using existing evidence signals.  
**Goal:** determine whether selection improves returns.

### Sprint 2 — Measure
Automatically compare top 5% / top 10% / top 20% / middle / bottom against full universe.  
**Goal:** discover whether the ranking has value.

### Sprint 3 — Failure analysis
For every losing/worst-performing decision: why?  
Aggregate failure modes. Let failures create the taxonomy — do not prescribe it in advance.  
**Goal:** find the dominant source of poor returns.

### Sprint 4 — Change one thing
Modify only the most promising failure mechanism. Deploy as Policy v2.  
**Goal:** targeted intervention in the finalist/controlled phase.

### Sprint 5 — A/B
Run Policy v1 vs Policy v2 on fresh decisions.  
**Goal:** genuine product iteration with measurable comparison.

### Sprint 6 — Repeat
v3 → v4 → v5 until meaningful improvement or evidence that the approach isn't working.

*Note: Sprint 1–3 are the discovery phase (broad exploration allowed). Sprint 4–5 are the finalist/controlled phase. Sprint 6 repeats the cycle.*

---

## Constraints

1. **TIME-009 is not touched.** The 300 completed decisions are observation material, not a tuning dataset.
2. **Paper/live-data only** until a policy demonstrates consistent walk-forward improvement. Real capital follows evidence.
3. **Exploration broad, validation strict.** Broad search on development set is fine. Walk-forward boundary is inviolable.
4. **Complete record.** Every policy version, its rationale, and its outcomes are logged before the next version is deployed.
5. **No hindsight leakage.** For every historical decision, the policy must never see future information when making the decision.
6. **Composite objective.** Rank policies on return + drawdown + win rate + downside + frequency + consistency. Never on a single metric.

---

## Authorization gate

This proposal does not open v0.4 scope.

v0.4 opens only when:
- [ ] Explicit authorization to begin historical Optimal Trade Discovery is given

When authorization is given: skip further design. Go straight to data. First action is to inventory the actual decision-time fields in the existing ChronoSentiment artifacts and build the candidate policy search space from them.