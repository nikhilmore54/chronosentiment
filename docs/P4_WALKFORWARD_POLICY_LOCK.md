# P4 Walk-Forward Policy Lock

**Locked:** 2026-09-11  
**Status:** FROZEN — no subsequent modification permitted  
**Authority:** P4 Optimal Trade Discovery, authorized 2026-09-11

---

## Temporal boundary

| Period | Dates | Role |
|---|---|---|
| Development set | 2026-08-20, 2026-08-21 | Policy discovery and boundary computation |
| Walk-forward period | 2026-08-24 onward | Prospective evaluation — outcomes not yet known at lock time |

All policy parameters below were computed exclusively from the Aug 20–21 development set.  
No Aug 24+ outcome data was observed before this document was written.

---

## Track A — Daily walk-forward policy

### Policy definition

**Policy name:** SHORT Watch target_rate Q2  
**Policy version:** v1  
**Locked:** 2026-09-11

**Selection rule:**
- `direction` == SHORT
- `action` == Watch
- `target_rate` >= 0.314607 AND `target_rate` < 0.327778

### Boundary derivation

Computed from the 61 SHORT Watch decisions in the Aug 20–21 development set.

| Quartile | N | target_rate range |
|---|---|---|
| Q1 (lowest) | 15 | < 0.314607 |
| **Q2 (policy)** | **14** | **>= 0.314607 AND < 0.327778** |
| Q3 | 15 | >= 0.327778 AND < 0.345900 |
| Q4 (highest) | 17 | >= 0.345900 |

**Q2 lower boundary (frozen):** 0.314607  
**Q2 upper boundary (frozen):** 0.327778

### Development set performance (in-sample, for reference only)

From Sprint 3 analysis:
- N=15 (all), median +1.4%, N=8 (gap-through excluded), median +0.2%, win rate 75.0%
- This is a weak signal. Walk-forward result is the only valid evaluation.

### Walk-forward evaluation procedure

1. For each cohort from Aug 24 onward, identify decisions where:
   - `direction` == SHORT
   - `action` == Watch
   - `target_rate` >= 0.314607 AND `target_rate` < 0.327778
2. Record the decision_id and cohort_date at the time of decision (before outcome is known)
3. When the observation status becomes COMPLETE, record `realized_return`
4. Compare against the baseline: all SHORT Watch decisions in the same cohort period

**No policy adjustment is permitted after this document is written.**

---

## Track B — Intraday walk-forward candidates

Three candidates locked from the Aug 20–21 intraday discovery (Track B).  
These will be evaluated against Aug 24+ cohorts using 5m intraday data.

### Candidate 1 — PRIMARY

**Policy name:** SHORT Watch target_rate Q3, H60  
**Horizon:** H60 (12 × 5m bars = 60 minutes after decision snapshot)

**Selection rule:**
- `direction` == SHORT
- `action` == Watch
- `target_rate` >= 0.327778 AND `target_rate` < 0.345900

**Q3 boundaries (frozen from Aug 20–21 dev set):**  
- Lower: 0.327778 (= Q2 upper boundary)  
- Upper: 0.345900 (= Q3 upper boundary, value at index 3*61//4 = 45 → 0.3459)

**Development set performance (in-sample):**
- N=15, H60 mean +0.39%, median +0.41%, win rate 93.3%
- Extreme move rate: 0.0% (no gap-through artifact at intraday horizons)

### Candidate 2 — STRONG

**Policy name:** SHORT Watch Exact, H300  
**Horizon:** H300 (60 × 5m bars = 300 minutes / ~5 hours after decision snapshot)

**Selection rule:**
- `direction` == SHORT
- `action` == Watch
- `degradation_level` == Exact

**Development set performance (in-sample):**
- N=21, H300 mean +0.23%, median +0.26%, win rate 76.2%
- Extreme move rate: 0.0%

### Candidate 3 — BROAD

**Policy name:** SHORT Watch sample 51–150, H15  
**Horizon:** H15 (3 × 5m bars = 15 minutes after decision snapshot)

**Selection rule:**
- `direction` == SHORT
- `action` == Watch
- `sample_size` >= 51 AND `sample_size` <= 150

**Development set performance (in-sample):**
- N=29, H15 mean +0.15%, median +0.11%, win rate 75.9%
- Extreme move rate: 0.0%

---

## Baseline for walk-forward comparison

All walk-forward results will be compared against:

| Baseline | Definition |
|---|---|
| All SHORT Watch | All SHORT Watch decisions in the same cohort period |
| All Watch | All Watch decisions in the same cohort period |
| All COMPLETE | All decisions in the same cohort period |

---

## Integrity constraints

1. **No retroactive adjustment.** Policy boundaries cannot be changed after this document is written.
2. **No peeking.** Aug 24+ outcomes must not be used to select or modify any policy parameter.
3. **No cherry-picking.** All qualifying decisions in the walk-forward period must be included, not just favorable ones.
4. **Separate accounting.** Track A (daily) and Track B (intraday) results must be reported separately.
5. **Gap-through tracking.** Daily results must separately report gap-through outcomes (|return| > 50%).
6. **TIME-009 frozen.** TIME-009 observations remain a scientific reference only — not a tuning target.

---

## Next action

When Aug 24+ cohorts begin completing their horizons:
1. Run the walk-forward evaluation script against the locked policies above
2. Report results without modification
3. Compare against baselines
4. If a policy fails: record the failure, do not adjust the policy retroactively
5. If a policy succeeds: carry forward to Sprint 4 (Change One Thing)

**The walk-forward result is the only valid evaluation of these policies.**  
In-sample development set performance is reference only.